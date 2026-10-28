# Run 3 of the memory-model rework: the plan (owner, 2026-10-24/25)

**NOT started.** Owner, 2026-10-24: "all of a phase at once is okay in my opinion, no pauses and please
dont start today we dont have credits". Owner, 2026-10-25, on the safeguards list: "I like all of your
suggestions"; and every agent must know the goal, "they just have different duties". Start only when
the owner says so. **Size approved 2026-10-25: 27 agents worst case** ("approved, the limit resets at
1:30 PM IST").

## How it runs

- One workflow, phases back to back. **No pauses for approval.** The script stops on its own only when
  an automatic check fails (owner allowed the safeguards).
- **All agents of a phase run at once.** At most 5 at once.
- Every agent hands back a filled-in form (fixed fields) plus its own report file; the script checks
  the forms. No agent writes a file another agent writes. A dead or empty agent is retried once; if it
  fails again the run stops and says so.
- The script is frozen at launch. Everything that might change lives in `run3/inputs/`, so a restart
  reuses every finished agent.
- I never message a running agent and never stop a phase on the owner's behalf.

## Before launch (the lead, no agents)

1. Power watcher started (`power_watch.ps1`, copied into the repo's `target/redesign3/tools/`); mains
   confirmed; keep-awake on for the run.
2. `ember.exe` copied to `target/redesign3/bin/ember.exe`; every agent uses that copy.
3. Inputs frozen into `tasks/96/redesign/run3/inputs/`: `JOB.md`, this plan, the shared brief. The spec's
   sha256 checked against `development-target.json`; if the spec changed, the slice lines rechecked.
4. Pop-up check: a deliberately crashing C program run through `harness.h` shows no error box.
5. Must-cover lists for the readers: the spec's keyword list (II.4: 47 reserved and the contextual
   ones), every `@attribute` in the spec, the library words that act as language words (`Box`,
   `Shared`, `Weak`, `RefCell`, `Cell`, `ref`, `ref mut`, `Span`, `MutSpan`, `str`, `String`,
   `.clone()`, `mem.take`, `mem.replace`, `mem.swap`).
6. Smoke test (1 agent, separate small run): one reader on the smallest slice at low effort, to prove
   the instructions and the form work. Its output is thrown away.

## The shared brief (every agent reads it first, word for word)

**The goal, the same for every agent; you only have a different duty.** Rework Ember's memory model so
that a program written in Ember is, all at once:
1. as fast as C and Rust doing the same work;
2. as memory-safe as Rust or safer, checked when the program is compiled wherever possible;
3. readable like Python: easy to read and to write. That means readability, not a copy of Python. What
   the programmer means stays written in plain words on the thing it describes; the compiler never
   decides what the code does not say.

Rust shows that 1 and 2 can be had together; what makes Rust annoying is how it makes you write things,
not its safety. For every annoying part the answer is one of: a readable spelling of Rust's guarantee;
a smarter compile-time solution; or a new mechanism that is clever, readable, as fast as C and Rust and
as safe or safer. An understandable limitation stays, written clearly, with an error message that says
what to write instead. Safety is never traded for looking like Python. Every answer starts from what the
program is meant to do. Before anything is called a failure, try to save it.

Then, in every brief: the owner's test (for every name, the word in the line that says what it is and
whether it may change it), the error-message rule, the agreed table (binding, never changed), the
vocabulary (from phase 2 on), the whole team (every agent below, one line each, so each agent knows
who feeds it and who checks it), the machine limits, and then **your duty**.

Machine limits: no compiler changes, no git writes (no add, commit, reset, checkout); `ember check`
with `target/redesign3/bin/ember.exe` only, never running an Ember program directly; programs run only
through `harness.h` or `run.ps1` (no `cmd start`, no windows); build only in `target/redesign3/`; write
only your own files; every claim about today's Ember carries the real `ember check` output; every number
names its source (a log line, or a file and line); if the laptop goes onto battery, stop.

## Phase 1: Vocabulary (6 agents)

| # | Agent | Duty | Model, effort |
|---|---|---|---|
| 1 | Reader, Parts I-IV, lines 1-1491 | judge every word in its slice (below) | Opus 5.5, high |
| 2 | Reader, Parts V-IX, lines 1491-3177 | same | Opus 5.5, high |
| 3 | Reader, Parts X-XVI, lines 3177-5015 | same | Opus 5.5, high |
| 4 | Reader, Parts XVII-XVIII, Annexes A-D, Appendices E-F, lines 5015-6930 | same, plus every error message that does not say what to write instead | Opus 5.5, high |
| 5 | Builder | one vocabulary table (columns: word, "today it means", "it will mean", where it is written, which word it replaces, what stays refused); keep is the default, a rename needs a failed test and a reason a programmer would care about; plus the full list of annoyances, each given to exactly one design area | Opus 5.5, xhigh |
| 6 | Checker | every word through the owner's tests; fixes or rejects | Opus 5.5, high |

Readers judge, for each word: meaning at each use with line numbers; same meaning everywhere and for
every type; plain English for what it does; sits on the thing it describes; keep / rename / merge /
remove with the reason. Not read: Appendices G-I (history and index; they define no words).

**Automatic check:** all six returned; every word on the must-cover lists judged; every agreed row in
the vocabulary unchanged; every annoyance given to exactly one area.

## Phase 2: Design (7 agents)

| # | Agent | Duty | Model, effort |
|---|---|---|---|
| 7 | Word planner | before any design, proposes the new words all five areas will need, in one place, so the areas start from one vocabulary | Opus 5.5, xhigh |
| 8 | Area: lists and loops, temporary values (keeps 5, "delete after the loop") | its annoyances, answered | Opus 5.5, high |
| 9 | Area: references kept or returned, locks, objects still being built (keeps 3, 4, 7) | same | Opus 5.5, high |
| 10 | Area: ownership and sharing, `owned` replacing `Box`, copies, hand-over, state machines (keeps 8, 9) | same | Opus 5.5, high |
| 11 | Area: linked objects, callbacks, interface types (keeps 6) | same | Opus 5.5, high |
| 12 | Area: threads, locks across threads, `unsafe` | same | Opus 5.5, high |
| 13 | Old-code re-checker | re-runs `ember check` on every OLD snippet and compares with the pasted output | Sonnet 5.5, low |

Every answer (the form): intent in one line; OLD code with its real `ember check` output; NEW code;
for every name in NEW code the word that says what it is and the word that says whether it may change
it (an empty field fails the answer); "this means exactly this": NEW translated into today's Ember that
compiles, or into C; what is still refused, with its error message; cost; which kind of answer. A new
word goes in the "new words" table with its reason, never silently into code. Areas start from last
run's report for their area (`phase3b-<area>.md`) and repair it.

**Automatic check:** all returned; every assigned annoyance answered; every form complete; the
re-checker's mismatches are handed to the Combiner to correct.

## Phase 3: Combine and check (7 agents, 9 at worst)

| # | Agent | Duty | Model, effort |
|---|---|---|---|
| 14 | Combiner | one model and one vocabulary; rules on every new word; where two areas solved one thing differently, picks one and rewrites the other; corrects the re-checker's mismatches; lists every clash | Opus 5.5, xhigh |
| 15 | Vocabulary reviewer | does the combined vocabulary read right as a whole (word position, as `fn() with copied score:` vs `copied fn():`), one meaning everywhere; checks every use of every word in the spec against phase 1's line lists; scans every NEW block for words not in the vocabulary; fixes words and the code using them, listing each change | Opus 5.5, xhigh |
| 16 | Safety attacker, one thread | makes the model accept a double free or a read of freed memory; each proven as the C the compiler would emit, run under AddressSanitizer through the harness | Opus 5.5, xhigh |
| 17 | Safety attacker, threads and shared objects | makes the model accept a data race or a shared object freed while in use; same proof | Opus 5.5, xhigh |
| 18 | Readability attacker | every NEW block against the owner's tests, the error-message rule and the agreed table; compares each NEW block's meaning with its translation and the intent line | Opus 5.5, high |
| 19 | Fixer | fixes or saves every problem from 16-18; each fix passes the test that caught it; may not add or change words (flags them for the owner) | Opus 5.5, xhigh |
| 20 | Fix re-attacker | attacks only the fixes | Opus 5.5, xhigh |
| (21, 22) | Fixer and re-attacker, round 2 | only if round 1 found something; after round 2, anything open is listed for the owner and the run goes on | Opus 5.5, xhigh |

Order: 14, then 15, then 16-18 together, then 19, 20 (and 21, 22).

**Automatic checks:** after 14, every new word has a ruling and the agreed rows are unchanged.

## Phase 4: Measure (3 agents, 4 at worst; nothing else running)

| # | Agent | Duty | Model, effort |
|---|---|---|---|
| 23 | C writer | the C the final model compiles W1-W4 into (ECS, tree with parent links, game objects referring to each other, callbacks removing themselves), with a list of every run-time check the model keeps and the line in the C where each one is | Opus 5.5, high |
| 24 | Faithfulness reviewer | confirms that list against the combined model; a missing or extra check sends the C back once (25) | Opus 5.5, high |
| 26 | Timer | plain C, the model and safe Rust with `target/redesign2/bench/run.ps1` (P-cores, mains each round, no windows); medians of 5; memory too; runs that disagree by more than 10% are re-run (at most twice); every checksum must equal plain C's | Sonnet 5.5, medium |

Please do not use the laptop heavily during phase 4.

## Phase 5: Write-up (the lead)

Old code, new code, what is refused, cost; every number with its source.

## After the run

Stray processes checked; build outputs deleted (reports and source kept); `run3/` reports listed.

## Size

| | Agents | Worst case |
|---|---|---|
| Smoke test | 1 | 1 |
| Phase 1 | 6 | 6 |
| Phase 2 | 7 | 7 |
| Phase 3 | 7 | 9 |
| Phase 4 | 3 | 4 |
| **Total** | **24** | **27** |

Outputs: `tasks/96/redesign/run3/` (one report per agent, named by its number and duty);
code in `target/redesign3/`.
