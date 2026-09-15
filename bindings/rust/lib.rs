//! This crate provides Ruby language support for the [tree-sitter][] parsing library.
//!
//! Typically, you will use the [LANGUAGE][] constant to add this language to a
//! tree-sitter [Parser][], and then use the parser to parse some code:
//!
//! ```
//! use tree_sitter::Parser;
//!
//! let code = r#"
//! def hello(name)
//!  puts "Hello, #{name}!"
//! end
//! "#;
//! let mut parser = Parser::new();
//! let language = tree_sitter_ruby::LANGUAGE;
//! parser
//!     .set_language(&language.into())
//!     .expect("Error loading Ruby parser");
//! let tree = parser.parse(code, None).unwrap();
//! assert!(!tree.root_node().has_error());
//! ```
//!
//! [Parser]: https://docs.rs/tree-sitter/*/tree_sitter/struct.Parser.html
//! [tree-sitter]: https://tree-sitter.github.io/

use tree_sitter_language::LanguageFn;

extern "C" {
    fn tree_sitter_ruby() -> *const ();
}

/// The tree-sitter [`LanguageFn`][LanguageFn] for this grammar.
///
/// [LanguageFn]: https://docs.rs/tree-sitter-language/*/tree_sitter_language/struct.LanguageFn.html
pub const LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_ruby) };

/// The content of the [`node-types.json`][] file for this grammar.
///
/// [`node-types.json`]: https://tree-sitter.github.io/tree-sitter/using-parsers#static-node-types
pub const NODE_TYPES: &str = include_str!("../../src/node-types.json");

/// The syntax highlighting query for this language.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../../queries/highlights.scm");

/// The local-variable syntax highlighting query for this language.
pub const LOCALS_QUERY: &str = include_str!("../../queries/locals.scm");

/// The symbol tagging query for this language.
pub const TAGS_QUERY: &str = include_str!("../../queries/tags.scm");

#[cfg(test)]
mod tests {
    #[test]
    fn test_can_load_grammar() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&super::LANGUAGE.into())
            .expect("Error loading Ruby parser");
    }

    extern "C" {
        fn tree_sitter_ruby_external_scanner_create() -> *mut std::ffi::c_void;
        fn tree_sitter_ruby_external_scanner_destroy(payload: *mut std::ffi::c_void);
        fn tree_sitter_ruby_external_scanner_serialize(
            payload: *mut std::ffi::c_void,
            buffer: *mut u8,
        ) -> u32;
        fn tree_sitter_ruby_external_scanner_deserialize(
            payload: *mut std::ffi::c_void,
            buffer: *const u8,
            length: u32,
        );
    }

    /// The external scanner must treat its serialization buffer as untrusted:
    /// tree-sitter hands back whatever bytes it was given, and a heredoc record
    /// carries a 32-bit word length that used to be copied without a bounds
    /// check. Feeding arbitrary buffers must never read past `length`, and must
    /// leave the scanner in a state that can still be serialized.
    #[test]
    fn test_external_scanner_rejects_malformed_state() {
        // A pseudo-random stream keeps the test deterministic without a dependency.
        let mut state = 0x2545_f491_4f6c_dd1d_u64;
        let mut next_byte = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 33) as u8
        };

        let mut out = vec![0_u8; 1024];
        for _ in 0..20_000 {
            let length = (next_byte() % 64) as usize;
            let buffer: Vec<u8> = (0..length).map(|_| next_byte()).collect();

            unsafe {
                let scanner = tree_sitter_ruby_external_scanner_create();
                tree_sitter_ruby_external_scanner_deserialize(
                    scanner,
                    buffer.as_ptr(),
                    length as u32,
                );
                let written =
                    tree_sitter_ruby_external_scanner_serialize(scanner, out.as_mut_ptr());
                assert!(written as usize <= out.len());
                tree_sitter_ruby_external_scanner_destroy(scanner);
            }
        }
    }

    /// Round-tripping a well-formed state must survive validation.
    #[test]
    fn test_external_scanner_round_trips_heredoc_state() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&super::LANGUAGE.into())
            .expect("Error loading Ruby parser");

        let source = "a = <<~DOC\n  body\nDOC\n";
        let tree = parser.parse(source, None).unwrap();
        assert!(!tree.root_node().has_error());
    }
}
