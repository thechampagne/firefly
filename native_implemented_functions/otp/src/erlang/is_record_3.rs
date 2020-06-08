// wasm32 proptest cannot be compiled at the same time as non-wasm32 proptest, so disable tests that
// use proptest completely for wasm32
//
// See https://github.com/rust-lang/cargo/issues/4866
#[cfg(all(not(target_arch = "wasm32"), test))]
mod test;

use liblumen_alloc::erts::exception;
use liblumen_alloc::erts::term::prelude::Term;

use crate::erlang::is_record;

#[native_implemented::function(is_record/3)]
pub fn result(term: Term, record_tag: Term, size: Term) -> exception::Result<Term> {
    is_record(term, record_tag, Some(size))
}
