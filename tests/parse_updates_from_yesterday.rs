//! Module to test each collector.

use routeviews::{stream::*, BgpStreamError};
use time::{Duration, OffsetDateTime};

macro_rules! test_route_view {
    ($name:ident, $collector:expr) => {
        #[test]
        fn $name() {
            for hour in (1..=10) {
                std::thread::sleep(std::time::Duration::from_millis(1000));
                let start = OffsetDateTime::now_utc() - Duration::days(1) - Duration::hours(hour);
                let stop = Some(start + Duration::minutes(5));
                let stream = match Query::new()
                    .collector($collector)
                    .record_type(RecordType::Updates)
                    .interval(FilterInterval::Interval { start, stop })
                    .run()
                {
                    Ok(stream) => stream,
                    Err(BgpStreamError::RecordSourceEmptyOrCorrupted) => continue,
                    Err(e) => panic!("Received an unexpected error: {e:?}"),
                };

                for element in stream {
                    let _element = element.unwrap();
                }

                return;
            }

            panic!("None of the 10 tries resulted in a usable result!")
        }
    };
}

test_route_view!(route_view_amsix, Collector::RouteView(RouteView::Amsix));
// test_route_view!(route_view_bdix, Collector::RouteView(RouteView::Bdix));
test_route_view!(route_view_bknix, Collector::RouteView(RouteView::Bknix));
test_route_view!(route_view_chicago, Collector::RouteView(RouteView::Chicago));
test_route_view!(route_view_chile, Collector::RouteView(RouteView::Chile));
test_route_view!(route_view_eqix, Collector::RouteView(RouteView::Eqix));
test_route_view!(route_view_flix, Collector::RouteView(RouteView::Flix));
test_route_view!(
    route_view_fortaleza,
    Collector::RouteView(RouteView::Fortaleza)
);
// test_route_view!(route_view_gixa, Collector::RouteView(RouteView::Gixa));
test_route_view!(route_view_gorex, Collector::RouteView(RouteView::Gorex));
test_route_view!(route_view_isc, Collector::RouteView(RouteView::Isc));
test_route_view!(route_view_kixp, Collector::RouteView(RouteView::Kixp));
test_route_view!(route_view_linx, Collector::RouteView(RouteView::Linx));
// test_route_view!(route_view_mwix, Collector::RouteView(RouteView::Mwix));
test_route_view!(
    route_view_napafrica,
    Collector::RouteView(RouteView::Napafrica)
);
test_route_view!(route_view_nwax, Collector::RouteView(RouteView::Nwax));
test_route_view!(route_view_ny, Collector::RouteView(RouteView::Ny));
test_route_view!(route_view_perth, Collector::RouteView(RouteView::Perth));
test_route_view!(route_view_peru, Collector::RouteView(RouteView::Peru));
test_route_view!(route_view_phoix, Collector::RouteView(RouteView::Phoix));
test_route_view!(route_view_rio, Collector::RouteView(RouteView::Rio));
test_route_view!(route_view_sfmix, Collector::RouteView(RouteView::Sfmix));
test_route_view!(route_view_sg, Collector::RouteView(RouteView::Sg));
test_route_view!(route_view_soxrs, Collector::RouteView(RouteView::Soxrs));
test_route_view!(route_view_sydney, Collector::RouteView(RouteView::Sydney));
test_route_view!(route_view_telxatl, Collector::RouteView(RouteView::Telxatl));
test_route_view!(route_view_uaeix, Collector::RouteView(RouteView::Uaeix));
test_route_view!(route_view_wide, Collector::RouteView(RouteView::Wide));
test_route_view!(route_view_view2, Collector::RouteView(RouteView::View2));
test_route_view!(
    route_view2_sao_paulo,
    Collector::RouteView(RouteView::View2SaoPaulo)
);
test_route_view!(route_view3, Collector::RouteView(RouteView::View3));
test_route_view!(route_view4, Collector::RouteView(RouteView::View4));
test_route_view!(route_view5, Collector::RouteView(RouteView::View5));
test_route_view!(route_view6, Collector::RouteView(RouteView::View6));
test_route_view!(ripe_ris_amsterdam, Collector::RipeNcc(RipeNcc::Amsterdam));
test_route_view!(ripe_ris_london, Collector::RipeNcc(RipeNcc::London));
test_route_view!(
    ripe_ris_amsterdam_ix,
    Collector::RipeNcc(RipeNcc::AmsterdamIx)
);
test_route_view!(ripe_ris_geneva, Collector::RipeNcc(RipeNcc::Geneva));
test_route_view!(ripe_ris_vienna, Collector::RipeNcc(RipeNcc::Vienna));
test_route_view!(ripe_ris_otemachi, Collector::RipeNcc(RipeNcc::Otemachi));
test_route_view!(ripe_ris_stockholm, Collector::RipeNcc(RipeNcc::Stockholm));
test_route_view!(ripe_ris_milan, Collector::RipeNcc(RipeNcc::Milan));
test_route_view!(ripe_ris_new_york, Collector::RipeNcc(RipeNcc::NewYork));
test_route_view!(ripe_ris_frankfurt, Collector::RipeNcc(RipeNcc::Frankfurt));
test_route_view!(ripe_ris_moscow, Collector::RipeNcc(RipeNcc::Moscow));
test_route_view!(ripe_ris_palo_alto, Collector::RipeNcc(RipeNcc::PaloAlto));
test_route_view!(ripe_ris_sao_paolo, Collector::RipeNcc(RipeNcc::SaoPaolo));
test_route_view!(ripe_ris_miami, Collector::RipeNcc(RipeNcc::Miami));
test_route_view!(ripe_ris_barcelona, Collector::RipeNcc(RipeNcc::Barcelona));
test_route_view!(
    ripe_ris_johannesburg,
    Collector::RipeNcc(RipeNcc::Johannesburg)
);
test_route_view!(ripe_ris_zurich, Collector::RipeNcc(RipeNcc::Zurich));
test_route_view!(ripe_ris_paris, Collector::RipeNcc(RipeNcc::Paris));
test_route_view!(ripe_ris_bucharest, Collector::RipeNcc(RipeNcc::Bucharest));
test_route_view!(ripe_ris_singapore, Collector::RipeNcc(RipeNcc::Singapore));
test_route_view!(ripe_ris_montevideo, Collector::RipeNcc(RipeNcc::Montevideo));
test_route_view!(ripe_ris_amsterdam2, Collector::RipeNcc(RipeNcc::Amsterdam2));
test_route_view!(ripe_ris_dubai, Collector::RipeNcc(RipeNcc::Dubai));
