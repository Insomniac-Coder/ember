# Associated-type consistency review — 2026-10-09

Two read-only reviews examined `phase-next` at `08a63db` for gaps related to
D-532. One covered capability predicates and operators; the other covered
projection resolution, inference and bound substitution. Neither reviewer ran
the compiler, modified source, or launched further agents.

**Historical probe record. The [October 10 follow-up](../2026-10-10/associated-types.md)
records contract triage, fixes and regression coverage.** After the owner suspended the temperature condition on
October 9, all eight compile-only probes ran in under four seconds. Sources
are retained in [probes/](probes/), with passing setup controls in
[controls/](controls/). No type-checker change is included in the map-performance
batch. These small checks do not establish how every concrete instantiation
behaves or whether each capability issue is specific to projections.

| Probe | Observed result | Passing control |
|---|---|---|
| `lookalike_operator` | Accepted `+` with only the unrelated `Lookalike` bound. | A direct parameter with the actual `Add[Output = T]` bound. |
| `unbounded_comparison` | Accepted `<` with an unconstrained associated type. | Add the `Ord` bound. |
| `float_projection_print` | E0900: printing `H.Item` is not implemented. | Add an explicit `Display` bound alongside `Float`. |
| `result_projection` | E2020: expected `Result[S.Item, i64]`, found `Result[Self.Item, i64]`. | Use a concrete `int` result in both signatures. |
| `method_bound_projection` | E2040: `C` does not implement `Build[Self.Item]`. | Use `Build[int]` in the interface method's bound. |
| `generic_assoc_bound` | E2040: `Value` does not implement `Tag[T]`. | Use `Tag[int]` in the associated-type constraint. |
| `assoc_binding_enforcement` | Accepted the implementation whose associated type binds `Item = str`, despite the `Item = int` requirement. | Bind the implementation's `Item` to `int`. |
| `nested_projection_binding` | E2020: expected `Option[O.Atom]`, found `Option[Self.Atom]`. | Use concrete `int` in the nested binding and result. |

Two initial controls needed repair: `Self.Item` in an `Add` binding did not
parse, and leaving `S.Item` in the supposedly concrete Result control still
exercised projection resolution. Neither failed control is evidence for a
separate defect; the corrected controls above pass.

| Candidate | Source path through `compiler/ember_typeck/src/lib.rs` | Probe | Expected contract |
|---|---|---|---|
| An unrelated interface with `add` may satisfy generic `+` | `synth_bound_operator` (23634), `has_operator_method` (40299), compared with nominal `operator_implemented` (40315) | `lookalike_operator.em` | Reject `H.Item: Lookalike` using `+` when it does not implement the actual `Add` interface. D-315's concrete-type rule should also govern generic bodies. |
| Unconstrained projections may compare without `Eq`/`Ord` | `c_comparable` (36972), comparison checks around 41261–41487 | `unbounded_comparison.em` | Reject `a < b` in a generic body where `H.Item` has no `Ord` bound. |
| A `Float` bound may satisfy printing's interface test but not its formatting path | `formattable_in` (10150), compared with `implements` (29938) | `float_projection_print.em` | Accept printing a Float-bound projection if Float supplies the Display/Debug capabilities claimed by `implements`. This may affect direct parameters too. |
| Associated-type resolution may stop inside `Result`, enums/classes and builtin wrappers | `resolve_assoc` (29858), compared with `substitute_ty` | `result_projection.em` | Resolve `Source.get() -> Result[Item, int]` to `Result[S.Item, int]` for an opaque `S: Source`. Probe the other container shapes separately if this reproduces. |
| A bound method's own positional interface arguments may retain `Self.Item` | `bound_call_signature` (34240), compared with `bound_resolved_for` | `method_bound_projection.em` | A call through `S: Source[Item = int]` should accept `C: Build[int]` where the method requires `C: Build[Item]`. |
| Instantiating a generic interface may leave associated-type constraints unsubstituted | `instantiate_interface` (8285), copied `assoc`/`assoc_bindings` | `generic_assoc_bound.em` | Instantiating `Outer[int]` should change `type Member: Tag[T]` to `Tag[int]`. |
| Concrete implementations may omit equality requirements on associated-type bounds | `check_implementation_of` (7220), `assoc_bindings` consumers | `assoc_binding_enforcement.em` | Reject `Holder.Value = Wrong` when the bound is `Element[Item = int]` but `Wrong.Item = str`. |
| Projection bindings may fail to substitute associated names nested inside containers | `assoc_names_in`, `replace_assoc` (6830) | `nested_projection_binding.em` | Resolve `Inner[Item = Option[Atom]]` to `Option[O.Atom]` when projecting through `O.Wrapped`. |

The first three are generic/concrete capability consistency questions and are
not necessarily specific to projections. Ordinary `T.Name` generally becomes
a hidden `TyKind::Param`; therefore a `Param` match without an `Assoc` arm is
not by itself a defect.

An additional hypothesis remains too weak to report as a finding: some predicates
index `current_generics` directly while `generic_bounds(index, name)` can recover
an imported hidden projection by name. No concrete producer-to-predicate example
was established.

Inspected without another actionable finding: D-532's inherited-bound closure,
ordinary/lazy projection creation, name-based recovery of displaced parameters,
direct bare projection argument/result substitution, direct associated method
lookup, declaring-bound selection, and ordinary callable-bound inference.
Field lookup was only partially inspected.

Next work should trace each reproduced outcome against the governing rule,
exercise concrete instantiations and direct-parameter equivalents, then add a
conformance regression before fixing it. No broad conformance run is needed
merely to triage these small probes.
