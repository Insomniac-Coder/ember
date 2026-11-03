# The workflow that fixes the specification for good (PLAN, for the owner's approval, 2026-11-03)

The owner, 2026-11-03: "the final mile ... make it as foolproof as possible. While keeping its C-like
performance, python-ish syntax, rust-like memory safety without the annoying bits and consistency remove
all the gaps, flaws and memory issues. Make the spec near 100% complete so that any agent can pick it and
start implementing without any disruptions. Also answer all the open questions in this pass with the
best judgement ... you can use Fable 5.1 ... I'm willing to make no compromises on this workflow."

## What "done" means

1. **No hole:** no program accepted by the document breaks a promise of `[PHIL-10]` (use after free, a
   race, a double end, a leak through a loop, a skipped check, memory the rules cannot see), under
   separate compilation, hot reload and the FFI included.
2. **No guess:** an implementer reading one Part can build it without inventing a rule; every open
   question (`docs/AUDIT-0.9.20.md` O1–O14, `decisions.md` N38–N40) is ruled and written in.
3. **No contradiction:** every cross-reference claims what its rule says; every code has one defining
   rule and one shape; every example compiles or names its unbuilt syntax.
4. **The three goals kept:** C speed (nothing new runs while a program runs unless the property is
   run-time data; every cost has a row in §X.4), Python readability (a word on the thing it describes,
   the owner's test), Rust safety without the annoying bits (checked when compiling, no lifetime names).
5. **General, never for one test** (`docs/AUTOPILOT.md` §4), and nothing the owner ruled is overturned
   silently: a ruling that touches an owner verdict says so and is reported.

## Inputs every agent gets

The development target (`docs/spec-source/Ember_v0.9.20_Hardened_1.md`, or its Part from
`tasks/spec-0.9.9/parts/`), `docs/AUDIT-0.9.20.md` (what was found, fixed, left open and judged sound),
`tasks/96/redesign/JOB.md` (the owner's verdicts: constraints), `decisions.md` §§8–12 (the lead's
rulings), the unsafe-features inventory, `MILESTONES.md`, and one brief: the five points above, the
document's voice (`[DIA-2]`, `[DIA-25]`: no "you", the line to write), and the form of a finding.

## Shape: five workflows in sequence, the lead reading each result before the next

All agents are **Fable 5.1 at `max` effort** (`model: 'fable', effort: 'max'`), except dedup and bookkeeping,
which are plain code. No agent edits `tasks/spec-0.9.9/parts/`, the compiler or git: finders and verifiers
return structured results; writers return edit pairs (exact old text → new text) that the lead applies
with asserting scripts, part by part, running the spec's checks after each; the lead cuts each hardening
and commits. Every workflow's results are saved under `tasks/96/redesign/audit-0.9.20/<workflow>/` (JSON
per agent) and committed, so a context reset resumes from the files, and the Workflow tool's own resume
replays unchanged calls from its journal.

### W1. The rulings panel (the open questions)

For each open question, three **designers** write a ruling independently, each from a different angle
(speed first; safety first; the newcomer first), as rule text in the document's voice with: every rule
it touches, a program it refuses and one it accepts, what runs while the program runs and its §X.4 row,
and how an implementer builds it. Three **judges** score the three against the five points and name the
best with the ideas to graft from the others. The lead rules from the judged set and writes
`decisions.md` §13 (each ruling with its reasons; a ruling that reverses an owner verdict is marked so).

The questions, with the lead's current leaning as the panel's starting point (not its answer):

| Q | Question | The lead's leaning |
|---|---|---|
| O1 | three rules let a whole-program fact decide a package's layout or code (`[OBJ-1]`'s weak count, `[THR-2]`'s hand-over compare, `[WK-18]`'s deferred-drop region) | a per-program fact never decides a package's code: the weak count on every object of a `pub` class or of a class some `weakref` in its package names; the compare over those classes; the region only for a class whose objects can never be `thread_shared`, read from its type (O2) |
| O2 | how a link to a `thread_shared` target is written (the opening compiles two ways) | `weakref thread_shared T` is a type of its own (the grammar parses it); `weakref T` links a one-thread target; `weakref shares T` is `E0100` |
| O3 | channels and `Send` | `Sender[T]`, `Receiver[T]` are `Send`, and `Sender[T]` `ThreadShared`, exactly when `T` is `Send` |
| O4 | a hand-over out of an `owns T` block | `E3013`, help `mem.take(ref mut b)` or hand the block over |
| O5 | `jobs.local_arena()` | a `ref mut ThreadArena` whose region is the job's function; a value made in it that leaves is `E3063` |
| O6 | `[MAN-9]` writes `ember.toml` where a compile command runs (the owner's verdicts 127, 135) | keep "every program is a package"; a compile command never writes a file: the package is implicit and the note says `ember init` writes the manifest (reverses the "writes a default one" half of 135: reported) |
| O8 | which thread made a value, read from its chunk | a chunk belongs to its making thread for the process's life |
| O9 | hash flooding with the fixed-seed default | one sentence in `[STD-11]`, and a lint for a `Map` keyed from `io`/`fs`/`args` with the default hasher |
| N38 | `runtime` before a class type | yes: `fn show(a: runtime A):`, the fact read from the virtual slot's entry as for `dyn` |
| N39 | a local whose kind is chosen while running | `runtime` is written in any declared type before `dyn` or a class type (`q: runtime owns dyn Picker = …`), not only a parameter's |
| N40 | `owns` at a call binding as `-` | a value word written first in an argument, or first on the right of `=`, covers the whole expression there (`m.insert(owns i * 7, owns i)`, `b = owns n * 2`); elsewhere it binds as `-` |

### W2. Break it (adversarial finding, loop until dry)

Three partitions of finders, each blind to the others, each returning findings in one schema
(`{rule, kind: SEC|MEM|GAP|CON|ED, severity, program, why_the_text_allows_it, proposed_rule_text,
milestone}`):

* **Part readers** (one per Part, 24): every rule of their Part under the four lenses, writing the
  breaking program for each hole and the question an implementer could not answer for each gap.
* **Lens sweepers over the whole document** (8, the lenses of `docs/AUDIT-0.9.20.md` §5): separate
  compilation; the runtime's own races (counts, the queue, the trampoline, first-access statics, reload's
  commit); FFI contracts versus what C may do within them; what an attacker controls (dependencies'
  overlays and compile-time code, exports, the lock file, untrusted keys); the summary and call checks;
  every word's write site against every read site; the inventory C01–C34 against the text; codes,
  shapes, cross-references and examples.
* **Implementers** (one per Part, 24): write the implementation plan of their Part from the text alone
  and return every point where they had to guess, with the sentence that would have removed the guess.

Then: dedup in code (rule + kind + the program's first line); each finding to **three refuters** with
distinct lenses (does the text really allow the program; does it really break a promise or leave a
guess; does another rule already answer it), refuting by default when unsure; a finding survives with
two confirmations; a surviving GAP gets a fourth agent that drafts the missing rule. Rounds repeat with
the confirmed list as "already found" until two rounds find nothing new (the loop-until-dry pattern).
Expected: 80–150 findings in round one, most of them GAPs of the implementer kind.

### W3. Write the fixes

For each Part with confirmed findings, one **writer** drafts every change of that Part as edit pairs in
the document's voice, with the `*(changed in 0.9.20)*` marker, the codes it needs (reusing an existing
code where the shape fits; a new code with its registry row and shape), and the §H.52 bullet; two
**reviewers** per Part check the draft against the whole document (contradictions, voice, `[DIA-25]`
messages that compile, the three goals, generality). The lead applies the pairs with asserting scripts,
runs the spec's checks, cuts the next hardening, commits, and feeds the writers' questions back to W1's
panel where a draft needs a ruling.

### W4. Completeness and consistency

After each cut: the 24 implementers again on the new text (zero guesses is the exit); a **critic**
asking what the round did not do (a lens not run, a Part not re-read, a claim not verified); the
mechanical gates (`check_ids`, `check_citations`, `code_registry`, `rule_index`, `spec_check`,
`check_examples`, `phase_progress`); and the compiler's spec-tied tests. W2–W4 repeat until W2 is dry
and W4 reports no guess, two rounds in a row.

### W5. The final read

Three whole-document readers with different vantages (a compiler implementer starting from nothing; a
security auditor of the FFI and the build; a Python programmer judging readability), and one last
attacker with the final text, each with the five points; anything they find goes once more through W3.
Then the last cut, the handoffs, the audit file brought up to date, the push.

## Size and cost

| Workflow | Agents (about) | Tokens (about) |
|---|---|---|
| W1 | 11 questions × 6, plus synthesis | 70; 10M |
| W2 | 56 finders + 3 refuters per finding, 2–3 rounds | 500–800; 50–70M |
| W3 | writers and reviewers per changed Part, 2–3 rounds | 80–120; 20M |
| W4 | 24 implementers + critic per round, 2–3 rounds | 60–80; 12M |
| W5 | 4 | 4; 2M |
| Total | 700–1,100 agent runs | 90–120M tokens, about half a day at 16 agents at a time |

Each workflow stays under its own 1,000-agent cap; the cloud container runs up to 16 at once. The owner
may cap the whole with a token budget (`+Nk`), which the scripts honour by reducing refuters first.

## Safeguards

One brief for every agent; structured output enforced by schema (a malformed result is retried once, then
dropped and logged); no agent edits the specification, the compiler, the tests or git; the lead applies
every edit with a script that asserts what it replaces, and runs the spec's checks before each cut; an
owner verdict (`JOB.md`) is a constraint to every agent, and a ruling that touches one is marked and
reported; no power or thermal rules apply in the cloud; results are committed after each workflow so a
context reset resumes from the files.

## What the owner decides

1. Go, with this shape; or a smaller first round (W1 and one round of W2) to see the findings first.
2. A token cap, or none.
3. Whether O6's reversal of verdict 135's written manifest is acceptable, or the written file stays.
