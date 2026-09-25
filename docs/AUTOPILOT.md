# Working on Ember unattended

The owner's rules for any agent session on this repository, and especially for a cloud
session that works through the night while the owner sleeps. Everything goes to `main`.
Written 2026-09-25 from the owner's standing instructions and corrections.

## 1. Start here

1. Read this file.
2. Read `docs/HANDOFF.md` §0.355, the subsection "Start here after a context reset": the state,
   the next task, the backlog, and the recipes (build, test, gates, CI, cutting a hardening).
3. The development target is the file named in `docs/spec-source/development-target.json`
   (Hardened_15 at the time of writing). Its working sources are `tasks/spec-0.9.9/parts/`.

**The handoff is the memory.** A cloud session cannot see the owner's local notes. Anything the
next session needs to know goes into `docs/HANDOFF.md`, in the start-here subsection.

## 2. What to work on

Implement Ember 0.9.9 in the compiler until it is done, or until the owner says stop. Do not wait
for a go-ahead between tasks while the owner is asleep; keep going. The owner's order
(2026-09-25):

**A. Finish `std.math`, where it was left on 2026-09-25** (`std/src/math.em`, ODR-038):

1. D-272, so `i128` and `u128` reach C; then add both to `Number` in `std/src/math.em`.
2. `[STD-20]`: the float constants (`f64.INF`, `NAN`, `EPSILON`, `MIN`, `MAX`: an associated
   constant on a scalar type, not built yet) and the integer methods (`abs`, `pow`, `signum`,
   `div_trunc`, `rem_trunc`, `checked_*`, `wrapping_*`, `saturating_*`, `overflowing_*`,
   `count_ones`, `leading_zeros`, `trailing_zeros`, `is_power_of_two`, `next_power_of_two`, `MIN`,
   `MAX`); `math.fma` (`[STD-3]`).
3. The operator interfaces of Part IV §8 (`Add[Rhs = Self]` with `type Output`, likewise `Sub`,
   `Mul`, `Div`, `FloorDiv`, `Rem`, `Pow`, `Neg`, `Not`, the bit operators and the `…Assign`
   forms), with a bound's associated-type binding (`Add[Output = T]`) and every number type
   meeting them. `[TYP-17]`'s own `sum[T: Add[Output = T] + Default]` example must run.
4. The rest of `[STD-21]`: `Vec2/3/4`, `IVec2/3/4`, `UVec2/3/4`, `Mat2/3/4`, `Quat`, `Transform`,
   `Aabb`, `Sphere`, `Ray`, `Plane`, `Frustum`, with `dot`, `cross`, `length`, `normalize`,
   `normalize_or_zero` and the operators; `KahanSum`; `std.math.det` (`[DET-4]`: the same bits on
   every target, so its own implementations, not the platform's); `NonZero[T]` (`[STD-4]`,
   needs the `Option` niche).

**B. Then back to the language, where it stood before the maths:**

1. D-284 (a generic body is region-checked again per instance), `for k in owned m` (owned
   `Map`/`Set` iteration), D-305 (a user type named like a prelude type loses to it).
2. The other open defects in `docs/DEFECTS.md`: D-270, D-273, D-218 (needs `[EXC-18]`), D-220,
   D-202 (needs per-field access words, M2), D-201 (`String` and `Array[u8]` are one type), D-198.
3. The coroutine transform, generator expressions and adapters with `[CTL-3b]` fusion.
4. Owned callables: `DEVIATIONS.md` D6 (`once fn` parameter types) and `[CLO-3]` owned callable
   values.
5. Declare `Display`, `Debug` and `Copy` in std (needs `Formatter`, `FmtError`, user-written
   `Display`); the table answers them today.
6. The remaining "not yet probed" rows in `docs/AUDIT-0.9.9.md`.
7. Then the phases themselves, lowest completion first where one blocks nothing else: Phase 3
   (objects, M2), Phase 4 (effects, comptime, derives), Phase 5 (C FFI, starting with calling a C
   function from Ember, `[FFI-10]`), Phase 6, Phase 7a. The handoff's phase table and the 0.9.8
   plan's exit criteria (Part XXI of `Ember_v0.9.8_Hardened_3.md`) say what each still needs.

The handoff's start-here subsection keeps this list current: tick items off there as they land.

## 3. Deciding things while the owner is away

- **Spec ambiguities are yours to rule** (the owner delegated this). Record an ODR in
  `docs/OWNER-QUEUE.md` (the question, the options with their costs, the ruling), edit
  `tasks/spec-0.9.9/parts/`, cut the next `Hardened_N` with the handoff's recipe, and re-pin
  `development-target.json`.
- **Build the better implementation, never the easier one.** Price both shapes for the record,
  then build the correct one. Do not hand the owner a choice between an inferior option and
  nothing; if analysis finds the correct design mid-task, build and test it.
- **Never overturn what the owner ruled himself** (ODR-038 is his) or a design he chose. If one
  looks wrong, write a new ODR marked OPEN with the question, and move on to other work.
- **When something truly needs the owner** (reversing his ruling, anything outside 0.9.9,
  anything touching RageV), do not block: record it as an OPEN ODR and continue with the next task.
- **No subagents or workflows while unattended.** The owner's words: "no workflows, you go solo".
  `docs/AGENT-WORKFLOW.md` holds his rules for multi-agent work, which apply only when he is
  present and has approved the plan.

## 4. Rules of this codebase

- **Never edit the spec to fit the compiler**, and never edit `docs/spec-source/as-received/`.
  The spec changes only through an ODR and a new hardening.
- **An implementation gap is not a spec defect.** Before recording a contradiction in the
  document, quote both halves; three of four such "contradictions" were misreadings.
- **Every fix has a test that fails without it.** Break-test it: remove the fix, see the test fail.
- **A defect fix updates up to four documents:** its row in `docs/DEFECTS.md` (what, which rule,
  status **fixed** or **open**, and how the fix was verified), the reasoning in
  `docs/HANDOFF.md`, an ADR in `docs/DECISIONS.md` if the fix took a decision the spec does not
  force, and the spec only through an ODR where its own wording was wrong.
- **A reserved word that blocks a standard-library method name becomes a contextual keyword.**
  Never rename the method; never require `r#` (ODR-030 did this for `extend`).
- **A standard-library file must make sense on its own.** Which types implement an interface is
  written in the file (`extend f32 implements Float`), never hard-coded in the compiler by the
  interface's name, and the file's header says in plain words how it fits together. The owner
  rejected a `std.math` whose meaning lived only in the compiler (ODR-037, then ODR-038).
- **`std.math` works for any number type** (ODR-038, the owner's ruling): each function is
  `fn f[T: Number](x: T) -> T.Real`, an integer's answer is an `f64`, an `f32`'s an `f32`. New
  maths follows that model and the spec (vectors, matrices, `KahanSum`, `std.math.det`).
- **Tests never print the last digits of a platform maths function.** `cbrt(27.0)` is `3.0` with
  MSVC and `3.0000000000000004` with glibc (`[DET-2]`); that broke CI once. Use exact values.
- **Never touch RageV** (`Code/RageV`), even where the spec describes plans for it.

## 5. Build, test, commit, push

- **Quick check** (about 30 s): the `annotations.py` command in the handoff's recipes.
  **Full suite:** `cargo test --workspace --no-fail-fast`. **Gates:** the nine commands in the
  handoff (`hardening_check`, `split_spec --check …`, `rule_index`, `check_branding`,
  `generate_runtime --check`, `test_runtime_generation`, `spec_check`, `error_pages`,
  `unicode_case --check`), plus `spec_check.py --emit-appendix` leaving `ember-spec.md` unchanged.
  On Linux use `EMBER_CC=clang` or `gcc`; CI runs Ubuntu gcc and clang and Windows MSVC and
  clang-cl. Use `python3` if `python` is missing.
- **Never edit repository files while the full suite runs**: it reads the runtime's C and the
  test directories during the run.
- **Commit and push to `main` about every five features**, and always before stopping, only after
  the quick check, the full suite and every gate pass. End each commit message with
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- **Then watch CI** with the handoff's `curl` recipe: one watcher, polling every 120 s (the
  unauthenticated GitHub API allows 60 requests an hour). If a job fails, read its annotation
  (recipe in the handoff), fix the cause and push again. Never end a session with `main` red.
- **Never rewrite history**: no force-push, no reset of pushed commits, no deleted branches.
- **Edit with scripts that check what they replace.** Python files whose replacements assert the
  old text occurs exactly once; shell heredocs mangle backslashes.
- **Windows only** (if run on the owner's PC): after any toolchain, runtime or C-runtime change,
  run ONE panicking program first and make sure no window opens; run batches under a window
  watchdog; never launch system or Visual Studio batch files with an empty environment.

## 6. When to stop

- **A classifier or permission denial ends that piece of work.** Do not retry it or route
  around it. Commit and push what passes, write exactly what was blocked into the handoff, and
  carry on only with work that does not need the blocked action.
- **Before the session ends**, update the handoff's start-here subsection: the commits made, the
  state, the next task, every decision taken on the owner's behalf (with its ODR number),
  anything blocked or left failing, and the phase table.

## 7. Reporting to the owner

- **Plain words.** Answer the exact question first, in a sentence or two, then stop. No term
  goes in unexplained, however standard it is.
- **Say each defect's status plainly**: fixed or open.
- **Phase status is a table first**: phase, name, one bold percentage, a text bar, and an overall
  row weighted by the length of each phase's rules (`tasks/impl-0.9.9/rule_sizes.py`).
- **Explaining code**: answer the literal question in one plain sentence. If the owner cannot
  follow a file, treat it as a design problem, not a reason to explain at greater length.
- **Options are shown as real code, one example per option**, never picked for him while he is
  there to choose.
