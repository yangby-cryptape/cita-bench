// Copyright 2019 Boyu Yang<yangby@cryptape.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate clap;

mod config;
mod execute;
mod mission;
mod report;
mod transaction;

use config::{build_commandline, parse_arguments};
use execute::generate_report;
use mission::generate_mission;

fn main() {
    let matches = build_commandline().get_matches();
    let config = parse_arguments(matches);
    let mission = generate_mission(&config);
    let report = generate_report(config, mission);
    report.print();
}
