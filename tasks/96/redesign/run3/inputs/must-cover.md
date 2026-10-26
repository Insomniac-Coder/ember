# Must-cover words: every one of these must be judged by at least one reader (01-04)

Built by the lead before launch from `docs/spec-source/Ember_v0.9.10_Hardened_35.md`. A reader judges
each one that appears in its slice; the Builder (05) judges any that no reader judged.

## Reserved keywords (II.4, 47)

`and` `as` `break` `class` `comptime` `const` `continue` `defer` `dyn` `elif` `else` `enum` `extern` `false` `fn` `for` `if` `implements` `import` `in` `interface` `is` `let` `match` `mut` `not` `or` `override` `owned` `pass` `pub` `ref` `return` `self` `Self` `static` `struct` `super` `true` `type` `unsafe` `virtual` `void` `where` `while` `with` `yield`

## Contextual keywords and reserved-for-the-future words (II.4, [LEX-15])

`abstract` `open` `from` `extend` `gen` `once` `safe` `some` `c` `cpp` `language` `threads` `overlay` `rename` `hide` `union` `async` `await` `macro`

## Attributes (every `@name` in lines 1-6930, with how often it appears)

`@sync` (35) `@ffi` (34) `@derive` (27) `@noalloc` (25) `@parallel` (15) `@static_safe` (14) `@nopanic` (13) `@layout` (13) `@deterministic` (12) `@simd` (11) `@fastmath` (11) `@ser` (10) `@fp` (10) `@borrows` (10) `@noreload` (9) `@must_drop` (9) `@export` (9) `@view` (7) `@reflect` (7) `@nosync` (7) `@repr` (6) `@packed` (6) `@overflow` (6) `@nolock` (6) `@noblock` (6) `@align` (6) `@realtime` (5) `@noio` (5) `@inline` (5) `@attribute` (5) `@unroll` (4) `@safety` (4) `@export_table` (4) `@error` (4) `@test` (3) `@renamed_from` (3) `@reinit_on_reload` (3) `@never_specialize` (3) `@gpu` (3) `@assume_noalloc` (3) `@allow` (3) `@soa` (2) `@should_panic` (2) `@reloadable` (2) `@non_exhaustive` (2) `@noinline` (2) `@no_derive` (2) `@must_use` (2) `@hot` (2) `@grade` (2) `@gpu_layout` (2) `@from` (2) `@deprecated` (2) `@default` (2) `@component` (2) `@cold` (2) `@always_specialize` (2) `@allow_reload_terminate` (2) `@prelude` (1) `@no_runtime_checks` (1) `@name` (1) `@dataclass` (1) `@bench` (1) `@allocator` (1) 

(Some, such as `@dataclass`, may be Python's, quoted in Appendix E: say so.)

## Library words that act as language words

`Box` `Shared` `Weak` `RefCell` `Cell` `ref` `ref mut` `Span` `MutSpan` `str` `String` `.clone()` `.copy()` `mem.take` `mem.replace` `mem.swap` `mem.drop` `Option` `None` `Mutex` `RwLock` `Retained` `Pool` `Handle` `dyn` (`Box[dyn I]`) `some I` `Send` `Sync` `unsafe` blocks and `unsafe fn`
