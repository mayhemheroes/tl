#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate tl;

fuzz_target!(|data: &str| {
    if let Ok(mut dom) = tl::parse(data, tl::ParserOptions::default()) {
        // ... some random DOM mutations ...
        for node in dom.nodes_mut() {
            if let Some(tag) = node.as_tag_mut() {
                tag.attributes_mut().insert("test", Some("testing"));
            }
        }
    }
});
