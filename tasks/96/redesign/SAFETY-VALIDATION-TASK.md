# Ember compiler soundness and memory-safety validation (the owner's task, 2026-10-25)

The owner's text, kept as given. Owner's choice: **both, (b) first** (see `PLAN-run4.md`, "Safety
validation"): (b) the invariant matrix and tests for the new model on paper; then (a) this pipeline on
today's compiler.

---

Task: Ember Compiler Soundness and Memory-Safety Validation

Context and authorization
I am developing Ember, an experimental statically typed programming language, and I am testing the
correctness of its own reference compiler and runtime. This work is authorized and limited to my
repository, locally generated test programs, test harnesses, and isolated test environments.
The objective is to find implementation defects that violate Ember's specified safety guarantees,
especially cases where the compiler accepts an invalid program or generates incorrect or unsafe C code
from a valid safe program.
This is defensive compiler testing. Do not test external systems, target third-party software, or
perform network-based exploitation.

Phase 1: Establish the baseline
Inspect the current repository before proposing changes. Read:
* The README and pinned development specification.
* The normative rules covering ownership, borrowing, regions, classes, reference counting, destruction,
  FFI and unsafe operations.
* The conformance tests and their existing positive and negative test conventions.
* `docs/phase-status.json` and `docs/DEFECTS.md`.
* The borrow/region analysis implementation, C code generator and relevant runtime code.
Record the exact commit, specification revision, supported platforms, test commands and compiler
configurations. Distinguish implemented features from features that must currently be rejected. Do not
infer correctness merely from passing test counts or completion percentages.

Phase 2: Construct a safety-invariant matrix
For each applicable specification rule, document:
1. The required language guarantee.
2. The compiler or runtime component responsible for enforcing it.
3. Existing tests and missing edge cases.
4. A proposed positive test, negative test, or runtime test.
5. The expected result and how that result will be verified.
Prioritize ownership and lifetime safety, mutable aliasing, region escape analysis, container-held
views, object destruction, reference counting, class-field exclusivity, and FFI ownership contracts.

Phase 3: Build a bounded testing harness
Design separate test targets for:
* Lexer and parser fuzzing, including malformed and unusual inputs.
* Structured source generation for type checking and borrow/region analysis.
* Negative conformance tests for programs that should be rejected.
* Execution of valid programs to test runtime semantics and generated C.
* Local FFI test harnesses using small, controlled C or C++ stub libraries where the relevant
  interoperation feature is implemented.
* Regression testing across supported compilers and platforms.
Use grammar-aware generation where possible, plus mutations of existing conformance tests. Run the
generated tests in disposable temporary directories with CPU, memory and execution-time limits. Do not
expose host secrets or require network access.

Phase 4: Validate safety properties
Use suitable compiler instrumentation and sanitizers, such as AddressSanitizer and
UndefinedBehaviorSanitizer, where supported. Check program output and lifecycle invariants in addition
to looking for crashes.
Distinguish:
* Expected rejection of invalid code.
* Defined runtime errors or panics.
* Compiler crashes or hangs.
* Incorrect program results.
* Potential memory-safety or undefined-behavior defects.
A sanitizer-clean run is useful evidence, not proof that a program is memory-safe.

Phase 5: Reproduce and fix findings
For each finding:
* Save the original input and minimize it to a small reproducible test.
* Record the exact compiler commit, specification revision, flags, platform, expected behavior,
  observed behavior and diagnostic or sanitizer output.
* Identify the governing language rule and the responsible implementation component.
* Add a permanent regression test before considering the fix complete.
* Verify that the fix does not regress neighboring valid programs.

Required deliverables
First provide a prioritized implementation plan, the invariant matrix, the proposed directory structure,
test commands, resource limits and CI stages. Identify which parts can reuse existing test
infrastructure.
Then implement the workflow incrementally, beginning with a small high-value subset around ownership,
regions and generated-C safety. Keep changes scoped, preserve existing conventions, and report what was
actually tested separately from what is only proposed.
The final outcome should be a repeatable compiler-safety validation pipeline that can run locally and in
CI, preserve minimized regressions, and produce actionable findings—not merely a large collection of
random test cases.
