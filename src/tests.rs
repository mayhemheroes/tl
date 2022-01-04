use crate::{parse, parse_owned};
use crate::{parser::*, HTMLTag, Node};

fn force_as_tag<'a, 'b>(actual: &'a Node<'b>) -> &'a HTMLTag<'b> {
    match actual {
        Node::Tag(t) => t,
        _ => panic!("Failed to force tag as Node::Tag (got {:?})", actual),
    }
}

#[test]
fn inner_html() {
    let dom = parse("abc <p>test</p> def", ParserOptions::default()).unwrap();
    let parser = dom.parser();

    let tag = force_as_tag(dom.children()[1].get(parser).unwrap());

    assert_eq!(tag.inner_html().as_utf8_str(), "<p>test</p>");
}

#[test]
fn children_len() {
    let dom = parse(
        "<!-- element 1 --> <div><div>element 3</div></div>",
        ParserOptions::default(),
    )
    .unwrap();
    assert_eq!(dom.children().len(), 2);
}

#[test]
fn get_element_by_id_default() {
    let dom = parse(
        "<div></div><p id=\"test\"></p><p></p>",
        ParserOptions::default(),
    )
    .unwrap();

    let tag = dom.get_element_by_id("test").expect("Element not present");

    let el = force_as_tag(tag.get(dom.parser()).unwrap());

    assert_eq!(el.inner_html().as_utf8_str(), "<p id=\"test\"></p>")
}

#[test]
fn get_element_by_id_tracking() {
    let dom = parse(
        "<div></div><p id=\"test\"></p><p></p>",
        ParserOptions::default().track_ids(),
    )
    .unwrap();

    let tag = dom.get_element_by_id("test").expect("Element not present");

    let el = force_as_tag(tag.get(dom.parser()).unwrap());

    assert_eq!(el.inner_html().as_utf8_str(), "<p id=\"test\"></p>")
}

#[test]
fn get_element_by_class_name_default() {
    let dom = parse(
        "<div></div><p class=\"a b\">hey</p><p></p>",
        ParserOptions::default(),
    )
    .unwrap();

    let tag = dom.get_elements_by_class_name("a").next().unwrap();

    let el = force_as_tag(tag.get(dom.parser()).unwrap());

    assert_eq!(el.inner_text(dom.parser()), "hey");
}

#[test]
fn get_element_by_class_name_tracking() {
    let dom = parse(
        "<div></div><p class=\"a b\">hey</p><p></p>",
        ParserOptions::default().track_ids(),
    )
    .unwrap();

    let tag = dom.get_elements_by_class_name("a").next().unwrap();

    let el = force_as_tag(tag.get(dom.parser()).unwrap());

    assert_eq!(el.inner_text(dom.parser()), "hey");
}

#[test]
fn html5() {
    let dom = parse("<!DOCTYPE html> hello", ParserOptions::default()).unwrap();

    assert_eq!(dom.version(), Some(HTMLVersion::HTML5));
    assert_eq!(dom.children().len(), 1)
}

#[test]
fn nested_inner_text() {
    let dom = parse(
        "<p>hello <p>nested element</p></p>",
        ParserOptions::default(),
    )
    .unwrap();
    let parser = dom.parser();

    let el = force_as_tag(dom.children()[0].get(parser).unwrap());

    assert_eq!(el.inner_text(parser), "hello nested element");
}

#[test]
fn owned_dom() {
    let owned_dom = {
        let input = String::from("<p id=\"test\">hello</p>");
        let dom = unsafe { parse_owned(input, ParserOptions::default()).unwrap() };
        dom
    };

    let dom = owned_dom.get_ref();
    let parser = dom.parser();

    let el = force_as_tag(dom.children()[0].get(parser).unwrap());

    assert_eq!(el.inner_text(parser), "hello");
}

#[test]
fn move_owned() {
    let input = String::from("<p id=\"test\">hello</p>");

    let guard = unsafe { parse_owned(input, ParserOptions::default()).unwrap() };

    fn move_me<T>(p: T) -> T {
        p
    }

    let guard = std::thread::spawn(|| guard).join().unwrap();
    let guard = move_me(guard);

    let dom = guard.get_ref();
    let parser = dom.parser();

    let el = force_as_tag(dom.children()[0].get(parser).unwrap());

    assert_eq!(el.inner_text(parser), "hello");
}

#[test]
fn with() {
    let input = r#"<p>hello <span>whats up</span></p>"#;

    let dom = parse(input, ParserOptions::default()).unwrap();
    let parser = dom.parser();

    let tag = dom
        .nodes()
        .iter()
        .find(|x| x.as_tag().map_or(false, |x| x.name() == "span".into()));

    assert_eq!(
        tag.map(|tag| tag.inner_text(parser)),
        Some("whats up".into())
    )
}

#[test]
fn abrupt_attributes_stop() {
    let input = r#"<p "#;
    parse(input, ParserOptions::default()).unwrap();
}

#[test]
fn dom_nodes() {
    let input = r#"<p><p><a>nested</a></p></p>"#;
    let dom = parse(input, ParserOptions::default()).unwrap();
    let parser = dom.parser();
    let element = dom
        .nodes()
        .iter()
        .find(|x| x.as_tag().map_or(false, |x| x.name().eq(&"a".into())));

    assert_eq!(element.map(|x| x.inner_text(parser)), Some("nested".into()));
}

#[test]
fn fuzz() {
    // Some tests that would previously panic or end in an infinite loop
    // We don't need to assert anything here, just see that they finish
    parse("J\x00<", ParserOptions::default()).unwrap();
    parse("<!J", ParserOptions::default()).unwrap();

    // Miri is too slow... :(
    let count = if cfg!(miri) { 100usize } else { 10000usize };

    parse(&"<p>".repeat(count), ParserOptions::default()).unwrap();
}

#[test]
fn query_selector_simple() {
    let input = "<div><p class=\"hi\">hello</p></div>";
    let dom = parse(input, ParserOptions::default()).unwrap();
    let parser = dom.parser();
    let mut selector = dom.query_selector(".hi").unwrap();
    let el = force_as_tag(selector.next().and_then(|x| x.get(parser)).unwrap());

    assert_eq!(dom.nodes().len(), 3);
    assert_eq!(el.inner_text(parser), "hello");
}

#[test]
fn mutate_dom() {
    let input = r#"<img src="test.png" />"#;
    let mut dom = parse(input, ParserOptions::default()).unwrap();

    let mut selector = dom.query_selector("[src]").unwrap();
    let handle = selector.next().unwrap();

    let parser = dom.parser_mut();

    let el = handle.get_mut(parser).unwrap();
    let tag = el.as_tag_mut().unwrap();
    let attr = tag.attributes_mut();
    let bytes = attr.get_attribute_mut("src").flatten().unwrap();
    bytes.set("world.png".as_bytes()).unwrap();

    assert_eq!(attr.get_attribute("src"), Some(Some("world.png".into())));
}

#[cfg(feature = "simd")]
mod simd {
    // These tests make sure that SIMD functions do the right thing

    use crate::util;

    #[test]
    fn string_search() {
        assert_eq!(util::find_fast(b"a", b' '), None);
        assert_eq!(util::find_fast(b"", b' '), None);
        assert_eq!(util::find_fast(b"a ", b' '), Some(1));
        assert_eq!(util::find_fast(b"abcd ", b' '), Some(4));
        assert_eq!(util::find_fast(b"ab cd ", b' '), Some(2));
        assert_eq!(util::find_fast(b"abcdefgh ", b' '), Some(8));
        assert_eq!(util::find_fast(b"abcdefghi ", b' '), Some(9));
        assert_eq!(util::find_fast(b"abcdefghi", b' '), None);
        assert_eq!(util::find_fast(b"abcdefghiabcdefghi .", b' '), Some(18));
        assert_eq!(util::find_fast(b"abcdefghiabcdefghi.", b' '), None);

        let count = if cfg!(miri) { 500usize } else { 1000usize };

        let long = "a".repeat(count) + "b";
        assert_eq!(util::find_fast(long.as_bytes(), b'b'), Some(count));
    }

    #[test]
    fn string_search_4() {
        const NEEDLE: [u8; 4] = [b'a', b'b', b'c', b'd'];

        assert_eq!(util::find_fast_4(b"e", NEEDLE), None);
        assert_eq!(util::find_fast_4(b"a", NEEDLE), Some(0));
        assert_eq!(util::find_fast_4(b"ea", NEEDLE), Some(1));
        assert_eq!(util::find_fast_4(b"ef", NEEDLE), None);
        assert_eq!(util::find_fast_4(b"ef a", NEEDLE), Some(3));
        assert_eq!(util::find_fast_4(b"ef g", NEEDLE), None);
        assert_eq!(util::find_fast_4(b"ef ghijk", NEEDLE), None);
        assert_eq!(util::find_fast_4(b"ef ghijkl", NEEDLE), None);
        assert_eq!(util::find_fast_4(b"ef ghijkla", NEEDLE), Some(9));
        assert_eq!(util::find_fast_4(b"ef ghiajklm", NEEDLE), Some(6));
        assert_eq!(util::find_fast_4(b"ef ghibjklm", NEEDLE), Some(6));
        assert_eq!(util::find_fast_4(b"ef ghicjklm", NEEDLE), Some(6));
        assert_eq!(util::find_fast_4(b"ef ghidjklm", NEEDLE), Some(6));
        assert_eq!(util::find_fast_4(b"ef ghijklmnopqrstua", NEEDLE), Some(18));
        assert_eq!(util::find_fast_4(b"ef ghijklmnopqrstub", NEEDLE), Some(18));
        assert_eq!(util::find_fast_4(b"ef ghijklmnopqrstuc", NEEDLE), Some(18));
        assert_eq!(util::find_fast_4(b"ef ghijklmnopqrstud", NEEDLE), Some(18));
        assert_eq!(util::find_fast_4(b"ef ghijklmnopqrstu", NEEDLE), None);
    }

    #[test]
    #[rustfmt::skip]
    fn search_non_ident() {
        assert_eq!(util::search_non_ident_fast(b"this-is-a-very-long-identifier<"), Some(30));
        assert_eq!(util::search_non_ident_fast(b"0123456789Abc_-<"), Some(15));
        assert_eq!(util::search_non_ident_fast(b"0123456789Abc-<"), Some(14));
        assert_eq!(util::search_non_ident_fast(b"0123456789Abcdef_-<"), Some(18));
        assert_eq!(util::search_non_ident_fast(b""), None);
        assert_eq!(util::search_non_ident_fast(b"short"), None);
        assert_eq!(util::search_non_ident_fast(b"short_<"), Some(6));
        assert_eq!(util::search_non_ident_fast(b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_"), None);
        assert_eq!(util::search_non_ident_fast(b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_<"), Some(64));
        assert_eq!(util::search_non_ident_fast(b"0123456789ab<defghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_<"), Some(12));
    }
}

mod bytes {
    use crate::bytes::*;

    #[test]
    fn from_str() {
        let x = Bytes::from("hello");
        assert_eq!(x.as_bytes(), b"hello");
    }

    #[test]
    fn from_bytes() {
        let x = Bytes::from(b"hello" as &[u8]);
        assert_eq!(x.as_bytes(), b"hello");
    }

    #[test]
    fn as_bytes_borrowed() {
        let xb = Bytes::from(b"hello" as &[u8]);
        assert_eq!(xb.as_bytes_borrowed(), Some(b"hello" as &[u8]));

        let mut xc = xb.clone();
        xc.set(b"test2" as &[u8]).unwrap();
        assert_eq!(xc.as_bytes_borrowed(), None);
    }

    #[test]
    fn as_utf8_str() {
        assert_eq!(Bytes::from("hello").as_utf8_str(), "hello");
    }

    #[test]
    fn clone_shallow() {
        // cloning a borrowed slice does not deep-clone
        let x = Bytes::from("hello");
        let xp = x.as_ptr();

        let y = x.clone();
        let yp = y.as_ptr();

        assert_eq!(xp, yp);
    }

    #[test]
    fn drop_old_owned() {
        let mut x = Bytes::from("");
        x.set("test".as_bytes()).unwrap();
        x.set("test2".as_bytes()).unwrap();
    }

    #[test]
    fn clone_owned_deep() {
        let mut x = Bytes::from("");
        x.set(b"hello" as &[u8]).unwrap();
        let xp = x.as_ptr();

        let y = x.clone();
        let yp = y.as_ptr();

        assert_eq!(x, y);
        assert_ne!(xp, yp);
    }

    #[test]
    fn set() {
        let mut x = Bytes::from("hello");
        let xp = x.as_ptr();

        x.set(b"world" as &[u8]).unwrap();
        let xp2 = x.as_ptr();

        // check that the changes are reflected
        assert_eq!(x.as_bytes(), b"world");

        // pointer must be different now as the call to `set` should cause an allocation
        assert_ne!(xp, xp2);
    }

    #[test]
    fn clone_deep() {
        let x = Bytes::from("hello");
        let xp = x.as_ptr();

        let mut y = x.clone();
        y.set(b"world" as &[u8]).unwrap();
        let yp = y.as_ptr();

        assert_ne!(xp, yp);
    }
}

#[test]
fn valueless_attribute() {
    // https://github.com/y21/tl/issues/11
    let input = r#"
        <a id="u54423">
            <iframe allowfullscreen></iframe>
        </a>
    "#;

    let dom = parse(input, ParserOptions::default()).unwrap();
    let element = dom.get_element_by_id("u54423");

    assert!(element.is_some());
}

#[test]
fn unquoted() {
    // https://github.com/y21/tl/issues/12
    let input = r#"
        <a id=u54423>Hello World</a>
    "#;

    let dom = parse(input, ParserOptions::default()).unwrap();
    let parser = dom.parser();
    let element = dom.get_element_by_id("u54423");

    assert_eq!(
        element.and_then(|x| x.get(parser).map(|x| x.inner_text(parser))),
        Some("Hello World".into())
    );
}
