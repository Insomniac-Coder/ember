# Ember — defect ledger

## Which ledger a finding belongs in

Every finding sorts into one of four, and the sort decides who moves. Getting
this wrong in either direction is how an implementation workaround quietly
becomes the language's semantics.

| Kind | Evidence | Where it goes | Who moves |
|---|---|---|---|
| **Implementation defect** | the rule is clear, the compiler violates it | `DEFECTS.md` (here) | the compiler |
| **Missing coverage** | the compiler may be right and nobody had proved it | a conformance case; no ledger entry unless it fails | nobody |
| **Normative contradiction** | two rules require different things, neither marked as governing | `spec-errata.md` | **neither, until the owner rules** |
| **Deliberate divergence** | the rule is clear, the compiler knowingly differs, with a reason | `DEVIATIONS.md` | the compiler, later |

The third is the one that needs discipline, because a compiler cannot be called
wrong against a document that says both things — so the temptation is to pick
the reading that matches what is already built and move on. ERR-044 is the
worked example: `[TYP-15]`'s enumeration forbids a static-region `str` in a
class field, `[LT-3]` permits it in as many words, and `[TYP-15]`'s own
principle sides with `[LT-3]`. The compiler follows the enumeration and stays
there, unchanged, until the owner decides.


Every defect found and what closed it. One row per defect, newest first.

**Why this exists.** Defects were recorded in prose, spread across
`docs/HANDOFF.md`'s per-block sections, where "found" and "fixed" read the
same. A reader could not tell which of them are closed. This file answers that
one question, and links to where the reasoning lives.

**How to use it.** Fixing a defect updates four documents, and the row records
which:

1. **this ledger** — the row, with how the fix was *verified*: a program that
   failed before and passes now, or a test that goes red when the fix is
   removed;
2. **`docs/HANDOFF.md`** — the reasoning, in the section for the block it
   belongs to;
3. **`docs/DECISIONS.md`** — an ADR, where the fix took a decision the
   specification does not force;
4. **the specification** — `docs/spec-source/ember-spec.md`, regenerated into
   `docs/spec/`, where the document's own wording admitted the wrong reading.
   That goes through `docs/spec-errata.md` and its procedure, never by hand.

The fourth is the one to be careful with: **an implementation gap is not a spec
defect.** Where the document was right and the compiler was wrong, say so in the
row and leave the specification alone — ERR-014, ERR-019 and ERR-022 record what
the opposite mistake costs.

Status is one of **fixed**, **open**, or **won't fix** with the reason.

---

## 2026-09-22 — class destructors could publish their own handle

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-170 | **The compiler defined `E3016` but never ran a class-destructor escape check.** Because class handles are `Copy`, `items.push(self)` inside `drop(mut self)` bypassed the move checker and was accepted, leaving the runtime as the only resurrection defense. | `[CLS-7a]`, `[OBJ-5]` | **fixed** | The analysis now recognizes nominal `drop` methods, tracks copies of their receiver handle through locals and control flow, and rejects a copy written into another object’s field or passed to a compiler-known storage operation. The latter covers `Array.push`, `Cell` replacement, arena storage, `MaybeUninit.write`, `Box`, `Shared`, and `mem.forget`; opaque dispatch remains the runtime case required by `[OBJ-5]`. `class_drop_self_escape.em` and `class_drop_self_escape_field.em` were accepted before the correction and now report `E3016` for publication into an `Array[Token]` and another class object’s field, in debug, release, and shipping. The adopted rules already draw this boundary, so no specification change, ADR, ODR, owner decision, or version change was needed. |

---

## 2026-09-22 — `let` class fields lost their immutability after parsing

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-169 | **The parser recorded a class field’s `let` modifier, but type collection discarded it before the class metadata was built.** Consequently the assignment checker could not distinguish an immutable binding from an ordinary field, and `mut self` methods silently reassigned `let` fields after initialization. | `[CLS-9]`, `[CLS-9a]`, `[TYP-16]` | **fixed** | Field metadata now carries the `let` fact from source collection through generic-class substitution. Direct assignment and augmented assignment to that binding outside the declaring constructor emit `E1010`; constructor initialization remains valid, and a mutable borrow or method call through the contained non-`Copy` value remains valid. `class_let_field_reassign.em` and `generic_class_let_field_reassign.em` were accepted before the correction and now reject the ordinary and materialized-generic forms in debug, release, and shipping. `class_let_field_mutate_through.em` keeps the legal constructor initialization and mutate-through behavior runnable in every profile. The existing rules define this boundary completely, so no specification change, ADR, ODR, owner decision, or version change was needed. |

---

## 2026-09-22 — same-method class-field reborrows duplicated a runtime access

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-168 | **A `mut self` method that invoked a mutating method through one of its class-valued fields opened the containing object's write access both at method entry and again around the field call.** The runtime correctly rejected those overlapping intervals, even though the field call is a reborrow through the same receiver and must be permitted. | `[CLS-9a]`, `[EXC-1]`, `[EXC-4]`, `[EXC-5]` | **fixed** | Mutable-argument lowering now recognizes when its derived caller-side access place is exactly the method's already-active class access and leaves that outer interval in place. It still opens a distinct access for another class object and the callee still opens its receiver's own interval. `class_let_field_mutate_through.em` was red before the correction with an exclusivity violation and now prints `42` in debug, release, and shipping; generated C contains exactly two matching begin/end accesses: one for `Holder.bump_counter` and one for `Counter.bump`. The frozen rules explicitly define same-method reborrows and mutation through a `let` class field, so no specification change, ADR, ODR, owner decision, or version change was needed. |

---

## 2026-09-21 — generic interfaces could not be specialized for `dyn`

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-167 | **Interface declarations accepted generic parameters syntactically, but the checker neither bound those parameters while reading member signatures nor materialized a specialized interface contract.** `interface Inspect[T]: fn inspect(self, value: T)` therefore reported `T` as unresolved, and a valid `dyn Inspect[i32]` bound was rejected. | `[TYP-16]`, `[TYP-22]`, `[IFC-1]` | **fixed** | Interface collection now retains and scopes owner generic parameters. A concrete use materializes a specialized declaration identity with substituted signatures and specialized generic supertraits, and generic nominal instantiation binds its owner arguments while registering `implements Inspect[T]`. `generic_enum_generic_interface_imported.em` was red before the correction and now forms `Box[dyn Child[i32]]` from an imported `Signal[i32]`, dispatches its `i32` parameter through inherited and child slots in debug, release, and shipping, and pins the concrete C adapter table. The specification already defines generic interfaces and only forbids generic methods in `dyn`, so no specification, ADR, ODR, owner decision, or version changed. |

---

## 2026-09-21 — unknown manifest lint keys were silently accepted

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-166 | **The driver's nearest-manifest reader enabled `L3014` but ignored every other `[lints]` key.** A program with `not_a_lint = "warn"` therefore compiled successfully instead of receiving the required manifest error. | `[MAN-3]` | **fixed** | The driver now validates each `[lints]` key against the registered `L`-code namespace plus the manifest's documented descriptive names (`unused`, `potential_cycle`, and `large_copy`), and reports `E9010` on the manifest entry itself. The focused milestone was intentionally red before the correction (the unknown-key package compiled); it now rejects that package with `E9010` and separately accepts all documented key forms. The rule is explicit, so no specification, ADR, ODR, owner decision, or version changed. |

---

## 2026-09-21 — imported generic-enum variants did not resolve in patterns

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-165 | **A materialized generic enum retained its source origin under the fully qualified declaring-module name, but pattern resolution compared an imported qualifier such as `Entry.Value` only to the raw spelling.** Construction and `clone()` of `Entry[i32]` worked across the import, then the valid qualified variant pattern was rejected with `E1010`. | `[MOD-2]`, `[MOD-3]`, `[TYP-16]`, `[ENM-1]`, `[OWN-8]` | **fixed** | Variant-pattern lookup now resolves the written enum qualifier through the current module's import bindings before comparing it with the concrete enum name or generic origin. `generic_enum_derived_clone_imported.em` was rejected before this change and now constructs, clones, and matches the imported `Entry[i32]` in debug, release, and shipping, while pinning the generated clone body. The specification already defines import bindings and generic materialization, so no specification, ADR, owner decision, or version changed. |

---

## 2026-09-20 — weak class handles and match-arm ownership

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-164 | **A value bound by a `match` pattern was assigned to a MIR local but never registered for arm-exit destruction.** A `Some(value)` binding of a `Copy` class handle retained the handle, then no matching release ran; the object stayed alive after every visible owner had ended. | `[GRM-13]`, `[OWN-2]`, `[OBJ-3]` | **fixed** | Pattern bindings are now owned arm locals, and both ordinary and direct variant-dispatch match lowering emit their drops when the arm completes. `weak_class_handle.em` initially printed no final `Token.drop` after a live `Some(value)` upgrade; it now prints the exact two destructor events in debug, release, and shipping. The source already fixes by-value `Copy` binding and class-release semantics, so no specification, ADR, owner decision, or version changed. |
| D-163 | **`Weak[C]` was specified and supported by the runtime ABI but was absent from the type checker, MIR, and C backend.** Every `Weak(token)` or `Weak[C]` use was rejected as an unknown type/constructor, so `[WK-2]` and `[WK-3]` could not be exercised at all. | `[OBJ-3]`, `[WK-1]`, `[WK-2]`, `[WK-3]` | **fixed** | `Weak[C]` is now a compiler-known Copy wrapper around a class handle with weak-count retain/release glue; `Weak(h)`, `Weak[C].empty()`, and `upgrade() -> Option[C]` lower to the existing runtime weak-reference ABI. `weak_class_handle.em` was red with `E1010` before the change and now proves live upgrade, expired upgrade, empty upgrade, weak-copy accounting, and destructor timing in every profile; `weak_nonclass.em` rejects `Weak(1)` with `E2020`. The normative rules supply all behavior, so no ODR, specification edit, ADR, owner decision, or version changed. |

---

## 2026-09-20 — generic-class interface extensions were not materialized

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-162 | **A generic interface extension such as `extend[T] Holder[T] implements Measure:` was sent through the ordinary extension pass after `T` had left scope.** It therefore reported `E1010` for its target and never registered either the extension methods or `Holder[i32]: Measure`; a conforming `ref Holder[i32]` consequently could not form `ref dyn Measure`. | `[TYP-16]`, `[TYP-17]`, `[IFC-1]`, `[TYP-22]` | **fixed** | Generic class extensions now retain their implemented interfaces, declaring module, source span, and resolved bounds as a recipe. A concrete class materialization structurally matches the target, admits the recipe only when its bounds hold, registers its members under the interface identity, and records/checks conformance before dynamic adapter formation. `generic_class_interface_extension.em` was rejected before the correction and now dispatches through `ref dyn Measure` in debug, release, and shipping; its imported-module companion proves resolution remains at the extension site. `generic_class_bounded_interface_extension.em` proves the bounded form, while `generic_class_bounded_interface_extension_unsatisfied.em` proves that an unsatisfied bound leaves the class constructible but without the conditional dynamic-interface conversion. The specification already determines the behavior, so no specification, ADR, owner decision, or version changed. |

---

## 2026-09-20 — generic-class inherent extensions were not materialized

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-161 | **A generic inherent extension such as `extend[T] Holder[T]:` was neither collected with `T` in scope nor materialized for `Holder[i32]`.** The ordinary extension pass resolved its target after generic declarations had been cleared, producing `E1010` for `T`; even if that resolution had been bypassed, no extension method recipe was registered on the concrete generic class. The same omission initially let an `override` in this path bypass `[CLS-4]` validation. | `[TYP-16]`, `[IFC-1]`, `[CLS-4]`, `[DSP-2]` | **fixed** | Inherent generic-class extensions are retained as recipes, structurally matched against each concrete class argument list, and materialized with the extension's own substitutions when the class is instantiated. Method bodies and method generics use those bindings, and virtual layout plus `E2110` validation run at the same materialization boundary. `generic_class_inherent_extension*.em` covers direct, reordered, nested, generic-method, and virtual-override cases in every profile; `generic_class_inherent_extension_override_nonvirtual.em` proves that two concrete uses still report one `E2110`. Generic interface-implementation extensions are not claimed by this correction; their conformance/materialization path remains separate. The specification already determines the inherent-extension behavior, so no specification, ADR, owner decision, or version changed. |

---

## 2026-09-20 — classes accepted `@derive(Clone)` without generating `Clone`

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-160 | **`@derive(Clone)` was accepted on a class declaration but registered no `clone` method or `std.core.Clone` implementation.** The grammar explicitly permits the attribute on classes, and `[OWN-8]` defines class cloning as a shallow handle copy; nevertheless both direct and materialized generic classes reported that they had no `clone` method. | `[OWN-8]`, `[TYP-16]`, `[DRV-1]` | **fixed** | Class derives now synthesize `clone(self) -> Self` as a handle return, which the ordinary class return lowering retains, and register `std.core.Clone`. Generic-class recipes retain the derive request and apply it at each concrete materialization. `tests/run-pass/derived_clone_class.em` was rejected before the correction and now proves the shallow shared-object result and emitted retain in debug, release, and shipping. `derived_clone_generic_class.em` proves `Counter[Marker]` satisfies `T: Clone` although `Marker` itself has no `Clone`, with the same all-profile and C-retain checks. The specification already fixes the semantics; no specification, ADR, owner decision, or version changed. |

---

## 2026-09-20 — generic-class substitutions did not rebuild nested classes

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-159 | **Substitution rebuilt instantiated generic structs but not generic classes.** A method on `Factory[T]` with a `Payload[T]` parameter or result retained the symbolic materialization `Payload_T` when `Factory[i32]` was checked, so a real `Payload[i32]` argument or return was treated as a different type. The same gap prevented a generic class from exercising an inherited generic interface default through that method. | `[TYP-16]`, `[IFC-1]` | **fixed** | `substitute_ty` now reconstructs an instantiated class from its recorded generic origin after substituting its arguments, exactly as it already reconstructs an instantiated struct. `tests/run-pass/generic_class_interface_default_generic_method.em` was rejected before the correction and now verifies direct and late calls to the specialized interface default, plus a nested generic-class parameter and result, in debug, release, and shipping. `generic_class_interface_default_dyn_box.em` covers the complementary non-generic default through borrowed and owned dynamic interface adapters. The specification already composes these constructs; no specification, ADR, owner decision, or version changed. |

---

## 2026-09-19 — owned interface receivers were accepted as dyn-compatible

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-158 | **`dyn I` formation accepted an interface method with `owned self`.** The later method-call path rejected it only through `ref dyn`, leaving the interface incorrectly dyn-compatible and inviting an impossible move from a borrowed fat pointer. `[CLO-6a]` explicitly excludes `CallableOnce` for this reason. | `[TYP-22]`, `[CLO-6a]` | **fixed** | The formation check now rejects any owned receiver with `E2050`, alongside receiver-less, generic, and by-value-`Self` members. `tests/conformance/TYP-22/reject_dyn_interface_with_owned_receiver.em` was accepted before the fix and now requires the formation error in debug, release, and shipping. This applies the existing v1 dyn boundary; no specification, ADR, owner decision, or version changed. |

## 2026-09-19 — dynamic vtable layouts lost inherited slot signatures

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-157 | **The C backend reconstructed a dynamic interface table from only the slots called in one module.** For `interface Child: Parent`, a program that called only `Child.child()` emitted the inherited `slot0` as `void (*)(void*)` instead of `Parent.parent()`'s `i32 (*)(void*)`. A later concrete table or cross-module caller would therefore disagree about the same `Child` vtable layout. | `[TYP-22]`, `[CG-C-1]` | **fixed** | Type checking now carries the complete declaration-order layout — including inherited ABI signatures — through HIR and MIR. C emission uses that canonical layout, and the MIR verifier rejects a call whose selected slot/signature disagrees with it before code generation. `tests/compile-pass/dyn_interface_inherited_vtable_layout.em` was red before the fix and pins both inherited and declared C slots; `ember_mir` has a direct malformed-layout verifier test. `Self: Sized` defaults retain only an explicitly non-callable placeholder: D-156 rejects their calls, while their eventual concrete-table materialization remains out of scope. No specification, ADR, owner decision, or version changed. |

## 2026-09-19 — sized-only defaults were callable through dyn

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-156 | **Dynamic interface calls ignored `where Self: Sized` on default methods.** `x.clone()` with `x: ref dyn Factory` was accepted even though `clone` returned `Self` and required a sized receiver. The unresolved result reached generated C as `void*`. The same omission admitted an inherited sized-only default returning `i32`, so checking only for a `Self` result would not fix the boundary. | `[TYP-22]` | **fixed** | Dynamic call checking reuses the declaration's existing sized-default metadata and rejects the call with `E2020` before producing HIR. Both direct and inherited minimal probes were accepted before the change; `tests/conformance/TYP-22/reject_dyn_sized_default_call.em` and `reject_dyn_inherited_sized_default_call.em` require rejection in debug, release, and shipping. The compile-pass counterpart retains interface formation and an ordinary callable member alongside the sized-only default. The specification already distinguishes an unsized `dyn` receiver from the sized-only default; no specification, ADR, owner decision, or version changed. |

## 2026-09-15 — callable mode mismatches used the generic type error

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-155 | **A callable parameter-mode mismatch was reported as generic `E2020` at the callable boundary.** This affected direct callable assignment, generic callable parameters, and the ordinary `std.borrow.with_views*_mut` path. A late-bound callback could also produce a duplicate mode diagnostic because lambda synthesis and generic-call validation both attempted to report the same source mismatch. The compiler therefore failed to expose the H3 `B15`/`E2228` diagnostic boundary required by `[FN-6a]` and `[LT-11a]`. | `[FN-6a]`, `[LT-11a]`, `[DIA-7]` | **fixed** | Callable checking now canonicalizes capture-free functions, lambdas, and capturing closure environments to their compile-time callable signatures for mode comparison, reports one `E2228` with the differing parameter position, expected/supplied modes, and full signatures, and leaves late-bound region checking independent of mode checking. `tests/ui/borrow/B15/callable_parameter_mode_mismatch.em` pins the exact diagnostic and its compiling repair; the FN-6 and LT-8 conformance cases cover direct assignment, generic callable parameters, and `with_views2_mut`. `cargo test -p ember_driver --test ui`, the full milestone/conformance suite, and `python tools/error_pages.py` pass. This is a compiler-only correction to the H3 target; no adopted specification, ADR, owner decision, or version changed. |

## 2026-09-15 — class virtual dispatch ignored inherent extensions

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-154 | **The virtual-dispatch metadata pass considered only methods written in the class body.** An inherent `extend Child:` block could declare a valid `override` or new `virtual` method, but the class vtable layout did not see it, so base-typed calls could select the wrong slot or bypass dynamic dispatch. The extension path also lacked the same non-virtual-override diagnostic as a class body, and the formatter dropped dispatch markers from member headers. | `[DSP-1]`, `[DSP-2]`, `[CLS-4]`, `[IFC-1]`, `[FMT-1]` | **fixed** | Virtual declarations from inherent class extensions now append in source/module order to the class layout; inherited slots are reused for extension overrides, invalid extension overrides receive `E2110`, and `ember fmt` preserves `virtual`/`override`. `tests/run-pass/class_virtual_dispatch_extend.em` and `tests/compile-fail/class_extend_override_nonvirtual.em` cover the positive, negative, and formatting boundaries. This is a compiler-only correction to the existing specification; no adopted specification, ADR, owner decision, or version changed. Interface extensions remain separate from class-vtable declarations. |

## 2026-09-15 — virtual methods were always lowered as direct calls

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-153 | **The compiler accepted `virtual`/`override` declarations but did not carry their dispatch contract into calls.** Method lookup selected a source definition and MIR always emitted a direct symbol call; generated class type-info also left `vtable` null. A base-typed handle therefore invoked the base implementation even when its object was a derived instance. | `[DSP-1]`, `[DSP-2]`, `[CLS-4]` | **fixed** | Type checking assigns deterministic base-first slots and records class-method ownership in HIR/MIR. MIR now lowers calls to virtual methods through a distinct virtual-call reference. The C backend emits inherited-prefix vtable layouts, override adapters for the nominal receiver type, and type-info vtable pointers. `tests/run-pass/class_virtual_dispatch.em` proves a `Base`-typed handle invokes `Child`'s override in debug, release, and shipping; the existing `class_virtual_override.em` and `class_override_nonvirtual.em` retain declaration validation coverage. This is a compiler-only correction to the existing specification; no adopted specification, ADR, owner decision, or version changed. Indexed class-object access, interface/dyn dispatch, static access elision, generic classes, and the complete Phase 3 matrix remain open. |

## 2026-09-15 — class-handle identity was parsed but not lowered

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-151 | **The compiler parsed class-handle identity operators but rejected them before HIR lowering.** The adopted contract treats `is` and `is not` as identity checks on class handles, distinct from value equality. The typechecker had no accepted path for either operator, so valid related-class comparisons could not reach code generation. | `[CLS-4]`, `[OBJ-2]` | **fixed** | HIR now preserves identity as a distinct operation and code generation uses the existing pointer representation. The typechecker accepts equal or related class handles, inserting the existing ordinary derived-to-base upcast where required, and rejects non-class or unrelated operands with `E2020`. `tests/run-pass/class_identity.em` covers same-object, distinct-object, derived/base, and negated identity in all profiles; `tests/compile-fail/class_identity_nonclass.em` covers the invalid non-class case. This is a compiler-only correction to the existing specification; no adopted specification, ADR, owner decision, or version changed. Virtual/override dispatch, downcast operators, indexed class-object access, static elision, generic classes, and the complete Phase 3 matrix remain open. |

## 2026-09-15 — class downcasts were parsed but not lowered

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-152 | **The parser recognized `as?` and `as!`, and the runtime already exposed the base-chain query, but the typechecker fell through to the unsupported-expression diagnostic.** The adopted contract requires `h as? D` to produce `Option[D]` and `h as! D` to panic on a failed related-class query. | `[DSP-4]`, `[OBJ-2]`, `[EFF-16]` | **fixed** | HIR now carries the target class and option shape to MIR. MIR performs one `ember_downcast` query, constructs `None` or an owning `Some` for `as?`, and asserts success for `as!`; successful results retain exactly once, while the non-owning query temporary is never dropped. The backend emits the target type-info lookup and the existing abort-only `invalid downcast` panic. `tests/run-pass/class_downcast.em` covers successful and failed optional downcasts plus a successful forced downcast in all profiles; `tests/compile-fail/class_downcast_unrelated.em` rejects unrelated classes; `tests/run-fail/class_downcast_forced.em` covers the forced failure. This is a compiler-only correction to the existing specification; no adopted specification, ADR, owner decision, or version changed. Virtual/override dispatch, indexed class-object access, static elision, generic classes, and the complete Phase 3 matrix remain open. |

## 2026-09-15 — floating comparison facts were discarded in `if` arms

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-150 | **The range checker refined comparison facts only for integral locals.** `[RNG-4]` covers numeric expressions, so a bounded floating value such as `0.0 < value < 1.0` should be usable to construct a compatible range inside the true arm. The checker instead left the float interval unknown and required an unnecessary checked or clamped conversion. | `[RNG-4]`, `[RNG-4a]`, `[RNG-10]` | **fixed** | Conditional refinement now accepts finite floating bounds and records conservative closed intervals for both strict and non-strict comparisons. Strict comparisons intentionally widen to the nearest representable closed bound because the interval lattice has no predecessor/successor operation; this can lose an optimisation opportunity but cannot prove an out-of-branch value. Numeric locals without a prior fact start from the finite representation range, while NaN and non-finite comparison bounds remain fail-closed. `accept_float_branch_refinement.em` proves the positive path and `reject_float_branch_fact_does_not_escape.em` proves the fact remains branch-local in all profiles. No specification, ADR, owner decision, or version changed. |

## 2026-09-15 — conservative bitwise range facts were missing

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-146 | **The range checker discarded every bitwise range fact, including exact constant expressions and masks whose result is necessarily non-negative and bounded.** `[RNG-4]` requires known ranges for numeric expressions it can track, so `4 | 8` could not construct a `0 ..= 15` value and `value & 15` could not construct a bounded mask result even when `value` was otherwise unconstrained. | `[RNG-4]`, `[RNG-4a]`, `[RNG-10]` | **fixed** | `range_of` now evaluates exact integer `&`, `|`, and `^` constants and transfers `&` through an exact non-negative mask as `0 ..= mask`. General OR/XOR, sign-bearing masks, unknown operands, and non-integral cases remain fail-closed. `accept_bitwise_range_refinement.em` covers both accepted forms in all profiles, while `reject_bitwise_unproven_range.em` proves that an unconstrained OR does not manufacture a range fact. No specification, ADR, owner decision, or version changed. |

## 2026-09-15 — unary numeric range facts were missing

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-147 | **The range checker discarded unary numeric facts.** `[RNG-4]` covers numeric expressions, but a negation of a bounded value and an exact integer bitwise complement could not construct a compatible range without a redundant checked conversion. | `[RNG-4]`, `[RNG-4a]`, `[RNG-10]` | **fixed** | `range_of` now transfers negation by reversing exactly-negated endpoints and transfers `~` only for an exact integer. Integer overflow and non-finite floating endpoints fail closed; boolean `not` and general complements remain unknown. `accept_unary_range_refinement.em` covers bounded negation and exact complement in all profiles. No specification, ADR, owner decision, or version changed. |

## 2026-09-15 — non-negative interval AND facts were missing

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-148 | **The range checker discarded a sound interval bound for bitwise AND.** Exact masks were handled by D-146, but `a & b` remained unknown even when both operands were known non-negative intervals, although the result is necessarily non-negative and no greater than either operand's upper bound. | `[RNG-4]`, `[RNG-4a]`, `[RNG-10]` | **fixed** | `bitwise_interval` now transfers two non-negative operand intervals as `0 ..= min(a.hi, b.hi)`. Signed intervals that may include negative values and general OR/XOR remain fail-closed. `accept_bitwise_range_refinement.em` covers the two-interval case in all profiles. No specification, ADR, owner decision, or version changed. |

## 2026-09-15 — non-negative interval OR/XOR facts were missing

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-149 | **The range checker discarded safe non-negative interval bounds for OR and XOR.** Exact constants were handled by D-146, but two known non-negative operands still could not construct a target range even though their result fits within the all-bits mask for the largest operand upper bound. | `[RNG-4]`, `[RNG-4a]`, `[RNG-10]` | **fixed** | `bitwise_interval` now derives `0 ..= (2^k - 1)` for non-negative `|`/`^` operands, where `k` is the bit width needed for the larger upper bound. Negative-capable intervals remain fail-closed. `accept_bitwise_range_refinement.em` covers both operators in all profiles. No specification, ADR, owner decision, or version changed. |

## 2026-09-15 — range facts were not refined in `if` arms

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-141 | **The range checker ignored comparison facts in conditional arms.** `[RNG-4]` explicitly includes the arms of an `if` that compared a value, but the compiler tracked only literals, bindings, and arithmetic. A proven `0 <= value <= 100` therefore still failed when constructing `Percent` from `value` inside the true arm; branch-local facts also were not isolated before the else arm. | `[RNG-4]`, `[RNG-4a]`, `[RNG-10]` | **fixed** | The typechecker now refines the existing interval lattice for integer comparisons, combines conjunctive conditions in the true arm, applies safe negated bounds in the false arm, and joins the two branch maps by interval hull while discarding facts absent from either path. D-150 extends the same conservative mechanism to finite floating bounds; disjunctions remain fail-closed. `tests/conformance/RNG-4/accept_branch_refinement.em` proves the integer path, while `reject_branch_fact_does_not_escape.em` proves that branch facts do not escape. No specification, ADR, or owner decision changed. |

## 2026-09-15 — canonical `std.math` range transfers were missing

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-142 | **The range checker ignored the specified interval behavior of the canonical `std.math` `min`/`max`/`clamp` helpers.** `[RNG-4]` requires these helpers to preserve useful range facts, but a `clamp_i32(value, 0, 100)` result was still unknown when `value` had no prior interval, and a `max_i32`/`min_i32` chain inside a proven branch could not construct `Percent`. | `[RNG-4]`, `[RNG-4a]`, `[RNG-5a]`, `[RNG-10]` | **fixed** | `range_of` now transfers facts through resolved direct calls to `std.math.min_i32`, `max_i32`, `clamp_i32`, and their `f32` counterparts. `min`/`max` combine operand interval endpoints; `clamp` proves the supplied bound interval first and otherwise composes the conservative interval operations. Unknown or mixed-representation inputs fail closed. `tests/conformance/RNG-4/accept_math_range_refinement.em` covers a parameter-clamp and a branch-refined min/max chain in all profiles. No specification, ADR, or owner decision changed. |

## 2026-09-15 — division range facts were always discarded

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-143 | **The range checker treated every division as unknown even when the divisor interval was provably non-zero.** `[RNG-4]` requires arithmetic range tracking, so a quotient with a non-zero divisor could not construct a narrower range type without a redundant checked/clamped conversion. | `[RNG-4]`, `[RNG-4a]`, `[TYP-8]`, `[RNG-10]` | **fixed** | The interval transfer now evaluates the four quotient corners only when the divisor is wholly positive or wholly negative. Integer `checked_div` rejects `MIN / -1`, and floating results must remain finite; zero-capable or otherwise unsafe divisors fail closed and retain ordinary runtime checks. `tests/conformance/RNG-4/accept_division_range_refinement.em` covers integer and float quotients in all profiles, while `reject_division_unknown_divisor.em` proves an unknown divisor still produces `E2215` for a range construction. No specification, ADR, or owner decision changed. |

## 2026-09-15 — remainder range facts were always discarded

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-144 | **The range checker treated every remainder as unknown even when the divisor interval was provably non-zero.** `[RNG-4]` requires arithmetic range tracking, so a remainder whose divisor was known to be `10` could not construct a `-9 ..= 9` range without an unnecessary checked conversion. | `[RNG-4]`, `[RNG-4a]`, `[TYP-8]`, `[RNG-10]` | **fixed** | The interval transfer now derives a conservative integer remainder interval from the divisor's maximum magnitude, preserving the sign of the dividend and requiring the divisor interval to exclude zero. It intentionally does not assume corner monotonicity. Unknown or zero-capable divisors remain unknown. `accept_division_range_refinement.em` covers a non-zero remainder in all profiles, and `reject_division_unknown_divisor.em` proves the fail-closed boundary with `E2215`. No specification, ADR, or owner decision changed. |

## 2026-09-15 — shift range facts were always discarded

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-145 | **The range checker treated every shift as unknown even when the operand and shift amount were both bounded.** `[RNG-4]` and `[TYP-10]` permit a range fact when the amount is proven within the operand width and a left-shift mathematical result is representable, but `value << 1` and `value >> 1` could not construct narrower range types. | `[RNG-4]`, `[RNG-4a]`, `[TYP-10]`, `[RNG-10]` | **fixed** | The interval transfer now handles `<<` and `>>` when the amount interval is non-negative and strictly below the operand bit width. Left shifts use checked endpoint arithmetic and therefore remain unknown when mathematical overflow is possible; right shifts use sign-aware arithmetic-shift bounds. Unknown or out-of-width amounts fail closed and preserve the ordinary shift runtime check. `tests/conformance/RNG-4/accept_shift_range_refinement.em` covers positive left and signed right shifts in all profiles, while `reject_shift_unknown_amount.em` proves an unconstrained amount still produces `E2215`. No specification, ADR, or owner decision changed. |

---

## 2026-09-15 — inherited class defaults were fail-closed

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-140 | **A derived class constructor was rejected whenever its base or derived fields had defaults, even though `[CLS-2]` and `[CLS-4]` permit those defaults to be established before the corresponding constructor body runs.** The compiler could materialize defaults for non-inheriting classes, but `super.init(...)` had no flattened default metadata and the constructor gate rejected the whole inheritance chain. | `[CLS-1]`, `[CLS-2]`, `[CLS-4]`, `[OWN-5]` | **fixed** | The type checker now flattens source defaults in physical base-first field order, checks each expression against its declared type, and marks the derived constructor's own defaults live while `super.init(...)` marks the base portion initialized. MIR materializes the complete default vector immediately after allocation, before the derived body, and preserves ordinary overwrite/drop ordering for explicit assignments. `tests/run-pass/class_inherited_defaults.em` proves an inherited default is readable in `Base.init` and a derived default is readable after `super.init`, in debug, release, and shipping. No specification, ADR, or owner decision changed. |

## 2026-09-15 — class default expressions were limited to literals

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-139 | **Non-inheriting class construction only materialized literal field defaults even though `[CLS-2]` permits a field with a default expression to be omitted.** Memberwise construction and user-defined `init` construction both had the same literal-only side table, so `value: i32 = default_value()` either failed at the custom-init boundary or at the memberwise omitted-field boundary. | `[CLS-1]`, `[CLS-2]`, `[CLS-3]`, `[CLS-6]`, `[OWN-5]` | **fixed** | The checker now preserves source default expressions for class fields and checks them at the construction site against the declared field type, reusing ordinary expression typing, coercion, and lowering. MIR still stores defaults immediately after allocation and before a user `init`, and explicit constructor assignments to those fields remain ordinary overwrites. `tests/run-pass/class_construct_nonliteral_defaults.em` covers a function-call default read inside `init`, overwrite after that read, and an omitted memberwise field with the same non-literal default in debug, release, and shipping. Inherited/defaulted derived construction remains fail-closed because base-default materialization through `super.init` still needs layout-offset metadata. No specification, ADR, or owner decision changed. |

---

## 2026-09-14 — direct calls ignored named parameter binding

## 2026-09-14 — B8 UI snapshot lagged the class-field call span

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-137 | **The committed B8 UI snapshot retained the old narrow receiver caret after the class-field method-call checker began reporting the complete conflicting call span.** The diagnostic code, message, required help, and rule classification were unchanged; only the exact primary span in the snapshot lagged the already-intended class call boundary. | `[DIA-1]`, `[DIA-7a]`, `[BRW-4]` | **fixed** | The snapshot now matches the compiler's full-call primary span, and the UI test passes. This is a stale test fixture exposed by the full regression run, not a language or compiler semantic change. |

---

## 2026-09-14 — custom class init rejected literal field defaults

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-138 | **A user-defined class `init` rejected all defaulted fields even though `[CLS-2]` permits a field with a default to be omitted.** The compiler already had a narrow literal-default materialization path for memberwise classes, but the custom-constructor path rejected it before checking the constructor body. | `[CLS-1]`, `[CLS-2]`, `[CLS-3]`, `[OWN-5]` | **fixed** | Commit `01a1eab` carries literal defaults through HIR and MIR, stores them after allocation and before the custom `init` body, and marks those fields readable during definite-initialization checking. An explicit constructor assignment to a defaulted field remains an ordinary overwrite, including drop-before-store ordering for a future drop-capable literal. At that checkpoint, non-literal defaults and inherited/defaulted derived construction remained fail-closed; D-139 later closes the non-literal boundary for non-inheriting classes. `tests/run-pass/class_construct_init_default_literals.em` covers readable defaults, explicit overwrite, generated construction, and all profiles. No specification, ADR, or owner decision changed. |

---

## 2026-09-14 — indexed class-field mutable arguments were fail-closed

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-136 | **A mutable argument rooted in an indexed class handle was rejected even when the place was valid.** `increment(items[next_index(state)].value)` failed with `E1010` because the class-field access checker deliberately rejected an indexed class root; the compiler had not yet shared the bounds-checked place between the mutable reference and the dynamic exclusivity interval. | `[EXC-1]`, `[EXP-1]`, `[FN-2a]`, `[CLS-1]` | **fixed** | The type checker now admits indexed class-field mutable places. MIR lowers the place once, preserves its bounds check and source evaluation order, forms the mutable reference from that place, and derives the class access object from the same MIR projection prefix. Existing class-valued receiver behavior keeps its containing-object boundary, so `holder.child.bump()` does not open the child twice. `tests/run-pass/class_indexed_mut_argument_access.em` uses a side-effecting index to prove single evaluation, checks runtime access instrumentation, and passes in all profiles. No specification, ADR, or owner decision changed. |

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-134 | **Direct function and source-method calls zipped arguments by position even when `[TYP-25]` named arguments were present.** A call such as `encode(right=2, left=1)` therefore passed the values to the opposite parameters; generic functions and generic methods had the same gap. The compiler also did not consistently reject unknown, duplicate, or positional-after-named arguments at this boundary. Function-value calls were intentionally not included because their callable types have no source-level parameter names and remain positional. | `[TYP-25]`, `[EXP-1]`, `[TYP-16]`, `[TYP-24]` | **fixed** | Commit `a2f4454` adds one binding pass in the type checker, checks argument expressions in source order, emits HIR operands in declaration order, and carries explicit source-evaluation-order metadata into MIR. The path is shared by direct, qualified, associated, inherited-bound, generic, and generic-method calls; unknown names and positional-after-named use `E2020`, duplicate parameters use `E1030`. `tests/run-pass/named_function_arguments.em` covers ordinary and generic functions, `tests/run-pass/named_method_arguments.em` covers ordinary and generic methods, and `tests/compile-fail/named_function_argument_errors.em` covers the three rejection boundaries in debug, release, and shipping. The tests were red before the fix and green after it. No specification or owner decision changed; user-defined class-`init` named construction remains a separate fail-closed gap. |

| D-135 | **The user-defined class-constructor path rejected named arguments even though `[TYP-25]` requires source arguments to bind to the declared parameter names.** `Point(y=23, x=19)` was therefore rejected with `E1010`, and the constructor path did not carry `[EXP-1]` source evaluation order through its declaration-order ABI arguments. | `[CLS-1]`, `[CLS-2]`, `[CLS-3]`, `[TYP-25]`, `[EXP-1]` | **fixed** | Commit `9b21634` makes the class-constructor path reuse the direct-call binder for the `init` parameters, validate unknown/duplicate/positional-after-named arguments, check expressions in source order, store them in constructor-parameter order, and carry explicit evaluation-order metadata into MIR. `tests/run-pass/class_named_init_construct.em` covers out-of-order binding and side-effect order; `tests/compile-fail/class_named_init_argument_errors.em` covers all three rejection boundaries in debug, release, and shipping. No specification or owner decision changed. |

---

## 2026-09-14 — memberwise class construction rejected named fields

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-133 | **The memberwise class-constructor path rejected every named argument even though `[CLS-3]` makes a no-`init` class constructor memberwise, and `[STR-1]` defines memberwise positional/named binding.** `Point(y=2, x=1)` was therefore rejected with `E1010` despite containing valid field values. | `[CLS-2]`, `[CLS-3]`, `[STR-1]`, `[TYP-25]` | **fixed** | `2b70423` applies the established struct-style named-field binding to memberwise classes, including out-of-order fields, duplicate-field diagnostics, missing/defaulted fields, and existing positional behavior. `tests/run-pass/class_named_memberwise_construct.em` was red before the fix and is green in debug, release, and shipping. At the time of that fix, named binding for user-defined `init` remained a separate fail-closed gap; it is recorded as D-135. No specification, ADR, or owner decision changed. |

---

## 2026-09-14 — copied aggregates duplicated class handles without retaining

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-132 | **The C backend treated aggregate `Copy` operations as raw byte copies even when an aggregate contained class handles.** A minimal `Wrapped { thing: Thing }` copied into `Array[Wrapped]` first released the temporary class handle after aggregate construction, then copied the now-deinitialising pointer into the Array without retaining it. The later Array access therefore attempted to use invalid ownership, and destruction could release a handle with no strong references. The same missing retain boundary existed for `q = p` and other copied nested aggregates. | `[RC-1]`, `[RC-2]`, `[OWN-7]`, `[STR-3]`, `[ARN-8]` | **fixed** | `7f83b14` adds one recursive C-backend retain walk for direct class handles nested in structs, tuples, fixed arrays, and active enum payloads. It runs for `Copy` assignments, aggregate construction operands, and `Array.push` copies; moves do not retain. Compiler-known `MaybeUninit` storage is explicitly excluded because its field is not an initialized owner. `tests/run-pass/array_copy_struct_class_handle.em` was red before the fix (first missing the expected retain, then reproducing `retain of deinitialising or invalid object` after the partial boundary) and is green in debug, release, and shipping. No specification, ADR, or owner decision changed. |

---

## 2026-09-14 — inherited mutable calls through class fields lost the outer access

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-131 | **The compiler-generated derived-to-base borrow cast hid a class-field receiver's containing-object access.** A call such as `holder.child.bump(5)` where `bump` is inherited and declares `mut self` reached the base method's own dynamic access interval, but the caller no longer opened the long-term access on `holder` because the MIR classifier only recognized a direct `ref mut` receiver and did not look through the borrow-preserving cast. The resulting generated C had one access pair instead of the two required by the established class-field boundary. | `[CLS-4]`, `[CLS-7]`, `[EXC-1]` | **fixed** | `class_access_for_mut_argument` now unwraps only the compiler-generated cast around a mutable receiver borrow before finding the containing class place. The cast remains a non-owning pointer adjustment. `tests/run-pass/class_field_inherited_mut_method.em` asserts two caller/callee access pairs, inherited mutation, and output `7`; it was red before the fix and green after it in all profiles. No specification or owner decision changed. |

---

## 2026-09-14 — growable Arrays did not retain copied class handles

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-130 | **`Array[Class]` copied a class handle into its backing buffer without retaining it.** Class handles are language-level `Copy` values, so `items.push(child)` must give the Array its own strong reference. The byte copy in `ember_vec_push` left the buffer holding the caller's sole reference; the caller's later release then made the Array contain a dangling pointer and the eventual buffer release panicked with `release of object with no strong references`. The newly added indexed-method fixture exposed this through a real run, even though the method itself printed the expected value. | `[RC-1]`, `[RC-2]`, `[OWN-7]`, `[CLS-1]` | **fixed** | The `ArrayPush` C-lowering path emits one `retain` for a direct class element before the byte copy, while move-only element types retain the existing move path. `tests/run-pass/class_indexed_mut_method.em` now covers Array ownership, indexed class-method use, runtime output, and the generated retain. Verified red before the fix and green after it with the focused run-pass gate. No specification or owner decision changed. |

---

## 2026-09-14 — inherited-destructor run-pass fixture asserted the wrong result

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-129 | **The `class_inherited_drop.em` run-pass fixture expected the derived destructor output twice for one object.** Its broad `assert-c-order` needles also matched the standalone base adapter before the derived adapter, so the fixture rejected the correct derived-then-base lowering. The rule and compiler behavior were already correct: one release runs the derived source destructor once, then the base source destructor once. | `[CLS-6]`, `[TST-1]`, `[TST-4]` | **fixed** | `6415303` changes the expected output to `derived` then `base` and anchors the order assertion to the calls inside the derived adapter. **Verified:** `run_pass_programs_pass` and `cargo test --workspace --locked` pass; no compiler, specification, or owner decision changed. This was a test-fixture defect. |

---

## 2026-09-14 — source `@borrows` accepted empty or malformed arguments

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-128 | **The source checker did not validate the shape of every `@borrows` argument.** `@borrows()` produced no contract diagnostic and fell through to a downstream provenance error, while non-name forms such as `@borrows(0)` or a named argument were ignored instead of being rejected as invalid parameter references. The artifact boundary already rejected an empty position vector, but source programs needed the same fail-closed contract check. | `[LT-1a]`, `[DIA-7]`, `[BLD-2]` | **fixed** | `523c030` rejects empty, non-name, and named `@borrows` arguments with the existing `E2031` contract diagnostic, then uses ordinary elision rather than manufacturing an empty contract. Three LT-1a conformance cases cover the source boundary. This is a compiler/diagnostic defect against an existing specification invariant; no specification or owner ruling changed. |

---

## 2026-09-14 — EMIF accepted an empty explicit borrow contract

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-127 | **The module-interface validator accepted `borrows = Some([])` even though `[LT-1a]` requires `@borrows` to name one or more parameters.** A forged or stale artifact could therefore cross the decode boundary with an impossible explicit return-region contract. | `[LT-1a]`, `[BLD-2]`, `[LT-40]`, `[IMP-7]` | **fixed** | `c4fccdc` rejects an empty borrow-position vector during callable-contract validation and adds an artifact-builder regression. This is a compiler integrity defect against an existing specification invariant; no specification or owner ruling changed. |

---

## 2026-09-14 — EMIF decoding did not verify the cache-key inputs

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-126 | **A module-interface artifact verified its callable interface hash but accepted a serialized cache key that did not match its own source, compiler, language, package-config, and dependency inputs.** A standalone artifact consumer could therefore accept self-inconsistent cache identity metadata. The on-disk prepared-cache path happened to compare the key with a fresh artifact, but `ModuleInterfaceArtifact::from_bytes` is the public decode boundary and must fail closed on corrupted identity data. | `[BLD-2]`, `[LT-40]`, `[IMP-7]` | **fixed** | `5332572` recomputes the canonical cache key during artifact validation and rejects a mismatch with `StaleCacheKey` before the artifact can be consumed. `compiler/ember_build/src/interface.rs` adds a regression that mutates only the serialized cache key and proves decoding rejects it. This is a compiler integrity defect against the existing artifact contract; no specification or owner ruling changed. |

---

## 2026-09-14 — callback results did not infer independent generic types

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-124 | **Generic inference solved the hidden `Callable` binder but did not traverse its canonical callback signature.** In `fn apply[T, R](value: T, f: fn(T) -> R) -> R`, `T` could be inferred from `value`, but a named function or closure was recorded only as the opaque callable generic, leaving `R` unbound and producing E2060. This blocked the ordinary generic `std.borrow.with_views*` wrappers where the callback is the only source of `R`. | `[TYP-18]`, `[TYP-23]`, `[CLO-3]`, `[FN-6a]` | **fixed** | Generic argument inference now unifies a concrete function value's full mode-bearing signature with the implicit Callable/CallableOnce bound after it records the opaque binder. A concrete capturing closure contributes the generated call signature after its environment receiver, so its result is equally visible. Unresolved generic callback returns are inferred from the closure body rather than prematurely forced. **Verified:** TYP-18 covers named-function, capture-free-lambda, and capturing-lambda inference of an independent result type; LT-8 uses the same path for `with_views2/3/4` and their mutable forms. Mode mismatch still reports E2020 without a spurious E2060. This was a compiler defect against existing generic/callable semantics; no specification or owner ruling changed. |

---

## 2026-09-14 — latebound storage case did not reach an instantiated body

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-125 | **The first `@latebound` storage conformance case placed the violating `Box(f(view))` only in an uninstantiated implicit generic callable body.** The compiler intentionally emits MIR for a generic body only after a concrete call reaches it, so the case had no body-level region analysis and falsely passed. A rule-named directory and a syntactically violating function were not enough to prove coverage. | `[LT-7]`, `[LT-10]`, `[TST-20]`, `[TST-21]` | **fixed** | The conformance case now calls the wrapper from `main` with a concrete callback and a live view, forcing monomorphisation and the normal MIR/region pipeline. The same source then rejects the Box publication with `E3063`. This is a **test defect**, not a compiler defect or a specification issue; the target semantics remain unchanged. |

## 2026-09-14 — a borrowing closure crossed an owned callable boundary

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-123 | **A normal closure that captured a local by reference could be passed to `owned f: fn(...)`, even though that parameter transfers a callable the callee may retain.** The compiler rejected an actual `Box` store in the callee, but accepted the ownership boundary itself, leaving `[CLO-4]` dependent on whether a particular currently visible callee body happened to store the closure. | `[CLO-1]`, `[CLO-2]`, `[CLO-3]`, `[CLO-4]`, `[TYP-15]` | **fixed** | Borrow analysis now consumes two explicit MIR facts at a direct call: the callee's declared parameter modes and the nominal identity of compiler-generated reference-capturing closure environments. An `owned` parameter rejects only such a non-static closure environment with E3063; arbitrary owned values, borrowed/mutable callable parameters, capture-free closures, and `owned fn` environments remain outside this rule. The check uses inferred provenance rather than generated names or C representation, so a static-only environment remains admissible. **Verified:** CLO-4 rejects a reference-capturing closure passed through `owned f`, while existing CLO-4 borrowed invocation and CLO-6 `owned fn`/`CallableOnce` paths remain accepted. This is a compiler defect against existing semantics; no specification or owner decision changed. |

---

## 2026-09-14 — closure mutation shadowed its capture

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-122 | **A bare assignment in a lambda silently declared a new closure-local instead of mutating the matching outer capture.** `counter = counter + 1` inside `fn() -> i32: ...` therefore compiled but returned `1, 1`, even though `[CLO-2]` requires a mutable borrow of the captured outer place. The same missing capability meant that an `owned fn` could not mutate a directly owned capture unless it moved that capture out. | `[CLO-1]`, `[CLO-2]`, `[CLO-3]`, `[FN-1]`, `[BRW-1]`, `[OWN-3]` | **fixed** | Capture discovery now takes precedence over `[GRM-4]`'s fresh-local rule for a matching outer name. Type checking builds a private, non-emitted typed probe environment—`ref mut` fields for normal closures and direct fields for `owned fn`—then derives per-field mutable capture and the independent one-shot move fact. The final environment uses `ref`/`ref mut` exactly where the checked body requires it; its generated call takes `mut`, `owned`, or borrowed `env` according to those facts. Existing outer parameter modes carry that behavior through monomorphized calls: `mut f: fn(...)` accepts a mutable closure, while plain borrowed `f` is E3023. **Verified:** CLO-2 covers normal mutation (1 then 2), `mut f`, borrowed-`f` rejection, the live-loan conflict, and reusable owned mutation; CLO-6 covers mutation followed by a capture move as one-shot with one Array destruction. No new callable interface, language rule, ABI field, or parallel ownership model was introduced. |

---

## 2026-09-14 — owned closure body moves did not select `CallableOnce`

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-121 | **An `owned fn` captured its environment by move, but a body that moved a non-`Copy` capture out of that environment was still callable repeatedly.** The compiler had represented source capture mode, yet it did not carry the independently required `[CLO-2]` fact that consuming a capture makes that particular closure `CallableOnce`, not `Callable`. A plain `f: fn(...)` generic boundary could therefore accept a one-shot closure, and direct calls borrowed its environment rather than moving it. | `[CLO-2]`, `[CLO-3]`, `[CLO-6]`, `[OWN-3]` | **fixed** | Type checking now derives a concrete closure's capability from its checked HIR body: an owned environment is passed by `owned` mode only if a non-`Copy` captured field occurs in a consuming value context. The same precise fact rejects that closure at a plain `Callable` boundary with E3030 and permits it at `owned f`/`CallableOnce`; ordinary move analysis then makes a second direct call E3040. A read-only `owned fn` capture remains reusable, while D-122 separately covers body mutation and the combined mutate-then-move case. **Verified:** the CLO-6 matrix covers direct one-shot execution and one destruction, passing the same closure through `owned f`, E3030 at a plain `Callable` parameter, E3040 on a second direct call, an owned closure that mutates then moves its capture, and the existing LT-42 reusable owned-capture contrast. This is a compiler defect against existing rules; no specification or owner ruling changed. |

---

## 2026-09-14 — an indirect `CallableOnce` callee was not transferred

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-120 | **The move/drop transfer examined an indirect call's arguments but not its callee.** Type checking and MIR lowering correctly represented a `[CLO-6]` `owned f: fn(A) -> R` invocation as `Move(f)`, yet `drops::step_terminator` neither read nor moved that operand. The binding consequently remained live after its first `CallableOnce` call, allowing a second call when the concrete function representation was `Copy`. | `[CLO-6]`, `[OWN-3]` | **fixed** | The ordinary call-terminator transfer and drop-flag update now process an indirect callee exactly as they process a call argument: it is read for use-after-move reporting and an `Operand::Move` consumes its move path. This is an ownership-pipeline correction, not a callable-specific ownership model. **Verified:** `tests/conformance/CLO-6/accept_owned_callable_parameter_is_consumed_once.em` runs once, while `reject_owned_callable_parameter_cannot_be_called_twice.em` is E3040 at the second call. The existing ordinary `Callable` closure-twice case remains the contrasting non-consuming path. The specification already required this behavior and did not change. |

---

## 2026-09-14 — closure capture field provenance

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-117 | **A non-`owned` closure captured a multi-region aggregate as one whole `ref` even when its body accessed only one field.** The direct closure body already had an exact `[LT-35]` access summary, but closure construction lowered `fn() => pair.left[0]` as a synthetic `&pair`. That whole-place borrow carried both `pair.left` and `pair.right` provenance slots, so `right.push(9)` before `read_left()` incorrectly reported E3021. | `[LT-24]`, `[LT-35]`, `[LT-36]`, `[LT-42]`, `[MIR-REG-1]` | **fixed** | Current checkpoint: a closure body now carries an explicit compiler-internal environment identity. Only a synthetic borrow whose temporary flows directly and uniquely into that generated environment may consume the body’s installed, rederived direct-call summary. Region liveness, provenance facts, access attribution, and loan conflict checking then use exactly the selected paths; ordinary references, opaque function values, missing/unknown summaries, whole-aggregate capture, and multiple selected fields remain conservative. **Verified:** `accept_closure_capture_uses_only_selected_field.em` mutates `right` and prints 7 in debug, release, and shipping; whole-aggregate and two-selected-field captures still report E3021; an indirect function-value call retains all slots under `[LT-41]`. The `[LT-28]` split/overlap matrix separately confirms that independent regions are never an aliasing proof, and `[LT-25]` confirms that ordinary E3021 prevents a view from borrowing an owner moved into its own field. `[LT-31a]` additionally proves two source-distinct `Pair` values share one nominal callable ABI: generated C contains only the `em_left_value` declaration and definition and no region-slot ABI data. `cargo test --workspace` exits 0 (208 tests), the 129-directory/432-source conformance walk is green, and debug/release builds are warning-free. The frozen target was already explicit, so no specification, ADR, or owner ruling changed. |

## 2026-09-14 — enum discriminants must not read payload provenance

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-118 | **An enum discriminant read was treated as a read of every borrowed payload field.** In `match maybe: Some(Pair(selected, _))`, the tag check for `Option[Pair]` unnecessarily kept the discarded `right` span live, so mutating `right` before the match incorrectly reported E3021 even though the selected branch moves and reads only `left`. | `[LT-20]`, `[LT-24]`, `[LT-36]`, `[LT-38]`, `[MIR-REG-1]` | **fixed** | Current checkpoint: `Rvalue::Discriminant` is now control-flow metadata in region liveness, callable access summaries, and ordinary borrow-access collection; it neither reads nor borrows an enum payload. `Some(Pair(selected, _))` therefore permits the independent `right` mutation and prints 7 in every profile. The boundary remains strict: `Some(pair)` moves the whole `Pair`, so `[LT-36]` still requires every carried region and the corresponding case reports E3021. This is a compiler defect, not a relaxation of whole-value move semantics. The frozen target was already explicit, so no specification, ADR, or owner ruling changed. |

## 2026-09-14 — `owned fn` capture mode was discarded

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-119 | **The parser recorded `owned fn`, but type checking discarded that flag and lowered every capturing closure as a shared-reference environment.** An Array capture therefore remained a borrow instead of moving into the closure, contradicting the source capture mode and preventing an escaping owned closure from having its required ownership boundary. | `[CLO-1]`, `[CLO-2]`, `[CLO-4]`, `[LT-42]`, `[TYP-15]` | **fixed** | Current checkpoint: a generated closure environment now carries a verified compiler-internal move-capture marker. An `owned fn` stores each capture directly and ordinary move/drop analysis consumes the source; a normal closure remains a `ref` environment. Borrow analysis recognizes only verified owned environments and requires every captured view operand to carry exclusively static provenance, yielding E3063 otherwise. The marker is rejected if it lacks a generated environment, and neither it nor provenance slots enters the C ABI. **Verified:** owned Array capture runs twice and emits one `ember_vec_free`; post-capture source use is E3040; static view capture is accepted; local view capture is E3063. D-121 separately records the distinct body-level `CallableOnce` capability decision. Mutable captures, dynamic dispatch, and escaping/storage matrices remain explicitly incomplete rather than inferred. No specification, ADR, or owner ruling changed. |

---

## 2026-09-13 — callable-region metadata and field replacement

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-116 | **The region solver treated every historical assignment to one borrowed field as simultaneously current.** Its global `Flow { from, to }` graph was sound only while a region slot had one definition. After `[LT-21]` made `@view` fields replaceable, a later use of `pair.selected` propagated through both the replacement edge and the earlier construction edge. In the minimal reproducer, `pair.selected = replacement.as_span()` followed by `old.push(11)` still reported E3021 because the compiler claimed `old` was retained by the later use of `pair.selected`. The same conflation would publish both old and new sources in a returned-field summary | `[LT-20]`, `[LT-21]`, `[LT-22]`, `[LT-24]`, `[LT-35]`, `[VERIFY-3]` | **fixed** | `af7c525` replaces the location-insensitive edge closure with a forward point-sensitive value/provenance fixpoint: assignment replaces the destination fact, CFG joins conservatively merge possible facts, and NLL points, origins, callable accesses, and result summaries are all projected through the fact current at each MIR point. **Verified:** six LT-21 cases cover straight-line release, replacement retention, wrapper-result provenance in both directions, a conditional path that may retain the old source, and both branches replacing it. The returned wrapper's MIR records `result.0 <- arg2`, not historical `arg0`, while generated C remains an ordinary field store with no region metadata. Mutating fact replacement into union makes the original positive case fail with the pre-fix E3021. The full 122-directory/410-source conformance walk and all 189 workspace tests pass. The specification already required provenance recomputation and did not change; no ADR or owner ruling was needed |
| D-115 | **Calling a function through a function-value local did not count as reading that local.** The unused analysis inspected call arguments and the destination but ignored an indirect call's callee operand, so a value used exactly as `operation(21)` received false lint `L1001` | `[LNT-1]`, `[CLO-3]` | **fixed** | `90059c8` marks the `FuncRef::Indirect` operand as read through the same path as every other operand. **Verified:** `tests/conformance/LNT-1/accept_called_function_value_is_used.em` calls a bound function value and prints 42 without the lint; reverting only the callee read reproduces `L1001`. The specification was already clear and did not change |

---

## 2026-09-13 — for-loop borrowing and B2 diagnostics

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-070 | **Source `for` loops lost both their borrowing semantics and their diagnostic identity during desugaring.** Canonical `for value in values` over an `Array[T]` moved the Array into a hidden local instead of borrowing it for the loop and yielding `ref T`, so an in-loop mutation reported E3050 after the move. The explicit-iterator spelling retained the loan but reported generic E3021/B3 rather than `[CTL-2]`'s E3020/B2 | `[CTL-1]`, `[CTL-2]`, `[BRW-2]`, `[DIA-7a]`, `[DIA-10]`, `[DIA-13]` | **fixed** | `e0ba765` lowers direct Array iteration through the existing shared-Span borrow producer and yields a shared reference, while an explicit semantic marker for a source `for` loop travels from HIR to MIR and is interpreted through the loan's region holders. A manually held iterator therefore remains ordinary E3021/B3. **Verified:** the CTL-1 run-pass case leaves the Array usable after iteration, observes `ref T`, and drops a non-`Copy` payload exactly once; both CTL-2 cases prove whole-loop retention and post-loop release; the B2 UI case pins exact E3020 output and a compiling fixed companion. Removing either the direct-loop or generic-loop marker makes the UI case report E3021, and the original direct-loop probe reported E3050. Generated C borrows the Array through a pointer-plus-length Span, represents the item as `const T*`, and performs one final payload drop/free. The specification was already correct and did not change |

---

## 2026-09-13 — concrete Box ownership and region storage

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-069 | **Arena-backed return provenance was represented as a loan but not as an origin in the region graph at a wrapper call boundary.** `Arena` is intentionally not a view type, so the ordinary operand-region edge is absent. A new unbounded-storage consumer could therefore mistake an Arena-backed returned view for a static one even though the existing borrow checker still kept the Arena loan live | `[LT-1a]`, `[LT-3]`, `[LT-4]`, `[LT-4a]`, `[TYP-15]` | **fixed** | `0fdd9b6` seeds the destination region from a tied growing-Arena argument while retaining Arena's non-view identity. It does not generalize the exception to arbitrary non-view arguments: a scalar argument ignored by a function returning a literal leaves that result static. **Verified:** `reject_arena_backed_view_in_box.em` follows an allocation through an `@borrows(arena)` wrapper and reports E3063, while `accept_static_region_view_provenance_in_box.em` accepts a literal returned from a function with an unrelated scalar argument. The specification already distinguished the Arena provenance exception and did not change |
| D-068 | **The first Box type path rejected every view at type formation and then recognized only direct literal syntax.** This made `Box[str]` illegal before a value was known and would reject a static view after binding it to a local, reading a named `static`, or returning it from a call. `[TYP-15]` is region-based and explicitly permits every-static-region view in unbounded storage | `[TYP-15]`, `[LT-3]`, `[DRP-6]` | **fixed** | `de641fb` moved the prohibition from type formation to the stored value; `0fdd9b6` moved the final decision after MIR region inference so provenance, not spelling, governs. **Verified:** direct literals, locals, named statics, and call-produced static views run; a local Array span and an Arena-backed mutable reference report E3063. The specification was already correct and was not weakened |
| D-067 | **The first Box backend path emitted a one-field wrapper struct around `T*`.** The checker deliberately uses a private logical field so existing place, borrow, and ownership machinery can see auto-dereference, but Part XIX §6 requires the C representation of concrete `Box[T]` itself to be `T*`; leaking the compiler wrapper into C changed layout and ABI | Part XIX §6, `[HEAP-1]`, `[DRP-6]`, `[CG-C-1]` | **fixed** | `de641fb` recognizes Box by compiler-owned generic-origin metadata and erases only that logical wrapper to a C pointer alias. Projections emit direct pointer dereference, while drop glue destroys `T` and then frees the same pointer. **Verified:** generated C contains `typedef em_Payload* em_Box_Payload;`, nested Box produces exactly two frees, destructor-before-free order is asserted, and all generated Box translation units pass Clang `-std=c11 -pedantic -Wall -Wextra -Werror`. The target specification did not change |

---

## 2026-09-13 — diagnostic snapshot foundation

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-066 | **An ordinary `mut` argument that was not a mutable place emitted the generic expression-place error E2140 instead of the mandatory ownership shape E3027/B10.** The compiler already rejected the program and accepted `[FN-1a]` view-producing expressions, but it lost the callee-mode context needed to name the correct structural repair | `[FN-2a]`, `[DIA-7a]`, `[DIA-13]` | **fixed** | `8f16a7f` gives the ordinary argument adjustment one classified producer, while leaving method receivers and compiler-known place requirements on their existing diagnostics and preserving the `MutSpan` view exception. **Verified:** `tests/ui/borrow/B10/` pins the exact rendering and compiles the bind-to-local repair; `tests/conformance/FN-2a/` checks both accepted inout behavior and the rejected value expression; changing the producer back to E2140 makes the UI test fail with the code mismatch; the existing FN-1a worked example remains green. E3027 now has an executable error page. The specification was already correct and did not change |
| D-065 | **Windows Error Reporting blocked every full conformance run at the first aborting program.** Ember printed the required panic and called `abort()`, but the UCRT then opened a WerFault dialog; the compiler process waited for that child indefinitely, so a semantically successful `run-fail` case could never return to the harness | `[PAN-1]`, `[TST-1]` | **fixed** | `c520a32` disables only `_CALL_REPORTFAULT` immediately before the existing Windows `abort()` call. Ember's own stderr, callback, and process termination are unchanged; non-Windows builds are untouched. **Verified:** `ARN-9/run_fail_write_at_is_bounds_checked.em` previously left `WerFault.exe` and the compiler waiting after its panic; it now returns nonzero immediately with the same panic text, and the complete conformance walker finishes green in 66 seconds. The specification already required abort-only panic behavior and did not change |
| D-064 | **Two overlapping mutable call arguments were diagnosed as a shared-plus-mutable conflict.** A mutable argument's two-phase reservation correctly remained non-active while later arguments were evaluated, but diagnostic selection reused that temporary access state and emitted E3021/B3 when the second mutable argument overlapped it. The source contains two mutable borrows, so `[DIA-7a]` requires E3022/B1 | `[BRW-1]`, `[BRW-3]`, `[DIA-7a]` | **fixed** | `c520a32` keeps reservation behavior unchanged but selects diagnostic identity from the borrow's declared mutable capability. Exact same-place conflicts now lead with a one-access structural repair and name `split_at_mut` for intended disjoint subparts. **Verified:** `tests/conformance/OWN-6/reject_swap_of_the_same_place.em` emitted E3021 before the change and now emits E3022 with both mutable sites identified. The specification was already clear and did not change |
| D-063 | **The required O2 repair did not compile because `std.mem` exported none of `take`, `replace`, or `swap`.** E3010 correctly forbade moving a field out of a `Drop` value and prescribed the `[OWN-6]` operations, but applying that repair produced E1010. Merely adding source stubs was also unsound: namespace-qualified calls instantiated empty generic bodies and returned uninitialized values | `[OWN-6]`, `[DIA-7]`, `[PHIL-8a]` | **fixed** | `c520a32` adds the public generic surface and one compiler-known vertical path through type checking, HIR, MIR, borrow checking, and portable C lowering. `replace` returns the old value without dropping it, `take` constructs `T.default()` before the exchange, and `swap` exchanges without an Ember-visible temporary or destructor. **Verified:** the O2 snapshot's fixed companion compiles; the OWN-6 run-pass case checks exact non-`Copy` values and destructor counts; missing `Default`, a live shared borrow, and overlapping swap operands are rejected; MIR contains explicit exchange operations; reachable generated C contains no stub calls and passes Clang C11 `-pedantic -Wall -Wextra -Werror`. The specification was already correct and did not change |
| D-062 | **Shape B1 was emitted with shape B3's repair.** A second mutable indexed borrow correctly reported E3022, but its primary help said only to shorten the first borrow. `[DIA-7]`/`[DIA-10]` require a concrete structural API such as `split_at_mut`, `iter_mut`, `columns_mut`, or `mem.assert_disjoint`; at discovery time none of the Array disjointness APIs needed to make that prescribed repair compile existed | `[DIA-7]`, `[DIA-10]`, `[BRW-5]`, `[PHIL-8a]` | **fixed** | `352ea64` adds compiler-known `Array.split_at_mut`, whose MIR performs one checked boundary evaluation, returns two disjoint `MutSpan` views, and retains the source Array's mutable loan. E3022/B1 now prescribes that API plus the other rule-named structural repairs. **Verified:** the exact `tests/ui/borrow/B1/` snapshot is paired with a compiling fixed program; BRW-5 cases execute ordinary and empty splits, reject mutation of the Array while either returned view lives, and reject an out-of-range boundary. Renaming the method made the fixed UI case fail with E1010 before restoration. The specification was already clear and did not change |
| D-061 | **The O3 diagnostic did not emit O3's required repair set.** E3041 correctly identified a move in a loop, but suggested a generic borrow and omitted the catalogue's declaration-inside-loop and `mem.take` repairs. A code-to-shape mapping without the shape's help still violates `[DIA-7]` | `[OWN-4]`, `[DIA-7]`, `[DIA-7a]` | **fixed** | the E3041 producer now names declaration inside the loop, per-iteration clone, and `mem.take` for replacement. **Verified:** `tests/ui/borrow/O3/move_in_loop.stderr` pins the exact rendering and `move_in_loop.fixed.em` applies the first repair and compiles. The specification was already clear and did not change |
| D-060 | **The compiler prescribed `mem.drop(x)`, but `std.mem` exported no `drop`.** Applying E3070's mandatory O7 help therefore produced E1010 instead of a compiling program, violating `[PHIL-8a]` even though explicit destructor calls were correctly rejected | `[DRP-1]`, `[DIA-7]`, `[PHIL-8a]` | **fixed** | `std.mem.drop[T](owned value: T)` is now an ordinary public generic function: ownership transfers to its parameter and ordinary scope-end destruction runs exactly once. **Verified:** `tests/conformance/DRP-1/accept_mem_drop_ends_a_value_once.em` prints `1, 2`; `tests/ui/borrow/O7/explicit_drop_call.fixed.em` now compiles, while the paired snapshot still rejects `r.drop()` with E3070. No compiler-only destruction path or specification edit was added |

---

## 2026-09-13 — H4 Hash and ArenaMap completion

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-059 | **Method-level attributes were read from the enclosing type instead of the method.** A method carrying `@static_safe` therefore entered type checking as an ordinary method, while an attribute on the enclosing type could be applied to every method accidentally. The gap became observable when `[UNS-10b]` required E3105 for an `UnsafeCell` constructed inside a `@static_safe` method | `[EFF-12]`, `[UNS-10b]` | **fixed** | every method-checking path now passes the member's own attributes into body checking, including generic recipes and concrete instantiations. **Verified:** `tests/conformance/UNS-10b/reject_unsafe_cell_in_static_safe_method.em` reports E3105; restoring the enclosing-item attribute source makes the program compile. The specification already attaches attributes to the declaration they annotate, so no specification or ADR changed |
| D-058 | **An explicitly imported prelude interface acquired two identities during method ambiguity checking.** `from std.core import Ord` correctly resolved a generic bound and implementation to `std.core.Ord`, but an `extend T implements Ord` method entry retained the unqualified spelling `Ord`. Once the source-backed prelude was loaded, an ordinary `a.cmp(b)` was therefore reported as offered by both `Ord` and `std.core.Ord`, even though both names denoted one definition | `[MOD-3]`, `[MOD-5]`, `[TYP-24]` | **fixed** | extension methods now retain the resolved interface identity used by bounds and implementation records. **Verified:** the existing adversarial module case `tests/conformance/TYP-17/accept_an_imported_bound.em` again prints 7, and the complete conformance walk passes. The specification already treats an import as a binding, not a duplicate interface, so no specification or ADR changed |
| D-057 | **Concrete generic-instantiation diagnostics were silently discarded unless they were E3090.** Generic recipes are rechecked after substitution, but the quiet diagnostic sink forwarded only one ownership code. A concrete view key could therefore violate `ArenaMap`'s no-view storage rule during monomorphization while compilation continued; the same hole could hide any non-E3090 error visible only after substitution | `[TYP-16]`, `[TYP-18]`, `[TYP-15a]` | **fixed** | every distinct concrete-instantiation diagnostic is now forwarded; ownership diagnostics retain classified emission and all others use the ordinary sink. **Verified:** `tests/conformance/TYP-18/reject_concrete_instantiation_diagnostic_is_not_discarded.em` keeps the generic recipe valid for opaque `K` but instantiates it with a `@view` key and now reports E2130. Before the fix that concrete diagnostic was dropped. The specification was already correct and did not change |
| D-056 | **Compiler-known `Eq + Hash` key capability changed when passed through a generic bound.** Direct `ArenaMap[i32, V]` construction was accepted, but the same type supplied to `fn reserve[K: Eq + Hash, V]` failed E2040 because ordinary bound solving did not recognize the compiler-known scalar capability | `[TYP-17]`, `[TYP-18]`, `[HASH-4]` | **fixed** | the standard capability predicate now participates in ordinary interface-bound solving as well as direct ArenaMap checking. **Verified:** `tests/conformance/HASH-4/accept_arena_map_with_a_custom_move_only_key.em` instantiates one generic constructor with `i32` and one with a custom key and runs both. The rule was already uniform; no specification change was made |
| D-055 | **ArenaMap lowering assumed keys and values were `Copy`.** Search consumed an inserted move-only key before the missing-slot path stored it; replacement and removal copied move-only values; compaction copied move-only keys and values; and the returned `Option` copied the moved-out value again. This made the new user-defined-key surface either reject valid moves or duplicate values behind the compiler's ownership model | `[OWN-3]`, `[ARN-5d]`, `[HASH-4]` | **fixed** | search borrows the staged key and insertion performs the one final move; replacement, removal, result construction, and suffix compaction select `Copy` or `Move` from the actual type. **Verified:** `tests/conformance/ARN-5d/accept_custom_keys_and_move_only_values_replace_and_remove.em` exercises duplicate replacement, removal, and compaction with move-only `Key` and `Payload`; the HASH-4 custom-key case exercises the same paths through generic wrappers. The specification already required ordinary ownership semantics and was not changed |
| D-054 | **Making source-backed prelude interfaces available caused unreachable standard-library bodies to pollute every generated executable.** Loading `std.collections` made the C emitter include all `DefaultHasher` methods, including dormant checked-operation panic paths. The existing `[RNG-3]` test for a statically in-range construction then saw `ember_panic` in generated C even though the program could not reach it | `[MOD-5]`, `[RNG-3]`, `[CG-C-1]` | **fixed** | executable C emission now retains all package bodies but includes standard bodies only when transitively referenced by a direct call or function constant; standard drop bodies remain roots because drop glue names them through type information. Analysis and MIR inspection still see the complete program. **Verified:** `tests/conformance/RNG-3/accept_constant_in_range_emits_no_check.em` again contains no panic path, while HASH-1 programs retain and execute the exact `DefaultHasher` methods they call. Generated C for the custom-key path also passes Clang C11 `-pedantic -Wall -Wextra -Werror`. The specification distinguishes availability from reachability and did not change |

---

## 2026-09-13 — Arena-collection generic integration

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-053 | **An unknown generic-looking type was diagnosed as a known non-generic type.** Using `ArenaArray[i32]` without importing `std.collections.ArenaArray` reported that `ArenaArray` “does not take type arguments in this phase,” implying that the name resolved and its arity was wrong. The actual failure was name resolution | `[MOD-3]`, `[DIA-2]` | **fixed** | type-application resolution now distinguishes a known non-generic type from an unknown type name. **Verified:** `tests/conformance/ARN-5a/reject_collection_types_are_not_prelude_names.em` reports E1010 `cannot find type 'ArenaArray' in this scope`. The specification was already clear and did not change |
| D-052 | **Compiler-known Arena type recognition could panic on unrelated generic types.** The helper predicates for `ArenaArray`, `ArenaMap`, and their iterators used Rust `then_some(...)`, whose payload is evaluated eagerly. A generic owner with the wrong arity therefore indexed absent arguments even when its name did not match, crashing the compiler during an existing `[TYP-18]` program | `[TYP-18]`, `[TST-1]` | **fixed** | all three recognizers now guard identity and arity before indexing arguments. **Verified:** `tests/conformance/TYP-18/accept_generic_associated_function_on_a_generic_owner.em` again prints `56` and `57`; the full conformance run reaches completion instead of panicking. No specification change was needed |
| D-051 | **Imported generic type recipes were unavailable while root signatures were resolved.** Generic struct bodies were collected module-by-module, so a root signature naming an imported generic collection could resolve the public name before the imported module's instantiation recipe existed. This made otherwise valid generic standard-library types order-dependent | `[MOD-4]`, `[TYP-16]`, `[TYP-18]` | **fixed** | type collection now registers nominal headers globally, collects generic recipes dependency-before-importers, and only then performs ordinary body collection. **Verified:** the public ARN-5a generic wrapper/import cases compile and preserve their Arena result provenance. The mechanism is general and does not special-case one test or collection type; the specification was already correct |

---

## 2026-09-13 — generic-call and generic-method closure

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-050 | **Generic substitution and inference did not traverse `Span`/`MutSpan`.** A parameter `Span[T]` could not infer `T`, and a generic method on `Holder[A]` kept its method parameter at the owner-offset index inside a span after `Holder` was instantiated. Calls therefore reported E2020/E2060 instead of preserving the view type and its provenance | `[TYP-18]`, `[SPN-1]`, `[LT-1a]` | **fixed** | type substitution and unification now recurse through the canonical span representation; checker substitution also rebuilds nested generic structs and compiler-known generic wrappers. **Verified:** `tests/conformance/TYP-18/accept_generic_inference_through_a_span.em` prints 61, and `reject_mutation_while_a_generic_owner_method_result_borrows_its_argument.em` now reaches the intended E3021 rather than E2020/E2060. The latter also pins the shifted `@borrows` parameter position after the concrete receiver is restored. The specification was already clear and did not change |
| D-049 | **A solved generic type did not flow into a callable parameter's expected signature.** In `fn apply[T](value: T, transform: fn(T) -> T)`, inference solved `T`, but the lambda was still checked and emitted against opaque `T`; the generated program returned zero instead of the argument. A callable written before the argument that solved `T` had the same order-sensitive hole | `[TYP-18]`, `[TYP-23]`, `[CLO-3]` | **fixed** | generic-call inference collects non-lambda constraints before checking lambdas, then substitutes every fact already solved into the callable hint. **Verified:** `tests/conformance/TYP-18/accept_solved_type_flows_into_callable_parameter.em` puts the callable first and prints 54; `accept_generic_method_callable_parameter.em` prints 51. Before the substitution fix the method case printed 0. The specification was already clear and did not change |
| D-048 | **An explicit generic argument was contradicted by an unsuffixed literal's default type before coercion.** `identity[i64](53)` and `.pick[i64](42)` reported E2020 because inference compared the fixed `i64` with prematurely defaulted `i32`, even though `[TYP-5]` makes the call argument a coercion site and `[TYP-18]` makes the written argument authoritative | `[TYP-5]`, `[TYP-18]` | **fixed** | unification now treats the explicit prefix as fixed and leaves compatibility to the substituted argument check, where the literal adopts `i64`; excessive explicit arguments are rejected rather than ignored. **Verified:** `tests/conformance/TYP-18/accept_explicit_generic_argument_controls_literal_coercion.em` prints 53 and `accept_source_generic_method_inference_and_explicit_arguments.em` prints both inferred 41 and explicit `i64` 42. Before the fix the latter emitted E2020. The specification was already clear and did not change |

---

## 2026-09-13 — associated functions and interface conformance

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-047 | **An interface implementation was accepted by member name alone.** `implements Default` was considered satisfied by any receiver-less function named `default`, even if it took arguments or returned the wrong type. Any compiler-known consumer trusting the interface contract could therefore lower a zero-argument call against an incompatible body | `[IFC-1]`, `[TYP-17]` | **fixed** | whole-program interface checking now compares receiver presence/mode, parameter count, parameter types/modes after `Self` and associated-type substitution, and return type. **Verified:** `tests/conformance/TYP-17/reject_an_associated_implementation_with_the_wrong_signature.em` reports E2040; disabling only the signature comparison makes it compile with exit 0. The specification was already clear, so it did not change |
| D-046 | **Interface conformance and default-body registration were module-order dependent.** Conformance was checked once after each module rather than after the whole method-collection pass, and default bodies compared an unqualified declaration name with qualified implementation identities. A valid implementation supplied later could be rejected early; an imported interface's default body could be registered without a checked/emitted function body, leading to an internal compiler panic when called | `[IFC-1]`, `[MOD-4]` | **fixed** | conformance now runs once after all modules are collected; default registration and body checking use the qualified interface identity and bind `Self` to the implementing type. **Verified:** `tests/conformance/TYP-17/accept_an_imported_interface_default_body.em` prints 41. Reverting only qualified default-body lookup makes the compiler panic because the registered `DefId` has no checked function body; restoring the fix returns 41. The specification was already clear, so neither it nor an ADR changed |

## 2026-09-13 — 0.9.6 Arena initialization boundary

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-045 | **`Cell.set` and `Cell.replace` evaluated a value argument before resolving their receiver place.** The special lowering correctly preserved `[CELL-1]`'s store-new-before-drop-old replacement order, but called `lower_operand(value)` before `lower_place(cell)`. For `cells[choose()].set(make_value())`, the observable call order was therefore `make_value()` then `choose()`, contrary to the receiver-before-arguments order required for every method call. This survived because existing Cell ordering tests observed only replacement/destruction, and explicit nested type arguments in the adversarial `Array[Cell[i32]]` probe first exposed an independent generic-resolution gap | `[EXP-1]`, `[CELL-1]` | **fixed** | MIR lowering now resolves the complete receiver place, including index side effects and bounds machinery, before evaluating the value argument; it still moves the old payload, stores the new payload, and only then drops the old value. **Verified:** `tests/run-pass/cell_receiver_is_evaluated_before_its_argument.em` prints `1, 2, 7`. Mutation-testing only the two lowering calls back to their old order prints `2, 1, 7`, making the run-pass suite red; restoring the fix returns `1, 2, 7`. The specification was already explicit, so neither the specification nor an ADR changed |

---

## 2026-09-12 — `[CELL-10]` / borrowed-parameter write closure

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-044 | **Writes through default-mode value parameters were accepted and silently lost.** `[FN-1]` makes an omitted parameter mode a shared borrow, but the type checker retained only the parameter's ABI-local type. A small value therefore looked like an ordinary writable local: `fn increment(counter: Counter): counter.value = counter.value + 1` compiled, mutated only the callee's private by-value copy, and returned with the caller unchanged. The same hole admitted `ref mut`, mutable-view creation, `mut self`, and forwarding to a `mut` parameter | `[FN-1]`, `[BRW-1]`, `[CELL-10]`, `[DIA-7]` shape B4 | **fixed** | type checking now retains default-mode parameter provenance independently of ABI representation and rejects every write-capable access rooted there as `E3023`. The diagnostic's first help is the structural single-owner/`mut` repair; `Cell`/`RefCell` and class are later, costed alternatives only at sites proven not simultaneous. Call boundaries conservatively omit `RefCell` until the whole call can establish that condition. **Verified:** the minimal assignment case failed the conformance suite before the fix; disabling the new provenance check made all five E3023 probes under `tests/conformance/CELL-10/` compile with exit 0. Ordered and forbidden-help directives were each mutation-tested red, and `docs/errors/E3023.md` contains an executable failing/fixed pair. The specification was already explicit; neither it nor an ADR changed |

## 2026-09-12 — D-042 move-path closure

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-043 | **An `owned` parameter was never destroyed by its callee.** `[FN-1]` transfers ownership at the call, but MIR lowering registered only body locals for scope-end destruction; owned parameters were absent from `Builder.owned`. A callee that accepted `owned resource: Resource` and did not move it onward returned without running `Resource.drop`, leaking everything the argument owned. The per-field call-argument probe for D-042 exposed this as a missing destructor line, independently of D-042's source-field flag | `[FN-1]`, `[OWN-2]`, `[OWN-3]` | **fixed** | owned, droppable parameters are registered as the function's outer ownership scope; explicit returns already discharge that scope and fallthrough now does so after body locals. Move-path flags start live for arguments and call-terminator moves clear them. **Verified:** `tests/conformance/OWN-2/accept_owned_parameters_drop_in_the_callee.em` covers both the move-onward and not-moved paths; `tests/conformance/EXP-6/accept_partial_move_into_a_call_clears_the_field_flag.em` first failed with the transferred value never dropped by the callee (`10, 2, 2, 1` instead of `10, 1, 2, 2, 1`). The specification was already explicit; it was not changed |

## 2026-09-09 — task 2, the D-035 sweep

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-036 | **Pushing a moved value destroyed it twice.** `Array.push` spilled its value argument through a temporary and erased the move to a copy, so the temporary kept its statement-end drop while the buffer took ownership and dropped the element at scope end. `a.push(R(1))` printed 1, 2, 1, 2 — for a value owning a buffer that is a double free. An `owned` parameter of a user function never had this shape (`take(R(a))` destroys once); only the builtin push path erased the move | `[OWN-3]`, `[OWN-2]` | **fixed** | the push argument keeps the shape `lower_operand` gives it; only constants (no address: `&10` is not C) still spill. **Verified:** `tests/conformance/DRP-2/accept_array_elements_drop_in_index_order.em` prints 1, 2 and goes "1 2 1 2" red when the fix is reverted — checked by doing exactly that |
| D-037 | **`as_str()` built a view with no borrow behind it.** It lowered straight to the builtin, so `collect_loans` saw no `Rvalue::Ref`, the elision table tied the result to nothing, and `s.push_str("!")` while the `str` was live compiled — the view dangled across the reallocation. The twin of D-022, which fixed the same hole for spans; strings were missed | `[SPN-1]`, `[BRW-1]`, `[UNS-4]`, `[PHIL-10]` | **fixed** | `as_str()` goes through the one producer (`view_of`, generalised to take the builtin tag), the elision table ties `StringAsStr` results to the receiver, the backend reads the borrowed pointer, and `verify_views` checks the new path too. **Verified:** `tests/conformance/SPN-1/reject_mutating_around_a_live_str_view.em` fails to compile as required and compiles when the fix is reverted — checked by doing exactly that |
| D-038 | **Implicit `String`→`str` coercion was rejected.** `[SPN-1]` lists "String to `str`" among the coercions and Part XIX §5 records it as one the checker applies, but `v: str = s` was `E2020`. The explicit `as_str()` spelling already worked soundly after D-037, so this failed closed | `[SPN-1]` | **fixed** | `coerce` recognizes the compiler-known `String` representation when `str` is expected and routes it through the existing `view_of(..., StringAsStr)` producer. The implicit and explicit spellings therefore share the same explicit IR borrow, elision, verifier, and backend path. **Verified:** three `[SPN-1]` cases cover assignment and argument coercions, NLL ending a call-local borrow, E3021 on mutation across a live implicit view, and E3060 when returning a view of non-view parameter storage. Removing only the coercion arm restores E2020 in all three probes. The specification was already explicit and was not changed |
| D-039 | **Reservation windows opened for user-local borrows, misclassifying conflicts.** Any borrow whose borrower was later used as a call argument was treated as two-phase-reserved — including plain bindings — so two `ref mut` borrows of one field reported B3/`E3021` instead of B1/`E3022`. A reservation is taken *for* a call; its borrower is always a lowering temporary | `[BRW-3]`, `[BRW-4]`, `[DIA-7a]` | **fixed** | windows open only for temporaries. **Verified:** `tests/conformance/BRW-4/reject_two_mutable_borrows_of_one_field.em` reports `E3022` and reports `E3021` when the fix is reverted — checked by doing exactly that; the full suite stays green, including the two-phase accept cases |
| D-040 | **Method-call conflicts have no code: `E3025` is registered, shape-mapped, and emitted by nothing.** A method call defeating disjoint-field access (`[BRW-4]`'s parenthetical) reports `E3022`, where `[DIA-7a]` keys shape B8 to `E3025`. Keying it needs call provenance at the report point — the conflicting access is recorded as a plain borrow — which is design work, not a lookup | `[BRW-4]`, `[DIA-7a]` | **fixed** | the reporter walks the autoref temporary forward to its consuming call and reports `E3025`/B8 when the callee is a method body; free-function `mut` arguments keep `E3022`. **Verified:** `tests/conformance/BRW-4/reject_method_defeats_disjoint_fields.em` reports `E3025` and reports `E3022` when the fix is reverted — checked by doing exactly that; `docs/errors/E3025.md` holds the page |
| D-041 | **Moves out of borrowed places are unchecked.** `x = r.inner` through a shared `ref` and a field move out of a borrowed call parameter both compile; each value then drops twice — once with its new owner, once with its original place. `[EXP-6]` names `E3013` ("cannot move out of a reference") for exactly this, and it has no emitter outside drop bodies. A borrowed parameter arrives as a bitwise copy whose borrowed-ness reaches no analysis (no loan, no marker), so the callee cannot tell it apart from owned data | `[EXP-6]`, `[BRW-1]`, `[OWN-3]`, `[FN-1]` | **fixed** | borrowed-ness threaded HIR→MIR (`borrowed_params` on `Body`, from the parameter modes); `check_borrowed_moves` in `compiler/ember_analysis/src/drops.rs` reports every owning `Move` through a safe-reference `Deref` and every owning `Move` out of a borrowed value parameter (whole or field, assignment and call-argument/return paths) as `E3013`/O2 — raw-pointer derefs exempt (`[UNS-*]` territory), `drop`'s `self` keeps D-030's messages (skipped, never double-reported), moves of values owning nothing stay legal. **Verified:** `tests/conformance/EXP-6/` holds five rejects that each compile with the check reverted (exit 0; the suite fails "expected compilation to fail, but it succeeded") plus a run-pass accept whose stdout mutation fails the suite — every new case broken red once, checked by doing exactly that (the return-path case additionally fails on a deliberately wrong code, pinning `E3013` specifically). `docs/errors/E3013.md` holds the page (baseline shrinks by one). The specification is untouched: `[EXP-6]`/`[FN-1]` already forbid both shapes (cf. D-035). Adjacent symptom, lost-write only: writes through borrowed places go nowhere (same by-copy ABI), e.g. through a borrowed `o` — needs a spec reading on the intended code before filing separately (not ERR-041) |
| D-042 | **Partial moves out of owned places double-destroyed at scope end.** `[EXP-6]` allows moving a field out of a plain struct ("leaves the struct partially moved"), but drop elaboration tracked whole-local movedness only — `moved_by_operand` discarded field projections — so the scope-end drop destroyed the moved-from field again. `x = o.inner` emitted both the new owner's drop and `o.inner`'s old drop. `E3042` was registered and shape-mapped (O4) but unreachable | `[EXP-6]`, `[OWN-2]`, `[OWN-3]`, `[DRP-2]` | **fixed** | `drops.rs` now builds recursive move paths for plain structs/tuples, carries `Live`/`Moved`/`Partial` possibilities through control flow, preserves disjoint siblings, restores an aggregate after field reinitialisation, expands a partial aggregate's cleanup into reverse-order live-field drops, and maintains per-path flags for conditional moves including call terminators. Whole-value use after a partial move emits `E3042`; `docs/errors/E3042.md` is the page. **Verified:** six adversarial cases in `tests/conformance/EXP-6/` cover exact destructor count, conditional flags, call-argument transfer, nested paths, reinitialisation, sibling use and `E3042`; the old implementation failed the first probe with `1, 2, 1` and failed the conditional probe with `10, 1, 2, 1`. Removing projected-move tracking makes the new suite red. The normative specification and frozen H8 target were unchanged: the compiler was wrong |

## 2026-09-09 — block I, `Cell[T]`

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-035 | **Overwriting a place ran no destructor at all.** `[OWN-5]`: "Overwriting a place that holds a live value drops the old value first (after evaluating the new value)." `[OWN-2]` names three occasions to drop — scope end, overwrite, temporary at statement end. D-027 built the first and D-031 the third; **the second was never built.** `r = R(1)` then `r = R(2)` ran `R(1)`'s `drop` zero times, and `a = Array[i32]()` over a live array emitted one `ember_vec_free` for two allocated buffers, leaking the first. Silent in both directions: no diagnostic, and for the container the program's output is identical either way | `[OWN-5]`, `[OWN-2]` | **fixed** | ADR-020. `lower_assign` in `ember_mir/src/lower.rs`: the new value goes to a temporary, then `Drop`, then the store — the order the rule gives. Which of those drops survives is left to `[OWN-3]`'s existing elaboration, so a first initialisation (local `Moved` after `StorageLive`) deletes it and a conditionally-moved local gets a flag. **Verified:** two cases in `tests/conformance/OWN-5/`, both of which lose their first line of output when `lower_assign` is reverted to `lower_into` — checked by doing exactly that. The buffer case is written observably (a `Bag` whose `drop` prints its length) rather than as `assert-c: contains("ember_vec_free")`, which would have passed on the broken compiler: it emitted one free and `contains` cannot count |
| — | **Why the specification does not move.** `[OWN-5]` states the requirement and the order in one sentence and admits no other reading; the compiler simply did not implement it. An implementation gap is not a spec defect, so there is no errata entry and `ember-spec.md` is untouched | `[OWN-5]` | n/a | — |
| — | **Why `tests/conformance/OWN-5/` did not catch it.** The directory's one case, `accept_the_new_value_is_evaluated_first.em`, tests the parenthetical only, and it is built so the main clause cannot fire: `x = grow(x)` **moves** `x` into the call, so nothing is live at the store and the missing drop is unobservable. A rule stated in two clauses needs a case per clause | `[OWN-5]`, `[TST-4a]` | n/a | — |

## 2026-09-09 — the region work

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-001 | A borrow copied into another local was not tracked: `s = r` killed the loan with `r`, so writing the owner afterwards compiled | `[BRW-1]`, `[LT-5]` | **fixed** | `1204a5e`, ADR-010 |
| D-002 | A reborrow (`q = ref mut r`) did not keep the borrow it derived from alive | `[BRW-6]`, `[LT-5]` | **fixed** | `1204a5e` |
| D-003 | A call handing a reference back did not keep its argument borrowed | `[LT-1]` | **fixed** | `1204a5e` |
| D-004 | `[DIA-3]`'s "later used here" label was missing from every borrow diagnostic, and the help named the local the borrow *started* in | `[DIA-3]` | **fixed** | `1204a5e` |
| D-005 | `@borrows` was checked as a signature only: `@borrows(a)` on a function returning `b` compiled | `[LT-1a]` | **fixed** | `c1bd89c`, ADR-011 |
| D-006 | `E3060` exempted every parameter, so a borrow of a **by-value** parameter could be returned | `[LT-1]`, XVIII §4.7 step 6 | **fixed** | `c1bd89c`, ERR-024, ADR-011 |
| D-007 | `[TYP-14]`'s read-through worked only for a named local, so a call returning `ref i32` was not an operand of `+` (`E2020`) and reached `println` as a pointer | `[TYP-14]` | **fixed** | `c1bd89c`, ERR-023 |
| D-008 | `ember fmt` deleted a doc comment on a method | `[FMT-1]` | **fixed** | `28aa05d` |
| D-009 | `ember fmt` deleted a doc comment on an enum variant | `[FMT-1]` | **fixed** | `28aa05d` |
| D-010 | The AST printer showed a variant as its bare name, so `[FMT-1]`'s round-trip test could not see D-009 | `[FMT-1]`, `[TST-1]` | **fixed** | `28aa05d` |
| D-011 | Under the adopted one-region `[LT-2]` model, `E3064` (two independent regions in one view struct) was registered but emitted by nothing | `[LT-2]`, `[DIA-7a]` | **fixed** | The adopted-source compiler originally classified the reachable intersection conflict as generic B3; the fix made it E3064/B13. The frozen H6 target later superseded that one-region model with field-sensitive region vectors, so `90059c8` removes the rejection and reserves the stable diagnostic identity instead of reusing it. Current H6 behavior is ordinary E3021 for a real live-field conflict or E3065/B14 when a multi-region result lacks sound field provenance. This preserves the history without presenting obsolete target behavior as current |

## 2026-09-09 — v0.8.3, the standard library and range types

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-014 | A `##` doc comment above an `import` was `E0100 expected a declaration`. `[LEX-11]` says such a comment "documents nothing and is **discarded in silence**: a comment never affects compilation, and that includes producing a warning" — and an import is not a declaration | `[LEX-11]` | **fixed** | the parser skips a doc-comment run followed by an import; two parser tests and `tests/conformance/LEX-11/` |
| D-015 | Interfaces were collected per module, immediately before that module's `implements`, so a module checked before the one declaring an interface could not implement it. `[MOD-4]` allows import cycles inside a package, so **no** load order would have worked | `[IFC-1]`, `[MOD-4]` | **fixed** | interface collection is whole-program, like the name pass above it; `tests/conformance/TYP-17/` |
| D-016 | A generic bound, an `implements` clause and a supertrait each recorded the name **as written**, so an imported `Ord` did not match an implementation of the same interface | `[TYP-17]`, `[IFC-3]` | **fixed** | all three record the resolved name |
| D-017 | `Self` did not resolve in any signature: `interface Clone: fn clone(self) -> Self` was "this type is not supported yet". Part IV §8 declares nine interfaces over `Self`, so `std.core` could not be written at all | Part IV §8, `[TYP-22]` | **fixed** | `Self` is the concrete type in a body and a parameter in an interface, substituted at each use |
| D-025 | `RangeError` was created on **first use**, so it did not exist while signatures were being collected and `[RNG-3]`'s own worked example — `fn from_slider(x: f32) -> Result[Roughness, RangeError]` — did not compile. Found by `tools/error_pages.py` refusing a page whose "fix" did not compile | `[RNG-3]` | **fixed** | registered as a **prelude type** before any module is walked. Not a `std.core` declaration: `checked` is a language-defined construction under `[RNG-10]`, not a library function, so its error type cannot wait on an import; `tests/conformance/RNG-3/accept_range_error_is_nameable.em` is the spec's own signature, compiled and run |
| D-026 | A write through a **shared** `ref` passed every check in the compiler and was caught only by the C backend's `const` (`error: read-only variable is not assignable`). ADR-010 makes a reference local non-re-seatable, so `r = 99` and `r = ref y` both write *through* `r` — sound for `ref mut`, and exactly the write `[BRW-1]` forbids for a shared borrow. Found while auditing `[LT-2]`/`E3064` on the owner's list | `[BRW-1]`, `[TYP-14]`, `[CG-C-1]` | **fixed** | rejected in typeck, where the type alone answers it and no flow analysis is needed; `tests/conformance/BRW-1/` has the reject and the `ref mut` accept beside it. Aliasing-XOR-mutability was being upheld by the backend rather than the language, which is luck, not a rule |
| D-024 | A call argument was parsed as a full expression, so a lambda's `:` body inside brackets consumed an indented block. `[LEX-6a]` makes it "a single `small_stmt`, terminated by the enclosing closing bracket or by a `,`", and indentation is not significant inside brackets, so there was nothing for a second statement to belong to | `[LEX-6a]`, `[GRM-17]` | **fixed** | arguments parse with block lambdas disabled, and `E0106` — registered and emitted by nobody until now — reports a body that is not one statement, with `[GRM-17]`'s mandated help |
| D-022 | `[SPN-1]`'s coercion built a view without borrowing its container, so `v: Span[i32] = a` followed by `a.push(…)` compiled — the push reallocates and the view dangles. Found by asking the question rather than by a test failing | `[BRW-1]`, `[SPN-1]`, `[UNS-4]`, `[PHIL-10]` | **fixed** | the coercion takes an explicit borrow, which is what `collect_loans` looks for, and the call's elision carries the loan's region to the view; `tests/conformance/BRW-1/` |
| D-023 | `lower_span_get` assumed `Some` was `Option`'s variant 0. `option_of` builds `None` first | `[SPN-2]` | **fixed** | the variant order is read from the type table: it is a choice inside one function, not a language rule |
| D-019 | Field visibility was not enforced at all: `[MOD-2]` makes a field private unless `pub`, and every field of every struct could be read from any module | `[MOD-2]` | **fixed** | `FieldDef` carries `FieldVis`; a private field read from another module is `E1020`. It rejected `tests/run-pass/modules_across_files.em`, which had been passing on the strength of the gap |
| D-020 | `[MOD-7]`'s `pub(read)` was parsed and then dropped: `E1050` was registered and emitted by nobody, so a read-only field was writable from anywhere | `[MOD-7]` | **fixed** | the flag reaches `FieldDef`; assignment, augmented assignment, `ref mut` and a `mut` argument each check the whole projection chain |
| D-021 | `[STR-1]`'s memberwise constructor was `pub` regardless of its fields, so a struct with private or `pub(read)` fields could be constructed from any module — which writes them | `[STR-1]`, `[MOD-7]` | **fixed** | the constructor is checked against every field's visibility at the call |
| D-018 | `[RNG-4]`'s range tracking derived a range for constants only, so `[RNG-10]`(d) — "a value whose `[RNG-4]` range is contained in the target's" — admitted a constant and nothing else | `[RNG-4]`, `[RNG-4a]`, `[RNG-10]` | **fixed** | `range_of` returns an **interval**, not a point. The fact carrying most of the weight is not arithmetic: a value at a range type is in its declared range, which `[RNG-9]` makes an invariant and `[RNG-4]` may assume. Intervals propagate through unary negation, `+`, `-`, `*`, proven-safe division, proven-safe remainder, proven-safe shifts, exact bitwise constants, proven-safe non-negative `&` masks, non-negative interval `&`, non-negative interval `|`/`^`, and finite floating comparison facts in `if` arms, survive bindings, and are dropped by later writes. `[RNG-4a]` is honoured: a derived fact is kept only where the operation provably cannot overflow its representation in any profile. The remaining `[RNG-4]` transfers are disjunctions and general bitwise facts; D-141 through D-150 record the completed conditional and arithmetic slices. |

## 2026-09-09 — the v0.6 draft

Defects in a **proposed** document, not in the compiler. Re-check both against the
expanded v0.6 specification when it arrives: they were introduced by fixes, and the
same fixes are likely to have been carried forward.

| # | Defect | Rule | Status | Where |
|---|---|---|---|---|
| D-012 | `E1020` is used for "you referred to a type from a library layer this build omitted", and `[GRM-4]` already uses `E1020` for redeclaring a name in the same block | `[BLD-11]`, `[GRM-4]` | **fixed by v0.8.3** | `[BLD-11]` now reports `E1021` and says in terms that `E1020` "remains `[GRM-4]`'s and MUST NOT be reused" |
| D-013 | `[STD-7a]` says const generics are governed by `[CG-1]`..`[CG-4]`; none of those four rules is defined anywhere in the document | `[STD-7a]` | **fixed by v0.8.3** | it now cites `[GRM-8]`, `[CT-1]`, `[MONO-1]` and `[TYP-19]`, all of which exist |

**Two claims of mine that were wrong**, recorded so they are not re-raised as
defects. I reported that `roughness + metallic` compiles — unprovable, because the
conversion rule never says *where* an implicit conversion applies and an operand has
no expected type to convert against, so the real gap is the missing "where". And I
reported that fixed-capacity containers need a language feature Ember lacks — the
grammar has const-generic arguments and fixed arrays already use them, so the gap is
in the compiler, not the design. A claimed contradiction that turns out not to exist
has cost this project three errata already; check before recording one.

### How each was verified

**D-001, D-002, D-003.** Three programs that write through a dangling
reference. Each compiled in silence before and is `E3021` now:

```ember
r: ref mut i32 = ref mut n
s: ref mut i32 = r      ## the loan is held by `s` now
n = 5                   ## …and this was accepted
s = 7
```

`tests/compile-fail/borrows_travel_with_the_reference.em` holds all three, and
`tests/run-pass/references_that_travel.em` holds the same shapes written
correctly, so the fix is not over-rejection.

**D-004.** Visible in the rendered diagnostic: the conflict now carries a
`borrow later used here` label at the next read, and the help names the local
that actually keeps the loan alive — `s` in the program above, where it used to
say `r`.

**D-005.** `@borrows(a) fn pick(a: ref i32, b: ref i32) -> ref i32: return b`
is `E3062`. `[LT-1]` rule 1's case — a method whose result points into an
argument rather than into its receiver — is rejected with the `@borrows` that
would fix it in the help line.
`tests/compile-fail/a_returned_view_elision_cannot_tie.em`.

**D-006.** `fn peek(self) -> ref i32: return ref self.n` is `E3060`. The
exemption is now for view-typed parameters only: what the caller owns outlives
the call, a copy in this frame does not.

**D-007.** `println(give(ref n))` emitted C passing a `const int32_t *` where
`ember_str` was expected — clang caught it, Ember did not — and
`give(ref n) + 1` was `E2020`. Both work now;
`tests/run-pass/returned_views.em` runs them.

**D-008, D-009, D-010.** `ember fmt` on a file with a documented method and a
documented variant keeps both. Verified the way the handoff asks: the fix was
removed and `formatting_is_idempotent_and_preserves_the_tree` went red, then
restored. For D-009 that test went red **only after D-010 was fixed** — the
instrument was blind to the defect it exists to catch.

**D-008 and D-009 needed no specification change, and that is deliberate.**
`[FMT-1]` says "the formatter preserves comments" and `parse(fmt(x)) ≡
parse(x)`; a doc comment is a comment and the rule is unambiguous. Likewise
`[DIA-3]` for D-004: the label is a MUST and the compiler emitted none. Both are
recorded in the errata's closing section so the question is not re-asked.

**D-011 is historical, not open.** Its adopted-source E3064/B13 behavior was
implemented and verified, then superseded by H6's owner-approved multi-region
model. E3064 remains reserved so old logs keep their meaning; the current
target uses field-sensitive regions and E3065/B14 at an opaque result boundary.

---

## Before 2026-09-09

Reconstructed from `docs/HANDOFF.md`, which is where the reasoning for each of
these lives. All are fixed; the ledger did not exist when they were.

| # | Defect | Rule | Status |
|---|---|---|---|
| D-101 | A `##` comment inside a function body deleted the statement under it — the parser returned "no statement" and the caller treated that as a parse failure | `ERR-007` | **fixed** |
| D-102 | `let` was dropped by the formatter, turning an immutable field into a mutable one | `[CLS-9]`, `[FMT-1]` | **fixed** |
| D-103 | Doc comments were dropped by the formatter at the item level | `[FMT-1]` | **fixed** |
| D-104 | The formatter dropped every type parameter list: `struct Pair[A, B]` printed as `struct Pair` | `[FMT-1]` | **fixed** |
| D-105 | The formatter flattened `unsafe:` blocks — the catch-all preserved text and destroyed structure | `[FMT-1]`, `[UNS-1]` | **fixed** |
| D-106 | Methods on a generic struct were silently dropped, so an instantiation had no methods | `[TYP-16]` | **fixed** |
| D-107 | `alloc`'s `arg_ty` was taken from its first argument, a count, so every allocation was `usize`-sized | `[UNS-4]` | **fixed** |
| D-108 | `StmtKind::CheckedBinaryOp` was invisible to the move analysis, so `bigger = cap * 2` read as a use after move | XVIII §4.6 | **fixed** |
| D-109 | A panicking program reported exit code 0: `abort()` arrives as `0xC0000409` and `clamp(0, 255)` made it a clean exit | `[TST-1]` | **fixed** |
| D-110 | `println(10.0)` printed `1e+01` — the shortest *precision* that round-trips, rather than the shortest text | `[FMT-2]` | **fixed** |
| D-111 | `n = match d:` with indented arms did not parse | `[GRM-11]` | **fixed** |
| D-112 | `t.0.1` did not parse: the lexer read `0.1` as one float literal | `[GRM-8]` | **fixed** |
| D-113 | An unused pattern binding tripped `-Wunused-but-set-variable`, breaking the warning-free C requirement | `[CG-C-1]` | **fixed** |
| D-114 | `interface From[T]` could not be declared because `from` was reserved | `ERR-017` | **fixed** |
| D-027 | **A declared `drop` method never ran.** `drop_lines` dropped a struct's *fields* and never called the type's own destructor, so a struct whose only claim on `needs_drop` was its `drop` method produced no lines at all and the `Drop` statement lowered to nothing. Silent: no diagnostic, and output that looks right because the destructor's effects are simply absent | `[OWN-2]`, `[DRP-1]` | **fixed** | the call is emitted **before** the fields, as the rule orders it, so the destructor still sees a whole value — which is the only reason it can read its own fields. Enums too, unit-only ones included. `tests/conformance/OWN-2/` covers the call and reverse declaration order. `drop_symbol` in the backend must agree with `method_symbol` in typeck and nothing checks that it does; those tests are what would notice |
| D-028 | A move inside a loop was reported as `E3040`, shape O1 ("use after move"), where `[OWN-4]` specifies `E3041` and `[DIA-7a]` keys it to shape O3 ("move in a loop"). The rejection was right and the **help was wrong** — O1 says clone it or borrow it, when the problem is that the next iteration finds nothing there | `[OWN-4]`, `[DIA-7a]` | **fixed** | the reporter learns whether its block can reach itself, and a *maybe*-moved local inside a cycle is O3. The states discriminate cleanly: a move and a use in one iteration leaves it definitely moved (O1); a move reaching its own use round a back edge leaves it maybe-moved, because the loop head joins "not yet moved" with "moved last time". `tests/conformance/OWN-4/` has the reject and the escape hatch the rule names |
| D-029 | **Calling `drop` explicitly was accepted, and ran the destructor twice.** `[DRP-1]` says `drop` "is invoked exactly once per value at the end of its life. It may not be called explicitly (`E3070`)" — and an explicit call does not replace the scope-end one, it adds to it. For a struct owning an `Array[T]` that is a **double free**, whose second run also reads the buffer after freeing it, with no `unsafe` in the program | `[DRP-1]`, `[OWN-2]` | **fixed** | `E3070` at the call, with `mem.drop(owned x)` named as the way to end a life early. `tests/conformance/DRP-1/` has the reject and the accept that must keep working |
| D-030 | `[DRP-5]` — a `drop` body moving a field out of `mut self` was accepted, where the rule says it may not (`[EXP-6]`). The field was then dropped again after `drop` returns | `[DRP-5]`, `[EXP-6]` | **fixed** | `check_drop_moves` in `ember_analysis/src/drops.rs` reports every `Move` out of `drop`'s `self` (field moves as `E3010`, whole-through-borrow moves as `E3013`); only a receiver named `self` counts, so a free function named `drop` is untouched. **Verified:** `tests/conformance/DRP-5/` holds two rejects that compile with the check reverted, plus an accept proving reads still work — checked by doing exactly that |
| D-031 | **Temporaries were never dropped.** `[DRP-3]`: "Temporaries drop at the end of the enclosing statement." A bare `R(1)` ran no destructor at all, and a temporary owning an `Array[T]` leaked its buffer. Silent — the program's output is identical either way, which is why it survived block A's leak work | `[DRP-3]`, `[EXP-4]` | **fixed** | every temporary comes through one function, so it registers there rather than at each construction site; a site that forgot would be invisible. Dropped at statement end, last first, kept separate from `[OWN-2]`'s scope-end list because the two differ in *when* and running them through one list gives a temporary the wrong lifetime in either direction. `tests/conformance/DRP-3/` asserts on the emitted C, since the output does not change |
| D-032 | **`[BRW-5]`'s constant-index exemption was unreachable.** The rule says `ref mut a[i]` and `ref mut a[j]` conflict "unless both indices are constants and different", and `overlaps` is written to tell two `ConstIndex` projections apart — but `lower_index` put *every* index into a runtime slot, so no `ConstIndex` was ever produced by an index expression and `v[0]`/`v[1]` were rejected as though the indices might be equal | `[BRW-5]` | **fixed** | a constant index lowers to `ConstIndex`. The bounds check is still emitted: for an `Array[T]` the *length* is dynamic even when the index is a literal, so what the constant buys is disjointness, not the elision of a check. A side benefit — the diagnostic now names `v[0]` instead of `v[…]` (`[DIA-2]`) |
| D-033 | For an `Array[T]`, a second thing blocked the same exemption: `lower_index` reads the length through the container's internal `Field(1)` to bounds-check, and `overlaps` treated that read as overlapping an element borrow | `[BRW-5]`, `[BRW-4]` | **fixed** | a `Field` beside an `Index` is disjoint. Ember has no `v.len` *field* — only a `len()` method — so that pair is only ever the compiler's own header access, never something a program can write. Nothing user-visible is weakened: `push` takes `mut v` with an empty projection, which overlaps every place under `v`, so the reallocation hazard is still caught, and `v.len()` while an element is borrowed is still rejected because `[BRW-4]` says a method takes all of `self`. Both are conformance cases |
| D-034 | Two mutable index borrows of an `Array[T]` reported `E3021` ("cannot be read while mutably borrowed") where `[DIA-7a]` keys shape B1, "two mutable indices", to `E3022`. The fixed-array path already used `E3022` | `[BRW-5]`, `[DIA-7a]` | **fixed** | a consequence of D-033: with the header read no longer overlapping, the conflict reported is the second *borrow* rather than the bounds check's read, which is both the right code and the right description |
