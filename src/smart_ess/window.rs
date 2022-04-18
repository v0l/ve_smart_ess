use std::cmp::Ordering;
use std::ops::Index;
use std::str::FromStr;
use chrono::{Datelike, DateTime, Timelike, Utc};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl From<chrono::Weekday> for Weekday {
    fn from(d: chrono::Weekday) -> Self {
        match d {
            chrono::Weekday::Mon => Weekday::Monday,
            chrono::Weekday::Tue => Weekday::Tuesday,
            chrono::Weekday::Wed => Weekday::Wednesday,
            chrono::Weekday::Thu => Weekday::Tuesday,
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
    pub fn next(&self) -> Option<DateTime<Utc>> {
        self.next_from(Utc::now())
    }

    pub fn next_from(&self, from: DateTime<Utc>) -> Option<DateTime<Utc>> {
        let mut days = self.days.clone();
        days.sort();

        let today: Weekday = from.weekday().into();
        let next_weekday = if days.contains(&today) {
            Some(&today)
        } else {
            let wrap = days.iter().filter(|d| d.lt(&&today));
            days.iter().filter(|d| d.gt(&&today)).chain(wrap).next()
        };

        None
    }

    pub fn inside(&self, time: &RateTime) -> bool {
        self.start.minute_of_day() <= time.minute_of_day() &&
            self.end.minute_of_day() >= time.minute_of_day()
    }

    fn cross_day(&self) -> bool {
        self.end < self.start
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RateTime {
    hour: u8,
    minute: u8,
}

impl RateTime {
    pub fn new(hour: u8, minute: u8) -> Result<Self, RateError> {
        if hour > 23 || minute > 59 {
            return Err(RateError("Invalid time range".to_owned()));
        }

        Ok(RateTime {
            hour,
            minute,
        })
    }

    pub fn active_at(time: RateTime) {}

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
        RateTime { hour: dt.hour() as u8, minute: dt.minute() as u8 }
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

    #[test]
    fn rate_time_from_str() {
        let nine_am = RateTime { hour: 9, minute: 59 };

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
    fn today() {
        let now = Utc::now();

        let rate = RateWindow {
            start: RateTime::from_str("09:00").unwrap(),
            end: RateTime::from_str("16:59").unwrap(),
            days: vec![now.weekday().into()],
        };

        let next = rate.next_from(now);
        assert_ne!(None, next);
    }

    #[test]
    fn tomorrow() {}

    #[test]
    fn few_days() {}

    #[test]
    fn rollover() {}

    #[test]
    fn never() {}
}