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
//!
//! * `parse_http_date` to parse a HTTP datetime string to a system time
//! * `fmt_http_date` to format a system time to a IMF-fixdate

use std::error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::num::ParseIntError;
use std::time::SystemTime;

use datetime::DateTime;

mod datetime;

/// An opaque error type for all parsing errors.
#[derive(Debug)]
pub struct Error(());

impl error::Error for Error {
    fn description(&self) -> &str {
        "string contains no or an invalid date"
    }
    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_str(error::Error::description(self))
    }
}

impl From<ParseIntError> for Error {
    fn from(_: ParseIntError) -> Error {
        Error(())
    }
}

impl From<Error> for io::Error {
    fn from(e: Error) -> io::Error {
        io::Error::new(io::ErrorKind::Other, e)
    }
}

/// Parse a date from an HTTP header field.
///
/// Supports the preferred IMF-fixdate and the legacy RFC 805 and
/// ascdate formats. Two digit years are mapped to dates between
/// 1970 and 2069.
pub fn parse_http_date(s: &str) -> Result<SystemTime, Error> {
    s.parse::<DateTime>().map(|d| d.into())
}

/// Format a date to be used in a HTTP header field.
///
/// Dates are formatted as IMF-fixdate: `Fri, 15 May 2015 15:34:21 GMT`.
pub fn fmt_http_date(d: SystemTime) -> String {
    format!("{}", DateTime::from(d))
}

#[cfg(test)]
mod tests {
    use std::str;
    use std::time::{Duration, UNIX_EPOCH};

    use super::{parse_http_date, fmt_http_date};

    #[test]
    fn test_rfc_example() {
        let d = UNIX_EPOCH + Duration::from_secs(784111777);
        assert_eq!(d, parse_http_date("Sun, 06 Nov 1994 08:49:37 GMT").expect("#1"));
        assert_eq!(d, parse_http_date("Sunday, 06-Nov-94 08:49:37 GMT").expect("#2"));
        assert_eq!(d, parse_http_date("Sun Nov  6 08:49:37 1994").expect("#3"));
    }

    #[test]
    fn test2() {
        let d = UNIX_EPOCH + Duration::from_secs(1475419451);
        assert_eq!(d, parse_http_date("Sun, 02 Oct 2016 14:44:11 GMT").expect("#1"));
        assert!(parse_http_date("Sun Nov 10 08:00:00 1000").is_err());
        assert!(parse_http_date("Sun Nov 10 08*00:00 1000").is_err());
        assert!(parse_http_date("Sunday, 06-Nov-94 08+49:37 GMT").is_err());
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

    fn testcase(data: &[u8]) {
        if let Ok(s) = str::from_utf8(data) {
            println!("{:?}", s);
            if let Ok(d) = parse_http_date(s) {
                let o = fmt_http_date(d);
                assert!(!o.is_empty());
            }
        }
    }

    #[test]
    fn test_fuzz() {
        testcase(b"\x38\x38\x38\x38\x38\x38\x38\x38\x38\x38\x1F\x00\x00\x42\x00\x00\x42\x00\x00\x00\x00\x00\x00\x00\xC8\xBF\x38\x38\x38");
        testcase(b"\x54\x75\x65\x20\x4E\x6F\x76\x20\x38\x37\x37\x37\x37\x37\x37\x35\x35\x35\x35\x31\x31\x31\x34\x34");
        testcase(b"\x54\x75\x65\x20\x4A\x61\x6E\x20\x30\x30\x30\x30\x30\x30\x30\x36\x32\x37\x30\x39\x37\x31\x35\x37");
        testcase(b"\x54\x75\x65\x20\x4A\x61\x6E\x20\x30\x30\x30\x30\x30\x30\x33\x37\x37\x34\x39\x30\x39\x32\x39\x37")
    }
}
