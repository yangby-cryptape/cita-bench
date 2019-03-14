// Copyright 2019 Boyu Yang<yangby@cryptape.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::boxed::Box;
use std::fmt;
use std::marker::{Send, Sync};
use std::ops::Fn;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Barrier, Mutex, RwLock};
use std::thread;
use std::time::Instant;

use crate::config::{AppConfig, Node};
use crate::report::{CaptainReport, GeneralReport, SimpleReport, SoldierReport};

pub struct Mission<T> {
    pub data: T,
    pub doing: Box<Fn(&Node, &T) -> SimpleReport>,
}

unsafe impl<T> Send for Mission<T> {}
unsafe impl<T> Sync for Mission<T> {}

impl<T> Mission<T> {
    pub fn start(&self, node: &Node) -> SimpleReport {
        (self.doing)(node, &self.data)
    }
}

impl<T> fmt::Debug for Mission<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "A mission.")
    }
}

#[derive(Debug)]
struct FullMission<T>
where
    T: Send + Sync,
{
    mission: Mission<T>,
    category: String,
    node: Vec<Node>,
    captain_num: usize, // how many teams
    soldier_num: usize, // the size of team
}

impl<T> FullMission<T>
where
    T: Send + Sync,
{
    fn new(m: Mission<T>, c: &AppConfig) -> Self {
        let captain_num = c.node.len();
        let soldier_num = c.thread;
        Self {
            mission: m,
            category: c.category.clone(),
            node: c.node.clone(),
            captain_num,
            soldier_num,
        }
    }

    fn run(&self, id: usize) -> SimpleReport {
        self.mission.start(&self.node[id].clone())
    }
}

pub fn generate_report<T: 'static>(config: AppConfig, mission: Mission<T>) -> GeneralReport
where
    T: Send + Sync,
{
    log::debug!("Running for: {}", config);
    let full_mission = Arc::new(RwLock::new(FullMission::new(mission, &config)));
    assign_task_to_general(full_mission)
}

fn assign_task_to_general<T: 'static>(task: Arc<RwLock<FullMission<T>>>) -> GeneralReport
where
    T: Send + Sync,
{
    log::debug!("General has accepted the mission ...");
    let countdown_max = {
        let task = task.read().unwrap();
        task.captain_num * task.soldier_num + 1
    };
    let countdown = Arc::new(Barrier::new(countdown_max));
    let now = Instant::now();
    let mut captain_team = vec![];
    let (category, captain_num) = {
        let task = task.read().unwrap();
        (task.category.clone(), task.captain_num)
    };
    let report = Arc::new(Mutex::new(GeneralReport::new(category, captain_num)));
    {
        for n in task.read().unwrap().node.iter() {
            report.lock().unwrap().node.push(n.clone())
        }
    }
    for captain_id in 0..captain_num {
        let task = task.clone();
        let report = report.clone();
        let countdown = countdown.clone();
        let captain = thread::spawn(move || {
            assign_task_to_captain(task, report, captain_id, countdown);
        });
        captain_team.push(captain);
    }
    {
        log::trace!("General is waiting for all soldiers to be ready.");
        countdown.clone().wait();
    }
    {
        report.lock().unwrap().ready_tm = now.elapsed();
        log::debug!("Soldiers are ready to carry out their task.");
    }
    for captain in captain_team {
        captain.join().unwrap();
    }
    let mut report_original = Arc::try_unwrap(report).ok().unwrap().into_inner().unwrap();
    report_original.cost_tm = now.elapsed() - report_original.ready_tm;
    log::debug!("General has finished his task.");
    report_original
}

fn assign_task_to_captain<T: 'static>(
    task: Arc<RwLock<FullMission<T>>>,
    report: Arc<Mutex<GeneralReport>>,
    captain_id: usize,
    countdown: Arc<Barrier>,
) where
    T: Send + Sync,
{
    log::trace!("Captain#{} has accepted a task ...", captain_id);
    let now = Instant::now();
    let soldier_num = { task.read().unwrap().soldier_num };
    let (tx, rx): (Sender<SoldierReport>, Receiver<SoldierReport>) = channel();
    for soldier_id in 0..soldier_num {
        let task = task.clone();
        let sender = tx.clone();
        let countdown = countdown.clone();

        thread::spawn(move || {
            assign_task_to_soldier(task, sender, captain_id, soldier_id, countdown);
        });
    }
    let mut subreport = CaptainReport::new(captain_id, soldier_num);
    subreport.ready_tm = now.elapsed();
    for _ in 0..soldier_num {
        subreport.soldier_report.push(rx.recv().unwrap());
    }
    subreport.cost_tm = now.elapsed();
    {
        report.lock().unwrap().captain_report.push(subreport);
    }
    log::trace!("Captain#{} has finished his task.", captain_id);
}

fn assign_task_to_soldier<T>(
    task: Arc<RwLock<FullMission<T>>>,
    sender: Sender<SoldierReport>,
    captain_id: usize,
    soldier_id: usize,
    countdown: Arc<Barrier>,
) where
    T: Send + Sync,
{
    log::trace!(
        "Soldier#{}-{} has accepted a task ...",
        captain_id,
        soldier_id
    );
    let now = Instant::now();
    countdown.wait();
    log::trace!(
        "Soldier#{}-{} is doing his task ...",
        captain_id,
        soldier_id
    );
    let ready_tm = now.elapsed();
    let result = { task.read().unwrap().run(captain_id) };
    let cost_tm = now.elapsed();
    {
        sender
            .send(SoldierReport::new(soldier_id, ready_tm, cost_tm, result))
            .unwrap();
    }
    log::trace!(
        "Soldier#{}-{} has finished his task.",
        captain_id,
        soldier_id
    );
}
