# Specification amendments

Every difference between `docs/spec-source/as-received/Ember_v0.8.3_spec.md`
and `docs/spec-source/ember-spec.md`, with the reason for it.

**These are the only permitted edits to the document.** The standing rule is
that the specification is the contract and is never altered to make the
compiler agree with it; where the two disagree, the compiler is what changes.
An entry belongs here only when the owner has authorised it explicitly, and
each says which instruction authorised it.

`as-received/` is never edited under any circumstance. `docs/spec/` is
generated from `ember-spec.md` and `tools/split_spec.py --check` fails CI if
anyone hand-edits it.

## History

**2026-09-09, morning — fourteen edits, all reverted.** Passages had been
rewritten in place so the document would agree with the readings recorded in
`docs/spec-errata.md`. That made the document and the errata agree at the cost
of the document no longer being the owner's, which is the wrong trade. Every
one was reverted; the file was byte-identical to as-received again
(`2bdffee6510b8668cf828185266efedb`) before the amendments below were applied.
The reverted set is listed in the commit *"Revert every edit to the
specification; the document is the owner's"*.

**2026-09-09, afternoon — ten amendments, below.** Authorised by the owner:
*"update the v0.8.3 spec doc with the missing implementation details, if there
was ambiguity that led to these or if there was any missing details that would
have made implementation much straightforward"*, together with the specific
changes named in the combined fix list (items 1, 3, 6, 7, 8 and 17).

Each amendment **adds** to a rule and removes nothing. No new rule id is
introduced, so `tools/rule_index.py` reports the same 833 rules and no new
conformance directory is owed. Each is marked in the text with
*(clarified 2026-09-09; see `docs/spec-amendments.md`)* so a reader of the
document can tell owner text from clarification at a glance.

---

## The ten

| # | Rule | What was missing | What it cost |
|---|---|---|---|
| A1 | `[SPN-1]` | that the coercion **takes a borrow** | D-022 — a use-after-free reachable from Safe Ember |
| A2 | `[BRW-1]` | what assigning to a reference local means | D-026 — a write through a shared `ref`, caught only by clang |
| A3 | `[RNG-3]` | where `RangeError` lives | D-025 — the rule's own signature would not compile |
| A4 | `[LT-2]` | whether `E3064` is reachable in v1 | an open question the ledger could not close |
| A5 | `[CLO-3]` | how a `fn(A) -> R` parameter is realised | ADR-018 — a function pointer, so capturing lambdas are rejected |
| A6 | `[FN-1]` | a `mut` parameter whose type is itself a borrow | ERR-041 — the document's own example is `E2140` |
| A7 | `[FFI-17d]` | any definition of `@ffi(no_virtual_dtor)` | ERR-034 — an attribute named and defined nowhere |
| A8 | `fn_header` | a production for `extern "C" fn` | ERR-036 — XVI.10's example does not parse |
| A9 | `extern_class` | a production for `extern class` | ERR-037 — `[FFI-39]` rests on syntax that does not exist |
| A10 | `item_body` | admitting A9's production | — |

---

### A1 — `[SPN-1]`: the coercion takes a borrow

**Authorised by** fix-list item 1, which gives the substance: *"A coercion from
`Array[T]` to `Span[T]` or `MutSpan[T]` creates a borrow of the source array.
The resulting view's region is tied to the source according to the normal
borrow rules."*

**Why it was needed.** The rule said only that `Array[T]` *coerces to*
`Span[T]`, and "coerces" is the same word the document uses for numeric
widening and range erasure, neither of which borrows anything. Implemented as
a conversion, this compiled:

```ember
a = Array[i32]([1, 2, 3])
v: Span[i32] = a
a.push(4)        # reallocates
print(v[0])      # reads freed memory
```

No `unsafe` appears in it. `[PHIL-10]` says that cannot happen, and the borrow
rules already had every mechanism needed to stop it — nothing said to use them.

The amendment also states the three spellings are one construction, and
requires the borrow to be **explicit in the implementation's IR**. That last
clause is the part that generalises: an implied borrow is invisible to the
analysis that would catch this, so "the coercion implies a borrow" is not a
sufficient instruction to an implementer.

### A2 — `[BRW-1]`: a reference local is not re-seatable

**Authorised by** the general instruction: this is a missing detail, not a
contradiction. No rule anywhere said what `r = e` means when `r` is a
reference.

**Why it was needed.** Two readings were available — re-seat `r`, or write
through it — and they differ observably. ADR-010 chose write-through, because
it is what makes regions insensitive to program location. That choice is sound
for `ref mut` and is *exactly the forbidden write* for a shared `ref`, and
because no rule stated the choice, the shared case was never checked:

```ember
r: ref i32 = ref x
r = 99            # accepted by every check in the compiler
```

It emitted `(*_2) = 99` and **clang** refused it — `read-only variable is not
assignable`. Aliasing-XOR-mutability was being upheld by the backend happening
to emit `const`, not by the language.

### A3 — `[RNG-3]`: `RangeError` is a prelude type

**Authorised by** fix-list item 12, *"`RangeError` must become a real
user-facing type… resolvable by the compiler; usable in `Result`; documented;
testable; stable for public APIs"*, which prefers the prelude.

**Why it was needed.** The rule writes the signature
`T.checked(v) -> Result[T, RangeError]` and no rule declares `RangeError`. It
was therefore synthesised on first use — which meant it did not exist while
*signatures* were being collected, and the rule's own worked example did not
compile:

```text
error[E1010]: cannot find type `RangeError` in this scope
```

The amendment adds the two facts an implementer needs and neither of which was
derivable: that it is a prelude type rather than a `std` declaration (because
`checked` is a language construction, not a library function, so it cannot
depend on an import), and that the name must resolve during signature
collection.

### A4 — `[LT-2]`: `E3064` is unreachable in v1

**Authorised by** fix-list item 6, which asks for the determination to be made
and recorded, and says not to change the rule if the narrowing is correct.

**The determination: the narrowing is correct.** The rule says the intersection
is *taken*, so construction never fails; and `[LT-1]`'s elision extends the
same treatment to a returned view, which is why `pick(p, q)` returning its
first argument still borrows both. The amendment records that, and states what
`E3064` is actually for — a program that **demands** two independent regions —
and that no v1 source can express one, since named lifetimes are v2 (`[LT-6]`)
and `[TYP-15a]`'s `BorrowList`/`ViewList` are the only construct that could.

A v1 implementation is therefore conforming with the code registered and never
emitted. Without this, every implementer must re-derive the same conclusion or
leave a rule apparently unimplemented.

### A5 — `[CLO-3]`: `fn(A) -> R` is a bound, not a representation

**Authorised by** fix-list item 3, which is explicit that the rule stands and
the compiler must catch up, and lists the pieces.

**Why it was needed.** The rule said a `fn(A) -> R` parameter "is a generic
over `Callable` (static dispatch, monomorphised)" — a true statement of what it
*is*, with nothing about how it is realised. A C function pointer satisfies
"static dispatch" and is far easier, so that is what was built, and it rejects
every lambda that captures anything.

The amendment adds what the shortcut leaves out: the parameter is an implicit
generic bounded by `Callable[(A), R]`; a named function has a zero-sized type
so its call is direct; a lambda has an anonymous type whose fields are its
captures; and — stated as a prohibition, because that is the part an
implementer will otherwise get wrong — a conforming implementation **must not**
use a function pointer, which is what `extern "C" fn` is for.

### A6 — `[FN-1]`: a `mut` parameter whose type is itself a borrow

**Not yet ruled on by the owner.** ERR-041 remains open; this records the
reading the document's own example forces, and it is the one amendment here
that the owner may wish to reverse.

**Why it was needed.** `[FN-1]` says a `mut` argument "MUST be a mutable
place". Part VII's own worked example passes `buf.as_mut_span()`, a call
result, which is not a place. Read literally the document's own example is
`E2140`, and `split_at` — which `[SPN-*]` names as the sanctioned way to obtain
two mutable borrows into one container — could not be called on its own result
either.

The reading: where the parameter's type is itself a borrow, the argument *is*
that borrow and travels by value, and the place requirement lands on whatever
the borrow was taken of. A `MutSpan[T]` already carries the exclusivity that
`ref mut` would add, and `[SPN-3]` makes it move-only precisely so there is one
of it.

**If the owner rules the other way** — that `[FN-1]` is literal — then Part
VII's three example lines change, `split_at` grows a binding before every use,
and the compiler change is one line in `mut_param_ty`.

### A7 — `[FFI-17d]`: `@ffi(no_virtual_dtor)` is defined

**Authorised by** fix-list item 17, which asks for the cross-reference to point
at the rule that defines the attribute.

**Why the fix is a definition rather than a re-pointing.** `[FFI-17d]` cited
`[FFI-17b]`, which is about templates being importable only as explicit
instantiations and says nothing about destructors. Following the citation to
the right rule was impossible: **no rule defines the attribute at all**. So the
amendment defines it where the concept lives, and says what it does not do —
it licenses no operation and grants no tier, recording an obligation the way
`[UNS-7]`'s `@safety` does.

Item 17 also asks for a rule-index check that every rule reference resolves.
That is not built yet and is tracked as follow-up work.

### A8, A9, A10 — the two missing productions

**Authorised by** fix-list items 7 and 8.

`pub extern "C" fn on_update(...)` appears in XVI.10 as the shape of the
embedding story, and `extern class` carries the whole of `[FFI-39]`'s bounded
foreign-inheritance model. Neither had a production, so neither could be
parsed by an implementation following Part III.

A8 adds `["extern" string_lit]` to `fn_header`, with a note distinguishing it
from `extern_block`: this **defines** a function with a foreign ABI, where the
block **declares** foreign functions. It carries the constraints that follow
from rules already written — FFI-safe types under `[FFI-5]`, `E5054` for a
range type under `[RNG-10b]`, `[FFI-20]`'s panic boundary.

A9 adds `extern_class`, with the distinction `[FFI-39]` already draws: an
`extern class` is declared, sized and implicitly `open`, where `extern type` is
opaque, unsized and not inheritable. Per item 8 it does **not** make arbitrary
foreign classes inheritable — inheriting one requires
`@ffi(trampoline, virtuals=[...])`, and multiple inheritance, virtual bases and
unnamed virtuals stay unsupported.

A10 admits A9 in `item_body`.

---

## Not amended, and why

**`[RNG-8]` — its opening is missing and only the owner can supply it.** The
rule begins mid-sentence on an ellipsis:

> `[RNG-8]` "…and crosses an FFI boundary as its representation (`[FFI-5]`).…"

The missing first half survives nowhere in the document. It had been filled in
with invented wording — *"A range type erases to its representation at every
coercion site (`[TYP-5]`) and"* — which was reverted. Per fix-list item 9 a
reconstruction must not be presented as recovered text, so the rule stands as
the owner wrote it and this is flagged for an owner decision.

The surrounding rules constrain what it must say: `[RNG-10b]` already forbids a
range type in a foreign signature and requires foreign values to enter at the
representation type, so the missing clause is most likely about erasure at
coercion sites — but "most likely" is not a specification.

**The diagnostic-code collisions.** `[GRM-23]` names `E0104` where Part III's
precedence table names `E0102`, and `[MAN-3]` names `E9010`, which `[TYP-9c]`
also owns. The owner ruled on both (items 4 and 5): the normative code wins and
the compiler changes. No amendment is needed for the compiler to be correct, so
none was made — the tension with `[DIA-6a]`'s one-code-per-rule is recorded at
the registry entries in `compiler/ember_diag/src/codes.rs` and in
`docs/spec-errata.md` under ERR-026 and ERR-039.

**Everything in `docs/spec-errata.md` not listed above.** Duplicated sentences,
stale counts superseded by a later rule, editorial instructions pasted into the
source, leftovers from the layer 0.6.2 removed. Each is recorded there with the
reading taken; none is edited into the document. Fix-list items 25 and 26 ask
for several of these to be repaired properly, which is follow-up work and will
be appended here when done.
