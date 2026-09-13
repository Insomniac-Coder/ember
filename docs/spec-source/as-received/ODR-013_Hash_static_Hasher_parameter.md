# ODR-013 — Hash uses a statically generic Hasher parameter

## Owner ruling — 2026-09-13

`Hash.hash` uses a statically generic hasher parameter. Its canonical Ember
signature is:

```ember
interface Hash:
    fn hash[H: Hasher](self, mut h: H)
```

The method must not use bare `Hasher` as a concrete parameter type, because
Ember requires either an explicit generic bound or a `dyn` representation for
interface-typed values.

For ordinary hashing operations, `H` is inferred from the concrete hasher
supplied by the caller and is normally monomorphized. The `Hash` contract
therefore introduces no mandatory dynamic dispatch.

`dyn Hasher` may be used by a separately specified API that intentionally
requires runtime polymorphism, but `Hash.hash` itself is not such an API.

`std.collections` provides the default concrete hasher `DefaultHasher`, with:

```ember
DefaultHasher.new() -> DefaultHasher
```

`DefaultHasher` implements `Hasher`. `Map` and `Set` use `DefaultHasher`
unless a future language/library revision explicitly specifies a custom-hasher
API.

## Transport normalization

The supplied patch included angle brackets around the ordinary parameter list
in `fn hash[H: Hasher](<self, mut h: H>)`. Existing Ember grammar writes an
ordinary function parameter list directly in parentheses. The canonical
signature above removes those transport/diff markers without changing the
owner-selected generic-bound semantics.
