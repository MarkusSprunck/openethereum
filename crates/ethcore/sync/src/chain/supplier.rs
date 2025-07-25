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

use bytes::Bytes;

#[cfg(not(test))]
use devp2p::PAYLOAD_SOFT_LIMIT;
#[cfg(test)]
pub const PAYLOAD_SOFT_LIMIT: usize = 100_000;

use enum_primitive::FromPrimitive;
use ethereum_types::H256;
use network::{self, PeerId};
use parking_lot::RwLock;
use rlp::{Rlp, RlpStream};
use std::cmp;
use types::{ids::BlockId, BlockNumber};

use sync_io::SyncIo;

use super::{
    request_id::{prepend_request_id, strip_request_id, RequestId},
    sync_packet::{PacketInfo, SyncPacket, SyncPacket::*},
};

use super::{
    ChainSync, PacketProcessError, RlpResponseResult, SyncHandler, MAX_BODIES_TO_SEND,
    MAX_HEADERS_TO_SEND, MAX_RECEIPTS_HEADERS_TO_SEND,
};
use chain::MAX_NODE_DATA_TO_SEND;
use std::borrow::Borrow;

/// The Chain Sync Supplier: answers requests from peers with available data
pub struct SyncSupplier;

impl SyncSupplier {
    /// Dispatch incoming requests and responses
    // Take a u8 and not a SyncPacketId because this is the entry point
    // to chain sync from the outside world.
    pub fn dispatch_packet(
        sync: &RwLock<ChainSync>,
        io: &mut dyn SyncIo,
        peer: PeerId,
        packet_id: u8,
        data: &[u8],
    ) {
        if let Some(id) = SyncPacket::from_u8(packet_id) {
            let rlp_result = strip_request_id(data, sync.read().borrow(), &peer, &id);

            let result = match rlp_result {
                Ok((rlp, request_id)) => match id {
                    GetPooledTransactionsPacket => SyncSupplier::return_rlp(
                        io,
                        &rlp,
                        peer,
                        request_id,
                        SyncSupplier::return_pooled_transactions,
                        |e| format!("Error sending pooled transactions: {e:?}"),
                    ),

                    GetBlockBodiesPacket => SyncSupplier::return_rlp(
                        io,
                        &rlp,
                        peer,
                        request_id,
                        SyncSupplier::return_block_bodies,
                        |e| format!("Error sending block bodies: {e:?}"),
                    ),

                    GetBlockHeadersPacket => SyncSupplier::return_rlp(
                        io,
                        &rlp,
                        peer,
                        request_id,
                        SyncSupplier::return_block_headers,
                        |e| format!("Error sending block headers: {e:?}"),
                    ),

                    GetNodeDataPacket => SyncSupplier::return_rlp(
                        io,
                        &rlp,
                        peer,
                        request_id,
                        SyncSupplier::return_node_data,
                        |e| format!("Error sending node data: {e:?}"),
                    ),

                    GetReceiptsPacket => SyncSupplier::return_rlp(
                        io,
                        &rlp,
                        peer,
                        request_id,
                        SyncSupplier::return_receipts,
                        |e| format!("Error sending receipts: {e:?}"),
                    ),

                    GetSnapshotManifestPacket => SyncSupplier::return_rlp(
                        io,
                        &rlp,
                        peer,
                        request_id,
                        SyncSupplier::return_snapshot_manifest,
                        |e| format!("Error sending snapshot manifest: {e:?}"),
                    ),

                    GetSnapshotDataPacket => SyncSupplier::return_rlp(
                        io,
                        &rlp,
                        peer,
                        request_id,
                        SyncSupplier::return_snapshot_data,
                        |e| format!("Error sending snapshot data: {e:?}"),
                    ),

                    StatusPacket => {
                        sync.write().on_packet(io, peer, packet_id, data);
                        Ok(())
                    }
                    // Packets that require the peer to be confirmed
                    _ => {
                        if !sync.read().peers.contains_key(&peer) {
                            debug!(target: "sync", "Unexpected packet {} from unregistered peer: {}:{}", packet_id, peer, io.peer_version(peer));
                            return;
                        }
                        debug!(target: "sync", "{peer} -> Dispatching packet: {packet_id}");

                        match id {
                            ConsensusDataPacket => SyncHandler::on_consensus_packet(io, peer, &rlp),
                            TransactionsPacket => {
                                let res = {
                                    let sync_ro = sync.read();
                                    SyncHandler::on_peer_transactions(&sync_ro, io, peer, &rlp)
                                };
                                if res.is_err() {
                                    // peer sent invalid data, disconnect.
                                    io.disable_peer(peer);
                                    sync.write().deactivate_peer(io, peer);
                                }
                            }
                            _ => {
                                sync.write().on_packet(io, peer, packet_id, data);
                            }
                        }
                        Ok(())
                    }
                },
                Err(e) => Err(e.into()),
            };

            match result {
                Err(PacketProcessError::Decoder(e)) => {
                    debug!(target:"sync", "{peer} -> Malformed packet {packet_id} : {e}")
                }
                Err(PacketProcessError::ClientBusy) => {
                    sync.write().add_delayed_request(peer, packet_id, data)
                }
                Ok(()) => {}
            }
        }
    }

    /// Dispatch delayed request
    /// The main difference with dispatch packet is the direct send of the responses to the peer
    pub fn dispatch_delayed_request(
        sync: &RwLock<ChainSync>,
        io: &mut dyn SyncIo,
        peer: PeerId,
        packet_id: u8,
        data: &[u8],
    ) {
        if let Some(id) = SyncPacket::from_u8(packet_id) {
            let rlp_result = strip_request_id(data, sync.read().borrow(), &peer, &id);

            let result = match rlp_result {
                Ok((rlp, request_id)) => match id {
                    GetBlockHeadersPacket => SyncSupplier::send_rlp(
                        io,
                        &rlp,
                        peer,
                        request_id,
                        SyncSupplier::return_block_headers,
                        |e| format!("Error sending block headers: {e:?}"),
                    ),

                    _ => {
                        debug!(target: "sync", "Unexpected packet {packet_id} was dispatched for delayed processing");
                        Ok(())
                    }
                },
                Err(e) => Err(e.into()),
            };

            match result {
                Err(PacketProcessError::Decoder(e)) => {
                    debug!(target:"sync", "{peer} -> Malformed packet {packet_id} : {e}")
                }
                Err(PacketProcessError::ClientBusy) => {
                    sync.write().add_delayed_request(peer, packet_id, data)
                }
                Ok(()) => {}
            }
        }
    }

    /// Respond to GetBlockHeaders request
    fn return_block_headers(io: &dyn SyncIo, r: &Rlp, peer_id: PeerId) -> RlpResponseResult {
        // Cannot return blocks, if forks processing is in progress,
        // The request should be postponed for later processing
        if io.chain().is_processing_fork() {
            return Err(PacketProcessError::ClientBusy);
        }
        // Packet layout:
        // [ block: { P , B_32 }, maxHeaders: P, skip: P, reverse: P in { 0 , 1 } ]
        let max_headers: usize = r.val_at(1)?;
        let skip: usize = r.val_at(2)?;
        let reverse: bool = r.val_at(3)?;
        let last = io.chain().chain_info().best_block_number;
        let number = if r.at(0)?.size() == 32 {
            // id is a hash
            let hash: H256 = r.val_at(0)?;
            trace!(target: "sync", "{peer_id} -> GetBlockHeaders (hash: {hash}, max: {max_headers}, skip: {skip}, reverse:{reverse})");
            match io.chain().block_header(BlockId::Hash(hash)) {
                Some(hdr) => {
                    let number = hdr.number();
                    debug_assert_eq!(hdr.hash(), hash);

                    if max_headers == 1
                        || io.chain().block_hash(BlockId::Number(number)) != Some(hash)
                    {
                        // Non canonical header or single header requested
                        // TODO: handle single-step reverse hashchains of non-canon hashes
                        trace!(target:"sync", "Returning single header: {hash:?}");
                        let mut rlp = RlpStream::new_list(1);
                        rlp.append_raw(&hdr.into_inner(), 1);
                        return Ok(Some((BlockHeadersPacket, rlp)));
                    }
                    number
                }
                None => return Ok(Some((BlockHeadersPacket, RlpStream::new_list(0)))), //no such header, return nothing
            }
        } else {
            let number = r.val_at::<BlockNumber>(0)?;
            trace!(target: "sync", "{peer_id} -> GetBlockHeaders (number: {number}, max: {max_headers}, skip: {skip}, reverse:{reverse})");
            number
        };

        let mut number = if reverse {
            cmp::min(last, number)
        } else {
            cmp::max(0, number)
        };
        let max_count = cmp::min(MAX_HEADERS_TO_SEND, max_headers);
        let mut count = 0;
        let mut data = Bytes::new();
        let inc = skip.saturating_add(1) as BlockNumber;
        let overlay = io.chain_overlay().read();

        // We are checking the `overlay` as well since it's where the ForkBlock
        // header is cached : so peers can confirm we are on the right fork,
        // even if we are not synced until the fork block
        while (number <= last || overlay.contains_key(&number)) && count < max_count {
            if let Some(hdr) = overlay.get(&number) {
                trace!(target: "sync", "{peer_id}: Returning cached fork header");
                data.extend_from_slice(hdr);
                count += 1;
            } else if let Some(hdr) = io.chain().block_header(BlockId::Number(number)) {
                data.append(&mut hdr.into_inner());
                count += 1;
                // Check that the packet won't be oversized
                if data.len() > PAYLOAD_SOFT_LIMIT {
                    break;
                }
            } else {
                // No required block.
                break;
            }
            if reverse {
                if number <= inc || number == 0 {
                    break;
                }
                number = number.saturating_sub(inc);
            } else {
                number = number.saturating_add(inc);
            }
        }
        let mut rlp = RlpStream::new_list(count);
        rlp.append_raw(&data, count);
        trace!(target: "sync", "{peer_id} -> GetBlockHeaders: returned {count} entries");
        Ok(Some((BlockHeadersPacket, rlp)))
    }

    /// Respond to GetPooledTransactions request
    fn return_pooled_transactions(io: &dyn SyncIo, r: &Rlp, peer_id: PeerId) -> RlpResponseResult {
        let mut added = 0;
        let mut rlp = RlpStream::new();
        rlp.begin_unbounded_list();
        for v in r {
            if let Ok(hash) = v.as_val::<H256>() {
                if let Some(tx) = io.chain().queued_transaction(hash) {
                    tx.signed().rlp_append(&mut rlp);
                    added += 1;
                    if rlp.len() > PAYLOAD_SOFT_LIMIT {
                        break;
                    }
                }
            }
        }
        rlp.finalize_unbounded_list();

        trace!(target: "sync", "{peer_id} -> GetPooledTransactions: returned {added} entries");
        Ok(Some((PooledTransactionsPacket, rlp)))
    }

    /// Respond to GetBlockBodies request
    fn return_block_bodies(io: &dyn SyncIo, r: &Rlp, peer_id: PeerId) -> RlpResponseResult {
        let mut count = r.item_count().unwrap_or(0);
        if count == 0 {
            debug!(target: "sync", "Empty GetBlockBodies request, ignoring.");
            return Ok(None);
        }
        count = cmp::min(count, MAX_BODIES_TO_SEND);
        let mut added = 0usize;
        let mut data = Bytes::new();
        for i in 0..count {
            if let Some(body) = io.chain().block_body(BlockId::Hash(r.val_at::<H256>(i)?)) {
                data.append(&mut body.into_inner());
                added += 1;
                // Check that the packet won't be oversized
                if data.len() > PAYLOAD_SOFT_LIMIT {
                    break;
                }
            }
        }
        let mut rlp = RlpStream::new_list(added);
        rlp.append_raw(&data, added);
        trace!(target: "sync", "{peer_id} -> GetBlockBodies: returned {added} entries");
        Ok(Some((BlockBodiesPacket, rlp)))
    }

    fn return_node_data(io: &dyn SyncIo, rlp: &Rlp, peer_id: PeerId) -> RlpResponseResult {
        let count = cmp::min(rlp.item_count().unwrap_or(0), MAX_NODE_DATA_TO_SEND);
        trace!(target: "sync", "{peer_id} -> GetNodeData: {count} entries");
        if count == 0 {
            debug!(target: "sync", "Empty GetNodeData request, ignoring.");
            return Ok(None);
        }

        let mut added = 0usize;
        let mut data = Vec::new();
        let mut total_bytes = 0;
        for i in 0..count {
            if let Some(node_data) = io.chain().state_data(&rlp.val_at::<H256>(i)?) {
                total_bytes += node_data.len();
                // Check that the packet won't be oversized
                if total_bytes > PAYLOAD_SOFT_LIMIT {
                    break;
                }
                data.push(node_data);
                added += 1;
            }
        }

        let mut rlp = RlpStream::new_list(added);
        for d in data {
            rlp.append(&d);
        }
        trace!(target: "sync", "{peer_id} -> GetNodeData: returned {added} entries");
        Ok(Some((NodeDataPacket, rlp)))
    }

    fn return_receipts(io: &dyn SyncIo, rlp: &Rlp, peer_id: PeerId) -> RlpResponseResult {
        let mut count = rlp.item_count().unwrap_or(0);
        trace!(target: "sync", "{peer_id} -> GetReceipts: {count} entries");
        if count == 0 {
            debug!(target: "sync", "Empty GetReceipts request, ignoring.");
            return Ok(None);
        }
        count = cmp::min(count, MAX_RECEIPTS_HEADERS_TO_SEND);
        let mut added_headers = 0usize;
        let mut data = Bytes::new();
        let mut total_bytes = 0;
        for i in 0..count {
            if let Some(receipts) = io.chain().block_receipts(&rlp.val_at::<H256>(i)?) {
                let mut receipts_bytes = ::rlp::encode(&receipts);
                total_bytes += receipts_bytes.len();
                if total_bytes > PAYLOAD_SOFT_LIMIT {
                    break;
                }
                data.append(&mut receipts_bytes);
                added_headers += 1;
            }
        }
        let mut rlp_result = RlpStream::new_list(added_headers);
        rlp_result.append_raw(&data, added_headers);
        Ok(Some((ReceiptsPacket, rlp_result)))
    }

    /// Respond to GetSnapshotManifest request
    fn return_snapshot_manifest(io: &dyn SyncIo, r: &Rlp, peer_id: PeerId) -> RlpResponseResult {
        let count = r.item_count().unwrap_or(0);
        trace!(target: "warp", "{peer_id} -> GetSnapshotManifest");
        if count != 0 {
            debug!(target: "warp", "Invalid GetSnapshotManifest request, ignoring.");
            return Ok(None);
        }
        let rlp = match io.snapshot_service().manifest() {
            Some(manifest) => {
                trace!(target: "warp", "{peer_id} <- SnapshotManifest");
                let mut rlp = RlpStream::new_list(1);
                rlp.append_raw(&manifest.into_rlp(), 1);
                rlp
            }
            None => {
                trace!(target: "warp", "{peer_id}: No snapshot manifest to return");
                RlpStream::new_list(0)
            }
        };
        Ok(Some((SnapshotManifestPacket, rlp)))
    }

    /// Respond to GetSnapshotData request
    fn return_snapshot_data(io: &dyn SyncIo, r: &Rlp, peer_id: PeerId) -> RlpResponseResult {
        let hash: H256 = r.val_at(0)?;
        trace!(target: "warp", "{peer_id} -> GetSnapshotData {hash:?}");
        let rlp = match io.snapshot_service().chunk(hash) {
            Some(data) => {
                let mut rlp = RlpStream::new_list(1);
                trace!(target: "warp", "{peer_id} <- SnapshotData");
                rlp.append(&data);
                rlp
            }
            None => {
                trace!(target: "warp", "{peer_id}: No snapshot data to return");
                RlpStream::new_list(0)
            }
        };
        Ok(Some((SnapshotDataPacket, rlp)))
    }

    fn return_rlp<FRlp, FError>(
        io: &mut dyn SyncIo,
        rlp: &Rlp,
        peer: PeerId,
        request_id: Option<RequestId>,
        rlp_func: FRlp,
        error_func: FError,
    ) -> Result<(), PacketProcessError>
    where
        FRlp: Fn(&dyn SyncIo, &Rlp, PeerId) -> RlpResponseResult,
        FError: FnOnce(network::Error) -> String,
    {
        let response = rlp_func(io, rlp, peer);
        if let Some((packet_id, rlp_stream)) = response? {
            let rlp_stream = prepend_request_id(rlp_stream, request_id);
            io.respond(packet_id.id(), rlp_stream.out())
                .unwrap_or_else(|e| debug!(target: "sync", "{:?}", error_func(e)));
        }
        Ok(())
    }

    fn send_rlp<FRlp, FError>(
        io: &mut dyn SyncIo,
        rlp: &Rlp,
        peer: PeerId,
        request_id: Option<RequestId>,
        rlp_func: FRlp,
        error_func: FError,
    ) -> Result<(), PacketProcessError>
    where
        FRlp: Fn(&dyn SyncIo, &Rlp, PeerId) -> RlpResponseResult,
        FError: FnOnce(network::Error) -> String,
    {
        let response = rlp_func(io, rlp, peer);
        match response {
            Err(e) => Err(e),
            Ok(Some((packet_id, rlp_stream))) => {
                let rlp_stream = prepend_request_id(rlp_stream, request_id);
                io.send(peer, packet_id, rlp_stream.out())
                    .unwrap_or_else(|e| debug!(target: "sync", "{:?}", error_func(e)));
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{super::tests::*, *};
    use blocks::SyncHeader;
    use bytes::Bytes;
    use ethcore::{
        client::{BlockChainClient, EachBlockWith, TestBlockChainClient},
        spec::Spec,
    };
    use ethereum_types::H256;
    use parking_lot::RwLock;
    use rlp::{Rlp, RlpStream};
    use std::{collections::VecDeque, str::FromStr};
    use tests::{helpers::TestIo, snapshot::TestSnapshotService};

    #[test]
    fn return_block_headers() {
        fn make_hash_req(h: &H256, count: usize, skip: usize, reverse: bool) -> Bytes {
            let mut rlp = RlpStream::new_list(4);
            rlp.append(h);
            rlp.append(&count);
            rlp.append(&skip);
            rlp.append(&if reverse { 1u32 } else { 0u32 });
            rlp.out()
        }

        fn make_num_req(n: usize, count: usize, skip: usize, reverse: bool) -> Bytes {
            let mut rlp = RlpStream::new_list(4);
            rlp.append(&n);
            rlp.append(&count);
            rlp.append(&skip);
            rlp.append(&if reverse { 1u32 } else { 0u32 });
            rlp.out()
        }
        fn to_header_vec(
            rlp: ::chain::RlpResponseResult,
            eip1559_transition: BlockNumber,
        ) -> Vec<SyncHeader> {
            Rlp::new(&rlp.unwrap().unwrap().1.out())
                .iter()
                .map(|r| SyncHeader::from_rlp(r.as_raw().to_vec(), eip1559_transition).unwrap())
                .collect()
        }

        let mut client = TestBlockChainClient::new();
        let eip1559_transition = client.spec.params().eip1559_transition;
        client.add_blocks(100, EachBlockWith::Nothing);
        let blocks: Vec<_> = (0..100)
            .map(|i| {
                (&client as &dyn BlockChainClient)
                    .block(BlockId::Number(i as BlockNumber))
                    .map(|b| b.into_inner())
                    .unwrap()
            })
            .collect();
        let headers: Vec<_> = blocks
            .iter()
            .map(|b| {
                SyncHeader::from_rlp(
                    Rlp::new(b).at(0).unwrap().as_raw().to_vec(),
                    eip1559_transition,
                )
                .unwrap()
            })
            .collect();
        let hashes: Vec<_> = headers.iter().map(|h| h.header.hash()).collect();

        let queue = RwLock::new(VecDeque::new());
        let ss = TestSnapshotService::new();
        let io = TestIo::new(&mut client, &ss, &queue, None);

        let unknown: H256 = H256::default();
        let result = SyncSupplier::return_block_headers(
            &io,
            &Rlp::new(&make_hash_req(&unknown, 1, 0, false)),
            0,
        );
        assert!(to_header_vec(result, eip1559_transition).is_empty(),);
        let result = SyncSupplier::return_block_headers(
            &io,
            &Rlp::new(&make_hash_req(&unknown, 1, 0, true)),
            0,
        );
        assert!(to_header_vec(result, eip1559_transition).is_empty());

        let result = SyncSupplier::return_block_headers(
            &io,
            &Rlp::new(&make_hash_req(&hashes[2], 1, 0, true)),
            0,
        );
        assert_eq!(
            to_header_vec(result, eip1559_transition),
            vec![headers[2].clone()]
        );

        let result = SyncSupplier::return_block_headers(
            &io,
            &Rlp::new(&make_hash_req(&hashes[2], 1, 0, false)),
            0,
        );
        assert_eq!(
            to_header_vec(result, eip1559_transition),
            vec![headers[2].clone()]
        );

        let result = SyncSupplier::return_block_headers(
            &io,
            &Rlp::new(&make_hash_req(&hashes[50], 3, 5, false)),
            0,
        );
        assert_eq!(
            to_header_vec(result, eip1559_transition),
            vec![
                headers[50].clone(),
                headers[56].clone(),
                headers[62].clone()
            ]
        );

        let result = SyncSupplier::return_block_headers(
            &io,
            &Rlp::new(&make_hash_req(&hashes[50], 3, 5, true)),
            0,
        );
        assert_eq!(
            to_header_vec(result, eip1559_transition),
            vec![
                headers[50].clone(),
                headers[44].clone(),
                headers[38].clone()
            ]
        );

        let result =
            SyncSupplier::return_block_headers(&io, &Rlp::new(&make_num_req(2, 1, 0, true)), 0);
        assert_eq!(
            to_header_vec(result, eip1559_transition),
            vec![headers[2].clone()]
        );

        let result =
            SyncSupplier::return_block_headers(&io, &Rlp::new(&make_num_req(2, 1, 0, false)), 0);
        assert_eq!(
            to_header_vec(result, eip1559_transition),
            vec![headers[2].clone()]
        );

        let result =
            SyncSupplier::return_block_headers(&io, &Rlp::new(&make_num_req(50, 3, 5, false)), 0);
        assert_eq!(
            to_header_vec(result, eip1559_transition),
            vec![
                headers[50].clone(),
                headers[56].clone(),
                headers[62].clone()
            ]
        );

        let result =
            SyncSupplier::return_block_headers(&io, &Rlp::new(&make_num_req(50, 3, 5, true)), 0);
        assert_eq!(
            to_header_vec(result, eip1559_transition),
            vec![
                headers[50].clone(),
                headers[44].clone(),
                headers[38].clone()
            ]
        );
    }

    #[test]
    fn respect_packet_limit() {
        let small_num_blocks = 10;
        let large_num_blocks = 50;
        let tx_per_block = 100;

        let mut client = TestBlockChainClient::new();
        client.add_blocks(large_num_blocks, EachBlockWith::Transactions(tx_per_block));

        let mut small_rlp_request = RlpStream::new_list(small_num_blocks);
        let mut large_rlp_request = RlpStream::new_list(large_num_blocks);

        for i in 0..small_num_blocks {
            let hash: H256 = client.block_hash(BlockId::Number(i as u64)).unwrap();
            small_rlp_request.append(&hash);
            large_rlp_request.append(&hash);
        }

        for i in small_num_blocks..large_num_blocks {
            let hash: H256 = client.block_hash(BlockId::Number(i as u64)).unwrap();
            large_rlp_request.append(&hash);
        }

        let queue = RwLock::new(VecDeque::new());
        let ss = TestSnapshotService::new();
        let io = TestIo::new(&mut client, &ss, &queue, None);

        let small_result =
            SyncSupplier::return_block_bodies(&io, &Rlp::new(&small_rlp_request.out()), 0);
        let small_result = small_result.unwrap().unwrap().1;
        assert_eq!(
            Rlp::new(&small_result.out()).item_count().unwrap(),
            small_num_blocks
        );

        let large_result =
            SyncSupplier::return_block_bodies(&io, &Rlp::new(&large_rlp_request.out()), 0);
        let large_result = large_result.unwrap().unwrap().1;
        assert!(Rlp::new(&large_result.out()).item_count().unwrap() < large_num_blocks);
    }

    #[test]
    fn return_receipts_empty() {
        let mut client = TestBlockChainClient::new();
        let queue = RwLock::new(VecDeque::new());
        let ss = TestSnapshotService::new();
        let io = TestIo::new(&mut client, &ss, &queue, None);

        let result = SyncSupplier::return_receipts(&io, &Rlp::new(&[0xc0]), 0);

        assert!(result.is_ok());
    }

    #[test]
    fn return_receipts() {
        let mut client = TestBlockChainClient::new();
        let queue = RwLock::new(VecDeque::new());
        let sync = dummy_sync_with_peer(H256::default(), &client);
        let ss = TestSnapshotService::new();
        let mut io = TestIo::new(&mut client, &ss, &queue, None);

        let mut receipt_list = RlpStream::new_list(4);
        receipt_list.append(
            &H256::from_str("0000000000000000000000000000000000000000000000005555555555555555")
                .unwrap(),
        );
        receipt_list.append(
            &H256::from_str("ff00000000000000000000000000000000000000000000000000000000000000")
                .unwrap(),
        );
        receipt_list.append(
            &H256::from_str("fff0000000000000000000000000000000000000000000000000000000000000")
                .unwrap(),
        );
        receipt_list.append(
            &H256::from_str("aff0000000000000000000000000000000000000000000000000000000000000")
                .unwrap(),
        );

        let receipts_request = receipt_list.out();
        // it returns rlp ONLY for hashes started with "f"
        let result = SyncSupplier::return_receipts(&io, &Rlp::new(&receipts_request.clone()), 0);

        assert!(result.is_ok());
        let rlp_result = result.unwrap();
        assert!(rlp_result.is_some());

        // the length of two rlp-encoded receipts
        assert_eq!(603, rlp_result.unwrap().1.out().len());

        io.sender = Some(2usize);
        SyncSupplier::dispatch_packet(
            &RwLock::new(sync),
            &mut io,
            0usize,
            GetReceiptsPacket.id(),
            &receipts_request,
        );
        assert_eq!(1, io.packets.len());
    }

    #[test]
    fn return_nodes() {
        let mut client = TestBlockChainClient::new_with_spec(Spec::new_test_round());
        let queue = RwLock::new(VecDeque::new());
        let sync = dummy_sync_with_peer(H256::zero(), &client);
        let ss = TestSnapshotService::new();
        let mut io = TestIo::new(&mut client, &ss, &queue, None);

        let mut node_list = RlpStream::new_list(3);
        node_list.append(
            &H256::from_str("0000000000000000000000000000000000000000000000005555555555555555")
                .unwrap(),
        );
        node_list.append(
            &H256::from_str("ffffffffffffffffffffffffffffffffffffffffffffaaaaaaaaaaaaaaaaaaaa")
                .unwrap(),
        );
        node_list.append(
            &H256::from_str("aff0000000000000000000000000000000000000000000000000000000000000")
                .unwrap(),
        );

        let node_request = node_list.out();
        // it returns rlp ONLY for hashes started with "f"
        let result = SyncSupplier::return_node_data(&io, &Rlp::new(&node_request.clone()), 0);

        assert!(result.is_ok());
        let rlp_result = result.unwrap();
        assert!(rlp_result.is_some());

        // the length of one rlp-encoded hashe
        let rlp = rlp_result.unwrap().1.out();
        let rlp = Rlp::new(&rlp);
        assert_eq!(Ok(1), rlp.item_count());

        io.sender = Some(2usize);

        SyncSupplier::dispatch_packet(
            &RwLock::new(sync),
            &mut io,
            0usize,
            GetNodeDataPacket.id(),
            &node_request,
        );
        assert_eq!(1, io.packets.len());
    }

    #[test]
    fn dispatch_get_node_data_request() {
        let mut client = TestBlockChainClient::new_with_spec(Spec::new_test_round());
        let queue = RwLock::new(VecDeque::new());
        let sync = dummy_sync(&client);
        let ss = TestSnapshotService::new();
        let mut io = TestIo::new(&mut client, &ss, &queue, None);

        let mut node_list = RlpStream::new_list(3);
        node_list.append(
            &H256::from_str("000000000000000000000000000000000000000000000000000000000000000a")
                .unwrap(),
        );
        node_list.append(
            &H256::from_str("000000000000000000000000000000000000000000000000000000000000000b")
                .unwrap(),
        );
        node_list.append(
            &H256::from_str("000000000000000000000000000000000000000000000000000000000000000c")
                .unwrap(),
        );

        let node_request = node_list;
        let node_request = prepend_request_id(node_request, Some(0x0b3a73ce2ff2));

        io.sender = Some(2usize);

        // it returns rlp ONLY for hashes ending with "a" and "c"
        SyncSupplier::dispatch_packet(
            &RwLock::new(sync),
            &mut io,
            0usize,
            GetNodeDataPacket.id(),
            &node_request.out(),
        );
        assert_eq!(1, io.packets.len());
        assert_eq!(
            &io.packets[0].data,
            &vec![
                0xcd, 0x86, 0x0b, 0x3a, 0x73, 0xce, 0x2f, 0xf2, 0xc5, 0x82, 0xaa, 0xaa, 0x81, 0xcc
            ]
        );
    }
}
