# Area E, v5: threads, locks across threads, `unsafe` (AN-71 to AN-84)

Agent 04 (answers-E), run 4, step 4 (v5). Every answer of area E in the eight-part form, brought to the
v5 rules and words, which apply all of the owner's verdicts 1 to 97 (`inputs/JOB.md`, "Owner verdicts
on run 4's open questions"; 26 replaces 1, 27 replaces 25, 95 reverses 90, and 86 voids the spellings it
names) and the corrected records of verdicts 53 and 58: `V5/rules.md` (rules R1 to R18, the codes),
`V5/vocabulary.md` (the words), `V5/changes.md` (what changed from v4) and the lead's
`V5/inputs/worklist.md` (what each of verdicts 83 to 97 changes, the codes and stop texts, and how every
point step 3 left open is settled). `RUN3` means `tasks/96/redesign/run3/`, `RUN4` means
`tasks/96/redesign/run4/`, `V2` to `V5` mean `tasks/96/redesign/run4/v2/` to `v5/`. "R1" .. "R18" are
the v5 rules; "row N" is the owner's agreed row N and "verdict N" the owner's verdict N in
`inputs/JOB.md`; "the worklist" is `V5/inputs/worklist.md`, and "the report" is
`V5/tools/step4_spell_report.md`. Terms are those of `V5/rules.md` section 0; the few this area adds are
in section 1 below.

No compiler change, no git command, no Ember program run, and no new `ember check`: this file makes no
new claim about what today's Ember accepts or refuses. Power was on mains when this step started
(`PowerLineStatus` printed `Online`). Before this step, the lead's spelling script changed 37 lines of
this file's NEW and still-refused code blocks (the report, section 3: `changes ...` taken off first
lines, lambdas and function types, `with` taken off captures); this step checked each of the 37 and
found every one right. It then made the changes the script could not make (prose, tables, messages, the
`text` blocks) and the ones verdicts 83 to 97 ask for (section 0): four of the 37 lines changed again,
`copy of t` to `ref t` (verdict 93), and three went with v4's refused button (verdict 86).

**How the OLD code and its output were carried.** This file is `V4/area-E.md`, copied into `V5` and
then changed by the script (the copy as this step found it is `target/redesign7/04/area-E_v4_as_found.md`),
edited in place. No OLD program of this area changes, so every OLD block, its output and every "means
exactly" block is v4's, and so v3's, v2's and run 4's, byte for byte: `target/redesign7/04/check_carried.py`
compares every fenced block of parts 2 and 5 of each answer with that copy (its output is in
`target/redesign7/04/check_carried.txt`). Run 4 checked every OLD and "means exactly" file with today's
compiler (`target/redesign4/06-area-E/recheck.txt`, on the owner's laptop); none of those files changed
since. The probe files the answers cite (`old/...`, `means/...`, `c/...`) were in the laptop's ignored
`target/` and are not in this checkout; their pasted outputs carry over unchanged. File names inside
carried text are relative to `target/redesign3/12-area-E/`, and "section N" inside carried text means a
section of run 3's area report `RUN3/12-area-E.md`. The "means exactly" programs are written in today's
Ember, so they keep today's `self`, `@sync`, `owned fn`, `Mutex` and a plain `MutSpan` parameter that
writes (`mut ps: MutSpan[Player]`), and AN-76's C file keeps v2's first lines in its comments: that is
the language today's compiler accepts, not a NEW spelling.

Each answer has the eight parts: 1 intent; 2 OLD with today's output; 3 NEW; 4 the owner's test for every
declared name; 5 means exactly (today's Ember or C doing what NEW means); 6 still refused, each with its
code and message; 7 cost; 8 kind. In the owner's-test tables "none (default: ...)" means no word in the
line answers that question and the stated default does (allowed only where the default never changes
anything the line does not show, never aliases silently and never copies silently); a name counts as
**without a word** when neither column has a word. Since verdict 86 no first line says what a function
changes, so a function's own name has no row; where a body changes something, the change itself is
written there (`+=`, `push`, `fetch_add`), and the table points at it.

## 0. What step 4 changed

Compared with `V4/area-E.md` (whose section 0 says what v4 changed). The verdicts and the worklist's
items that land in area E:
- **86** (`changes` and `reads` are removed; the compiler works out what every function changes from its
  code): no first line, lambda or function type of this area says what it changes any more
  (`fn() thread_shared log: log.push("a")`, `fn start(log: thread_shared Log):`, `fn add_point():`,
  `type Handler = fn()`). Every E2105 and E2113 refusal of v4 goes with its code (the rules' section 4
  removes both). The refusals that stay (a walk across a call that changes what it walks, E3020; a call
  that may take a lock this thread holds, E7018) are worked out from the bodies, and their messages name
  the chain of calls. The owner's-test rows that pointed at a `changes` clause point at the change in
  the body, or at row 6's default for a captured real variable.
- **83** (`with` dropped from what a function keeps): the script changed the code; this step changed the
  prose, the tables and every message (`fn() copy of t`, `fn() owns names`, `fn() thread_shared log`).
- **93** (a task takes the loop's text `copy of t` or `ref t`): AN-71, AN-72 and AN-74 take `ref t`,
  which copies nothing, since their copy was there only for safety (the worklist's row for 93); `copy of
  t` is shown as allowed too. A task that uses the loop's name with no word is E3026, whose message gives
  both lines; a `thread.spawn` thread is E7004, whose message gives `copy of` (worklist section 3).
- **92** (a decimal reduction is allowed): AN-71's refused float sum is accepted.
- **94** (`thread.spawn` takes a `thread_shared fn()`): the library's line in section 2, AN-75 and AN-76.
- **89** (a thing changed only through its lock may be reached through several names in one call):
  AN-75's `dump(log, fn() thread_shared log: log.push("seen"))` is allowed.
- **85** (`owns` on the making line inside an `Option`): AN-77's refused generator writes
  `next=Some(owns Link(value=1))`.
- **91 and 58's correction** (`into_raw()` only inside `unsafe`; `Retained` only for callbacks): AN-79's
  rule names `into_raw()`; AN-78 uses `Retained` only for its callback, as the corrected record says, and
  AN-81 says nothing of either.
- **53's correction** (only C code is unseen): section 2's safety argument (point 1).
- **84, 87, 88, 90, 95, 96, 97** change no line of area E: no answer hands a number over with `owns`
  (AN-73's `copy of n` copies), gives a `ref mut` argument into what its callee clears, names one
  variable twice in a call, writes a walked collection through its own name (AN-74's `for ref mut p in
  players` already changes each player through the loop's handle), tests a count limit, or gives an
  `owns` field a value after it is made.
- The worklist's section 4 items for area E: F-10 (AN-74's E7007 gives only the lock, as the rules' row
  now does), F-12 (every message gives the line to write and where it goes), area E's points 1 to 7
  (v4's questions for the owner, each settled: section 4.2), T-W6 1 (AN-78's trampoline tag: generation
  in the top 16 bits, slot in the low 48), T-W6 11 (AN-74's E7010 gives only its first way out).
- One point the rules do not say in words, settled here (section 4.3): a C call inside a walk of
  `game.names` is refused only where a callback C keeps may change `game.names`; AN-78's own callback
  changes only `game.score`.

| answer | what step 4 changed | meaning changed? |
|---|---|---|
| AN-71 | the task looks at its text with `ref t` (verdict 93); the float sum is accepted (verdict 92); `changes total` and its E2105 refusal gone (86); E3026's message gives both words; E3040's and E7015's messages give whole lines | **yes**. Verdict 93: each task reads its own text where it is in `texts`, with no copy (v4: a copy of its own). Verdict 92: the float sum `total_of`, refused in v4 (E7007), is accepted. The program prints what it printed |
| AN-72 | the refused task and the help's tasks take `ref t` (93); the help gives whole lines and where they go (F-12); the E7007 text block in the new spelling | **yes**, verdict 93, in the help's lines only: their task looks at its text instead of copying it. The program is refused as before, at the same name, for the same reason |
| AN-73 | `changes total` and its E2105 refusal gone (86); E3026's message gives the line it replaces | no |
| AN-74 | `ref t` (93); `changes words, tags`, `changes stats` and their two E2105 refusals gone (86); E7007's message in the rules' form, the lock only (F-10); E7010 with one way out (T-W6 11); E3026 with both words | **yes**, verdict 93: each task hands `add_all` a view of its text in `texts` (v4: of a copy of its own). The program prints what it printed |
| AN-75 | `changes Log` gone from the threads' functions, `start` and `main`, with three E2105 refusals (86); `dump`'s call allowed (89); E7018's two messages in the rules' form, with the line to move; `thread.spawn`'s line (94) | no for the NEW program. One call v4 left open (`dump(log, fn() thread_shared log: ...)`) is allowed, verdict 89 |
| AN-76 | no first line says what it changes (86), `type Handler = fn()`; six E2105 refusals and the E2113 one gone (v4's refused button is now the NEW one); the walk refusals named by their chain, with the line to move; `global`'s E0100 message; `thread.spawn`'s line (94) | no: the same lines are refused, worked out from the bodies. A loop over `LOG` may call `b.click()` when no handler the program stores changes `LOG` (v4: when the list's type said plain `fn()`) |
| AN-77 | `next=Some(owns Link(value=1))` in the refused generator (85); the defaults without `changes`; the in-place E3020 says where the line goes | no |
| AN-78 | no `changes Game` (86), with two E2105 refusals; the walk refusals shown with a callback that changes `game.names` (section 4.3); the trampoline's tag (T-W6 1); E3023, E7004, E7015 and E1010 with whole lines | no for the NEW program. Three lines v4 refused in this program (`poll_events()` and an unmarked `sqrt` inside the walk of `game.names`, and `setup` called inside it) are allowed, since its callback changes only `game.score`; they stay refused where a kept callback changes `game.names` (verdict 86, with row 3's per-field facts) |
| AN-79 | the rule names `into_raw()` (verdict 91) | no |
| AN-80 | `changes Stats` gone, with two E2105 refusals (86); E1010's bound written above the function (verdict 50) | no |
| AN-81 | the v5 line only; E4030 cited from `V5/rules.md` | no |
| AN-82 | the v5 line only | no |
| AN-83 | the v5 line only | no |
| AN-84 | E5013 names only C's promotion (worklist section 3, area E's point 7); `bump` writes no `changes HITS` (86) | no |

No answer of this area opens an `Option` or a `weakref` link, keeps a parent link, or holds a link
several fields share, so `exists as`, parent links and `sharedref mut` appear only where a message names
them. No answer uses `self` outside a carried OLD or "means exactly" block: inside a method the object is
`this`. The NEW blocks show no `import std.thread`, as no NEW block of any area does, and AN-73, AN-76
and AN-80 use `Atomic` or `Lock` without the `from std.sync import` line that AN-74, AN-75 and AN-84
show, as v4 had them (the tests in `V5/conformance/` write them); a message that gives a `Lock` or
`Atomic` line to such a program names no import, since the program already uses the type.

## 1. Terms of this area

Run 3's area terms, kept; every other term is `V5/rules.md` section 0's (among them **summary**, **kept
callback**, **unseen code** and **ended**).

- **thread**: a sequence of instructions the operating system runs alongside others, sharing memory.
  **task**, **scope**, **join**: as R15 (`with scope = thread.scope():`; the threads started in it;
  the end of the block, which waits for every task). An **unscoped thread** (`thread.spawn(...)`) may
  outlive the function that started it.
- **the loop's own variable**: the name a `for` line declares (`t` in `for t in texts:`), which the
  loop gives the next element every round, while a task or a thread started in an earlier round may
  still run (R15).
- **data race**: two threads touch one place in memory at the same time, at least one writes, and
  nothing orders them. In C and Rust it is undefined behaviour; in practice it loses updates or frees
  memory still in use.
- **lock**: an object at most one thread holds at a time; `with x.lock:` waits until it is free, holds
  it for the block, and lets it go at the block's end.
- **atomic**: an operation two threads cannot interleave (on this processor a `lock`-prefixed
  instruction), several times dearer than a plain one.
- **reduction**: per-task results combined after the join, on a local or on a field of an object the
  tasks reach, whole numbers or decimal ones (R15, verdicts 77 and 92).
- **trampoline**: the small C function Ember generates for a C library to call; it finds the Ember
  function and calls it. **user data**: the `void*` a C library hands back to its callback.
- **kind**: the job's outcomes: a readable spelling of Rust's guarantee; a smarter compile-time
  solution; a new mechanism; an understandable limitation kept.

## 2. Area E's rules, as the v5 rules state them

Run 3's nine area rules are all in `V5/rules.md`; nothing here adds to them.

| run 3's area rule | v5 rule | note |
|---|---|---|
| 1. scoped tasks need no words; reductions | R15 (third and fourth points) | no capture words but for a loop's own variable: a task uses the real variables (row 6), and what it changes is worked out from its body (R5, verdict 86); the loop's own variable is given to a scoped task as its own copy, `copy of t`, or as a look at the element it is on in that round, `ref t` (verdict 93), and a local of one round as a copy (`copy of n`); a reduction may add to a local or to a field of an object the tasks reach (`stats.total += x`, verdict 77), decimal totals included, which may differ in the last digit from adding one by one and are the same on every run (verdict 92); a task that reads the place first needs a lock (E7007); starting a task counts as making what its function changes (R15, verdict 30); a task started inside a walk of what it changes is E3020 (R18, verdict 73) |
| 2. an unscoped thread or a kept C callback says what it holds in its capture list | R15 (fifth point), R3, R12 | the capture words follow the brackets, with no `with` (verdict 83): `owns`, `thread_shared`, `copy of`, and, on this thread only, `shares`, `shares mut`, `weakref`; E7004 for a name with no word; `thread.spawn` takes a `thread_shared fn()` (verdict 94), so a one-thread value its function holds is E7015; the loop's own variable as a copy (`thread.spawn(fn() copy of r: worker(r))`, verdicts 65 and 83), never `ref t` (E3026, verdict 93) |
| 3. a lock guards the fields its class names (`guarded by`) | R15 (`guarded by`), R4, R5 | a `guarded by` field is a `Mutex` written as ordinary fields and follows the `Mutex`'s rules: changed under its lock through any name, with no word at the call (R4, verdict 44); inside a method the object is `this`: `with this.lock:`; what a function changes under a lock is worked out like any change (verdict 86, which voids verdict 29's spelling and keeps its gain); an object whose changes go only through its lock may reach one call through several names (R4, verdict 89); a call that may take a lock this thread holds is E7018 (verdicts 43, 52, 73) |
| 4. `thread_shared` | R3, R9, R12 | spelling kept (verdict 20); E7014, E7017; its count is compared with its maximum before each +1 (verdict 59); a function type whose functions any thread may run says `thread_shared fn()` and holds no one-thread value (verdict 61, E7015), and `thread.spawn` takes one (verdict 94) |
| 5. what may go to another thread is inferred from the words | R15 | unchanged; E7015, E7008 |
| 6. module variables, `static mut` | R5, R12, R15 (`static mut`) | `static mut` kept (verdict 8); what a function changes in one is worked out from its body, with no word on its first line (verdict 86); storing a lambda that changes one changes nothing (AN-76's `setup`, G-06), and starting a thread that runs it counts (verdict 30); `FRAME = 0` at module level and `global` are E0100 |
| 7. generators walk by position across `yield` | R15 (last point), R6, R8 (c) | a generator paused in a list is a walk of it: the list does not change while the generator will still be used (verdict 16, E3020) |
| 8. C callbacks through the checked trampoline | R15 (last point), R12 | what a kept callback changes is worked out from its body (verdict 86); handing it to C counts as making that change (verdict 30); every foreign call counts as possibly running it, unless the foreign function's declaration says `@ffi(no_callbacks)` (verdict 31); C holds a tagged number with the callback's generation in its top 16 bits and its slot in its low 48 (worklist section 3); a call after the token ended, a second call of a `callback(once)` one, or a call on a thread its contract does not allow stops the program with section 2's texts, the thread checked in every profile; a callback that ends its own token runs to its end (verdict 62) |
| 9. `unsafe`: one word, two positions | R16 | anything that can break a safety promise is possible only inside `unsafe` (verdict 60), `into_raw()` included (verdict 91) |

**Area E's safety argument** (run 3's, unchanged in substance). A data race needs two threads, one
place, a write, and nothing ordering them. Every place two threads can reach is one of these, and each
is ordered or never written:
1. a place a scoped task reads: the parent and the other tasks may not change it until the join (R15;
   the compiler reads every function a task may run, function values and `dyn` methods included, and
   only C code is unseen, where a call counts as running every callback C keeps; R5, verdict 53's
   corrected record); the loop's own variable, which the parent changes every round, reaches a task only
   as the task's own copy (`copy of t`) or as a view of the element it is on (`ref t`), which the walked
   collection keeps unchanged until the join (verdict 93);
2. a place tasks change by a reduction (a local, or a field of an object they reach): each task has its
   own accumulator; the join orders them; a task that reads the place first is E7007 (verdict 77);
3. an `Atomic`, `Mutex`, `RwLock`, channel end, or a `guarded by` field: ordered by the atomic or the
   lock, held by the thread that touches it (a task starts holding none);
4. a value handed over (`owns`): the giver's name has ended, and nothing reachable from it is a
   one-thread owner (`shares`, `shares mut`), a `weakref` link of a one-thread value, or thread-bound
   (E7015, E7008); a function any thread may run is a `thread_shared fn()`, `thread.spawn`'s included,
   and holds none of these (verdicts 61 and 94, E7015);
5. a `thread_shared` value: never changed directly by any owner (E7014), its count is atomic and is
   compared with its maximum before each +1, so it never rolls over (verdict 59);
6. a `static mut`: reached only by the main thread and its scoped tasks (E7012), or `guarded by`;
7. a `sharedref mut` target (R9): a scoped task changes it directly only where no other task and not
   the parent may touch it before the join, or by a reduction (E7007); an unscoped thread never holds
   one (E7015).

What a thread, a kept callback or a function value changes is not needed by these points; the compiler
works it out from the bodies (R5, verdict 86) so that a walk across such a change is refused when
compiling (R8 (c)): a C call inside a walk of a list a kept callback may change (AN-78), a scoped task
started inside a walk of what it changes (E3020, R15); and so that a wait for a lock this thread holds
is refused when compiling (E7018, R15): a call through a function value that may take the log's lock,
under that lock, inside a walk of its guarded list or not (AN-75), and a join inside the lock of a thread
started in a walk under it (AN-76).

Freed while in use: a scoped task's views (a `ref t` included) die at the join, which is before the
block ends; an unscoped thread holds only what its capture list names (an owner, a copy, or a `weakref`
link, which is counted and tested on read, R11); one-thread counts never cross threads (E7015); where
scoped tasks re-point `shares` fields of parent objects, count changes are netted per task and decreases
wait until every task has finished its loop (run 3's attack, 3.1, kept); a C callback holds a checked
slot, not an address, and one that ends its own token while it runs keeps its block, where its captures
live, until it returns (verdict 62).

**Area E's rulings** (run 3's, standing, in the v5 words): `@thread_shared` is not added once
`thread_shared` exists (AN-80); a class object held `shares` uses a one-thread count, and a value created
`shares` cannot later be made `thread_shared` ("create it `thread_shared` from the start: `names =
thread_shared Names()`", E7017); a safe `static mut` (AN-76); `@cached` across threads: on an object
several threads reach, the slot is filled under a once-flag, and a `@cached` method whose body reads a
field other threads change is refused (E2117: "`radius` reads `verts`, which other threads change under
`lock`; a kept result could be stale; remove `@cached` (line 18)"); the `unsafe fn` body rule (AN-79).

**The owner's verdicts this area rests on** (the readings v2 to v4 put to the owner, as decided):
1. **A change made under a lock or through an `Atomic` is a change like any other** (verdict 29; its
   spelling is void by verdict 86, R5): no first line names it; what a function changes in a `Log`
   under its lock, or in an `Atomic`, is worked out from its body. The gain stays: a call through a
   function value that may take the lock its thread already holds is refused when compiling (E7018,
   AN-75; verdicts 43 and 73). The call-site rule is unchanged: a `Mutex`, `RwLock`, `Atomic` or
   channel end takes no word at a call (verdict 3).
2. **Starting a thread, or handing C a callback to keep, counts as making what its function changes**
   (verdict 30, R12, R15): the library writes `[R]` above `pub fn spawn(owns f: thread_shared fn() -> R)
   -> JoinHandle[R]:` (verdict 94; verdict 50 puts the type parameters on their own line), and `[R]`
   above `fn spawn(owns f: fn() -> R) -> ScopedJoinHandle[R]:` for `scope.spawn`, whose tasks may use the
   caller's own variables; what `main` (AN-76), `start` (AN-75) and `setup` (AN-78) change by starting a
   thread or handing C a callback is worked out, and no first line says it (verdict 86).
3. **A foreign call counts as possibly running every callback C keeps**, and makes the changes they
   make, worked out from their bodies, unless the foreign function's declaration says
   `@ffi(no_callbacks)` (verdict 31, R12): AN-78's part 6 shows a walk of `game.names` across `sqrt`
   (marked) allowed and across `poll_events()` refused, for a kept callback that changes `game.names`.
4. **A plain `game: shares mut Game` parameter only reads the game** (verdict 32, R4), so `setup` takes
   `owns game: shares mut Game`, called `setup(shares mut game)`, with `fn() owns game`.
5. **A `guarded by` field is its lock's value, changed under the lock through any name that reaches its
   object, as a `Mutex`'s value is** (verdict 44: "Keep guarded by and the same way as mutex"; R4 says
   it in words). AN-75 relies on it three times: `start`'s plain `log: thread_shared Log` makes
   `thread_shared log` owners that change `lines` under the lock, which R3's "an owner made from another
   may do no more than it" allows because `log` may do the same; `start`'s read-only `log` makes no
   promise about `lines`, which is read and changed only under its lock (R8 (g)); and E7013's way out,
   `log.push("c")` in `start`. And such an object may reach one call through several names, a plain
   argument and a function value that holds it (verdict 89).
6. **A task takes the loop's own variable as `copy of t` or `ref t`** (verdict 93, R15): "both ways
   should be permissible ... as long as there is no clash"; `ref t` clashes only where the function may
   outlive the element (a `thread.spawn` thread, E3026).
7. **A decimal reduction is allowed** (verdict 92, R15): the same answer on every run, which may differ
   in the last digit from adding one by one, as Rust's Rayon and C's OpenMP give.

## 3. The answers of area E

### AN-71: threads (A15): the loop variable, `total +=` from tasks, E7004 offering `@sync`

**v5:** the task looks at the text it counts where it is, `fn() ref t:` (verdict 93): `t` is the loop's
own variable, which names the next text every round while a task started in an earlier round may still
run, and `ref t` captures what `t` views in its round, an element of `texts`, which cannot change until
every task has ended at the end of the `with scope` block (R15). v4's `fn() copy of t:` stays allowed and
gives each task its own copy; it was there only for safety, and costs one allocation and one copy per
text. The task no longer says `changes total` (verdict 86): it uses the real `total` (row 6), and what it
changes is worked out from its body. The float sum v4 refused is accepted (verdict 92, part 6).
**Meaning changed:** yes. Verdict 93: each task reads its text in `texts` itself, with no copy, where
v4's read a copy of its own. Verdict 92: `total_of`, refused in v4 (E7007), is accepted. The program
prints what it printed.

**1. Intent:** count the words of several texts on several threads, and let a thread read a list of
names.

**2. OLD** (`old/a15_threads.em`, phase 1's program, unchanged):

```ember
# A15: count words in several texts on several threads.
import std.thread

fn count_words(texts: Array[String]) -> int:
    total = 0
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn(): total += t.split_whitespace().count())
    return total

class Names:
    items: Array[String] = []

fn share_plain():
    names = Names()
    h = thread.spawn(owned fn() => names.items.len())
    println(h.join())

fn main():
    println(count_words(["a b".to_string(), "c".to_string()]))
    share_plain()
```

```text
error[E3021]: `t` cannot be written while it is borrowed
 --> a15_threads.em:7:9
  |
7 |         for t in texts:
  |         ^^^^^^^^^^^^^^^ conflicting access here
8 |             scope.spawn(fn(): total += t.split_whitespace().count())
  |                         ------------------------------------------- borrow of `t` starts here
  |             -------------------------------------------------------- borrow later used here
  |
  = help: end the borrow before this: `scope` is what keeps it alive, so shorten its last use or put it in a block of its own
  = note: a borrow lasts until its last use, not to the end of the scope (BRW-2)

error[E3022]: `total` is already mutably borrowed
 --> a15_threads.em:8:25
  |
8 |             scope.spawn(fn(): total += t.split_whitespace().count())
  |                         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ conflicting access here
  |                         ------------------------------------------- mutable borrow of `total` starts here
  |
  = help: use one mutable access rather than borrowing the same place twice; split the owner's mutable view with `split_at` when the intended operands are disjoint parts of it
  = note: a borrow lasts until its last use, not to the end of the scope (BRW-2)

error[E3060]: `t` does not live long enough
 --> a15_threads.em:8:25
  |
8 |             scope.spawn(fn(): total += t.split_whitespace().count())
  |                         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ this borrow remains live after its source's scope ends
  |             -------------------------------------------------------- `t` ends its scope here
  |
  = help: declare the borrowed value in an enclosing scope, or drop its borrower before leaving this scope
  = note: borrowed storage must remain valid through every use, including a destructor [DRP-7]

error[E7004]: the thread's function captures `names`, a `Names`, which is not Send
  --> a15_threads.em:16:22
   |
16 |     h = thread.spawn(owned fn() => names.items.len())
   |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: declare it `@sync class Names`: its fields are all `Send` and `Sync` [THR-1]
   = note: what a thread's function captures moves to that thread [THR-10]

error: could not compile due to 4 error(s)
exit=1
```

**3. NEW** (v4's, with `ref t` on the task; no method, no call to a `ref mut` or `MutSpan` parameter):

```ember
fn count_words(texts: Array[String]) -> int:
    total = 0
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn() ref t: total += t.split_whitespace().count())
    return total

class Names:
    items: Array[String] = []

fn share_plain():
    names = Names()
    h = thread.spawn(fn() owns names: names.items.len())
    println(h.join())

fn share_and_keep():
    names = thread_shared Names()
    h = thread.spawn(fn() thread_shared names: names.items.len())
    println(names.items.len(), h.join())
```

`count_words` is the OLD text with one addition, `ref t`; under R15 it compiles (`total +=` in a task is
a reduction). The task uses the real `total` (row 6: a name not in the capture list is the real
variable), and what it changes, `total`, is worked out from its body (R5); starting it counts as making
that change (R15), a change of `count_words`'s own local. It looks at this round's text with `ref t` (R3,
R15, verdict 93): a read-only view of what `t` views when the task is made, the element of `texts` the
loop is on. Nothing may change `texts` until the join (R15: the parent and the other tasks may not change
what a task reads), and the join is at the end of the `with` block. `scope.spawn(fn() copy of t: total +=
t.split_whitespace().count())` is allowed too, and gives each task its own copy of its text.
`share_plain` hands the object to the thread; `share_and_keep` creates it shared between threads and
keeps reading it. `thread.spawn` takes a function any thread may run (`thread_shared fn()`, R15, verdict
94): one that owns `names` alone, or holds a `thread_shared` owner of it, is one. A capture
`thread_shared x` of a plain local is refused (E7017, below).

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `texts` (1) | none (default: a plain parameter reads the caller's list, which does not change while `texts` is used, R4) | none (default: read-only, R4) |
| `total` (2) | none (default: a new local, the function's own) | none (default: the function's own variable; `+=` is the change, written) |
| `scope` (3) | `with` (bound for this block, ended at its end) | none (default: read-only binding) |
| `t` (4) | none (default: a plain loop name, a read-only view of each element of `texts`, R2) | none (default: read-only) |
| `t` (5, capture) | `ref` (a view of the text `t` is on in this round, an element of `texts`, R3, R15, verdict 93) | `ref` (read-only) |
| `total` (5, in the task) | none (default, row 6: a name not in the capture list is the real variable, R12) | none (default, row 6: the real variable, which the task may change; `+=` is the change, written) |
| `items` (9) | none (default: the object's own list) | none (default: its own; what may change it is worked out from the bodies that use it, R5, and none here does) |
| `names` (12) | none (default: a new local, the function's own object) | none (default: its own) |
| `h` (13) | none (default: the `JoinHandle` the call returns, the caller's own) | none |
| `names` (13, capture) | `owns` (handed to the thread; the caller's `names` ends here) | `owns` (the thread's own now) |
| `names` (17) | `thread_shared` (a new object with owners on any thread) | `thread_shared` (no owner changes it directly, R9) |
| `h` (18) | none (default as line 13) | none |
| `names` (18, capture) | `thread_shared` (one more owner, on any thread) | `thread_shared` (no owner changes it directly) |
| `names` (19) | `thread_shared` (line 17) | `thread_shared` (line 17: read-only) |

14 names; 8 without a word (`texts`, `total` (2), `t` (4), `total` (5), `items`, `names` (12), `h` (13),
`h` (18)), each with its default stated. No default changes anything the line does not show, aliases
silently, or copies silently: the task's view of its text is written (`ref t`), its change of the real
`total` is written (`+=`, row 6's default), and the per-task accumulator behind line 5 is a compile-time
translation no line can observe (5).

**5. Means exactly** (`means/an71_means.em`, today's Ember):

```ember
import std.thread
from std.sync import Atomic

fn count_words(texts: Array[String]) -> int:
    total = Atomic(0)
    tr = ref total
    with scope = thread.scope():
        for t in texts:
            scope.spawn(owned fn() => tr.fetch_add(t.split_whitespace().count()))
    return total.load()

struct Names:
    items: Array[String]

fn share_plain():
    names = Names(items=[])
    h = thread.spawn(owned fn() => names.items.len())
    println(h.join())

fn main():
    println(count_words(["a b".to_string(), "c".to_string()]))
    share_plain()
```

```text
exit=0
```

and `means/an71_means_shared.em` for `share_and_keep` (an object shared between threads, atomic
count, never changed after): `@sync class Names` with `h = thread.spawn(owned fn() =>
names.items.len())` followed by `println(names.items.len(), h.join())`: `exit=0`.

Meaning: every task adds its part to the real total; the total is read after the join. The compiled
NEW code does not use an atomic: each task keeps its own count and the parent adds them at the join, in
spawn order (`reduce-0` below). Nothing in the program can tell the two apart, because a task or a
parent line that reads `total` before the join is refused (6). One difference, stated: with integers
that check overflow (`[TYP-8]`), a task's partial sum may overflow where some order of single adds
would not; that is a panic, never a wrong value (last run 4.4). `share_plain`'s one-owner object is
written as a struct value moved to the thread, which is what a hand-over of a one-owner object is. The
NEW task reads its text where it is in `texts` (`ref t`), and the translation's task reads it through
the loop's view; neither copies a text.

**6. Still refused** (each message to R18):

- The task using the loop's own variable with no word, the OLD's line, `scope.spawn(fn(): total +=
  t.split_whitespace().count())`: `error[E3026]: the task on line 5 may still run in a later round, when
  \`t\` names another text; give it its own copy, \`scope.spawn(fn() copy of t: total +=
  t.split_whitespace().count())\`, or let it look at this round's text, \`scope.spawn(fn() ref t: total
  += t.split_whitespace().count())\`, in place of line 5` (R15, verdict 93; the rules' E3026 row, in this
  program's lines). v4 gave the OLD's line a second message, E2105 for the missing `changes total`,
  which went with verdict 86.
- `ref t` given to an unscoped thread, which may outlive the loop: `thread.spawn(fn() ref t:
  println(t.split_whitespace().count()))` in place of line 5: `error[E3026]: the thread may still run
  after the loop has moved on and \`texts\` is gone; give it its own copy: \`thread.spawn(fn() copy of t:
  println(t.split_whitespace().count()))\` in place of line 5` (R15, verdict 93: `ref t` is refused
  where it would clash; the rules' E3026 row).
- A task that reads `total` before the join, `scope.spawn(fn(): if total > 10: println("many"))` added
  as line 6, inside the loop: `error[E7007]: tasks add their parts to \`total\`, which is known only once
  the \`with\` block (lines 3 to 6) ends; read it after the block: remove line 6 and write \`if total >
  10: println("many")\` above \`return total\``.
- A task changes `total` in a way the compiler cannot add up per task: AN-72's message.
- Using `names` after the thread took it, `println(names.items.len(), h.join())` in place of line 14
  of `share_plain`: `error[E3040]: \`names\` was handed over to the thread on line 13 (\`fn() owns names\`)
  and is used on line 14; to keep using it, create it shared and give the thread one more owner: write
  \`names = thread_shared Names()\` in place of line 12 and \`h = thread.spawn(fn() thread_shared names:
  names.items.len())\` in place of line 13`.
- An unscoped thread using a name with no capture word, `h = thread.spawn(fn() => names.items.len())`
  on line 13: `error[E7004]: this thread may outlive \`share_plain\`, so it cannot use
  \`share_plain\`'s \`names\`; say what the thread holds: write \`h = thread.spawn(fn() owns names:
  names.items.len())\` in place of line 13 (handed over), or create it shared, \`names = thread_shared
  Names()\` in place of line 12, with \`h = thread.spawn(fn() thread_shared names: names.items.len())\`
  in place of line 13 (one more owner)`. It never offers `@sync`.
- `fn() thread_shared names` on the plain local of line 12: `error[E7017]: \`names\` is
  \`share_plain\`'s own object, which it may change; to share it between threads, create it shared:
  write \`names = thread_shared Names()\` in place of line 12` (the same for `keep = thread_shared
  names`).
- Handing over a value that holds a one-thread owner, in this refused program:

  ```ember
  class Cache:
      hits: int = 0

  class Names:
      items: Array[String] = []
      cache: shares Cache

  fn share_cached():
      c = shares Cache()
      names = Names(cache=shares c)
      h = thread.spawn(fn() owns names: names.items.len())
      println(c.hits, h.join())
  ```

  `error[E7015]: the thread gets \`names\`, which holds \`cache\`, and \`cache\`'s other owner \`c\`
  (line 9) stays on this thread; \`shares\` owners count without atomics, so two threads would change
  one count; share the cache between threads: write \`cache: thread_shared Cache\` in place of line 6,
  \`c = thread_shared Cache()\` in place of line 9 and \`names = Names(cache=thread_shared c)\` in place
  of line 10` (R12, R15: `thread.spawn` takes a `thread_shared fn()`, which holds no one-thread value,
  verdict 94). The same for a class that is the target of a `weakref` field anywhere in the build (its
  `weakref` links stay on this thread): if `Names` were one, the NEW's `share_plain` would be refused
  with "share it instead: write `names = thread_shared Names()` in place of line 12 and `h =
  thread.spawn(fn() thread_shared names: names.items.len())` in place of line 13".
- Changing a `thread_shared` value directly, in this refused program (`share_and_keep` given a name
  to add):

  ```ember
  class Names:
      items: Array[String] = []

  fn share_and_keep(s: String):
      names = thread_shared Names()
      names.items.push(copy of s)
      h = thread.spawn(fn() thread_shared names: names.items.len())
      println(names.items.len(), h.join())
  ```

  `error[E7014]: \`names\` is shared between threads (line 5), so no owner changes it directly, and line 6 pushes to its list; fill the list where the object is made: write \`names = thread_shared Names(items=[copy of s])\` instead of lines 5 and 6`. (A list that must change while threads share it is kept in a field
  `guarded by` a lock, and every read and change of it is made inside that lock, E7013, as AN-74's
  `Stats` does.)
- A thread-bound value sent to a task (a guard, a `Scope`, a raw pointer; run 3's attack `x05`,
  `x08`, `x09`): `error[E7008]: \`g\` is the guard of \`m\`'s lock, which belongs to the thread that
  took it; take the lock inside the task: write \`fn add_one(m: Mutex[Array[int]]):\` with the body
  \`with g = m.lock():\` and, inside it, \`g.push(1)\`, above line 11, and \`scope.spawn(fn():
  add_one(m))\` in place of line 16`` (the program of `R15/reject_a_lock_guard_sent_to_a_task.em`).

Accepted, no longer refused (verdict 92): a float place one task adds to more than once, in this
program, which v4 refused (E7007, "float sums depend on how the adds are grouped"):

```ember
fn total_of(parts: Array[Array[float]]) -> float:
    sum = 0.0
    with scope = thread.scope():
        for p in parts:
            scope.spawn(fn() copy of p: for x in p: sum += x)
    return sum
```

Each task adds its numbers to a private sum, and the sums are added to `sum` at the end of the scope, in
spawn order (R15): `total_of([[0.1], [0.2, 0.3]])` gives `0.6` (the tasks' sums are 0.1 and 0.5),
where adding one by one gives `0.6000000000000001`, and gives it on every run, as Rust's Rayon and C's
OpenMP do. The task takes `copy of p`, verdict 92's own spelling; `fn() ref p:` is allowed as well and
copies no list (verdict 93).

**7. Cost.** Scoped tasks and reductions, whole-number and decimal: nothing beyond plain C's per-thread
counter. Last run, `phase3b-threads.md:381-386` (64 MiB of text, 8 tasks; PROVISIONAL): `reduce-0`,
plain C with a local counter per thread summed after the join, 43.548 ms (1.00), which the reduction
compiles to; an atomic add per word (today's `Atomic` spelling) 10.22x; safe Rust's `AtomicU64` 10.35x,
its join-sum 1.08x. `ref t` gives each task a pointer to its text, as plain C hands each thread one: no
copy. `copy of t`, where written, adds one allocation and one copy of the text's bytes per task, and
its free at the task's end (not measured). `owns names`: a pointer handed over, no count.
`thread_shared names`: one atomic increment when the thread's owner is made, after one compare of the
count with its maximum (verdict 59, R9; at the maximum the program stops, "too many owners of a `Names`
(4,294,967,295)"), and one atomic decrement when it ends, Rust's `Arc` price, which makes the same
compare. Because the word says which values are thread-shared, the count kind is known when compiling:
last run's run-time "shared flag" (and its walk, 5.451 ms per 262,144 objects, `phase3b-threads.md:538`)
is not needed.

**8. Kind:** a smarter compile-time solution (the scope rule, reductions, and a look at the loop's
element with no copy, as Rust's scoped threads borrow it), with a readable spelling of Rust's `move` +
`Arc` for unscoped threads (the capture words).

---

### AN-72: errors land on the wrong variable in `a15_threads.em`

**v5:** the refused program's task, and the tasks in the help's lines, look at the loop's text with `ref
t` (verdict 93), as AN-71's NEW does: without a word the program would have a second mistake (E3026,
AN-71's) beside the one this answer is about. The tasks say no `changes total` or `changes parts`
(verdict 86). The help gives whole lines and where they go (F-12). **Meaning changed:** yes, verdict 93,
in the help's lines only: their task looks at its text where v4's took a copy. The program is refused as
before, at the same name, for the same reason.

**1. Intent:** read the error and fix the right line.

**2. OLD.** In `a15_threads.em` (AN-71's OLD, output pasted there) the errors on `t` (E3021, E3060)
are caused by the write to `total`. Taking the write out leaves no error (`old/an72_read_only.em`,
the same loop with `scope.spawn(fn(): println(t.split_whitespace().count()))`):

```text
exit=0
```

Taking the read of `t` out leaves only the error on `total` (`old/an72_write_only.em`, the task is
`fn(): total += 1`):

```text
warning[L1001]: `t` is never read
 --> an72_write_only.em:7:13
  |
7 |         for t in texts:
  |             ^ declared here and never used
  |
  = help: remove it, or rename it `_t` if that is deliberate

error[E3022]: `total` is already mutably borrowed
 --> an72_write_only.em:8:25
  |
8 |             scope.spawn(fn(): total += 1)
  |                         ^^^^^^^^^^^^^^^^ conflicting access here
  |                         ---------------- mutable borrow of `total` starts here
  |
  = help: use one mutable access rather than borrowing the same place twice; split the owner's mutable view with `split_at` when the intended operands are disjoint parts of it
  = note: a borrow lasts until its last use, not to the end of the scope (BRW-2)

error: could not compile due to 1 error(s)
exit=1
```

**3. NEW:** a program that stays refused, and the one error it gets:

```ember
fn count_words(texts: Array[String]) -> int:
    total = 0
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn() ref t: total = total * 2 + t.len())
    return total
```

```text
error[E7007]: two tasks change `total`, and `total = total * 2 + t.len()` uses its old value
  --> count.em:5:37
   |
 5 |             scope.spawn(fn() ref t: total = total * 2 + t.len())
   |                                     ^^^^^ every task of this scope runs this line at the same time as the others
   |
   = help: to add each task's part, write `scope.spawn(fn() ref t: total += t.len())` in place of line 5; to combine the parts in order, write `parts: Array[int] = []` above line 3, `scope.spawn(fn() ref t: parts.push(t.len()))` in place of line 5, and `for p in parts: total = total * 2 + p` below the `with` block, above line 6
```

The rule: a task's conflict is reported once, on the captured name that causes it; errors on other
names of the same function that only follow from it are not shown, and no error ever names a name the
program uses correctly (`t` here). The help's second way gives the same result as the program run on
one thread: `push` in a task is a reduction combined at the join in spawn order (R15), which is the
order of `texts`. (Run 3's help said only "have each task keep its own (`parts.push(t.len())`)", which
does not compile until `parts` is declared; R18 asks the help's lines to compile in the program.) The
help's tasks look at their text with `ref t`, as the program's task does (verdict 93). A program that
also leaves out the task's `ref t` (the OLD's task with this body) has two mistakes in one line; by
verdict 73 the message shown is this one, whose lines, each with `ref t`, fix both and compile, where
E3026's lines alone would leave the conflict.

**4. The owner's test:** `texts` (1), `total` (2), `t` (4): none (defaults as in AN-71's table, the same
lines); `t` (5, capture): `ref` for both (a read-only view of this round's text, verdict 93); `total`
(5): none for both (row 6's default: the real variable, which the task may change; `=` is the change,
written); `scope` (3): `with`. 6 names, 4 without a word. In the help's lines, `parts` is a new local,
`count_words`'s own (none: its own; `push` is the change, written), and `p` is a plain loop name (none:
a read-only view of each number, which as an `int` is copied, R2).

**5. Means exactly.** The program is refused; the line its help gives first (`total += t.len()`) is
AN-71's program, whose translation is `means/an71_means.em` (`exit=0`, pasted in AN-71).

**6. Still refused:** the NEW program itself, with the message above.

**7. Cost:** none; a compile-time message.

**8. Kind:** a smarter compile-time solution (the error found at its cause).

---

### AN-73: "cannot find `n`" (E1010) inside a lambda that calls an `Atomic` method

**v5:** no line of the program changes. The task no longer says `changes total` (verdict 86): it uses the
real `total` (row 6), and its change, `fetch_add`, is worked out from its body; v4's E2105 refusal of the
task without the word goes with its code. The task keeps `copy of n`: `n` is a local of one round, which
ends with it, so `ref n` would be refused (E3026, R15; verdict 93's `ref t` is for the loop's own
variable, whose element lives until the join). **Meaning changed:** no.

**1. Intent:** each task adds its text's word count to one `Atomic` total.

**2. OLD** (`old/an73_atomic_in_lambda.em`):

```ember
import std.thread
from std.sync import Atomic

fn count(texts: Array[String]) -> int:
    total = Atomic(0)
    with scope = thread.scope():
        for t in texts:
            n = t.split_whitespace().count()
            scope.spawn(fn() => total.fetch_add(n))
    return total.load()
```

```text
error[E1010]: cannot find `n` in this scope
  --> an73_atomic_in_lambda.em:10:49
   |
10 |             scope.spawn(fn() => total.fetch_add(n))
   |                                                 ^
   |

error: could not compile due to 1 error(s)
exit=1
```

Two problems sit behind the one message: a name-lookup defect (the same lambda written `owned fn`
finds `n`: `means/an73_means.em`, below, `exit=0`), and a real one the message hides: `n` is a local
of one round of the loop, which ends while the task may still run. With the call moved into a helper
(`old/an73_helper_round_local.em`: `fn add(total: Atomic[int], n: int)` and
`scope.spawn(fn() => add(total, n))`) today's compiler finds `n` and then says exactly that, though its
help does not give the line to write:

```text
error[E3021]: `n` cannot be written while it is borrowed
  --> an73_helper_round_local.em:13:17
   |
13 |             n = t.split_whitespace().count()
   |                 ^ conflicting access here
14 |             scope.spawn(fn() => add(total, n))
   |                         --------------------- borrow of `n` starts here
   |             ---------------------------------- borrow later used here
   |
   = help: end the borrow before this: `scope` is what keeps it alive, so shorten its last use or put it in a block of its own
   = note: a borrow lasts until its last use, not to the end of the scope (BRW-2)

error[E3060]: `n` does not live long enough
  --> an73_helper_round_local.em:14:25
   |
14 |             scope.spawn(fn() => add(total, n))
   |                         ^^^^^^^^^^^^^^^^^^^^^ this borrow remains live after its source's scope ends
   |             ---------------------------------- `n` ends its scope here
   |
   = help: declare the borrowed value in an enclosing scope, or drop its borrower before leaving this scope
   = note: borrowed storage must remain valid through every use, including a destructor [DRP-7]

error: could not compile due to 2 error(s)
exit=1
```

**3. NEW** (v4's, without `changes total`):

```ember
fn count(texts: Array[String]) -> int:
    total = Atomic(0)
    with scope = thread.scope():
        for t in texts:
            n = t.split_whitespace().count()
            scope.spawn(fn() copy of n: total.fetch_add(n))
    return total.load()
```

The defect is fixed (a name used as an argument of a method call inside a lambda is looked up like any
other), and the task names what it copies, row 6's words (R15: a local of one round, which ends with it,
is given to a task with a word): `n` is the round's own number, which ends with the round while the task
may still run, so the task takes `copy of n`. The task does not use `t`, the loop's own variable, so it
needs no word for it. `total` is the real variable (row 6), changed by `fetch_add`, the `Atomic`'s own
operation, which any name may use (R5); what the task changes is worked out from its body (R5, verdict
86). Two tasks changing `total` this way is not E7007: the atomic instruction orders each change (R9,
R15).

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `texts` (1) | none (default: a plain parameter reads the caller's list) | none (default: read-only) |
| `total` (2) | none (default: a new local, the function's own) | none (default: an `Atomic` changes only through its own operations, R5) |
| `scope` (3) | `with` | none (default: read-only binding) |
| `t` (4) | none (default: a read-only view of each element) | none |
| `n` (5) | none (default: a new local of this round, its own) | none (default: its own) |
| `n` (6, capture) | `copy of` (the task's own copy, row 6) | `copy of` (the task's own) |
| `total` (6, in the task) | none (default, row 6: the real `total`) | none (default: an `Atomic` changes through any name by its own operations, R5; `fetch_add` is the change, written) |

7 names, 5 without a word (`texts`, `total` (2), `t`, `n` (5), `total` (6)).

**5. Means exactly** (`means/an73_means.em`):

```ember
fn count(texts: Array[String]) -> int:
    total = Atomic(0)
    tr = ref total
    with scope = thread.scope():
        for t in texts:
            n = t.split_whitespace().count()
            scope.spawn(owned fn() => tr.fetch_add(n))
    return total.load()
```

```text
exit=0
```

Today's `owned fn` copies `n` (an `int`) and copies the view `tr` of the real `total`: the task has its
own `n` and adds to the real total.

**6. Still refused:**
- The task without the copy word, once the lookup is fixed: `scope.spawn(fn(): total.fetch_add(n))`:
  `error[E3026]: the task uses \`n\`, which ends at the end of this round of the loop (line 5), while
  the task may still be running; give the task its own copy: \`scope.spawn(fn() copy of n:
  total.fetch_add(n))\` in place of line 6` (R12, R15; the rules' E3026 row, in this program's lines).
  It does not offer `ref n`, which would be refused the same way (`n` ends with the round). This is
  E3026, "a function value that may outlive a place it uses and does not own" (R12); run 4 gave it
  E3060, whose meaning since v2 is a view used after its target changed.

**7. Cost:** one atomic add per task, as the program asks (an `Atomic` is the programmer's choice;
`total += n` on a plain `int` would be AN-71's reduction at plain C's cost). The copy of `n` is one
8-byte store into the task's state.

**8. Kind:** a smarter compile-time solution (a defect fixed, and the real cause named with row 6's
words).

---

### AN-74: a `@sync` class freezes every field after `init`; objects written by several tasks need a lock or a claim

**v5:** each task looks at the loop's text with `ref t` (verdict 93) and hands `add_all` that view; v4's
`copy of t` stays allowed. `add_all` writes no `changes words, tags`, and the task no `changes stats`
(verdict 86): what they change is worked out from their bodies, and v4's two E2105 refusals go with
their code. E7007's message gives only the lock (F-10, as the rules' row now does), and E7010's only its
first way out (T-W6 11: the second, guarding the fields of objects with several owners, would run into
E7015). **Meaning changed:** yes, verdict 93: each task hands `add_all` a view of its text in `texts`,
where v4's handed it a view of a copy of its own; the program prints what it printed.

**1. Intent:** several tasks update one statistics object; and each task updates its own player.

**2. OLD** (`old/an74_sync_class.em`, reader 03's t5):

```ember
from std.sync import Atomic

@sync
class Stats:
    words: Atomic[int] = Atomic(0)
    total: int = 0

    fn bump(self):
        self.words.fetch_add(1)
        self.total = self.total + 1
```

```text
error[E7003]: cannot write field `total` of an `@sync` class after `init` or after using `self` as a whole
  --> an74_sync_class.em:11:9
   |
11 |         self.total = self.total + 1
   |         ^^^^^^^^^^
   |
   = help: wrap `total` in `Atomic[int]` for mutation after `init`

error: could not compile due to 1 error(s)
exit=1
```

and each task changing its own class object (`old/an74_parallel_class.em`):

```ember
class Player:
    hp: int = 10

fn hurt_all(players: Array[Player]):
    @parallel
    for p in players:
        p.hp -= 1
```

```text
error[E7010]: a `@parallel` loop counts over a range `a..b`
 --> an74_parallel_class.em:7:14
  |
7 |     for p in players:
  |              ^^^^^^^
  |
  = help: make the iterations independent: an `Atomic`, `chunks_mut`, or `@parallel(reduce=…)`
  = note: a `@parallel` loop's chunks run at once [PAR-2]

error: could not compile due to 1 error(s)
exit=1
```

Last run's answers: a lock field whose guarded fields the compiler infers (`phase3b-threads.md`
4.5), and for class objects in a parallel loop a per-object "claim" at 1.12x to 1.36x of plain C, 1.9x
to 4.0x when the objects are also held elsewhere (the attack, 3.5; `FINDINGS-phase3b.md:94`).

**3. NEW:**

```ember
from std.sync import Lock

class Stats:
    lock: Lock = Lock()
    words: int guarded by lock = 0
    tags: Array[String] guarded by lock = []

    fn add_all(text: StrView):
        for w in text.split_whitespace():
            with this.lock:
                this.words += 1
                if w.starts_with("#"):
                    this.tags.push(String(w))

fn count(texts: Array[String]) -> int:
    stats = Stats()
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn() ref t: stats.add_all(t))
    with stats.lock:
        return stats.words

class Player:
    hp: int = 10
    x: int = 0

fn hurt_all(ref mut players: Array[Player]):
    @parallel
    for ref mut p in players:
        p.hp -= 1
        p.x += p.hp & 1
```

The rules:
- **`guarded by lock`** on a field (R15): the field is read and changed only inside `with x.lock:` on
  the same object `x` (the same name, not given another object in the block; `this` in methods), on
  the thread that holds it, except in `init`. Several fields may name one lock; a class may have several
  locks. A helper whose body uses a guarded field without taking the lock is compiled as "needs
  `this.lock` held" (run 3's attack; a fact worked out from the body and kept in its summary, and in the
  package's interface file across a package's edge, R17) and may be called only with the lock held. A
  `with` on a lock this thread is known to hold calls the callee's entry that skips taking it again (the
  attack's static re-entry; no run-time owner). A view of a guarded field does not leave the `with` block
  (row 4's rule, R8 (e)).
- `add_all` changes `this.words` and `this.tags`, under the lock, and its first line does not say so
  (row 3's method, verdict 86): the compiler works it out from the body (R5). Its object is `this` and is
  not listed among its parameters. The method is called on the object with no mark (`stats.add_all(t)`,
  R4). The task uses the real `stats` (row 6) and changes it through `add_all`, only under its lock, so
  two tasks changing it is not E7007 (R15). It looks at this round's text with `ref t` (R15, verdict 93)
  and hands `add_all` that view; `texts` does not change until every task has ended. Starting a task
  counts as making what it changes (R15), a change of `count`'s own local.
- No class mark: `@sync` is gone (AN-80). Fields not written after `init` need nothing.
- **Each task its own object**: `for ref mut p in players` in a `@parallel` loop hands each chunk its
  own elements. An element of `Array[Player]` has one owner, the list, because a second owner of a
  class object needs a word (R1: one owner unless a line says otherwise; run 3's clash 15, standing:
  `p2 = player` hands over; a list holding one object twice is `Array[shares mut Player]`). So no two
  chunks can reach one object, and no run-time claim is needed. (Run 3 kept the attack's per-object claim as a
  fallback in case area C allowed silent sharing of class objects; R1 settles that it does not, so the
  fallback is not needed.)

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `lock` (4) | none (default: the object's own lock) | none (default: a `Lock` changes only by being held with `with`) |
| `words` (5) | none (default: the object's own number) | `guarded by lock` (only while holding `lock`) |
| `tags` (6) | none (default: the object's own list) | `guarded by lock` |
| `this` (8, not listed: the method's object, R5) | `this` (the object `add_all` was called on) | none (default: what a method does to its object is worked out from its body, R5; lines 11 and 13 change `this.words` and `this.tags`, under the lock) |
| `text` (8) | `StrView` (a view of someone else's text) | none (default: read-only) |
| `w` (9) | none (default: a read-only view of each word) | none |
| `texts` (15) | none (default: reads the caller's list) | none |
| `stats` (16) | none (default: a new local, the function's own object) | none (default: its own; changed through `add_all`, under the lock) |
| `scope` (17) | `with` | none |
| `t` (18) | none (default: a read-only view of each element) | none |
| `stats` (19, task) | none (default, row 6: the real `stats`) | none (default, row 6: the real variable, which the task may change; it does, through `add_all`, under the lock) |
| `t` (19, capture) | `ref` (a view of this round's text, an element of `texts`, R15, verdict 93) | `ref` (read-only; `add_all` only reads it) |
| `players` (27) | `ref mut` (the caller's list) | `ref mut` (may change it) |
| `p` (29) | `ref mut` (each element of the caller's list, alone while used, R8 (f)) | `ref mut` |

14 names, 6 without a word (`lock`, `w`, `texts`, `stats` (16), `t` (18), `stats` (19)).

**5. Means exactly** (`means/an74_means_lock.em`, today's Ember): the guarded fields inside one
`Mutex`, the class `@sync`:

```ember
struct StatsData:
    words: int
    tags: Array[String]

@sync
class Stats:
    data: Mutex[StatsData] = Mutex(StatsData(words=0, tags=[]))

    fn add(self, w: str):
        with d = self.data.lock():
            d.words += 1
            if w.starts_with("#"):
                d.tags.push(w.to_string())
```

(with `count_into` and three scoped tasks in `main`, as in the file)

```text
exit=0
```

and `means/an74_means_each.em` for the players, each element changed by its own chunk with plain
writes:

```ember
struct Player:
    hp: int
    x: int

fn hurt_all(mut ps: MutSpan[Player]):
    @parallel
    for i in 0..ps.len():
        hp = ps[i].hp - 1
        ps[i] = Player(hp=hp, x=ps[i].x + (hp & 1))
```

```text
exit=0
```

(Today's `[PAR-2]` refuses `ps[i].hp -= 1`, a write to a field of the loop's own element:
`old/an74_parallel_field_write.em` gives `error[E7010]: the parallel loop writes through a place its
chunks share` at `ps[i].hp -= 1`, `exit=1`. The translation writes the whole element, the same stores.
Found in passing: section 10.)

**6. Still refused:**
- The task using the loop's own variable with no word, `scope.spawn(fn(): stats.add_all(t))`:
  `error[E3026]: the task on line 19 may still run in a later round, when \`t\` names another text; give
  it its own copy, \`scope.spawn(fn() copy of t: stats.add_all(t))\`, or let it look at this round's
  text, \`scope.spawn(fn() ref t: stats.add_all(t))\`, in place of line 19` (R15, verdict 93; the rules'
  E3026 row).
- A guarded field used outside its lock: `stats.words += 1` in `count`: `error[E7013]: \`words\` is
  guarded by \`lock\`; change it inside \`with stats.lock:\``.
- The lock taken on another object: `with a.lock: b.words += 1`: `error[E7013]: \`b.words\` is
  guarded by \`b.lock\`, and this block holds \`a.lock\`; write \`with b.lock:\``.
- A lock held by the parent "protecting" tasks it spawns (the attack's `y14`): the task body holds no
  lock, so `stats.words += 1` in it gets the first message.
- A field that two tasks read and then change, with no lock and no `guarded by`, in this refused
  program:

  ```ember
  class Stats:
      total: int = 0

  fn count(texts: Array[String]) -> int:
      stats = Stats()
      with scope = thread.scope():
          for t in texts:
              scope.spawn(fn() ref t: if stats.total < 10: stats.total += t.len())
      return stats.total
  ```

  `error[E7007]: two tasks change \`stats.total\` (line 8), and each reads it first (\`if stats.total <
  10:\`), so each may see a total the other is changing; guard it with a lock, which keeps the read and
  the change together: write \`from std.sync import Lock\` above line 1; in \`Stats\` write \`lock: Lock
  = Lock()\` and \`total: int guarded by lock = 0\` in place of line 2, and \`fn add_below(limit: int, n:
  int):\` whose body is \`with this.lock:\`, inside it \`if this.total < limit:\` and inside that
  \`this.total += n\`; write \`scope.spawn(fn() ref t: stats.add_below(10, t.len()))\` in place of line
  8; and in place of line 9 write \`with stats.lock:\` and inside it \`return stats.total\`` (R15, verdict
  77; the rules' E7007 row, in this program's lines, with the import, and the read after the scope inside
  the lock, so that the lines compile). It gives only the lock (F-10): an atomic read and an atomic add
  would be two steps, so two tasks could both read 9 and both add, the race AN-80's OLD shows for
  `longest`.

  Accepted (verdict 77): the same program with `scope.spawn(fn() ref t: stats.total += t.len())` on
  line 8, a task that adds to the field and never reads it: each task adds to a private sum, and the sums
  are added to `stats.total` at the end of the scope, in spawn order; nothing reads `stats.total` before
  then, and line 9 reads it after the join (R15).
- A list whose objects may have a second owner in a parallel loop: `for ref mut p in players` with
  `fn hurt_all(ref mut players: Array[shares mut Player]):` on line 27: `error[E7010]: \`players\` holds
  \`shares mut Player\` objects, and one object may be in the list twice, so two chunks could change it
  at once; give each player one owner: write \`fn hurt_all(ref mut players: Array[Player]):\` in place of
  line 27` (R15; the rules' E7010 row, in this program's lines). It does not offer v4's second way out,
  a lock guarding `Player`'s fields: the chunks run on other threads, and a `shares mut` owner stays on
  one (E7015), so that line would run into another error (T-W6 11, verdict 73).

**7. Cost.** A lock where the program writes one, the lock C takes (last run, `phase3b-threads.md:741`,
`lock-0`, lock-use-unlock in C, 1479.479 ms against safe Rust's `Mutex` 2170.016 ms, PROVISIONAL);
no field is wrapped, so the object is its plain fields plus one lock word. One thread that uses such a
class pays an uncontended lock per `with`, as Python and C pay. `ref t` hands each task a pointer to its
text, as C does: no copy. The parallel loop over one-owner elements: plain loads and stores, no count
test and no claim: the per-object claim's 1.12x to 1.36x (the attack, 3.5) is gone because the words
prove what the claim checked while running.

**8. Kind:** a readable spelling of Rust's guarantee (`Mutex<Inner>` spelled as a lock field, with the
field saying which lock guards it), plus a smarter compile-time solution for the per-element loop.

---

### AN-75: clone-before-move for shared handles (A34)

**v5:** the threads' functions, `start` and `main` write no `changes Log` (verdict 86): what they change,
the log's `lines` under its lock, is worked out from their bodies, and v4's three E2105 refusals go with
their code. `thread.spawn` takes a function any thread may run (verdict 94), which these are: they hold
`thread_shared` owners. The call `dump(log, fn() thread_shared log: log.push("seen"))`, a plain argument
and a function value that both reach one log, is allowed (verdict 89; v4 left it for the owner): the
log changes only through its lock. E7018's two messages are the rules', with the line to move.
**Meaning changed:** no for the NEW program. In part 6, the `dump` call v4 left open is allowed (verdict
89), and a call of `note()` inside the lock stays refused (E7018).

**1. Intent:** two threads append to one log.

**2. OLD** (`old/a34_clone_dance.em`, phase 1's):

```ember
import std.thread
from std.sync import Mutex

@sync
class Log:
    lines: Mutex[Array[String]] = Mutex([])

fn push(log: Log, s: str):
    with g = log.lines.lock():
        g.push(s.to_string())

fn start(log: Log):
    a = thread.spawn(owned fn() => push(log, "a"))
    b = thread.spawn(owned fn() => push(log, "b"))
    a.join()
    b.join()

fn main():
    start(Log())
```

```text
exit=0
```

It compiles only because `owned fn` silently retains each class handle it captures. Once `owned fn`
goes (vocabulary, `owned fn` row: removed), the same text with plain `fn()`
(`old/an75_no_owned_fn.em`) gives:

```text
error[E3026]: the thread's function borrows `log`, which it may outlive
  --> an75_no_owned_fn.em:14:22
   |
14 |     a = thread.spawn(fn() => push(log, "a"))
   |                      ^^^^^^^^^^^^^^^^^^^^^^ the thread may outlive what it views
   |
   = help: write it `owned fn`, which moves or copies each capture into it; or borrow it inside `with scope = thread.scope():`, which joins its threads before the block ends [THR-5]
   = note: a thread may outlive every local of the code that started it [THR-10]

error[E3026]: the thread's function borrows `log`, which it may outlive
  --> an75_no_owned_fn.em:15:22
...
error: could not compile due to 2 error(s)
exit=1
```

and Rust's way out is the dance: `let log2 = Arc::clone(&log); thread::spawn(move || ...)`.

**3. NEW:**

```ember
from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView):
        with this.lock:
            this.lines.push(String(s))

fn start(log: thread_shared Log):
    a = thread.spawn(fn() thread_shared log: log.push("a"))
    b = thread.spawn(fn() thread_shared log: log.push("b"))
    a.join()
    b.join()

fn main():
    start(thread_shared Log())
```

The capture word does what Rust's `Arc::clone` line does, on the name it is about: each thread gets
one more owner. No second name (`log2`) and no separate clone line. `log: thread_shared Log` reads the
caller's value with no count (R4); `thread_shared log` in the capture list makes one more owner from it
(R3, allowed because the value is already `thread_shared`). Each thread changes the log only through
`push`, which changes `lines`, a field `guarded by lock`, inside `with this.lock:` (R9: the one way a
`thread_shared` value is changed). What the threads' functions change, and what `start` and `main`
change by starting them, is worked out from the bodies (R5, R15; verdict 86: no first line says it).
`thread.spawn` takes a function any thread may run, `[R]` above `pub fn spawn(owns f: thread_shared fn()
-> R) -> JoinHandle[R]:` (R15, verdict 94); a function that holds only `thread_shared` owners is one.
Nothing more is checked while running: the rules are checked when compiling. `start`'s read-only `log`
promises that what it reads does not change while it is used (R4, R8 (g)); `lines` is read and changed
only under its lock, as a `Mutex`'s value is, which is the reading section 2 states (`guarded by` as the
lock's value).

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `lock` (4) | none (default: the object's own lock) | none (default: changes only by being held) |
| `lines` (5) | none (default: the object's own list) | `guarded by lock` |
| `this` (7, not listed: the method's object, R5) | `this` (the `Log` `push` was called on) | none (default: worked out from its body, R5; line 9 changes `this.lines`, under the lock) |
| `s` (7) | `StrView` (a view of someone else's text) | none (default: read-only) |
| `log` (11) | `thread_shared` (in its type: a log shared between threads; a plain parameter, so a view of the caller's, no count, R4) | `thread_shared` (no owner changes it directly; `push` changes a guarded field, under its lock) |
| `a` (12) | none (default: the `JoinHandle` the call returns, the caller's own) | none |
| `log` (12, capture) | `thread_shared` (one more owner) | `thread_shared` (it changes the log only through `push`, under the lock) |
| `b` (13) | none (default as `a`) | none |
| `log` (13, capture) | `thread_shared` | `thread_shared` |

9 names, 3 without a word (`lock`, `a`, `b`).

**5. Means exactly** (`means/an75_means.em`, which is phase 1's `a34_clone_dance.em` itself): a
`@sync` class (atomic counts), its list inside a `Mutex`, and `owned fn` retaining the handle once per
thread. `exit=0`.

**6. Still refused:**
- A call through a function value that may take the log's lock, inside a walk of the log's list under
  that lock (verdict 29's gain), in this refused program, written below the NEW program's `Log` (its
  line numbers are this block's):

  ```ember
  fn dump(log: thread_shared Log, note: fn()):
      with log.lock:
          for line in log.lines:
              println(line)
              note()

  fn report(log: thread_shared Log):
      dump(log, fn() thread_shared log: log.push("seen"))
  ```

  `error[E7018]: this loop walks \`log.lines\` (line 3) under \`log\`'s lock, and \`note()\` (line 5)
  may run the function given on line 8, which takes a \`Log\`'s lock (\`log.push\`, under \`lock\`), so
  it would wait for the lock this thread holds; call \`note()\` after the \`with log.lock:\` block: move
  line 5 below the block` (R15, R18, verdict 73; the rules' E7018 row, "in a walk", in this program's
  lines). The line also fits E3020 (a call that may change the walked list, R8 (c)); E7018 is shown
  because its fix compiles and runs, while E3020's, after the loop and still inside the lock, would wait
  for ever. With line 5 moved below the block the program is allowed, the call on line 8 included: `log`
  reaches that call twice, as a plain argument and inside the function value, and a thing whose changes
  go only through its lock may reach one call through several names (R4, exception 2; verdict 89, as
  Rust accepts the same with `&Log` and a `Mutex`). v4 left that call for the owner.
- A thread that takes the log over, `a = thread.spawn(fn() owns log: log.push("a"))` on line 12:
  `error[E3013]: \`log\` is lent to \`start\` (a plain parameter), so the thread cannot take it over
  (\`fn() owns log\`, line 12); to give each thread one more owner of the log, write \`a =
  thread.spawn(fn() thread_shared log: log.push("a"))\` in place of line 12`. (Run 4 gave this E3040,
  "handed to the first thread"; under R3 and E3013 a name that does not own the value cannot hand it
  over at all, so the refusal comes at the first capture.)
- A one-thread owner sent to a thread, `fn() shares log` on line 12: `error[E7015]: \`fn() shares log\`
  makes an owner of \`log\` that stays on one thread, and \`thread.spawn\` runs its function on a new one
  (it takes a \`thread_shared fn()\`); write \`a = thread.spawn(fn() thread_shared log: log.push("a"))\`
  in place of line 12` (R12, R15, verdict 94).
- No capture word, `a = thread.spawn(fn(): log.push("a"))` on line 12: `error[E7004]: this thread may
  outlive \`start\`, so it cannot use \`start\`'s \`log\`; say what the thread holds: write \`a =
  thread.spawn(fn() thread_shared log: log.push("a"))\` in place of line 12 (one more owner of the
  log)` (AN-71's E7004, in this program's names; `owns log` is not offered, since `start` does not own
  the log).
- `thread_shared log` where `log` is a plain parameter `log: Log` (a view of an object whose owner may
  be one-thread), with `start(Log())` in `main`: `error[E7017]: \`log\` is lent to \`start\`, and its
  owner may not be shared between threads; take it as \`fn start(log: thread_shared Log):\` in place of
  line 11, and create the log shared in \`main\`: \`start(thread_shared Log())\` in place of line 18`.
- `log.lines.push(String("c"))` in `start`, outside the lock: `error[E7013]: \`lines\` is guarded by
  \`lock\`; change it inside \`with log.lock:\`, or call \`log.push("c")\``.

`with log.lock:` around `note()` with no walk, where `note` runs a function that pushes to the same log,
would wait for ever while running (`push` takes the lock its own thread holds; AN-74's static re-entry
covers only direct calls, not a call through a function value). It is refused when compiling (R15,
verdict 43), E7018: "`note()` may run the function given on line 8, which takes a `Log`'s lock
(`log.push`, under `lock`), and this thread holds `log`'s lock, so taking it again would wait for ever;
call `note()` after the `with log.lock:` block: move line 5 below the block". C and Rust hang there
(Rust's `Mutex::lock` does not return when the thread already holds it).

**7. Cost:** one atomic increment per `thread_shared` capture, after one compare of the count with its
maximum (verdict 59, R9: at the maximum the program stops, "too many owners of a `Log`
(4,294,967,295)"), and one atomic decrement when the thread's owner ends: exactly Rust's `Arc::clone`,
which makes the same compare, and drop; today's `owned fn` does the increment and the decrement for a
`@sync` handle. The lock is C's. E7018 and what each function changes cost nothing while running: they
are worked out when compiling.

**8. Kind:** a readable spelling of Rust's guarantee (`Arc::clone` + `move`, written as one word on the
captured name).

---

### AN-76: global mutable state (A19)

**v5:** no first line says what a function changes (verdict 86): `fn tick():`, `fn worker(x: int):`,
`fn main():`, the threads' `fn(): worker(1)`, and in the button's block `type Handler = fn()`,
`fn on_click(owns f: Handler):`, `fn click():`, `fn report(ref mut b: Button):` and the handler
`fn(): LOG.push(String("clicked"))`. What each changes is worked out from its body, through the
functions it calls, the functions the program stores in a function value it calls, and the threads it
starts (R5, R12, R15). v4's six E2105 refusals and its E2113 one go with their codes: v4's refused
button, which kept a `LOG`-changing handler in an `Array[fn()]`, is now the NEW button itself. What stays
refused is refused as before, worked out: a loop over `LOG` across `tick()` or `b.click()` (E3020, the
message naming the chain and the line to move, F-12), a worker thread reaching the unguarded `FRAME` and
`LOG` (E7012), and a join inside the lock of a thread started in a walk under it (E7018). `global`'s
E0100 message is the rules' new one, and `thread.spawn` takes a `thread_shared fn()` (verdict 94), which
`fn(): worker(1)`, capturing nothing, is. **Meaning changed:** no: the same lines are refused. A loop
over `LOG` may call `b.click()` when no handler the program stores in a `Button` changes `LOG`, which v4
decided by the list's type (`Array[fn()]`).

**1. Intent:** count frames and keep a log in module variables; worker threads add results to a
module list.

**2. OLD** (`old/a19_globals.em`, phase 1's):

```ember
static FRAME: int = 0
static LOG: Array[String] = []

fn tick():
    FRAME += 1
    LOG.push("tick".to_string())
```

```text
error[E1010]: cannot find `FRAME` in this scope
 --> a19_globals.em:7:5
  |
7 |     FRAME += 1
  |     ^^^^^
  |

error[E3021]: cannot write through a shared reference
 --> a19_globals.em:8:5
  |
8 |     LOG.push("tick".to_string())
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ this is `ref Array[String]`, which only reads
  |
  = help: take the borrow as `ref mut` if the callee must write, or assign to the place itself rather than through the reference
  = note: while a shared borrow is live the owner may read and copy but not write [BRW-1]

error: could not compile due to 2 error(s)
exit=1
```

`static mut` (`old/an76_static_mut_today.em`):

```text
error[E1010]: `static mut` needs `unsafe`, which is not in this phase
 --> an76_static_mut_today.em:2:12
  |
2 | static mut FRAME: int = 0
  |            ^^^^^
  |

error[E1010]: `static mut` needs `unsafe`, which is not in this phase
 --> an76_static_mut_today.em:3:12
...
error[E1010]: cannot find `FRAME` in this scope
...
error[E1010]: cannot find `LOG` in this scope
...
error: could not compile due to 4 error(s)
exit=1
```

Python's spelling (`old/an76_module_assign.em`, `FRAME = 0` at module level):

```text
error[E1030]: `main` is already declared in this module
 --> an76_module_assign.em:2:1
  |
2 | FRAME = 0
  | ^^^^^^^^^
  |

error[E1010]: cannot find `FRAME` in this scope
...
error[E1010]: cannot find `LOG` in this scope
...
error: could not compile due to 3 error(s)
exit=1
```

**3. NEW:**

```ember
static mut FRAME: int = 0
static mut LOG: Array[String] = []

fn tick():
    FRAME += 1
    LOG.push(String("tick"))

static RESULTS_LOCK: Lock = Lock()
static mut RESULTS: Array[int] guarded by RESULTS_LOCK = []

fn worker(x: int):
    r = x * x
    with RESULTS_LOCK:
        RESULTS.push(r)

fn main():
    tick()
    a = thread.spawn(fn(): worker(1))
    b = thread.spawn(fn(): worker(2))
    a.join()
    b.join()
```

and the button of verdict 17, whose handlers push to the log:

```ember
type Handler = fn()

class Button:
    handlers: Array[Handler] = []

    fn on_click(owns f: Handler):
        this.handlers.push(f)

    fn click():
        for ref mut h in this.handlers:
            h()

fn setup(ref mut b: Button):
    b.on_click(fn(): LOG.push(String("clicked")))

fn report(ref mut b: Button):
    for entry in LOG:
        println(entry)
    b.click()
```

The rules (R5, R12, R15):
1. `static mut NAME: T = e` declares a module variable that may change. It needs no `unsafe`; a
   foreign `static mut` (in an `extern` block) keeps `[STA-1]`'s `unsafe` at every use. The
   initialiser follows `[STA-3]`, as today's `static`s.
2. **What a function changes is worked out from its code** (R5, verdict 86), with no word on its first
   line: `tick` changes `FRAME` and `LOG` (its lines 5 and 6); `worker` changes `RESULTS`, under
   `RESULTS_LOCK`; `main` changes `FRAME` and `LOG` by calling `tick()`, and `RESULTS` by starting
   threads that run `worker`: **starting a thread counts as making what its function changes, now or
   later** (R12, R15; verdict 30: "who said a change needs to be bound by a moment?"). The rules below
   use that fact; the threads' safety is rule 4's and E7012's. **Storing a lambda is not changing what
   it changes**: `setup` hands the handler to `on_click`, which keeps it for `click` to run, so `setup`
   changes nothing by it (R12: the line falls at who runs the function; G-06); `click` runs it, so
   `click` changes `LOG`, and so do `report`, which calls `b.click()`, and every function that calls
   `report`.
3. One thread: a loop over `LOG`, or any view into it, across a call that may change `LOG` is refused
   (R8 (c), (g); the agreed limitation: change after the loop), and the message names the chain of
   calls. A call through a function value may change `LOG` when a function the program stores there
   does (R12: the compiler reads every function the program can put in `handlers`, a library's `Button`
   included): `b.click()` changes `LOG` because `setup` stores a handler that pushes to it. In a program
   whose stored handlers change nothing outside themselves, clicking inside a loop over `LOG` is allowed.
   A `spawn` inside a loop over what its thread changes under the loop's lock is allowed, its function
   taking its own copy of the loop's variable (`copy of r`, verdict 65), and joining that thread inside
   the lock is refused (E7018; verdict 52). A foreign call counts as possibly running every callback C
   keeps, and makes the changes they make, unless the foreign function's declaration says
   `@ffi(no_callbacks)` (R12, verdict 31; the attack's 5.1, `y16`; AN-78).
4. Threads: a `static mut` is used only by the main thread and the tasks of scopes it opens (where
   tasks' changes follow the scope rule and reductions, AN-71). A function reachable from
   `thread.spawn`, an unscoped job of the job system, a C callback whose contract is not
   `threads(main)`, or an `@export(threads=any|creator)` may not read or change it (E7012) unless it
   is `guarded by` a `static` lock, inside `with` that lock.
5. A `static` of a type that changes only through itself (`Lock`, `Atomic`, `Mutex`, `RwLock`, a
   channel end) is usable from any thread, as today.

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `FRAME` (1) | `static` (the module's own, one per program) | `mut` |
| `LOG` (2) | `static` | `mut` |
| `FRAME` (5) | `static` (line 1) | `mut` (line 1: the module variable may change); `+=` is the change, written |
| `LOG` (6) | `static` (line 2) | `mut` (line 2); `push` is the change, written |
| `RESULTS_LOCK` (8) | `static` | none (default: a `Lock` changes only by being held) |
| `RESULTS` (9) | `static` | `mut`, `guarded by RESULTS_LOCK` |
| `x` (11) | none (default: a plain parameter reads the caller's value; an `int` is copied in, R4) | none (default: read-only) |
| `r` (12) | none (default: a new local, the function's own) | none (default: its own) |
| `RESULTS` (14) | `static` (line 9) | `guarded by RESULTS_LOCK` (line 9), held by line 13's `with`; `push` is the change, written |
| `a` (18) | none (default: the `JoinHandle` the call returns, the caller's own) | none |
| `b` (19) | none (default as `a`) | none |

The two lambdas of lines 18 and 19 declare no name and capture nothing; what they change, `RESULTS`
through `worker`, is worked out (R5). In the button's block:

| name (line) | what it is | may it change it |
|---|---|---|
| `Handler` (1) | `type` (a name for the function type written after `=`, verdict 48) | none (a type changes nothing; what its functions change is worked out from those the program stores, R12) |
| `handlers` (4) | none (default: the object's own list) | none (default: what may change it is worked out from the methods' bodies, R5; `on_click` pushes to it) |
| `this` (6, not listed: the method's object, R5) | `this` (the `Button` `on_click` was called on) | none (default: worked out from its body; line 7 pushes to `this.handlers`) |
| `f` (6) | `owns` (the function is handed to `on_click`) | `owns` (its own) |
| `this` (9, not listed) | `this` (the `Button` `click` was called on) | none (default: worked out from its body; line 11 runs each stored handler through `h`) |
| `h` (10) | `ref mut` (each stored function, the list's own) | `ref mut` (a call may change the function's own state, R12) |
| `b` (13) | `ref mut` (the caller's button) | `ref mut` |
| `b` (16) | `ref mut` (the caller's button) | `ref mut` |
| `entry` (17) | none (default: a read-only view of each line of `LOG`, R2) | none (default: read-only; `LOG` does not change while the loop walks it, R8 (c)) |

20 names in the two blocks, 6 without a word (`x`, `r`, `a`, `b` in the first block; `handlers`,
`entry` in the second), each with its default stated.

**5. Means exactly.** Today's Ember cannot say a changeable module variable without `unsafe` (the OLD
output above). In C (`c/an76_static_mut.c`): plain globals, `tick()` changing them, the guarded list
changed only under an SRW lock (the runtime's lock word) by four worker threads:

```c
static int64_t FRAME = 0;                 /* static mut FRAME: int = 0 */
static StrArray LOG = {0};                /* static mut LOG: Array[String] = [] */
static SRWLOCK RESULTS_LOCK = SRWLOCK_INIT;   /* static RESULTS_LOCK: Lock = Lock() */
static IntArray RESULTS = {0};            /* static mut RESULTS: Array[int] guarded by RESULTS_LOCK */

__declspec(noinline) static void tick(void) {   /* fn tick() changes FRAME, LOG: */
    FRAME += 1;
    str_push(&LOG, "tick");
}

static DWORD WINAPI worker(LPVOID p) {          /* fn worker(x: int): */
    int64_t x = (int64_t)(intptr_t)p;
    int64_t r = x * x;
    AcquireSRWLockExclusive(&RESULTS_LOCK);     /* with RESULTS_LOCK: */
    int_push(&RESULTS, r);
    ReleaseSRWLockExclusive(&RESULTS_LOCK);
    return 0;
}
```

Built with `clang -O2 -I target/redesign/bench`, run with `timeout 300 ./an76_static_mut.exe`
(`main` calls `tick()` 1,000,000 times and starts 4 workers with `x` = 0..3):

```text
an76_static_mut ms=26.241 peak_kb=27884 checksum=2000014
```

(checksum = `FRAME` 1,000,000 + `LOG.len()` 1,000,000 + the sum of `RESULTS` 0 + 1 + 4 + 9 = 14.)

(The C file's comments quote v2's first lines, `fn tick() changes FRAME, LOG:`; the NEW program writes
`fn tick():`, verdict 86. The C program is the same either way.)

**6. Still refused:**
- A loop over `LOG` that calls `tick()`, `for entry in LOG:` (line 18) with `tick()` (line 19) in its
  body, written in `main` below line 17: `error[E3020]: \`tick()\` (line 19) adds to \`LOG\` (\`tick\`'s
  line 6) while this loop walks \`LOG\` (line 18); call \`tick()\` after the loop: move line 19 below the
  loop` (R8 (c), R5; the rules' E3020 row, in this program's lines).
- A worker thread reaching the unguarded ones, `thread.spawn(fn(): tick())`: `error[E7012]: \`tick\`
  changes \`FRAME\` and \`LOG\`, module variables of the main thread, and this thread runs beside it;
  guard both with one lock: write \`static TICK_LOCK: Lock = Lock()\` above line 1, \`static mut FRAME:
  int guarded by TICK_LOCK = 0\` in place of line 1 and \`static mut LOG: Array[String] guarded by
  TICK_LOCK = []\` in place of line 2, and put \`tick\`'s body, lines 5 and 6, inside \`with
  TICK_LOCK:\`` (R15; the rules' E7012 row). The program already uses `Lock` (line 8), so the message
  names no import; in a program that does not, the rules' row adds `from std.sync import Lock` at the
  top. In the whole program, with the button's block, the message also puts that block's two uses of
  `LOG` inside `with TICK_LOCK:` (line 14's push, inside its handler, and the loop of lines 17 and 18),
  since every read and change of a guarded variable is made under its lock (E7013).
- `FRAME = 0` at module level (Python's spelling): `error[E0100]: \`FRAME = 0\` is a statement, and a
  module holds declarations; for a module variable that may change write \`static mut FRAME: int =
  0\`` (the rules' E0100 row; v3 gave it E1030, which keeps one meaning, two items with one name,
  worklist section 2; today's compiler gives the E1030 pasted in part 2).
- `global FRAME`, Python's spelling, in this refused program:

  ```ember
  static mut FRAME: int = 0

  fn tick():
      global FRAME
      FRAME += 1
  ```

  today: `error[E0100]: \`global\` is Python's; Ember declares no names as shared` (probed by the
  lead, `target/redesign4/lead-phase3/global_frame.em`); now: `error[E0100]: \`global\` is Python's; in
  Ember a function changes a module variable by its name, with no word: remove line 4` (the rules' E0100
  row; v4's message gave `fn tick() changes FRAME:`, which verdict 86 removed).
- A thread started inside a walk of what it changes, then joined inside the lock: below line 21, `with
  RESULTS_LOCK:` (line 22), `for r in RESULTS:` (line 23), `c = thread.spawn(fn() copy of r:
  worker(r))` (line 24) and `c.join()` (line 25): `error[E7018]: \`c.join()\` (line 25) waits for the
  thread started on line 24, whose function changes \`RESULTS\` under \`RESULTS_LOCK\`, and this thread
  holds that lock, so each would wait for the other for ever; join after the \`with RESULTS_LOCK:\`
  block: move line 25 below the block` (R15, verdict 52; the rules' E7018 row). Without line 25 the
  program is accepted: the new thread waits for the lock until the walk ends, as Rust allows
  (`probes/rust_spawn_in_loop.txt`: rustc compiles both; the joining one, run, was still waiting after 5
  s). Line 24 takes its own copy of `r`, the loop's own variable, which the loop changes every round
  while the thread may still run (verdicts 65 and 83, R15); written without it, `c = thread.spawn(fn():
  worker(r))` is refused: `error[E7004]: the thread may outlive the loop, and \`r\` names another number
  each round; give it its own copy: \`c = thread.spawn(fn() copy of r: worker(r))\` in place of line 24`
  (R15; the rules' E7004 row). It does not offer `ref r`, which a `thread.spawn` thread may not take
  (E3026, verdict 93).
- A loop over `LOG` across a call that runs the stored handlers (run 3's `20-re-attacker.md`, F-04's
  program, in this block's names: `b.click()` moved into `report`'s loop, above `println(entry)`, so
  that the loop is line 17 and the call line 18): `error[E3020]: \`b.click()\` (line 18) adds to \`LOG\`
  (through \`click\`, which runs the handler stored on line 14, which pushes to \`LOG\`) while this loop
  walks \`LOG\` (line 17); click after the loop: move line 18, \`b.click()\`, below the loop` (R8 (c),
  R12; the rules' E3020 row, in this program's lines). The program that message leads to (the NEW's
  order, `b.click()` on line 19) is `RUN3/21-fixer/c/f4_fnvalue_static_loop.c` mode 2: clean under
  AddressSanitizer (run 3's proof, not run again).

Accepted: the same loop over `LOG` with `b.click()` inside it, in a program whose stored handlers change
nothing outside themselves (`b.on_click(fn(): println("clicked"))` on line 14, the only function the
program stores in a `Button`): the compiler reads every function the program can put in `handlers` (R12),
none changes `LOG`, so the walk is safe. v4 decided it by the list's type (`Array[fn()]` against
`Array[Handler]` with `changes LOG`); now no type says it (verdict 86), and v4's refusal of a
`LOG`-changing handler kept in an `Array[fn()]` (E2113) is gone: that program is the NEW button.

**7. Cost:** plain C globals: nothing counted, nothing checked while running. Last run measured the
same work (`phase3b-threads.md:854-863`, `tick()` called 100,000,000 times, PROVISIONAL): plain C
globals 192.219 ms (1.00); today's Ember workaround (`Atomic` + `Mutex`) 3.10x on one thread, 7.15x
once a second thread has started; safe Rust's `AtomicU64` + `Mutex` 7.44x; Rust's `thread_local!`
1.04x. The guarded list costs the lock C takes.

**8. Kind:** a smarter compile-time solution (Rust needs `unsafe` or atomics for this; here the compiler
works out what each function changes and proves that only one thread group reaches each unguarded
variable).

---

### AN-77: async and generators (A14)

**v5:** the refused generator's `Link` writes `owns` where it makes the list's next link,
`next=Some(owns Link(value=1))` (verdict 85: `owns` on the making line inside an `Option`; v4 left it for
the owner). The defaults in the owner's test no longer point at a `changes` clause (verdict 86), and the
in-place E3020 says where the line goes (F-12). No line of the NEW program changes: its generators
change nothing outside themselves, start no thread and have no `drop`. **Meaning changed:** no. What v2
settled stays: verdict 16 ("a, same rule for every loop") made a generator paused in the middle of a list
a loop over that list (R6, R8 (c)). What that settles, as run 4 had it:
1. **No "walk by position" way out.** A generator that walks by position (`while i <
   len(this.log):`) is a loop over the list too, so `g = d.replay()`, `d.log.push(...)`, `g.next()` is
   refused whichever way `replay` walks. A program that wants to add to the log between resumes
   collects the additions and pushes them after the generator's last use: the agreed limitation ("a
   list may not change while a loop walks it"), now the owner's for every loop.
2. **`from log` is optional.** A generator method's result views its whole object without it, so
   `gen fn replay() -> Generator[int]:` compiles; `from log` narrows it to `d.log`, so that changing
   another field of `d` stays allowed while `g` is used. This answer keeps `from log`, which says on
   the line what the generator walks.
What changed from run 4 (in v2) is where the refusal of a stored generator lands, and its code: at the
change, E3020, with the loop's own message ("push after `g`'s last use"), where run 4 gave E3060 at the
generator's later use.

**1. Intent:** a generator yields the lines of its own buffer; another yields the lengths of a
class object's log; an async server function.

**2. OLD** (`old/a14_generators.em`, phase 1's):

```ember
gen fn lines() -> Generator[String]:
    buf = ["a".to_string(), "b".to_string()]
    for s in buf:
        yield s.clone()

class Door:
    angle: float = 0.0
    log: Array[String] = []

    gen fn replay(self) -> Generator[int]:
        for entry in self.log:
            yield entry.len()
```

```text
error[E2221]: a borrow of `buf`, a local of this generator, is held across this `yield`
 --> a14_generators.em:7:15
  |
7 |         yield s.clone()
  |               ^ the generator suspends here
  |
  = help: iterate the collection `owned` (`for x in owned xs:`), or by index; borrows of the generator's parameters may be held
  = note: a suspended frame is a value that may move, which would leave the borrow pointing where the local was [CORO-6]

error[E2229]: a borrow of a field of a class object is held across this `yield`
  --> a14_generators.em:15:19
   |
15 |             yield entry.len()
   |                   ^^^^^ the generator suspends here
   |
   = help: copy what is needed out of the field before the `yield`, or iterate by index
   = note: a long-term access to an object's field may not span a suspension, when any other code may run [CORO-12]

error: could not compile due to 2 error(s)
exit=1
```

`old/a14_async_kw.em` (`async fn fetch() -> int:`):

```text
error[E0005]: `async` is reserved for v2
 --> a14_async_kw.em:3:1
  |
3 | async fn fetch() -> int:
  | ^^^^^
  |
  = help: rename the item, or write `r#async` to use the word as an identifier

error: could not compile due to 1 error(s)
exit=1
```

**3. NEW:**

```ember
gen fn lines() -> Generator[String]:
    buf = [String("a"), String("b")]
    for s in buf:
        yield copy of s

class Door:
    angle: float = 0.0
    log: Array[String] = []

    gen fn replay() -> Generator[int] from log:
        for entry in this.log:
            yield entry.len()

fn main():
    for s in lines():
        println(s)
    d = Door()
    for n in d.replay():
        println(n)
```

The rule (R15, R6): a `for` loop over a list, string, map or set whose body contains `yield` keeps a
position in the generator's frame and, after each resume, reads the collection's header again from
where it lives now (the frame, or the object's field) and checks the position against the current
length. No address is held across the suspension, so the frame may move. What the list may do while
the generator lives is R6's: the generator is a view of its object (of `log` alone, with `from log`),
so while it is still used nothing it views is reshaped, wherever the change is written, whether the
generator is consumed in place (`for n in d.replay():`) or stored (`g = d.replay()`, while `g` will
still be used): either way it is a loop over the list, and a change to it is E3020 at the change (R8
(c), verdict 16). The position kept in the frame stays: it is what lets the frame move.

`async` stays reserved in this version (understandable limitation: no async runtime exists). Its
design is run 2's (`phase3b-threads.md` 7.3 rule 3, the attack's 6.3): task groups, which are scopes;
frames that never move; the scope rule in place of `Send`.

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `buf` (2) | none (default: a new local, the generator's own) | none (default: its own) |
| `s` (3) | none (default: a read-only view of each element) | none |
| `s` (4, yielded) | `copy of` (row 9: a new, independent `String` is given out) | `copy of` (the receiver's own) |
| `angle` (7) | none (default: the object's own field) | none (default: what may change it is worked out from the methods' bodies, R5; no method here changes it) |
| `log` (8) | none (default: the object's own list) | none (default as `angle`) |
| `this` (10, not listed: the method's object, R5) | `this` (the `Door` `replay` was called on) | none (default: its body only reads `this.log`, worked out, R5; its first line does not say `owns this`) |
| `log` (10, in `from log`) | `from`: the field the generator `replay` returns walks (R6) | none (default: `replay` only reads it; while the generator will still be used, `d.log` does not change, R8 (c)) |
| `entry` (11) | none (default: a read-only view of each element) | none |
| `s` (15) | none (default: the `String` the generator gives, the loop's own) | none (default: its own) |
| `d` (17) | none (default: a new local, the function's own object) | none |
| `n` (18) | none (default: the `int` the generator gives) | none |

11 names, 8 without a word.

**5. Means exactly** (`means/an77_means.em`, today's Ember): the walk by position written by hand,
which reads `len()` again after each resume:

```ember
gen fn lines() -> Generator[String]:
    buf = ["a".to_string(), "b".to_string()]
    i = 0
    while i < buf.len():
        yield buf[i].clone()
        i += 1

class Door:
    angle: float = 0.0
    log: Array[String] = []

    gen fn replay(self) -> Generator[int]:
        i = 0
        while i < self.log.len():
            yield self.log[i].len()
            i += 1
```

```text
exit=0
```

That position walk is what the NEW `for` compiles to. Its meaning matches the NEW program's wherever
the NEW program compiles: a consumer that changed `d.log` while the generator lives, which this
translation would let see the change, is refused in the NEW program (6).

**6. Still refused:**
- A reference to the frame's own inline local across `yield`, in this refused program:

  ```ember
  gen fn numbers() -> Generator[int]:
      n = 5
      r = ref n
      yield 1
      yield r
  ```

  `error[E2221]: \`r\` (line 3) points into this generator's own frame, which may move while it is
  suspended at the \`yield\` on line 4; use \`n\` itself: remove line 3 and write \`yield n\` instead
  of line 5`.
- The in-place consumer changing the walked field, `for n in d.replay(): d.log.clear()`:
  `error[E3020]: \`d.log.clear()\` changes \`d.log\`, which this loop walks through \`d.replay()\`
  (\`from log\`); clear after the loop: move \`d.log.clear()\` below the loop`.
- A stored generator and a change to its list before its last use: `g = d.replay()` (line 1),
  `d.log.push(String("x"))` (line 2), `println(g.next())` (line 3): `error[E3020]:
  \`d.log.push(String("x"))\` (line 2) changes \`d.log\`, which \`g\` walks (\`g = d.replay()\`, line
  1), and \`g\` is used again on line 3; push after \`g\`'s last use: move
  \`d.log.push(String("x"))\` below line 3` (R8 (c), verdict 16; the rules' E3020 row for a generator, in
  this program's lines; no "walk by position" way out, since a position walk is refused the same way).
- A walk with no position (a linked structure held by views), in this refused program:

  ```ember
  struct Link:
      value: int
      next: Option[owns Link] = None

  gen fn countdown() -> Generator[int]:
      head = Link(value=2, next=Some(owns Link(value=1)))
      cur = ref head
      while true:
          yield cur.value
          if cur.next exists as nx:
              cur = ref nx
          else:
              break
  ```

  `error[E2221]: \`cur\` (line 7) views \`head\`, a local of this generator's own frame, which may
  move while it is suspended at the \`yield\` on line 9; a view of what the caller keeps may be held:
  take the list as a parameter, \`gen fn countdown(head: Link) -> Generator[int] from head:\` instead
  of line 5, and remove line 6` (today's E2221, whose help says "borrows of the generator's parameters
  may be held", in this program's lines). Line 6 writes `owns` before the `Link` it makes for `next`, an
  `Option[owns Link]` (verdict 85, R3; without it, E2123 on line 6), so the program's one mistake is
  `cur`.
- `async`: `error[E0005]: \`async\` is planned for a later version; for work that waits, use a
  generator or threads (\`with scope = thread.scope():\`)`.

No longer refused (run 3's E3062): `gen fn replay() -> Generator[int]:` with `for entry in this.log:`
across `yield`; the generator then views all of `d` (R6).

**7. Cost:** the measured model's own. The attack (`phase3b-threads-attack.md:766-775`, 104,857,600
values over a 1 MiB string, PROVISIONAL): plain C loop 69.943 ms (1.00); position re-read each resume,
`next()` inlined, 82.222 ms (1.18); an address held across `yield` (what the old rule allowed) 82.020
ms (1.17); a class object's field walked by position 71.308 ms (1.02). The gap to plain C is the
generator protocol, not the walk.

**8. Kind:** a smarter compile-time solution (generators); `async` is an understandable limitation kept.

---

### AN-78: unsafe and FFI (A20): a C callback with user data

**v5:** no first line says what a function changes (verdict 86): `fn add_point():`, `fn setup(owns game:
shares mut Game) -> Retained[ForeignCallback]:`, `fn main():` and the callback `fn() owns game:
game.add_point()`. What each changes is worked out from its body, a callback handed to C counting as run
by every later foreign call (verdicts 30 and 31), and v4's two E2105 refusals go with their code. What
the callback changes is now known exactly: `add_point` adds to `game.score`, and nothing changes
`game.names`, so in this program a C call inside the walk of `game.names`, and `setup` called inside it,
change nothing the walk walks; v4's three refusals of those lines (E3020) are shown in part 6 with a
callback that changes `game.names`, where they stay refused (R5, R8 (c), row 3; section 4.3). The
trampoline's tagged number holds the callback's generation in its top 16 bits and its slot in its low 48
(worklist section 3). `Retained` is the token that keeps the callback on Ember's side while C holds it,
and gives no object away (verdict 58's corrected record, verdict 91). E3023's, E7004's, E7015's and
E1010's messages give whole lines. **Meaning changed:** no for the NEW program. Three lines v4 refused in
it are allowed (`poll_events()` and an unmarked `sqrt` inside the walk of `game.names`, and
`setup(shares mut game)` inside it), since its callback changes only `game.score` (verdict 86, with row
3's per-field facts).

**1. Intent:** a C event library calls back into the game, and each call adds a point to that game;
between event polls the game prints its players' names with the help of a C maths function.

**2. OLD** (`old/a20_ffi.em`, phase 1's; `setup_raw` passes a capturing lambda where C wants a plain
function pointer, `setup` uses the contract and `owned fn`):

```ember
unsafe extern "C":
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    fn register_raw(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)

fn setup_raw(game: Game):
    # SAFETY(ffi): test of the refusal
    unsafe:
        register_raw(fn(ud: *mut void): game.score += 1, null[*mut void]())

fn setup(game: Game):
    token = register_cb(owned fn(): game.score += 1)
```

```text
error[E5040]: expected `extern "C" fn(*mut void) -> void`, found a lambda that captures
  --> a20_ffi.em:22:22
   |
22 |         register_raw(fn(ud: *mut void): game.score += 1, null[*mut void]())
   |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ this lambda captures
   |
   = help: pass the captured values as arguments, or give C a function that captures nothing
   = help: where C hands the function a user data pointer back, contract the parameter `callback(borrowed, user_data=p)`: it then takes this lambda
   = note: a C function pointer has no room for captured state: `extern "C" fn` holds capture-free functions only ([CLO-3], [FFI-21])

warning[L1001]: `token` is never read
  --> a20_ffi.em:26:5
   |
26 |     token = register_cb(owned fn(): game.score += 1)
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: remove it, or rename it `_token` if that is deliberate

error: could not compile due to 1 error(s)
exit=1
```

With the contract the call compiles, but through `owned fn` (removed) and today's trampoline, 1.47x
plain C per callback (`phase3b-threads.md:1133`, `ffi-1`).

**3. NEW:**

```ember
class Game:
    score: i32 = 0
    names: Array[String] = []

    fn add_point():
        this.score += 1

unsafe extern "C":
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)
    safe fn poll_events()
    @ffi(no_callbacks)
    safe fn sqrt(x: f64) -> f64

fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:
    return register_cb(fn() owns game: game.add_point())

fn main():
    game = shares mut Game(names=[String("ann"), String("bo")])
    token = setup(shares mut game)
    for name in game.names:
        println(name, sqrt(name.len() as f64))
    poll_events()
    println(game.score)
```

The rules:
- The capture list says what the callback C keeps holds (row 6, R15): here `setup`'s owner of the
  game, handed to the callback (`fn() owns game`). A one-thread owner (`shares mut`) may go to it
  because the contract says `threads(creator)`, which the trampoline checks; a `threads(any)` callback
  is a function any thread may run, which holds `thread_shared` values (R12, verdict 61). A plain
  capture (the real variable) is allowed only for `callback(borrowed)`, which C calls only during the
  call.
- **Checked trampoline** (run 2's 8.3 with the attack's fixes, 7.1 and 7.2): C holds a tagged number,
  never an Ember address, with the callback's generation in its top 16 bits and its slot in its low 48
  (R15); this callback carries `0x4080` (16512) in its top 16 bits, which no user-space address has, so
  a C library that wrongly writes through it faults before writing anything. The trampoline checks the
  slot is inside a table that never moves, then the generation and the function's type in one compare,
  then the calling thread for `threads(creator)`/`threads(main)`, in every profile (R15; today only in
  the debug profile, `tests/conformance/FFI-33b`). A call after the token ended stops the program with
  "C called the callback registered on line 17 after its token ended; it was unregistered", and a call
  on another thread with "C called the callback registered on line 17 on another thread, but its
  contract says `threads(creator)`" (`V5/rules.md` section 2's texts, in this program's lines); a
  `callback(once)` callback called a second time stops it too (section 2), which this
  `callback(retained)` one cannot be. A `threads(any)` callback that can be unregistered while it runs
  announces the running slot, and unregistering waits. A callback that ends its own token while it runs
  (a body that sets the field holding its `Retained` token to `None` and then uses what it captured) is
  unregistered at once, so C never runs it again, and its block, where its captures live, is freed only
  when it returns, so the rest of its body runs as written (R12, verdict 62); this program's callback
  keeps its token in `main`'s `token`.
- The contract may be read from the header when its shape is a function pointer plus a `void*` it
  hands back (run 2's 8.4); it is listed by `ember tcb` as an asserted fact, like a written one.
- **Who may change the game** (R3, R4, R9). `main`'s `game` is the first owner and may change it, and
  it keeps what it could do after sharing it (verdict 23). `setup(shares mut game)` makes one more owner
  that may change it and hands it to `setup`'s `owns` parameter (R4: the caller writes `shares mut game`
  for an `owns x: shares mut T` parameter); `owns game` in the callback's capture list hands that owner
  on to the callback. What a function may do with a parameter is set by the word in front of its name,
  for every type (R4, verdict 32): a plain parameter `game: shares mut Game` would be a read-only view of
  `main`'s owner (the `shares mut` after the colon says what `main`'s game is, not what `setup` may do),
  and an owner made from it may do no more than it, so `fn() shares mut game` from it is refused (E3023,
  below); that is why `setup` takes an owner.
- **What the callback changes** (R5, R12; verdict 86). The callback runs `game.add_point()`, which adds
  to the game's `score` (line 6); the compiler works that out from the bodies, through `add_point`, and
  no line says it. It changes the game through an owner it shares with `main`, which `shares mut` in its
  type allows (R9).
- **Handing it to C counts** (R12, verdict 30). `register_cb`'s parameter says `callback(retained)`: C
  keeps the function and runs it during later foreign calls, so the call that hands it over counts as
  making what it changes, now and later: `setup`, and `main` through it, change `game.score`. (Storing a
  function for its caller to run is not a change; handing it to something that runs it on its own, a
  thread or C, is.)
- **Every foreign call may run it** (R12, R15, verdict 31). The compiler cannot see inside C, so a call
  of a foreign function counts as possibly running every callback C keeps, on the threads their
  contracts allow (here the creator thread), and makes the changes they make: `poll_events()` on line
  24 may add to `game.score`, and so may every function that calls it (a `fn tick(): poll_events()`
  would, verdict 39), so no view of `game.score` lives across it (R8 (g)). The walk of `game.names` does
  not meet it: the callback changes only `score`, and R8 (c) refuses a change of the walked list, as row
  3 allows a method that changes `this.log` inside a loop over `this.players`. `poll_events()` is called
  after the walk because the program prints the names between event polls, not because a rule asks.
  `sqrt` says on its declaration that it never calls back into Ember, `@ffi(no_callbacks)` (line 13): a
  fact the programmer vouches for, listed by `ember tcb` with the other asserted facts (`[TCB-1]`); on a
  function whose own contract takes a callback it is refused (E5069). With a kept callback that changes
  `game.names` (part 6), the mark is what keeps `sqrt` allowed inside the walk.

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `score` (2) | none (default: the object's own number) | none (default: what may change it is worked out from the methods' bodies, R5; `add_point` adds to it, line 6) |
| `names` (3) | none (default: the object's own list) | none (default as `score`; no method changes it) |
| `this` (5, not listed: the method's object, R5) | `this` (the `Game` `add_point` was called on) | none (default: worked out from its body, R5; line 6 adds to `this.score`) |
| `cb` (9-11) | `extern "C" fn` (a C function pointer, its type) | none (default: a parameter only reads) |
| `ud` (9-11) | `*mut void` (C's raw pointer; the contract names it user data) | none (default: Ember never reads it here) |
| `x` (14) | none (default: a plain parameter reads the caller's value; an `f64` is copied in, R4) | none (default: read-only) |
| `game` (16) | `owns` (one owner of the game, handed to `setup` by the call) | `shares mut` in its type: an owner that may change the game (R4, R9) |
| `game` (17, capture) | `owns` (`setup`'s owner, handed to the callback C keeps; `setup`'s `game` ends here) | `shares mut` (its type: it may change the game, as `add_point` does) |
| `game` (20) | `shares mut` (a new object with several owners; this is the first) | `shares mut` (this owner may change it, and keeps that after line 21, verdict 23) |
| `token` (21) | none (default: the `Retained` token the call returns, the caller's own; dropping it unregisters) | none |
| `name` (22) | none (default: a read-only view of each name, R2) | none (default: read-only; `game.names` does not change while the loop walks it, R8 (c)) |

11 names, 5 without a word (`score`, `names`, `x`, `token`, `name`), each with its default stated.

**5. Means exactly** (`means/an78_means.em`, today's Ember): `owned fn` retains the class handle,
which is "one more owner":

```ember
fn setup(game: Game) -> Retained[ForeignCallback]:
    return register_cb(owned fn(): game.score += 1)

fn main():
    g = Game()
    token = setup(g)
    println(g.score)
```

```text
warning[L1001]: `token` is never read
  --> means/an78_means.em:18:5
   |
18 |     token = setup(g)
   |     ^^^^^^^^^^^^^^^^ declared here and never used
   |
   = help: remove it, or rename it `_token` if that is deliberate

exit=0
```

That program is the callback part of NEW. The walk of `game.names` and the two C calls v3 added have no
translation run here: today's Ember has no `@ffi(no_callbacks)` (E0104, the rule writer's
`r4_no_callbacks.em`, `V3/changes.md` section 7), and they do what they say: print each name with the
square root of its length, then let C deliver its events, each of which runs the callback and adds a
point.

**6. Still refused:**

A C call, or a callback handed to C, inside the walk is refused where a callback C keeps may change the
walked list (R8 (c), R12). AN-78's own callback changes only `game.score`, so the three refusals v4 gave
are shown for a callback that adds a guest's name on each event, `return register_cb(fn() owns game:
game.names.push(String("guest")))` in place of line 17, the rest of the program as it is:
- `poll_events()` inside the walk (line 24 moved into the loop's body, below line 23): `error[E3020]:
  \`poll_events()\` (line 24) is a C function, which may run the callback C keeps from \`register_cb\`
  (line 17), and that callback pushes to \`game.names\` (line 17), which this loop walks (line 22); call
  \`poll_events()\` after the loop: move line 24 below the loop; if \`poll_events\` never calls back into
  Ember, say so on its declaration: \`@ffi(no_callbacks) safe fn poll_events()\`` (R8 (c), R12, verdict
  31; the rules' E3020 row, in this program's lines). With AN-78's own callback, or one that changes
  nothing (`register_cb(fn(): println("event"))`), the same walk is allowed.
- `sqrt` declared without the mark (line 13 removed, so the lines after it move up by one):
  `error[E3020]: \`sqrt(name.len() as f64)\` (line 22) is a C function, which may run the callback C
  keeps from \`register_cb\` (line 16), and that callback pushes to \`game.names\`, which this loop walks
  (line 21); if \`sqrt\` never calls back into Ember, say so on its declaration: \`@ffi(no_callbacks)
  safe fn sqrt(x: f64) -> f64\`` (the rules' E3020 row: a missing mark refuses only such a walk, and the
  message names the mark). The rules' row ends "otherwise move line 21 below the loop"; here that line
  would not compile, since the call reads the loop's own `name`, so by verdict 73 this program's message
  leaves it out.
- The callback handed to C inside the walk (`token = setup(shares mut game)` moved from line 21 into the
  loop's body, so the loop is on line 21 and the call on line 22): `error[E3020]: \`setup(shares mut
  game)\` (line 22) hands C a callback that pushes to \`game.names\` (\`setup\`'s line 17), which this
  loop walks (line 21); hand it over before the loop: move line 22 above the \`for\` line` (R8 (c), R12,
  verdict 30).

With AN-78's own program:
- The mark on a function whose own contract takes a callback, `@ffi(no_callbacks, param(cb,
  callback(retained, user_data=ud), threads(creator)), destroys(unregister))` on line 10:
  `error[E5069]: \`register_cb\` takes a callback (\`param(cb, callback(retained, user_data=ud))\`), so
  it may call back into Ember; remove \`no_callbacks\` from its \`@ffi(...)\`` (R12).
- No capture word for a kept callback, `return register_cb(fn(): game.add_point())` on line 17:
  `error[E7004]: C keeps this function after \`setup\` returns (\`callback(retained)\`), so it cannot
  use \`setup\`'s \`game\`; say what it holds: write \`return register_cb(fn() owns game:
  game.add_point())\` in place of line 17 (\`setup\`'s owner of the game), or capture \`fn() weakref
  game\` with \`if game exists as ref mut g: g.add_point()\` as its body (a link that lets the game
  end)`.
- A changing owner made from a read-only view: `setup` taking a plain parameter, `fn setup(game: shares
  mut Game) -> Retained[ForeignCallback]:` on line 16, `return register_cb(fn() shares mut game:
  game.add_point())` on line 17 and `token = setup(game)` on line 21: `error[E3023]: \`game\` only reads
  the \`Game\` that \`main\` lends to \`setup\` (\`shares mut Game\` says what \`main\`'s game is, not
  what \`setup\` may do with it), so an owner made from it only reads too; to hand on an owner that may
  change the game, take one: write \`fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:\`
  in place of line 16, \`return register_cb(fn() owns game: game.add_point())\` in place of line 17, and
  \`token = setup(shares mut game)\` in place of line 21` (R4, verdict 32; the rules' E3023 row).
- The game kept by a `threads(any)` callback (line 10 with `threads(any)`): `error[E7015]: C may call
  this function from any thread (\`threads(any)\`, line 10), and \`shares\` owners stay on one thread;
  share the game between threads: write \`from std.sync import Atomic\` above line 1, \`score:
  Atomic[i32] = Atomic(0)\` in place of line 2 and \`this.score.fetch_add(1)\` in place of line 6 (since
  \`add_point\` changes \`score\`), \`fn setup(game: thread_shared Game) -> Retained[ForeignCallback]:\`
  in place of line 16, \`return register_cb(fn() thread_shared game: game.add_point())\` in place of line
  17, \`game = thread_shared Game(names=[String("ann"), String("bo")])\` in place of line 20, \`token =
  setup(game)\` in place of line 21, and \`println(game.score.load())\` in place of line 25` (R9, R12,
  R15, verdict 61; the rules' E7015 row, with the import its `Atomic` lines need and every line it
  replaces, R18). A function C may run on any thread is a function any thread may run, which holds no
  one-thread value. `add_point` then adds through an `Atomic`, which any name may change by its own
  operations (R5).
- A capturing lambda where C wants a bare function pointer: E5040, today's message (it already gives
  the line to write).
- Today's `owned fn(...)`, as `return register_cb(owned fn(): game.add_point())`: `error[E1010]:
  \`owned fn\` is not a word of Ember; name what the function takes or copies after its brackets: write
  \`return register_cb(fn() owns game: game.add_point())\` in place of line 17; a name not listed is the
  real variable` (and `owns fn(...)`: E0109, the same advice).

**7. Cost:** one plain count add at `setup(shares mut game)`, after one compare of the count with its
maximum (verdict 59, R9: at the maximum the program stops, "too many owners of a `Game`
(4,294,967,295)"), and one subtract when the callback is unregistered (Rust's `Rc::clone`, which makes
the same compare, and drop); the capture hands the owner over with no count. The checked trampoline,
1.05x plain C (the attack, `phase3b-threads-attack.md:902`, 164.575 against 156.179 ms; 1.02x in its
second batch; the tag adds no instruction, 163.755 ms, line 960), against today's trampoline 1.47x
(`phase3b-threads.md:1133`); the thread check in every profile adds, in the release profile, one compare
of the calling thread per `threads(creator)` callback, not measured (`V5/rules.md` section 2). Holding a
callback's block until it returns, when it ends its own token (verdict 62), is the trampoline's own
bookkeeping, not measured, and this program never takes that path. The announced `threads(any)` version
that may be unregistered while running costs 1.42x to 1.58x on an empty callback (the attack, 7.1): above
C's cost, and only on that path; Rust's workaround (0.97x, `phase3b-threads.md:1137`) checks none of
these facts. What each function changes, the foreign-call rule and `@ffi(no_callbacks)` cost nothing
while running: they are worked out when compiling, and `sqrt` is called as C calls it.

**8. Kind:** a new mechanism (the checked trampoline), safer than Rust's: a stale, forged or
wrong-thread call is caught, where Rust's workaround is an unbounded `unsafe` reference. The walk across
a C call is a smarter compile-time solution: what a kept callback changes, worked out from its body, is
what a C call may change, and `@ffi(no_callbacks)` exempts a C function that never calls back, a fact
vouched for and listed.

---

### AN-79: `unsafe` points two ways, and an `unsafe fn` body is unchecked with no mark per operation

**v5:** the rule names `into_raw()`, which verdict 91 allows only inside `unsafe:` (E3100), as one of
the things verdict 60 puts there. No line of the program changes; E3100's, L3015's and W3013's messages
are the rules'. **Meaning changed:** no.

**1. Intent:** write a small unsafe helper and call it.

**2. OLD** (`old/an79_unsafe_fn_body.em`, the spec's own example IX.4 with its notes): the helper's
body reads through a raw pointer four times with no `unsafe:` block and no note at any read.

```ember
@safety("p points to four readable bytes")
# SAFETY: the caller guarantees `p` points to four readable bytes
unsafe fn read_u32_le(p: *u8) -> u32:
    return (p.read() as u32) | ((p.add(1).read() as u32) << 8) | ((p.add(2).read() as u32) << 16) | ((p.add(3).read() as u32) << 24)

fn parse(data: Span[u8]) -> Option[u32]:
    if data.len() < 4:
        return None
    # SAFETY: the length check above guarantees four readable bytes
    unsafe:
        return Some(read_u32_le(data.as_ptr()))
```

```text
exit=0
```

**3. NEW** (run 4's, unchanged):

```ember
@safety("p points to four readable bytes")
unsafe fn read_u32_le(p: *u8) -> u32:
    # SAFETY: the caller promised four readable bytes at `p` (@safety above); these are bytes 0 to 3
    unsafe:
        return (p.read() as u32) | ((p.add(1).read() as u32) << 8) | ((p.add(2).read() as u32) << 16) | ((p.add(3).read() as u32) << 24)

fn parse(data: Span[u8]) -> Option[u32]:
    if data.len() < 4:
        return None
    # SAFETY: the length check above guarantees four readable bytes
    unsafe:
        return Some(read_u32_le(data.as_ptr()))
```

The rule (R16): one word, two places, each with its own sentence. `unsafe` before a declaration
(`unsafe fn`, `unsafe interface`, an `unsafe fn(...)` type) says "whoever uses this must vouch", and
`@safety` says what they must make true. `unsafe` before a block or an implementation (`unsafe:`,
`unsafe extend`, `unsafe extern`, `unsafe overlay`) says "this code vouches", and `# SAFETY:` says why.
Inside an `unsafe fn` body every operation of `[UNS-1]` must still sit in an `unsafe:` block, with its
own note (Rust 2024's `unsafe_op_in_unsafe_fn`, made an error, E3100). And anything that can break a
safety promise is possible only inside an `unsafe` block (R16, verdict 60): `into_raw()`, which hands C
an object's address and stops Ember freeing it, compiles only there (E3100: "`into_raw()` stops Ember
from freeing the widget, so a mistake leaks it or frees it twice; write it inside `unsafe:`", verdict
91), and `mem.forget` is removed (verdict 58).

Today's compiler asks for a `# SAFETY:` note on the `unsafe fn` line too (`[UNS-8]`): the NEW text
without it gets W3012 there (`target/redesign3/15-vocabulary-review/an79_no_fn_note.em`, checked again
in run 4 as `target/redesign4/06-area-E/old/an79_no_fn_note.em`, the same output):

```text
warning[W3012]: this `unsafe` has no safety note
 --> an79_no_fn_note.em:3:1
  |
3 | unsafe fn read_u32_le(p: *u8) -> u32:
  | ^^^^^^ nothing here says why it is sound
  |
  = help: write `# SAFETY: <why it is sound>` on the line before it or at the end of this line; `SAFETY(ffi)`, `(layout)`, `(aliasing)`, `(intrinsic)`, `(performance)` or `(uninit)` names its category

exit=0
```

In the model W3012 is for `unsafe:` blocks and implementations; on an `unsafe fn` the check is
L3015's (`@safety`, asked of every `unsafe fn`), and a `# SAFETY:` note there is W3013. The translation
below keeps the line only because today's compiler asks for it; it changes nothing the program does.

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `p` (2) | `*u8` (a raw pointer to someone else's bytes) | `*u8` without `mut`: reads only |
| `data` (7) | none (default: a plain parameter reads the caller's bytes; `Span` is a view) | none (default: read-only) |

2 names, 1 without a word.

**5. Means exactly** (`means/an79_means.em`, which is the NEW text with today's extra note above the
`unsafe fn` line: today accepts it):

```text
exit=0
```

What changes is what is refused.

**6. Still refused:**
- An unsafe operation in an `unsafe fn` body outside a block: the OLD helper: `error[E3100]:
  \`read_u32_le\` is an \`unsafe fn\`, which says its callers must vouch; the reads through \`p\` on
  line 4 are this function's own unsafe steps, and nothing says why they are sound; put line 4 in an
  \`unsafe:\` block with a \`# SAFETY:\` note saying why the read is safe: above line 4 write
  \`# SAFETY:\` and the reason, then \`unsafe:\`, and indent line 4 under it` (the rules' E3100 row,
  worklist item 9; the NEW's lines 3 to 5 are that, with the reason written).
- `unsafe fn` with no `@safety` on any function, not only `pub` ones (today `L3015` for `pub`):
  `warning[L3015]: \`read_u32_le\` is an \`unsafe fn\` that reads through the raw pointer \`p\`, and
  nothing says what its callers must make true for that; say it above the function: \`@safety("...")\`,
  with what \`p\` must point to` (the rules' text, worklist item 11: v3's message wrote the promise for
  the programmer, "p points to four readable bytes", which the compiler cannot know).
- An `unsafe:` block with no note: `W3012`, today's message (it already gives the line).
- A `# SAFETY:` note on the `unsafe fn` line itself (the OLD's line 2): `warning[W3013]:
  \`read_u32_le\` says what its callers must make true with \`@safety\`; a \`# SAFETY:\` note says why
  one block is sound and goes above each \`unsafe:\` block inside it: remove this line`.
- Calling an `unsafe fn` outside `unsafe`, `parse` with its lines 10 and 11 removed, so that
  `return Some(read_u32_le(data.as_ptr()))` is line 10: `error[E3100]: \`read_u32_le(data.as_ptr())\`
  (line 10) calls an \`unsafe fn\`, whose callers must make true what its \`@safety\` says; put line 10
  in an \`unsafe:\` block with a \`# SAFETY:\` note saying why that holds here: above line 10 write
  \`# SAFETY:\` and the reason, then \`unsafe:\`, and indent line 10 under it` (the rules' E3100 row,
  worklist item 9; today's text, "calling an `unsafe fn` requires an `unsafe` block", gives no line).

**7. Cost:** none; compile-time only.

**8. Kind:** a readable spelling of Rust's guarantee (Rust 2024's rule, with the two directions told
apart by where `unsafe` sits and by their two notes).

---

### AN-80: `Sync` names three things; `@sync` reads "synchronised"; `@nosync` reads "not `@sync`"

**v5:** the thread's function and `main` write no `changes Stats` (verdict 86): what they change, `st`'s
counters through `note`, under its lock, is worked out from their bodies, `main`'s own `st.note(5)`
included, and v4's two E2105 refusals go with their code. `ThreadShared` in a bound is written above the
function (verdict 50). **Meaning changed:** no: `longest` ends as 5 whichever thread runs first.

**1. Intent:** mark a class that threads may share; forbid locks in a hot function.

**2. OLD** (`old/an80_sync_nosync.em`): `@sync` read as "synchronised": two tasks call `note`, which
reads `longest` and then writes it in two separate atomic steps (the race `[THR-7]` warns about), and
`@nosync` on a hot function:

```ember
@sync
class Stats:
    words: Atomic[int] = Atomic(0)
    longest: Atomic[int] = Atomic(0)

    fn note(self, n: int):
        self.words.fetch_add(n)
        if n > self.longest.load():
            self.longest.store(n)

@nosync
fn sum(xs: Span[int]) -> int:
    s = 0
    for x in xs:
        s += x
    return s

fn main():
    st = Stats()
    with scope = thread.scope():
        scope.spawn(fn(): st.note(3))
        scope.spawn(fn(): st.note(5))
```

```text
exit=0
```

Accepted, and `longest` may end as 3: both tasks read 0, the one with 5 stores first, the one with 3
stores last. Writing the field plainly instead is E7003 (AN-74's OLD).

**3. NEW:**

```ember
class Stats:
    lock: Lock = Lock()
    words: int guarded by lock = 0
    longest: int guarded by lock = 0

    fn note(n: int):
        with this.lock:
            this.words += n
            if n > this.longest:
                this.longest = n

@nosync
fn sum(xs: Span[int]) -> int:
    s = 0
    for x in xs:
        s += x
    return s

fn main():
    st = thread_shared Stats()
    a = thread.spawn(fn() thread_shared st: st.note(3))
    st.note(5)
    a.join()
```

The three meanings get three spellings (run 3's ruling, standing):
- the **effect** keeps `Sync` and **`@nosync`** keeps its spelling: "no synchronisation", which is what
  it forbids (locks, atomics stronger than relaxed, channels, spawn and join, spec 3184-3186);
- the **property** "several threads may hold and read one value" is **`ThreadShared`**, written only in
  bounds (`[T: ThreadShared]` above `fn keep(x: T)`) and messages, inferred for every type as Rust infers
  `Sync` (R15);
- the **marker** `@sync` is **removed**: what it said on the class (handles cross threads, atomic
  counts, fields frozen) is said on the value where it is shared, `thread_shared Stats()`, and which
  fields may change is said on each field (`guarded by`, `Atomic`). With the marker gone, `@nosync` can
  no longer be read as "not `@sync`".

The read and the write of `longest` now sit under one lock, so `longest` ends as 5 whichever thread
runs first (the OLD's race, both reading 0 and the one with 3 storing last, cannot happen). The
thread's function changes `st` through `note`, under its lock; `thread.spawn` counts as making that
change (R15, verdict 30), and `main`'s own `st.note(5)` makes the same kind of change through `main`'s
own link; all of it is worked out from the bodies (R5, verdict 86), and no line says it. `thread.spawn`
takes a function any thread may run (verdict 94); one that holds a `thread_shared` owner is one. Nothing
more runs: the rules are checked when compiling.

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `lock` (2) | none (default: the object's own lock) | none (default: changes only by being held) |
| `words` (3) | none (default: the object's own number) | `guarded by lock` |
| `longest` (4) | none (default: the object's own number) | `guarded by lock` |
| `this` (6, not listed: the method's object, R5) | `this` (the `Stats` `note` was called on) | none (default: worked out from its body, R5; lines 8 and 10 change `this.words` and `this.longest`, under the lock) |
| `n` (6) | none (default: a plain parameter reads the caller's value) | none |
| `xs` (13) | none (default: reads the caller's numbers; `Span` is a view) | none |
| `s` (14) | none (default: a new local, the function's own) | none (default: its own) |
| `x` (15) | none (default: a read-only view of each number, copied as an `int`, R2) | none |
| `st` (20) | `thread_shared` (one of several owners, on any thread) | `thread_shared` (changed only through `note`, under the lock) |
| `a` (21) | none (default: the `JoinHandle` the call returns, the caller's own) | none |
| `st` (21, capture) | `thread_shared` (one more owner) | `thread_shared` (it changes the stats only through `note`, under the lock) |

11 names, 6 without a word (`lock`, `n`, `xs`, `s`, `x`, `a`).

**5. Means exactly** (`means/an80_means.em`, today's Ember): the two counters inside one `Mutex`, so the
read and the write of `longest` are one step; `@sync` for the cross-thread object; `@nosync` as today:

```ember
struct StatsData:
    words: int
    longest: int

@sync
class Stats:
    data: Mutex[StatsData] = Mutex(StatsData(words=0, longest=0))

    fn note(self, n: int):
        with d = self.data.lock():
            d.words += n
            if n > d.longest:
                d.longest = n
```

(with `sum` and two scoped tasks in `main`, as in the file)

```text
exit=0
```

**6. Still refused:**
- `@sync class Stats`, on the OLD's class: `error[E0104]: \`@sync\` is gone: a class needs no mark to be shared between threads, and \`Stats\`'s fields are \`Atomic\`, which threads may change as they are; remove \`@sync\` (line 1)`.
- `Sync` in a bound, `[T: Sync]` above `fn keep(x: T)`: `error[E1010]: \`Sync\` is the effect
  \`@nosync\` forbids, not a property of a type; for "threads may share it" write \`[T: ThreadShared]\`
  above \`fn keep(x: T)\``.
- `st.longest = n` outside the lock: `error[E7013]: \`longest\` is guarded by \`lock\`; change it
  inside \`with st.lock:\`` (AN-74's message, in this program's names).
- `@nosync` reaching a lock: `E4002`, today's, which names the chain (`[EFF-6]`).

**7. Cost:** none from the words (checked when compiling); the lock is C's (AN-74). The `thread_shared
st` capture makes one atomic count increment, after one compare with the count's maximum (verdict 59,
R9), and one decrement when the thread's owner ends, as AN-75's.

**8. Kind:** a readable spelling of Rust's guarantee (Rust's `Sync`, inferred, named `ThreadShared`;
the class marker replaced by words on the value and the fields).

---

### AN-81: `@static_safe` forbids only aliasing checks

**v5:** no verdict of 83 to 97 changes a line of this answer (verdicts 58 and 91 are about giving an
object to C, which it does not do); E4030's message is cited from `V5/rules.md`. **Meaning changed:**
no.

**1. Intent:** mark a hot function as having no run-time checks.

**2. OLD** (`old/an81_static_safe.em`):

```ember
@static_safe
fn total(xs: Span[int], n: int) -> int:
    s = 0
    for i in 0..n:
        s += xs[i]
    return s
```

```text
exit=0
```

Accepted, and `xs[i]` keeps its bounds check (`[EFF-12]`, spec 3260: `@static_safe` permits bounds,
arithmetic and stale checks).

**3. NEW** (run 4's, unchanged):

```ember
@no_alias_checks
fn total(xs: Span[int], n: int) -> int:
    s = 0
    for i in 0..n:
        s += xs[i]
    return s
```

The same contract, spelled by what it forbids: it reads as the rest of the `@no...` family
(`@noalloc` forbids allocation; `@no_alias_checks` forbids run-time alias checks, E4030; R16).

**4. The owner's test:** `xs` (2) none (default: reads the caller's numbers), `n` (2) none (default:
reads the caller's value), `s` (3) none (default: the function's own local), `i` (4) none (default: the
loop's own number). 4 names, 4 without a word; none changes anything of the caller's.

**5. Means exactly** (`means/an81_means.em`, the OLD text: `@static_safe` is today's spelling of the
same contract): `exit=0`.

**6. Still refused:**
- `@static_safe`: `error[E0104]: \`@static_safe\` is now \`@no_alias_checks\`: it forbids run-time
  alias checks only; bounds and overflow checks still run; write \`@no_alias_checks\``.
- The one run-time alias check left, `assert_disjoint`, inside a `@no_alias_checks` function: AN-14's
  `mix` marked `@no_alias_checks` (`V5/rules.md` section 3, E4030 row; the test
  `R16/reject_assert_disjoint_inside_no_alias_checks.em`). R4's index compare is gone (verdict 98): two
  elements of one list whose positions are not proved different are refused when compiling, E3022.

**7. Cost:** none; the contract is checked when compiling.

**8. Kind:** a readable spelling of Rust's guarantee (a rename: the contract keeps its meaning and is
named for it).

---

### AN-82: `@no_runtime_checks` is reserved, promising a switch that will never exist

**v5:** no verdict of 83 to 97 changes a line of this answer. **Meaning changed:** no.

**1. Intent:** turn off safety checks in a release build.

**2. OLD** (`old/an82_no_runtime_checks.em`):

```ember
@no_runtime_checks
fn total(xs: Span[int], n: int) -> int:
    s = 0
    for i in 0..n:
        s += xs[i]
    return s
```

```text
error[E0104]: `@no_runtime_checks` is reserved for a later version
 --> an82_no_runtime_checks.em:2:1
  |
2 | @no_runtime_checks
  | ^^^^^^^^^^^^^^^^^^
  |

error: could not compile due to 1 error(s)
exit=1
```

The message says the switch is coming; `[EXC-1]` (2708) and `[DSJ-8]` (3173) say it never will.

**3. NEW:** no such word. The function the programmer can write:

```ember
fn total(xs: Span[int]) -> int:
    s = 0
    for x in xs:
        s += x
    return s
```

**4. The owner's test:** `xs` (1) none (default: reads the caller's numbers), `s` (2) none (default:
its own local), `x` (3) none (default: a read-only view of each number). 3 names, 3 without a word.

**5. Means exactly** (`means/an82_means.em`, the NEW text): `exit=0`.

**6. Still refused:** `@no_runtime_checks`: `error[E0104]: there is no switch that turns safety
checks off, in any build (\`[PHIL-13]\`); to forbid run-time alias checks here write
\`@no_alias_checks\`; to skip one bounds check you have proved, write \`xs.get_unchecked(i)\` inside
\`unsafe:\` with a \`# SAFETY:\` note`.

**7. Cost:** the checks the program needs; a loop over the view itself (`for x in xs`) reads each
element once with no index to check.

**8. Kind:** an understandable limitation kept (safety is never switched off), with a message that says
what to write.

---

### AN-83: `@fp(contract)` allows fused multiply-add while "contract" elsewhere means a checked promise

**v5:** no verdict of 83 to 97 changes a line of this answer. **Meaning changed:** no.

**1. Intent:** allow fused multiply-add in one function.

**2. OLD** (`old/an83_fp_contract.em`):

```ember
@fp(contract)
fn dot(ax: float, ay: float, bx: float, by: float) -> float:
    return ax * bx + ay * by
```

```text
exit=0
```

**3. NEW:**

```ember
@fp(allow_fma)
fn dot(ax: float, ay: float, bx: float, by: float) -> float:
    return ax * bx + ay * by
```

**4. The owner's test:** `ax`, `ay`, `bx`, `by` (2): none (default: plain parameters read the
caller's values). 4 names, 4 without a word.

**5. Means exactly** (`means/an83_means.em`, the OLD text, today's spelling): `exit=0`.

**6. Still refused:** `@fp(contract)`: `error[E0104]: \`@fp(contract)\` is now \`@fp(allow_fma)\`: it
lets the compiler fuse \`a * b + c\` into one fused multiply-add; write \`@fp(allow_fma)\``.
`@fp(allow_fma)` inside a `@deterministic` function: E4072, as `@fastmath` today ("`dot` is
`@deterministic`, and a fused multiply-add rounds once where two operations round twice; remove
`@fp(allow_fma)`").

**7. Cost:** none more; one instruction where two were.

**8. Kind:** a readable spelling of Rust's guarantee (a rename of an argument; Rust has no such
attribute, its `f64::mul_add` is explicit).

---

### AN-84: two error codes mean two unrelated things each: E5015 and E7010

**v5:** E5013's message names only what the compiler knows, C's promotion of `narrow` to an `int`, and
gives the line to write (worklist section 3; area E's point 7, v4's item 7 for the owner): what
`sum_wide` reads is in C, which the compiler does not read (R5, R18). `bump` writes no `changes HITS`
(verdict 86; the script). **Meaning changed:** no.

**1. Intent:** read an error code and look it up.

**2. OLD.** `old/an84_e5015_variadic.em`: an owning value in an exported signature, and a variadic
argument C would promote; both are E5015:

```text
error[E5015]: exported parameter `text` reaches an owning Ember value
 --> an84_e5015_variadic.em:4:29
  |
4 | pub extern "C" fn read_text(text: *String) -> i32:
  |                             ^^^^^^^^^^^^^
  |

error[E5015]: C promotes a `i16` passed as a further argument to `int`
  --> an84_e5015_variadic.em:10:31
   |
10 |         println(v.sum_wide(1, narrow))
   |                               ^^^^^^
   |
   = help: pass it as C reads it: `x as c_int` (or `c_uint`)
   = note: C reads a further argument with `va_arg` as the type it expects, after its default argument promotions [FFI-8]
   = note: declared in the header at variadic.h:8: `int64_t sum_wide(int count, ...)` [DIA-5]

error: could not compile due to 2 error(s)
exit=1
```

`old/an84_e7010_parallel.em` (a parallel loop writing through `xs[idx[i]]`):

```text
error[E7010]: the parallel loop writes `xs`, and reaches it here at an index that is not `i` plus a constant
 --> an84_e7010_parallel.em:5:9
  |
5 |         xs[idx[i]] = i
  |         ^^^^^^^^^^
  |
  = help: index it by the loop's own index, `i + k`, or make it a reduction
  = note: a `@parallel` loop's chunks run at once [PAR-2]

error: could not compile due to 1 error(s)
exit=1
```

The spec gives E7010 a second meaning (4966: an `any`-thread export reaching a static that is not
`Sync`); the compiler emits E7010 only for parallel loops (`compiler/ember_typeck/src/lib.rs`, lines
51927, 52238, 52274, 52292, 52308, and `coroutines.rs:1107`), but it also uses it for "a `@parallel`
loop counts over a range `a..b`" with the unrelated help "an `Atomic`, `chunks_mut`, or
`@parallel(reduce=…)`" (AN-74's OLD, `old/an74_parallel_class.em`). The code table (spec 5774, 5816)
lists one meaning each.

**3. NEW:** one meaning per code (`V5/rules.md` section 4):

| code | means | was |
|---|---|---|
| `E5015` | this type may not cross into C as written (an owning Ember value in an exported signature; a type with no C form) | the same, plus the variadic case |
| **`E5013`** (new) | C would change this further argument of a variadic call (a default promotion, or an untyped literal); pass it as the C function reads it | E5015 |
| `E7010` | a parallel loop's chunks may write one place | the same, plus the export case (spec only) and the loop-shape case |
| **`E7012`** (new) | a module variable of the main thread (`static mut` not `guarded by` a lock) is reached by code that runs on another thread (an unscoped thread, an `any`/`creator` export, a C callback not `threads(main)`) | spec 4966's E7010 |
| **`E7016`** (new) | a `@parallel` loop is over something it cannot split into chunks: neither a range (`for i in 0..n:`) nor a list walked by element (`for ref mut p in xs:`), for example a `while` loop | the compiler's E7010 at `for p in players` |

Messages under the new codes:

```text
error[E5013]: C passes `narrow` as an `int` through `...`; if `sum_wide` reads another type, convert it first: `v.sum_wide(1, narrow as i64)`
  --> an84_e5015_variadic.em:10:31
   = help: write `println(v.sum_wide(1, narrow as i64))` in place of line 10
```

(The rules' E5013 row, worklist section 3, in this program's lines. It names only C's promotion, which
the compiler knows from the language; what `sum_wide` reads is in C, which the compiler does not read
(R5: only C code is unseen; R18: a message says only what the compiler knows), even where the program
imports the `.c` file itself, as this one does. `sum_wide` reads each further argument with
`va_arg(arguments, int64_t)` (`tests/conformance/FFI-8/variadic.c` line 9), which the programmer reads,
so the NEW program passes `narrow as i64`; v3's `narrow as c_int` passed 4 bytes where C reads 8, which
is undefined behaviour in C (`V4/safety/completeness.md`, G2 and its section 5, item 11). v4's message
said "`sum_wide` reads an `int64_t`", which the compiler cannot know, and was v4's item 7 for the owner;
the worklist settles it.)

E7012's message, on this refused program (an export that changes an unguarded module variable):

```ember
static mut HITS: int = 0

@export(threads=any)
fn bump():
    HITS += 1
```

```text
error[E7012]: `bump` may be called from any thread (`@export(threads=any)`, line 3), and it changes `HITS`, a module variable of the main thread (line 1)
   = help: guard it with a lock: write `from std.sync import Lock` above line 1, `static HITS_LOCK: Lock = Lock()` and `static mut HITS: int guarded by HITS_LOCK = 0` instead of line 1, and put line 5 inside `with HITS_LOCK:`; or count it with an atomic: write `from std.sync import Atomic` above line 1, `static HITS: Atomic[int] = Atomic(0)` instead of line 1, and `HITS.fetch_add(1)` instead of line 5
```

(The imports are the lines the help's `Lock` and `Atomic` lines need in this program, which imports
neither. What `bump` changes is worked out from its body, verdict 86, and no line says it.)

E7016's message, on the refused program written out (`place` with its loop as a `while` loop):

```ember
fn place(ref mut xs: MutSpan[int], idx: Span[int]):
    i = 0
    @parallel
    while i < idx.len():
        xs[i] = idx[i]
        i += 1
```

```text
error[E7016]: `@parallel` splits a range or a list into chunks, and `while i < idx.len():` (line 4) in `place` is neither
   = help: instead of `i = 0`, the `while` line and `i += 1`, write `for i in 0..idx.len():`
```

(G-11: run 3's help, "count over the range: `for i in 0..idx.len():`", applied as given left `i = 0`
and `i += 1` behind; R18 asks a help that replaces several lines to name them all.) The help gives one
line: the body indexes both `xs` and `idx` by `i`, so a walk of `xs` by element (the other form E7016
names) would not say it; the line it gives is the accepted `place` below. The owner's test on the
refused program: `xs`, `idx` as in the table below; `i` (line 2): none (default: `place`'s own number),
changed by `i += 1`, written.

The programs the new codes' messages lead the programmer to, in NEW spelling (run 4's, with three
changes: the argument given to `place`'s `MutSpan` parameter says `ref mut` at the call, verdict 3; the
parameter says `ref mut` in front of its name, verdict 34; R4; and the variadic argument is passed as
the `i64` that `sum_wide` reads, worklist item 12, so v3's `from std.ffi import c_int` line goes):

```ember
import c "./variadic.c" with (overlay = "overlays/variadic.em") as v
from std.sync import Lock

pub extern "C" fn read_text(text: *u8) -> i32:         # E5015's help: a C type in the export
    return 0

fn place(ref mut xs: MutSpan[int], idx: Span[int]):     # E7010's help: indexed by its own `i`
    @parallel
    for i in 0..idx.len():
        xs[i] = idx[i]

static HITS_LOCK: Lock = Lock()
static mut HITS: int guarded by HITS_LOCK = 0           # E7012's help

@export(threads=any)
fn bump():
    with HITS_LOCK:
        HITS += 1

fn main():
    narrow: i16 = 3
    unsafe:  # SAFETY(ffi): the further argument is passed as the `int64_t` that `sum_wide` reads
        println(v.sum_wide(1, narrow as i64))           # E5013's help
    a = [0, 0, 0]
    order = [2, 0, 1]
    place(ref mut a.as_mut_span(), order.as_span())
    println(a)
```

`place(ref mut a.as_mut_span(), order.as_span())`: `ref mut` says the call may change `a`'s elements
(before a `MutSpan` the word names its elements, so it stands before the span made in the call, R4;
without it, E3028: "`place` changes the elements of `a` (its first line says `ref mut xs:
MutSpan[int]`); say so at the call: `place(ref mut a.as_mut_span(), order.as_span())`"). The parameter
says the same words, `ref mut xs: MutSpan[int]` (R4, verdict 34): a plain `xs: MutSpan[int]` only reads
the caller's numbers, and `xs[i] = idx[i]` through it is refused (E3023, part 6). The two arguments are
views of two different lists, `a` and `order`, so R4's "arguments do not overlap a changeable one" is
proved when compiling, with nothing checked while running, and `xs` is compiled with noalias (R4).
`narrow as i64` widens the number to the 8 bytes `sum_wide` reads; `main`'s call of `v.sum_wide` is a
foreign call, and C keeps no callback in this program, so it changes nothing (R12, verdict 31).

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `v` (1) | `import ... as v`: the C module's name | none (default: a module's name changes nothing) |
| `text` (4) | `*u8` (its type): a raw pointer to bytes C owns | `*u8` without `mut`: reads only |
| `xs` (7) | `ref mut` (the caller's numbers, through its `MutSpan`: the only name for them while `place` runs, R8 (f)) | `ref mut` (it may write them, R4, verdict 34); the caller says `ref mut` too (line 26) |
| `idx` (7) | `Span` (its type): a view of the caller's numbers | `Span`: read only |
| `i` (9) | none (default: the loop's own number) | none (default: only the loop changes it) |
| `HITS_LOCK` (12) | `static`: the module's own lock | none (default: a `Lock` changes only by being held with `with`) |
| `HITS` (13) | `static`: the module's own number | `mut`, `guarded by HITS_LOCK` |
| `HITS` (18) | `static` (line 13) | `mut`, `guarded by HITS_LOCK` (line 13), held by line 17's `with`; `+=` is the change, written |
| `narrow` (21) | none (default: `main`'s own number) | none (default: its own) |
| `a`, `order` (24, 25) | none (default: `main`'s own new lists) | none (default: its own; `place` writes `a`'s elements, said at the call: `ref mut a.as_mut_span()`, line 26) |

11 names, 4 without a word (`i`, `narrow`, `a`, `order`), each with its default stated; none of the
defaults changes, aliases or copies anything the line does not show.

**5. Means exactly** (`means/an84_means.em`): the programs after following each help (the refusals
stay; only the codes change): `narrow as c_int`, an export taking `*u8`, a parallel loop indexed by
its own `i`:

```text
exit=0
```

The carried file follows today's help, `narrow as c_int`, and so does not mean what the NEW line means:
it passes a 4-byte `int` where `sum_wide` reads an 8-byte `int64_t` (`variadic.c` line 9), which is
undefined behaviour in C, and its `exit=0` shows only that it ran. The NEW line, `narrow as i64`, passes
what C reads; no translation of it was run in this step (nothing was).

**6. Still refused:** everything refused today stays refused, under the codes above. And, by verdict 34,
v2's plain parameter: `fn place(xs: MutSpan[int], idx: Span[int]):` with `xs[i] = idx[i]` (line 10):
`error[E3023]: \`xs[i] = idx[i]\` writes an element of the caller's span, but \`xs\` only reads it;
write \`fn place(ref mut xs: MutSpan[int], idx: Span[int]):\` in place of line 7; the call already says
\`place(ref mut a.as_mut_span(), order.as_span())\`` (R4, R5; the rules' E3023 row, in this program's
names). The call's `ref mut` before an argument for that plain parameter fits E3028 too; by verdict 73
the message shown is E3023's, whose line makes the program compile. And `narrow` passed as it is,
`v.sum_wide(1, narrow)` on line 23: E5013, the message above.

**7. Cost:** none.

**8. Kind:** a readable spelling of Rust's guarantee (the same refusals; each code now names one
thing, as Rust's codes do).

## 4. Words needed, for the owner, and a point settled here

### 4.1 Words needed

None. Every word this area's NEW blocks and messages use has a row in `V5/vocabulary.md`. v4 checked
every name of its NEW blocks and messages against the vocabulary (`target/redesign6/06/check_words.py`);
the words this step adds to the area are rows of `V5/vocabulary.md` too: `ref` as a capture word (rows
`fn` and `Scope`: `fn() ref t`), `thread_shared fn()` on `thread.spawn`'s parameter (rows `thread.spawn`
and `thread_shared`), and `into_raw()` (its own row). Checked by `target/redesign7/04/check_words.py`
(output in `target/redesign7/04/check_words.txt`): of the 178 names in the NEW blocks, the 27 that occur
in no `` `...` `` span of the vocabulary are the programs' own names (`count_words`, `Button`,
`poll_events`, `sum_wide`, `HITS_LOCK`, ...), and the three spellings above are rows.

### 4.2 For the owner

None. v4's seven points (`V4/area-E.md` section 4.2) are each settled:
1. **Verdict 65's copy of every text**: verdict 93 lets a task look at the loop's element, `ref t`, with
   no copy; AN-71, AN-72 and AN-74 use it, and `copy of t` stays allowed.
2. **The code for a task or thread that uses the loop's own variable with no word**: E3026 for a scoped
   task, its message giving `copy of t` and `ref t`; E7004 for a `thread.spawn` thread, giving
   `copy of t` (worklist section 3; R15).
3. **E7007's atomic way out for a task that reads first**: dropped (F-10); the rules' row and AN-74 give
   only the lock.
4. **`thread.spawn`'s first line and verdict 61**: verdict 94, `[R]` above `pub fn spawn(owns f:
   thread_shared fn() -> R) -> JoinHandle[R]:`.
5. **Verdict 74 and a field of type `Option[owns T]`**: verdict 85, `next=Some(owns Link(value=1))`
   (AN-77).
6. **A plain `thread_shared Log` argument beside a function value that holds the same log**: verdict 89,
   allowed (AN-75).
7. **E5013 for a function known only by its header**: the message names only C's promotion, for every
   variadic C function (worklist section 3; AN-84).

### 4.3 Settled in this step: what a kept callback changes is known per field

Rule: R5 ("The compiler works out what every function may change, read and end from its code"), R8 (c)
("nothing changes the collection ... through any call that may change it (... a foreign call while a
callback C keeps changes it ...)"), R12 ("a call of a foreign function counts as possibly running every
callback C keeps ... and makes every change they make"), and row 3 as R5 states it (`fn note(msg:
String):` that pushes to `this.log` makes `this.note(...)` refused inside a loop over `this.log` and
allowed inside a loop over `this.players`). Line: AN-78's kept callback, `fn() owns game:
game.add_point()`, whose `add_point` adds to `this.score` and changes nothing in `game.names`. So
`for name in game.names:` with `poll_events()`, an unmarked `sqrt`, or `setup(shares mut game)` inside
it changes no element and no length of what it walks, and R8 (c) does not refuse it. v4 refused those
lines because the callback's first line said `changes Game`, the whole object (the spelling of verdicts
29, 63 and 64, void by verdict 86). This file follows R5's fact per field: it is the precise fact, it is
what row 3 asks, it lets through nothing that could break memory, and a message saying that this
callback changes `game.names` would say what is not so (R18: a message says only what the compiler
knows). AN-78's part 6 shows the three refusals with a callback that does change `game.names`. It
changes no program's meaning when it runs; it decides only whether three safe lines compile, which R5,
R8 (c) and row 3 already decide.

For the rule writer and the test writers, a pointer, not a ruling: R12's sentence "So with
`register_cb(fn() owns game: game.add_point())` kept, ... `for name in game.names: poll_events()` is
refused", and section 3's E3020 messages that say this callback "changes `game.names` (through
`add_point`, line 6)", hold only for a callback that changes the names; so do the R15 tests
`reject_a_c_call_inside_a_walk_a_kept_callback_may_change.em`, `reject_an_unmarked_c_function_inside_the_walk.em`
and `reject_handing_c_the_callback_inside_the_walk.em`, whose `add_point` changes only `score`.
