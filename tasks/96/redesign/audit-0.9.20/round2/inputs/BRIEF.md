# Brief for every agent of round 2 (0.9.20_Hardened_3)

You are reviewing the Ember language specification, development target 0.9.20 (the file CURRENT-SPEC.md
beside this brief is the whole current text, Parts I to XVIII and the annexes; each rule is a bullet
`* `[ID]` ...`). The owner's goal: a complete specification with no holes, keeping four things at once:
C-like speed (nothing new runs while a program runs unless the property is run-time data, and every cost
has a row in section X.4), Python-like readability (a word on the thing it describes), Rust-like memory
safety without lifetime names (checked when compiling), and consistency (every cross-reference claims
what its rule says; one code, one defining rule; examples compile or name unbuilt syntax).

Rules of the work (the owner's, binding):
1. Safety is never compromised. Everything else is weighed and the trade-off is written down.
2. A fix goes where the problem is caused, as the smallest change there, in the document's own voice
   (no "you"; the message gives the line to write; a rule states what the compiler reads, never a fact
   of the whole program a package cannot see, see [BLD-16]).
3. An owner verdict (tasks/96/redesign/JOB.md, verdicts 1 to 166) is a constraint: never overridden.
   Where the text and a verdict disagree, say so; where a better solution would depart from a verdict,
   propose it and mark it "departs from verdict N".
4. Read only. Do not edit any file. Do not run scripts, builds or tests. Your output is data for the lead.
5. Every claim quotes the current text (the rule id and the exact words) it rests on. A finding without
   a program that the text accepts and that breaks a promise, or without the question an implementer
   could not answer, is not a finding.
6. Refute by default when unsure: a hole that the current text already closes is "answered", with the
   quote. The earlier reviews often raised questions the text already decided.
7. General, never for one test: a rule that fixes a program's shape is wrong; a rule that states the
   property is right.
