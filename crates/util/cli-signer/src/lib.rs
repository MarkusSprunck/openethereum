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

extern crate ethereum_types;
extern crate futures;
extern crate rpassword;

extern crate parity_rpc as rpc;
extern crate parity_rpc_client as client;

use client::signer_client::SignerRpc;
use ethereum_types::U256;
use rpc::signer::ConfirmationRequest;
use std::{
    fs::File,
    io::{stdin, stdout, BufRead, BufReader, Write},
    path::PathBuf,
};

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    futures::executor::block_on(f)
}

fn sign_interactive(signer: &mut SignerRpc, password: &str, request: ConfirmationRequest) {
    print!("\n{request}\nSign this transaction? (y)es/(N)o/(r)eject: ");
    let _ = stdout().flush();
    match BufReader::new(stdin()).lines().next() {
        Some(Ok(line)) => match line.to_lowercase().chars().next() {
            Some('y') => match sign_transaction(signer, request.id, password) {
                Ok(s) | Err(s) => println!("{s}"),
            },
            Some('r') => match reject_transaction(signer, request.id) {
                Ok(s) | Err(s) => println!("{s}"),
            },
            _ => (),
        },
        _ => println!("Could not read from stdin"),
    }
}

fn sign_transactions(signer: &mut SignerRpc, password: String) -> Result<String, String> {
    let reqs = block_on(signer.requests_to_confirm())
        .map_err(|err| format!("{err:?}"))?;
    if reqs.is_empty() {
        return Ok("No transactions in signing queue".to_owned());
    }
    for r in reqs {
        sign_interactive(signer, &password, r);
    }
    Ok("".to_owned())
}

fn list_transactions(signer: &mut SignerRpc) -> Result<String, String> {
    let reqs = block_on(signer.requests_to_confirm())
        .map_err(|err| format!("{err:?}"))?;
    if reqs.is_empty() {
        return Ok("No transactions in signing queue".to_owned());
    }
    Ok(format!(
        "Transaction queue:\n{}",
        reqs.iter()
            .map(|r| format!("{r}"))
            .collect::<Vec<String>>()
            .join("\n")
    ))
}

fn sign_transaction(signer: &mut SignerRpc, id: U256, password: &str) -> Result<String, String> {
    block_on(signer.confirm_request(id, None, None, None, password))
        .map(|u| format!("Signed transaction id: {u:#x}"))
        .map_err(|e| format!("{e:?}"))
}

fn reject_transaction(signer: &mut SignerRpc, id: U256) -> Result<String, String> {
    match block_on(signer.reject_request(id)).map_err(|e| format!("{e:?}"))? {
        true => Ok(format!("Rejected transaction id {id:#x}")),
        false => Err("No such request".to_string()),
    }
}

// cmds

pub fn signer_list(signerport: u16, authfile: PathBuf) -> Result<String, String> {
    let addr = &format!("ws://127.0.0.1:{signerport}");
    let mut signer = SignerRpc::new(addr, &authfile).map_err(|err| format!("{err:?}"))?;
    list_transactions(&mut signer)
}

pub fn signer_reject(
    id: Option<usize>,
    signerport: u16,
    authfile: PathBuf,
) -> Result<String, String> {
    let id = id.ok_or("id required for signer reject".to_string())?;
    let addr = &format!("ws://127.0.0.1:{signerport}");
    let mut signer = SignerRpc::new(addr, &authfile).map_err(|err| format!("{err:?}"))?;
    reject_transaction(&mut signer, U256::from(id))
}

pub fn signer_sign(
    id: Option<usize>,
    pwfile: Option<PathBuf>,
    signerport: u16,
    authfile: PathBuf,
) -> Result<String, String> {
    let password;
    match pwfile {
        Some(pwfile) => match File::open(pwfile) {
            Ok(fd) => match BufReader::new(fd).lines().next() {
                Some(Ok(line)) => password = line,
                _ => return Err("No password in file".to_string()),
            },
            Err(e) => return Err(format!("Could not open password file: {e}")),
        },
        None => {
            password = match rpassword::prompt_password("Password: ") {
                Ok(p) => p,
                Err(e) => return Err(format!("{e}")),
            }
        }
    }

    let addr = &format!("ws://127.0.0.1:{signerport}");
    let mut signer = SignerRpc::new(addr, &authfile).map_err(|err| format!("{err:?}"))?;

    match id {
        Some(id) => sign_transaction(&mut signer, U256::from(id), &password),
        None => sign_transactions(&mut signer, password),
    }
}
