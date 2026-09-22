# Migrating to Ember 0.9.8

**State as of 2026-09-22.** This guide covers the additive 0.9.8 language
revision selected by the owner to close ODR-017. The current frozen development
target is `docs/spec-source/Ember_v0.9.8_Hardened_2.md`; its `Hardened_2` delta
is a CLI/tooling hardening and does not change this source migration. The
adopted repository-normative source remains `ember-spec.md`
(`0.8.5_Hardened_1`) until the target passes its adoption gates and is explicitly
installed.

This document is a migration aid, not a second specification. The frozen target
and ADR-037 are authoritative for the 0.9.8 ownership and borrowing contract.

## 1. Select the 0.9.8 language contract

Set the package default in `ember.toml`:

```toml
[package]
language = "0.9.8"
```

Or select it for one module with a first-line directive:

```ember
#! language "0.9.8"
```

When both are present, they must resolve to the same exact language contract.
`0.9.8` is additive over `0.9.7`: an earlier valid program remains valid. An
explicit `0.9`, `0.9.5`, `0.9.6`, or `0.9.7` selector deliberately retains that
older contract; it does not silently opt a module into 0.9.8 ownership
semantics.

## 2. Use the canonical `Shared[T]` owner API

0.9.8 defines the previously unspecified `Shared[T]` surface. Construct the
strong owner with `Shared(value)`, borrow it with `get()`, and take a mutable
borrow with `get_mut()`:

```ember
struct Counter:
    value: i32

fn main():
    shared = Shared(Counter(40))
    reader = shared.get()             # ref Counter
    seen = reader.value

    writer = shared.get_mut()         # ref mut Counter
    writer.value = seen + 2
    println(writer.value)
```

`Shared[T]` is a counted owner: copying it retains the same allocation and its
last strong release runs `T`'s ordinary destruction. `get()` is an ordinary
shared borrow; it neither copies nor retains the owner. `get_mut(mut self)` is
an ordinary mutable borrow plus the existing dynamic-exclusivity check. It does
not require that the handle be uniquely owned. As with other non-lexical loans,
the reader or writer remains live only through its last use; overlapping access
through a second alias is rejected or fails the required dynamic check.

## 3. Spell the weak owner exactly

`Weak` is the weak form of a **counted owner**, not a separate payload-pointer
family. The existing class form is unchanged:

```ember
class Node:
    value: i32

node = Node(7)
class_weak: Weak[Node] = Weak(node)
```

The weak companion of a `Shared[T]` is explicitly `Weak[Shared[T]]`:

```ember
struct Token:
    value: i32

fn main():
    owner = Shared(Token(7))
    weak: Weak[Shared[Token]] = Weak(owner)

    match weak.upgrade():
        Some(live):
            value = live.get()
            println(value.value)
        None:
            pass
```

Do not write `Weak[Token]` to mean a weak `Shared[Token]`: that would be
ambiguous with the established class-owner spelling. Use `Weak[Shared[Token]]`.
`Weak[O].empty()`, copying, dropping, and `upgrade() -> Option[O]` use the
existing weak-count and no-resurrection model. A weak owner does not keep its
allocation alive; an upgrade after final strong release returns `None`.

`Weak[Class]`, `Weak[Shared[T]]`, and C++ bridge owners remain distinct forms.
They do not interconvert.

## 4. What existing projects must change

There is no mandatory source rewrite for an existing valid 0.9.7 project.
0.9.8 makes these formerly undefined public operations available under one
defined contract. New shared-value code should use the exact forms above:

| Need | 0.9.8 form |
|---|---|
| create a shared value owner | `Shared(value)` |
| read a shared payload | `owner.get()` -> `ref T` |
| mutably access a shared payload | `owner.get_mut()` -> `ref mut T` |
| weakly observe a class owner | `Weak[Class]` |
| weakly observe a shared value owner | `Weak[Shared[T]]` |

Keep the version selector at its older value if the project intentionally needs
the earlier language contract. Do not substitute an invented `Weak[T]` API or a
unique-owner requirement for `get_mut`; neither is the 0.9.8 contract.

## 5. Evidence and implementation boundary

The implementation and conformance work for this revision belongs to `[HEAP-3]`
through `[HEAP-7]`, `[WK-11]` through `[WK-14]`, and `[TST-26]`. Current
executable conformance evidence is under `HEAP-5` through `HEAP-7` and `WK-11`
through `WK-14`; it covers reader/writer access, dynamic exclusivity, final
release, weak upgrade, no resurrection, and the non-interoperation boundary.
`[HEAP-3]` and `[HEAP-4]` remain part of the target's required strong-owner
surface and must not be inferred complete merely because selector recognition
or the later probes pass.

`0.9.8_Hardened_2` additionally records ODR-018's cycle-explanation command.
That is a tooling hardening: it changes neither the 0.9.8 source selector nor
the `Shared`/`Weak` migration described here.
