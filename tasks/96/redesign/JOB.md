# The job for the memory-model rework (owner, 2026-10-24)

The first phase 3 missed this: it replaced compile-time checking with run-time checks, so every
design came out slower than C and still not safe. This is the job as the owner set it.

**The spec this rework produces is named `Ember_v0.9.15_Hardened_1`** (owner, 2026-10-27: "The spec that
will be formed now will be called 0.9.15_Hardened_1 because there have been major changes in this"). It
follows today's development target, `Ember_v0.9.10_Hardened_35`.

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

## Owner verdicts on run 4's open questions (2026-10-25)

1. **Opening an `Option` or a `weak` link: `holds`.** `if cam.target holds p:`,
   `if ui holds ref mut me:`, `if scores.get(name) holds copied v:` (and `while ... holds ...:`). Owner:
   "c, holds reads the best". Replaces run 4's `if p = cam.target:` and run 3's `if Some(p) = x:`.
   **Replaced by 26 (`exists as`).**
2. **`ref mut` keeps others from reading and changing.** While `boss = ref mut enemies[0]` is in use,
   neither `enemies[0].hp = 1` nor `x = enemies[0].hp` is allowed (run 4's R8 (f), Rust's rule); adding
   to the list stays allowed (`boss` is found again). Owner: "love your recommendation".
3. **A `MutSpan` argument says `ref mut` at the call** (`integrate(ref mut ps, vs, 0.5)`), as every call
   that changes an argument does. **`Mutex`, `RwLock`, `Atomic` and channel ends take no word at the
   call**: they exist to be shared between threads, and `ref mut` ("only me for the whole call") would be
   untrue for them; their own "only me" is the lock's block. Owner: "okay got it, go with your
   recommendation".
4. **`shared mut` for an owner that may change the value.** `other = shared items` makes a read-only
   owner; `other = shared mut items` one that may change it. The family: `ref` (someone else's, read),
   `ref mut` (someone else's, change, only me), `sharedref mut` (someone else's, change, others too),
   `shared` / `shared mut` (one of several owners, read / change). Owner: "b, go with shared mut".
5. **A read-only name is a promise that what it reads does not change while it is in use.**
   `boss = enemies[0]` then `enemies[0].hp = 1` while `boss` is still used is refused, with a message
   giving both fixes (change it after `boss`'s last use, or `boss = ref mut enemies[0]` and change it
   through `boss`). A copy of your own is `boss = copied enemies[0]`, which nothing restricts. No silent
   aliasing, no silent copy. Owner: "yes" (to option b).
6. **`changes this` on a method's first line, after the brackets** (replacing `ref mut this` there):
   `fn reset() changes this:` may change anything of its object (`this` is `ref mut` inside it);
   `fn note(msg: String) changes log:` may change only `this.log` (row 3); a method with neither only
   reads `this`. One word, `changes`, for both sizes. Owner: "c, changes this reads better".
7. **`static fn` for a function in a class that has no object** (`static fn starting_at(k: int) ->
   Counter:`, called `Counter.starting_at(5)`), as in Java, C# and C++. Owner: "a, static fn".
8. **`static mut` stays** for a global that may change (`static mut LOG: Array[String] = []`); `static`
   alone never changes. `mut` means "may change" here as in `ref mut` and `shared mut`. Owner: "yes".
9. **`weak` is renamed `weakref`**: "weak is not self explanatory make it weakref" (`keep: weakref Node`,
   `h.keep = weakref first`). It joins the `ref` family. Also from the owner: `a = copied self.keep` "is
   a terrible statement it does not say what kind of copy it is" (it copied the link, not the node).
   `Some(...)` no longer appears in `if`/`while` lines (`holds`); it stays for making a value
   (`best = Some(5)`) and in a `match` branch.
10. **Another link to the same target is `a = weakref self.keep`**; `copied self.keep` is refused, with a
    message giving `weakref self.keep` (another link) or `if self.keep holds n:` then `copied n` (a copy
    of the target itself). The copy word always copies the thing itself, never a link. Owner: "yes".
11. **G-01 is made safe, not refused**: every weakref to a target is counted, so a program that keeps a
    second weakref, drops the first and then checks `if a holds n:` runs correctly ("gone"). The
    program was correct; only run 3's counting was wrong. Owner: "yes".
12. **`copied` is renamed `copy of`**: `snapshot = copy of items`, `boss = copy of enemies[0]`,
    `fn() with copy of score:`, `c = copy of n`. `of` is a keyword only right after `copy`. Agreed rows
    6 and 9 take the new spelling, meaning unchanged. Owner: "copy of".
13. **A change that is silently lost is refused** (AN-10): a method whose only effect is changing a
    temporary that is thrown away at the end of the line (`enemy_at(es, 0).heal()`, where `enemy_at`
    returns a copy). The message gives both ways out: `es[0].heal()`, or make the function give back the
    element itself (`-> ref mut Enemy`, `return ref mut xs[i]`, called `enemy_at(ref mut es, 0)`). A call
    whose result is used (`line.split(" ").next()`) stays allowed. Owner: "yes, a".
14. **`set by` is removed.** Owner: "horrible addition I still think that it will cause more problems
    than solve them and all of that just because you couldn't optimise the existing way of writing."
    A parent link is an ordinary field the program sets itself: `child.parent = weakref this`, then
    `this.children.push(child)`. The job for the next run: make that ordinary way fast through smarter
    compile-time work (prove, where it can be proved, that a parent read needs no "still exists?" check),
    not through a new feature; if it cannot be made fast, report the measured cost honestly. (Run 3's
    tree test: 1.58x of plain C with the check on every step, 1.18x without it.) Question 11 (a walk
    that visits a just-deleted node) went with it: it only existed because of `set by`.
15. **A walk up a tree stops at the first deleted parent.** When a node above is deleted during a walk
    (with everything under it, held in memory until the walk ends, then freed), the walk's check on the
    next parent link finds it gone and stops; it never visits a deleted node; nothing panics. Owner,
    after a step-through picture: "yes remove that horrible addition and make the natural way more
    optimised".
16. **One rule for every loop, paused or not.** A generator (`gen fn`, `yield`, `g.next()`) paused in
    the middle of a list is a loop going through it: changing that list while the generator will still be
    used is refused (`g = d.replay()`, `g.next()`, `d.log.push("open")` while `g` is used later); add after
    the generator's last use. Owner: "a, same rule for every loop".
17. **Functions say what they change, and so do lists of stored functions.** An inline function that
    changes something says so on its first line: `fn() changes LOG: LOG.push(String("clicked"))`. A
    field or list holding functions says what they may change: `handlers: Array[fn() changes LOG]`.
    Plain `fn()` means "changes nothing outside itself", so storing a `changes LOG` function there is
    refused, and calling what such a list holds (`b.click()`) inside a loop over `LOG` is allowed.
    Owner, after seeing the full OLD and NEW programs (`Button`, `LogButton`, `static mut LOG`): "yes".
18. **`changes this` only marks that a method may change its own object; it promises nothing about other
    names.** Owner: "I never said only this touches itself? it's one of the ways you can point at self and
    it helps in differentiating functions that can change this (themselves)". So a turret aimed at itself
    (`if this.target holds ref mut t: t.health -= 1` where `t` is this turret) is allowed with no extra
    check, and the compiler does not assume exclusivity (no noalias) for `this` in a `changes this`
    method. `ref mut` names and parameters keep verdict 2's "only this name touches it".
19. **`owned` becomes `owns`, `shared` becomes `shares`, everywhere, and a memory manager is named with
    `using`.** The owner's own idea: `mesh: owns Mesh using LevelArena`, `mesh: shares Mesh using
    LevelArena`; also `fn add(owns child: Node)`, `other = shares items` (another owner, read only),
    `other = shares mut items` (may change), `fn() with shares count:`, `names: Array[String] using
    LevelArena`. Without `using`, the built-in memory manager. (Run 3's `Box[T, A]` and `Array[T, A]`
    allocator parameter become `using`.) Owner chose the verb everywhere ("go with a"), including in
    front of a value.
21. **Library functions that are handed a function accept one that changes things, if it says so.**
    `scores.for_each(fn(s) changes LOG: LOG.push(s.name))` is allowed, and the call counts as changing
    `LOG` (so inside a loop over `LOG` it is refused); without the word it is refused with the line to
    write. The library writes this as, for example, `fn for_each[C](f: fn(T) changes C) changes C:`.
    Owner: "adding change should change the outcome ... that's how we handle it", then "yes".
22. **A function that changes a variable it captured says so too, for consistency.** Row 6 becomes
    `b.on_click(fn() changes score: score += 1)`; plain `fn()` changes nothing at all, captured
    variables included. Owner: "no for consistency sake lets go with b". Open follow-up: how a class
    that stores such functions (`Button`) says so without naming the caller's variable: settled as
    `handlers: Array[fn() changes captured]` (functions that may change the variables they captured;
    a function changing a global such as `LOG` does not fit). Owner: "yes, changes captured is fine".
    Then the owner's own refinement: **`captured` means exactly one captured variable, `captured...`
    any number** ("I want both captured and captured..."). A function names each one
    (`fn() changes score, lives, LOG:`); a list names globals and covers captured variables with the
    word: `Array[fn() changes captured..., LOG]`. Storing a two-variable function in a `captured` list
    is refused ("changes 2 captured variables, the list allows 1").
23. **The first owner keeps what it could do after sharing.** After `other = shares mut items` (or
    `shares items`), `items` may still change the list; both owners may. One thread runs one line at a
    time, the list rule still refuses a change during a loop through either name, and a `shares` value
    is never used from two threads (that is `thread_shared`, through a lock). Owner: "yes a".
24. **Which arena: the field says the kind, the line that makes the value says which one.**
    `struct Enemy: mesh: owns Mesh using LevelArena` (from some `LevelArena`);
    `boss = Enemy(hp=500, mesh=owns Mesh(points=[0.0, 1.0]) using current)` (from the arena `current`),
    which calls `current.allocate(...)`; the compiler checks `current` is a `LevelArena` and that no
    value outlives the arena it came from. Owner, after the full program (a `LevelArena` implementing
    `Allocator`, two arenas `current` and `next`): "now I get the full picture and yes I agree".
25. **Downcasts: `cast s as Circle` when sure, `cast s when Circle` when not.** `cast s as Circle`
    gives the Circle or stops the program; `cast s when Circle` gives "maybe a Circle", opened with
    `holds`: `if cast s when Circle holds c:`, or `found = cast s when Circle` then
    `if found holds c:`. Replaces the earlier `cast s if Circle` (owner, on finding that `if` then did
    two jobs: "I see if being used twice so technically if has two meanings"). Owner: "go with cast
    when". **The "not sure" half is replaced by 27 (`if s is Circle as c:`); `cast s as Circle` stays.**
20. **`thread_shared` and `sharedref mut` keep their spelling** (owner: "no" to `thread_shares`; "keep
    it as it is" for `sharedref mut`).
26. **`holds` is replaced by `exists as`.** `if leaf exists as n:`, `while up exists as n:`,
    `if n.parent exists as p:`, `if cam.target exists as p:`, `if scores.get(name) exists as copy of v:`,
    `if ui exists as ref mut me:`. `exists` says what the line checks (the link's node still exists, or
    the lookup found something); `as n` gives the name, as in `import x as y`. `as` keeps its two
    existing uses (naming; converting, `x as f32`, `cast s as Circle`) and gets no third. Owner, while
    question 6's tree walk was being explained: "holds has to go as this word fails the readability test
    miserably, we need something else", then "yeah I like exists as here".
27. **A "not sure" downcast is `if s is Circle as c:`**; `if s is Circle:` tests without a name.
    `cast s as Circle` stays for "sure" (stops the program if it is not one); `cast s when Circle`
    (verdict 25) is removed. `is` gets a second job: before a type name it means "is of this kind" (as in
    Swift, Kotlin and C#); before a value it stays "the same object" (`a is b`, `x is None`). Owner:
    "a, is as reads better" (over keeping `found = cast s when Circle` then `if found exists as c:`).
28. **A walk whose visit function may delete nodes holds a `weakref` and checks it every step** (AN-62).
    With `on_visit: fn(Node) changes Node`, the walk is written `up = weakref leaf`,
    `while up exists as n:`, `if n.parent exists as p:` / `up = weakref p` / `else: break`, and stops at
    the first deleted node; a plain-`ref` walk (`cur = ref n`, `while cur.parent exists as p:`) across
    such a call is refused. With a visit function that changes nothing (`fn(Node)`), the plain walk
    stays allowed and pays nothing. Owner: "the cost is justified however we will still try to optimise
    it if possible": the next run looks for compile-time proof that removes the per-step check or the
    link-count updates where it can, and reports the measured cost where it cannot.
29. **A function that changes something under a lock says so too** (area E, reading 1; AN-75, AN-80).
    `fn() with thread_shared log changes Log: log.push("a")`; the same for a value inside a `Mutex`,
    `RwLock` or `Atomic` (v2's R12 "need no word" for lock types is withdrawn for function types; the
    call-site rule of verdict 3, no `ref mut` for them, is unchanged). Gain: one rule with no exception,
    and a call through a function value that would take a lock its thread already holds, inside a loop
    over the guarded list (`with log.lock: for line in log.lines: note()`), is refused when compiling
    instead of hanging when running. Costs nothing while running. Owner: "b".
30. **`changes` says what a function causes to change, whenever that happens; starting a thread
    counts** (area E, reading 2; AN-76). The owner: "who said a change needs to be bound by a moment?
    changes only tells that a function will change something". The library writes
    `pub fn spawn[R, CHANGED](owns f: fn() changes CHANGED -> R) changes CHANGED -> JoinHandle[R]:`
    (today: `pub fn spawn[R](owned f: fn() -> R) -> JoinHandle[R]:`), and `scope.spawn` the same, so
    `main`, which starts threads running `fn() changes RESULTS: worker(1)`, says
    `fn main() changes FRAME, LOG, RESULTS:`. The thread rules (locks, `guarded by`, E7012) still do
    the safety work; a `spawn` inside a loop over `RESULTS` is now refused (start it after the loop).
    Owner: "b". **Narrowed by 52:** a `spawn` in a loop that holds the lock the thread changes under is
    allowed; joining it inside that lock is refused.
31. **A C call counts as possibly running the callbacks C keeps, unless its declaration says it never
    calls back** (area E, reading 3; AN-78). The compiler cannot see inside C, so every C function
    counts as possibly running the kept callbacks, and a call to it makes the changes their first lines
    list: with `register_cb(fn() with owns game changes Game: game.add_point())` kept,
    `for name in game.names: poll_events()` is refused (E3020, "call `poll_events()` before or after the
    loop"); with a kept callback that changes nothing (`fn(): println("event")`) it is allowed. A C
    function that never calls back is marked on its declaration,
    `@ffi(no_callbacks) safe fn sqrt(x: f64) -> f64`; a missing mark only refuses a loop, with a
    message naming the mark. Registering a kept callback counts as making its changes (verdict 30), so
    `setup` and `main` say `changes Game`. Owner: "B", over (c) `@ffi(runs_callbacks)` on the functions
    that do call back (a forgotten mark lets a crash through). A (d) that guessed by `extern` block was
    withdrawn (it treated `unregister` and `register_cb` as calling back and barely differed from (c)).
32. **What a function may do with a parameter is set by the word in front of its name, for every type**
    (area E, point 4; AN-78). `fn setup(game: shares mut Game)` only reads the game (the `shares mut`
    after the colon describes the caller's game, not `setup`'s rights), so making an owner that changes
    it from that name (`with shares mut game`) is refused (E3023), with the lines to write:
    `fn setup(owns game: shares mut Game)`, called `setup(shares mut game)`, and `with owns game`.
    Owner: "A" (over letting a plain `shares mut` parameter change and pass on, which would give the
    same spot two meanings by type).
33. **The iterator method `.copied()` is renamed `.copy_each()`** (readability F-12; AN-46):
    `names = sorted(m.keys().copy_each())` gives a list of the map's keys as strings of its own, kept
    past changes to `m`; E3060's help says `.copy_each()`. Owner: "b" (it says what it copies, and keeps
    the word `copy` of `copy of`).
34. **A `MutSpan` name says `ref mut` in front, like every other name that may change something it
    does not own** (readability F-14; AN-14, AN-16): `d = ref mut tracks[dst_name].as_mut_span()`,
    `fn blend(ref mut dst: MutSpan[float], src: Span[float]):`, called `blend(ref mut d, s)`. R5's
    "exception by type" for `MutSpan` is withdrawn; a plain `d = ....as_mut_span()` or a plain
    `dst: MutSpan[float]` that writes is refused with the line to write. Owner: "b" (one rule, as
    verdict 32 and `boss = ref mut enemies[0]`; the line says "mut" twice).

35. **A function that runs stored functions names their captured variables in the word their type uses**
    (the rule writer's question 1). `class Button: handlers: Array[fn() changes captured] = []`, so
    `fn click() changes handlers, captured:`, as a list of `fn() changes LOG` makes `click` say `LOG`;
    `captured...` only where the list's functions each change several variables at once
    (`Array[fn() changes captured...]`). `captured` describes one function that changes one variable it
    captured, whichever; `captured...` one function that changes several at once; neither counts the
    functions in the list. Owner, on that meaning: "captured only tells that one variable is captured it
    doesnt matter which one it is and captured... means it will accept multiple variables at once", then "a".

36. **A send on a channel is named on the first line, as a change under a lock is** (the rule writer's
    question 2). A channel is a queue two threads share (C++: a `std::queue` guarded by a `std::mutex`);
    `tx.send(v)` pushes onto it, the same kind of change as `log.push("a")` under the log's lock (verdict
    29). So `fn produce(tx: Sender[int]) changes Sender[int]:`, the thread's function
    `fn() with owns tx changes Sender[int]: produce(tx)`, and `fn main() changes Sender[int]:`, which
    starts that thread (verdict 30). It refuses nothing more (no loop walks a channel's queue); it says
    what the function does. Owner: "Yes go with a". Taking values out (`rx.recv()`) was not asked.

37. **Taking values out of a channel through a receiving end a function is given is named too**, as a send
    is (verdict 36): `fn consume(rx: Receiver[int]) changes Receiver[int] -> int:` for a function whose
    body does `for v in rx:` or `rx.recv()`. A function's own local end needs no word, as any own local.
    Owner: "Yes A".

38. **Library functions that call a function right away take it as a template parameter** (the rule
    writer's question 4): `fn for_each[C, F: fn(Item) changes C](owns f: F) owns this changes C:`, and
    `retain`, `inspect` and `sort_by` the same way. `F` names the exact function given, so each lambda
    gets its own copy and the call is direct (C++'s `template <typename F> void for_each(F f)`); the
    shared type written on the parameter, `owns f: fn(Item) changes C` (C++'s `std::function`), is legal
    too and is what keeps functions of different kinds together (`handlers: Array[fn() changes
    captured]`), and a program's own functions may write either. Measured in C++ (g++ 13 -O2, 100
    million items, median of 5, cloud container): template 73.3 ms, `std::function` 265.6 ms
    (`run4/v3/probes/q4_template_vs_function.txt`). Owner: "a".

39. **A function that calls C names what C's kept callbacks change** (the rule writer's question 5).
    With `setup` handing C `fn() with owns game changes Game, LOG: game.add_point()`, a function that calls
    `poll_events()` says so: `fn tick() changes Game, LOG:`, as does every function that calls `tick`; a
    C function marked `@ffi(no_callbacks)` is not such a call. Owner: "tick is a" (the question as written
    in `run4/v3/question-foreign-call.md`).
40. **`captured` and `captured...` cover whatever a line cannot see by name.** A name visible where the
    line is written is written by its name (`Array[fn() changes captured, LOG]` when `LOG` is visible);
    anything not declared in or visible from that scope is covered by `captured` (one) or `captured...`
    (several): a local of the function that wrote a lambda, or a program's `Game` and `LOG` seen from a
    library (`pub fn wait_frame() changes captured...:`). Wherever the word stands for something unseen,
    the compiler fills it in from the whole program (every function value stored into that type, every
    callback C keeps), so a library `Button` holding a program's `LOG`-changing handler still makes
    `b.click()` inside a loop over `LOG` refused. Owner: "Then change what captured and captured... [mean]
    and both should able to pull any variables that need to be captured for change", "Any variables that
    need to be captured for a change if they are not known to function or declared within that scope
    right?", then "yes".
41. **A named change list** (the owner's own proposal): `world: changelist = (Game, LOG)` at module level,
    used as `fn tick() changes world:`. Strict: a function may say `changes world` only if it changes
    everything in it. Owner: "we can provide a way to declare the change list and then we can just mention
    that list after changes <list>", then, on whether a list may name more than a function changes: "no it
    has to be strict".

42. **Every `ref mut` name may be given another ref; `const ref mut` never is** (areas A and B's
    questions). `r = ref mut a`, `r = 5` writes `a`, `r = ref mut b` points `r` at `b`; the same for a
    span, `d = ref mut a.as_mut_span()` then `d = ref mut c.as_mut_span()`; a `ref mut dst:
    MutSpan[float]` parameter may point the caller's span elsewhere. `const` keeps one meaning, the name
    itself stays as it is: on a value it cannot be changed, before `ref mut` it cannot be given another
    ref (`r = const ref mut x`, `fn blend(const ref mut dst: MutSpan[float], src: Span[float]):`), as
    C++'s `const int` and `float* const`. For every `ref mut` name, not only spans. Owner: "How about we
    add ref mut and const ref mut", "Const should either mean that it cannot be changed or cannot be given
    another ref", then "yes, every ref mut name".

43. **A call that may take a lock this thread already holds is refused when compiling** (area E's
    question 2). `flush` holds `log`'s lock (`with log.lock:`) and calls `note()`, a `fn() changes Log`
    whose `push` takes the same lock: in C++ `m.lock(); m.lock();`, the program freezes. Ember refuses the
    call (E7018: "call `note()` after the `with log.lock:` block"), with no loop needed; nothing is checked
    while running. A function that changes a different `Log` is refused too (the compiler cannot tell
    which). Owner: "B".

44. **`guarded by` stays, and follows the `Mutex`'s rules** (area E's question 1). `lines: Array[String]
    guarded by lock` is a `Mutex` written as ordinary fields (one lock may cover several fields, no extra
    struct, no guard name), shown beside the `Mutex`-only version (`struct LogData` inside
    `Mutex[LogData]`). So a `guarded by` field may be changed through a plain name inside its lock, with
    no word at the call, and the change is named on the first line, as for a `Mutex` (verdicts 3, 29):
    `fn add_line(log: Log) changes Log: log.push("hi")`; `fn start(log: thread_shared Log) changes Log:`.
    Owner: "Keep guarded by and the same way as mutex".

45. **A lambda written with `=>` says what it changes after its parameters, as one written with `:` does**
    (area D's missing word). `enemies.retain(fn(e) changes removed => keep(e, ref mut removed))`, where
    the lambda gives back a `bool` and adds to `update`'s `removed`. Owner: "A".

46. **A `dyn` type may say what its values change, as a function type does** (the rule writer's question
    3). `xs.iter().inspect(fn(x) changes LOG: LOG.push(String("saw")))` given to
    `fn show(owns it: owns dyn Iterator[Item = int] changes LOG) changes LOG:`; without `changes LOG` on
    the `dyn` type it is refused with that line. Owner: "B".

47. **A value made in an arena carries a pointer to its arena, accepted for now; lists the same way** (the
    rule writer's question 6). `lvl = Level(mesh=owns Mesh(verts=[0.0]) using current)`: the mesh keeps a
    pointer to `current` (8 bytes per arena-made value, as C++'s `std::pmr` and Rust's `Box<T, &A>`), so it
    goes back to the right arena when it grows or ends; a list is made in an arena the same way,
    `[String("a")] using current`. Not measured; the measuring phase looks for ways to drop the pointer
    (an arena that never frees single values; memory laid out so a pointer finds its arena). Owner: "Yes".

48. **A function type written several times is named once with `type`** (the survey: AN-76's `Button`).
    `type Handler = fn() changes LOG`, then `handlers: Array[Handler] = []` and
    `fn on_click(owns f: Handler) changes handlers:`. Owner: "I like option 1".
49. **A long function type in a first line is named once with `type`, its change list as a parameter**
    (the survey's longest line, AN-05). `type DeathHandler[C] = fn(ref mut Array[Enemy], int) changes C`,
    then `fn update[C](ref mut enemies: Array[Enemy], on_death: DeathHandler[C]) changes C:`, over
    writing each parameter on its own line. Owner: "I meant A".

50. **Type parameters go on a line of their own above the declaration, for every generic function,
    method, struct and class** (the survey: the library's first lines). `[C, F: fn(Item) changes C]` over
    `fn for_each(owns f: F) owns this changes C:`; `[T: Add[Output = T] + Default]` over
    `fn sum(xs: Array[T]) -> T:`; `[T]` over `struct Pool:`. Nothing is dropped; a type is used as today,
    `Pool[int]`; a lambda has no type parameters. Enums, interfaces and `type` aliases were not named and
    keep the brackets after the name. Owner: "Template parameters on their own line seems good", "All
    generics", "Structs too and even classes".

51. **A long lambda may be written inside the call or as a named local function passed by its name; both
    are allowed** (the survey: thread and C lambdas). `a = thread.spawn(fn() with thread_shared log
    changes Log: log.push("a"))`, or `fn push_a() with thread_shared log changes Log:` / `log.push("a")`
    then `a = thread.spawn(push_a)`; a local function captures as a lambda does. Owner: "I think both ways
    should be permissible".

52. **A thread may be started inside a loop over what it changes when it makes that change under the
    lock the loop holds; joining it inside that lock is refused** (area E's note). With `RESULTS` guarded
    by `RESULTS_LOCK` and the loop inside `with RESULTS_LOCK:`, `thread.spawn(fn() with copy of r changes
    RESULTS: worker(r))` in the loop is allowed (line corrected, verdict 65) (the thread waits for the lock until the loop ends), as Rust
    allows it; `t.join()` inside the lock, which freezes (each waits for the other; Rust compiles it and
    it hangs, `run4/v3/probes/rust_spawn_in_loop.txt`), is refused when compiling (E7018). Owner: "B".
53. **A function that uses a view parameter after changing what it changes is caught at the call, by a
    fact the compiler works out from its body** (safety validation (b), hole 1: `run4/v3/safety/findings.md`
    section 1, check-V3). With `fn note(s: StrView) changes LOG:` whose body is `LOG.clear()` then
    `println(s)`, the compiler works out from `note`'s body that it uses `s` after changing `LOG` (its own
    lines, and the same fact of the functions it calls, a function value's call included: it reads every
    function the program can put in that value), keeps that fact with `note` (in the package's interface file, R17), and checks it at each call:
    `note(LOG[0])` is refused, the message giving both fixes (`note(copy of LOG[0])`, or use `s` before
    `LOG.clear()` in `note`); `note("bye")` is allowed; a `note` that prints before it clears is allowed
    with `note(LOG[0])`. Nothing is checked while running. A change of the fact by reordering lines inside
    a library function is reported as a change to its interface (R17). Owner: "C", over (a) checking only
    the `fn` line (it refused the print-then-clear `note` called with `note(LOG[0])`, the owner's own
    counter-example) and (b) checking only inside the function (it refused the clear-then-print `note`
    called only with other text). Corrected 2026-10-27: the lead's record had added "a call whose body it
    cannot see counts as doing so", which was not the owner's decision and refused verdict 28's walk at
    `this.on_visit(p)`; only a C function's code is truly unseen (R12's own rule).
54. **Verdict 53 covers a method's object too** (safety validation (b), hole 2: C1's G19). `this` counts like
    a view parameter: with `fn hit() changes hp, ENEMIES:` whose body is `ENEMIES.clear()` then
    `this.hp -= 1`, the compiler works out that `hit` uses `this` after changing `ENEMIES`, and refuses
    `ENEMIES[0].hit()` (the enemy is part of `ENEMIES`); the message says to change `this.hp` before
    `ENEMIES.clear()`. A `hit` that uses `this` only before the clear is allowed. Owner: "yes, cover it
    too".
55. **A function that may destroy objects it owns says so in its type** (safety validation (b), hole 3:
    V4's note on R10 point 1). R12's "its own state is itself, and a change to it needs no word" no longer
    covers ending an object of a class type (one a `weakref` elsewhere may point at): that is named by the
    object's type, `reset = fn() with owns players changes Player: players.clear()`. The compiler then
    sees that `reset()` may destroy players, and a view opened from a link (`if cam.target exists as p:`)
    is not used after it: `println(p.hp)` after `reset()` is refused (use `p` before `reset()`, or open the
    link again after it). Nothing runs while the program runs. Owner: "b", over (a), protecting such views
    with a count around every call through a stored function. The owner then asked why `with` is needed
    (the capture list, verdict 22 and row 6).
56. **An object counts as ended the moment its last owner lets it go, before its `drop` runs**
    (safety validation (b), hole 4, first part: C1's G14, W4's question 7). A `weakref` opened during its
    `drop`, or during the `drop` of anything it owns (a child's `drop` opening `this.parent` while the
    parent is being destroyed), finds it ended, so the `if ... exists as p:` block is skipped and the
    half-destroyed object is never read, nor counted again (the count pair that would end it a second
    time cannot arise). As C++'s `weak_ptr` and Rust's `Weak`. Ending the link itself
    (`this.parent = None`) stays allowed: it only subtracts the link's weak count in the target's header,
    which the weak count keeps in memory (section 0). Owner: "a".
57. **Destroying an object counts as calling its `drop`** (safety validation (b), hole 4, second part: C1's
    G12). Anything that may end an object (a list cleared, a value replaced, a local ending with its
    function) counts as a call of the object's `drop`, with what its `fn` line says it changes. So with
    `class Enemy:` whose `fn drop() changes LOG:` pushes to `LOG`, `enemies.clear()` inside
    `for line in LOG:` is refused (E3020: clear after the loop), as C++'s `~Enemy()` pushing to the vector
    a range-for walks breaks the loop; and, by verdict 30, a function that may destroy an `Enemy` says
    `changes LOG`. A `drop` may still change things outside its object (a log line, leaving a list).
    Owner: "a that makes sense", over (b), a `drop` changing nothing outside its own object.
58. **`mem.forget` is removed** (safety validation (b), hole 5's discussion). The owner, on what forgetting an
    owner is good for: "what is so good about leaving a count higher and causing a leak", then "REMOVE
    FORGET". Ending a value without its clean-up leaves a shared count too high for good (a leak), and was
    the cheap way to roll a count over (hole 5); handing an object to C goes through the `Retained` token
    and the checked trampoline instead. To do when the rules are next revised: the vocabulary's
    `mem.forget` row (and `@must_drop`'s "no `forget`"), and what in today's library and tests uses it.
59. **A count of owners is checked before it rolls over** (safety validation (b), hole 5: C1's G16). The
    owner count is a 32-bit number in the object's header (R9); with `p = shares Player()` and
    4,294,967,296 more owners pushed into a list, it would roll over to 1, and one owner ending would
    delete the player while the others still point at it. Before each +1 the count is compared with its
    maximum, and the program stops with "too many owners of a `Player`" (as Rust's `Rc`); one compare per
    added owner, no extra memory. Owner: "a", over (b), 64-bit counts (every shared object 4 to 8 bytes
    bigger). The weak count (also 32 bits) gets the same check: owner, "yes, same check".
60. **Anything that can break a safety promise is possible only inside an `unsafe` block** (the owner, after
    `mem.forget` turned out to need no `unsafe`: "for unsafe stuff it should only be possible in an unsafe
    block"). The inventory of escape hatches and leaky features (planned next, for the owner's approval)
    checks every one against this: today's marked ones (`unsafe` blocks and `unsafe fn`, raw pointers,
    `assume_disjoint`, the `@ffi(...)` promises written in an `unsafe extern "C":` block) and every
    feature that is not marked but can leak, skip a check, or reach memory the rules cannot see.
61. **A function type says whether any thread may run its functions: `thread_shared fn()`** (safety
    validation (b), hole 6a: C1's G26). A function that holds a one-thread value (`with shares mut count`)
    stored in an object threads share (`bus = thread_shared Bus()`) and run by another thread
    (`bus.fire()`) would let two threads change `count` at once. A list or parameter whose functions may
    run on any thread says so in its type: `handlers: Array[thread_shared fn() changes Counter] guarded by
    lock = []`, `fn on_event(owns f: thread_shared fn() changes Counter) changes handlers:`; such a
    function may hold `thread_shared` values or copies, never a `shares` or `shares mut` value, and a
    `thread_shared` object's fields that hold functions use such types. So `bus.on_event(fn() with shares
    mut count changes Counter: count.n += 1)` is refused (make the counter shareable:
    `count = thread_shared Counter()` with `n: Atomic[int]`). As Rust's `Send`. Owner: "b", over (a), the
    compiler finding it from the whole program with no word in `Bus`.
62. **A C callback that removes its own token stays alive until it finishes** (safety validation (b), hole
    6b: C1's G27). With `callback = fn() with owns my_game changes Game:` whose body is
    `my_game.token = None` then `my_game.add_point()`, the first line destroys the token (the field owns it),
    and the compiler-inserted drop runs the generated `callback_release`, which unregisters the callback and
    frees its block, where `my_game` lives, while the callback is still running; the second line would read
    `my_game` from freed memory. Now: when a token is destroyed while its callback is running, C forgets
    the callback at once, but `drop_env` and `ember_free` of its block wait until the callback returns, so the
    second line works, as the program reads (the same protection R12 gives an Ember function that removes
    itself). Owner: "a", over (b), refusing a callback that destroys its own token. What finally explained
    it: the generated C (`=` inserts the old token's drop, which ends in `callback_release`), then "my_game
    is the link and lives in the callback block; the programmer still calls add_point because nothing
    tells them line 1 kills my_game".
63. **A change through a link is named with `changes T` even when the link came in as a `ref mut`
    parameter** (safety validation (b), open question 1, raised by W2, V2, V3, W4, V4, W5 and V5: R5's
    "`changes T` for objects it changes through a link" against "a change through a `ref mut` parameter
    needs no clause"). `fn tick(ref mut c: shares mut Clock) changes Clock:` with `c.frame += 1`; the
    `ref mut` parameter alone is not enough when what it reaches is behind a link. R5's "needs no clause"
    is narrowed to a `ref mut` parameter that is not a link. Owner: "b", over (a), `ref mut c` alone.
64. **A change through a link that is the function's own local is named too** (open question 2: R5's "a
    change to an own local needs no clause" against "`changes T` through a link"). `fn main() changes
    Clock:` with `clock = shares mut Clock()` and `clock.frame += 1`. So a change through a link always
    says `changes T`, whatever holds the link (verdicts 63 and 64). Owner: "b".
65. **Verdict 52's example line takes its own copy of the loop's variable**: `thread.spawn(fn() with copy of r
    changes RESULTS: worker(r))`. `r` changes every round and a `thread.spawn` thread may still run in a
    later round, so it keeps its own copy (a `thread.spawn` function names what it keeps; the loop's own
    variable needs `with copy of` for scoped tasks too; a module variable such as `RESULTS` needs no word).
    Owner: "yes, correct it".
66. **`exists as copy of v` stays, and the two-line form is valid too** (open question 4). Both
    `if scores.get("ana") exists as copy of v:` and `if scores.get("ana") exists as v:` followed by
    `mine = copy of v` are allowed. Owner: "let it be the way it is as long c is also valid". (Whether the
    `v` of `exists as copy of v` may be changed is asked next.)
67. **The `v` of `exists as copy of v` may be changed**: it is a copy of your own, as `boss = copy of
    enemies[0]` then `boss.hp = 1`; the map is untouched. A name `exists as` makes without `copy of` still
    never changes. Owner: "yes a".
68. **A `Copy` type always works with `copy of`** (open question 5): `@derive(Copy)` gives `Clone` too, as
    in Rust, so `b = copy of a` for a `Point` marked only `Copy` makes a copy of your own; E2116 is not
    given for it. Owner: "allow making copy of copy".
69. **`drop` never writes `changes this`** (open question 6). Changing its own object is a given for
    `drop`, since it destroys it, so its first line names only what it changes outside `this`:
    `fn drop():` for one that only reads or cleans up its object, `fn drop() changes LOG:` for one that
    also pushes to `LOG` (verdict 57). Owner: "make exception for drop since it's a given that it will
    change this as it destroys it so no changes this for it unless it changes something outside the
    scope of this". (Writing `changes this` on a `drop` anyway: the message says to remove it.)
70. **A call to a `const ref mut` parameter writes the same words** (open question 7): with
    `fn blend(const ref mut dst: MutSpan[float], src: Span[float]):`, the call is
    `blend(const ref mut d, s)`, as R4's "the same words as the parameter". Owner: "b", over (a),
    `blend(ref mut d, s)`.
71. **A lambda's parameter says what it receives, matching the function type** (open question 8: E2020
    against AN-56). Where the type hands the value over (`fn(owns String)`), the lambda writes
    `fn(owns s) => println(s.len())`; `fn(s) => ...` there is refused (E2020), the message giving the
    `owns s` line. Owner: "b".
72. **The loop rule covers everything a loop goes through: lists, maps, sets, strings** (open question 9).
    `for k in counts.keys(): counts.insert(...)` is refused at the change, E3020, with the list's message
    ("change it after the loop"), not at the next use of `k` (E3060). Owner: "a".
73. **A message's fix must not run into another error; where two errors fit one line, the one whose fix
    makes the program compile is shown** (open question 10, E3020 against E7018). In
    `with log.lock: for line in log.lines: note()`, with `note` a `fn() ... changes Log` that pushes to
    the log, the lock error (E7018: "move `note()` after the `with log.lock:` block") is shown, not the loop
    error, whose fix (after the loop, still inside the lock) would still freeze. The owner: "a rule is
    supposed to tell how to fix an issue not how to run into another issue". This is R18's "the line a
    message gives compiles" made the tie-breaker for every pair of errors on one line (E7007 against
    E3020, E3060 against E3022, and the rest).
74. **`owns` is written on the line that makes a value for an `owns` field** (open question 11: verdict
    24's spelling against R3's "no `owns` before a value its place hands over anyway"). With
    `mesh: owns Mesh using LevelArena`, `boss = Enemy(hp=500, mesh=owns Mesh(points=[0.0, 1.0]) using
    current)` is required, and the line without `owns` is refused ("write `mesh=owns Mesh(...)`"); the
    same for an `owns` field with no arena (AN-40's `Entity(mesh=Mesh(...))` becomes
    `Entity(mesh=owns Mesh(...))`). R3's rule is narrowed accordingly. Owner: "a", over (b) both allowed
    and (c) never written.
75. **A function that changes an `Atomic` it captured names it by the variable** (open question 12):
    `count_hit = fn() changes hits: hits.fetch_add(1)` for `main`'s real `hits = Atomic(0)`; naming by
    type (`changes Atomic[int]`) is for an `Atomic` reached through a link, where no name is visible.
    Owner: "a".
76. **"Can see by name", for `captured`, means declared or imported in that file** (open question 13).
    `class Button: handlers: Array[fn() changes captured] = []` in `game/button.em`, which does not import
    `game.log`, may hold a function that changes `LOG`; `captured` covers it there, and the compiler
    fills it in from the whole program (verdict 40). A name declared or imported in the file is written
    by name. Owner: "a".
77. **Adding to an object's field from several scoped tasks is a reduction too** (open question 14: the
    reduction rule's `total += e` against E7007 naming `stats.total`). `scope.spawn(fn() with copy of x
    changes stats: stats.total += x)` gives each task a private sum, added together at the end of the
    scope (always the right total); nothing reads `stats.total` before then. A task that reads the
    current value (`if stats.total < 10: stats.total += x`), or a `thread.spawn` thread, still needs an
    `Atomic` or a lock (E7007). Owner: "okay so for reduction it should be fine" (after Rust's way, an
    atomic, a lock or Rayon's explicit `sum`, was shown). A whole-number total reached through two
    `sharedref mut` links to one object is one too (recheck U10): with `a` and `b` two `Watcher`s linking
    one `p = Player(hp=10)`, `scope.spawn(fn(): a.target.hp -= 1)` and `scope.spawn(fn(): b.target.hp -=
    2)` leave 7 every time; `a.target.hp = 5` from a task still needs a lock (E7007). Owner, asked on
    2026-10-27: "yes", over refusing it. It was decided on whole numbers
    (`xs = [1, 2, 3]`, "always 6"); a float total a task adds to more than once (recheck U7) was a
    separate question: verdict 92.
78. **A field of a value may be handed over on its own, as in Rust** (open question 15, R2's "partly
    handed over"). `names.push(p.name)` moves `p`'s name into the list; `p.hp` stays usable; `p.name` is
    refused until it gets a new value ("`p.name` was handed over to `names` on line N; give it a new
    value first, or keep a copy: `names.push(copy of p.name)`"), and so is `p` used whole. Owner: "a", over
    (b), refusing a field hand-over. (For a class object, which other names may reach, taking a field out
    stays refused, as today's EXP-6: others would see the hole; `mem.replace` or `mem.take` is the way.)
79. **`copy of` a struct that holds links copies the links** (open question 16, C1's G17): for
    `struct Team: captain: shares Player, scout: weakref Player`, `t2 = copy of t` is a new `Team` pointing
    at the same player, with one more owner in `captain` and one more link in `scout` (as copying a C++
    struct holding a `shared_ptr` and a `weak_ptr`, or Python's `copy.copy`). Fields the struct owns alone
    are copied themselves. Owner: "a", over (b) a deep copy of the player and (c) refusing it.
    (The owner reconsidered a deep copy for `shares` fields, then kept (a): "a next question".)
80. **A function that outlives the function that made it takes the variables it keeps with `with owns`**
    (open question 17: R12's placement of a real variable in the function's own state, against E3026).
    `return fn() changes n: n += 1` in `make_counter`, whose local `n` ends when it returns, is refused
    (E3026), the message giving `return fn() with owns n:`. With `with owns`, `n` is the function's own
    state, and changing its own state needs no `changes` word (R12), so the returned type is a plain
    `fn()`: `fn make_counter() -> fn():`. Without `with`, a function only borrows (the owner's own
    summary of `with`). Owner: "b", over (a), the compiler moving `n` in by itself as Python does.
81. **Adding to a list does not end a link to an object in its own block** (open question 18: R8 (b)'s
    "any reshape ends a view into what the list owns" against AN-52). With `players: Array[Player]` (class
    objects, each in its own block) and `w = Watcher(target=sharedref mut players[0])`,
    `players.push(Player())` moves at most the list's addresses, not the players, so `w.target.hp -= 1`
    after it is allowed. A link to a value stored inside the list's own block (an inline struct) is still
    ended by a push. Removing, clearing or replacing still ends links to what may be destroyed. The owner
    also noted that nothing on `players` says its players can be shared: a `sharedref mut` link owns
    nothing, so the word sits on the line that makes it, as `ref mut enemies[0]` does. Owner: "a".
82. **A name opened from a link is not used across a call that may destroy what it reads** (open question
    19: R8 (b) against R10). With `if cam.target exists as p:` and `remove_all(ref mut players)` emptying
    the list that owns the player, `println(p.hp)` after the call is refused ("`remove_all` may destroy the
    player `p` reads; use `p` before that line, or open the link again after it"); nothing runs while the
    program runs. R10's keep-alive (deferred-drop region, count pair) is no longer used for such a call; it
    remains for a `thread_shared` object reached through a `weakref` (R10 point 4). As verdict 55. Owner:
    "a", over (b), keeping the player alive with a count while `p` is used.
83. **`with` is dropped from what a function keeps** (step 3's readability check, 2026-10-26). The word
    after `fn(...)` says it on its own: `return fn() owns count -> int:`, `b.on_click(fn() copy of score:
    println(score))`, `reset = fn() owns players changes Player: players.clear()`, `thread.spawn(fn() copy
    of r changes RESULTS: worker(r))`. Row 6 and every verdict's line take the new spelling, meaning
    unchanged. Owner, on readability over the earlier agreement: "IT DOES NOT READ WORSE COPY OF SCORE IS
    DESCRIPTIVE", then "YES" to dropping it everywhere.
84. **`owns` hands over every type the same way, numbers included** (step 3's readability check: verdict
    80's `owns n` on an `int` against E2085). After `owns count`, the one who gave it can no longer use
    `count`; the receiver uses it under the same name. `copy of` copies, and the giver keeps its own, for
    every type. E2085 (`owns` on a `Copy` type) and its "write `copy of`" messages are removed; verdict 80's
    `return fn() owns count -> int:` stands as written. The machine copies the number either way, as fast
    as C. Owner: "EVERY OTHER THING HAS TO BE OWNED TO TRANSFER WHY THE FUCK INT GETS THE SPECIAL
    TREATMENT?", then "yes".
85. **`owns` is written on the making line wherever the place owns the value, also inside an `Option` or
    a list** (step 3's readability check: two R15 tests against AN-41, AN-40 and the R3 and R14 tests).
    With `left: Option[owns Tree]`, `t = Tree(value=1, left=Some(owns Tree(value=2)))`; with `pools:
    Array[owns dyn Pool]`, `r.pools.push(owns IntPool(items=[1, 2]))` (shown with the same choice). The
    line without it is refused, as verdict 74. Owner: "a", over (b), the word only on an `owns` field
    itself.
86. **`changes` and `reads` are removed from the language; the compiler works out what every function
    changes and reads from its code** (step 3's readability check, after open points on what follows
    `changes`: a type or a name). `fn note(msg: String): LOG.push(msg)` needs no word; a loop over `LOG`
    that calls `tick()`, which calls `note`, is refused as before, the message naming the chain ("`tick()`
    adds to `LOG` (through `note`, line 4) while this loop walks `LOG`; call `tick()` after the loop").
    The same for function values, function types (`Array[fn()]`), methods (`fn note(msg: String):`,
    `fn load_assets() -> Array[String]:`, `@cached fn radius() -> f64:`) and libraries (their code is
    read too). Safety and speed are unchanged: the same lines are refused, nothing runs at run time.
    Costs the owner accepted: the `fn` line no longer shows what a function changes; an edit inside one
    function (or a library update) can make a far-away line stop compiling, the error naming the chain;
    rows 3 and 7 lose their words. `@ffi(no_callbacks)` stays for C functions, whose code cannot be read.
    Every verdict that spells a `changes` or `reads` clause keeps its meaning without the clause (2, 6,
    17, 18, 21, 22, 28-32, 35-41, 43-46, 48-51, 53-55, 57, 61-65, 69, 73, 75, 76, 80, 82, 83, 85); the
    ones only about how to write it (22's `changes captured`, 29, 30's change list, 40, 63, 64, 69, 75,
    76) are void. Owner: "THIS CHANGES THING IS ANNOYING AS FUCK I THINK IT NEEDS TO GO AWAY", then, on
    `reads`, "DONT NEED IT" and "I ALREADY SAID YES".
87. **Verdict 53's call check covers a `ref mut` parameter too** (step 3's question 8). With `fn
    rename(ref mut s: String):` whose body is `LOG.clear()` then `s = String("renamed")`,
    `rename(ref mut LOG[0])` is refused (it would free `"hello"` twice and write into freed memory), the
    message giving `mine = copy of LOG[0]` then `rename(ref mut mine)`, or changing `s` before
    `LOG.clear()`. Owner: "I am not concerned about log[0] case that error is understandable".
88. **The same whole variable under two names inside a function is allowed; the compiler makes a copy of
    the function for that call with one name** (the owner's own proposal, step 3's question 8). With the
    same `rename` but `ref mut s: Array[String]` and the call `rename(ref mut LOG)`, `s` and the global
    `LOG` are one list: the call is allowed (it frees nothing: `clear()` empties the list, `s = ...`
    fills it again), and the compiler compiles that call against its own copy of `rename` in which `s`
    is replaced by `LOG`; other calls use the normal copy, where a `ref mut` parameter is the only way
    in (as fast as C either way; the cost is one more copy of the function per such pattern). Only where
    the compiler knows when compiling that the names are the same whole variable; items whose positions
    are not known (`ref mut log[i]` beside `ref mut log[j]`, or beside `log` itself) stay refused, since
    they may lead to the `LOG[0]` problem. Two `ref mut` parameters proven to be the same whole variable
    are covered the same way: `fn add_both(ref mut a: int, ref mut b: int):` (`a += 1`, `b += 1`,
    `println(a)`) called as `add_both(ref mut x, ref mut x)` runs a copy with one variable and prints 2
    (owner: "if it can be proven that ref mut x and ref mut y are both pointing towards the same variable
    then the function gets simplified to be just one variable and it prints 2"). Rust refuses all of
    these (E0499). Owner: "would it be a
    problem if ... the language replaces all the different references with one single name since a
    function is using the same variable with different name?", then "this should only be for log and
    ref mut s which points at log", with `log[i]`, `log[j]` "still remains an error as we don't know the
    values of i and j".
89. **A thing whose changes go only through its lock may be reached through any number of names, as in
    Rust** (step 3's question 9; `check-V6.md` item 10). With `class Log:` holding `lock: Lock` and
    `lines: Array[String] guarded by lock`, `dump(log, fn() thread_shared log: log.push("seen"))`, whose
    `dump` reads the lines under `with log.lock:` and calls `note()` after the block, is allowed: the lock
    keeps the two ways in apart (Rust accepts the same with `&Log` and a `Mutex`). `note()` called inside
    the `with log.lock:` block stays refused when compiling (E7018: "call `note()` after the `with
    log.lock:` block"), which Rust only meets while running, stuck for ever. Owner: "a", over (b),
    refusing it as one log reached two ways.
90. **A loop over a map may change the values stored under its existing keys** (step 3's question 10;
    area A's AN-12, R8 (c) against verdict 72). `for k in counts.keys(): counts[k] *= 2` doubles every
    count; `for k, v in names: names[k] = String("bob")` is allowed and `v` then reads `"bob"` (`v` names
    the value's place, which stays; today's `Map.insert` replaces an existing key's value in place,
    `std/src/collections.em:603-613`). Adding or removing keys inside the loop stays refused (verdict 72),
    and so does a key not known to be in the map (`counts["kiwi"] = 1`). A window into the replaced
    value's old letters (`w = v.trim()` before the replacement, used after it) stays refused, as
    everywhere (R8 (b)). Python allows the same; Rust refuses it (`values_mut()`). Owner: "a", over (b),
    refusing any change of the map while the loop walks it. **Reversed by verdict 95.**
91. **`into_raw()` stays, usable only inside `unsafe`, built into the compiler; `mem.forget` stays removed**
    (step 3's question 11; the rule writer's item 1). A C function declared as keeping an object
    (`@ffi(param(widget, owned, one))` with `owns widget: owns Widget`) is given one by an ordinary
    hand-over, `widget_take(first)`: the name ends and Ember frees nothing, no word needed. For a C
    function whose declaration cannot say so (a plain pointer: `fn set_user_data(p: *mut Widget)`, a C
    function with `...`), `second.into_raw()` gives the address and stops Ember freeing the object (as
    C++'s `unique_ptr::release()`); the compiler does that step itself, so nothing else can skip a value's
    clean-up. Outside `unsafe:` it is refused ("`into_raw()` stops Ember from freeing the widget, so a
    mistake leaks it or frees it twice; write it inside `unsafe:`"). Verdict 60's rule. Owner: "can you
    make it so that into_raw() is only available for use in unsafe blocks?" (Correction to verdict 58's
    record: the lead had said handing an object to C "goes through the `Retained` token"; `Retained`
    keeps an object on Ember's side while C holds a callback, it does not give an object away.)
92. **A reduction of decimal numbers is allowed, and may differ in the last digit from adding one by one**
    (recheck U7; AN-71's refused `total_of`). With `for part in [[0.1], [0.2, 0.3]]:` and
    `scope.spawn(fn() copy of part: for x in part: total += x)`, each thread keeps its own total (0.1 and
    0.5) and they are added at the end (0.6), where one by one gives 0.6000000000000001; the same answer
    on every run, as Rust (Rayon) and C (OpenMP). AN-71's refusal and its test go. Owner: "YES" to "is
    that acceptable?", over refusing it.
93. **A task may take its own copy of the loop's text or look at it where it is: `copy of t` or `ref t`**
    (area E's item 1, AN-71; verdict 65 gave only the copy). In `for t in texts:`, `scope.spawn(fn() copy
    of t: total += t.split_whitespace().count())` gives each task its own copy (one allocation and copy
    per text), and `scope.spawn(fn() ref t: ...)` lets the task look at the text `t` is on in that round,
    with no copy (as Rust's scoped threads borrow `&t`): safe because `texts` cannot change until every
    task has ended at the end of the `with scope` block. `ref t` is refused only where it would clash:
    the task could still look after the text is changed or gone (a `thread.spawn` thread may outlive the
    loop), the message giving `copy of t`. Owner: "both ways should be permissible right? like if you
    want to pass a copy that's fine and if you want it to look at the same variable that is also fine as
    long as there is no clash".
94. **`thread.spawn` takes a `thread_shared fn()`** (area E's item 4; verdict 61 applied to the library's
    own line). `pub fn spawn[R](owns f: thread_shared fn() -> R) -> JoinHandle[R]:`; calls are written as
    before (`h = thread.spawn(fn() copy of r: worker(r))`), and a function holding a one-thread owner is
    refused by verdict 61's general rule (E7015) instead of a rule of `spawn`'s own. `scope.spawn` keeps
    a plain `fn()`: its tasks may use the caller's own variables (`total`, `ref t`), since they all end
    with the `with scope` block. Owner: "yes".
95. **Rust's way for lists and maps: while a loop walks one, nothing changes it through its own name;
    changes go through the loop's own handle** (area C/W3's question on `xs[0] = 5` inside `for x in xs:`;
    reverses verdict 90). `for x in xs: xs[0] = 5` is refused (E3020), and so are `for k in
    counts.keys(): counts[k] *= 2` and `for k, v in names: names[k] = String("bob")`, whatever the change.
    The loop's handle may change the item it is on: `for ref mut x in xs: x = 5` (Rust's `iter_mut()`,
    `[5, 5, 5]`), `for k, ref mut v in counts: v *= 2`, `for k, ref mut v in names: v = String("bob")`.
    Owner, on changing what a loop reads: "idk why the fuck would you suggest changing a variable that is
    being read", then, after Rust's refusals of all three were shown, "go with rust's way for both lists
    and maps".
96. **The count-limit tests use a real 8-bit counter** (W4's question on the two stop tests that need about
    32 GB with `mem.forget` gone). Those two test programs (too many owners, too many `weakref` links) are
    compiled with an 8-bit count in each object, stored as the real one is, so 255 owners reach its limit
    and the next stops the program ("too many owners of a `Player` (255)"); every other test and every
    real program keeps the 32-bit count, and one more small test checks that real programs use it (limit
    4,294,967,295). No test-only shortcut in the language. Owner: "or just for the sake of test use a
    smaller counter that is easier to overflow", "an actual counter ... that only allows 255 owners max",
    then "yes".
97. **A value made later for an `owns` place says `owns` too** (W1's question; verdicts 74 and 85 against
    E0109). With `class Entity:` holding `mesh: owns Mesh`, `e.mesh = owns Mesh(points=[1.0, 2.0])` gives
    `e` a new mesh (the old one is freed); `e.mesh = Mesh(...)` without it is refused, as verdict 74's
    line. E0109 keeps only its own case: `owns` before a name that already exists (`e.mesh = m` hands `m`
    over and writes no `owns`). Owner: "yes a".
98. **Two changeable elements of one list whose positions are not known when compiling are refused when
    compiling** (verdict 88's record made precise; step 4's verdict check). With `fn collide(ref mut a:
    Body, ref mut b: Body):`, `collide(ref mut bodies[i], ref mut bodies[j])` in `fn step(ref mut bodies:
    Array[Body], i: int, j: int):` is refused (E3022): `i` and `j` may be equal. It compiles where the
    compiler can see they differ: under `if i != j:` written above it, or in `for j in i + 1..n:`. The
    run-time compare that stopped the program when `i == j` (run 3 to step 4, AN-03) goes. Rust refuses
    the same line. Owner: "b", over (a), one compare before the call that stops the program.
99. **A loop of owners is allowed only where the compiler proves it is opened again before its objects lose
    their last owner from outside; otherwise it is refused** (the unsafe-features inventory, C19: a loop of
    `shares` owners leaked with only warning L3001). With `class Node:` holding `next: Option[shares mut
    Node] = None`, `make_pair` doing `a.next = Some(shares mut b)` then `b.next = Some(shares mut a)` is
    refused at the line that closes the loop ("this closes a loop of owners (`a` → `b` → `a`) that nothing
    opens again before `a` and `b` end, so neither node is ever freed; open it before the function ends:
    `b.next = None`, or make the link back `weakref`: `next: Option[weakref Node]`"); the same with
    `b.next = None` before `a` and `b` end is accepted. Proved: a loop made and opened again in code the
    compiler can follow (the same function, the functions it calls), or opened by its owning object (a
    `clear()` it always calls before its last owner ends). Refused: an opening that depends on what is only
    known while running, or a loop handed out of reach; `weakref` always works, and a `let` field (set
    when the object is made) cannot close a loop. The same for loops through function values' state,
    `thread_shared` owners and channel ends. Nothing runs while the program runs; L3001 becomes this
    refusal. Owner: "Can we make it so that this is only accepted if the compiler can prove that they get
    freed at some point, otherwise the error is thrown?", then "Yes" to the rule as written, over (a)
    refusing every such field and (b) keeping the warning as Rust does.
100. **The ECS is EnTT's, written in the new model, and what runs at once is checked when compiling**
    (the spec's writers: `[ECS-4]` checked systems given as values against each other while running).
    The owner: "Refuse when compiling also I think you are talking about ECS system that ECS system is
    going through a redesign too. It is supposed to be like EnTT". So the ECS follows ODR-279 (owner,
    2026-10-23) and its mapping (`tasks/95/ecs-entt-design.md`): a `Registry` of sparse-set pools,
    entities, `emplace`/`get`/`try_get`/`patch`/`remove`, views (`view[(A, B)]()` reads,
    `view_mut[(A, B)]()` changes), signals, groups, snapshots; no scheduler, so `World`, `Query`,
    `Commands`, `System`, `World.run`, `run_parallel` and `E7020` go, and ODR-286's pools are kept as
    `owns dyn AnyPool` values. Where anything does run several pieces of work at once, a clash between
    them is refused when compiling (worked out from every function the program can put there), never
    checked while running, as verdict 98.
101. **A `thread_shared` object's owners are counted in two numbers** (owner, 2026-10-27, after the
    measuring phase; `tasks/96/redesign/measure-0.9.15/brc/REPORT.md`).
    - **The two numbers.**
      - The thread that made the object counts its owners in one number, A, which only it ever reads or
        writes, with normal fast instructions.
      - Every other thread uses the second number, B, with locked instructions.
      - The object has A + B owners.
      - Only the making thread can read both, so when another thread takes the last owner it hands the
        object to the making thread. That thread deletes it at its next check point (placed by the
        compiler; always before the thread ends), or the other thread deletes it at once if the making
        thread has ended.
    - **The fixes.**
      - The maker's id is read from the object's memory chunk (fix 1).
      - A hand-over the compiler sees moves A into B at that moment (fix 2), so the delay remains only for
        an object that travels inside another object.
      - Rule Q: the drop that hands the object over marks it in the same atomic step, and only the making
        thread's check then deletes it.
      - A merged object's −1 is one locked add (the shortcut).
    - **Measured against one locked count**, 550 stress runs clean:
      - a thread's own objects, 25% faster;
      - the camera program, 89 ms against 544 ms;
      - several threads reading one thread's objects, up to 28% faster;
      - objects passed back and forth all the time, 9% to 13% slower;
      - each object 16 bytes bigger.
    - Owner, asked "Do you want the two-number design in the spec?" (with fix 2): "yes". The owner first
      kept the view's count on every read ("why the fuck would remove the counter for a read
      operation?"), and asked for a fix for the language, not for one benchmark.

102. **In a C contract the word for "C takes this object" is `owns`** (owner, 2026-10-27, gap map Q-A:
    "owned is now owns").
    - `@ffi(param(widget, owns, one))`: `[FFI-11]`'s table and its examples write `owns`, as verdict 19
      does everywhere else.
    - `owned` is refused everywhere, a C contract included (E1010, Part II's removed words).
103. **`shares` cannot be made from a place that has one owner** (owner, 2026-10-27, gap map Q-B, (a)).
    - A list element (`shares mut players[1]` of an `Array[Player]`), a field (`shares this.items`) or a
      plain parameter (`fn keep(p: Player): LIST.push(shares p)`) is refused when compiling, with a code
      of its own (a sibling of E2119).
    - The help names the rewrite: the holder shares its values from the start, as the help of R7's
      `reject_removing_a_player_a_kept_link_points_at.em` writes it
      (`players = [shares mut Player(), shares mut Player()]`, then `shares mut players[1]`).
    - Why: a value in a one-owner place has no other owner, so its end frees at once with no check;
      only values written `shares` pay for counting. Allowing it for class objects would make every end
      of a one-owner place of that class test the count; allowing it for inline values would change the
      holder's layout everywhere in the program.
    - Owner, after the shorter explanation (one program, the table of who owns the second player on
      each line): "yes a".
104. **After `h.join()` the parent may use what the task used** (owner, 2026-10-27, gap map Q-C: "okay
    cool use it").
    - `[THR-11]` and `[THR-13]` check a task's claims until its join, not until the end of the
      `with scope:` block: `counts[0] += 1` after `h.join()`, inside the same block, is allowed.
    - Safe: after the join the task has finished.
105. **For now, a sum split across tasks stops where a task's own total overflows** (owner, 2026-10-27,
    gap map Q-D, (b): "b for now might revise it later").
    - Each task's private total (`[THR-21]`'s "one private accumulator per task, combined at the join")
      has the reduction variable's type, and its `+=` stops the program on overflow as any `+=` does
      (`[TYP-8]`); the join's additions are checked the same way.
    - So `[[-100], [100, 100]]` summed into an `i8` by two tasks stops in the second task (100 + 100),
      though adding one by one gives 100, which fits. `[THR-21]`'s sentence that the total is "always
      the one adding one by one gives" is narrowed for whole numbers: equal when no task's total
      overflows.
    - Rust does the same in a debug build (a panic in the thread); its release build wraps and can print a
      wrong total with no error, which `[TYP-8]` does not allow. The other choice, (a), keeps each
      task's total in a wider type so that only a real total that does not fit stops the program; the
      owner may revisit it.
106. **A contract on a function type is written above its `type` line** (owner, 2026-10-27, gap map Q-G,
    (a): "a works").
    - `@noalloc` over `type Handler = fn()`: `[EFF-2]`'s "a contract written on a function type is
      checked against every function stored into it" has its place in the grammar.

107. **A value made with `using` carries no word naming its manager when the manager can do without
    it** (owner, 2026-10-28, gap map Q-F, (b) and (c) together, after the measured costs: "GOOD B AND C
    IT IS").
    - (b) A manager kind that frees nothing one by one (its `deallocate` does nothing, checked when
      compiling) gives values that carry no word: nothing is ever given back value by value, and
      everything is freed together when the manager is cleared or ends. A value that grows cannot be
      made in it this way.
    - (c) A manager that hands out its memory from pieces starting at a multiple of 65,536 bytes, with
      its own address in each piece's first 64 bytes (one piece of its own for a value bigger than a
      piece), promised by its writer in `unsafe`, gives values that carry no word: the manager is found
      from the value's address, as C's `free(p)` finds its heap.
    - Every other manager's values keep the word (verdict 47).
    - Safety is unchanged: the checks that no value outlives its manager or is used after a reset run
      when compiling and never read the word.
    - Measured (`measure-0.9.15/RESULTS.md` section 7, `measure-0.9.15/M3.md`), against plain C: a
      million small values 191 ms and 28.2 MB with (b), 241 ms and 44.7 MB with the word, C 191 ms and
      28.2 MB; growing lists 243 ms and 17.1 MB with (c), 248 ms and 19.1 MB with the word, C 217 ms and
      17.1 MB; a pool freeing values one by one 296 ms and 13.7 MB with (c), 384 ms and 15.8 MB with the
      word, C 290 ms and 13.7 MB.
    - Goes into the next hardening (`[ALC-1]`, `[ALC-2]`, `[OBJ-4]`, `[COST-3]`) with milestone M10.

108. **A `pub` generic function writes everything its type parameters need** (owner, 2026-10-28, after
    `[IFC-6]` was explained, a rule the design writers proposed (AN-69) and the lead put into 0.9.15
    without an owner verdict: "yes pub functions need to declare it"). The lead's reading, stated to the
    owner: the rest of `[IFC-6]` stands, so a function that is not `pub` may leave the list out and the
    compiler works it out from the body. Q-H (a function that is not `pub` writing part of its list)
    stays open.

109. **What a type parameter needs is written after `needs`; several go in round brackets, separated by
    commas** (owner, 2026-10-28, option (b): "it can become a problem when multiple interfaces are
    required so it will turn into needs A and B and C so instead I'll go with b"). The owner suggested
    `[T: needs Add[T, Output = T], Copy]`; the comma already separates type parameters
    (`extend[K: Eq + Hash, V, Q: AsKey[K]] Map[K, V] implements Index[Q]`), so the choices were (a)
    `needs A and B`, (b) round brackets, (c) one bracket line per type parameter.
    - `[T: needs (Add[T, Output = T], Copy)]` above `pub fn sum(xs: Span[T]) -> T:`; one need has no
      brackets, `[T: needs Copy]`.
    - Replaces `+` between bounds in a type-parameter line (0.9.10's spelling, Rust's, never put to
      the owner).
    - The same spelling goes everywhere a list of needs is written: verdicts 110 to 112.

110. **An interface's parents are written with `needs` too** (owner, 2026-10-28: "yes", asked whether
    `interface Error: Debug + Display:` becomes `interface Error: needs (Debug, Display):`). One parent
    has no brackets, `interface Ord: needs Eq:` (verdict 109's rule).

111. **`extend`'s type parameters are written with `needs` too** (owner, 2026-10-28: "yes", asked whether
    `extend[T: Display] Array[T] implements Display:` becomes
    `extend[T: needs Display] Array[T] implements Display:`). Several go in round brackets,
    `extend[K: needs (Eq, Hash), V, Q: needs AsKey[K]] Map[K, V] implements Index[Q]`.

112. **An interface's associated type writes its needs with `needs` too** (owner, 2026-10-28: "yes", asked
    whether `type Iter: Iterator[Item = Item]` in `interface Iterable:` becomes
    `type Iter: needs Iterator[Item = Item]`).

113. **A function that is not `pub` writes its full list of needs or none** (owner, 2026-10-28, gap map
    Q-H: "they either write full or no list"). None: the compiler works the list out from the body
    (`[IFC-6]`, now the owner's; verdict 108's reading confirmed). Part of one is refused, with a message
    giving the complete line: `[T: needs Lookalike]` above `fn direct_add(a: T, b: T) -> T:` with
    `return a + b`, where `Lookalike` declares an ordinary `fn add` and not `Add`, gets
    `[T: needs (Lookalike, Add[T, Output = T])]`.

114. **An object handed to another thread while a `weakref` on this thread points at it goes, and the
    links here then say "gone"** (owner, 2026-10-28, gap map Q-E, "aight approved with these fixes").
    Rust's run-time way (`Rc::try_unwrap`), which the lead checked with `rustc` 1.98.1: Rust's
    compile-time way refuses even `drop(w)` then `thread::spawn(move || names.items.len())` (E0277,
    "`Rc<Names>` cannot be sent between threads safely"), so it refuses fine programs as (a) did; with
    `Rc::try_unwrap(names)` the thread gets the object and `w.upgrade()` is `None` ("0 true").
    - At the hand-over the program reads the object's weak count, one compare. No link: the object
      goes as it is. A link: its contents move to a new place for the other thread, the old place is
      marked ended (its `drop` does not run; the new one's runs once, on the other thread) and stays
      here until its last link ends, so every `weakref` here says "gone". Nothing is refused and
      nothing stops. `names = Names()`, `w = weakref names`,
      `h = thread.spawn(fn() owns names: names.items.len())`, then `if w exists as p:` is skipped and
      `println(h.join())` prints 0 (the design test that refused this becomes a run-pass test).
    - Fix 1, objects inside it: the check also covers every object the handed object owns whose class
      a `weakref` is made to anywhere in the program (the compiler knows those classes);
      `cam.target = weakref level.enemies[0]` then `thread.spawn(fn() owns level: ...)` moves enemy 0
      and `cam.target` says "gone". One compare per such object (10,000 enemies, 10,000 compares).
      Today's text of the rule misses this case too.
    - Fix 2, a link open at that moment: a hand-over to another thread joins the lines after which a
      view opened from a link to an object of that type is not used (`if ROSTER[0] exists as p:`, the
      hand-over of a `Names`, then `p.items.len()` is refused when compiling); nothing runs for it.
    - Fix 3, an address C keeps: an object given to C under a `retained` contract never moves; for it
      a link at the hand-over stops the program (gap map Q-E's (b)). `unsafe` code gets the written
      obligation that an object handed to another thread may move.
    - Checked and fine: the other thread never touches this thread's counts (the old place stays here
      with the links; the new one starts with none); an object holding a `weakref` to a one-thread
      object cannot go to another thread at all; a lock in the object is one futex word, movable, and
      cannot be held across the hand-over.
    - Goes into the next hardening (`[THR-2]`, `[RC-5]`, `[WK-12]`, `[FFI-35a]`, `[UNS-4]`) with M9.

115. **Ember gets typed variadic generics and typed variadic functions** (owner, 2026-10-28: "go for
    both generics and functions, note it down"). It began at gap map Q-I: the ECS writes several
    component types as one tuple (`reg.view[(A, B)]()`) because Ember had no variadic generics, which
    capped a view at 8 types and left one type as `(A,)`, a tuple `[TYP-27]` refuses. The owner: "this is
    not a good limit to have what if I want to access and object with 10 components?"
    - **Variadic generics:** a function or type takes any number of types in its square brackets,
      `reg.view[Position, Velocity, Health, Armor, Speed]()`, one type `reg.view[Position]()`, with no
      limit and no tuple; each use is made for its exact types, as generics are, so nothing runs for it.
      This settles Q-I (shown to the owner: "there's no limit and no round brackets, so Q-I goes away
      too").
    - **Typed variadic functions,** built on them: a call passes any number of values, each of a type
      known when compiling and checked as an ordinary parameter, ownership words included
      (`log("hp", hp, pos)`). C's `...` stays as it is, only in `unsafe` and only to call C
      (`printf("%s", 42)` is the unsafe kind this is not).
    - Uses named to the owner: the ECS (`view`, `view_mut`, `exclude`, `all_of`, `any_of`), `zip` over any
      number of iterators, a tuple's interfaces written once for every size (Rust stops at 12), events
      and callbacks passing several values (`Event[int, String]` with handlers `fn(int, String)`).
    - Costs named to the owner (none a safety issue): a copy of the function per set of argument types,
      as generics; error messages to keep readable; the rules for ownership words over "all the
      remaining arguments".
    - Changes `[TYP-19]` (no variadic generic), `[TYP-26]` and section I.6 (no variadic functions but
      the output functions), `[ECS-3]` and `[ECS-4]` (the tuple list and its 1-to-8 limit), `[STD-*]`'s
      `zip`. Still to put to the owner, with real code, when the hardening writes them: the spelling of
      "any number of types" in a declaration and of the values a variadic function takes, and whether
      `print`, `println`, `eprint`, `eprintln` and `format_to` become ordinary library functions.

116. **A pack is written with C++'s dots** (owner, 2026-10-28, asked to choose among Python's star
    (`[*Ts: needs Display]`, `fn log(*values: Ts):`), Swift's words (`[each T: needs Display]`,
    `fn log(values: repeat each T):`) and C++'s dots: "I prefer c++", then "that ... makes sense from
    readability perspective because it gives the to be continued... vibe"). The lead had recommended the
    star.
    - `[Ts...: needs Display]` above `fn log(values: Ts...):`, called `log("hp", hp, pos)`;
      `[Cs...]` above `fn view() -> View[Cs...]:`, used `reg.view[Position, Velocity]()`.
    - Following C++: `Cs...` among type arguments spells the pack out, `values...` among a call's
      arguments passes the values on, `values.len()` is their number; `for v in values:` is written out
      once per value.
    - Written as `[TYP-47]`; Python's `*args` (P19) is still `E0100`, whose message now gives
      `[Ts...]` and `args: Ts...`.

117. **`print`, `println`, `eprint`, `eprintln` and `format_to` are ordinary library functions with a pack**
    (owner, 2026-10-28: "a", over keeping them known to the compiler). `[Ts...: needs Printable]` above
    `pub fn println(values: Ts..., sep: StrView = " ", end: StrView = "
"):`; calls are unchanged
    (`println("hp:", hp, pos)`), each call is made for its exact types, and a program's own function
    takes any number of printable values the same way. The lead's details (decisions.md section 7):
    `Printable`, implemented by the compiler for every type with `Display` or `Debug` (Display's text,
    else Debug's, since Ember has no specialisation), and parameters after a pack passed by name only.
    Written into `[STD-9]`, `[STD-18]`, `[TYP-26]`, `[TYP-47]` and §IV.8.

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
