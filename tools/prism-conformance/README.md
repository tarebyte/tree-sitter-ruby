# Prism conformance

This test uses Prism as a syntax-validity oracle. It requires tree-sitter-ruby
to parse every Prism-valid Ruby file without recovery errors.

From the repository root, check out `ruby/spec` at the revision pinned in
`.github/workflows/ci.yml`, then run:

```sh
PRISM_RUBY_SPEC_ROOT=examples/ruby_spec \
  cargo test --locked --manifest-path tools/prism-conformance/Cargo.toml \
  --test conformance -- --nocapture
```

Known parser failures belong in `test/prism/ruby-spec-allowlist.txt`. Each entry
must identify an existing mismatch and explain why it remains accepted.
