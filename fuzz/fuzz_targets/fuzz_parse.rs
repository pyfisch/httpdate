#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate httpdate;

use std::str;

use httpdate::{parse_http_date, fmt_http_date};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = str::from_utf8(data) {
        if let Ok(d) = parse_http_date(s) {
            let o = fmt_http_date(d);
            assert!(!o.is_empty());
            assert_eq!(parse_http_date(&o).expect("formatting to round trip"), d);
        }
    }
});
