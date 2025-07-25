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

//! ActionParams parser for wasm

use parity_wasm::{
    elements::{self, Deserialize},
    peek_size,
};
use vm;
use wasm_utils::{self, rules};

fn gas_rules(wasm_costs: &vm::WasmCosts) -> rules::Set {
    rules::Set::new(wasm_costs.regular, {
        let mut vals = ::std::collections::BTreeMap::new();
        vals.insert(
            rules::InstructionType::Load,
            rules::Metering::Fixed(wasm_costs.mem),
        );
        vals.insert(
            rules::InstructionType::Store,
            rules::Metering::Fixed(wasm_costs.mem),
        );
        vals.insert(
            rules::InstructionType::Div,
            rules::Metering::Fixed(wasm_costs.div),
        );
        vals.insert(
            rules::InstructionType::Mul,
            rules::Metering::Fixed(wasm_costs.mul),
        );
        vals
    })
    .with_grow_cost(wasm_costs.grow_mem)
    .with_forbidden_floats()
}

/// Splits payload to code and data according to params.params_type, also
/// loads the module instance from payload and injects gas counter according
/// to schedule.
pub fn payload<'a>(
    params: &'a vm::ActionParams,
    wasm_costs: &vm::WasmCosts,
) -> Result<(elements::Module, &'a [u8]), vm::Error> {
    let code = match params.code {
        Some(ref code) => &code[..],
        None => {
            return Err(vm::Error::Wasm("Invalid wasm call".to_owned()));
        }
    };

    let (mut cursor, data_position) = match params.params_type {
        vm::ParamsType::Embedded => {
            let module_size = peek_size(code);
            (::std::io::Cursor::new(&code[..module_size]), module_size)
        }
        vm::ParamsType::Separate => (::std::io::Cursor::new(code), 0),
    };

    let deserialized_module = elements::Module::deserialize(&mut cursor)
        .map_err(|err| vm::Error::Wasm(format!("Error deserializing contract code ({err:?})")))?;

    if deserialized_module
        .memory_section()
        .is_some_and(|ms| !ms.entries().is_empty())
    {
        // According to WebAssembly spec, internal memory is hidden from embedder and should not
        // be interacted with. So we disable this kind of modules at decoding level.
        return Err(vm::Error::Wasm(
            "Malformed wasm module: internal memory".to_string(),
        ));
    }

    let contract_module =
        wasm_utils::inject_gas_counter(deserialized_module, &gas_rules(wasm_costs))
            .map_err(|_| vm::Error::Wasm("Wasm contract error: bytecode invalid".to_string()))?;

    let contract_module =
        wasm_utils::stack_height::inject_limiter(contract_module, wasm_costs.max_stack_height)
            .map_err(|_| {
                vm::Error::Wasm("Wasm contract error: stack limiter failure".to_string())
            })?;

    let data = match params.params_type {
        vm::ParamsType::Embedded => {
            if data_position < code.len() {
                &code[data_position..]
            } else {
                &[]
            }
        }
        vm::ParamsType::Separate => match params.data {
            Some(ref s) => &s[..],
            None => &[],
        },
    };

    Ok((contract_module, data))
}
