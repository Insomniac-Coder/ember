# Rules for the team (method 4; every agent reads this first, in full)

## A. The owner's words (verbatim; they bind)

* The goal of the job: "use a workflow ... that goes through the spec and finds all gaps and tries to
  fix them with consistency in mind".
* The bar: "I need perfectly engineering no compromise, safe, high readability, high performance, goal
  compliant solution that are in line with the ODRs I have approved unless you can do something really
  REALLY better"; "pleassseeee no specific solutions, this is a programming language and people will use
  it to write billions of things so any fix that only targets a fix for a specific case is not good";
  "make sure the lnaguage does not lose its ease of you and readability"; "the complexities can be
  abstracted in the complier like we've been doing already".
* This method, in his words (2026-11-03): "while I did give you strict points to follow I see not all
  can be followed 100% but regardless safety remains the only no compromise part and the complexities
  should mostly be abstracted in compiler like things are right now and I also agree that sometimes the
  fix is somewhere else and not where it was reported in that case let the agents use the best
  judgement but let them also provide the reasoning and impact of the change. Design a team that works
  towards a common goal and something that beats each other down to death."
* Decisions: "I want the workflow to make the decision based on the parameters that I set, then maybe
  later once stuff is done then you can discuss the changes". Nothing goes into the specification before
  he has reviewed it.

## B. The common goal

A language as fast as C and Rust, as memory-safe as Rust or safer, and as easy to read and write as
Python, all three at once (`docs/OWNER-DIRECTION.md`, "The goal"; read its principles and checklist).
You are one team of seasoned engineers with that goal. You are hard on the work and fair to each other:
every objection is a whole program and a walk through the rules; every answer to an objection is a fix
or a rebuttal that quotes the rules; when the other side is right, you say so.

## C. The pass mark: only safety is never compromised

A change is **blocked** only by these, each shown by a whole program walked through the rules (part file,
lines, words):

* **S. A safety hole caused by the change:** a program the changed specification lets through that
  reads freed memory, races, ends a value twice, has undefined behaviour, or faults without a clear stop
  at its line; or a safety guarantee of today's specification that the change takes away.
* **U. The question left undecided:** the question's own program is still not decided by the changed
  words.
* **X. A contradiction the change creates:** after the change, two rules decide one program differently.

Everything else is **weighed, never ignored**: a correct program refused, readability, cost over C, a
word the programmer must write that the compiler could work out, a special case, a clash with a verdict,
a program whose meaning changes. The designer answers each one (fix it, or keep the trade-off with the
reasoning and its impact); the arbiter judges whether the trade-off is justified by section B; the owner
reads every trade-off later. A departure from an owner verdict is never applied silently: it is always
listed for the owner, with the evidence that it is really, really better.

## D. Where the fix goes

* **At the cause, wherever it is.** A question names where the problem showed. The fix goes where it is
  caused, in any rule of the specification. When that is not where the problem was reported, say why
  the cause is there.
* **The language's own tools first.** Find the rule whose property already decides the case and the
  verdicts that bear on it, before writing a word. Extend an existing rule's words before adding a rule.
* **The smallest change at the cause.** A property, never a list of cases, never the question's example
  named as a case. Three sibling programs (other types, other constructs, other kinds of software) are
  decided by the same words with nothing added. No clause per objection: a revision rewrites the change
  as a whole.
* **The compiler carries the complexity.** A word the programmer writes is for meaning only (who owns,
  who may change, who may run it); everything the compiler can work out, it works out.
* **Impact, always.** Every change carries its impact report: each rule changed (before, after, why the
  cause is there); every program whose outcome changes, before and after; readability before and after
  for an ordinary program; cost per operation against C (`to measure` where it must be measured); the
  verdicts it keeps or departs from; the decisions made.
* **Decisions by the owner's parameters**, in this order: B's three goals; safety never given up;
  compile time first, run time only where compile time would refuse correct programs, then the cheapest
  way; a written word only for meaning; the ordinary program no longer or harder; consistency with the
  nearest rule and the verdicts. Each decision is recorded: the options as code with their outcomes, the
  one chosen, the parameters that chose it.
* **Outcomes:** `decided` (an existing property reaches the case once its words say so),
  `decided_by_parameters` (a choice was needed and was made), `already_decided` (the specification as
  written decides it: quote the lines), `owner_question` (the parameters cannot all hold, two verdicts
  clash, or a verdict would be reversed or widened: no change, options and one recommendation),
  `whole_feature` (a feature the specification does not have yet, with many cases no rule decides: no
  change, a design question for the owner with at least three undecided cases, options and one
  recommendation).

## E. Evidence

* Every program whole, in the specification's 0.9.15 spelling (`owns`, `ref mut` at calls, `this`,
  `exists as`), every name defined in it, from a real kind of software (a game engine, a renderer, a
  server, a parser); name the danger in C++ terms in one phrase. A test file may begin with the C or C++
  header it imports.
* A test's expected result is exact: an error code the specification names (or "E-new" with the
  message's gist), or the exact text the program prints. Never a description, never "as above".
* Every claim about the specification: the part file, the lines, the words. No invented numbers.
* A problem is caused by the change only when the same program does not go wrong the same way without
  it; otherwise it is a gap that already exists, listed for later, never a reason to block.

## F. How you work

* **Read-only, strictly** (the owner: "keep the rule strict"). Read and search with the Read, Grep and
  Glob tools; in a shell only `cat`, `sed -n`, `grep`, `ls`. Never `node`, `python`, `wc`, any script or
  one-liner, never anything that writes, builds, runs or tests, never the compiler. Edit, create or
  delete no file. Skip the repository's start-up reading (`docs/AUTOPILOT.md`, `docs/HANDOFF.md`): it is
  for compiler work.
* The question's input is `deep/v3/inputs/<cluster>.md` (plain text, short lines). The specification is
  `docs/spec-source/Ember_v0.9.15_Hardened_11.md`, from `tasks/spec-0.9.9/parts/p*.md` (quote the part
  files' lines). The verdicts: `tasks/96/redesign/JOB.md`. The direction: `docs/OWNER-DIRECTION.md`.
* `finished` is true only when your job is done. If you are cut off, say so in `notes` and set it false;
  never return an empty result as a pass. Plain words, no compiler internals.
* What the owner reads (`owner_short`): one line naming the thing in C++ terms; a step table "step | who
  | what happens | result" (five rows at most); today in one line; each option in one line; the
  recommendation in one line. No rule ids in it.
