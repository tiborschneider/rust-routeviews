//! # Convenient wrapper around [BGPStream](https://bgpstream.caida.org).
//!
//! Library that performs a [`Query`] and yields the results as an iterator. Look at [`Query`] as
//! the main entry point of using this library.
//!
//! The following example reads all updates from the AMSIX RouteView within one hour and prints the
//! time of each update.
//!
//! ```
//! use bgpstream::*;
//! use time::macros::datetime;
//!
//! fn main() {
//!     let stream = Query::new()
//!         .collector(stream::Collector::RouteView(stream::RouteView::Amsix))
//!         .record_type(stream::RecordType::Updates)
//!         .interval(stream::FilterInterval::Interval {
//!             start: datetime!(2023-11-08 09:55 UTC).into(),
//!             stop: datetime!(2023-11-08 10:55 UTC).into(),
//!         })
//!         .cache("/tmp/bgpstream_cache")
//!         .run()
//!         .unwrap();
//!
//!     for element in stream {
//!         let element = element.unwrap();
//!         println!("{:?}", element.time.to_hms());
//!     }
//! }
//! ```

pub mod element;
pub mod record;
pub mod stream;

pub use stream::Query;

use std::{
    ffi::{CString, IntoStringError, NulError},
    net::IpAddr,
    slice,
};

use ipnet::{IpNet, PrefixLenError};
use libbgpstream_sys::{
    bgpstream_addr_version_t_BGPSTREAM_ADDR_VERSION_IPV4 as ADDR_VERSION_IPV4,
    bgpstream_addr_version_t_BGPSTREAM_ADDR_VERSION_IPV6 as ADDR_VERSION_IPV6,
    union_bgpstream_ip_addr_t, union_bgpstream_pfx_t,
};
use thiserror::Error;
use time::error::ComponentRange;

fn array_to_string(array: &[i8]) -> Result<String, IntoStringError> {
    let s: &[u8] = unsafe { slice::from_raw_parts(array.as_ptr() as *const u8, array.len()) };

    let null_pos = s.iter().position(|x| *x == 0).unwrap_or(s.len());
    let s = CString::new(&s[..null_pos]).expect("already checked");
    s.into_string()
}

unsafe fn parse_bgpstream_ip(ip: union_bgpstream_ip_addr_t) -> Result<IpAddr, BgpStreamError> {
    let version = ip.__bindgen_anon_1.__bindgen_anon_1.version;
    match version {
        ADDR_VERSION_IPV4 => Ok(IpAddr::V4(ip.bs_ipv4.addr.s_addr.into())),
        ADDR_VERSION_IPV6 => Ok(IpAddr::V6(ip.bs_ipv6.addr.__in6_u.__u6_addr8.into())),
        _ => return Err(BgpStreamError::InvalidIpAddress),
    }
}

unsafe fn parse_bgpstream_prefix(prefix: union_bgpstream_pfx_t) -> Result<IpNet, BgpStreamError> {
    let prefix_len = prefix
        .__bindgen_anon_1
        .__bindgen_anon_1
        .__bindgen_anon_1
        .mask_len;
    let ip = parse_bgpstream_ip(prefix.__bindgen_anon_1.address)?;
    Ok(IpNet::new(ip, prefix_len)?)
}

#[derive(Debug, Error)]
pub enum BgpStreamError {
    #[error("Cannot create the BGP stream object")]
    Create,
    #[error("Cannot start the BGP stream")]
    Start,
    #[error("Error adding a filter")]
    AddFilter,
    #[error("Error adding the recent interval")]
    AddRecentInterval,
    #[error("Error adding the interval")]
    AddInterval,
    #[error("Error adding the RIB period")]
    AddRibPeriod,
    #[error("Error getting the next record")]
    GetNextRecord,
    #[error("The next record computed is a NULL pointer")]
    GetNextRecordNull,
    #[error("The record is corrupted")]
    RecordCorrupted,
    #[error("The record is unsupported")]
    RecordUnsupported,
    #[error("The record source is empty, corrupted, or contains no valid record")]
    RecordSourceEmptyOrCorrupted,
    #[error("Received a record with an unknown status")]
    UnknownRecordStatus,
    #[error("Error getting the next element of a record")]
    GetNextElement,
    #[error("The next element computed is a NULL pointer")]
    GetNextElementNull,
    #[error("Unknown element type")]
    UnknownElementType,
    #[error("Invalid IP address")]
    InvalidIpAddress,
    #[error("Element was detached without fetching the requested data.")]
    ElementIsDetached,
    #[error("Unknown peer state recieved")]
    UnknownPeerState,
    #[error("Unknown origin type received")]
    UnknownOriginType,
    #[error("Interface with name {0} does not exist")]
    InterfaceNotFound(String),
    #[error("Interface option with name {0} does not exist")]
    InterfaceOptionNotFound(String),
    #[error("Failed to set the interface option")]
    SetInterfaceOption,
    #[error("Invalid prefix mask length: {0}")]
    InvalidMaskLen(#[from] PrefixLenError),
    #[error("A provided string contains a NULL character!")]
    StringContainsNull(#[from] NulError),
    #[error("Error converting from a timestamp into date and time: {0}")]
    Timestamp(#[from] ComponentRange),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_query() {
        Query::new()
            .collector("route-views.amsix")
            .record_type(stream::RecordType::Updates)
            .interval(stream::FilterInterval::Since {
                amount: 1,
                unit: stream::TimeUnit::Hours,
                live: false,
            })
            .run()
            .unwrap();
    }
}
