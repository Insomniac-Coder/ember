# The list of unsafe features (verdict 60) (PLAN, for the owner's approval)

## Goal

Find every feature of Ember that can leak memory, skip a safety check, race between threads, or reach
memory the rules cannot see, and check each one against verdict 60: "anything that can break a safety
promise is possible only inside an `unsafe` block". For each one that is not fenced: what Rust does, and
a recommendation (fence it inside `unsafe`, make it safe with a rule, or remove it). `mem.forget` and
`into_raw()` were two such features; this finds the rest, once, before compiler work.

## Where to look

- the new design, `run4/v5` (the rules and the keyword list);
- today's spec, `docs/spec-source/Ember_v0.9.10_Hardened_35.md` (10,000 lines), for the parts the
  redesign keeps: C calls, `unsafe`, hot reload, generators, SIMD, compile-time code, memory managers,
  arenas, casts, build flags;
- today's standard library (`std/src`, 9,000 lines), the compiler's built-in functions and the runtime
  that programs can reach.

## Agents

| # | job | why its own job | model | effort |
|---|---|---|---|---|
| 1 | the new design (rules and keyword list) | the language as it will be | Opus 5.5 | high |
| 2 | today's spec: C calls, `unsafe`, the runtime, build and command-line flags | the biggest family (C calls alone are 440 rules) | Opus 5.5 | xhigh |
| 3 | today's spec: everything else the redesign keeps | the other half of the spec | Opus 5.5 | high |
| 4 | the standard library, built-in functions, runtime | what programs can call without `unsafe` | Opus 5.5 | high |
| 5 | attacker: write safe programs that break safety anyway | finds what reading misses (leaks through shared-owner loops, races through globals, a stop inside `drop`, ...) | Opus 5.5 | xhigh |
| 6 | checker: try to disprove every finding of 1 to 5 against its source | keeps only what is real | Opus 5.5 | xhigh |

6 agents, worst case 12 with one retry each. No Fable.

## Phases

1. Agents 1 to 5 together; each writes its findings with the source line.
2. Agent 6 checks them; the lead merges the confirmed ones into one list.

## Cost

Step 4 averaged 520 thousand tokens per agent (5.22 million for 10). Estimate for 6 agents: 2.5 to 3.5
million tokens, about an hour.

## Output

`run4/v5/unsafe-inventory.md`: one row per feature (what it does, how it can break safety, fenced or not
today and in the new design, Rust's way, the recommendation), and a short list of points for the owner,
which are brought one at a time, each first checked against `JOB.md`.

## Safeguards (as before)

The same brief for every agent; a form with `finished`; one retry, then the run halts; a safety filter
stop halts and is reported; read-only except the agents' own finding files; no compiler changes, no git
writes, nothing compiled or run; scratch under `target/redesign8/`; no power checks (the owner's rule).
