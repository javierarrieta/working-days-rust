extern crate chrono;

use chrono::prelude::*;

fn main() {
    let year = 2017;
    let iter = DateIter { cur: UTC.ymd(year, 1, 1) };
    for days in iter {
        println!("{}", days);
    }
}

struct DateIter {
    cur: Date<UTC>
}

impl Iterator for DateIter {
    type Item = Date<UTC>;

    fn next(&mut self) -> Option<Date<UTC>> {
        let succ = self.cur.succ();
        if succ.year() > self.cur.year() {
            None
        } else {
            self.cur = succ;
            Some(succ)
        }
    }
}