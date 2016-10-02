//! Date and time utils for HTTP.
//!
//! Multiple HTTP header fields store timestamps.
//! For example a response created on May 15, 2015 may contain the header
//! `Date: Fri, 15 May 2015 15:34:21 GMT`. Since the timestamp does not
//! contain any timezone or leap second information it is equvivalent to
//! writing 1431696861 Unix time. Rust’s `SystemTime` is used to store
//! these timestamps.
//!
//! This crate provides two public functions:
//! * `parse_http_date` to parse a HTTP datetime string to a system time
//! * `fmt_http_date` to format a system time to a IMF-fixdate

use std::time::SystemTime;

use datetime::DateTime;

mod datetime;

/// Parse a date from an HTTP header field.
///
/// Supports the preferred IMF-fixdate and the legacy RFC 805 and
/// ascdate formats. Two digit years are mapped to dates between
/// 1980 and 2079.
pub fn parse_http_date(s: &str) -> Result<SystemTime, ()> {
    s.parse::<DateTime>().map(|d| d.into())
}

/// Format a date to be used in a HTTP header field.
pub fn fmt_http_date(d: SystemTime) -> String {
    format!("{}", DateTime::from(d))
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use super::{parse_http_date, fmt_http_date};

    #[test]
    fn test_rfc_example() {
        let d = UNIX_EPOCH + Duration::from_secs(784111777);
        assert_eq!(d, parse_http_date("Sun, 06 Nov 1994 08:49:37 GMT").unwrap());
        assert_eq!(d, parse_http_date("Sunday, 06-Nov-94 08:49:37 GMT").unwrap());
        assert_eq!(d, parse_http_date("Sun Nov  6 08:49:37 1994").unwrap());
    }

    #[test]
    fn test2() {
        let d = UNIX_EPOCH + Duration::from_secs(1475419451);
        assert_eq!(d, parse_http_date("Sun, 02 Oct 2016 14:44:11 GMT").unwrap());
    }

    #[test]
    fn test3() {
        let mut d = UNIX_EPOCH;
        assert_eq!(d, parse_http_date("Thu, 01 Jan 1970 00:00:00 GMT").unwrap());
        d += Duration::from_secs(3600);
        assert_eq!(d, parse_http_date("Thu, 01 Jan 1970 01:00:00 GMT").unwrap());
        d += Duration::from_secs(86400);
        assert_eq!(d, parse_http_date("Fri, 02 Jan 1970 01:00:00 GMT").unwrap());
        d += Duration::from_secs(2592000);
        assert_eq!(d, parse_http_date("Sun, 01 Feb 1970 01:00:00 GMT").unwrap());
        d += Duration::from_secs(2592000);
        assert_eq!(d, parse_http_date("Tue, 03 Mar 1970 01:00:00 GMT").unwrap());
        d += Duration::from_secs(31536005);
        assert_eq!(d, parse_http_date("Wed, 03 Mar 1971 01:00:05 GMT").unwrap());
        d += Duration::from_secs(15552000);
        assert_eq!(d, parse_http_date("Mon, 30 Aug 1971 01:00:05 GMT").unwrap());
        d += Duration::from_secs(6048000);
        assert_eq!(d, parse_http_date("Mon, 08 Nov 1971 01:00:05 GMT").unwrap());
        d += Duration::from_secs(864000000);
        assert_eq!(d, parse_http_date("Fri, 26 Mar 1999 01:00:05 GMT").unwrap());
    }

    #[test]
    fn test_fmt() {
        let d = UNIX_EPOCH;
        assert_eq!(fmt_http_date(d), "Thu, 01 Jan 1970 00:00:00 GMT");
        let d = UNIX_EPOCH + Duration::from_secs(1475419451);
        assert_eq!(fmt_http_date(d), "Sun, 02 Oct 2016 14:44:11 GMT");
    }
}
