# Run 4, step 2: bring the design up to date with verdicts 21 to 34 (PLAN, for the owner's approval)

## Goal

Rewrite `run4/v2/` as `run4/v3/` so the rules, the word list and all 84 answers follow every owner
verdict, 1 to 34 (`JOB.md`, "Owner verdicts on run 4's open questions"). `v2` was made from verdicts 1 to
20 only; 21 to 34 came after it. Verdict 25 (`cast when`) is replaced by 27 and is not applied.

| verdict | what changes | where it lands most |
|---|---|---|
| 21 | library functions that run a given function: `fn for_each[C](f: fn(T) changes C) changes C:` | rules R12, areas A, D |
| 22 | a function that changes what it captured says so (`fn() changes score:`); lists say `captured` / `captured...` | rules R12, agreed row 6, areas D, E |
| 23 | the first owner keeps its rights after `shares mut` (v2's reading confirmed) | rules R3, area C (check only) |
| 24 | arenas: the field names the kind (`using LevelArena`), the making line the arena (`using current`) | rules R3, vocabulary, area C |
| 26 | `holds` becomes `exists as` | everywhere (phase 0 script) |
| 27 | "not sure" downcast `if s is Circle as c:`; `cast s as Circle` stays; `cast ... if/when` gone | rules, vocabulary, areas B, C, D |
| 28 | a walk whose visit function may delete nodes holds a `weakref`; cost accepted, optimise later | area D (AN-62), rules R11 |
| 29 | a change under a lock is named in function types (`changes Log`), `Mutex`/`RwLock`/`Atomic` too | rules R12, areas B, E |
| 30 | `changes` = whatever a function causes, whenever: `spawn`/`scope.spawn` and registering a kept callback count | rules R5, R12, R15, vocabulary, areas A, D, E |
| 31 | every C call counts as possibly running kept callbacks unless `@ffi(no_callbacks)` | rules R12, R15, new message, areas D, E |
| 32 | a parameter's rights come from the word in front of its name, for every type | rules R4, areas C, D, E |
| 33 | `.copied()` becomes `.copy_each()` | area C, vocabulary (phase 0 script) |
| 34 | a `MutSpan` name or parameter says `ref mut`; R5's exception by type withdrawn | rules R4, R5, areas A, B, E |

## Phases

**Phase 0, the lead, no agents.** Copy `v2` to `v3`. A script makes the purely mechanical spellings in
code only (fenced blocks and inline code; English "holds" in prose is left alone): `holds` to `exists
as`, `.copied()` to `.copy_each()`, and `if cast X if T holds c:` to `if X is T as c:`. Every replaced line
is listed in `v3/spelling-changes.txt` for the agents to check. Inputs are frozen in `v3/inputs/`
(brief, duties, `JOB.md` with the 34 verdicts).

**Phase 1, one agent.** The rule writer brings `v3/rules.md`, `v3/vocabulary.md`, `v3/changes.md` up to
date with verdicts 21 to 34: new and changed rules, error codes and messages (each message names what was
refused in the program's names, says why in one line, and gives the exact line to write).

**Phase 2, five agents, after phase 1.** One per area, each updating its own `v3/area-<letter>.md`:
every NEW block, owner's-test row and message brought to the v3 rules; checks the phase 0 lines in its
area; marks every answer whose meaning changed; "words needed" table for anything not in the vocabulary.

**Phase 3, the lead, no agents.** Fix the error-message failures of `v2/readability.md` (F-01 to F-11,
F-13) in the v3 files, and any message an area updater flags.

**Phase 4, one agent.** The readability checker: the owner's test on every NEW block, the message rule on
every message, every verdict 1 to 34 applied everywhere with no old spelling left (`holds`, `cast ... if`,
`cast ... when`, `.copied()`, a plain `MutSpan` name, `self`, `owned`, `copied`, `weak` alone, `set by`),
and the agreed rows exactly as agreed.

## Agents

| # | job | why its own job | model | effort |
|---|---|---|---|---|
| 01 | rule writer | every area depends on it; settles the rule consequences of 21-34 | Opus 5.5 | xhigh |
| 02 | area A (AN-01..16) | `MutSpan` in most answers (34), parallel loops (30), `for_each` (21) | Opus 5.5 | high |
| 03 | area B (AN-17..33) | locks in function types (29), downcasts (27) | Opus 5.5 | high |
| 04 | area C (AN-34..52) | arenas (24), parameters of `shares mut` types (32), `.copy_each()` (33) | Opus 5.5 | high |
| 05 | area D (AN-53..70) | callbacks (22, 31), downcasts (27), the deleting walk (28) | Opus 5.5 | high |
| 06 | area E (AN-71..84) | threads and C callbacks: 29, 30, 31, 32, 34 all land here | Opus 5.5 | xhigh |
| 07 | readability checker | the independent check, after the lead's message fixes | Opus 5.5 | high |

7 agents; worst case 14 (one retry each). No Fable.

## Safeguards (as in step 1)

Every agent reads the same brief (the goal, word for word); every form has `finished`, which is false if
the agent was stopped (a classifier stop included); a not-finished agent is retried once, then the
workflow halts. No compiler changes, no git writes, `ember check` only, never run an Ember program, no
windows, scratch under `target/redesign4/v3-<number>-<name>/`. Power checked; on battery everything stops.

## Outputs

`run4/v3/`: `rules.md`, `vocabulary.md`, `changes.md`, `area-A.md` to `area-E.md`, `readability.md`,
`spelling-changes.txt`. The owner gets a short report: what changed per verdict, the readability
check's result, and any new question, one at a time.
