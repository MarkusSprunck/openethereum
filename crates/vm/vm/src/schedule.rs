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

//! Cost schedule and other parameterisations for the EVM.

// Gas per non accessed address when sload
pub const EIP2929_COLD_SLOAD_COST: usize = 2100;
// Gas per non accessed address accessing account from other opcodes defined in EIP2929
pub const EIP2929_COLD_ACCOUNT_ACCESS_COST: usize = 2600;
// Gas per already accessed address
pub const EIP2929_WARM_STORAGE_READ_COST: usize = 100;
// Gas per sstore reset
pub const EIP2929_SSTORE_RESET_GAS: usize = 5000 - EIP2929_COLD_SLOAD_COST;
/// Gas per received storage key
pub const EIP2930_ACCESS_LIST_STORAGE_KEY_COST: usize = 1900;
/// Gas per received address
pub const EIP2930_ACCESS_LIST_ADDRESS_COST: usize = 2400;
/// Gas used per transaction divided by this number is the maximum refundable amount
pub const MAX_REFUND_QUOTIENT: usize = 2;
pub const EIP3529_MAX_REFUND_QUOTIENT: usize = 5;
/// Reduced SSTORE refund as by EIP-3529
pub const EIP3529_SSTORE_CLEARS_SCHEDULE: usize =
    EIP2929_SSTORE_RESET_GAS + EIP2930_ACCESS_LIST_STORAGE_KEY_COST;

/// Definition of the cost schedule and other parameterisations for the EVM.
#[derive(Debug)]
pub struct Schedule {
    /// Does it support exceptional failed code deposit
    pub exceptional_failed_code_deposit: bool,
    /// Does it have a delegate cal
    pub have_delegate_call: bool,
    /// Does it have a CREATE2 instruction
    pub have_create2: bool,
    /// Does it have a REVERT instruction
    pub have_revert: bool,
    /// Does it have a EXTCODEHASH instruction
    pub have_extcodehash: bool,
    /// VM stack limit
    pub stack_limit: usize,
    /// Max number of nested calls/creates
    pub max_depth: usize,
    /// Gas prices for instructions in all tiers
    pub tier_step_gas: [usize; 8],
    /// Gas price for `EXP` opcode
    pub exp_gas: usize,
    /// Additional gas for `EXP` opcode for each byte of exponent
    pub exp_byte_gas: usize,
    /// Gas price for `SHA3` opcode
    pub sha3_gas: usize,
    /// Additional gas for `SHA3` opcode for each word of hashed memory
    pub sha3_word_gas: usize,
    /// Gas price for loading from storage
    pub sload_gas: usize,
    /// Gas price for setting new value to storage (`storage==0`, `new!=0`)
    pub sstore_set_gas: usize,
    /// Gas price for altering value in storage
    pub sstore_reset_gas: usize,
    /// Gas refund for `SSTORE` clearing (when `storage!=0`, `new==0`)
    pub sstore_refund_gas: usize,
    /// Gas price for `JUMPDEST` opcode
    pub jumpdest_gas: usize,
    /// Gas price for `LOG*`
    pub log_gas: usize,
    /// Additional gas for data in `LOG*`
    pub log_data_gas: usize,
    /// Additional gas for each topic in `LOG*`
    pub log_topic_gas: usize,
    /// Gas price for `CREATE` opcode
    pub create_gas: usize,
    /// Gas price for `*CALL*` opcodes
    pub call_gas: usize,
    /// EIP-2929 COLD_SLOAD_COST
    pub cold_sload_cost: usize,
    /// EIP-2929 COLD_ACCOUNT_ACCESS_COST
    pub cold_account_access_cost: usize,
    /// EIP-2929 WARM_STORAGE_READ_COST
    pub warm_storage_read_cost: usize,
    /// Stipend for transfer for `CALL|CALLCODE` opcode when `value>0`
    pub call_stipend: usize,
    /// Additional gas required for value transfer (`CALL|CALLCODE`)
    pub call_value_transfer_gas: usize,
    /// Additional gas for creating new account (`CALL|CALLCODE`)
    pub call_new_account_gas: usize,
    /// Refund for SUICIDE
    pub suicide_refund_gas: usize,
    /// Gas for used memory
    pub memory_gas: usize,
    /// Coefficient used to convert memory size to gas price for memory
    pub quad_coeff_div: usize,
    /// Cost for contract length when executing `CREATE`
    pub create_data_gas: usize,
    /// Maximum code size when creating a contract.
    pub create_data_limit: usize,
    /// Transaction cost
    pub tx_gas: usize,
    /// `CREATE` transaction cost
    pub tx_create_gas: usize,
    /// Additional cost for empty data transaction
    pub tx_data_zero_gas: usize,
    /// Additional cost for non-empty data transaction
    pub tx_data_non_zero_gas: usize,
    /// Gas price for copying memory
    pub copy_gas: usize,
    /// Price of EXTCODESIZE
    pub extcodesize_gas: usize,
    /// Base price of EXTCODECOPY
    pub extcodecopy_base_gas: usize,
    /// Price of BALANCE
    pub balance_gas: usize,
    /// Price of EXTCODEHASH
    pub extcodehash_gas: usize,
    /// Price of SUICIDE
    pub suicide_gas: usize,
    /// Amount of additional gas to pay when SUICIDE credits a non-existant account
    pub suicide_to_new_account_cost: usize,
    /// If Some(x): let limit = GAS * (x - 1) / x; let CALL's gas = min(requested, limit). let CREATE's gas = limit.
    /// If None: let CALL's gas = (requested > GAS ? [OOG] : GAS). let CREATE's gas = GAS
    pub sub_gas_cap_divisor: Option<usize>,
    /// Don't ever make empty accounts; contracts start with nonce=1. Also, don't charge 25k when sending/suicide zero-value.
    pub no_empty: bool,
    /// Kill empty accounts if touched.
    pub kill_empty: bool,
    /// Blockhash instruction gas cost.
    pub blockhash_gas: usize,
    /// Static Call opcode enabled.
    pub have_static_call: bool,
    /// RETURNDATA and RETURNDATASIZE opcodes enabled.
    pub have_return_data: bool,
    /// SHL, SHR, SAR opcodes enabled.
    pub have_bitwise_shifting: bool,
    /// CHAINID opcode enabled.
    pub have_chain_id: bool,
    /// SELFBALANCE opcode enabled.
    pub have_selfbalance: bool,
    /// BEGINSUB, JUMPSUB and RETURNSUB opcodes enabled.
    pub have_subs: bool,
    /// Kill basic accounts below this balance if touched.
    pub kill_dust: CleanDustMode,
    /// Enable EIP-1283 rules
    pub eip1283: bool,
    /// Enable EIP-1706 rules
    pub eip1706: bool,
    /// VM execution does not increase null signed address nonce if this field is true.
    pub keep_unsigned_nonce: bool,
    /// Wasm extra schedule settings, if wasm activated
    pub wasm: Option<WasmCosts>,
    /// Enable EIP-2929 rules
    pub eip2929: bool,
    /// Enable EIP-2930 rules for optional access list transactions. it depends on EIP-2929
    pub eip2930: bool,
    /// Enable EIP-1559 rules
    pub eip1559: bool,
    /// Elasticity multiplier
    pub eip1559_elasticity_multiplier: usize,
    /// EIP-1559 bumps the gas_limit of fork block by elasticity_multiplier
    pub eip1559_gas_limit_bump: usize,
    /// Enable BASEFEE opcode
    pub eip3198: bool,
    /// Gas used in transaction divided by this number is the maximum refundable amount.
    pub max_refund_quotient: usize,
    // Enable EIP-3541 rule
    pub eip3541: bool,
}

/// Wasm cost table
#[derive(Debug)]
pub struct WasmCosts {
    /// Default opcode cost
    pub regular: u32,
    /// Div operations multiplier.
    pub div: u32,
    /// Div operations multiplier.
    pub mul: u32,
    /// Memory (load/store) operations multiplier.
    pub mem: u32,
    /// General static query of U256 value from env-info
    pub static_u256: u32,
    /// General static query of Address value from env-info
    pub static_address: u32,
    /// Memory stipend. Amount of free memory (in 64kb pages) each contract can use for stack.
    pub initial_mem: u32,
    /// Grow memory cost, per page (64kb)
    pub grow_mem: u32,
    /// Memory copy cost, per byte
    pub memcpy: u32,
    /// Max stack height (native WebAssembly stack limiter)
    pub max_stack_height: u32,
    /// Cost of wasm opcode is calculated as TABLE_ENTRY_COST * `opcodes_mul` / `opcodes_div`
    pub opcodes_mul: u32,
    /// Cost of wasm opcode is calculated as TABLE_ENTRY_COST * `opcodes_mul` / `opcodes_div`
    pub opcodes_div: u32,
    /// Whether create2 extern function is activated.
    pub have_create2: bool,
    /// Whether gasleft extern function is activated.
    pub have_gasleft: bool,
}

impl Default for WasmCosts {
    fn default() -> Self {
        WasmCosts {
            regular: 1,
            div: 16,
            mul: 4,
            mem: 2,
            static_u256: 64,
            static_address: 40,
            initial_mem: 4096,
            grow_mem: 8192,
            memcpy: 1,
            max_stack_height: 64 * 1024,
            opcodes_mul: 3,
            opcodes_div: 8,
            have_create2: false,
            have_gasleft: false,
        }
    }
}

/// Dust accounts cleanup mode.
#[derive(Debug, PartialEq, Eq)]
pub enum CleanDustMode {
    /// Dust cleanup is disabled.
    Off,
    /// Basic dust accounts will be removed.
    BasicOnly,
    /// Basic and contract dust accounts will be removed.
    WithCodeAndStorage,
}

impl Schedule {
    /// Schedule for the Frontier-era of the Ethereum main net.
    pub fn new_frontier() -> Schedule {
        Self::new(false, false, 21000)
    }

    /// Schedule for the Homestead-era of the Ethereum main net.
    pub fn new_homestead() -> Schedule {
        Self::new(true, true, 53000)
    }

    /// Schedule for the post-EIP-150-era of the Ethereum main net.
    pub fn new_post_eip150(
        max_code_size: usize,
        fix_exp: bool,
        no_empty: bool,
        kill_empty: bool,
    ) -> Schedule {
        Schedule {
            exceptional_failed_code_deposit: true,
            have_delegate_call: true,
            have_create2: false,
            have_revert: false,
            have_return_data: false,
            have_bitwise_shifting: false,
            have_chain_id: false,
            have_selfbalance: false,
            have_subs: false,
            have_extcodehash: false,
            stack_limit: 1024,
            max_depth: 1024,
            tier_step_gas: [0, 2, 3, 5, 8, 10, 20, 0],
            exp_gas: 10,
            exp_byte_gas: if fix_exp { 50 } else { 10 },
            sha3_gas: 30,
            sha3_word_gas: 6,
            sload_gas: 200,
            sstore_set_gas: 20000,
            sstore_reset_gas: 5000,
            sstore_refund_gas: 15000,
            jumpdest_gas: 1,
            log_gas: 375,
            log_data_gas: 8,
            log_topic_gas: 375,
            create_gas: 32000,
            call_gas: 700,
            cold_account_access_cost: 0,
            cold_sload_cost: 0,
            warm_storage_read_cost: 0,
            call_stipend: 2300,
            call_value_transfer_gas: 9000,
            call_new_account_gas: 25000,
            suicide_refund_gas: 24000,
            memory_gas: 3,
            quad_coeff_div: 512,
            create_data_gas: 200,
            create_data_limit: max_code_size,
            tx_gas: 21000,
            tx_create_gas: 53000,
            tx_data_zero_gas: 4,
            tx_data_non_zero_gas: 68,
            copy_gas: 3,
            extcodesize_gas: 700,
            extcodecopy_base_gas: 700,
            extcodehash_gas: 400,
            balance_gas: 400,
            suicide_gas: 5000,
            suicide_to_new_account_cost: 25000,
            sub_gas_cap_divisor: Some(64),
            no_empty,
            kill_empty,
            blockhash_gas: 20,
            have_static_call: false,
            kill_dust: CleanDustMode::Off,
            eip1283: false,
            eip1706: false,
            keep_unsigned_nonce: false,
            wasm: None,
            eip2929: false,
            eip2930: false,
            eip1559: false,
            eip1559_elasticity_multiplier: 1,
            eip1559_gas_limit_bump: 1,
            eip3198: false,
            max_refund_quotient: MAX_REFUND_QUOTIENT,
            eip3541: false,
        }
    }

    /// Schedule for the Byzantium fork of the Ethereum main net.
    pub fn new_byzantium() -> Schedule {
        let mut schedule = Self::new_post_eip150(24576, true, true, true);
        schedule.have_create2 = true;
        schedule.have_revert = true;
        schedule.have_static_call = true;
        schedule.have_return_data = true;
        schedule
    }

    /// Schedule for the Constantinople fork of the Ethereum main net.
    pub fn new_constantinople() -> Schedule {
        let mut schedule = Self::new_byzantium();
        schedule.have_bitwise_shifting = true; // EIP 145
        schedule.have_extcodehash = true; // EIP 1052
        schedule
    }

    /// Schedule for the Istanbul fork of the Ethereum main net.
    pub fn new_istanbul() -> Schedule {
        let mut schedule = Self::new_constantinople();
        schedule.have_chain_id = true; // EIP 1344
        schedule.tx_data_non_zero_gas = 16; // EIP 2028
        schedule.sload_gas = 800; // EIP 1884
        schedule.balance_gas = 700; // EIP 1884
        schedule.extcodehash_gas = 700; // EIP 1884
        schedule.have_selfbalance = true; // EIP 1884
        schedule
    }

    /// Schedule for the Yolov3 testnet of the Ethereum main net.
    pub fn new_berlin() -> Schedule {
        let mut schedule = Self::new_istanbul();
        schedule.have_subs = true; // EIP 2315

        schedule.eip1283 = true;
        schedule.eip2929 = true;
        schedule.eip2930 = true;

        schedule.cold_sload_cost = EIP2929_COLD_SLOAD_COST;
        schedule.cold_account_access_cost = EIP2929_COLD_ACCOUNT_ACCESS_COST;
        schedule.warm_storage_read_cost = EIP2929_WARM_STORAGE_READ_COST;

        schedule.sload_gas = EIP2929_WARM_STORAGE_READ_COST;
        schedule.call_gas = EIP2929_COLD_ACCOUNT_ACCESS_COST;
        schedule.balance_gas = EIP2929_COLD_ACCOUNT_ACCESS_COST;
        schedule.extcodecopy_base_gas = EIP2929_COLD_ACCOUNT_ACCESS_COST;
        schedule.extcodehash_gas = EIP2929_COLD_ACCOUNT_ACCESS_COST;
        schedule.extcodesize_gas = EIP2929_COLD_ACCOUNT_ACCESS_COST;
        schedule.sstore_reset_gas = EIP2929_SSTORE_RESET_GAS;

        schedule
    }

    /// Schedule for the London fork of the Ethereum main net.
    pub fn new_london() -> Schedule {
        let mut schedule = Self::new_berlin();

        schedule.eip1559 = true;
        schedule.eip1559_elasticity_multiplier = 2;
        schedule.eip3198 = true;

        schedule.suicide_refund_gas = 0;
        schedule.sstore_refund_gas = EIP3529_SSTORE_CLEARS_SCHEDULE;
        schedule.max_refund_quotient = EIP3529_MAX_REFUND_QUOTIENT;

        schedule.eip3541 = true;

        schedule
    }

    fn new(efcd: bool, hdc: bool, tcg: usize) -> Schedule {
        Schedule {
            exceptional_failed_code_deposit: efcd,
            have_delegate_call: hdc,
            have_create2: false,
            have_revert: false,
            have_return_data: false,
            have_bitwise_shifting: false,
            have_chain_id: false,
            have_selfbalance: false,
            have_subs: false,
            have_extcodehash: false,
            stack_limit: 1024,
            max_depth: 1024,
            tier_step_gas: [0, 2, 3, 5, 8, 10, 20, 0],
            exp_gas: 10,
            exp_byte_gas: 10,
            sha3_gas: 30,
            sha3_word_gas: 6,
            sload_gas: 50,
            sstore_set_gas: 20000,
            sstore_reset_gas: 5000,
            sstore_refund_gas: 15000,
            jumpdest_gas: 1,
            log_gas: 375,
            log_data_gas: 8,
            log_topic_gas: 375,
            create_gas: 32000,
            call_gas: 40,
            cold_account_access_cost: 0,
            cold_sload_cost: 0,
            warm_storage_read_cost: 0,
            call_stipend: 2300,
            call_value_transfer_gas: 9000,
            call_new_account_gas: 25000,
            suicide_refund_gas: 24000,
            memory_gas: 3,
            quad_coeff_div: 512,
            create_data_gas: 200,
            create_data_limit: usize::max_value(),
            tx_gas: 21000,
            tx_create_gas: tcg,
            tx_data_zero_gas: 4,
            tx_data_non_zero_gas: 68,
            copy_gas: 3,
            extcodesize_gas: 20,
            extcodecopy_base_gas: 20,
            extcodehash_gas: 400,
            balance_gas: 20,
            suicide_gas: 0,
            suicide_to_new_account_cost: 0,
            sub_gas_cap_divisor: None,
            no_empty: false,
            kill_empty: false,
            blockhash_gas: 20,
            have_static_call: false,
            kill_dust: CleanDustMode::Off,
            eip1283: false,
            eip1706: false,
            keep_unsigned_nonce: false,
            wasm: None,
            eip2929: false,
            eip2930: false,
            eip1559: false,
            eip1559_elasticity_multiplier: 1,
            eip1559_gas_limit_bump: 1,
            eip3198: false,
            max_refund_quotient: MAX_REFUND_QUOTIENT,
            eip3541: false,
        }
    }

    /// Returns wasm schedule
    ///
    /// May panic if there is no wasm schedule
    pub fn wasm(&self) -> &WasmCosts {
        // *** Prefer PANIC here instead of silently breaking consensus! ***
        self.wasm.as_ref().expect(
            "Wasm schedule expected to exist while checking wasm contract. Misconfigured client?",
        )
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Schedule::new_frontier()
    }
}

#[test]
#[cfg(test)]
fn schedule_evm_assumptions() {
    let s1 = Schedule::new_frontier();
    let s2 = Schedule::new_homestead();

    // To optimize division we assume 2**9 for quad_coeff_div
    assert_eq!(s1.quad_coeff_div, 512);
    assert_eq!(s2.quad_coeff_div, 512);
}
