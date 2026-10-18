//! ImHex pattern language ("hexpat") tooling.
//!
//! - [`preprocess`]: `#define` / `#include` / `#pragma` handling.
//! - [`lower`]: tree-sitter parse plus lowering into the [`ast`].
//! - [`reader`]: primitive decoding through `vest_lib` combinators.
//! - [`interp`]: an evaluator that produces an ImHex-style pattern tree.
//! - [`emit`]: translation of the declarative subset to the Vest DSL.
//! - [`emit_asn1`]: translation of the declarative subset to ASN.1 + ACN.

pub mod ast;
pub mod emit;
pub mod emit_asn1;
pub mod error;
pub mod interp;
pub mod lower;
pub mod preprocess;
pub mod reader;

pub use error::{Error, Result};
pub use interp::{Pattern, PatternKind, PatternRef, Runtime, Value};
pub use lower::parse_program;
pub use preprocess::Preprocessor;
