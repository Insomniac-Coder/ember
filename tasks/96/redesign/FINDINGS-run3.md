# Run 3 of the memory-model rework: what it found (2026-10-25)

The run followed [PLAN-run3.md](PLAN-run3.md) and [JOB.md](JOB.md). 26 agents ran, from 13:35 to
18:18 IST, with no stop and no agent failing. Every report is in `run3/` (named by agent number), the
code in `target/redesign3/`. Nothing in the compiler was changed and nothing is committed.

## 1. The most important result: the model was never attacked for memory safety

**Both safety attackers (16 and 17) were stopped at the start by a safety classifier and did no
attacking.** Under your rule that a classifier stop ends the work, each wrote "stopped, no result" in its
report (`run3/16-safety-one-thread.md`, `run3/17-safety-threads.md`) and handed back a form with no
problems in it.

**My script counted that as "no problems".** The form had no field saying whether the agent finished
its duty, so the automatic checks could not tell "found nothing" from "did nothing". That is a mistake
in how I built the workflow. Every form needs a "finished: yes or no" field, and "no" has to stop the run.

What safety checking did happen: the two re-attackers (20 and 22) attacked **only the fixes**, and were
not stopped. They proved 9 memory errors with the memory checker (AddressSanitizer), all in rules the
fixers had just changed:

| Round | Proven memory errors in the fixes | Still open at the end |
|---|---|---|
| after fix round 1 | 4 (F-01 to F-04) | fixed in round 2 |
| after fix round 2 | 5 (G-01 to G-05) | **all 5 open** |

The five open ones, in plain words:
1. **G-01:** a copy of a `weak` link made inside a loop that delays freeing can outlive what it points to,
   and reading it reads freed memory.
2. **G-02:** a view kept inside an object survives a `push` on one of that object's lists, so it points
   into memory the list has moved away from.
3. **G-03:** a changeable view of several lists and a view of numbers inside one of those lists pass the
   "are these different?" test, so a write frees what the other view shows.
4. **G-04:** a loop over a module variable `LOG` is accepted across a call that passes a handler on to
   another function, which runs it and changes `LOG`.
5. **G-05:** a handler written with `with weak ui` changes the list that a loop is walking.

So the final model (`run3/combined-v4.md`) is **not** shown to be memory-safe. It has 5 proven holes, and
its base (`combined-v2.md`) was never attacked.

## 2. The second result: each round of fixes made new problems

| Step | Problems found | Of them, proven memory errors |
|---|---|---|
| readability attack on the combined model (18), plus 15's leftover library names | 22 + 26 | 0 (nobody attacked safety) |
| re-attack of round 1's fixes (20) | 12 | 4 |
| re-attack of round 2's fixes (22) | 11 | 5 |

The fixers fixed 45 of the first 48 (3 need a word only you can decide), and all 12 of round 1's. Each
fix made the rule it touched longer and more special-cased. The model grew from 10,644 lines
(`combined.md`) to 11,342 (`combined-v4.md`), and rule R2 (what `n = p.name` means) is now a page long,
with "Was: ..." notes inside it.

The lesson: patching the model rule by rule does not settle. The number of problems found per round is
not falling (48, then 12, then 11), and the fixes keep opening memory holes. That means the model as a
whole is too intricate to fix one rule at a time.

## 3. What did work

- **The vocabulary phase.** The four readers judged 213 distinct words across the whole spec. The Builder
  produced a 255-word table and the Checker changed 22 rows. Every must-cover word got a row, and your
  agreed rows 3 to 9 came through unchanged at every step (the script checked).
- **84 annoyances, all answered.** The Builder found 84 annoyances in today's Ember (A 16, B 17, C 19,
  D 18, E 14) and every one was answered in the eight-part form. The automatic check sent 2 incomplete
  answers to the Combiner, which completed them.
- **Every claim about today's Ember was checked.** The old-code re-checker re-ran `ember check` on 245
  snippets and found **0 mismatches**.
- **One vocabulary, with clashes settled.** The Combiner settled 16 clashes between areas, and every new
  word got a ruling.
- **Error messages, your new rule.** The readers found many messages that don't say what to write
  instead: 56, 4, 51 and 205 in the four slices (the 205 are reader 4's pass over the error table). That
  is a large, separate piece of work.

## 4. The words the model uses (`run3/vocabulary-final.md`)

- **New:** `thread_shared` (several owners, on any thread; replaces `Arc`-style sharing), `weak` (a link
  that does not keep its target alive), `set by` (`parent: weak Node set by children`: the language keeps
  the back-link right), `@cached`, `guarded by <lock>`, `Lock`, `Commands.create()`, `from static`.
- **Replaced:** `Box[T]` becomes `owned T` (your rule); `RefCell` and its `borrow` calls are removed (the
  method says `changes entities`); `@sync` is removed (the value says `thread_shared`, each field
  `guarded by` or `Atomic`).
- **Kept:** `ref mut` (your example 8 ruling), `copied`, `shared`, `changes` and `reads` on a first line,
  `fn() with copied score:`.

## 5. Speed (`run3/26-timer.md`)

Median of 5 rounds, fast cores, mains power, nothing else running. Every checksum equals plain C's.

| Workload | Plain C | The model | Safe Rust |
|---|---|---|---|
| W1, the ECS | 93.3 ms | 81.4 ms (0.87x) | 100.4 ms (1.08x) |
| W2, tree with parent links | 576.0 ms | 909.6 ms (1.58x) | 923.4 ms (1.60x) |
| W3, game objects referring to each other | 443.5 ms | 439.5 ms (0.99x) | 635.4 ms (1.43x) |
| W4, callbacks that remove themselves | 23.7 ms | 29.7 ms (1.26x) | 32.1 ms (1.36x) |

- **W2's cost** is one test on every step through a `weak` link, which rule R11 requires. Without that
  test (`diag_w2_hop_proved`) it is 1.18x. The model contradicts itself here: AN-62 says no test is
  needed, R11 says it is. The faithfulness reviewer found this, and only you can rule on it.
- **W4's cost** is one small box per captured variable, as the model's text says. With the variables
  kept in the handler's own state instead, it is 1.09x.
- **Noise:** some timings spread more than 10% even after two re-runs. `plain_w2` spread 20.6% and
  `plain_same_w1` 27.2%, so W1 and W2 are a few percent uncertain.
- **The C is the model's, checked:** the faithfulness reviewer found 10 gaps in the first C, and the
  second C writer fixed every one it could. The timings use the fixed C.

## 6. Decisions only you can make (3 word questions from the fixers)

1. **R18-16:** `bump(a)` changes `a`, and no word on that line says so. It needs a word at the call site.
2. **R18-14:** `mut self` and `static mut` keep a bare `mut`, which your example 8 test refuses everywhere
   else. The alternative is `ref mut self`.
3. **R18-22:** `ref mut` means "the only way in" on a parameter, but "one of several ways in" on a kept view.

I will explain these one at a time when you want them.

## 7. What I recommend next

1. **Don't build on `combined-v4.md`.** It has 5 proven memory holes, and its base was never attacked.
2. **Fix the workflow first:** a "finished?" field on every form, and a stopped agent stops the run.
3. **Then the safety attack has to happen.** The attackers were stopped by a classifier, and your rule
   says not to work around a classifier. So how to get the attack done is your call: run it again as it
   was, let me do the attacking myself, or reword the duty.
4. **Change how fixes are made.** Fixing one rule at a time kept opening holes. Before more fixing, the
   model's core rules (R2 on what a name for a part is, R10 on keeping objects alive, R11 on `weak`) need
   simplifying as a whole.
