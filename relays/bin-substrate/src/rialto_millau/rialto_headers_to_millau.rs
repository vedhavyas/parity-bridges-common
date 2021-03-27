// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

//! Rialto-to-Millau headers sync entrypoint.

use super::{MillauClient, RialtoClient};
use crate::finality_pipeline::{SubstrateFinalitySyncPipeline, SubstrateFinalityToSubstrate};

use codec::Encode;
use relay_millau_client::{Millau, SigningParams as MillauSigningParams};
use relay_rialto_client::{Rialto, SyncHeader as RialtoSyncHeader};
use relay_substrate_client::{finality_source::Justification, Chain, TransactionSignScheme};
use sp_core::{Bytes, Pair};

/// Rialto-to-Millau finality sync pipeline.
pub(crate) type RialtoFinalityToMillau = SubstrateFinalityToSubstrate<Rialto, Millau, MillauSigningParams>;

impl SubstrateFinalitySyncPipeline for RialtoFinalityToMillau {
	const BEST_FINALIZED_SOURCE_HEADER_ID_AT_TARGET: &'static str = bp_rialto::BEST_FINALIZED_RIALTO_HEADER_METHOD;

	type TargetChain = Millau;

	fn transactions_author(&self) -> bp_millau::AccountId {
		self.target_sign.signer.public().as_array_ref().clone().into()
	}

	fn make_submit_finality_proof_transaction(
		&self,
		transaction_nonce: <Millau as Chain>::Index,
		header: RialtoSyncHeader,
		proof: Justification<bp_rialto::BlockNumber>,
	) -> Bytes {
		let call = millau_runtime::BridgeGrandpaRialtoCall::<
			millau_runtime::Runtime,
			millau_runtime::RialtoGrandpaInstance,
		>::submit_finality_proof(header.into_inner(), proof.into_inner())
		.into();

		let genesis_hash = *self.target_client.genesis_hash();
		let transaction = Millau::sign_transaction(genesis_hash, &self.target_sign.signer, transaction_nonce, call);

		Bytes(transaction.encode())
	}
}

/// Run Rialto-to-Millau finality sync.
pub async fn run(
	rialto_client: RialtoClient,
	millau_client: MillauClient,
	millau_sign: MillauSigningParams,
	metrics_params: Option<relay_utils::metrics::MetricsParams>,
) -> Result<(), String> {
	crate::finality_pipeline::run(
		RialtoFinalityToMillau::new(millau_client.clone(), millau_sign),
		rialto_client,
		millau_client,
		metrics_params,
	)
	.await
}
