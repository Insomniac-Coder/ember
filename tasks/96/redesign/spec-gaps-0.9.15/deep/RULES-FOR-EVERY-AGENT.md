# Rules for every agent of the deep review (read first, in full; every output is checked against them)

The owner, 2026-11-02: "please follow all the instructions I gave, remember what I told you about this
workflow and what it needs to satisfy all agents HAVE to abide by those rules". An output that breaks
any rule below is rejected, whatever else it gets right.

## A. The owner's words for this work

* "use a workflow ... that goes through the spec and finds all gaps and tries to fix them with
  consistency in mind"
* "I need perfectly engineering no compromise, safe, high readability, high performance, goal compliant
  solution that are in line with the ODRs I have approved unless you can do something really REALLY
  better"
* "I don't want ANY compromise on quality ... the language is supposed to be robust and fault tolerant
  and pleassseeee no specific solutions, this is a programming language and people will use it to write
  billions of things so any fix that only targets a fix for a specific case is not good, generalized
  solution that meet all the other parameters please"
* "make sure the lnaguage does not lose its ease of you and readability"
* "the complexities can be abstracted in the complier like we've been doing already"
* After the first run patched instead of generalising: "Why is the workflow not giving general solution
  that is EXACTLY what I asked for"
* On the decisions a rule needs: "I want the workflow to make the decision based on the parameters that I
  set, then maybe later once stuff is done then you can discuss the changes"

## B. The goal, all three at once (`docs/OWNER-DIRECTION.md`, "The goal")

Speed on par with C and Rust; memory safety the same as Rust's or better; Python-like to read and write.
A solution that gives up one of the three, or is a shortcut, is not a solution.

## C. The bar (`deep/owner-bar.md`, in full)

1. **General, never specific.** One rule per cause, answering every case that shares the cause and the
   cases nobody has asked about. No clause for one construct, one type, one example or one program. A
   rule with an exception in it is suspect; an exception stands only where the general rule would say
   something untrue, and it says why (principle 6).
2. **Ease of use and readability are not traded for anything.** The ordinary program (a game loop, a
   server handler, a parser) stays as short and as plain as today; show it before and after.
3. **The compiler carries the complexity.** What the compiler can work out, it works out (bodies,
   values, facts); a word the programmer writes is the last resort, for meaning only (principle 2).
4. **Safe, robust, fault tolerant.** No allowed program reads freed memory, races or has undefined
   behaviour; a mistake is refused when compiling where it can be, else a clear stop at its line.
5. **Fast.** Compile time first; run time only where compile time cannot decide without refusing correct
   programs, and then the cheapest way, its cost stated per operation against C.
6. **Consistent with the owner's verdicts** (`tasks/96/redesign/JOB.md`, 1 to 166), unless something is
   really, really better: then say so plainly, with the evidence, as an option for the owner.

## D. How a rule is made (learned from the first run, which patched)

1. **The language's own tools first.** Before writing anything new, search the specification and the
   verdicts for a rule that already answers the cause, and use it as it is. Example: verdict 53, a fact
   the compiler works out from a function's body is kept with the function and checked at every call;
   it answers "a body compiled once cannot see its caller" for every function, every kind, with no word
   and no run-time cost.
2. **One rule per cause, not one clause per hole.** A list of holes is grouped by what causes them; the
   rule answers the cause, and every hole of that cause follows from it.
3. **No growth.** A repaired rule is no longer than the first design's rule. A longer text is a patch.
4. **Decide by the owner's parameters, and record the decision.** The owner delegated this workflow's
   decisions (quote in A). Where a cause needs a choice (the compiler cannot work the fact out, so the
   choice is a written word, a run-time check or a refusal), make it yourself, never leave it open, and
   never patch around it. Decide by the owner's parameters in this order: B's three goals all at once;
   safety is never given up (C4); compile time first, run time only where compile time would refuse
   correct programs, and then the cheapest way (C5, principle 8); a written word only for meaning (who
   owns, who may change, who may run it), never for bookkeeping (C3, principles 1 and 2); the ordinary
   program no longer or harder (C2); consistency with the nearest rule and the verdicts (C6, principle 5).
   Record each one in `decisions_taken`: the options as code and outcome, the one chosen, and which
   parameters chose it. The owner reviews them later.
5. **Verdicts and the specification.** Keep every verdict; depart from one only when the result is
   really, really better by the owner's own parameters, and record that departure in `decisions_taken`
   with the evidence. Never widen a verdict silently. Nothing here is written into the specification; a
   design is a proposal until the owner has reviewed it.

## E. The checklist, every item (`docs/OWNER-DIRECTION.md`, "Before ruling anything")

1 the program's intent, done exactly; 2 every correct program accepted, only the wrong refused;
3 compile time first, else the cheapest way measured against C/C++; 4 the programmer writes only meaning;
5 nothing said twice, one word one meaning for every type; 6 consistent with the nearest rule, an
exception only where the general wording is untrue, saying why; 7 general, production-ready, no ceiling;
8 a forgotten mark refuses, never crashes; 9 every message gives a line that compiles; 10 a change to what
the language does is the owner's (in this workflow: decided by D4 and recorded for his review); 11 an idea already spelled is spelled that way, the older second
spelling goes; 12 no exemption because syntax is missing, add the syntax; 13 no special or hidden path,
the normal model holds; 14 nothing is run here: every claim that would need running is marked
`to verify` (the lead runs it); 15 a written declaration means only what it says; 16 a verdict's
spelling reaches every place; 17 one diagnostic for several places names all of them; 18 messages read
aloud are plain English; 19 only a language decision left open needs a choice (in this workflow: made by
D4, recorded in `decisions_taken`).

## F. Evidence

* Every claim with the concrete program, in the specification's 0.9.15 spelling, every name defined,
  and the rule lines it rests on, quoted with their line numbers.
* No invented numbers. A cost is the mechanism and what it costs per operation against C, marked
  `to measure`; the lead measures on the laptop.
* A hole is real only when its program is shown to do what it claims under the rules as written.

## G. How you work

* **Read-only, strictly** (the owner, after agents ran `node -e` to read JSON: "b, keep the rule
  strict"). Read and search files only: the Read, Grep and Glob tools, or a shell command that only
  prints or searches a file (`cat`, `sed -n`, `grep`, `ls`). Never `node`, `python` or any other script
  or line of code (no `-e`, no `-c`), never a command that writes, builds, runs or tests anything, and
  never the compiler or `cargo`. Do not edit, create or delete files.
* **Read the plain-text copies** in `deep/v2/text/`: `<group>.md` (your group's input), `phase1-groups.md`,
  `phase2.md`, `phase3.md`. They hold exactly what the JSON files hold, with lines short enough for the
  Read tool (it cuts longer lines), so no script is ever needed. The workflow measures the length cap
  itself and tells the designer when a text is over it; nobody needs to count, and never with a script.
* Set `finished` true only when you checked everything your job lists. If you are cut off or stopped,
  say so in `notes` and set `finished` false; never return an empty result as a pass.
* Plain words: no compiler internals, no unexplained terms.
* **Realistic programs** from real kinds of software (a game engine, a renderer, a server, a parser),
  never toy names. The owner writes C++ game engines: name the danger in C++ terms in one phrase.

## H. What the owner reads later (`owner_short`, for every entry of `decisions_taken`)

`one_line` naming the thing in C++ terms; `step_table`, a markdown table "step | thread | what happens |
the value"; `today`, one line; `options`, one line each, letters fixed; `recommendation`, one line.

## I. Compliance

Every output carries `rules_checked`: for each of A to H, how this output keeps it. The gate and the
lead check it; an untrue entry rejects the output.
