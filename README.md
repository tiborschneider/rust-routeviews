# Convenient wrapper around [BGPStream](https://bgpstream.caida.org).

This Library performs a `Query` and yields the results as an iterator. Look at `Query` as the main entry point of using this library.

The following example reads all updates from the AMSIX RouteView within one hour and prints the time of each update.

```rust
use routeviews::stream::*;
use time::macros::datetime;

fn main() {
    let stream = Query::new()
        .collector(Collector::RouteView(RouteView::Amsix))
        .record_type(RecordType::Updates)
        .interval(FilterInterval::Interval {
            start: datetime!(2023-11-08 09:55 UTC).into(),
            stop: datetime!(2023-11-08 10:55 UTC).into(),
        })
        .cache("/tmp/bgpstream_cache")
        .run()
        .unwrap();

    for element in stream {
        let element = element.unwrap();
        println!("{:?}", element.time.to_hms());
    }
}
```
