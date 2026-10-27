# The spec Ember_v0.9.15_Hardened_1: the new memory model written into the spec (PLAN, for the owner's approval)

## Goal

Write the new design (`run4/v5`: the rules, the keyword list, verdicts 1 to 99) and the 34 fixes of the
unsafe-features inventory into the spec, replacing today's memory model, and cut it as
`docs/spec-source/Ember_v0.9.15_Hardened_1.md`. Nothing in the compiler changes in this step.

## How the spec is built today

28 parts in `tasks/spec-0.9.9/parts/` (`p00-front.md` .. `p27-appx-i.md`, about 640 KB of hand-written
text and 250 KB generated), joined into one file; tools there check rule IDs, citations, examples and
error codes, and write the change history (Appendix H). Each cut freezes a copy of the parts.

## Phase 0, the lead, no agents

- The agents' inputs: the shared brief; the duties; a crosswalk from each rule of `run4/v5/rules.md` and
  each keyword-list row to the spec parts and rule IDs it replaces (the keyword list already names today's
  spec lines); the 34 inventory fixes and the 3 unsure points (U1 to U3, settled safely by the lead) mapped
  to parts; the style the spec is written in.

## Agents

| # | parts | why its own job | effort |
|---|---|---|---|
| 1 | p01 overview, p07 ownership, p09 memory | the core of the new model (owners, links, views, ends, arenas) | xhigh |
| 2 | p10 effects, p11 concurrency | worked-out changes (no `changes`), function values, threads, reductions | xhigh |
| 3 | p02 lexical, p03 grammar, p04 types | the new words and types (`owns`, `shares`, `weakref`, `ref mut`, `copy of`, `exists as`, casts) | xhigh |
| 4 | p05 declarations, p06 expressions, p08 classes | parameters, captures, `init`, `drop`, methods, kind tests | xhigh |
| 5 | p12 data-oriented, p14 compile-time, p15 standard library | the library's changed types and functions (8 inventory fixes) | high |
| 6 | p16 C and C++, p18 implementation | C calls, `into_raw()`, callbacks, `unsafe` (12 inventory fixes) | xhigh |
| 7 | p00 front, p13 errors, p17 toolchain, p17a codes | error codes removed and added, flags, stack probes | high |
| 8 | the annexes p19 to p24 | examples, hot reload (3 inventory fixes, U1, U2), GPU draws | high |
| 9 | check: every rule, verdict and fix is in the spec; nothing of the old model is left | | xhigh |
| 10 | check: the spec's own tools, cross references, codes, examples | | high |

All Opus 5.5, no Fable. 10 agents, worst case 20 with one retry each.

## Phases

1. Phase 0 (the lead), then agents 1 to 8 together.
2. Agents 9 and 10 together.
3. The lead fixes what the checks find, writes the change history (verdicts 1 to 99 as this cut's record),
   rebuilds and cuts `Ember_v0.9.15_Hardened_1.md`, and freezes its parts.

Not in this step, each for the owner's word: pointing the development target (`development-target.json`)
at the new spec, which makes the compiler measured against it; renaming the 842 tests' rule names to the
spec's rule IDs (the writers record the mapping).

## Cost

Step 4 averaged 520 thousand tokens per agent; the writers here rewrite more text. Estimate 5 to 7 million
tokens, about 2 hours.

## Safeguards (as before)

The same brief for every agent; a form with `finished`; one retry, then the run halts; a safety filter stop
halts and is reported; each writer edits only its own parts; no compiler changes, no git writes, nothing
compiled or run; scratch under `target/redesign9/`; no power checks.
