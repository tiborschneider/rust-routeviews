use std::{
    fmt::Display,
    net::IpAddr,
    ptr::{addr_of, null_mut},
    time::Duration,
};

use ipnet::IpNet;
use itertools::Itertools;
use libbgpstream_sys::{
    bgpstream_as_path_get_next_seg, bgpstream_as_path_iter, bgpstream_as_path_iter_reset,
    bgpstream_as_path_seg_t,
    bgpstream_as_path_seg_type_t_BGPSTREAM_AS_PATH_SEG_ASN as AS_PATH_SEG_ASN,
    bgpstream_community_set_get,
    bgpstream_elem_origin_type_t_BGPSTREAM_ELEM_BGP_UPDATE_ORIGIN_EGP as ORIGIN_EGP,
    bgpstream_elem_origin_type_t_BGPSTREAM_ELEM_BGP_UPDATE_ORIGIN_IGP as ORIGIN_IGP,
    bgpstream_elem_origin_type_t_BGPSTREAM_ELEM_BGP_UPDATE_ORIGIN_INCOMPLETE as ORIGIN_INCOMPLETE,
    bgpstream_elem_peerstate_t_BGPSTREAM_ELEM_PEERSTATE_ACTIVE as ELEM_PEERSTATE_ACTIVE,
    bgpstream_elem_peerstate_t_BGPSTREAM_ELEM_PEERSTATE_CLEARING as ELEM_PEERSTATE_CLEARING,
    bgpstream_elem_peerstate_t_BGPSTREAM_ELEM_PEERSTATE_CONNECT as ELEM_PEERSTATE_CONNECT,
    bgpstream_elem_peerstate_t_BGPSTREAM_ELEM_PEERSTATE_DELETED as ELEM_PEERSTATE_DELETED,
    bgpstream_elem_peerstate_t_BGPSTREAM_ELEM_PEERSTATE_ESTABLISHED as ELEM_PEERSTATE_ESTABLISHED,
    bgpstream_elem_peerstate_t_BGPSTREAM_ELEM_PEERSTATE_IDLE as ELEM_PEERSTATE_IDLE,
    bgpstream_elem_peerstate_t_BGPSTREAM_ELEM_PEERSTATE_OPENCONFIRM as ELEM_PEERSTATE_OPENCONFIRM,
    bgpstream_elem_peerstate_t_BGPSTREAM_ELEM_PEERSTATE_OPENSENT as ELEM_PEERSTATE_OPENSENT,
    bgpstream_elem_peerstate_t_BGPSTREAM_ELEM_PEERSTATE_UNKNOWN as ELEM_PEERSTATE_UNKNOWN,
    bgpstream_elem_t,
    bgpstream_elem_type_t_BGPSTREAM_ELEM_TYPE_ANNOUNCEMENT as ELEM_TYPE_ANNOUNCEMENT,
    bgpstream_elem_type_t_BGPSTREAM_ELEM_TYPE_PEERSTATE as ELEM_TYPE_PEERSTATE,
    bgpstream_elem_type_t_BGPSTREAM_ELEM_TYPE_RIB as ELEM_TYPE_RIB,
    bgpstream_elem_type_t_BGPSTREAM_ELEM_TYPE_WITHDRAWAL as ELEM_TYPE_WITHDRAWAL,
    bgpstream_record_get_next_elem,
};
use time::OffsetDateTime;

use crate::{parse_bgpstream_ip, parse_bgpstream_prefix, record::Record, BgpStreamError};

#[derive(Debug, Clone)]
pub struct Element {
    pub time: OffsetDateTime,
    pub peer_ip: IpAddr,
    pub peer_asn: u32,
    pub e: ElementType,
}

impl Element {
    pub(crate) fn new(record: &mut Record<'_>) -> Result<Option<Element>, BgpStreamError> {
        unsafe {
            let mut p_elem = null_mut::<bgpstream_elem_t>();
            let p_p_elem: *mut *mut bgpstream_elem_t = &mut p_elem;
            let res = bgpstream_record_get_next_elem(record.p_record, p_p_elem);

            match res {
                1 => {}
                0 => return Ok(None),
                _ => return Err(BgpStreamError::GetNextElement),
            }

            // check that p_record is non-null
            if p_elem.is_null() {
                return Err(BgpStreamError::GetNextElementNull);
            };

            let elem = &*p_elem;

            let time = if elem.orig_time_sec == 0 {
                record.time
            } else {
                let secs = elem.orig_time_sec;
                let micros = elem.orig_time_usec;
                OffsetDateTime::from_unix_timestamp(secs as i64)?
                    + Duration::from_micros(micros as u64)
            };

            let peer_ip = parse_bgpstream_ip(elem.peer_ip)?;
            let peer_asn = elem.peer_asn;

            let e = match elem.type_ {
                ELEM_TYPE_ANNOUNCEMENT | ELEM_TYPE_RIB => {
                    let update = Update {
                        prefix: parse_bgpstream_prefix(elem.prefix)?,
                        next_hop: parse_bgpstream_ip(elem.nexthop)?,
                        as_path: extract_as_path(p_elem),
                        communities: extract_communities(p_elem),
                        origin_type: if elem.has_origin != 0 {
                            Some(elem.origin.try_into()?)
                        } else {
                            None
                        },
                        med: if elem.has_med != 0 {
                            Some(elem.med)
                        } else {
                            None
                        },
                        local_pref: if elem.has_local_pref != 0 {
                            Some(elem.local_pref)
                        } else {
                            None
                        },
                    };

                    if elem.type_ == ELEM_TYPE_ANNOUNCEMENT {
                        ElementType::Announcement(update)
                    } else {
                        ElementType::RIB(update)
                    }
                }
                ELEM_TYPE_PEERSTATE => ElementType::PeerState {
                    from: elem.old_state.try_into()?,
                    to: elem.new_state.try_into()?,
                },
                ELEM_TYPE_WITHDRAWAL => {
                    ElementType::Withdrawal(parse_bgpstream_prefix(elem.prefix)?)
                }
                _ => return Err(BgpStreamError::UnknownElementType),
            };

            Ok(Some(Element {
                time,
                peer_ip,
                peer_asn,
                e,
            }))
        }
    }

    pub fn prefix(&self) -> Option<IpNet> {
        match &self.e {
            ElementType::RIB(u) | ElementType::Announcement(u) => Some(u.prefix),
            ElementType::Withdrawal(p) => Some(*p),
            ElementType::PeerState { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AsSegment {
    Num(u32),
    Set(Vec<u32>),
}

impl Display for AsSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AsSegment::Num(x) => x.fmt(f),
            AsSegment::Set(list) => write!(f, "[{}]", list.iter().join(", ")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ElementType {
    RIB(Update),
    Announcement(Update),
    Withdrawal(IpNet),
    PeerState { from: PeerState, to: PeerState },
}

#[derive(Debug, Clone)]
pub struct Update {
    pub prefix: IpNet,
    pub next_hop: IpAddr,
    pub as_path: Vec<AsSegment>,
    pub communities: Vec<(u16, u16)>,
    pub origin_type: Option<OriginType>,
    pub med: Option<u32>,
    pub local_pref: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PeerState {
    Idle,
    Connect,
    Active,
    OpenSent,
    OpenConfirm,
    Established,
    Clearing,
    Deleted,
    Unknown,
}

impl TryFrom<u32> for PeerState {
    type Error = BgpStreamError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            ELEM_PEERSTATE_ACTIVE => Ok(Self::Active),
            ELEM_PEERSTATE_CLEARING => Ok(Self::Clearing),
            ELEM_PEERSTATE_CONNECT => Ok(Self::Connect),
            ELEM_PEERSTATE_DELETED => Ok(Self::Deleted),
            ELEM_PEERSTATE_ESTABLISHED => Ok(Self::Established),
            ELEM_PEERSTATE_IDLE => Ok(Self::Idle),
            ELEM_PEERSTATE_OPENCONFIRM => Ok(Self::OpenConfirm),
            ELEM_PEERSTATE_OPENSENT => Ok(Self::OpenSent),
            ELEM_PEERSTATE_UNKNOWN => Ok(Self::Unknown),
            _ => Err(BgpStreamError::UnknownPeerState),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OriginType {
    Igp,
    Egp,
    Incomplete,
}

impl Display for OriginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OriginType::Igp => f.write_str("igp"),
            OriginType::Egp => f.write_str("bgp"),
            OriginType::Incomplete => f.write_str("incomplete"),
        }
    }
}

impl TryFrom<u32> for OriginType {
    type Error = BgpStreamError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            ORIGIN_EGP => Ok(Self::Egp),
            ORIGIN_IGP => Ok(Self::Igp),
            ORIGIN_INCOMPLETE => Ok(Self::Incomplete),
            _ => Err(BgpStreamError::UnknownOriginType),
        }
    }
}

unsafe fn extract_as_path(p_elem: *mut bgpstream_elem_t) -> Vec<AsSegment> {
    let mut iter = bgpstream_as_path_iter { cur_offset: 0 };
    let mut path: Vec<AsSegment> = Vec::new();
    let elem = &*p_elem;
    let iter = &mut iter as *mut bgpstream_as_path_iter;
    // reset the iterator
    bgpstream_as_path_iter_reset(iter);

    loop {
        let seg = bgpstream_as_path_get_next_seg(elem.as_path, iter);
        if seg.is_null() {
            break;
        }
        path.push(parse_as_path_seg(seg));
    }

    path
}

unsafe fn extract_communities(p_elem: *mut bgpstream_elem_t) -> Vec<(u16, u16)> {
    // read the full as path length
    let mut communities = Vec::new();
    let elem = &*p_elem;

    for i in 0.. {
        let comm = bgpstream_community_set_get(elem.communities, i);
        if comm.is_null() {
            break;
        }
        let comm = &*comm;
        let asn = comm.__bindgen_anon_1.__bindgen_anon_1.asn;
        let value = comm.__bindgen_anon_1.__bindgen_anon_1.value;
        communities.push((asn, value))
    }

    communities
}

unsafe fn parse_as_path_seg(seg: *mut bgpstream_as_path_seg_t) -> AsSegment {
    let seg = &*seg;
    if *seg.__bindgen_anon_1.type_.as_ref() == AS_PATH_SEG_ASN as u8 {
        // single AS number
        AsSegment::Num(seg.__bindgen_anon_1.asn.as_ref().asn)
    } else {
        // AS set
        let set = seg.__bindgen_anon_1.set.as_ref();
        let len = set.asn_cnt as isize;
        let slice_base = addr_of!(set.asn) as *const u32;
        let mut list = Vec::with_capacity(len as usize);
        for i in 0..len {
            list.push(std::ptr::read_unaligned(slice_base.offset(i)));
        }
        AsSegment::Set(list.to_vec())
    }
}
