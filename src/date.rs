use std::cmp;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::Error;

/// HTTP timestamp type.
///
/// Parse using `FromStr` impl.
/// Format using the `Display` trait.
/// Convert timestamp into/from `SytemTime` to use.
/// Supports comparsion and sorting.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
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
        self.sec < 60
            && self.min < 60
            && self.hour < 24
            && self.day > 0
            && self.mon > 0
            && self.mon <= 12
            && self.year >= 1970
            && self.year <= 9999
            && self.day <= datealgo::days_in_month(self.year as i32, self.mon)
            && self.wday == datealgo::date_to_weekday((self.year as i32, self.mon, self.day))
    }
}

impl From<SystemTime> for HttpDate {
    fn from(v: SystemTime) -> HttpDate {
        let dur = v
            .duration_since(UNIX_EPOCH)
            .expect("all times should be after the epoch");
        let secs_since_epoch = dur.as_secs();

        if secs_since_epoch >= 253402300800 {
            // year 9999
            panic!("date must be before year 9999");
        }

        let (year, mon, day, hour, min, sec, _) = datealgo::systemtime_to_datetime(v).unwrap();
        let wday = datealgo::date_to_weekday((year, mon, day));
        HttpDate {
            sec,
            min,
            hour,
            day,
            mon,
            year: year as u16,
            wday,
        }
    }
}

impl From<HttpDate> for SystemTime {
    fn from(v: HttpDate) -> SystemTime {
        datealgo::datetime_to_systemtime((
            v.year as i32,
            v.mon,
            v.day,
            v.hour,
            v.min,
            v.sec,
            0
        )).expect("datetime not representable as SystemTime")
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
        let wday = match self.wday {
            1 => b"Mon",
            2 => b"Tue",
            3 => b"Wed",
            4 => b"Thu",
            5 => b"Fri",
            6 => b"Sat",
            7 => b"Sun",
            _ => unreachable!(),
        };

        let mon = match self.mon {
            1 => b"Jan",
            2 => b"Feb",
            3 => b"Mar",
            4 => b"Apr",
            5 => b"May",
            6 => b"Jun",
            7 => b"Jul",
            8 => b"Aug",
            9 => b"Sep",
            10 => b"Oct",
            11 => b"Nov",
            12 => b"Dec",
            _ => unreachable!(),
        };

        let mut buf: [u8; 29] = *b"   , 00     0000 00:00:00 GMT";
        buf[0] = wday[0];
        buf[1] = wday[1];
        buf[2] = wday[2];
        buf[5] = b'0' + (self.day / 10);
        buf[6] = b'0' + (self.day % 10);
        buf[8] = mon[0];
        buf[9] = mon[1];
        buf[10] = mon[2];
        buf[12] = b'0' + (self.year / 1000) as u8;
        buf[13] = b'0' + (self.year / 100 % 10) as u8;
        buf[14] = b'0' + (self.year / 10 % 10) as u8;
        buf[15] = b'0' + (self.year % 10) as u8;
        buf[17] = b'0' + (self.hour / 10);
        buf[18] = b'0' + (self.hour % 10);
        buf[20] = b'0' + (self.min / 10);
        buf[21] = b'0' + (self.min % 10);
        buf[23] = b'0' + (self.sec / 10);
        buf[24] = b'0' + (self.sec % 10);
        f.write_str(std::str::from_utf8(&buf[..]).unwrap())
    }
}

impl Ord for HttpDate {
    fn cmp(&self, other: &HttpDate) -> cmp::Ordering {
        SystemTime::from(*self).cmp(&SystemTime::from(*other))
    }
}

impl PartialOrd for HttpDate {
    fn partial_cmp(&self, other: &HttpDate) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn toint_1(x: u8) -> Result<u8, Error> {
    let result = x.wrapping_sub(b'0');
    if result < 10 {
        Ok(result)
    } else {
        Err(Error(()))
    }
}

fn toint_2(s: &[u8]) -> Result<u8, Error> {
    let high = s[0].wrapping_sub(b'0');
    let low = s[1].wrapping_sub(b'0');

    if high < 10 && low < 10 {
        Ok(high * 10 + low)
    } else {
        Err(Error(()))
    }
}

#[allow(clippy::many_single_char_names)]
fn toint_4(s: &[u8]) -> Result<u16, Error> {
    let a = u16::from(s[0].wrapping_sub(b'0'));
    let b = u16::from(s[1].wrapping_sub(b'0'));
    let c = u16::from(s[2].wrapping_sub(b'0'));
    let d = u16::from(s[3].wrapping_sub(b'0'));

    if a < 10 && b < 10 && c < 10 && d < 10 {
        Ok(a * 1000 + b * 100 + c * 10 + d)
    } else {
        Err(Error(()))
    }
}

fn parse_imf_fixdate(s: &[u8]) -> Result<HttpDate, Error> {
    // Example: `Sun, 06 Nov 1994 08:49:37 GMT`
    if s.len() != 29 || &s[25..] != b" GMT" || s[16] != b' ' || s[19] != b':' || s[22] != b':' {
        return Err(Error(()));
    }
    Ok(HttpDate {
        sec: toint_2(&s[23..25])?,
        min: toint_2(&s[20..22])?,
        hour: toint_2(&s[17..19])?,
        day: toint_2(&s[5..7])?,
        mon: match &s[7..12] {
            b" Jan " => 1,
            b" Feb " => 2,
            b" Mar " => 3,
            b" Apr " => 4,
            b" May " => 5,
            b" Jun " => 6,
            b" Jul " => 7,
            b" Aug " => 8,
            b" Sep " => 9,
            b" Oct " => 10,
            b" Nov " => 11,
            b" Dec " => 12,
            _ => return Err(Error(())),
        },
        year: toint_4(&s[12..16])?,
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
    let mut year = u16::from(toint_2(&s[7..9])?);
    if year < 70 {
        year += 2000;
    } else {
        year += 1900;
    }
    Ok(HttpDate {
        sec: toint_2(&s[16..18])?,
        min: toint_2(&s[13..15])?,
        hour: toint_2(&s[10..12])?,
        day: toint_2(&s[0..2])?,
        mon: match &s[2..7] {
            b"-Jan-" => 1,
            b"-Feb-" => 2,
            b"-Mar-" => 3,
            b"-Apr-" => 4,
            b"-May-" => 5,
            b"-Jun-" => 6,
            b"-Jul-" => 7,
            b"-Aug-" => 8,
            b"-Sep-" => 9,
            b"-Oct-" => 10,
            b"-Nov-" => 11,
            b"-Dec-" => 12,
            _ => return Err(Error(())),
        },
        year,
        wday,
    })
}

fn parse_asctime(s: &[u8]) -> Result<HttpDate, Error> {
    // Example: `Sun Nov  6 08:49:37 1994`
    if s.len() != 24 || s[10] != b' ' || s[13] != b':' || s[16] != b':' || s[19] != b' ' {
        return Err(Error(()));
    }
    Ok(HttpDate {
        sec: toint_2(&s[17..19])?,
        min: toint_2(&s[14..16])?,
        hour: toint_2(&s[11..13])?,
        day: {
            let x = &s[8..10];
            {
                if x[0] == b' ' {
                    toint_1(x[1])
                } else {
                    toint_2(x)
                }
            }?
        },
        mon: match &s[4..8] {
            b"Jan " => 1,
            b"Feb " => 2,
            b"Mar " => 3,
            b"Apr " => 4,
            b"May " => 5,
            b"Jun " => 6,
            b"Jul " => 7,
            b"Aug " => 8,
            b"Sep " => 9,
            b"Oct " => 10,
            b"Nov " => 11,
            b"Dec " => 12,
            _ => return Err(Error(())),
        },
        year: toint_4(&s[20..24])?,
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
