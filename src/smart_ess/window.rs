use chrono::{DateTime, Datelike, Local, Timelike, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::ops::{Sub};
use std::str::FromStr;

#[cfg(test)]
pub const ALL_WEEKDAYS: [Weekday; 7] = [
    Weekday::Monday,
    Weekday::Tuesday,
    Weekday::Wednesday,
    Weekday::Thursday,
    Weekday::Friday,
    Weekday::Saturday,
    Weekday::Sunday,
];

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl Weekday {
    pub fn days_from(from: &Weekday, to: &Weekday) -> u8 {
        let mut days = *to as i64 - *from as i64 % 7;
        if days < 0 {
            days += 7;
        }
        days as u8
    }
}

impl From<chrono::Weekday> for Weekday {
    fn from(d: chrono::Weekday) -> Self {
        match d {
            chrono::Weekday::Mon => Weekday::Monday,
            chrono::Weekday::Tue => Weekday::Tuesday,
            chrono::Weekday::Wed => Weekday::Wednesday,
            chrono::Weekday::Thu => Weekday::Thursday,
            chrono::Weekday::Fri => Weekday::Friday,
            chrono::Weekday::Sat => Weekday::Saturday,
            chrono::Weekday::Sun => Weekday::Sunday,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateError(pub String);

impl<ToStr: ToString> From<ToStr> for RateError {
    fn from(s: ToStr) -> Self {
        RateError(s.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RateWindow {
    pub start: RateTime,
    pub end: RateTime,
    pub days: Vec<Weekday>,
}

impl RateWindow {
    pub fn schedule(&self, from: DateTime<Utc>) -> Vec<RateWindowAbsolute> {
        let mut days = self.days.clone();
        days.sort();

        // we pick the day before `from` to catch rates which cross days
        let from_sch = from.sub(Duration::days(1));
        let from_weekday: Weekday = from_sch.weekday().into();
        let wrap = days.iter().filter(|d| **d < from_weekday);

        let mut ret: Vec<RateWindowAbsolute> = days
            .iter()
            .filter(|d| **d >= from_weekday)
            .chain(wrap)
            .map(|wd| {
                let days = Weekday::days_from(&from_weekday, &wd);
                let start_local = from_sch.date().with_timezone(&Local).and_hms(
                    self.start.hour as u32,
                    self.start.minute as u32,
                    0,
                ) + Duration::days(days as i64);
                let start_utc = start_local.with_timezone(&Utc);
                RateWindowAbsolute {
                    start: start_utc,
                    end: start_utc + Duration::minutes(self.period() as i64),
                }
            })
            .filter(|d| d.start >= from || d.is_inside(from))
            .collect();
        ret.sort_by(|a, b| a.start.cmp(&b.start));
        ret
    }

    /// Number of minutes in this window
    fn period(&self) -> i16 {
        let end_m = self.end.minute_of_day() as i16;
        let start_m = self.start.minute_of_day() as i16;
        let v = end_m - start_m;
        if v < 0 {
            1440 + v
        } else {
            v
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RateWindowAbsolute {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl RateWindowAbsolute {
    pub fn is_inside(&self, v: DateTime<Utc>) -> bool {
        self.start <= v && v <= self.end
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct RateTime {
    hour: u8,
    minute: u8,
}

impl RateTime {
    pub fn new(hour: u8, minute: u8) -> Result<Self, RateError> {
        if hour > 23 || minute > 59 {
            return Err(RateError("Invalid time range".to_owned()));
        }

        Ok(RateTime { hour, minute })
    }

    pub fn minute_of_day(&self) -> u16 {
        (self.hour as u16 * 60) + self.minute as u16
    }
}

impl FromStr for RateTime {
    type Err = RateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ss: Vec<&str> = s.split(":").collect();
        RateTime::new(u8::from_str(ss[0])?, u8::from_str(ss[1])?)
    }
}

impl From<DateTime<Utc>> for RateTime {
    fn from(dt: DateTime<Utc>) -> Self {
        RateTime {
            hour: dt.hour() as u8,
            minute: dt.minute() as u8,
        }
    }
}

impl Eq for RateTime {}

impl PartialEq for RateTime {
    fn eq(&self, other: &Self) -> bool {
        self.minute_of_day().eq(&other.minute_of_day())
    }
}

impl PartialOrd for RateTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.minute_of_day().partial_cmp(&other.minute_of_day())
    }
}

impl Ord for RateTime {
    fn cmp(&self, other: &Self) -> Ordering {
        self.minute_of_day().cmp(&other.minute_of_day())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn rate_time_from_str() {
        let nine_am = RateTime {
            hour: 9,
            minute: 59,
        };

        let test_nine_am = RateTime::from_str("09:59").unwrap();
        assert_eq!(&nine_am, &test_nine_am);

        assert_eq!(RateTime::from_str("24:00").is_err(), true);
        assert_eq!(RateTime::from_str("00:60").is_err(), true);
    }

    #[test]
    #[should_panic]
    fn rate_time_from_str_bad() {
        RateTime::from_str("hello").unwrap();
        RateTime::from_str("24").unwrap();
        RateTime::from_str("a:1").unwrap();
    }

    #[test]
    fn rate_next_from() {
        let a_monday = Local.ymd(2022, 04, 18).and_hms(8, 0, 0);
        let a_saturday = Local.ymd(2022, 04, 16).and_hms(8, 0, 0);
        let a_friday = Local.ymd(2022, 04, 22).and_hms(8, 59, 59);
        let a_sunday_inside = Local.ymd(2022, 04, 24).and_hms(16, 0, 0);

        let rate = RateWindow {
            start: RateTime::from_str("09:00").unwrap(),
            end: RateTime::from_str("16:59").unwrap(),
            days: vec![Weekday::Sunday, Weekday::Friday],
        };

        let sch = rate.schedule(a_monday.with_timezone(&Utc));
        let next = sch.first().unwrap();
        assert_eq!(Local.ymd(2022, 04, 22).and_hms(9, 0, 0), next.start);
        assert_eq!(Local.ymd(2022, 04, 22).and_hms(16, 59, 0), next.end);

        let sch = rate.schedule(a_saturday.with_timezone(&Utc));
        let next = sch.first().unwrap();
        assert_eq!(Local.ymd(2022, 04, 17).and_hms(9, 0, 0), next.start);

        let sch = rate.schedule(a_friday.with_timezone(&Utc));
        let next = sch.first().unwrap();
        assert_eq!(Local.ymd(2022, 04, 22).and_hms(9, 0, 0), next.start, "same day before schedule");

        let sch = rate.schedule(a_sunday_inside.with_timezone(&Utc));
        let next = sch.first().unwrap();
        assert_eq!(Local.ymd(2022, 04, 24).and_hms(9, 0, 0), next.start);
    }

    #[test]
    fn rate_never() {
        let rate = RateWindow {
            start: RateTime::from_str("09:00").unwrap(),
            end: RateTime::from_str("16:59").unwrap(),
            days: vec![],
        };

        let sch = rate.schedule(Utc::now());
        assert_eq!(None, sch.first());
    }

    #[test]
    fn rate_period() {
        let rate = RateWindow {
            start: RateTime::from_str("00:00").unwrap(),
            end: RateTime::from_str("00:01").unwrap(),
            days: vec![],
        };
        assert_eq!(1, rate.period());

        let rate = RateWindow {
            start: RateTime::from_str("23:00").unwrap(),
            end: RateTime::from_str("02:00").unwrap(),
            days: vec![],
        };
        assert_eq!(180, rate.period());

        let rate = RateWindow {
            start: RateTime::from_str("00:00").unwrap(),
            end: RateTime::from_str("23:59").unwrap(),
            days: vec![],
        };
        assert_eq!(23 * 60 + 59, rate.period());
    }

    #[test]
    fn rate_cross() {
        let rate = RateWindow {
            start: RateTime::from_str("23:00").unwrap(),
            end: RateTime::from_str("08:59").unwrap(),
            days: ALL_WEEKDAYS.into(),
        };

        let from = Local.ymd(2022, 05, 03).and_hms(2, 0, 0).with_timezone(&Utc);
        let sch = rate.schedule(from);
        let next = sch.first().unwrap();

        assert_eq!(next.start, Local.ymd(2022, 05, 02).and_hms(23, 0, 0));
    }

    #[test]
    fn weekday_days_from() {
        assert_eq!(Weekday::days_from(&Weekday::Monday, &Weekday::Tuesday), 1);
        assert_eq!(Weekday::days_from(&Weekday::Sunday, &Weekday::Monday), 1);
        assert_eq!(Weekday::days_from(&Weekday::Monday, &Weekday::Sunday), 6);
        assert_eq!(Weekday::days_from(&Weekday::Wednesday, &Weekday::Tuesday), 6);
        assert_eq!(Weekday::days_from(&Weekday::Sunday, &Weekday::Sunday), 0);
    }
}
