// Copyright 2019 Boyu Yang<yangby@cryptape.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{env, fmt, str};

use clap::{App, Arg, ArgMatches};

use crate::transaction::JSONRPC_METHODS;

const APPNAME: &str = "CITA Bench";
const VERNUM: &str = "0.0.1";
const LOG_LEVEL_ENV: &str = "APP_LOG_LEVEL";

#[derive(Debug, Clone)]
pub struct Node {
    pub host: String,
    pub port: u16,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.host, self.port)
    }
}

impl str::FromStr for Node {
    type Err = ParseNodeError;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = s.split(':').collect::<Vec<&str>>();
        if v.len() != 2 {
            return Err(ParseNodeError {
                address: s.to_string(),
            });
        }
        let h = v[0].trim();
        let p = v[1].parse::<u16>();
        if h.is_empty() || p.is_err() {
            return Err(ParseNodeError {
                address: s.to_string(),
            });
        }
        Ok(Node {
            host: h.to_string(),
            port: p.unwrap(),
        })
    }
}

pub struct ParseNodeError {
    address: String,
}

impl fmt::Display for ParseNodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (&format!("the address [{}] for node is malformed", &self.address)).fmt(f)
    }
}

pub struct AppConfig {
    pub node: Vec<Node>,
    pub protocol: String,
    pub thread: usize,
    pub amount: usize,
    pub interval: usize,
    pub category: String,
}

impl<'a> From<&'a ArgMatches<'a>> for AppConfig {
    fn from(matches: &'a ArgMatches) -> Self {
        let node = values_t!(matches, "node", Node).unwrap_or_else(|e| e.exit());
        let protocol = value_t!(matches, "protocol", String).unwrap_or_else(|e| e.exit());
        let thread = value_t!(matches, "thread", usize).unwrap_or_else(|e| e.exit());
        let amount = value_t!(matches, "amount", usize).unwrap_or_else(|e| e.exit());
        let interval = value_t!(matches, "interval", usize).unwrap_or_else(|e| e.exit());
        let category = value_t!(matches, "category", String).unwrap_or_else(|e| e.exit());
        Self {
            node,
            protocol,
            thread,
            amount,
            interval,
            category,
        }
    }
}

impl fmt::Display for AppConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ret = "\nAppConfig: {{\n".to_string();
        ret.push_str(&format!("    node[{}]:\n", self.node.len()));
        for node in self.node.iter() {
            ret.push_str(&format!("        {}\n", node));
        }
        ret.push_str(&format!("    thread: {}\n", self.thread));
        ret.push_str(&format!("    amount: {}\n", self.amount));
        ret.push_str(&format!("    interval: {}\n", self.interval));
        ret.push_str(&format!("    category: {}\n", self.category));
        ret += "}}\n";
        write!(f, "{}", ret)
    }
}

pub fn build_commandline<'a>() -> App<'a, 'a> {
    App::new(APPNAME)
        .version(VERNUM)
        .author("Boyu Yang <yangby@cryptape.com>")
        .about("Benchmark CITA.")
        .arg(
            Arg::with_name("quiet")
                .long("quiet")
                .short("q")
                .conflicts_with("verbose")
                .help("No logs printed to stdout. Only print the result."),
        )
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .short("v")
                .multiple(true)
                .help(
                    "Use verbose [Warn] output \
                     (support -vv [Info] / -vvv [Debug] / -vvvv [Trace] \
                     / -vvvvv.. [More Logs]).",
                ),
        )
        .arg(
            Arg::with_name("node")
                .long("node")
                .short("N")
                .required(true)
                .takes_value(true)
                .multiple(true)
                .value_delimiter(",")
                .help("Set the host:port[,host:port[...]] of nodes to send transactions."),
        )
        .arg(
            Arg::with_name("protocol")
                .long("protocol")
                .short("p")
                .takes_value(true)
                .possible_value("http")
                .possible_value("https")
                .default_value("http")
                .help("Set the protocol."),
        )
        .arg(
            Arg::with_name("thread")
                .long("thread")
                .short("t")
                .takes_value(true)
                .default_value("1")
                .help("Set the number of threads for each node."),
        )
        .arg(
            Arg::with_name("amount")
                .long("amount")
                .short("a")
                .takes_value(true)
                .default_value("1")
                .help("Set the amount of messages for each node. 0 means infinite."),
        )
        .arg(
            Arg::with_name("interval")
                .long("interval")
                .short("i")
                .takes_value(true)
                .default_value("1000")
                .help("Wait interval millisecond between sending each request. 0 means no wait."),
        )
        .arg(
            Arg::with_name("category")
                .long("category")
                .short("c")
                .takes_value(true)
                .possible_values(JSONRPC_METHODS)
                .default_value(JSONRPC_METHODS[0])
                .help("Set the category of messages to send."),
        )
}

fn progname() -> String {
    env::args()
        .next()
        .as_ref()
        .map(::std::path::Path::new)
        .and_then(::std::path::Path::file_name)
        .and_then(::std::ffi::OsStr::to_str)
        .or(Some("main"))
        .map(String::from)
        .expect("get program name failed")
}

fn init_logger(matches: &ArgMatches) {
    let pkg_name = str::replace(env!("CARGO_PKG_NAME"), "-", "_");
    let prog_name = str::replace(progname().as_str(), "-", "_");
    let log_lv = if matches.is_present("quiet") {
        "off".to_owned()
    } else {
        match matches.occurrences_of("verbose") {
            0 => "error".to_owned(),
            1 => format!("error,{}=warn,{}=warn", pkg_name, prog_name),
            2 => format!("error,{}=info,{}=info", pkg_name, prog_name),
            3 => format!("error,{}=debug,{}=debug", pkg_name, prog_name),
            4 => format!("error,{}=trace,{}=trace", pkg_name, prog_name),
            5 => format!("warn,{}=trace,{}=trace", pkg_name, prog_name),
            6 => format!("info,{}=trace,{}=trace", pkg_name, prog_name),
            7 => format!("debug,{}=trace,{}=trace", pkg_name, prog_name),
            _ => "trace".to_owned(),
        }
    };
    env::set_var(LOG_LEVEL_ENV, log_lv.as_str());
    pretty_env_logger::try_init_timed_custom_env(LOG_LEVEL_ENV).unwrap();
}

pub fn parse_arguments(matches: ArgMatches) -> AppConfig {
    init_logger(&matches);
    AppConfig::from(&matches)
}
