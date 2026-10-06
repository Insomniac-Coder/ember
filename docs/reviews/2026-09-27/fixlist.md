# Review fixes (2026-09-27)

Confirmed or partly confirmed findings, most severe first. Status: open / fixed / owner.

| id | severity | verdict | category | status | finding |
|---|---|---|---|---|---|
| A1 | critical | confirmed | soundness | open | Span[u8].to_str() validates once and then hands out a zero-copy str over bytes that other class handles may overwrite without any check under [EXC-17], so Safe code can h |
| B1 | critical | confirmed | correctness | open | Array.pop() on an empty array returns zero bytes as its None, which for the new niches (bool, char, str, Span, most unit enums and ranges) is a Some value, not None. |
| D1 | critical | confirmed | correctness | open | An assignment through an array element of a foreign record static (`s.arr[i] = v`, `s.arr[i].x = v`) writes a temporary copy and leaves the C global unchanged. On an `@ff |
| F1 | critical | confirmed | soundness | open | When a field loan's (or EXC-18 call result's) last use is a call terminator, the access's EndAccess is emitted before that call, so the field is unprotected for the whole |
| F2 | critical | confirmed | soundness | open | A result that holds more than one view (a tuple or struct of views) never carries its EXC-18 field accesses to the caller, because `regions.local_region(dest)` is None fo |
| F3 | critical | confirmed | soundness | open | D-218 made class receivers of view-returning methods pass by address (`self: ref C`, C type `struct C**`), but the dyn/interface adapter still passes the bare object poin |
| F4 | critical | confirmed | soundness | open | A view returned through a `Box[dyn I]` or `ref dyn I` call whose concrete type is a class carries no access to the caller: `call_result_accesses` only handles Class and C |
| F5 | critical | confirmed | soundness | open | ODR-072 is enforced only for the literal statement `self = ...`. Passing `self` to a `mut` handle parameter (or `mem.swap`/`mem.replace`) still re-points the caller's han |
| A2 | major | confirmed | correctness | open | When a mut self method calls a plain self method on self, the callee's accesses to a non-Copy field panic against the whole-object write its caller holds. This was alread |
| B2 | major | confirmed | spec-conformance | open | Option[bool] is stored as uint8_t, but a by-reference payload binding takes its address as a bool*, so the emitted C mixes incompatible pointer types and accesses a uint8 |
| B3 | major | confirmed | correctness | open | For an integer range whose representation is i128/u128, the niche test and the None value are emitted as C `==` and a C cast on ember_i128/ember_u128, which are structs o |
| B4 | major | confirmed | soundness | open | THR-1's 'inside init, a field assignment after self has been used as a whole is E7003' misses uses in a while condition and uses that come later in a loop body, so a @syn |
| C2 | major | confirmed | correctness | open | A non-nullable `count(n)` input passes the span's data pointer unchanged. An empty Array's span has a NULL pointer, so C gets NULL for a pointer the contract says is neve |
| C3 | major | partly | spec-process | open | c718267 shipped two surfaces Hardened_31 did not define: the safe `ref T as *T` cast and a `p.is_null()` method. The next commit's ODR-075 then counted the shipped cast a |
| C6 | major | partly | correctness | open | Whether a foreign declaration has a wrapper is decided in three places that disagree. The wrapper (`ffi_counted`) is also built for fixed arrays and array results, but th |
| D2 | major | confirmed | correctness | open | Using a hand-declared foreign function with a borrowed (by-address) record parameter as a function value crashes the compiler at HEAD. Between b41d35d and 9821bbe it inst |
| D3 | major | confirmed | correctness | open | D-367 is recorded as fixed, but class type-info globals still have external linkage and no package namespace in static-library mode. Two Ember archives that each define a |
| E1 | major | confirmed | spec-conformance | open | Under @overflow(wrap), the written operator `MIN // -1` still panics at run time, while ODR-084/H40 TYP-28 say it yields MIN; constant evaluation returns MIN, so compile  |
| E2 | major | confirmed | correctness | open | The inlined `sum(...)` builtin builds its accumulation `acc + element` as a plain HIR Binary in the caller with no OverflowScope, so an integer sum wraps or saturates sil |
| E3 | major | confirmed | spec-process | open | ODR-084 declares that integer methods keep fixed contracts but leaves TYP-21's 'Such a method is the operator: `3.add(4)` is `3 + 4`' unchanged. The implementation theref |
| F10 | major | confirmed | spec-process | open | ODR-071 adds "a tuple type has two or more elements", which overrides clear spec text: the grammar and [GRM-38] define one-element tuples. |
| F6 | major | confirmed | correctness | open | A statement or call that reads and writes the same class field begins a write and a read on one word at the same point, so ordinary code panics. |
| F7 | major | confirmed | spec-conformance | open | A two-phase `ref mut` of a field begins its write at the Ref statement, before the other arguments are evaluated, so `b.items.push(len(b.items))` panics. The spec gives a |
| F8 | major | confirmed | correctness | open | EXC-18's caller-side accesses ignore [EXC-15] coverage and ordering. A view-returning getter called on `self` inside a `mut self` method panics, and a `mut self` method t |
| F9 | major | confirmed | correctness | open | For a @sync class, every virtual or interface call returning a view panics, because the caller now emits `ember_object_begin_read`, which rejects Sync objects. |
| A3 | minor | confirmed | docs | open | The HANDOFF next-task line no longer lists lifting the 'mutable class-field access requires a mut self class method in this phase' restriction, but D-362 lifted it only f |
| A4 | minor | confirmed | test-quality | open | The new D-362 tests cover a whole-field assignment and push only. No test covers an element store, a compound assignment on a non-Copy field, a by-address (view-returning |
| A5 | minor | partly | design | open | The D-360 clone fix is structural only: tuples, Option and Result use pointer helpers, but derived clones of structs and user enums still return by value at every level.  |
| A6 | minor | confirmed | docs | open | The shape of Utf8Error (one `Invalid` variant, with no position of the first bad byte) is an implementation decision the spec leaves open, and no ADR records it. The test |
| B10 | minor | partly | spec-conformance | open | E7001 names the field's whole type rather than 'the component that is not' Send/Sync, as THR-1 requires. Separately, the E7003 help recommends Atomic[int]/Mutex, which do |
| B5 | minor | partly | spec-conformance | open | Any match expression (and any lambda) on the right-hand side in a @sync init counts as using self as a whole, so a field assignment that never touches self is rejected wi |
| B7 | minor | confirmed | correctness | open | The debug access stack matches an end only by (word, writing), so when two reads of the same field end out of order, the survivor's record carries the wrong location and  |
| B8 | minor | confirmed | correctness | open | The manifest check rejects `exclusivity`/`overflow`/`bounds_checks` as keys in every section, and it validates profile keys only in the `[profiles.X]` header form. |
| B9 | minor | confirmed | test-quality | open | Several new tests check less than their names or the HANDOFF claim, and several new niche paths have no test at all. |
| C1 | minor | partly | soundness | open | The rule that a `safe fn` may not expose a raw pointer or C function pointer is checked only at the top level (and through an Option niche). The same pointer inside a by- |
| C11 | minor | confirmed | spec-conformance | open | These commits add more uses of E5050 for "no foreign representation" and "cannot be generic". The spec defines E5050 as "a foreign fact claims a grade whose evidence is a |
| C4 | minor | partly | spec-conformance | open | The mapped type for a `borrowed, one, exclusive` parameter, `ref mut T`, is rejected with E5002. Only the `mut` mode form is accepted, while `Option[ref mut T]`, `ref mut |
| C5 | minor | confirmed | test-quality | open | No test checks that a `from(p)` result is actually tied to `p` at the call site. The only result-lifetime tests are declaration-shape rejects and accept cases, and they w |
| C7 | minor | confirmed | spec-conformance | open | The spec's E5040 (capturing closure where a C function pointer is expected) is never emitted. The test instead expects a generic E2020 mismatch that shows the internal ty |
| C8 | minor | confirmed | spec-conformance | open | The count witness type is matched by spelling, so a type alias of an integer is rejected with E5002 as a "non-integer witness". ODR-074 reserves E5002 for a missing or no |
| C9 | minor | confirmed | correctness | open | Generated C prototypes give shared pointers different constness: a shared `ref T` becomes `T*` in a direct extern prototype, but a shared Span or fixed-array pointer beco |
| D4 | minor | partly | spec-conformance | open | `cstr` is classified as neither Send nor Sync, with no spec basis and no ODR or ADR, and a conformance test locks this in. So `static GREETING: cstr = c"hello"` is reject |
| D5 | minor | confirmed | spec-process | open | The owner's rules files (docs/AUTOPILOT.md, docs/AGENT-WORKFLOW.md) were rewritten inside unrelated feature commits, each change credited as an 'owner update, 2026-09-27' |
| D6 | minor | confirmed | design | open | The distributable static library keeps shipping's MSVC `/GL`, so the shipped archive holds link-time-code-generation objects rather than ordinary object files. |
| D7 | minor | confirmed | correctness | open | The export-symbol collision check compares only Ember function symbols, and only when a program entry exists. Clashes with generated non-function names or runtime symbols |
| D8 | minor | confirmed | test-quality | open | Two things CString safety depends on have no test: that a user cannot build a `CString` without the terminator, and that the owner cannot be moved or dropped while a `.as |
| E4 | minor | partly | spec-process | open | The commit rewrote the owner's rules file (and AGENT-WORKFLOW.md) to widen a 'delegation by default' rule and add a model-routing policy (Astra/Sol/Luna). The working tre |
| E5 | minor | confirmed | spec-conformance | open | D-371's fix treats a minus sign in front of a suffixed signed literal (`-128i8`) as one negative literal, which goes beyond LEX-24 (limited to untyped literals), but trea |
| E6 | minor | partly | docs | open | SIMD-8 and the Part X check table still describe the exemption as 'the function is @overflow(wrap)'. They do not cover module policy or saturation, which H40 now defines. |
| F12 | minor | partly | spec-conformance | open | The [EXC-19] table row "calling an owned callable value stored in a field — write — the call" is not implemented: no access is taken for a callable held in a class field. |
| F13 | minor | confirmed | spec-conformance | open | The dyn adapter's whole-object write passes `(ember_loc){ NULL, 0, 0 }`, so an exclusivity panic from an interface or dyn `mut self` call names no file, line or column, c |
| F14 | minor | confirmed | correctness | open | The access-word member `_access_<field>` can collide with a user field of that name, producing duplicate C struct members. |
| F15 | minor | confirmed | test-quality | open | The five EXC-8 tests were rewritten from `holder.child.bump()` to `counter.bump()` rather than adding the new shape, so no test covers a mutating call on a field-held rec |
| F16 | minor | confirmed | docs | open | The new helpers were inserted under the existing doc comment of `drop_glue_symbol`, so `c_size`'s doc now starts with "D-182 — the `index`th out-of-line drop-glue functio |
| F17 | minor | confirmed | design | open | `less_behind_pointers` still treats every `TyKind::Vec` as text (byte comparison with `len` elements taken as bytes), contrary to ADR-061's "text is keyed on the flag". I |
