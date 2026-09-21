//! Rust bindings for the ImHex pattern language ("hexpat") tree-sitter grammar.
//!
//! ```
//! let mut parser = tree_sitter::Parser::new();
//! parser
//!     .set_language(&tree_sitter_hexpat::LANGUAGE.into())
//!     .expect("hexpat grammar loads");
//! let tree = parser.parse("struct Header { u32 magic; };", None).unwrap();
//! assert!(!tree.root_node().has_error());
//! ```

use tree_sitter_language::LanguageFn;

unsafe extern "C" {
    fn tree_sitter_hexpat() -> *const ();
}

/// The tree-sitter [`LanguageFn`] for hexpat.
pub const LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_hexpat) };

/// The `node-types.json` describing every node kind and its fields.
pub const NODE_TYPES: &str = include_str!("../../src/node-types.json");

#[cfg(test)]
mod tests {
    #[test]
    fn test_can_load_grammar() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&super::LANGUAGE.into())
            .expect("Error loading hexpat parser");
    }
}
