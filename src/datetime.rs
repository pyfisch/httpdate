use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use Error;

#[derive(Debug)]
pub struct DateTime {
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
    /// 1000...9999
    year: u16,
    /// 1...7
    wday: u8,
}

impl From<SystemTime> for DateTime {
    fn from(v: SystemTime) -> DateTime {
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

        DateTime {
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

impl From<DateTime> for SystemTime {
    fn from(v: DateTime) -> SystemTime {
        let leap_years = ((v.year - 1) - 1968) / 4
            - ((v.year - 1) - 1900) / 100
            + ((v.year - 1) - 1600) / 400;
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
        UNIX_EPOCH + Duration::from_secs(v.sec as u64
            + v.min as u64 * 60
            + v.hour as u64 * 3600
            + days * 86400)
    }
}

impl FromStr for DateTime {
    type Err = Error;

    fn from_str(s: &str) -> Result<DateTime, Error> {
        let s = s.trim();
        parse_imf_fixdate(s)
        .or_else(|_| parse_rfc850_date(s))
        .or_else(|_| parse_asctime(s))
    }
}

impl Display for DateTime {
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

fn parse_imf_fixdate(s: &str) -> Result<DateTime, Error> {
    if s.len() != 29 || &s[25..] != " GMT" {
        return Err(Error(()));
    }
    Ok(DateTime {
        sec: try!(s[23..25].parse()),
        min: try!(s[20..22].parse()),
        hour: try!(s[17..19].parse()),
        day: try!(s[5..7].parse()),
        mon: match &s[7..12] {
            " Jan " => 1,
            " Feb " => 2,
            " Mar " => 3,
            " Apr " => 4,
            " Mai " => 5,
            " Jun " => 6,
            " Jul " => 7,
            " Aug " => 8,
            " Sep " => 9,
            " Oct " => 10,
            " Nov " => 11,
            " Dec " => 12,
            _ => return Err(Error(())),
        },
        year: try!(s[12..16].parse()),
        wday: match &s[..5] {
            "Mon, " => 1,
            "Tue, " => 2,
            "Wed, " => 3,
            "Thu, " => 4,
            "Fri, " => 5,
            "Sat, " => 6,
            "Sun, " => 7,
            _ => return Err(Error(())),
        },
    })
}

fn parse_rfc850_date(s: &str) -> Result<DateTime, Error> {
    if !s.ends_with(" GMT") {
        return Err(Error(()))
    }
    let (wday, s) = if s.starts_with("Monday, ") { (1, &s[8..]) }
        else if s.starts_with("Tuesday, ") { (2, &s[9..]) }
        else if s.starts_with("Wednesday, ") { (3, &s[11..]) }
        else if s.starts_with("Thursday, ") { (4, &s[10..]) }
        else if s.starts_with("Friday, ") { (5, &s[8..]) }
        else if s.starts_with("Saturday, ") { (6, &s[10..]) }
        else if s.starts_with("Sunday, ") { (7, &s[8..]) }
        else { return Err(Error(())); };
    if s.len() != 22 {
        return Err(Error(()));
    }
    let mut year = try!(s[7..9].parse::<u16>());
    if year < 70 {
        year += 2000;
    } else {
        year += 1900;
    }
    Ok(DateTime {
        sec: try!(s[16..18].parse()),
        min: try!(s[13..15].parse()),
        hour: try!(s[10..12].parse()),
        day: try!(s[0..2].parse()),
        mon: match &s[2..7] {
            "-Jan-" => 1,
            "-Feb-" => 2,
            "-Mar-" => 3,
            "-Apr-" => 4,
            "-Mai-" => 5,
            "-Jun-" => 6,
            "-Jul-" => 7,
            "-Aug-" => 8,
            "-Sep-" => 9,
            "-Oct-" => 10,
            "-Nov-" => 11,
            "-Dec-" => 12,
            _ => return Err(Error(())),
        },
        year: year,
        wday: wday,
    })
}

fn parse_asctime(s: &str) -> Result<DateTime, Error> {
    if s.len() != 24 {
        return Err(Error(()));
    }
    Ok(DateTime {
        sec: try!(s[17..19].parse()),
        min: try!(s[14..16].parse()),
        hour: try!(s[11..13].parse()),
        day: try!(s[8..10].trim_left().parse()),
        mon: match &s[4..8] {
            "Jan " => 1,
            "Feb " => 2,
            "Mar " => 3,
            "Apr " => 4,
            "Mai " => 5,
            "Jun " => 6,
            "Jul " => 7,
            "Aug " => 8,
            "Sep " => 9,
            "Oct " => 10,
            "Nov " => 11,
            "Dec " => 12,
            _ => return Err(Error(())),
        },
        year: try!(s[20..24].parse()),
        wday: match &s[0..4] {
            "Mon " => 1,
            "Tue " => 2,
            "Wed " => 3,
            "Thu " => 4,
            "Fri " => 5,
            "Sat " => 6,
            "Sun " => 7,
            _ => return Err(Error(())),
        },
    })
}

fn is_leap_year(y: u16) -> bool {
    y % 4 == 0 && (!(y % 100 == 0) || y % 400 == 0)
}
