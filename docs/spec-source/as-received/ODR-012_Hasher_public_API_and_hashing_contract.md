# ODR-012 — Hasher public API and hashing contract

## Owner approval — 2026-09-13

The owner approved the following completion for the hashing boundary on which
`ArenaMap[K, V]` and ordinary `Map[K, V]` depend.

```ember
interface Hash:
    fn hash(self, mut h: Hasher)

interface Hasher:
    fn write_bytes(mut self, bytes: Span[u8])
    fn write_u8(mut self, x: u8)
    fn write_u16(mut self, x: u16)
    fn write_u32(mut self, x: u32)
    fn write_u64(mut self, x: u64)
    fn write_i8(mut self, x: i8)
    fn write_i16(mut self, x: i16)
    fn write_i32(mut self, x: i32)
    fn write_i64(mut self, x: i64)
    fn write_usize(mut self, x: usize)
    fn write_isize(mut self, x: isize)
    fn finish(self) -> u64
```

`std.collections` publicly exports `Hash` and `Hasher`. Neither is a prelude
name unless separately listed by the existing prelude contract. `Hasher` is a
stateful, move-only hashing context, and `finish()` consumes it. A `Hash`
implementation feeds a deterministic representation into the supplied hasher.
Equal values under `Eq` must produce equal hashes; unequal values need not.
`Hash.hash` must not inspect or retain the hasher after returning, and the
hasher must not retain a supplied `Span` beyond the call.

`std.collections` provides `DefaultHasher` and
`DefaultHasher.new() -> DefaultHasher`. `Map` and `Set` use it unless a future
revision specifies a custom-hasher API. The exact mixing algorithm remains an
implementation detail and is not source-level compatibility behavior.

`ArenaMap[K, V]` and ordinary `Map[K, V]` require `K: Eq + Hash`. A container
may invoke hashing more than once, and callers cannot rely on an invocation
count. Safe APIs must not expose mutable access to resident keys in a manner
that can invalidate equality/hash invariants.

The owner additionally confirmed the read-only key boundary:

```ember
get(key)      -> ref V
get_mut(key)  -> ref mut V
iter()        -> (ref K, ref V)
```

There is no safe `ref mut K` API. This completion is an owner-approved
standard-library API completion, not a new language feature.

## Canonical-mode reconciliation

The submitted signature wrote `finish(self)` while its accompanying normative
prose explicitly said that `finish()` consumes the move-only hasher. Existing
Ember parameter modes make an omitted receiver mode borrowed and spell a
consuming receiver `owned self`. The H3 canonical declaration therefore uses
`finish(owned self) -> u64`; this is a syntax reconciliation preserving the
approved consumption semantics, not an additional semantic decision.

Existing `[MOD-5]` separately lists `Hash` as a prelude name. H3 therefore
preserves that existing export and does not add `Hasher` or `DefaultHasher` to
the prelude.
