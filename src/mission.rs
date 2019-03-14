// Copyright 2019 Boyu Yang<yangby@cryptape.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use crate::config::{AppConfig, Node};
use crate::execute::Mission;
use crate::report::SimpleReport;
use crate::transaction::get_func_core;

#[derive(Debug)]
pub struct MissionData {
    pub terminate: Arc<RwLock<bool>>,
    pub protocol: String,
    pub amount: usize,
    pub interval: usize,
    pub category: String,
}

impl MissionData {
    fn from(c: &AppConfig) -> Self {
        let terminate = Arc::new(RwLock::new(false));
        let terminate_clone = terminate.clone();
        ctrlc::set_handler(move || {
            log::warn!("Stopping since Ctrl+C ...");
            *terminate_clone.write().unwrap() = true;
        })
        .unwrap();
        Self {
            terminate,
            protocol: c.protocol.clone(),
            amount: c.amount,
            interval: c.interval,
            category: c.category.clone(),
        }
    }
}

fn doing(node: &Node, data: &MissionData) -> SimpleReport {
    let url = format!("{}://{}:{}", data.protocol, node.host, node.port);
    let amount = data.amount;
    let interval = data.interval;
    let mut count = 0;
    let mut report = SimpleReport::new();
    let wait_millis = Duration::from_millis(data.interval as u64);
    let (_eloop, transport) = cita_web3::web3::transports::Http::new(url.as_str()).unwrap();
    let web3 = cita_web3::web3::Web3::new(transport);
    let func_core = get_func_core(&data.category);
    loop {
        if *data.terminate.read().unwrap() || (amount != 0 && count == amount) {
            break;
        }
        count += 1;

        let (dur, nums) = func_core(&web3);

        report.add(dur, nums);
        if interval != 0 {
            thread::sleep(wait_millis);
        }
    }
    report
}

pub fn generate_mission(config: &AppConfig) -> Mission<MissionData> {
    Mission {
        data: MissionData::from(config),
        doing: Box::new(doing),
    }
}
