// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of Parity Bridges Common.

// Parity Bridges Common is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Bridges Common is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Bridges Common.  If not, see <http://www.gnu.org/licenses/>.

//! Westend-to-Circuit headers sync entrypoint.

use crate::finality_pipeline::{SubstrateFinalitySyncPipeline, SubstrateFinalityToSubstrate};

use bp_header_chain::justification::GrandpaJustification;
use codec::Encode;
use relay_circuit_client::{Circuit, SigningParams as CircuitSigningParams};
use relay_substrate_client::{Chain, TransactionSignScheme};
use relay_utils::metrics::MetricsParams;
use relay_westend_client::{SyncHeader as WestendSyncHeader, Westend};
use sp_core::{Bytes, Pair};

/// Westend-to-Circuit finality sync pipeline.
pub(crate) type WestendFinalityToCircuit = SubstrateFinalityToSubstrate<Westend, Circuit, CircuitSigningParams>;

impl SubstrateFinalitySyncPipeline for WestendFinalityToCircuit {
	const BEST_FINALIZED_SOURCE_HEADER_ID_AT_TARGET: &'static str = bp_westend::BEST_FINALIZED_WESTEND_HEADER_METHOD;

	type TargetChain = Circuit;

	fn customize_metrics(params: MetricsParams) -> anyhow::Result<MetricsParams> {
		crate::chains::add_polkadot_kusama_price_metrics::<Self>(params)
	}

	fn transactions_author(&self) -> bp_circuit::AccountId {
		(*self.target_sign.public().as_array_ref()).into()
	}

	fn make_submit_finality_proof_transaction(
		&self,
		transaction_nonce: <Circuit as Chain>::Index,
		header: WestendSyncHeader,
		proof: GrandpaJustification<bp_westend::Header>,
	) -> Bytes {
		let call = circuit_runtime::BridgeGrandpaWestendCall::<
			circuit_runtime::Runtime,
			circuit_runtime::WestendGrandpaInstance,
		>::submit_finality_proof(header.into_inner(), proof)
		.into();

		let genesis_hash = *self.target_client.genesis_hash();
		let transaction = Circuit::sign_transaction(genesis_hash, &self.target_sign, transaction_nonce, call);

		Bytes(transaction.encode())
	}
	fn make_submit_finality_proof_transaction_and_roots(
		&self,
		transaction_nonce: <Circuit as Chain>::Index,
		header: WestendSyncHeader,
		proof: GrandpaJustification<bp_westend::Header>,
		state_root: Self::Hash,
		extrinsics_root: Self::Hash,
	) -> Bytes {

		let call = circuit_runtime::BridgePolkadotLikeMultiFinalityVerifierCall::<
			circuit_runtime::Runtime,
			circuit_runtime::PolkadotLikeGrandpaInstance,
		>::submit_finality_proof_and_roots(header.into_inner(), proof, self.gateway_id, state_root, extrinsics_root)
			.into();

		let genesis_hash = *self.target_client.genesis_hash();
		let transaction = Circuit::sign_transaction(genesis_hash, &self.target_sign, transaction_nonce, call);

		Bytes(transaction.encode())
	}
}
