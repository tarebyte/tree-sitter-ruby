; Method definitions

; A Sorbet `sig` block sits between a method's doc comment and its definition, so
; the comment is not adjacent to the definition and the general pattern below
; cannot attach it. Anchoring adjacency on the `sig` call instead recovers the
; doc comment. This must precede the general pattern: when several patterns match
; the same node, tree-sitter-tags keeps the one from the earliest pattern.
(
  (comment)* @doc
  .
  (call
    method: (identifier) @reference.call
    block: [
      (block)
      (do_block)
    ])
  .
  [
    (method
      name: (_) @name) @definition.method
    (singleton_method
      name: (_) @name) @definition.method
  ]
  (#eq? @reference.call "sig")
  (#strip! @doc "^#\\s*")
  (#select-adjacent! @doc @reference.call)
)

(
  (comment)* @doc
  .
  [
    (method
      name: (_) @name) @definition.method
    (singleton_method
      name: (_) @name) @definition.method
  ]
  (#strip! @doc "^#\\s*")
  (#select-adjacent! @doc @definition.method)
)

(alias
  name: (_) @name) @definition.method

(setter
  (identifier) @ignore)

; Class definitions

(
  (comment)* @doc
  .
  [
    (class
      name: [
        (constant) @name
        (scope_resolution
          name: (_) @name)
      ]) @definition.class
    (singleton_class
      value: [
        (constant) @name
        (scope_resolution
          name: (_) @name)
      ]) @definition.class
  ]
  (#strip! @doc "^#\\s*")
  (#select-adjacent! @doc @definition.class)
)

; Module definitions

(
  (module
    name: [
      (constant) @name
      (scope_resolution
        name: (_) @name)
    ]) @definition.module
)

; Calls

(call method: (identifier) @name) @reference.call

(
  [(identifier) (constant)] @name @reference.call
  (#is-not? local)
  (#not-match? @name "^(lambda|load|require|require_relative|__FILE__|__LINE__)$")
)
