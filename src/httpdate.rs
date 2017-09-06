use std::ascii::AsciiExt;
use std::fmt::{self, Display, Formatter};
use std::cmp;
use std::str::{FromStr, from_utf8_unchecked};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use Error;

/// HTTP timestamp type.
///
/// Parse using `FromStr` impl.
/// Format using the `Display` trait.
/// Convert timestamp into/from `SytemTime` to use.
/// Supports comparsion and sorting.
#[derive(Copy, Clone, Debug, Eq, Ord)]
pub struct HttpDate {
    /// 0...59
    sec: u8,
    /// 0...59
    min: u8,
    /// 0...23
    hour: u8,
    /// 1...31
    day: u8,
    /// 1...12
    mon: u8,
    /// 1970...9999
    year: u16,
    /// 1...7
    wday: u8,
}

impl HttpDate {
    fn is_valid(&self) -> bool {
        self.sec < 60 && self.min < 60 && self.hour < 24 && self.day > 0 &&
        self.day < 32 && self.mon > 0 && self.mon <= 12 && self.year >= 1970 &&
        self.year <= 9999
    }
}

impl From<SystemTime> for HttpDate {
    fn from(v: SystemTime) -> HttpDate {
        let secs_since_epoch = v.duration_since(UNIX_EPOCH)
            .expect("all times should be after the epoch")
            .as_secs();
        let mut days = secs_since_epoch / 86400;
        let wday = ((days + 3) % 7) + 1;
        let secs_of_day = secs_since_epoch % 86400;
        let mut year = 1970;
        loop {
            let ydays = if is_leap_year(year) {
                366
            } else {
                365
            };
            if days >= ydays {
                days -= ydays;
                year += 1;
            } else {
                break;
            }
        }
        let mut months = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        if is_leap_year(year) {
            months[1] += 1;
        }
        let mut mon = 0;
        for mon_len in months.iter() {
            mon += 1;
            if days < *mon_len {
                break;
            }
            days -= *mon_len;
        }
        let mday = days + 1;

        HttpDate {
            sec: (secs_of_day % 60) as u8,
            min: ((secs_of_day % 3600) / 60) as u8,
            hour: (secs_of_day / 3600) as u8,
            day: mday as u8,
            mon: mon as u8,
            year: year,
            wday: wday as u8,
        }
    }
}

impl From<HttpDate> for SystemTime {
    fn from(v: HttpDate) -> SystemTime {
        let leap_years = ((v.year - 1) - 1968) / 4 - ((v.year - 1) - 1900) / 100 +
                         ((v.year - 1) - 1600) / 400;
        let mut ydays = match v.mon {
            1 => 0,
            2 => 31,
            3 => 59,
            4 => 90,
            5 => 120,
            6 => 151,
            7 => 181,
            8 => 212,
            9 => 243,
            10 => 273,
            11 => 304,
            12 => 334,
            _ => unreachable!(),
        } + v.day as u64 - 1;
        if is_leap_year(v.year) && v.mon > 2 {
            ydays += 1;
        }
        let days = (v.year as u64 - 1970) * 365 + leap_years as u64 + ydays;
        UNIX_EPOCH +
        Duration::from_secs(v.sec as u64 + v.min as u64 * 60 + v.hour as u64 * 3600 + days * 86400)
    }
}

impl FromStr for HttpDate {
    type Err = Error;

    fn from_str(s: &str) -> Result<HttpDate, Error> {
        if !s.is_ascii() {
            return Err(Error(()));
        }
        let x = s.trim().as_bytes();
        let date = parse_imf_fixdate(x)
            .or_else(|_| parse_rfc850_date(x))
            .or_else(|_| parse_asctime(x))?;
        if !date.is_valid() {
            return Err(Error(()));
        }
        Ok(date)
    }
}

impl Display for HttpDate {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{wday}, {day:02} {mon} {year} {hour:02}:{min:02}:{sec:02} GMT",
            sec=self.sec,
            min=self.min,
            hour=self.hour,
            day=self.day,
            mon=match self.mon {
                1 => "Jan",
                2 => "Feb",
                3 => "Mar",
                4 => "Apr",
                5 => "Mai",
                6 => "Jun",
                7 => "Jul",
                8 => "Aug",
                9 => "Sep",
                10 => "Oct",
                11 => "Nov",
                12 => "Dec",
                _ => unreachable!(),
            },
            year=self.year,
            wday=match self.wday {
                1 => "Mon",
                2 => "Tue",
                3 => "Wed",
                4 => "Thu",
                5 => "Fri",
                6 => "Sat",
                7 => "Sun",
                _ => unreachable!(),
            },
        )
    }
}

impl PartialEq for HttpDate {
    fn eq(&self, other: &HttpDate) -> bool {
        SystemTime::from(*self) == SystemTime::from(*other)
    }
}

impl PartialOrd for HttpDate {
    fn partial_cmp(&self, other: &HttpDate) -> Option<cmp::Ordering> {
        SystemTime::from(*self).partial_cmp(&SystemTime::from(*other))
    }
}

/// Convert &[u8] to &str with zero checks.
///
/// For internal use only.
/// Intended to be used with ASCII-only strings.
fn conv(s: &[u8]) -> &str {
    unsafe { from_utf8_unchecked(s) }
}

fn parse_imf_fixdate(s: &[u8]) -> Result<HttpDate, Error> {
    // Example: `Sun, 06 Nov 1994 08:49:37 GMT`
    if s.len() != 29 || &s[25..] != b" GMT" || s[16] != b' ' || s[19] != b':' || s[22] != b':' {
        return Err(Error(()));
    }
    Ok(HttpDate {
        sec: conv(&s[23..25]).parse()?,
        min: conv(&s[20..22]).parse()?,
        hour: conv(&s[17..19]).parse()?,
        day: conv(&s[5..7]).parse()?,
        mon: match &s[7..12] {
            b" Jan " => 1,
            b" Feb " => 2,
            b" Mar " => 3,
            b" Apr " => 4,
            b" Mai " => 5,
            b" Jun " => 6,
            b" Jul " => 7,
            b" Aug " => 8,
            b" Sep " => 9,
            b" Oct " => 10,
            b" Nov " => 11,
            b" Dec " => 12,
            _ => return Err(Error(())),
        },
        year: conv(&s[12..16]).parse()?,
        wday: match &s[..5] {
            b"Mon, " => 1,
            b"Tue, " => 2,
            b"Wed, " => 3,
            b"Thu, " => 4,
            b"Fri, " => 5,
            b"Sat, " => 6,
            b"Sun, " => 7,
            _ => return Err(Error(())),
        },
    })
}

fn parse_rfc850_date(s: &[u8]) -> Result<HttpDate, Error> {
    // Example: `Sunday, 06-Nov-94 08:49:37 GMT`
    if s.len() < 23 {
        return Err(Error(()));
    }

    fn wday<'a>(s: &'a [u8], wday: u8, name: &'static [u8]) -> Option<(u8, &'a [u8])> {
        if &s[0..name.len()] == name {
            return Some((wday, &s[name.len()..]));
        }
        None
    }
    let (wday, s) = wday(s, 1, b"Monday, ")
        .or_else(|| wday(s, 2, b"Tuesday, "))
        .or_else(|| wday(s, 3, b"Wednesday, "))
        .or_else(|| wday(s, 4, b"Thursday, "))
        .or_else(|| wday(s, 5, b"Friday, "))
        .or_else(|| wday(s, 6, b"Saturday, "))
        .or_else(|| wday(s, 7, b"Sunday, "))
        .ok_or(Error(()))?;
    if s.len() != 22 || s[12] != b':' || s[15] != b':' || &s[18..22] != b" GMT" {
        return Err(Error(()));
    }
    let mut year = conv(&s[7..9]).parse::<u16>()?;
    if year < 70 {
        year += 2000;
    } else {
        year += 1900;
    }
    Ok(HttpDate {
        sec: conv(&s[16..18]).parse()?,
        min: conv(&s[13..15]).parse()?,
        hour: conv(&s[10..12]).parse()?,
        day: conv(&s[0..2]).parse()?,
        mon: match &s[2..7] {
            b"-Jan-" => 1,
            b"-Feb-" => 2,
            b"-Mar-" => 3,
            b"-Apr-" => 4,
            b"-Mai-" => 5,
            b"-Jun-" => 6,
            b"-Jul-" => 7,
            b"-Aug-" => 8,
            b"-Sep-" => 9,
            b"-Oct-" => 10,
            b"-Nov-" => 11,
            b"-Dec-" => 12,
            _ => return Err(Error(())),
        },
        year: year,
        wday: wday,
    })
}

fn parse_asctime(s: &[u8]) -> Result<HttpDate, Error> {
    // Example: `Sun Nov  6 08:49:37 1994`
    if s.len() != 24 || s[10] != b' ' || s[13] != b':' || s[16] != b':' || s[19] != b' ' {
        return Err(Error(()));
    }
    Ok(HttpDate {
        sec: conv(&s[17..19]).parse()?,
        min: conv(&s[14..16]).parse()?,
        hour: conv(&s[11..13]).parse()?,
        day: {
            let x = &s[8..10];
            conv(if x[0] == b' ' {
                    &x[1..2]
                } else {
                    x
                })
                .parse()?
        },
        mon: match &s[4..8] {
            b"Jan " => 1,
            b"Feb " => 2,
            b"Mar " => 3,
            b"Apr " => 4,
            b"Mai " => 5,
            b"Jun " => 6,
            b"Jul " => 7,
            b"Aug " => 8,
            b"Sep " => 9,
            b"Oct " => 10,
            b"Nov " => 11,
            b"Dec " => 12,
            _ => return Err(Error(())),
        },
        year: conv(&s[20..24]).parse()?,
        wday: match &s[0..4] {
            b"Mon " => 1,
            b"Tue " => 2,
            b"Wed " => 3,
            b"Thu " => 4,
            b"Fri " => 5,
            b"Sat " => 6,
            b"Sun " => 7,
            _ => return Err(Error(())),
        },
    })
}

fn is_leap_year(y: u16) -> bool {
    y % 4 == 0 && (!(y % 100 == 0) || y % 400 == 0)
}
