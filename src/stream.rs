use std::{
    ffi::{c_char, CString},
    fmt::Display,
    ptr::NonNull,
};

use libbgpstream_sys::{
    bgpstream_add_filter, bgpstream_add_interval_filter, bgpstream_add_recent_interval_filter,
    bgpstream_add_rib_period_filter, bgpstream_create, bgpstream_destroy, bgpstream_filter_type_t,
    bgpstream_filter_type_t_BGPSTREAM_FILTER_TYPE_COLLECTOR as FILTER_TYPE_COLLECTOR,
    bgpstream_filter_type_t_BGPSTREAM_FILTER_TYPE_ELEM_ASPATH as FILTER_TYPE_ELEM_ASPATH,
    bgpstream_filter_type_t_BGPSTREAM_FILTER_TYPE_ELEM_COMMUNITY as FILTER_TYPE_ELEM_COMMUNITY,
    bgpstream_filter_type_t_BGPSTREAM_FILTER_TYPE_ELEM_IP_VERSION as FILTER_TYPE_ELEM_IP_VERSION,
    bgpstream_filter_type_t_BGPSTREAM_FILTER_TYPE_ELEM_NOT_PEER_ASN as FILTER_TYPE_ELEM_NOT_PEER_ASN,
    bgpstream_filter_type_t_BGPSTREAM_FILTER_TYPE_ELEM_ORIGIN_ASN as FILTER_TYPE_ELEM_ORIGIN_ASN,
    bgpstream_filter_type_t_BGPSTREAM_FILTER_TYPE_ELEM_PEER_ASN as FILTER_TYPE_ELEM_PEER_ASN,
    bgpstream_filter_type_t_BGPSTREAM_FILTER_TYPE_ELEM_PREFIX_ANY as FILTER_TYPE_ELEM_PREFIX_ANY,
    bgpstream_filter_type_t_BGPSTREAM_FILTER_TYPE_ELEM_PREFIX_EXACT as FILTER_TYPE_ELEM_PREFIX_EXACT,
    bgpstream_filter_type_t_BGPSTREAM_FILTER_TYPE_ELEM_PREFIX_LESS as FILTER_TYPE_ELEM_PREFIX_LESS,
    bgpstream_filter_type_t_BGPSTREAM_FILTER_TYPE_ELEM_PREFIX_MORE as FILTER_TYPE_ELEM_PREFIX_MORE,
    bgpstream_filter_type_t_BGPSTREAM_FILTER_TYPE_ELEM_TYPE as FILTER_TYPE_ELEM_TYPE,
    bgpstream_filter_type_t_BGPSTREAM_FILTER_TYPE_PROJECT as FILTER_TYPE_PROJECT,
    bgpstream_filter_type_t_BGPSTREAM_FILTER_TYPE_RECORD_TYPE as FILTER_TYPE_RECORD_TYPE,
    bgpstream_get_data_interface_id_by_name, bgpstream_get_data_interface_option_by_name,
    bgpstream_set_data_interface_option, bgpstream_start, bgpstream_t,
};
use time::OffsetDateTime;

use crate::{element::Element, record::Record, BgpStreamError};

#[derive(Default, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum FilterInterval {
    #[default]
    Open,
    /// Only those records that fall within the given interval will be included in the stream.
    /// Setting the stop parameter to `None` will enable live mode and effectively set an endless
    /// interval.
    Interval {
        start: OffsetDateTime,
        stop: Option<OffsetDateTime>,
    },
    /// Time range starting back a certain number of seconds, minutes, hours or days until now. If
    /// `live` is `true`, then the iterator will also get events in the future, once all past events
    /// were fetched (infinite iterator).
    Since {
        amount: usize,
        unit: TimeUnit,
        live: bool,
    },
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
    Days,
}

impl Display for TimeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeUnit::Seconds => f.write_str("s"),
            TimeUnit::Minutes => f.write_str("m"),
            TimeUnit::Hours => f.write_str("h"),
            TimeUnit::Days => f.write_str("d"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum IpVersion {
    IPv4,
    IPv6,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum RecordType {
    /// Individual and more frequent (incremental) updates.
    Updates,
    /// Entire RIBs (less frequent).
    RIBs,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum PrefixMatchType {
    /// Match either the exact prefix, a less specific or a more specific prefix.
    Any,
    /// Match only the exact prefix
    Exact,
    /// Match the exact prefix or a less specific one.
    Less,
    /// Match the exact prefix or a more specific one.
    More,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Project {
    RouteViews,
    RIS,
}

/// Enumeration of all available collectors
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Collector {
    /// Route view with RIBs every 2 hours and updates every 15 minutes
    RouteView(RouteView),
    /// RIPE NCC Routing Information Service with RIBs every 8 hours and updates every 5 minutes
    RipeNcc(RipeNcc),
}

impl Collector {
    fn cstring(&self) -> CString {
        match self {
            Collector::RouteView(rv) => rv.cstring(),
            Collector::RipeNcc(rv) => rv.cstring(),
        }
    }
}

/// Route view with RIBs every 2 hours and updates every 15 minutes. You can find the current state
/// [here](https://bgpstream.caida.org/data#!routeviews). Check the current peering status
/// [here](https://www.routeviews.org/peers/peering-status.html)
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum RouteView {
    /// Route view with URL `amsix.ams.routeviews.org` running FRR located at AMS-IX AM7, Amsterdam,
    /// Netherlands.
    Amsix,
    /// Route view with URL `route-views.bdix.routeviews.org` running FRR located at BDIX, Dhaka,
    /// Bangledesh.
    Bdix,
    /// Route view with URL `route-views.bknix.routeviews.org` running FRR located at BKNIX,
    /// Bangkok, Thailand.
    Bknix,
    /// Route view with URL `route-views.chicago.routeviews.org` running FRR located at Equinix CH1,
    /// Chicago, Illinois.
    Chicago,
    /// Route view with URL `route-views.chile.routeviews.org` running FRR located at NIC.cl
    /// Santiago, Chile.
    Chile,
    /// Route view with URL `route-views.eqix.routeviews.org` running FRR located at Equinix DC,
    /// Ashburn, Virgina.
    Eqix,
    /// Route view with URL `route-views.flix.routeviews.org` running FRR located at FL-IX, Atlanta,
    /// Georgia.
    Flix,
    /// Route view with URL `route-views.fortaleza.routeviews.org` running FRR located at IX.br
    /// (PTT.br), Fortaleza, Brazil.
    Fortaleza,
    /// Route view with URL `route-views.gixa.routeviews.org` running FRR located at GIXA, Ghana,
    /// Africa.
    Gixa,
    /// Route view with URL `route-views.gorex.routeviews.org` running FRR located at GOREX, Guam,
    /// US Territories.
    Gorex,
    /// Route view with URL `pit.scl.routeviews.org` running FRR located at PIT Chile Santiago,
    /// Santiago, Chile.
    Isc,
    // Jinx is disabled, last update was 5 years ago
    // /// Route view with URL `route-views.jinx.routeviews.org` running Quagga located at JINX,
    // /// Johannesburg, South Africa.
    // Jinx,
    /// Route view with URL `route-views.kixp.routeviews.org` running FRR located at KIXP, Nairobi,
    /// Kenya.
    Kixp,
    /// Route view with URL `route-views.linx.routeviews.org` running FRR located at LINX, London,
    /// United Kingdom.
    Linx,
    /// Route view with URL `route-views.mwix.routeviews.org` running FRR located at FD-IX,
    /// Indianapolis, Indiana.
    Mwix,
    /// Route view with URL `route-views.napafrica.routeviews.org` running FRR located at NAPAfrica,
    /// Johannesburg, South Africa.
    Napafrica,
    /// Route view with URL `route-views.nwax.routeviews.org` running FRR located at NWAX, Portland,
    /// Oregon.
    Nwax,
    /// Route view with URL `route-views.ny.routeviews.org` running FRR located at DE-CIX NYC, New
    /// York, USA.
    Ny,
    /// Route view with URL `route-views.perth.routeviews.org` running FRR located at West
    /// Australian Internet Exchange, Perth, Australia.
    Perth,
    /// Route view with URL `route-views.peru.routeviews.org` running FRR located at Peru IX, Lima,
    /// Peru.
    Peru,
    /// Route view with URL `route-views.phoix.routeviews.org` running FRR located at University of
    /// the Philippines, Diliman, Quezon City, Philippines.
    Phoix,
    /// Route view with URL `route-views.rio.routeviews.org` running FRR located at IX.br (PTT.br),
    /// Rio de Janeiro, Brazil.
    Rio,
    /// Route view with URL `route-views.saopaulo.routeviews.org` running Zebra located at SAOPAULO
    /// (PTT Metro, NIC.br), Sao Paulo, Brazil.
    // SaoPaolo is disabled, last update was 2 years ago.
    // Saopaulo,
    // /// Route view with URL `route-views.sfmix.routeviews.org` running FRR located at San Francisco
    // /// Metro IX, San Francisco, California.
    Sfmix,
    /// Route view with URL `route-views.sg.routeviews.org` running Zebra located at Equinix SG1,
    /// Singapore, Singapore.
    Sg,
    /// Route view with URL `route-views.siex.routeviews.org` running FRR located at Sothern Italy
    /// Exchange (SIEX), Rome, Italy.
    // Siex is disabled, last update 2 years ago
    // Siex,
    // /// Route view with URL `route-views.sox.routeviews.org` running Quagga located at Serbia Open
    // /// Exchange, Belgrade, Serbia.
    Soxrs,
    /// Route view with URL `route-views.sydney.routeviews.org` running FRR located at Equinix SYD1,
    /// Sydney, Australia.
    Sydney,
    /// Route view with URL `route-views.telxatl.routeviews.org` running FRR located at TELXATL,
    /// Atlanta, Georgia.
    Telxatl,
    /// Route view with URL `route-views.uaeix.routeviews.org` running FRR located at UAE-IX, Dubai,
    /// United Arab Emirates.
    Uaeix,
    /// Route view with URL `route-views.wide.routeviews.org` running Zebra located at DIXIE
    /// (NSPIXP), Tokyo, Japan.
    Wide,
    /// Route view with URL `route-views2.routeviews.org` running FRR located at University of
    /// Oregon, Eugene Oregon.
    View2,
    /// Route view with URL `route-views2.saopaulo.routeviews.org` running FRR located at SAOPAULO
    /// (PTT Metro, NIC.br), Sao Paulo, Brazil.
    View2SaoPaulo,
    /// Route view with URL `route-views3.routeviews.org` running FRR located at University of
    /// Oregon, Eugene Oregon.
    View3,
    /// Route view with URL `route-views4.routeviews.org` running FRR located at University of
    /// Oregon, Eugene Oregon.
    View4,
    /// Route view with URL `route-views5.routeviews.org` running FRR located at University of
    /// Oregon, Eugene Oregon.
    View5,
    /// Route view with URL `route-views6.routeviews.org` running FRR located at University of
    /// Oregon, Eugene Oregon.
    View6,
}

impl RouteView {
    fn cstring(&self) -> CString {
        match self {
            RouteView::Amsix => CString::new("route-views.amsix").unwrap(),
            RouteView::Bdix => CString::new("route-views.bdix").unwrap(),
            RouteView::Bknix => CString::new("route-views.bknix").unwrap(),
            RouteView::Chicago => CString::new("route-views.chicago").unwrap(),
            RouteView::Chile => CString::new("route-views.chile").unwrap(),
            RouteView::Eqix => CString::new("route-views.eqix").unwrap(),
            RouteView::Flix => CString::new("route-views.flix").unwrap(),
            RouteView::Fortaleza => CString::new("route-views.fortaleza").unwrap(),
            RouteView::Gixa => CString::new("route-views.gixa").unwrap(),
            RouteView::Gorex => CString::new("route-views.gorex").unwrap(),
            RouteView::Isc => CString::new("route-views.isc").unwrap(),
            RouteView::Kixp => CString::new("route-views.kixp").unwrap(),
            RouteView::Linx => CString::new("route-views.linx").unwrap(),
            RouteView::Mwix => CString::new("route-views.mwix").unwrap(),
            RouteView::Napafrica => CString::new("route-views.napafrica").unwrap(),
            RouteView::Nwax => CString::new("route-views.nwax").unwrap(),
            RouteView::Ny => CString::new("route-views.ny").unwrap(),
            RouteView::Perth => CString::new("route-views.perth").unwrap(),
            RouteView::Peru => CString::new("route-views.peru").unwrap(),
            RouteView::Phoix => CString::new("route-views.phoix").unwrap(),
            RouteView::Rio => CString::new("route-views.rio").unwrap(),
            RouteView::Sfmix => CString::new("route-views.sfmix").unwrap(),
            RouteView::Sg => CString::new("route-views.sg").unwrap(),
            RouteView::Soxrs => CString::new("route-views.soxrs").unwrap(),
            RouteView::Sydney => CString::new("route-views.sydney").unwrap(),
            RouteView::Telxatl => CString::new("route-views.telxatl").unwrap(),
            RouteView::Uaeix => CString::new("route-views.uaeix").unwrap(),
            RouteView::Wide => CString::new("route-views.wide").unwrap(),
            RouteView::View2 => CString::new("route-views2").unwrap(),
            RouteView::View2SaoPaulo => CString::new("route-views2saopaulo").unwrap(),
            RouteView::View3 => CString::new("route-views3").unwrap(),
            RouteView::View4 => CString::new("route-views4").unwrap(),
            RouteView::View5 => CString::new("route-views5").unwrap(),
            RouteView::View6 => CString::new("route-views6").unwrap(),
        }
    }
}

/// RIPE NCC Routing Information Service with RIBs every 8 hours and updates every 5 minutes. You
/// can find the current state [here](https://bgpstream.caida.org/data#!ris)
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum RipeNcc {
    /// RIPE RIS collector 00, located at Amsterdam, NL (with type *multihop*, scoped to *global*)
    /// sponsored by RIPE NCC
    Amsterdam,
    /// RIPE RIS collector 01, located at London, GB (with type *IXP*, scoped to *LINX, LONAP*)
    /// sponsored by LINX and LONAP.
    London,
    /// RIPE RIS collector 03, located at Amsterdam, NL (with type *IXP*, scoped to *AMS-IX, NL-IX*)
    /// sponsored by AMS-IX and NL-IX.
    AmsterdamIx,
    /// RIPE RIS collector 04, located at Geneva, CH (with type *IXP*, scoped to *CIXP*) sponsored
    /// by CERN.
    Geneva,
    /// RIPE RIS collector 05, located at Vienna, AT (with type *IXP*, scoped to *VIX*) sponsored by
    /// VIX.
    Vienna,
    /// RIPE RIS collector 06, located at Otemachi, JP (with type *IXP*, scoped to *DIX-IE, JPIX*)
    /// sponsored by RIPE NCC.
    Otemachi,
    /// RIPE RIS collector 07, located at Stockholm, SE (with type *IXP*, scoped to *Netnod*)
    /// sponsored by NETNOD.
    Stockholm,
    /// RIPE RIS collector 10, located at Milan, IT (with type *IXP*, scoped to *MIX*) sponsored by
    /// MIX.
    Milan,
    /// RIPE RIS collector 11, located at New York, NY, US (with type *IXP*, scoped to *NYIIX*)
    /// sponsored by Telehouse and GTT.
    NewYork,
    /// RIPE RIS collector 12, located at Frankfurt, DE (with type *IXP*, scoped to *DE-CIX*)
    /// sponsored by DE-CIX.
    Frankfurt,
    /// RIPE RIS collector 13, located at Moscow, RU (with type *IXP*, scoped to *MSK-IX*) sponsored
    /// by MSK-IX.
    Moscow,
    /// RIPE RIS collector 14, located at Palo Alto, CA, US (with type *IXP*, scoped to *PAIX*)
    /// sponsored by Equinix.
    PaloAlto,
    /// RIPE RIS collector 15, located at Sao Paolo, BR (with type *IXP*, scoped to *PTTMetro-SP*)
    /// sponsored by IX.br.
    SaoPaolo,
    /// RIPE RIS collector 16, located at Miami, FL, US (with type *IXP*, scoped to *Equinix Miami*)
    /// sponsored by RIPE NCC.
    Miami,
    /// RIPE RIS collector 18, located at Barcelona, ES (with type *IXP*, scoped to *CATNIX*)
    /// sponsored by CATNIX.
    Barcelona,
    /// RIPE RIS collector 19, located at Johannesburg, ZA (with type *IXP*, scoped to *NAP Africa
    /// JB*) sponsored by Network Platforms.
    Johannesburg,
    /// RIPE RIS collector 20, located at Zurich, CH (with type *IXP*, scoped to *SwissIX*)
    /// sponsored by SWISS-IX.
    Zurich,
    /// RIPE RIS collector 21, located at Paris, FR (with type *IXP*, scoped to *France-IX Paris and
    /// France-IX Marseille*) sponsored by France-IX.
    Paris,
    /// RIPE RIS collector 22, located at Bucharest, RO (with type *IXP*, scoped to *Interlan*)
    /// sponsored by InterLAN.
    Bucharest,
    /// RIPE RIS collector 23, located at Singapore, SG (with type *IXP*, scoped to *Equinix
    /// Singapore*) sponsored by Equinix.
    Singapore,
    /// RIPE RIS collector 24, located at Montevideo, UY (with type *multihop*, scoped to *LACNIC
    /// region*) sponsored by LACNIC.
    Montevideo,
    /// RIPE RIS collector 25, located at Amsterdam, NL (with type *multihop*, scoped to *global*)
    /// sponsored by RIPE NCC.
    Amsterdam2,
    /// RIPE RIS collector 26, located at Dubai, AE (with type *IXP*, scoped to *UAE-IX*) sponsored
    /// by Datamena and UAE-IX.
    Dubai,
}

impl RipeNcc {
    fn cstring(&self) -> CString {
        match self {
            RipeNcc::Amsterdam => CString::new("rrc00").unwrap(),
            RipeNcc::London => CString::new("rrc01").unwrap(),
            RipeNcc::AmsterdamIx => CString::new("rrc03").unwrap(),
            RipeNcc::Geneva => CString::new("rrc04").unwrap(),
            RipeNcc::Vienna => CString::new("rrc05").unwrap(),
            RipeNcc::Otemachi => CString::new("rrc06").unwrap(),
            RipeNcc::Stockholm => CString::new("rrc07").unwrap(),
            RipeNcc::Milan => CString::new("rrc10").unwrap(),
            RipeNcc::NewYork => CString::new("rrc11").unwrap(),
            RipeNcc::Frankfurt => CString::new("rrc12").unwrap(),
            RipeNcc::Moscow => CString::new("rrc13").unwrap(),
            RipeNcc::PaloAlto => CString::new("rrc14").unwrap(),
            RipeNcc::SaoPaolo => CString::new("rrc15").unwrap(),
            RipeNcc::Miami => CString::new("rrc16").unwrap(),
            RipeNcc::Barcelona => CString::new("rrc18").unwrap(),
            RipeNcc::Johannesburg => CString::new("rrc19").unwrap(),
            RipeNcc::Zurich => CString::new("rrc20").unwrap(),
            RipeNcc::Paris => CString::new("rrc21").unwrap(),
            RipeNcc::Bucharest => CString::new("rrc22").unwrap(),
            RipeNcc::Singapore => CString::new("rrc23").unwrap(),
            RipeNcc::Montevideo => CString::new("rrc24").unwrap(),
            RipeNcc::Amsterdam2 => CString::new("rrc25").unwrap(),
            RipeNcc::Dubai => CString::new("rrc26").unwrap(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ElementTypeDescr {
    RIBs,
    Withdrawals,
    Announcements,
    PeerStates,
}

/// Query the BGPStream server for an object.
/// Add a filter to an unstarted BGP Stream instance. Only those records/elems that match the
/// filter(s) will be included in the stream.
///
/// If multiple filters of the same type are added, a record/elem is considered a match if it
/// matches any of the filters. E.g. if `self.project("routeviews")` and `project("ris")` are used,
/// then records that are from either the Route Views, or the RIS project will be included.
///
/// If multiple filters of different types are added, a record/elem is considered a match if it
/// matches all of the filters. If `project("routeviews")` and `record_type(RecordType::Updates)`
/// are used, then records that are both from the Route Views project, and are updates will be
/// included.
#[derive(Default, Clone)]
pub struct Query {
    filters: Vec<(bgpstream_filter_type_t, CString)>,
    interval: FilterInterval,
    rib_period: Option<u32>,
    data_interface_options: Vec<(CString, CString, CString)>,
}

impl Query {
    /// Create a new, empty query
    pub fn new() -> Self {
        Default::default()
    }

    /// Filter by the record type (either `RecordType::Updates` or `RecordType::RIBs`).
    pub fn record_type(&mut self, t: RecordType) -> &mut Self {
        self.filters.push((
            FILTER_TYPE_RECORD_TYPE,
            match t {
                RecordType::Updates => CString::new("updates").unwrap(),
                RecordType::RIBs => CString::new("ribs").unwrap(),
            },
        ));
        self
    }

    /// Filter by the collector
    pub fn collector(&mut self, collector: Collector) -> &mut Self {
        self.filters
            .push((FILTER_TYPE_COLLECTOR, collector.cstring()));
        self
    }

    /// Filter by the collector, using the raw name. A list of available collectors can be found
    /// [here](https://bgpstream.caida.org/data#!routeviews).
    pub fn collector_name(&mut self, s: impl Into<Vec<u8>>) -> &mut Self {
        self.filters
            .push((FILTER_TYPE_COLLECTOR, CString::new(s).unwrap()));
        self
    }

    /// Use RouteViews or RIS.
    pub fn project(&mut self, project: Project) -> &mut Self {
        self.filters.push((
            FILTER_TYPE_PROJECT,
            match project {
                Project::RouteViews => CString::new("routeviews").unwrap(),
                Project::RIS => CString::new("ris").unwrap(),
            },
        ));
        self
    }

    /// Filter by the AS path.
    ///
    /// The aspath filter is specifed as a regular expression and will match if the AS path matches
    /// the regular expression. `^` can be used to represent the start of an AS path and `$` can be
    /// used to represent the end of an AS path. `_` can be used to separate adjacent ASNs in the
    /// path. E.g. if the filter value `^681_1444_` is used, only elements with an AS path beginning
    /// with AS681 followed by AS1444 will be included.
    pub fn as_path(&mut self, s: impl Into<Vec<u8>>) -> &mut Self {
        self.filters
            .push((FILTER_TYPE_ELEM_ASPATH, CString::new(s).unwrap()));
        self
    }

    /// Filter the community value. The value is a `ASN:COMM` string pair. You can provide an
    /// asterics (e.g., `*:COMM`) to match on all AS numbers (or all community values).
    pub fn community(&mut self, s: impl Into<Vec<u8>>) -> &mut Self {
        self.filters
            .push((FILTER_TYPE_ELEM_COMMUNITY, CString::new(s).unwrap()));
        self
    }

    /// The ipversion filter can be used to limit the stream to IPv4 or IPv6 prefixes only.
    pub fn ip_version(&mut self, version: IpVersion) -> &mut Self {
        self.filters.push((
            FILTER_TYPE_ELEM_IP_VERSION,
            match version {
                IpVersion::IPv4 => CString::new("4").unwrap(),
                IpVersion::IPv6 => CString::new("6").unwrap(),
            },
        ));
        self
    }

    /// Filter the origin AS number
    pub fn origin_asn(&mut self, s: impl Into<Vec<u8>>) -> &mut Self {
        self.filters
            .push((FILTER_TYPE_ELEM_ORIGIN_ASN, CString::new(s).unwrap()));
        self
    }

    /// Filter the peer AS number.
    pub fn peer_asn(&mut self, s: impl Into<Vec<u8>>) -> &mut Self {
        self.filters
            .push((FILTER_TYPE_ELEM_PEER_ASN, CString::new(s).unwrap()));
        self
    }

    /// Filter the peer AS number (negative).
    pub fn not_peer_asn(&mut self, s: impl Into<Vec<u8>>) -> &mut Self {
        self.filters
            .push((FILTER_TYPE_ELEM_NOT_PEER_ASN, CString::new(s).unwrap()));
        self
    }

    /// Match a given prefix. The kind pf prefix match is given by `kind`. The prefix `s` must be
    /// a string (either IPv4 or IPv6).
    pub fn prefix(&mut self, kind: PrefixMatchType, s: impl Into<Vec<u8>>) -> &mut Self {
        self.filters.push((
            match kind {
                PrefixMatchType::Any => FILTER_TYPE_ELEM_PREFIX_ANY,
                PrefixMatchType::Exact => FILTER_TYPE_ELEM_PREFIX_EXACT,
                PrefixMatchType::Less => FILTER_TYPE_ELEM_PREFIX_LESS,
                PrefixMatchType::More => FILTER_TYPE_ELEM_PREFIX_MORE,
            },
            CString::new(s).unwrap(),
        ));
        self
    }

    /// The element type filter can be used to limit the stream to only certain element types. See
    /// [`crate::element::ElementType`] for options.
    pub fn event_type(&mut self, t: ElementTypeDescr) -> &mut Self {
        self.filters.push((
            FILTER_TYPE_ELEM_TYPE,
            match t {
                ElementTypeDescr::RIBs => CString::new("ribs").unwrap(),
                ElementTypeDescr::Withdrawals => CString::new("withdrawals").unwrap(),
                ElementTypeDescr::Announcements => CString::new("announcements").unwrap(),
                ElementTypeDescr::PeerStates => CString::new("peerstates").unwrap(),
            },
        ));
        self
    }

    /// Set the time interval for the stream.
    pub fn interval(&mut self, interval: FilterInterval) -> &mut Self {
        self.interval = interval;
        self
    }

    /// Set the RIB period filter for the current stream. Configure the minimum BGP time interval
    /// between two consecutive RIB files that belong to the same collector. This information can
    /// be modified once the stream has started.
    pub fn rib_period(&mut self, secs: u32) -> &mut Self {
        self.rib_period = Some(secs);
        self
    }

    /// Set the directory of where to store the cache.
    pub fn cache(&mut self, dir: impl Into<Vec<u8>>) -> &mut Self {
        self.data_interface_options.push((
            CString::new("broker").unwrap(),
            CString::new("cache-dir").unwrap(),
            CString::new(dir).unwrap(),
        ));
        self
    }

    /// Set the data interface option.
    pub fn set_data_interface_option(
        &mut self,
        interface_name: impl Into<Vec<u8>>,
        option: impl Into<Vec<u8>>,
        value: impl Into<Vec<u8>>,
    ) {
        self.data_interface_options.push((
            CString::new(interface_name).unwrap(),
            CString::new(option).unwrap(),
            CString::new(value).unwrap(),
        ))
    }

    /// Create the BGP stream and start the iteration
    pub fn run(&self) -> Result<BgpStream, BgpStreamError> {
        BgpStream::new(&self)
    }
}

/// A BGP stream object to fetch new records. Use [`Query`] to construct a new BgpStream.
///
/// A BGP stream iterates over many [`Record`]s. Each `Record` represents data collected at a
/// specific time. A `Record` consists of many individual [`crate::element::Element`]s, which
/// are the actual RIB entries, BGP updates or peer state changes.
pub struct BgpStream {
    pub(crate) bs: NonNull<bgpstream_t>,
    // current record, used for the iterator.
    current_record: Option<Record<'static>>,
}

/// Iterator over elements.
impl BgpStream {
    fn new(query: &Query) -> Result<BgpStream, BgpStreamError> {
        unsafe {
            let Some(bs) = NonNull::new(bgpstream_create()) else {
                return Err(BgpStreamError::Create);
            };
            let s = Self {
                bs,
                current_record: None,
            };

            // add all filters
            for (filter, value) in &query.filters {
                let filter_value = CString::new(value.as_bytes())?;
                let res = bgpstream_add_filter(s.bs.as_ptr(), *filter, filter_value.as_ptr());
                if res != 1 {
                    return Err(BgpStreamError::AddFilter);
                }
            }

            // apply the interval
            match query.interval {
                FilterInterval::Open => {}
                FilterInterval::Interval { start, stop } => {
                    let start = start.unix_timestamp() as u32;
                    let stop = stop.map(|x| x.unix_timestamp() as u32).unwrap_or(0);
                    let res = bgpstream_add_interval_filter(s.bs.as_ptr(), start, stop);
                    if res != 1 {
                        return Err(BgpStreamError::AddInterval);
                    }
                }
                FilterInterval::Since { amount, unit, live } => {
                    let interval = CString::new(format!("{amount} {unit}").as_bytes()).unwrap();
                    let islive = if live { 1 } else { 0 };
                    let res = bgpstream_add_recent_interval_filter(
                        s.bs.as_ptr(),
                        interval.as_ptr() as *const c_char,
                        islive,
                    );
                    if res != 1 {
                        return Err(BgpStreamError::AddRecentInterval);
                    }
                }
            }

            // apply the RIB period
            if let Some(period) = query.rib_period {
                let res = bgpstream_add_rib_period_filter(s.bs.as_ptr(), period);
                if res != 1 {
                    return Err(BgpStreamError::AddRibPeriod);
                }
            }

            // configure the cache
            for (interface_name, option, value) in &query.data_interface_options {
                // get the broker data interface id
                let if_id =
                    bgpstream_get_data_interface_id_by_name(s.bs.as_ptr(), interface_name.as_ptr());
                if if_id == 0 {
                    return Err(BgpStreamError::InterfaceNotFound(
                        interface_name.to_string_lossy().to_string(),
                    ));
                }

                // get the data interface option
                let opt = bgpstream_get_data_interface_option_by_name(
                    s.bs.as_ptr(),
                    if_id,
                    option.as_ptr(),
                );
                if opt.is_null() {
                    return Err(BgpStreamError::InterfaceOptionNotFound(
                        option.to_string_lossy().to_string(),
                    ));
                }

                let s_value = CString::new(value.as_bytes())?;
                let res = bgpstream_set_data_interface_option(
                    s.bs.as_ptr(),
                    opt,
                    s_value.as_ptr() as *const c_char,
                );
                if res != 0 {
                    return Err(BgpStreamError::SetInterfaceOption);
                }
            }

            // start the stream
            let res = bgpstream_start(s.bs.as_ptr());
            if res != 0 {
                return Err(BgpStreamError::Start);
            }

            Ok(s)
        }
    }

    /// Get the next record.
    ///
    /// If you are using `self` as `Iterator`, then getting the next record will return the current
    /// record of the current iterator state.
    pub fn next_record<'a>(&'a mut self) -> Result<Option<Record<'a>>, BgpStreamError> {
        // delete the current record. That one is now lost!
        if let Some(record) = self.current_record.take() {
            Ok(Some(record))
        } else {
            Record::new(self)
        }
    }
}

impl Drop for BgpStream {
    fn drop(&mut self) {
        unsafe {
            bgpstream_destroy(self.bs.as_ptr());
        }
    }
}

impl Iterator for BgpStream {
    type Item = Result<Element, BgpStreamError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // safety: There does not exist a different record, because `self.stream`
            // is a mutable reference.
            if self.current_record.is_none() {
                // safety: self.record is None.
                match self.next_record() {
                    Ok(Some(r)) => unsafe {
                        self.current_record = Some(r.detach());
                    },
                    Ok(None) => return None,
                    Err(e) => return Some(Err(e)),
                }
            }
            let record = self.current_record.as_mut().unwrap();
            match record.next_element() {
                Ok(Some(e)) => return Some(Ok(e)),
                Ok(None) => {
                    self.current_record = None;
                }
                Err(e) => return Some(Err(e)),
            }
        }
    }
}
