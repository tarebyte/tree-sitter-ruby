# tree-sitter-ruby

[![CI][ci]](https://github.com/tarebyte/tree-sitter-ruby/actions/workflows/ci.yml)
[![discord][discord]](https://discord.gg/w7nTvsVJhm)
[![matrix][matrix]](https://matrix.to/#/#tree-sitter-chat:matrix.org)
[![crates][crates]](https://crates.io/crates/tree-sitter-ruby)
[![npm][npm]](https://www.npmjs.com/package/tree-sitter-ruby)
[![pypi][pypi]](https://pypi.org/project/tree-sitter-ruby)

Ruby grammar for [tree-sitter](https://github.com/tree-sitter/tree-sitter).

#### Prism conformance

The parser is checked against [Prism](https://github.com/ruby/prism), Ruby's
reference parser. Focused fixtures verify known valid and invalid syntax, while
a pinned [ruby/spec](https://github.com/ruby/spec) checkout provides broader
coverage with an explicit baseline for existing mismatches.

```sh
cargo test --locked --manifest-path tools/prism-conformance/Cargo.toml
PRISM_RUBY_SPEC_ROOT=examples/ruby_spec \
  cargo test --locked --manifest-path tools/prism-conformance/Cargo.toml \
  --test conformance ruby_spec_matches_prism -- --ignored --nocapture
```

#### References

- [AST Format of the Whitequark parser](https://github.com/whitequark/parser/blob/master/doc/AST_FORMAT.md)

[ci]: https://img.shields.io/github/actions/workflow/status/tarebyte/tree-sitter-ruby/ci.yml?logo=github&label=CI
[discord]: https://img.shields.io/discord/1063097320771698699?logo=discord&label=discord
[matrix]: https://img.shields.io/matrix/tree-sitter-chat%3Amatrix.org?logo=matrix&label=matrix
[npm]: https://img.shields.io/npm/v/tree-sitter-ruby?logo=npm
[crates]: https://img.shields.io/crates/v/tree-sitter-ruby?logo=rust
[pypi]: https://img.shields.io/pypi/v/tree-sitter-ruby?logo=pypi&logoColor=ffd242
