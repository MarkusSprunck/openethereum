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

//! Transaction Execution environment.
use bytes::Bytes;
use ethereum_types::{Address, BigEndianHash, H256, U256};
use executive::*;
use machine::EthereumMachine as Machine;
use state::{Backend as StateBackend, CleanupMode, State, Substate};
use std::{cmp, sync::Arc};
use trace::{Tracer, VMTracer};
use types::transaction::UNSIGNED_SENDER;
use vm::{
    self, AccessList, ActionParams, ActionValue, CallType, ContractCreateResult,
    CreateContractAddress, EnvInfo, Ext, MessageCallResult, ReturnData, Schedule, TrapKind,
};

/// Policy for handling output data on `RETURN` opcode.
pub enum OutputPolicy {
    /// Return reference to fixed sized output.
    /// Used for message calls.
    Return,
    /// Init new contract as soon as `RETURN` is called.
    InitContract,
}

/// Transaction properties that externalities need to know about.
pub struct OriginInfo {
    address: Address,
    origin: Address,
    gas_price: U256,
    value: U256,
}

impl OriginInfo {
    /// Populates origin info from action params.
    pub fn from(params: &ActionParams) -> Self {
        OriginInfo {
            address: params.address,
            origin: params.origin,
            gas_price: params.gas_price,
            value: match params.value {
                ActionValue::Transfer(val) | ActionValue::Apparent(val) => val,
            },
        }
    }
}

/// Implementation of evm Externalities.
pub struct Externalities<'a, T: 'a, V: 'a, B: 'a> {
    state: &'a mut State<B>,
    env_info: &'a EnvInfo,
    depth: usize,
    stack_depth: usize,
    origin_info: &'a OriginInfo,
    substate: &'a mut Substate,
    machine: &'a Machine,
    schedule: &'a Schedule,
    output: OutputPolicy,
    tracer: &'a mut T,
    vm_tracer: &'a mut V,
    static_flag: bool,
}

impl<'a, T: 'a, V: 'a, B: 'a> Externalities<'a, T, V, B>
where
    T: Tracer,
    V: VMTracer,
    B: StateBackend,
{
    /// Basic `Externalities` constructor.
    pub fn new(
        state: &'a mut State<B>,
        env_info: &'a EnvInfo,
        machine: &'a Machine,
        schedule: &'a Schedule,
        depth: usize,
        stack_depth: usize,
        origin_info: &'a OriginInfo,
        substate: &'a mut Substate,
        output: OutputPolicy,
        tracer: &'a mut T,
        vm_tracer: &'a mut V,
        static_flag: bool,
    ) -> Self {
        Externalities {
            state,
            env_info,
            depth,
            stack_depth,
            origin_info,
            substate,
            machine,
            schedule,
            output,
            tracer,
            vm_tracer,
            static_flag,
        }
    }
}

impl<'a, T: 'a, V: 'a, B: 'a> Ext for Externalities<'a, T, V, B>
where
    T: Tracer,
    V: VMTracer,
    B: StateBackend,
{
    fn initial_storage_at(&self, key: &H256) -> vm::Result<H256> {
        if self
            .state
            .is_base_storage_root_unchanged(&self.origin_info.address)?
        {
            self.state
                .checkpoint_storage_at(0, &self.origin_info.address, key)
                .map(|v| v.unwrap_or_default())
                .map_err(Into::into)
        } else {
            warn!(target: "externalities", "Detected existing account {:#x} where a forced contract creation happened.", self.origin_info.address);
            Ok(H256::zero())
        }
    }

    fn storage_at(&self, key: &H256) -> vm::Result<H256> {
        self.state
            .storage_at(&self.origin_info.address, key)
            .map_err(Into::into)
    }

    fn set_storage(&mut self, key: H256, value: H256) -> vm::Result<()> {
        if self.static_flag {
            Err(vm::Error::MutableCallInStaticContext)
        } else {
            self.state
                .set_storage(&self.origin_info.address, key, value)
                .map_err(Into::into)
        }
    }

    fn is_static(&self) -> bool {
        self.static_flag
    }

    fn exists(&self, address: &Address) -> vm::Result<bool> {
        self.state.exists(address).map_err(Into::into)
    }

    fn exists_and_not_null(&self, address: &Address) -> vm::Result<bool> {
        self.state.exists_and_not_null(address).map_err(Into::into)
    }

    fn origin_balance(&self) -> vm::Result<U256> {
        self.balance(&self.origin_info.address)
    }

    fn balance(&self, address: &Address) -> vm::Result<U256> {
        self.state.balance(address).map_err(Into::into)
    }

    fn blockhash(&mut self, number: &U256) -> H256 {
        if self.env_info.number + 256 >= self.machine.params().eip210_transition {
            let blockhash_contract_address = self.machine.params().eip210_contract_address;
            let code_res = self
                .state
                .code(&blockhash_contract_address)
                .and_then(|code| {
                    self.state
                        .code_hash(&blockhash_contract_address)
                        .map(|hash| (code, hash))
                });

            let (code, code_hash) = match code_res {
                Ok((code, hash)) => (code, hash),
                Err(_) => return H256::zero(),
            };

            let data: H256 = BigEndianHash::from_uint(number);

            let params = ActionParams {
                sender: self.origin_info.address,
                address: blockhash_contract_address,
                value: ActionValue::Apparent(self.origin_info.value),
                code_address: blockhash_contract_address,
                origin: self.origin_info.origin,
                gas: self.machine.params().eip210_contract_gas,
                gas_price: 0.into(),
                code,
                code_hash,
                data: Some(data.as_bytes().to_vec()),
                call_type: CallType::Call,
                params_type: vm::ParamsType::Separate,
                access_list: AccessList::default(),
            };

            let mut ex = Executive::new(self.state, self.env_info, self.machine, self.schedule);
            let r = ex.call_with_stack_depth(
                params,
                self.substate,
                self.stack_depth + 1,
                self.tracer,
                self.vm_tracer,
            );
            let output = match &r {
                Ok(ref r) => H256::from_slice(&r.return_data[..32]),
                _ => H256::default(),
            };
            trace!(
                "ext: blockhash contract({}) -> {:?}({}) self.env_info.number={}\n",
                number,
                r,
                output,
                self.env_info.number
            );
            output
        } else {
            // TODO: comment out what this function expects from env_info, since it will produce panics if the latter is inconsistent
            match *number < U256::from(self.env_info.number)
                && number.low_u64() >= cmp::max(256, self.env_info.number) - 256
            {
                true => {
                    let index = self.env_info.number - number.low_u64() - 1;
                    assert!(
                        index < self.env_info.last_hashes.len() as u64,
                        "Inconsistent env_info, should contain at least {:?} last hashes",
                        index + 1
                    );
                    let r = self.env_info.last_hashes[index as usize];
                    trace!(
                        "ext: blockhash({}) -> {} self.env_info.number={}\n",
                        number,
                        r,
                        self.env_info.number
                    );
                    r
                }
                false => {
                    trace!(
                        "ext: blockhash({}) -> null self.env_info.number={}\n",
                        number,
                        self.env_info.number
                    );
                    H256::zero()
                }
            }
        }
    }

    fn create(
        &mut self,
        gas: &U256,
        value: &U256,
        code: &[u8],
        address_scheme: CreateContractAddress,
        trap: bool,
    ) -> ::std::result::Result<ContractCreateResult, TrapKind> {
        // create new contract address
        let (address, code_hash) = match self.state.nonce(&self.origin_info.address) {
            Ok(nonce) => contract_address(address_scheme, &self.origin_info.address, &nonce, code),
            Err(e) => {
                debug!(target: "ext", "Database corruption encountered: {e:?}");
                return Ok(ContractCreateResult::Failed);
            }
        };

        // prepare the params
        let params = ActionParams {
            code_address: address,
            address,
            sender: self.origin_info.address,
            origin: self.origin_info.origin,
            gas: *gas,
            gas_price: self.origin_info.gas_price,
            value: ActionValue::Transfer(*value),
            code: Some(Arc::new(code.to_vec())),
            code_hash,
            data: None,
            call_type: CallType::None,
            params_type: vm::ParamsType::Embedded,
            access_list: self.substate.access_list.clone(),
        };

        if !self.static_flag
            && (!self.schedule.keep_unsigned_nonce || params.sender != UNSIGNED_SENDER)
        {
            if let Err(e) = self.state.inc_nonce(&self.origin_info.address) {
                debug!(target: "ext", "Database corruption encountered: {e:?}");
                return Ok(ContractCreateResult::Failed);
            }
        }

        if trap {
            return Err(TrapKind::Create(params, address));
        }

        // TODO: handle internal error separately
        let mut ex = Executive::from_parent(
            self.state,
            self.env_info,
            self.machine,
            self.schedule,
            self.depth,
            self.static_flag,
        );
        let out = ex.create_with_stack_depth(
            params,
            self.substate,
            self.stack_depth + 1,
            self.tracer,
            self.vm_tracer,
        );
        Ok(into_contract_create_result(out, &address, self.substate))
    }

    fn calc_address(&self, code: &[u8], address_scheme: CreateContractAddress) -> Option<Address> {
        match self.state.nonce(&self.origin_info.address) {
            Ok(nonce) => {
                Some(contract_address(address_scheme, &self.origin_info.address, &nonce, code).0)
            }
            Err(_) => None,
        }
    }

    fn call(
        &mut self,
        gas: &U256,
        sender_address: &Address,
        receive_address: &Address,
        value: Option<U256>,
        data: &[u8],
        code_address: &Address,
        call_type: CallType,
        trap: bool,
    ) -> ::std::result::Result<MessageCallResult, TrapKind> {
        trace!(target: "externalities", "call");

        let code_res = self
            .state
            .code(code_address)
            .and_then(|code| self.state.code_hash(code_address).map(|hash| (code, hash)));

        let (code, code_hash) = match code_res {
            Ok((code, hash)) => (code, hash),
            Err(_) => return Ok(MessageCallResult::Failed),
        };

        let mut params = ActionParams {
            sender: *sender_address,
            address: *receive_address,
            value: ActionValue::Apparent(self.origin_info.value),
            code_address: *code_address,
            origin: self.origin_info.origin,
            gas: *gas,
            gas_price: self.origin_info.gas_price,
            code,
            code_hash,
            data: Some(data.to_vec()),
            call_type,
            params_type: vm::ParamsType::Separate,
            access_list: self.substate.access_list.clone(),
        };

        if let Some(value) = value {
            params.value = ActionValue::Transfer(value);
        }

        if trap {
            return Err(TrapKind::Call(params));
        }

        let mut ex = Executive::from_parent(
            self.state,
            self.env_info,
            self.machine,
            self.schedule,
            self.depth,
            self.static_flag,
        );
        let out = ex.call_with_stack_depth(
            params,
            self.substate,
            self.stack_depth + 1,
            self.tracer,
            self.vm_tracer,
        );
        Ok(into_message_call_result(out))
    }

    fn extcode(&self, address: &Address) -> vm::Result<Option<Arc<Bytes>>> {
        Ok(self.state.code(address)?)
    }

    fn extcodehash(&self, address: &Address) -> vm::Result<Option<H256>> {
        if self.state.exists_and_not_null(address)? {
            Ok(self.state.code_hash(address)?)
        } else {
            Ok(None)
        }
    }

    fn extcodesize(&self, address: &Address) -> vm::Result<Option<usize>> {
        Ok(self.state.code_size(address)?)
    }

    fn ret(self, gas: &U256, data: &ReturnData, apply_state: bool) -> vm::Result<U256>
    where
        Self: Sized,
    {
        match self.output {
            OutputPolicy::Return => Ok(*gas),
            OutputPolicy::InitContract if apply_state => {
                let return_cost =
                    U256::from(data.len()) * U256::from(self.schedule.create_data_gas);
                if return_cost > *gas || data.len() > self.schedule.create_data_limit {
                    return match self.schedule.exceptional_failed_code_deposit {
                        true => Err(vm::Error::OutOfGas),
                        false => Ok(*gas),
                    };
                }
                if self.schedule.eip3541 && data.first() == Some(&0xefu8) {
                    return match self.schedule.exceptional_failed_code_deposit {
                        true => Err(vm::Error::InvalidCode),
                        false => Ok(*gas),
                    };
                }
                self.state
                    .init_code(&self.origin_info.address, data.to_vec())?;
                Ok(*gas - return_cost)
            }
            OutputPolicy::InitContract => Ok(*gas),
        }
    }

    fn log(&mut self, topics: Vec<H256>, data: &[u8]) -> vm::Result<()> {
        use types::log_entry::LogEntry;

        if self.static_flag {
            return Err(vm::Error::MutableCallInStaticContext);
        }

        let address = self.origin_info.address;
        self.substate.logs.push(LogEntry {
            address,
            topics,
            data: data.to_vec(),
        });

        Ok(())
    }

    fn suicide(&mut self, refund_address: &Address) -> vm::Result<()> {
        if self.static_flag {
            return Err(vm::Error::MutableCallInStaticContext);
        }

        let address = self.origin_info.address;
        let balance = self.balance(&address)?;
        if &address == refund_address {
            // TODO [todr] To be consistent with CPP client we set balance to 0 in that case.
            self.state
                .sub_balance(&address, &balance, &mut CleanupMode::NoEmpty)?;
        } else {
            trace!(target: "ext", "Suiciding {address} -> {refund_address} (xfer: {balance})");
            self.state.transfer_balance(
                &address,
                refund_address,
                &balance,
                self.substate.to_cleanup_mode(self.schedule),
            )?;
        }

        self.tracer.trace_suicide(address, balance, *refund_address);
        self.substate.suicides.insert(address);

        Ok(())
    }

    fn schedule(&self) -> &Schedule {
        self.schedule
    }

    fn env_info(&self) -> &EnvInfo {
        self.env_info
    }

    fn chain_id(&self) -> u64 {
        self.machine.params().chain_id
    }

    fn depth(&self) -> usize {
        self.depth
    }

    fn add_sstore_refund(&mut self, value: usize) {
        self.substate.sstore_clears_refund += value as i128;
    }

    fn sub_sstore_refund(&mut self, value: usize) {
        self.substate.sstore_clears_refund -= value as i128;
    }

    fn trace_next_instruction(&mut self, pc: usize, instruction: u8, current_gas: U256) -> bool {
        self.vm_tracer
            .trace_next_instruction(pc, instruction, current_gas)
    }

    fn trace_prepare_execute(
        &mut self,
        pc: usize,
        instruction: u8,
        gas_cost: U256,
        mem_written: Option<(usize, usize)>,
        store_written: Option<(U256, U256)>,
    ) {
        self.vm_tracer
            .trace_prepare_execute(pc, instruction, gas_cost, mem_written, store_written)
    }

    fn trace_failed(&mut self) {
        self.vm_tracer.trace_failed();
    }

    fn trace_executed(&mut self, gas_used: U256, stack_push: &[U256], mem: &[u8]) {
        self.vm_tracer.trace_executed(gas_used, stack_push, mem)
    }

    fn al_is_enabled(&self) -> bool {
        self.substate.access_list.is_enabled()
    }

    fn al_contains_storage_key(&self, address: &Address, key: &H256) -> bool {
        self.substate.access_list.contains_storage_key(address, key)
    }

    fn al_insert_storage_key(&mut self, address: Address, key: H256) {
        self.substate.access_list.insert_storage_key(address, key)
    }

    fn al_contains_address(&self, address: &Address) -> bool {
        self.substate.access_list.contains_address(address)
    }

    fn al_insert_address(&mut self, address: Address) {
        self.substate.access_list.insert_address(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethereum_types::{Address, U256};
    use evm::{CallType, EnvInfo, Ext};
    use state::{State, Substate};
    use std::str::FromStr;
    use test_helpers::get_temp_state;
    use trace::{NoopTracer, NoopVMTracer};

    fn get_test_origin() -> OriginInfo {
        OriginInfo {
            address: Address::zero(),
            origin: Address::zero(),
            gas_price: U256::zero(),
            value: U256::zero(),
        }
    }

    fn get_test_env_info() -> EnvInfo {
        EnvInfo {
            number: 100,
            author: Address::zero(),
            timestamp: 0,
            difficulty: 0.into(),
            last_hashes: Arc::new(vec![]),
            gas_used: 0.into(),
            gas_limit: 0.into(),
            base_fee: None,
        }
    }

    struct TestSetup {
        state: State<::state_db::StateDB>,
        machine: ::machine::EthereumMachine,
        schedule: Schedule,
        sub_state: Substate,
        env_info: EnvInfo,
    }

    impl Default for TestSetup {
        fn default() -> Self {
            TestSetup::new()
        }
    }

    impl TestSetup {
        fn new() -> Self {
            let machine = ::spec::Spec::new_test_machine();
            let env_info = get_test_env_info();
            let schedule = machine.schedule(env_info.number);
            TestSetup {
                state: get_temp_state(),
                schedule,
                machine,
                sub_state: Substate::new(),
                env_info,
            }
        }
    }

    #[test]
    fn can_be_created() {
        let mut setup = TestSetup::new();
        let state = &mut setup.state;
        let mut tracer = NoopTracer;
        let mut vm_tracer = NoopVMTracer;
        let origin_info = get_test_origin();

        let ext = Externalities::new(
            state,
            &setup.env_info,
            &setup.machine,
            &setup.schedule,
            0,
            0,
            &origin_info,
            &mut setup.sub_state,
            OutputPolicy::InitContract,
            &mut tracer,
            &mut vm_tracer,
            false,
        );

        assert_eq!(ext.env_info().number, 100);
    }

    #[test]
    fn can_return_block_hash_no_env() {
        let mut setup = TestSetup::new();
        let state = &mut setup.state;
        let mut tracer = NoopTracer;
        let mut vm_tracer = NoopVMTracer;
        let origin_info = get_test_origin();

        let mut ext = Externalities::new(
            state,
            &setup.env_info,
            &setup.machine,
            &setup.schedule,
            0,
            0,
            &origin_info,
            &mut setup.sub_state,
            OutputPolicy::InitContract,
            &mut tracer,
            &mut vm_tracer,
            false,
        );

        let hash = ext.blockhash(
            &"0000000000000000000000000000000000000000000000000000000000120000"
                .parse::<U256>()
                .unwrap(),
        );

        assert_eq!(hash, H256::zero());
    }

    #[test]
    fn can_return_block_hash() {
        let test_hash =
            H256::from_str("afafafafafafafafafafafbcbcbcbcbcbcbcbcbcbeeeeeeeeeeeeedddddddddd")
                .unwrap();
        let test_env_number = 0x120001;

        let mut setup = TestSetup::new();
        {
            let env_info = &mut setup.env_info;
            env_info.number = test_env_number;
            let mut last_hashes = (*env_info.last_hashes).clone();
            last_hashes.push(test_hash);
            env_info.last_hashes = Arc::new(last_hashes);
        }
        let state = &mut setup.state;
        let mut tracer = NoopTracer;
        let mut vm_tracer = NoopVMTracer;
        let origin_info = get_test_origin();

        let mut ext = Externalities::new(
            state,
            &setup.env_info,
            &setup.machine,
            &setup.schedule,
            0,
            0,
            &origin_info,
            &mut setup.sub_state,
            OutputPolicy::InitContract,
            &mut tracer,
            &mut vm_tracer,
            false,
        );

        let hash = ext.blockhash(
            &"0000000000000000000000000000000000000000000000000000000000120000"
                .parse::<U256>()
                .unwrap(),
        );

        assert_eq!(test_hash, hash);
    }

    #[test]
    #[should_panic]
    fn can_call_fail_empty() {
        let mut setup = TestSetup::new();
        let state = &mut setup.state;
        let mut tracer = NoopTracer;
        let mut vm_tracer = NoopVMTracer;
        let origin_info = get_test_origin();

        let mut ext = Externalities::new(
            state,
            &setup.env_info,
            &setup.machine,
            &setup.schedule,
            0,
            0,
            &origin_info,
            &mut setup.sub_state,
            OutputPolicy::InitContract,
            &mut tracer,
            &mut vm_tracer,
            false,
        );

        // this should panic because we have no balance on any account
        ext.call(
            &"0000000000000000000000000000000000000000000000000000000000120000"
                .parse::<U256>()
                .unwrap(),
            &Address::default(),
            &Address::default(),
            Some(
                "0000000000000000000000000000000000000000000000000000000000150000"
                    .parse::<U256>()
                    .unwrap(),
            ),
            &[],
            &Address::default(),
            CallType::Call,
            false,
        )
        .ok()
        .unwrap();
    }

    #[test]
    fn can_log() {
        let log_data = vec![120u8, 110u8];
        let log_topics = vec![H256::from_str(
            "af0fa234a6af46afa23faf23bcbc1c1cb4bcb7bcbe7e7e7ee3ee2edddddddddd",
        )
        .unwrap()];

        let mut setup = TestSetup::new();
        let state = &mut setup.state;
        let mut tracer = NoopTracer;
        let mut vm_tracer = NoopVMTracer;
        let origin_info = get_test_origin();

        {
            let mut ext = Externalities::new(
                state,
                &setup.env_info,
                &setup.machine,
                &setup.schedule,
                0,
                0,
                &origin_info,
                &mut setup.sub_state,
                OutputPolicy::InitContract,
                &mut tracer,
                &mut vm_tracer,
                false,
            );
            ext.log(log_topics, &log_data).unwrap();
        }

        assert_eq!(setup.sub_state.logs.len(), 1);
    }

    #[test]
    fn can_suicide() {
        let refund_account = &Address::default();

        let mut setup = TestSetup::new();
        let state = &mut setup.state;
        let mut tracer = NoopTracer;
        let mut vm_tracer = NoopVMTracer;
        let origin_info = get_test_origin();

        {
            let mut ext = Externalities::new(
                state,
                &setup.env_info,
                &setup.machine,
                &setup.schedule,
                0,
                0,
                &origin_info,
                &mut setup.sub_state,
                OutputPolicy::InitContract,
                &mut tracer,
                &mut vm_tracer,
                false,
            );
            ext.suicide(refund_account).unwrap();
        }

        assert_eq!(setup.sub_state.suicides.len(), 1);
    }

    #[test]
    fn can_create() {
        use std::str::FromStr;

        let mut setup = TestSetup::new();
        let state = &mut setup.state;
        let mut tracer = NoopTracer;
        let mut vm_tracer = NoopVMTracer;
        let origin_info = get_test_origin();

        let address = {
            let mut ext = Externalities::new(
                state,
                &setup.env_info,
                &setup.machine,
                &setup.schedule,
                0,
                0,
                &origin_info,
                &mut setup.sub_state,
                OutputPolicy::InitContract,
                &mut tracer,
                &mut vm_tracer,
                false,
            );
            match ext.create(
                &U256::max_value(),
                &U256::zero(),
                &[],
                CreateContractAddress::FromSenderAndNonce,
                false,
            ) {
                Ok(ContractCreateResult::Created(address, _)) => address,
                _ => panic!("Test create failed; expected Created, got Failed/Reverted."),
            }
        };

        assert_eq!(
            address,
            Address::from_str("bd770416a3345f91e4b34576cb804a576fa48eb1").unwrap()
        );
    }

    #[test]
    fn can_create2() {
        use std::str::FromStr;

        let mut setup = TestSetup::new();
        let state = &mut setup.state;
        let mut tracer = NoopTracer;
        let mut vm_tracer = NoopVMTracer;
        let origin_info = get_test_origin();

        let address = {
            let mut ext = Externalities::new(
                state,
                &setup.env_info,
                &setup.machine,
                &setup.schedule,
                0,
                0,
                &origin_info,
                &mut setup.sub_state,
                OutputPolicy::InitContract,
                &mut tracer,
                &mut vm_tracer,
                false,
            );

            match ext.create(
                &U256::max_value(),
                &U256::zero(),
                &[],
                CreateContractAddress::FromSenderSaltAndCodeHash(H256::default()),
                false,
            ) {
                Ok(ContractCreateResult::Created(address, _)) => address,
                _ => panic!("Test create failed; expected Created, got Failed/Reverted."),
            }
        };

        assert_eq!(
            address,
            Address::from_str("e33c0c7f7df4809055c3eba6c09cfe4baf1bd9e0").unwrap()
        );
    }

    #[test]
    fn eip_3541() {
        let call_ret = |schedule: Schedule, data: &ReturnData| {
            let mut setup = TestSetup::default();
            setup.schedule = schedule;
            let mut tracer = NoopTracer;
            let mut vm_tracer = NoopVMTracer;
            let origin = get_test_origin();
            let ext = Externalities::new(
                &mut setup.state,
                &setup.env_info,
                &setup.machine,
                &setup.schedule,
                0,
                0,
                &origin,
                &mut setup.sub_state,
                OutputPolicy::InitContract,
                &mut tracer,
                &mut vm_tracer,
                false,
            );
            ext.ret(&U256::from(10000), data, true)
        };

        let data = ReturnData::new(vec![0xefu8], 0, 1);

        let result = call_ret(Schedule::new_berlin(), &data);
        assert!(result.is_ok());

        let result = call_ret(Schedule::new_london(), &data);
        assert!(result.is_err());

        let data = ReturnData::new(vec![0xefu8, 0x00u8, 0x00u8], 0, 3);
        let result = call_ret(Schedule::new_london(), &data);
        assert!(result.is_err());

        let data = ReturnData::new(vec![0xee], 0, 1);
        let result = call_ret(Schedule::new_london(), &data);
        assert!(result.is_ok());
    }
}
