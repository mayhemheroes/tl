#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate tl;

const HTML: &str = r#"
<!DOCTYPE html>
<div>
    <p id="greeting">Hello World</p>
    <img id="img" src="image.png" />
</div>
"#;

// Fuzz the query-selector PARSER with arbitrary `data` as the selector string,
// against a fixed parsed DOM. (Upstream's fuzz/fuzz_targets/queryselector.rs is
// stale at this pinned commit: it calls .query_selector on the Result returned by
// tl::parse and ignores that query_selector now returns an Option — won't compile.
// We do not edit upstream; this is the additive copy.)
fuzz_target!(|data: &str| {
    let dom = match tl::parse(HTML, tl::ParserOptions::default()) {
        Ok(dom) => dom,
        Err(_) => return,
    };
    if let Some(iter) = dom.query_selector(data) {
        for _ in iter {
            // do nothing
        }
    }
});
