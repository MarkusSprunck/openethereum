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

//! Spec seal.

use ethereum_types::{H256, H520, H64};
use ethjson;
use rlp::RlpStream;

/// Classic ethereum seal.
pub struct Ethereum {
    /// Seal nonce.
    pub nonce: H64,
    /// Seal mix hash.
    pub mix_hash: H256,
}

impl From<Ethereum> for Generic {
    fn from(val: Ethereum) -> Self {
        let mut s = RlpStream::new_list(2);
        s.append(&val.mix_hash).append(&val.nonce);
        Generic(s.out())
    }
}

/// AuthorityRound seal.
pub struct AuthorityRound {
    /// Seal step.
    pub step: usize,
    /// Seal signature.
    pub signature: H520,
}

/// Tendermint seal.
pub struct Tendermint {
    /// Seal round.
    pub round: usize,
    /// Proposal seal signature.
    pub proposal: H520,
    /// Precommit seal signatures.
    pub precommits: Vec<H520>,
}

impl From<AuthorityRound> for Generic {
    fn from(val: AuthorityRound) -> Self {
        let mut s = RlpStream::new_list(2);
        s.append(&val.step).append(&val.signature);
        Generic(s.out())
    }
}

impl From<Tendermint> for Generic {
    fn from(val: Tendermint) -> Self {
        let mut stream = RlpStream::new_list(3);
        stream
            .append(&val.round)
            .append(&val.proposal)
            .append_list(&val.precommits);
        Generic(stream.out())
    }
}

pub struct Generic(pub Vec<u8>);

/// Genesis seal type.
pub enum Seal {
    /// Classic ethereum seal.
    Ethereum(Ethereum),
    /// AuthorityRound seal.
    AuthorityRound(AuthorityRound),
    /// Tendermint seal.
    Tendermint(Tendermint),
    /// Generic RLP seal.
    Generic(Generic),
}

impl From<ethjson::spec::Seal> for Seal {
    fn from(s: ethjson::spec::Seal) -> Self {
        match s {
            ethjson::spec::Seal::Ethereum(eth) => Seal::Ethereum(Ethereum {
                nonce: eth.nonce.into(),
                mix_hash: eth.mix_hash.into(),
            }),
            ethjson::spec::Seal::AuthorityRound(ar) => Seal::AuthorityRound(AuthorityRound {
                step: ar.step.into(),
                signature: ar.signature.into(),
            }),
            ethjson::spec::Seal::Tendermint(tender) => Seal::Tendermint(Tendermint {
                round: tender.round.into(),
                proposal: tender.proposal.into(),
                precommits: tender.precommits.into_iter().map(Into::into).collect(),
            }),
            ethjson::spec::Seal::Generic(g) => Seal::Generic(Generic(g.into())),
        }
    }
}

impl From<Seal> for Generic {
    fn from(val: Seal) -> Self {
        match val {
            Seal::Generic(generic) => generic,
            Seal::Ethereum(eth) => eth.into(),
            Seal::AuthorityRound(ar) => ar.into(),
            Seal::Tendermint(tender) => tender.into(),
        }
    }
}
