# Rules for every agent of the gap review, method 3 (read first, in full)

The owner, 2026-11-02: "please follow all the instructions I gave, remember what I told you about this
workflow and what it needs to satisfy all agents HAVE to abide by those rules". An output that breaks a
rule below is rejected, whatever else it gets right. The workflow script checks what a script can check
(scope, size, quotes); the breakers check the rest; nobody self-certifies.

## A. The owner's words for this work (verbatim; they bind)

* "use a workflow ... that goes through the spec and finds all gaps and tries to fix them with
  consistency in mind"
* "I need perfectly engineering no compromise, safe, high readability, high performance, goal compliant
  solution that are in line with the ODRs I have approved unless you can do something really REALLY
  better"
* "I don't want ANY compromise on quality, along with the things I said before the language is supposed
  to be robust and fault tolerant and pleassseeee no specific solutions, this is a programming language
  and people will use it to write billions of things so any fix that only targets a fix for a specific
  case is not good, generalized solution that meet all the other parameters please"
* "make sure the lnaguage does not lose its ease of you and readability"
* "the complexities can be abstracted in the complier like we've been doing already"
* "Why is the workflow not giving general solution that is EXACTLY what I asked for"
* "I want the workflow to make the decision based on the parameters that I set, then maybe later once
  stuff is done then you can discuss the changes"
* "you and your agents are the same so there is not guarantee" (a check by another agent proves
  nothing; a test on the real compiler does)
* "I am okay with a single repair round if that fails then no more tries"

## B. The goal, all three at once (`docs/OWNER-DIRECTION.md`, "The goal")

Speed on par with C and Rust; memory safety the same as Rust's or better; Python-like to read and write.
A solution that gives up one of the three, or is a shortcut, is not a solution.

## C. The bar (`deep/owner-bar.md`, read it in full)

1. **General, never specific.** An answer states a property that holds for every value, type and
   construct it reaches. It never names the question's own example, one type or one construct as a
   case. The test: three sibling programs (other types, other constructs, other kinds of software) are
   decided by the same words with nothing added. An exception stands only where the general wording
   would say something untrue, and it says why (principle 6).
2. **Ease of use and readability are not traded for anything.** The ordinary program (a game loop, a
   server handler, a parser) stays as short and as plain as today; show it before and after.
3. **The compiler carries the complexity.** What the compiler can work out (bodies, values, facts), it
   works out; a word the programmer writes is the last resort, for meaning only (who owns, who may
   change, who may run it).
4. **Safe, robust, fault tolerant.** No allowed program reads freed memory, races or has undefined
   behaviour; a mistake is refused when compiling where it can be, else a clear stop at its line.
5. **Fast.** Compile time first; run time only where compile time cannot decide without refusing
   correct programs, and then the cheapest way, its cost stated per operation against C.
6. **Consistent with the owner's verdicts** (`tasks/96/redesign/JOB.md`, 1 to 166). A verdict is never
   reversed or widened by an agent. "Really, really better" is shown as an option for the owner, with
   the evidence, never applied.

## D. The job: answer ONE question with the smallest change

This replaces the first run's job ("one general rule for the group"), which made agents rewrite whole
areas (about 10 to 25 existing rules per design) and broke rules that worked. The method-2 checker found
most of the resulting holes were in rules the specification already answered.

1. **One question, nothing else.** You answer the cluster you are given (one or more member questions
   on one point). You do not write a rule for its area. You do not touch a rule the question was not
   filed under; the script rejects an edit outside the allowed rules.
2. **The language's own tools first.** Before writing a word, find the rule whose property already
   decides the case, and the verdict that bears on it. Five outcomes, and only these:
   * `already_decided`: the specification as written decides it. Quote the lines. No edit. (Say which
     reading the finder missed.)
   * `decided`: an existing rule's property reaches the case, but its words do not say so. Extend the
     words of that rule so the property plainly covers it. No new choice is made.
   * `decided_by_parameters`: a choice is needed (the compiler cannot work the fact out, so only a
     written word, a run-time check or a refusal remain). Make it yourself by the owner's parameters,
     in this order: B's three goals all at once; safety never given up (C4); compile time first, run time
     only where compile time would refuse correct programs, then the cheapest way (C5); a written word
     only for meaning, never for bookkeeping (C3); the ordinary program no longer or harder (C2);
     consistency with the nearest rule and the verdicts (C6). Record it in `decisions`: the options as
     code with their outcomes, the one chosen, the parameters that chose it in order. The owner reviews
     it later; nothing is written into the specification before that.
   * `owner_question`: the owner's parameters cannot all hold, two verdicts clash, or the answer would
     reverse or widen a verdict. No edit. Give the options as code and one recommendation.
   * `whole_feature`: the question asks for a feature the specification does not have yet: a whole
     mechanism (how a family of constructs works, with many cases no rule decides), so no small edit can
     settle it. The owner: such a question "goes to the owner as a design question, not through the
     small-fix workflow". No edit. Write it as a design question: what is missing, at least three of the
     cases no rule decides (each as a program in `sibling_cases`), the options as code, one
     recommendation. Use it only when it is true: a question that an existing rule's property already
     reaches, or that one choice settles, is not a whole feature.
3. **Smallest change, measured by the script.** At most three rules edited; the new text together no
   longer than the cap the script applies (it tells you when you are over; never count characters
   yourself). An edit quotes the exact words it replaces (`before`) and gives the new words (`after`);
   the script checks `before` is really in the rule. Nothing else in the specification changes.
4. **One repair, no growth.** If the breakers find a real hole, one repair agent replaces the edit as a
   whole; the script rejects a repair with more new text than the first answer. A repair never adds a
   clause per hole. Where a hole needs the owner, the repair ends as `owner_question` instead (or as
   `whole_feature`, when the holes show the question is one).
5a. **The last holes are checked.** After the last two breakers, one checker confirms each hole they found
   is real and caused by the edit, as the repair does for the first round's (the owner: "both"). Only
   confirmed holes count; the pass mark is unchanged.
5. **General means this, here:** the answer comes from the property that already covers the case. A
   property reaches the cases nobody asked about; a list of cases does not. If you cannot write the
   answer as a property, it is a choice (outcome 3) or the owner's (outcome 4), not a clause.

## E. The checklist (`docs/OWNER-DIRECTION.md`, "Before ruling anything", items 1 to 19)

Read it in the file. Items 10 and 19 in this workflow mean: a choice the specification leaves open is
made by D2's parameters and recorded in `decisions`; a reversal or widening of a verdict is never made,
it is an `owner_question`. Item 14 means: nothing is run here; every claim that would need running is
listed in `to_verify` (the lead runs it on the compiler).

## F. Evidence

* Every program whole, in the specification's 0.9.15 spelling (`owns`, `ref mut` at calls, `this`,
  `exists as`), every name it uses defined in it, from a real kind of software (a game engine, a
  renderer, a server, a parser), never toy names. Name the danger in C++ terms in one phrase.
* Every claim about what the specification says: the part file, the line numbers, the words.
* No invented numbers. A cost is the mechanism and what it costs per operation against C, marked
  `to measure`.
* A hole is real only when its program is shown to do what it claims under the rules as written, and
  the same program does not go wrong the same way without the edit (else it is `pre_existing`: a new
  gap for the list, never a reason to repair this edit).

## G. How you work

* **Read-only, strictly** (the owner: "b, keep the rule strict"). Read and search files with the Read,
  Grep and Glob tools. If you have a shell, only `cat`, `sed -n`, `grep` and `ls`. Never `node`,
  `python`, `wc`, any script or one-liner (no `-e`, no `-c`), never a command that writes, builds,
  runs, counts or tests anything, never the compiler or `cargo`. Do not edit, create or delete files.
* **Nothing to count.** The script measures every length and tells you the result. The inputs are
  plain text with short lines, so the Read tool shows all of them; no script is ever needed.
* **Your input file** (`deep/v3/inputs/<cluster>.md`) holds the question(s) in full, the rules they
  were filed under with their text and line numbers, the group's principle and the places it lists, the
  verdicts involved, the critic's notes, and the allowed rules. The specification is
  `docs/spec-source/Ember_v0.9.15_Hardened_11.md`, assembled from `tasks/spec-0.9.9/parts/p*.md`
  (quote line numbers from the part files). The verdicts: `tasks/96/redesign/JOB.md`. The lead's
  rulings: `tasks/96/redesign/compiler-0.9.15/decisions.md`.
* Set `finished` true only when you checked everything your job lists. If you are cut off or stopped,
  say so in `notes`, set `finished` false, and leave what you have; never return an empty result as a
  pass. A stopped job is reported, never retried with other words.
* Plain words: no compiler internals, no unexplained terms.

## H. What the owner reads later (`owner_short`, for every entry of `decisions`)

`one_line` naming the thing in C++ terms; `step_table`, a markdown table "step | who | what happens |
result" (five rows at most); `today`, one line; `options`, one line each, letters fixed; and
`recommendation`, one line. No rule ids in it; say what a rule does instead.
