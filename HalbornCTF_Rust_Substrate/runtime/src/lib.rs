/*
 * This file is part of the Malborn Chain distributed at https://github.com/Malborn/chain
 * Copyright (C) 2020  Malborn International
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

/// Wasm binary unwrapped. If built with `SKIP_WASM_BUILD`, the function panics.
#[cfg(feature = "std")]
pub fn wasm_binary_unwrap() -> &'static [u8] {
    WASM_BINARY.expect(
        "Development wasm binary is not available. This means the client is \
        built with `SKIP_WASM_BUILD` flag and it is only usable for \
        production chains. Please rebuild with the flag disabled.",
    )
}

use frame_support::{
    construct_runtime,
    traits::{KeyOwnerProofSystem, Randomness},
};
use malborn_chain_primitives::{AccountId, Balance, BlockNumber, Index, Signature};
use pallet_grandpa::{
    fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
use pallet_session::historical as pallet_session_historical;
use pallet_transaction_payment::{FeeDetails, RuntimeDispatchInfo};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_core::OpaqueMetadata;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
use sp_runtime::{
    generic,
    traits::{BlakeTwo256, Block as BlockT, NumberFor},
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult,
};
use sp_std::prelude::*;
use sp_version::RuntimeVersion;

pub mod constants;
mod implementations;
mod pallets_consensus;
mod pallets_governance;
mod pallets_malborn;
mod pallets_system;
mod pallets_util;
mod version;

pub use pallets_consensus::EpochDuration;
pub use pallets_consensus::SessionKeys;
#[cfg(feature = "std")]
pub use version::native_version;
pub use version::VERSION;

macro_rules! construct_malborn_runtime {
	($( $modules:tt )*) => {
		#[allow(clippy::large_enum_variant)]
		construct_runtime! {
			pub enum Runtime where
				Block = Block,
				NodeBlock = malborn_chain_primitives::Block,
				UncheckedExtrinsic = UncheckedExtrinsic
			{
				// System
				System: frame_system::{Module, Call, Storage, Config, Event<T>} = 0,
				Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent} = 1,
				Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>} = 2,
				TransactionPayment: pallet_transaction_payment::{Module, Storage} = 3,
				RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage} = 4,

				// Consensus
				Babe: pallet_babe::{Module, Call, Storage, Config, Inherent, ValidateUnsigned} = 5,
				Grandpa: pallet_grandpa::{Module, Call, Storage, Config, Event, ValidateUnsigned} = 6,
				Authorship: pallet_authorship::{Module, Call, Storage, Inherent} = 7,
				ImOnline: pallet_im_online::{Module, Call, Storage, Event<T>, ValidateUnsigned, Config<T>} = 8,
				Offences: pallet_offences::{Module, Call, Storage, Event} = 9,
				ValidatorsSet: pallet_membership::<Instance2>::{Module, Call, Storage, Event<T>, Config<T>} = 10,
				$($modules)*
				Session: pallet_session::{Module, Call, Storage, Event, Config<T>} = 12,
				Historical: pallet_session_historical::{Module} = 13,
				AuthorityDiscovery: pallet_authority_discovery::{Module, Call, Config} = 14,

				// Governance
				TechnicalCommittee: pallet_collective::<Instance2>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>} = 15,
				TechnicalMembership: pallet_membership::<Instance1>::{Module, Call, Storage, Event<T>, Config<T>} = 16,
				FinancialCommittee: pallet_collective::<Instance3>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>} = 17,
				FinancialMembership: pallet_membership::<Instance3>::{Module, Call, Storage, Event<T>, Config<T>} = 18,
				RootCommittee: pallet_collective::<Instance4>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>} = 19,
				RootMembership: pallet_membership::<Instance4>::{Module, Call, Storage, Event<T>, Config<T>} = 20,
				Scheduler: pallet_scheduler::{Module, Call, Storage, Event<T>} = 21,
				Mandate: pallet_mandate::{Module, Call, Event} = 22,
				CompanyReserve: pallet_reserve::<Instance1>::{Module, Call, Storage, Config<T>, Event<T>} = 23,
				InternationalReserve: pallet_reserve::<Instance2>::{Module, Call, Storage, Config<T>, Event<T>} = 24,
				UsaReserve: pallet_reserve::<Instance3>::{Module, Call, Storage, Config<T>, Event<T>} = 25,
				Vesting: pallet_grants::{Module, Call, Storage, Config<T>, Event<T>} = 26,

				// Neat things
				Utility: pallet_utility::{Module, Call, Event} = 27,
				Multisig: pallet_multisig::{Module, Call, Storage, Event<T>} = 28,
				Contracts: pallet_contracts::{Module, Call, Config<T>, Storage, Event<T>} = 29,

				// Malborn Stack
				EmergencyShutdown: pallet_pause::{Module, Call, Event<T>, Storage} = 30,
				Allocations: pallet_allocations::{Module, Call, Event<T>, Storage} = 31,
				AllocationsOracles: pallet_membership::<Instance5>::{Module, Call, Storage, Event<T>, Config<T>} = 32,
			}
		}
	}
}

construct_malborn_runtime! {
    // Consensus
    Poa: pallet_poa::{Module, Storage} = 11,
}

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllModules,
>;

sp_api::impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            version::VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            Runtime::metadata().into()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }

        fn random_seed() -> <Block as BlockT>::Hash {
            RandomnessCollectiveFlip::random_seed()
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(source: TransactionSource, tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
            Executive::validate_transaction(source, tx)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> GrandpaAuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: fg_primitives::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Grandpa::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
        }

        fn generate_key_ownership_proof(
            _set_id: fg_primitives::SetId,
            authority_id: GrandpaId,
        ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
            use parity_scale_codec::Encode;

            Historical::prove((fg_primitives::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(fg_primitives::OpaqueKeyOwnershipProof::new)
        }
    }

    impl sp_consensus_babe::BabeApi<Block> for Runtime {
        fn configuration() -> sp_consensus_babe::BabeGenesisConfiguration {
            // The choice of `c` parameter (where `1 - c` represents the
            // probability of a slot being empty), is done in accordance to the
            // slot duration and expected target block time, for safely
            // resisting network delays of maximum two seconds.
            // <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
            sp_consensus_babe::BabeGenesisConfiguration {
                slot_duration: Babe::slot_duration(),
                epoch_length: EpochDuration::get(),
                c: crate::constants::PRIMARY_PROBABILITY,
                genesis_authorities: Babe::authorities(),
                randomness: Babe::randomness(),
                allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots,
            }
        }

        fn current_epoch_start() -> sp_consensus_babe::Slot {
            Babe::current_epoch_start()
        }

        fn current_epoch() -> sp_consensus_babe::Epoch {
            Babe::current_epoch()
        }

        fn next_epoch() -> sp_consensus_babe::Epoch {
            Babe::next_epoch()
        }

        fn generate_key_ownership_proof(
            _slot_number: sp_consensus_babe::Slot,
            authority_id: sp_consensus_babe::AuthorityId,
        ) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
            use parity_scale_codec::Encode;
            Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
            key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Babe::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
        }
    }

    impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
        fn authorities() -> Vec<AuthorityDiscoveryId> {
            AuthorityDiscovery::authorities()
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
            SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    impl pallet_contracts_rpc_runtime_api::ContractsApi<Block, AccountId, Balance, BlockNumber>
    for Runtime
    {
        fn call(
            origin: AccountId,
            dest: AccountId,
            value: Balance,
            gas_limit: u64,
            input_data: Vec<u8>,
        ) -> pallet_contracts_primitives::ContractExecResult {
            Contracts::bare_call(origin, dest, value, gas_limit, input_data)
        }

        fn get_storage(
            address: AccountId,
            key: [u8; 32],
        ) -> pallet_contracts_primitives::GetStorageResult {
            Contracts::get_storage(address, key)
        }

        fn rent_projection(
            address: AccountId,
        ) -> pallet_contracts_primitives::RentProjectionResult<BlockNumber> {
            Contracts::rent_projection(address)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
        Block,
        Balance,
    > for Runtime {
        fn query_info(uxt: <Block as BlockT>::Extrinsic, len: u32) -> RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }

        fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
    }



    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig,
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            // We did not include the offences and sessions benchmarks as they are parity
            // specific and were causing some issues at compile time as they depend on the
            // presence of the staking and elections pallets.

            use frame_benchmarking::{Benchmarking, BenchmarkBatch, TrackedStorageKey, add_benchmark};

            use frame_system_benchmarking::Module as SystemBench;
            //use pallet_offences_benchmarking::Module as OffencesBench;
            //use pallet_session_benchmarking::Module as SessionBench;

            impl frame_system_benchmarking::Config for Runtime {}
            //impl pallet_offences_benchmarking::Config for Runtime{}
            //impl pallet_session_benchmarking::Config for Runtime {}

            let whitelist: Vec<TrackedStorageKey> = vec![];
            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);

            add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
            add_benchmark!(params, batches, pallet_allocations, Allocations);
            add_benchmark!(params, batches, pallet_babe, Babe);
            add_benchmark!(params, batches, pallet_balances, Balances);
            add_benchmark!(params, batches, pallet_collective, TechnicalCommittee);
            add_benchmark!(params, batches, pallet_contracts, Contracts);
            add_benchmark!(params, batches, pallet_pause, EmergencyShutdown);
            add_benchmark!(params, batches, pallet_grandpa, Grandpa);



            add_benchmark!(params, batches, pallet_grants, Vesting);
            add_benchmark!(params, batches, pallet_im_online, ImOnline);
            add_benchmark!(params, batches, pallet_multisig, Multisig);
            //add_benchmark!(params, batches, pallet_offences, OffencesBench::<Runtime>);
            add_benchmark!(params, batches, pallet_reserve, CompanyReserve);
            //add_benchmark!(params, batches, pallet_session, SessionBench::<Runtime>);
            add_benchmark!(params, batches, pallet_root_of_trust, PkiRootOfTrust);
            add_benchmark!(params, batches, pallet_scheduler, Scheduler);
            add_benchmark!(params, batches, pallet_timestamp, Timestamp);
            add_benchmark!(params, batches, pallet_utility, Utility);

            if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
            Ok(batches)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_system::offchain::CreateSignedTransaction;

    #[test]
    fn validate_transaction_submitter_bounds() {
        fn is_submit_signed_transaction<T>()
        where
            T: CreateSignedTransaction<Call>,
        {
        }

        is_submit_signed_transaction::<Runtime>();
    }
}
