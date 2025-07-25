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

//! Custom panic hook with bug report link

extern crate backtrace;

use backtrace::Backtrace;
use std::{
    panic::{self, PanicHookInfo},
    process, thread,
};

/// Set the panic hook to write to stderr and abort the process when a panic happens.
pub fn set_abort() {
    set_with(|msg| {
        eprintln!("{msg}");
        process::abort()
    });
}

/// Set the panic hook with a closure to be called. The closure receives the panic message.
///
/// Depending on how OpenEthereum was compiled, after the closure has been executed, either the process
/// aborts or unwinding starts.
///
/// If you panic within the closure, a double panic happens and the process will stop.
pub fn set_with<F>(f: F)
where
    F: Fn(&str) + Send + Sync + 'static,
{
    panic::set_hook(Box::new(move |info| {
        let msg = gen_panic_msg(info);
        f(&msg);
    }));
}

static ABOUT_PANIC: &str = "
This is a bug. Please report it at:

    https://github.com/openethereum/openethereum/issues/new
";

fn gen_panic_msg(info: &PanicHookInfo) -> String {
    let location = info.location();
    let file = location.as_ref().map(|l| l.file()).unwrap_or("<unknown>");
    let line = location.as_ref().map(|l| l.line()).unwrap_or(0);

    let msg = match info.payload().downcast_ref::<&'static str>() {
        Some(s) => *s,
        None => match info.payload().downcast_ref::<String>() {
            Some(s) => &s[..],
            None => "Box<Any>",
        },
    };

    let thread = thread::current();
    let name = thread.name().unwrap_or("<unnamed>");

    let backtrace = Backtrace::new();

    format!(
        r#"

====================

{backtrace:?}

Thread '{name}' panicked at '{msg}', {file}:{line}
{ABOUT_PANIC}
"#
    )
}
