// Copyright 2019 Boyu Yang<yangby@cryptape.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use cita_web3::{
    api::Cita,
    types,
    web3::{futures::Future, transports::Http, ErrorKind as Web3ErrorKind, Web3},
};

pub const JSONRPC_METHODS: &[&str] = &[
    "peerCount",
    "blockNumber",
    "getMetaData",
    "sendRawTransaction",
];

type Web3Http = Web3<Http>;
type RespStat = (Duration, (usize, usize, usize));

pub fn get_func_core<'a>(category: &str) -> Box<Fn(&'a Web3Http) -> RespStat> {
    match category {
        "peerCount" => Box::new(peer_count),
        "blockNumber" => Box::new(block_number),
        "getMetaData" => Box::new(get_meta_data),
        "sendRawTransaction" => gen_send_raw_transaction(),
        _ => unreachable!(),
    }
}

macro_rules! send_request {
    ($web3:ident, $param:ident) => {{
        let now = Instant::now();
        let result = $web3.api::<Cita<Http>>().call($param).wait();
        let dur = now.elapsed();
        let nums = match result {
            Ok(_) => (1, 0, 0),
            Err(err) => match *err.kind() {
                Web3ErrorKind::Rpc(_)
                | Web3ErrorKind::InvalidResponse(_)
                | Web3ErrorKind::Decoder(_) => (0, 1, 0),
                _ => (0, 1, 0),
            },
        };
        (dur, nums)
    }};
}

fn peer_count(web3: &Web3Http) -> RespStat {
    let param = types::request::PeerCountParams::new();
    send_request!(web3, param)
}

fn block_number(web3: &Web3Http) -> RespStat {
    let param = types::request::BlockNumberParams::new();
    send_request!(web3, param)
}

fn get_meta_data(web3: &Web3Http) -> RespStat {
    let block_number = types::rpctypes::BlockNumber::latest();
    let param = types::request::GetMetaDataParams::new(block_number);
    send_request!(web3, param)
}

fn gen_send_raw_transaction<'a>() -> Box<Fn(&'a Web3Http) -> RespStat> {
    use cita_crypto::{CreateKey, KeyPair, PrivKey};
    use cita_types::{H256, U256};
    use libproto::{blockchain::Transaction, TryInto};
    use std::str::FromStr;
    use std::thread;

    fn import_keypair(keystr: &str) -> KeyPair {
        if &keystr[0..2] != "0x" {
            panic!("Please use 0x-prefix for private key.")
        }
        let privkey = PrivKey::from_str(&keystr[2..])
            .map_err(|err| panic!("failed to parse private key {}: {}", keystr, err))
            .unwrap();
        KeyPair::from_privkey(privkey)
            .map_err(|err| panic!("failed to load private key to keypair: {}", err))
            .unwrap()
    }

    fn generate_nonce() -> String {
        use rand::distributions::Alphanumeric;
        use rand::{thread_rng, Rng};
        let mut rng = thread_rng();
        rng.sample_iter(&Alphanumeric).take(12).collect()
    }

    fn fetch_chain_id(web3: &Web3Http) -> U256 {
        let block_number = types::rpctypes::BlockNumber::latest();
        let param = types::request::GetMetaDataParams::new(block_number);
        let metadata = web3.api::<Cita<Http>>().call(param).wait().unwrap();
        metadata.chain_id_v1.into()
    }

    fn fetch_height(web3: &Web3Http) -> u64 {
        let param = types::request::BlockNumberParams::new();
        let height: U256 = web3.api::<Cita<Http>>().call(param).wait().unwrap().into();
        height.low_u64()
    }

    let chain_id = Arc::new(RwLock::new(None));
    let height = Arc::new(RwLock::new(0u64));
    let height_reset = height.clone();

    let wait_secs = Duration::from_secs(120);
    thread::spawn(move || loop {
        thread::sleep(wait_secs);
        *height_reset.write().unwrap() = 0;
    });

    let keystr = "0x1000000000000000000000000000000000000000000000000000000000000000";
    let keypair = import_keypair(&keystr);

    let closure = move |web3| {
        let chain_id = {
            let id = { *chain_id.read().unwrap() };
            id.unwrap_or_else(|| {
                let id_new = fetch_chain_id(web3);
                {
                    *chain_id.write().unwrap() = Some(id_new);
                }
                id_new
            })
        };
        let height = {
            let h = { *height.read().unwrap() };
            if h == 0 {
                let h_new = fetch_height(web3);
                {
                    *height.write().unwrap() = h_new;
                }
                h_new
            } else {
                h
            }
        };
        let nonce = generate_nonce();
        let utx = {
            let mut tx = Transaction::new();
            tx.set_chain_id_v1(H256::from(chain_id).to_vec());
            tx.set_valid_until_block(height + 100);
            tx.set_nonce(nonce);
            tx.set_quota(1_000_000);
            tx.set_version(1);
            tx.set_data(Vec::new());
            tx.set_value(vec![0u8; 32]);
            tx.sign(*keypair.privkey()).take_transaction_with_sig()
        };

        let tx_bytes: Vec<u8> = utx.try_into().unwrap();
        let param = types::request::SendRawTransactionParams::new(tx_bytes.into());
        send_request!(web3, param)
    };
    Box::new(closure)
}
