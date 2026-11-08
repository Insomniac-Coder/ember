# The owner's bar for the deep review (2026-11-02), in their words

"are you sure that all of the solutions you've found are in line with the goal of the language and are
the best possible solution without cutting corners? I need perfectly engineering no compromise, safe, high
readability, high performance, goal compliant solution that are in line with the ODRs I have approved
unless you can do something really REALLY better"

"I don't want ANY compromise on quality, along with the things I said before the language is supposed to
be robust and fault tolerant and pleassseeee no specific solutions, this is a programming language and
people will use it to write billions of things so any fix that only targets a fix for a specific case is
not good, generalized solution that meet all the other parameters please"

"and please before you start make sure the lnaguage does not lose its ease of you and readability"

## What that means for every agent

* **Ease of use and readability are not traded for anything.** Judge every rule by the ordinary program
  people write (a game loop, a server handler, a parser): it must stay as short and as plain to read as it
  is today, with no new word, annotation or ceremony in code that does nothing unusual. A rule's cost
  falls on the rare, dangerous program, never on the common one. Each design shows an ordinary program
  before and after the rule; if the after is longer or harder to read, the design is not finished.
* **The compiler carries the complexity, not the programmer.** The owner: "the complexities can be
  abstracted in the complier like we've been doing already". Prefer the design where the compiler works
  it out (reads the bodies, follows the values, infers the fact) over one where the programmer writes it,
  as the language already does: what a function changes is inferred from its body, a `dyn` call is
  checked by the code of each kind that may reach it, a view's source is read from the body. A word
  the programmer must write is a last resort, for what the compiler cannot know.

* **General, never specific.** One rule per principle, which answers every question that shares the cause
  and the cases nobody has asked about yet. A fix that patches one construct, one type or one example is
  not acceptable. A rule with an exception in it is suspect.
* **Safe, robust, fault tolerant.** No program the rule allows reads freed memory, races, or has
  undefined behaviour. A mistake is refused when compiling wherever it can be; otherwise it stops the
  program with a clear message at the line; never silent corruption.
* **High performance.** As close to C as the safety allows: state each rule's cost per operation (time
  and memory) against what C pays.
* **High readability.** Python-like, the language's own spelling reused; no new word where one exists.
* **Goal compliant.** `docs/OWNER-DIRECTION.md`'s goal and checklist, word for word.
* **In line with the owner's verdicts** (`tasks/96/redesign/JOB.md`, 1 to 166), unless something is
  really, really better; then say so plainly, with the evidence, so the lead can put it to the owner.
