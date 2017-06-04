extern crate chrono;
extern crate immutable_map;

use chrono::prelude::*;
use Weekday::*;
use immutable_map::{TreeMap, TreeSet};
use std::fmt::{Display, Result, Formatter};
use std::collections::HashSet;
use std::string::String;
use std::borrow::Borrow;

fn main() {
    let years: Vec<i32> = vec![2017, 2018, 2019, 2020, 2021];
    let time_off_days: u32 = 27;
    let mut extra_leave = HashSet::new();
    extra_leave.insert(Weekday::Fri);

    let counts_with_time_off: Vec<TreeMap<DayType, u32>> = years.iter().map(|year|
        number_days_year(*year as i32, time_off_days, &PublicHolidays::irish_holidays, &extra_leave)
    ).collect();

    let counts = counts_with_time_off.iter().fold(TreeMap::new(), fold_map_add);
    let num = years.len() as u32;
    println!("Years: {}", num);
    println!("Extra leave days: {:?}", extra_leave);
    println!("------------------------------");
    for (k, v) in counts.iter() {
        println!("{} -> {} ({} avg)", k, v, v / num);
    }
}

fn number_days_year(year: i32, time_off_days: u32, pub_holidays: &Fn(i32) -> PublicHolidays, extra_leave: &HashSet<Weekday>) -> TreeMap<DayType, u32> {
    let iter = YearIter::new(year);
    let hols = pub_holidays(year);
    let days = iter.map(|d| DayType::from_day(d, &hols, &extra_leave));
    let counts: TreeMap<DayType, u32> = days.fold(TreeMap::new(), DayType::in_fold);
    let workdays = counts.get(&DayType::Workday).unwrap() - time_off_days;
    counts.insert(DayType::Workday, workdays).insert(DayType::TimeOff, time_off_days)
}

fn fold_map_add(acc: TreeMap<DayType, u32>, m: &TreeMap<DayType, u32>) -> TreeMap<DayType, u32> {
    m.iter().fold(acc, |acc, (k, v)| acc.insert(*k, acc.get(k).unwrap_or(&0) + v))
}

fn str_to_weekday(s: &str) -> std::result::Result<Weekday, String> {
    match s.to_lowercase().borrow() {
        "mon" | "monday" => Ok(Mon),
        "tue" | "tuesday" => Ok(Tue),
        "wed" | "wednesday" => Ok(Wed),
        "thu" | "thursday" => Ok(Thu),
        "fri" | "friday" => Ok(Fri),
        "sat" | "saturday" => Ok(Sat),
        "sun" | "sunday" => Ok(Sun),
        _ => Err(String::from(s) + "is not a valid weekday")
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash, PartialOrd, Ord)]
pub enum DayType {
    Workday = 0,
    Weekend = 1,
    PublicHoliday = 2,
    TimeOff = 3,
    ExtraLeave = 4
}

impl DayType {
    pub fn from_day(d: Date<UTC>, hols: &PublicHolidays, extra_leave: &HashSet<Weekday>) -> DayType {
        let wd = d.weekday();
        if wd == Weekday::Sat || wd == Weekday::Sun {
            DayType::Weekend
        } else if hols.holidays.contains(&d) {
            DayType::PublicHoliday
        } else if extra_leave.contains(&(d.weekday())) {
            DayType::ExtraLeave
        } else {
            DayType::Workday
        }
    }

    pub fn to_str(&self) -> &str {
        match *self {
            DayType::Workday => "Workday",
            DayType::Weekend => "Weekend",
            DayType::PublicHoliday => "PublicHoliday",
            DayType::TimeOff => "TimeOff",
            DayType::ExtraLeave => "ExtraLeave",
        }
    }

    pub fn in_fold(m: TreeMap<DayType, u32>, d: DayType) -> TreeMap<DayType, u32> {
        let update = m.get(&d).map(|v| v + 1).unwrap_or(1);
        m.insert(d, update)
    }
}

impl Display for DayType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.to_str())
    }
}

struct YearIter {
    year: i32,
    cur: Option<Date<UTC>>,
}

impl YearIter {
    pub fn new(year: i32) -> YearIter {
        YearIter { cur: None, year: year }
    }
}

impl Iterator for YearIter {
    type Item = Date<UTC>;

    fn next(&mut self) -> Option<Date<UTC>> {
        let succ = self.cur.map(|v| v.succ()).unwrap_or(UTC.ymd(self.year, 1, 1));
        self.cur = Some(succ);
        if self.year == succ.year() {
            Some(succ)
        } else {
            None
        }
    }
}

pub struct PublicHolidays {
    holidays: TreeSet<Date<UTC>>
}

impl PublicHolidays {
    fn no_hols() -> PublicHolidays { PublicHolidays { holidays: TreeSet::new() } }

    fn next_weekday(d: Date<UTC>) -> Date<UTC> {
        match DayType::from_day(d, &PublicHolidays::no_hols(), &HashSet::new()) {
            DayType::Weekend => PublicHolidays::next_weekday(d.succ()),
            _ => d,
        }
    }

    fn on_weekday(d: Date<UTC>, w: &Weekday, f: &Fn(Date<UTC>) -> Date<UTC>) -> Date<UTC> {
        if &d.weekday() == w { d } else { PublicHolidays::on_weekday(f(d), w, f) }
    }

    fn forward(d: Date<UTC>) -> Date<UTC> { d.succ() }
    fn backward(d: Date<UTC>) -> Date<UTC> { d.pred() }

    pub fn irish_holidays(year: i32) -> PublicHolidays {
        let new_year = PublicHolidays::next_weekday(UTC.ymd(year, 1, 1));
        let paddys_day = PublicHolidays::next_weekday(UTC.ymd(year, 3, 17));
        let easter_mon = PublicHolidays::on_weekday(UTC.ymd(year, 4, 1), &Weekday::Mon, &PublicHolidays::forward);
        let may_day = PublicHolidays::on_weekday(UTC.ymd(year, 5, 1), &Weekday::Mon, &PublicHolidays::forward);
        let june_bh = PublicHolidays::on_weekday(UTC.ymd(year, 6, 1), &Weekday::Mon, &PublicHolidays::forward);
        let aug_bh = PublicHolidays::on_weekday(UTC.ymd(year, 8, 1), &Weekday::Mon, &PublicHolidays::forward);
        let oct_bh = PublicHolidays::on_weekday(UTC.ymd(year, 10, 31), &Weekday::Mon, &PublicHolidays::backward);
        let xmas_day = PublicHolidays::next_weekday(UTC.ymd(year, 12, 25));
        let stephens_day = PublicHolidays::next_weekday(xmas_day.succ());
        PublicHolidays {
            holidays: TreeSet::new()
                .insert(new_year)
                .insert(paddys_day)
                .insert(easter_mon)
                .insert(may_day)
                .insert(june_bh)
                .insert(aug_bh)
                .insert(oct_bh)
                .insert(xmas_day)
                .insert(stephens_day)
        }
    }
}