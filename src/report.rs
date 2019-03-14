// Copyright 2019 Boyu Yang<yangby@cryptape.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io::{stdout, Write};
use std::time::Duration;

use tabwriter::TabWriter;

use crate::config::Node;

#[derive(Debug)]
pub struct GeneralReport {
    pub(crate) title: String,
    pub(crate) ready_tm: Duration,
    pub(crate) cost_tm: Duration,
    pub(crate) node: Vec<Node>,
    pub(crate) captain_report: Vec<CaptainReport>,
}

impl GeneralReport {
    pub fn new(title: String, sz: usize) -> Self {
        GeneralReport {
            title,
            ready_tm: Duration::new(0, 0),
            cost_tm: Duration::new(0, 0),
            node: Vec::<Node>::with_capacity(sz),
            captain_report: Vec::with_capacity(sz),
        }
    }

    pub fn print(&self) {
        println!("----    ----    ----    ----    ----    ----    ----    ----    ----    ----");
        println!("{:-24}Benchmark [{}]\n", "", self.title);
        let mut total_success_cnt = 0;
        let mut out = TabWriter::new(stdout());
        writeln!(
            out,
            "Node\tAmount\tThread\tSuccess\tFailure\tMissing\tSuccCostAvg (ms)"
        )
        .unwrap();
        for crpt in self.captain_report.iter() {
            let rpt = crpt.analyse();
            let node = &self.node[crpt.captain_id];
            write!(out, "{}:{}\t", node.host, node.port).unwrap();
            write!(
                out,
                "{}\t{}\t{}\t{}\t{}\t",
                rpt.success_cnt + rpt.failure_cnt + rpt.missing_cnt,
                crpt.soldier_report.len(),
                rpt.success_cnt,
                rpt.failure_cnt,
                rpt.missing_cnt
            )
            .unwrap();
            let success_tm = rpt.get_success_tm();
            let success_tm =
                success_tm.as_secs() as f64 * 1e3 + f64::from(success_tm.subsec_nanos()) * 1e-6;
            writeln!(out, "{:.6}", success_tm).unwrap();
            total_success_cnt += rpt.success_cnt;
        }
        writeln!(out).unwrap();
        out.flush().unwrap();
        let total_cost_tm =
            self.cost_tm.as_secs() as f64 * 1e3 + f64::from(self.cost_tm.subsec_nanos()) * 1e-6;
        let tps = total_success_cnt as f64 / (total_cost_tm / 1e3);
        println!("{:-24}Total Cost : {:12.3} ms", "", total_cost_tm);
        println!("{:-24}Total Succ : {:12} tx", "", total_success_cnt);
        println!("{:-24}    TPS    : {:12.3} tx/s", "", tps);
        println!("----    ----    ----    ----    ----    ----    ----    ----    ----    ----");
    }
}

#[derive(Debug)]
pub struct CaptainReport {
    captain_id: usize,
    pub(crate) ready_tm: Duration,
    pub(crate) cost_tm: Duration,
    pub(crate) soldier_report: Vec<SoldierReport>,
}

impl CaptainReport {
    pub fn new(id: usize, sz: usize) -> Self {
        CaptainReport {
            captain_id: id,
            ready_tm: Duration::new(0, 0),
            cost_tm: Duration::new(0, 0),
            soldier_report: Vec::with_capacity(sz),
        }
    }

    pub fn analyse(&self) -> SimpleReport {
        let mut rpt = SimpleReport::new();
        for srpt in self.soldier_report.iter() {
            rpt.add(
                srpt.success_tm_sum,
                (srpt.success_cnt, srpt.failure_cnt, srpt.missing_cnt),
            );
        }
        rpt
    }
}

#[derive(Debug)]
pub struct SoldierReport {
    soldier_id: usize,
    ready_tm: Duration,
    cost_tm: Duration,
    success_tm_sum: Duration,
    success_cnt: usize,
    failure_cnt: usize,
    missing_cnt: usize,
}

impl SoldierReport {
    pub fn new(id: usize, rt: Duration, ct: Duration, sr: SimpleReport) -> Self {
        SoldierReport {
            soldier_id: id,
            ready_tm: rt,
            cost_tm: ct,
            success_tm_sum: sr.success_tm_sum,
            success_cnt: sr.success_cnt,
            failure_cnt: sr.failure_cnt,
            missing_cnt: sr.missing_cnt,
        }
    }
}

#[derive(Debug)]
pub struct SimpleReport {
    success_tm_sum: Duration,
    success_cnt: usize,
    failure_cnt: usize,
    missing_cnt: usize,
}

impl SimpleReport {
    pub fn new() -> Self {
        SimpleReport {
            success_tm_sum: Duration::new(0, 0),
            success_cnt: 0,
            failure_cnt: 0,
            missing_cnt: 0,
        }
    }

    pub fn add(&mut self, st: Duration, c: (usize, usize, usize)) {
        self.success_tm_sum += st;
        self.success_cnt += c.0;
        self.failure_cnt += c.1;
        self.missing_cnt += c.2;
    }

    pub fn get_success_tm(&self) -> Duration {
        let cnt = self.success_cnt as u32;
        if cnt == 0 {
            self.success_tm_sum
        } else {
            self.success_tm_sum / cnt
        }
    }
}
