#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate tl;

// Mutation harness for tl's DOM. Mirrors the intent of upstream's parse_mut
// (parse arbitrary bytes, then mutate every tag node) but uses tl's CURRENT
// mutable API — attributes_mut().insert(...) and name_mut().set(...). Upstream's
// fuzz/fuzz_targets/parse_mut.rs is stale at this pinned commit (it calls
// insert_attribute / inner_html_mut, which no longer exist). We add this rather
// than edit the unmodified upstream library.
fuzz_target!(|data: &str| {
    let mut dom = match tl::parse(data, tl::ParserOptions::default()) {
        Ok(dom) => dom,
        Err(_) => return,
    };

    for node in dom.nodes_mut() {
        if let Some(tag) = node.as_tag_mut() {
            tag.attributes_mut().insert("test", Some("testing"));
            let _ = tag.name_mut().set("<b>Hello World</b>".as_bytes());
        }
    }
});
