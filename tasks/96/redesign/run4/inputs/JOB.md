# The job for the memory-model rework (owner, 2026-10-24)

The first phase 3 missed this: it replaced compile-time checking with run-time checks, so every
design came out slower than C and still not safe. This is the job as the owner set it.

## The goals, all at once

1. **Speed on par with C and Rust.**
2. **Memory safety the same as Rust's, or better.**
3. **Python-like: easy to read and easy to write.**

Rust already shows that 1 and 2 can be had together, with all the checking done when the program is
compiled and nothing paid while it runs. What makes Rust annoying is how it makes you write things
to satisfy its rules, not the safety itself.

## What "Python-like" means (owner, 2026-10-24, after the second phase 3)

"When I said something needs to be python like I meant in terms of readability not an exact 1 to 1
copy. If someone wants to use something like a box instead of having something like Box[dyn pool] you
could have gone for something like no_share(dyn Pool) or find a new way of marking things as no_share.
It also seems like you completely gave up on the idea of safety just to make everything look like
python."

So:
- **Python-like is about readability**, not about accepting Python's programs or Python's meaning as
  they are. A rule may stay; it must read clearly.
- **What a programmer means stays written down**, in readable words: whether a thing has one owner or is
  shared, whether a link keeps its target alive. Example: a one-owner value written as
  `no_share(dyn Pool)`, or a better word found for it, instead of `Box[dyn Pool]`. The compiler does not
  silently decide what the programmer did not say.
- **Safety is never traded for looking like Python.** Every answer shows what it still refuses or checks,
  next to what it now accepts.

## First, the intent of the program (owner, 2026-10-24)

"Understand the intent of the program before deciding anything."

- Every answer starts from what the program is meant to do, stated in one line ("the button adds 1 to
  `game`'s score"), and is judged on whether the code then does exactly that.
- A rule that makes code compile but do something the programmer did not mean is wrong, however safe
  it is. The example: `[CLO-15]` (the lead's, 0.9.9, never ruled on by the owner) makes
  `b.on_click(fn(): score += 1)` capture `score` by move, which for an `int` is a copy, so the button
  changes a private copy and `game`'s score never goes up.

## Keywords that make sense (owner, 2026-10-24)

"Usage of keywords for things that actually make sense, using owned for creating a copy is not
acceptable it hurt readability too, if you want something to have a copy use the copied keyword for it."

- A keyword says exactly what it does, and does the same for every type. Today's `owned fn`, which moves
  some captures and silently copies others (an `int`), is the example of what is not acceptable.
- `owned` means ownership handed over, nothing else: an `owned` parameter, and the one-owner field that
  replaces `Box` (`mesh: owned Mesh`). A copy is called `copied`.
- A keyword goes on the thing it describes and names it. Owner, 2026-10-24, on
  `b.on_click(fn() with copied score: score += 1)`: "it makes it more readable and tells whats going on"
  (placeholder spelling). Not `copied fn(): ...`, which reads as "a copied function" and does not say
  what is copied; what a function copies is named, and anything not named is the real variable.
- **The test for every proposal** (owner, 2026-10-24, on example 8): "what part of
  `State.Walking(path):` tells you you are [a] reference to walking's list?" For every name, point at
  the word in the line that says what it is (the owner's own thing, a copy, or handed over) and whether
  it may change it. If no word says it, the proposal hides meaning and fails. Two proposals failed this:
  plain `path` allowed to change `n`'s list, and `mut path` (says "may change" but not "`n`'s own list";
  in Rust `mut path` is a changeable copy). `ref mut path` passes: "in this case ref mut makes sense".
- **Making keywords** (owner, 2026-10-24): keywords may be made where they are needed, and the workflow
  builds the vocabulary; neither is an excuse to keep the owner doing the work by hand.

## Readability verdicts on run 3's syntax (owner, 2026-10-25)

- **`if Some(ref mut me) = ui:` needs a major rework**: "it is so not readable". This is the form run 3
  uses to open a `weak` link (or any `Option`) and name what is inside. A replacement must say in plain
  words "if the link still points at something, call it `me`" and pass the owner's test.
- **`self` makes no sense to the owner** ("if there is one thing I hate about python is self, it doesn't
  make any sense"): `h(self)` inside a method was unreadable to them. A method's own object needs a
  readable way to be named.
- **The owner's preferred spelling for a function that changes what it is handed is `ref mut` on the
  parameter** (`fn(ref mut u: Ui)`), which says it on the line. Run 3's hole G-05 used a `weak` capture to
  change the same object through a read-only-looking function; the owner: "ref mut would address the
  readability issue too".

## Owner verdicts on run 3's open word questions (2026-10-25)

1. **A call that changes an argument says so** (R18-16): `bump(ref mut a)`, the same words as the
   parameter (`fn bump(ref mut c: Counter):`), as Rust's `bump(&mut a)`. Owner, after first saying the
   function's first line was enough: "I didn't see the rust's way &mut seems okay so during a function
   call saying bump(ref mut a) seems reasonable and okay it tell you that the value is going to be
   touched." A method called on an object (`a.reset()`) is not marked: the object is before the dot and
   the method's first line says what it changes.
2. **No `self`; the object is `this`** (R18-14 and the `self` verdict). Owner: "I want ember to not have
   self, self is a python only quirk ... wherever you want to declare that a function is about to alter
   its own object you can say something like ref mut this, having this as a pointer to self makes more
   sense to me." So: methods do not list their object as a parameter, as in other OOP languages; inside
   a method the object is `this`; a method that changes its object says `ref mut this` on its first line
   (exact position for the vocabulary step, passing the owner's test). Consequences for the next run:
   agreed rows 3 and 7 lose `self` (`fn note(msg: String) changes log:`,
   `fn load_assets() reads renderer -> Array[String]:`) with their meaning unchanged; how
   `ref mut this` and `changes <field>` sit together is for the vocabulary step to settle.
3. **A new word for a changeable link others may share** (R18-22). `ref mut` keeps one meaning, "only
   this name may touch it until it is done" (a parameter, a call). A field that holds a changeable,
   non-owning link to something other fields may also hold (the camera's and the HUD's `target`, run 3's
   AN-52 (b)) gets its own word. Owner: "yeah use a new word for this like sharedref mut or sref mut"
   (the owner's suggested spellings; the vocabulary step settles the final one, next to `shared`, which
   owns and is counted). The rules for it, as the owner checked them: in one thread both holders may
   change it (a number such as `hp` at any time); a list may not change while a loop goes through it,
   whichever holder reaches it; on two threads, only with a lock.

## Error messages are part of readability (owner, 2026-10-24)

"Treat error message fixes as a part of readability and programmer friendliness."

- A refusal is only as friendly as its message. Every refused program gets a message that names what
  was refused in the program's own names, why in one line, and the exact line to write instead.
- A form that is accepted but does not do what its words say elsewhere is refused, with that advice.
- Example 8, approved: plain `State.Walking(path):` then `path.pop()` gets "to change that list
  through `path`, write `State.Walking(ref mut path):`" (today's help talks about `.clone()` and
  indexes); `State.Walking(mut path):`, accepted today with `path` still read-only, is refused with the
  same advice.

## Agreed with the owner, example by example (2026-10-24)

The list to hand to the workflow. Keywords are placeholder spellings unless the owner said otherwise.

| # | Program | Agreed |
|---|---|---|
| 3 | a method changes one field while a loop walks another | the method's first line says what it changes, `fn note(self, msg: String) changes log:`, checked against its body ("okay fair now that is a good change") |
| 4 | `return ref g[0]` inside `with g = scores.lock():` | compiles today and is unsafe: refuse it, message "this points into the locked list, which can change once the lock ends; return a copy, or do the work inside the `with` block" (yes) |
| 5 | `word = line.to_lower().trim()` then `word.len()` | the lowered string is kept until `word`'s last use; returning such a view out of the function stays refused, message "return `String(...)` to hand back your own copy" (yes) |
| 6 | `b.on_click(fn(): score += 1)` in `game` | the function changes `game`'s real `score`; a copy is only made when named: `fn() with copied score:` (liked: "it makes it more readable and tells whats going on") |
| 7 | `init` calls `self.load_assets()` before `assets` is set | the helper says what it reads, `fn load_assets(self) reads renderer -> Array[String]:`, checked against its body and at the call ("look really good") |
| 8 | `Npc.update` shortens the path inside `State.Walking` | `ref mut path` stays; the two error-message fixes above (yes) |
| 9 | keep a snapshot of a list before changing it | `snapshot = copied items` for a copy; `snapshot = items` hands the list over; the error names both ways out (`copied`, or `shared`) ("look really good") |

## Annoying, or an understandable limitation (owner, 2026-10-24)

"You need to make a clear distinction between what is annoying and what is understandable limitation,
deleting the values later on is fine."

- **Fix what is annoying.** Keep an **understandable limitation**, written clearly. Not every rule Rust
  has is an annoyance to remove.
- Example: a list may not change while a loop walks it. Deleting after the loop (a queued destroy, as
  Bevy's `commands.entity(e).despawn()`) is fine and stays; no clever mechanism to allow the change
  inside the loop is wanted.

## For each annoying part of Rust, three acceptable answers

1. **A Pythonic way of writing what Rust does:** the same compile-time guarantee, written the way a
   Python programmer would write it naturally.
2. **A smarter compile-time solution** to the problem Rust's rule addresses, so the rule is not needed.
3. **A new way to address the same problem** (owner, 2026-10-24: "I am not opposed to the idea of
   workflow finding a new way to address the same problem as long as it's clever, easy to read/write
   and is on par with C/Rust in terms of speed while maintaining same or better memory safety"), which
   may be a different mechanism, as long as it is:
   - clever,
   - easy to read and write,
   - on par with C and Rust in speed,
   - the same or better than Rust in memory safety.

## Before anything counts as failed: try to save it

Owner, 2026-10-24: "before considering anything as a failure, a question needs to be asked whether
there is something that can be fixed to make that approach faster, maybe the idea was good but the
implementation was not optimal and if it can't be saved then discard it, if some good parts are found
note them as they might be useful at some point".

When an answer falls short (slower than C or Rust doing the same work, less safe than Rust, or harder
to read or write than the Python version):

1. **Ask whether the idea is good and only the implementation is weak.** Look for what makes it slow
   or unsafe, and try to fix that: a better layout, a check that can be proven away, a cheaper
   mechanism for the same idea. Measure the fixed version.
2. **If it can be saved, keep the saved version**, with its measurements.
3. **If it cannot be saved, discard it**, and say why: the measurement or the counterexample that no
   fix removes.
4. **Note the good parts of a discarded answer.** Anything clever, fast or easy to write in it goes in
   a list of good parts, with where it came from, because it may be useful later.

A failure is never presented as a design choice.
