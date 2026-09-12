# Migration intake — Ember 0.9.5_Hardened_8

Status as of 2026-09-12: **target accepted for development; not yet installed
as the normative repository specification.**

This document is the evidence-backed bridge between the repository's current
normative language, `0.8.5_Hardened_1`, and the owner's proposed target,
received as `0.9.5_Hardened_3`, corrected and frozen as H4, advanced by source
recovery to H5, advanced through the mutable-helper API ruling as H6, and
clarified at the callable-parameter-mode boundary as H7, then completed at the
helper-input and `Callable` bridge boundaries as the `0.9.5_Hardened_8`
development target. It is not a second specification. Where this document and
a normative rule differ, the specification governs.

## 1. Source custody and exact inputs

The latest owner's supplied file is preserved byte-for-byte at:

    docs/spec-source/as-received/Ember_v0.9.5_Hardened_3_Implementation_Ready_Spec.md

SHA-256:

    EE046114EA1DBF10D607244BA7EF35BC14F6A508080FC67F3D83269E4E7CD9D6

It is 700,219 bytes and 7,424 lines. The preceding H2 input remains preserved
beside it with SHA-256
`6B4AE150740822361699519222A71A967910A6CD1736BA06B4A8B89F36F2D2AE`.
The current normative working source,
`docs/spec-source/ember-spec.md`, is 611,798 bytes and 5,607 lines, SHA-256:

    F8EC2010EADC6F7393E81C67C77DCB8268B8604578F419BC03EEDAD422DCC4AF

The frozen H4 predecessor is:

    docs/spec-source/Ember_v0.9.5_Hardened_4.md

It is 695,978 bytes and 7,425 lines, SHA-256:

    143E7B85805C44EA93F431CE6D05DC46DB7FA38636577B1DDA698B77D5389AB2

It retains every rule ID present in 0.8.5 and records the six-definition source
gap that was open when H4 froze.

The frozen H5 source-recovery predecessor is:

    docs/spec-source/Ember_v0.9.5_Hardened_5.md

It is 700,757 bytes and 7,493 lines, SHA-256:

    0421B0313D8768513F6F474BADE2782B8DA237DFF5D59054471CA9F8224AE1ED

H5 recovers `[LT-8]`–`[LT-13]` from the definitions supplied by the owner on
2026-09-12. Only transport-corrupted Markdown escaping and code fences were
normalized.

The frozen H6 mutable-helper predecessor is:

    docs/spec-source/Ember_v0.9.5_Hardened_6.md

It is 703,559 bytes and 7,522 lines, SHA-256:

    5AF6C2FD36908DCB7A379C06776986CE212A2B8E3EC2C80DF42A8164D3DD825D

H6 resolves ODR-005 using explicit all-mutable `with_views2_mut/3_mut/4_mut`
signatures with canonical `MutSpan[T]`.

The frozen H7 callable-mode predecessor is:

    docs/spec-source/Ember_v0.9.5_Hardened_7.md

H7 records the owner-approved callable parameter-mode ruling: callable types
admit borrowed/default, `mut`, and `owned` modes, and `_mut` helper callbacks
use explicit `mut`. It is 707,643 bytes and 7,579 lines, SHA-256:

    573BF0729BC6AEF51E872CBF773EDEAB642F91635BFA52F03797FD050D53BC8D

The current frozen development target is:

    docs/spec-source/Ember_v0.9.5_Hardened_8.md

H8 closes ODR-006 and ODR-007: mutable helper inputs are `mut` reborrows,
neither helper family consumes a view, and `Callable[Args, R]` preserves the
full mode vector as compile-time canonical type metadata. It is 713,482 bytes
and 7,644 lines, SHA-256:

    28FB191D8B8CE085AE878F55A15E5B0F0C9E364FD6BA58CD2416FA8101BA4156

H4 through H7 remain immutable under
their revision identities. Any later correction requires
`0.9.5_Hardened_9`, which then becomes the development target. Target
selection and normative repository
adoption are separate gates. `E3065` remains a specified-but-unimplemented
diagnostic.

The preserved file is evidence of what the owner supplied. It MUST NOT be
edited. The working source remains `docs/spec-source/ember-spec.md` until the
consolidation gate in §5 passes.

## 2. Intake result

The 0.9.5 target is usable, but neither supplied Markdown file is safe to copy
over the normative source verbatim. Its central language change is clear:

- `@view struct` values may carry multiple compiler-inferred region slots;
- fields retain independent provenance instead of being collapsed to one
  intersection region;
- field projections require only their own live region slots;
- whole-value and opaque operations conservatively require all relevant slots;
- region metadata and callable-access summaries are compile-time-only and
  erased before ABI/code generation;
- `[TYP-15]` remains an all-slots escape condition, and arbitrary owning
  containers of views remain forbidden by `[TYP-15a]`.

That is an owner-selected semantic target. The repository must implement it;
the target must not be narrowed to fit today's one-region compiler.

The file also contains document-integrity defects and unverified historical
claims. Those must be separated from the semantic target before the file can
become normative.

## 3. Specification findings

| ID | Class | Finding | Current treatment |
|---|---|---|---|
| SPEC-095-001 | document structure | H2 opened a historical block without closing it. | **Resolved.** H4 has one consolidated normative body with paired markers. |
| SPEC-095-002 | normative contradiction | H2 made the inherited language non-normative. H3 introduced a supersession algorithm over a body already edited in place. | **Resolved.** H4 is one flattened normative source; frozen predecessors remain separate history. |
| SPEC-095-003 | broken cross-reference | H2's boundary hid the only `[MIR-REG-1]` definition. | **Resolved.** The consolidated Part XIX definition is active and indexed. |
| SPEC-095-004 | missing normative source | H3 says `[LT-8]`–`[LT-13]` were recovered, but defines none of them. | **Resolved in H5.** ODR-004 closed when the owner supplied all six definitions. H4 remains the unchanged evidence of the original gap. |
| SPEC-095-005 | rule-ID collision | H2/H3 reused inherited `[CLI-15]` for cycle inspection and moved the historical meaning. | **Resolved.** Existing `CLI-15` retained; cycle inspection is `CLI-17`. |
| SPEC-095-006 | implementation/conformance gap | `E3065`/B14 has no repository registry entry, page, emitter, or test. | **Open implementation work.** H4's `DIA-19` defines required artifacts but claims no evidence. |
| SPEC-095-007 | tooling/document structure | New rules use valid Markdown heading definitions. | **Resolved in tooling.** `rule_index.py --spec` recognizes headings and standalone rule paragraphs, with false-definition tests. |
| SPEC-095-008 | evidence error | H2 described simulated cycles as implemented. | **Resolved by H3 and retained.** H4 distinguishes `SPECIFIED`, `IMPLEMENTED`, `VERIFIED`, and `CONFORMANT`. |
| SPEC-095-009 | stale self-identification | Embedded 0.8.5 front matter contradicted the current revision. | **Resolved.** It is condensed as historical lineage and an intake authority notice governs. |
| SPEC-095-010 | inherited known defect | `[TST-11]` retained the merge corruption covered by decided ERR-031. | **Resolved** using the already-decided source reading; no new semantics. |
| SPEC-095-011 | rule-ID collision | H3 reused inherited `RT-1`–`RT-4` for unrelated runtime-hardening rules. | **Resolved.** Existing IDs retained; additions are `RT-6`–`RT-9`. |
| SPEC-095-012 | rule-ID collision | H3 reused inherited `DIA-18` for E3065 artifact completeness. | **Resolved.** Existing `DIA-18` retained; addition is `DIA-19`. |
| SPEC-095-013 | duplicate definitions | H3's “canonical registry” restated 41 heading IDs as definition-shaped bullets and still omitted some heading rules. | **Resolved.** Direct heading extraction is authoritative; the remaining index is unbracketed and non-normative. |
| SPEC-095-014 | invalid example | An MRV example mutated `positions` declared as immutable `Span[Vec3]`. | **Resolved.** It is `MutSpan[Vec3]`. |
| SPEC-095-015 | version/evidence claim | H3 called itself mechanically audited despite the unresolved definitions and collisions. | **Resolved.** H4 is explicitly a frozen development target with normative adoption blocked. |
| SPEC-095-016 | version-selection wording | `[MOD-6a]` said `0.9.5` was added beyond the versions in `[MOD-6]`, but `[MOD-6]` already listed it. | **Resolved.** One exact supported set remains; `[MOD-6a]` only distinguishes the `0.9` and `0.9.5` contracts. |
| SPEC-095-017 | semantic/API ambiguity | Recovered `[LT-8]` gives shared-`Span` signatures, while `[LT-11]` and `[TST-16]` refer to mutable inputs. | **Resolved in H6 by owner ruling.** Separate all-mutable `_mut` helpers use `MutSpan[T]`; mixed overloads are not implied. ODR-005 closed. |
| SPEC-095-018 | semantic/callable-mode ambiguity | H6's `_mut` signatures use unmarked `MutSpan` parameters, but `[FN-1]` makes unmarked parameters shared and `fn_type` cannot encode callback parameter modes. | **Resolved across H7/H8 by owner rulings.** H7 adds callable grammar/modes; H8 makes helper inputs `mut` reborrows and forbids consuming them. ODR-006 closed. |
| SPEC-095-019 | semantic/callable-abstraction ambiguity | H7 distinguishes borrowed, `mut`, and `owned` callable parameters, but `[CLO-3]` maps callable types to `Callable[Args, R]`, whose `Args` tuple carries no mode vector. | **Resolved in H8 by owner ruling.** The existing public abstraction remains; the compiler's canonical callable type preserves the full mode vector as compile-time-only metadata. ODR-007 closed. |

The strengthened rule-index pass reports no duplicate definitions or orphaned
amendments in the current target, and LT-8 through LT-13, LT-8a, LT-11a,
FN-6a, TST-20, and TST-21 resolve as definitions. The supplied mnemonics
`TST-LT-MUT` and `TST-LT-MODE` were not valid indexed rule IDs and are
normalized to the next unused numeric TST IDs.
The existing four dangling-reference baseline remains unchanged. Missing test
directories and E3065 artifacts are expected implementation/conformance gaps,
not evidence that the specification should be weakened.

The alternate-spec report names six diagnostic codes absent from the current
compiler registry. `E3065` is the new 0.9.5 gap. `E4050`, `E4057`, `E4060`,
`E4064`, and `W4001` are inherited known-baseline gaps already present in the
current 0.8.5 contract; H4 neither creates nor conceals them.

## 4. Repository state versus the 0.9.5 target

Verified at commit `6aefe559e295812cc343443f8a8474cb9a80de1c` on `main` before
the H5 source-recovery change:

- `cargo build --workspace --locked`: green, no warnings;
- `cargo test --workspace --locked`: 178 passed, 0 failed;
- 187 Ember conformance cases in 67 rule directories pass;
- all six document/tooling gates pass;
- Appendix A matches its generated fixture;
- one implementation defect is open: D-038;
- four intentional deviations are open: D1–D4; D5 is closed and D6 withdrawn.

Current state after Gate B item 2: 178 Rust tests and 197 Ember conformance
cases across 70 rule directories pass; 58 defects are recorded and D-038 is
the only open one. `[CELL-6a]`, `[CELL-9]`, and `[CELL-10]` are covered, and
D-044/E3023 closes writes through borrowed value parameters. The frozen H8
target and adopted normative specification are unchanged by this compiler/test
block.

`cargo test` emits one Rust test-target style warning for
`from_is_contextual_so_that_From_can_declare_it`; this does not contradict the
warning-free `cargo build` result, but it should be cleaned up.

The supplementary `cargo fmt --all -- --check` is red over broad pre-existing
formatting drift in compiler sources. It is not currently one of the six gates,
was not caused by this specification intake, and was not hidden by a bulk
unrelated rewrite. Clean it in an isolated mechanical change before promoting
rustfmt to a required gate.

### Implemented or materially present

| Area | Evidence-backed state |
|---|---|
| Bootstrap | Rust workspace, diagnostics, lexer, parser, AST/HIR/MIR, analyses, C11 backend, C runtime, build/run/check/explain/fmt commands. |
| Core source language | Substantial scalar, struct, enum, tuple/array, expression/control-flow, module/visibility, interface/generic, range, formatting, and error-recovery support. Phase 1 is useful but not represented by complete per-rule conformance. |
| Ownership | Whole-local moves, `Copy`, deterministic drops, drop flags, overwrite and statement-temporary destruction, NLL loans, two-phase borrows, reborrows, disjoint fields, and one-region view propagation. |
| Views | `ref`, `ref mut`, `Span`, `MutSpan`, `str`, one-region `@view struct`, `[TYP-15]` storage checks, and static-region storage behavior. |
| Interior mutability | `Cell[T]` core surface and `RefCell[T]` borrow/try-borrow guards are compiler-known and tested. `RefCell` is move-only; guard views and `L3011` exist. |
| Diagnostics/tooling | Stable code registry, JSON/text diagnostics, borrow-shape classifier, error-page validation, rule-index ratchets, generated spec split, spec block checks, generated-C assertions including ordering, per-profile conformance runs, and ordered/forbidden diagnostic-help assertions. |

### Incomplete before 0.9.5-specific work can be called conformant

| Area | Current evidence |
|---|---|
| Per-field movedness | **Implemented by the D-042 fix.** Recursive move paths, per-path conditional flags, partial cleanup, sibling preservation, reinitialisation and `E3042` are covered by adversarial `[EXP-6]` cases. This is now available as a foundation for `[LT-38]`; field-sensitive region provenance itself is not implemented. |
| String view coercion | D-038: implicit `String` to `str` is rejected; explicit `as_str()` is sound. |
| Remaining `Cell`/`RefCell` obligations | `[CELL-6a]`, `[CELL-9]`, and `[CELL-10]` now have executable conformance evidence; E3023/B4 is live and D-044 is closed. `Cell.take`/the `Default` update arm await `Default`; `[CELL-3]`/`[CELL-8]` `!Sync` await `Send`/`Sync` and threading. `[CELL-9]`'s class-only unchecked-exclusivity interaction has no reachable trigger until that later subsystem exists. |
| Arena | `[ARN-*]` not started. Arena is a region allocator, not another interior-mutability primitive. |
| UnsafeCell | Normatively specified in 0.8.5, not implemented. |
| Phase 2 completeness | `mem.*`, `Clone`/`Default` derives, the full standard collection surface, several borrow/lifetime rules, and `[DIA-7..10]` UI snapshots remain incomplete. |
| Objects and later phases | Class/foreign-class/coroutine syntax exists in the parser, but class layout/RC/exclusivity/vtables, effects/comptime/derives, C and C++ importers, concurrency, DOD/ECS/SIMD, interpreter, LLVM backend, hot reload, deterministic execution, LSP/debugger, and the full standard library are not implemented. |
| 0.9.5 multi-region views | Not implemented. `regions.rs` stores one region per view local, `[LT-2]` currently collapses a view struct to one region, and `borrows.rs` emits old `E3064` for the independent-region case. There are no region vectors, field provenance, callable field-access summaries, summary hashing, `[VERIFY-3]`, or `[TST-17]`–`[TST-19]` evidence. |

The current implementation is therefore **mid-Phase 2**, not a 0.9 or 0.9.5
implementation. Version labels in the supplied document do not change that
fact.

## 5. Migration gates and work order

### Gate A — make 0.9.5 a sound normative input

1. Preserve each owner's file byte-for-byte — **done for H2 and H3**.
2. Resolve ODR-004 by recovering or explicitly deciding `[LT-8]`–`[LT-13]` —
   **done in H5 from owner-supplied definitions**.
3. Produce a consolidated source from the current 0.8.5 lineage: inherited
   rules remain normative; active 0.9/0.9.5 additions are inserted into their
   canonical Parts; superseded rules are replaced once; historical cycle prose
   remains clearly non-normative — **done in frozen H4 and carried forward in
   H5 without changing H4**.
4. Record every semantic addition as owner-approved and every structural
   repair by its actual class — **H4 records `HC-095-03`, H5 records
   `HC-095-04`, H6 records `HC-095-05`, H7 records `HC-095-06`, H8 records
   `HC-095-07`, this migration
   ledger classifies each correction, and ADR-023 records the hardening
   protocol; update the adopted-source ledger when the selected successor is
   installed**.
5. Recover the missing source in H5, resolve ODR-005 in H6, record callable
   modes in H7, and close the helper-input and `Callable` bridge boundaries in
   H8 — **done**. Explicitly adopt the current target:
   regenerate `docs/spec/`, add `0.9`/`0.9.5` version selection, and update
   ratchets only for gaps introduced by this explicit specification revision.
   H4 through H8 are frozen and MUST NOT be amended in place.
6. Make the rule tooling understand the consolidated rule forms and prove it
   still rejects references masquerading as definitions — **implemented and
   verified for headings and standalone paragraphs; extraction is not the
   adoption blocker**.

The normative source does not move until these checks pass together. This is
not resistance to the new semantics; it is what prevents their base language
from being accidentally discarded.

### Gate B — finish the load-bearing ownership foundation

1. ~~Fix D-042 with per-field move paths/drop flags and adversarial evidence.~~
   **Done.** The same probe found and closed D-043, the independent leak of
   owned parameters at callee exit.
2. ~~Add real coverage for `[CELL-6a]`, `[CELL-9]`, and `[CELL-10]`.~~
   **Done.** The work also exposed and closed D-044: default-mode value
   parameters had lost `[FN-1]`'s shared-borrow provenance, making their writes
   compile as changes to a private ABI copy. E3023/B4 now diagnoses all mutable
   access paths and applies `[CELL-10]`'s suggestion ordering.
3. Fix D-038.
4. Implement `Arena` and its region behavior.
5. Implement `UnsafeCell` exactly within `[UNS-10]`–`[UNS-10b]`.
6. Finish the remaining Phase 2 exit criteria, including UI snapshots.

### Gate C — implement inferred multi-region views

1. Design and ADR the internal region-vector and field-provenance
   representation; no source-level lifetime syntax and no runtime region
   object.
2. Extend HIR/MIR and `regions.rs` from one region per view value to
   field-sensitive slots while preserving the legacy one-slot case.
3. Add verified return-provenance and callable field-access summaries, with
   conservative all-fields behavior for opaque calls and interface/cache
   invalidation when summaries change.
4. Replace the old 0.9.5-invalid `E3064` rejection with the specified behavior;
   implement `E3065`/B14 for uninferrable returned provenance.
5. Implement `[VERIFY-3]` and conformance for every clause of `[LT-14]`–
   `[LT-43]`, `[TST-17]`–`[TST-19]`, plus generated-C checks proving region
   erasure.
6. Do not call the feature conformant until the acceptance condition is backed
   by repository evidence.

### Gate D — continue the existing phase plan

After Phase 2 and the multi-region foundation, continue Phase 3 onward in the
specification's dependency order: objects/RC/exclusivity; effects/comptime;
C FFI; concurrency/DOD/ECS; C++ FFI/interpreter; hot reload/determinism; then
1.0 tooling and full conformance. The 0.9.5 implementation-contract additions
strengthen the evidence required at each phase; they do not make later phases
already complete.

## 6. Development protocol

The standing procedure in `HANDOFF.md` §0.0 remains in force:

1. Read `COLD-START.md`, this migration intake, `HANDOFF.md` §0, the normative
   rule, `DECISIONS.md`, `DEFECTS.md`, `DEVIATIONS.md`, and `BACKLOG.md` before
   changing code.
2. Classify each finding before acting: language decision, specification
   ambiguity, compiler defect, test defect, or implementation limitation.
3. Probe the exact normative clause with a minimal adversarial program before
   implementation.
4. Add a conformance case that can fail; deliberately break it once and watch
   it go red.
5. Implement the semantic mechanism across every affected phase and verifier,
   not a test-specific exception.
6. Inspect generated C when ordering, layout, ownership, ABI, or compiler-made
   names matter; use `assert-c-order` for ordering rules and C11 `-pedantic`
   for portability.
7. Update the correct ledgers. A compiler defect does not edit the language;
   a missing test does not create semantics; a real contradiction stops for
   owner input.
8. Run warning-free build, all tests, all six gates, Appendix A verification,
   and any feature-specific backend/runtime checks before committing.
9. Report actual scope, evidence, remaining work, and any unverified claim.
   Wait for the owner's green signal before beginning the next bounded task.

## 7. Exact next task

**Fix D-038: implement implicit `String`→`str` coercion.**

The Cell/RefCell coverage block and D-044 are complete. D-038 must reuse the
existing `view_of`/region/verification path that made explicit `as_str()` sound
in D-037; do not add a second unverified view producer. After D-038, implement
Arena in Gate B order. Do not begin Arena or 0.9.5 region-vector work as part of
the D-038 task.

The parallel specification task is now explicit normative adoption of H8;
ODR-006 and ODR-007 are closed. Do not broaden ODR-005's all-mutable ruling
into unapproved mixed overloads, change `mut` inputs to `owned`, or turn the
compile-time callable mode vector into a public generic or runtime mechanism.
