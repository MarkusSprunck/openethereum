// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

//! Client tests of tracing

use block::*;
use client::{BlockChainClient, Client, ClientConfig, *};
use crypto::publickey::KeyPair;
use ethereum_types::{Address, U256};
use hash::keccak;
use io::*;
use miner::Miner;
use spec::*;
use std::{str::FromStr, sync::Arc};
use test_helpers::{self, get_temp_state_db};
use trace::{trace::Action::Reward, LocalizedTrace, RewardType};
use types::{
    header::Header,
    transaction::{Action, Transaction, TypedTransaction},
    view,
    views::BlockView,
};
use verification::queue::kind::blocks::Unverified;

#[test]
fn can_trace_block_and_uncle_reward() {
    let db = test_helpers::new_db();
    let spec = Spec::new_test_with_reward();
    let engine = &*spec.engine;

    // Create client
    let mut client_config = ClientConfig::default();
    client_config.tracing.enabled = true;
    let client = Client::new(
        client_config,
        &spec,
        db,
        Arc::new(Miner::new_for_tests(&spec, None)),
        IoChannel::disconnected(),
    )
    .unwrap();

    // Create test data:
    // genesis
    //    |
    // root_block
    //    |
    // parent_block
    //    |
    // block with transaction and uncle

    let genesis_header = spec.genesis_header();
    let genesis_gas = *genesis_header.gas_limit();

    let mut db = spec
        .ensure_db_good(get_temp_state_db(), &Default::default())
        .unwrap();
    let mut rolling_timestamp = 40;
    let mut last_hashes = vec![];
    let mut last_header = genesis_header.clone();
    last_hashes.push(last_header.hash());

    let kp = KeyPair::from_secret_slice(keccak("").as_bytes()).unwrap();
    let author = kp.address();

    // Add root block first
    let mut root_block = OpenBlock::new(
        engine,
        Default::default(),
        false,
        db,
        &last_header,
        Arc::new(last_hashes.clone()),
        author,
        (3141562.into(), 31415620.into()),
        vec![],
        false,
        None,
    )
    .unwrap();
    rolling_timestamp += 10;
    root_block.set_timestamp(rolling_timestamp);

    let root_block = root_block
        .close_and_lock()
        .unwrap()
        .seal(engine, vec![])
        .unwrap();

    if let Err(e) = client.import_block(
        Unverified::from_rlp(root_block.rlp_bytes(), spec.params().eip1559_transition).unwrap(),
    ) {
        panic!(
            "error importing block which is valid by definition: {:?}",
            e
        );
    }

    last_header =
        view!(BlockView, &root_block.rlp_bytes()).header(spec.params().eip1559_transition);
    let root_header = last_header.clone();
    db = root_block.drain().state.drop().1;

    last_hashes.push(last_header.hash());

    // Add parent block
    let mut parent_block = OpenBlock::new(
        engine,
        Default::default(),
        false,
        db,
        &last_header,
        Arc::new(last_hashes.clone()),
        author,
        (3141562.into(), 31415620.into()),
        vec![],
        false,
        None,
    )
    .unwrap();
    rolling_timestamp += 10;
    parent_block.set_timestamp(rolling_timestamp);

    let parent_block = parent_block
        .close_and_lock()
        .unwrap()
        .seal(engine, vec![])
        .unwrap();

    if let Err(e) = client.import_block(
        Unverified::from_rlp(parent_block.rlp_bytes(), spec.params().eip1559_transition).unwrap(),
    ) {
        panic!(
            "error importing block which is valid by definition: {:?}",
            e
        );
    }

    last_header =
        view!(BlockView, &parent_block.rlp_bytes()).header(spec.params().eip1559_transition);
    db = parent_block.drain().state.drop().1;

    last_hashes.push(last_header.hash());

    // Add testing block with transaction and uncle
    let mut block = OpenBlock::new(
        engine,
        Default::default(),
        true,
        db,
        &last_header,
        Arc::new(last_hashes.clone()),
        author,
        (3141562.into(), 31415620.into()),
        vec![],
        false,
        None,
    )
    .unwrap();
    rolling_timestamp += 10;
    block.set_timestamp(rolling_timestamp);

    let mut n = 0;
    for _ in 0..1 {
        block
            .push_transaction(
                TypedTransaction::Legacy(Transaction {
                    nonce: n.into(),
                    gas_price: 10000.into(),
                    gas: 100000.into(),
                    action: Action::Create,
                    data: vec![],
                    value: U256::zero(),
                })
                .sign(kp.secret(), Some(spec.network_id())),
                None,
            )
            .unwrap();
        n += 1;
    }

    let mut uncle = Header::new();
    let uncle_author = Address::from_str("ef2d6d194084c2de36e0dabfce45d046b37d1106").unwrap();
    uncle.set_author(uncle_author);
    uncle.set_parent_hash(root_header.hash());
    uncle.set_gas_limit(genesis_gas);
    uncle.set_number(root_header.number() + 1);
    uncle.set_timestamp(rolling_timestamp);
    block.push_uncle(uncle).unwrap();

    let block = block
        .close_and_lock()
        .unwrap()
        .seal(engine, vec![])
        .unwrap();

    let res = client.import_block(
        Unverified::from_rlp(block.rlp_bytes(), spec.params().eip1559_transition).unwrap(),
    );
    if res.is_err() {
        panic!("error importing block: {:#?}", res.err().unwrap());
    }

    block.drain();
    client.flush_queue();
    client.import_verified_blocks();

    // Test0. Check overall filter
    let filter = TraceFilter {
        range: (BlockId::Number(1)..BlockId::Number(3)),
        from_address: vec![],
        to_address: vec![],
        after: None,
        count: None,
    };

    let traces = client.filter_traces(filter);
    assert!(traces.is_some(), "Filtered traces should be present");
    let traces_vec = traces.unwrap();
    let block_reward_traces: Vec<LocalizedTrace> = traces_vec
        .clone()
        .into_iter()
        .filter(|trace| match (trace).action {
            Reward(ref a) => a.reward_type == RewardType::Block,
            _ => false,
        })
        .collect();
    assert_eq!(block_reward_traces.len(), 3);
    let uncle_reward_traces: Vec<LocalizedTrace> = traces_vec
        .clone()
        .into_iter()
        .filter(|trace| match (trace).action {
            Reward(ref a) => a.reward_type == RewardType::Uncle,
            _ => false,
        })
        .collect();
    assert_eq!(uncle_reward_traces.len(), 1);

    // Test1. Check block filter
    let traces = client.block_traces(BlockId::Number(3));
    client.shutdown();
    assert_eq!(traces.unwrap().len(), 3);
}
