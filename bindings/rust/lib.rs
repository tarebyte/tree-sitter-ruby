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

    /// Returns the doc comment the tags query attaches to each method definition,
    /// keyed by method name.
    fn method_docs(source: &str) -> std::collections::HashMap<String, Option<String>> {
        let config = tree_sitter_tags::TagsConfiguration::new(
            super::LANGUAGE.into(),
            super::TAGS_QUERY,
            super::LOCALS_QUERY,
        )
        .expect("Error loading tags query");
        let mut context = tree_sitter_tags::TagsContext::new();
        let (tags, _) = context
            .generate_tags(&config, source.as_bytes(), None)
            .expect("Error generating tags");

        tags.filter_map(Result::ok)
            .filter(|tag| tag.is_definition)
            .filter(|tag| config.syntax_type_name(tag.syntax_type_id) == "method")
            .map(|tag| {
                (
                    source[tag.name_range.clone()].to_string(),
                    tag.docs.clone(),
                )
            })
            .collect()
    }

    /// A Sorbet `sig` block sits between a method's doc comment and its definition.
    /// The tags query has to look past it, without losing definitions that follow an
    /// unrelated block call and without attaching comments separated by a blank line.
    #[test]
    fn test_tags_attach_doc_comments_across_sorbet_sig() {
        let docs = method_docs(
            r#"
# Converts to a string.
sig { params(a: Integer).returns(String) }
def sigged(a); end

# Returns nothing.
sig do
  void
end
def sigged_do; end

# Builds an instance.
sig { returns(Calc) }
def self.singleton; end

# Documents the block, not the method.

sig { void }
def spaced; end

# Documents the block, not the method.
memoize { :cached }
def after_non_sig_block; end

# Plain documented method.
def plain; end
"#,
        );

        assert_eq!(
            docs.get("sigged"),
            Some(&Some("Converts to a string.".to_string()))
        );
        assert_eq!(
            docs.get("sigged_do"),
            Some(&Some("Returns nothing.".to_string()))
        );
        assert_eq!(
            docs.get("singleton"),
            Some(&Some("Builds an instance.".to_string()))
        );
        assert_eq!(
            docs.get("plain"),
            Some(&Some("Plain documented method.".to_string()))
        );

        // A blank line still separates a comment from the definition it precedes.
        assert_eq!(docs.get("spaced"), Some(&None));

        // Only `sig` is skipped, so this comment stays with the block call, but the
        // definition itself must still be tagged.
        assert_eq!(docs.get("after_non_sig_block"), Some(&None));
    }
}
