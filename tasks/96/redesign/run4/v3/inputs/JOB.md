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
    lines, and the same fact of the functions it calls; a call whose body it cannot see counts as doing
    so), keeps that fact with `note` (in the package's interface file, R17), and checks it at each call:
    `note(LOG[0])` is refused, the message giving both fixes (`note(copy of LOG[0])`, or use `s` before
    `LOG.clear()` in `note`); `note("bye")` is allowed; a `note` that prints before it clears is allowed
    with `note(LOG[0])`. Nothing is checked while running. A change of the fact by reordering lines inside
    a library function is reported as a change to its interface (R17). Owner: "C", over (a) checking only
    the `fn` line (it refused the print-then-clear `note` called with `note(LOG[0])`, the owner's own
    counter-example) and (b) checking only inside the function (it refused the clear-then-print `note`
    called only with other text).
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
    atomic, a lock or Rayon's explicit `sum`, was shown).
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
