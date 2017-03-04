#![no_main]
extern crate libfuzzer_sys;
extern crate httpdate;

use std::str;

use httpdate::{parse_http_date, fmt_http_date};

#[export_name="rust_fuzzer_test_input"]
pub extern fn go(data: &[u8]) {
    if let Ok(s) = str::from_utf8(data) {
        if let Ok(d) = parse_http_date(s) {
            let o = fmt_http_date(d);
            assert!(!o.is_empty());
        }
    }
}
