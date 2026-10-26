# Area E: threads, locks across threads, `unsafe` (AN-71 to AN-84), brought to the v2 rules

Agent 06 (area E), run 4, step 1 (v2). Every answer of run 4's area E (`RUN4/area-E.md`) in the
eight-part form, brought to `V2/rules.md`, `V2/vocabulary.md` and `V2/changes.md`, which carry the
owner's 20 decisions of 2026-10-25 (`inputs/JOB.md`, section "Owner verdicts on run 4's open
questions"). `RUN3` means `tasks/96/redesign/run3/`, `RUN4` means `tasks/96/redesign/run4/`, `V2` means
`tasks/96/redesign/run4/v2/`. "R1" .. "R18" are the v2 rules; "row N" is the owner's agreed row N and
"decision N" the owner's decision N in `inputs/JOB.md`. Terms are those of `V2/rules.md` section 0; the
few this area adds are in section 1 below.

Power: `PowerLineStatus` printed `Online` at the start of this work and before writing this file. No
compiler change, no git command, no Ember program run.

**How the OLD code and its output were carried.** No OLD program of this area changes, so every OLD
block, its output and every "means exactly" block is run 4's, word for word (run 4 carried them from
run 3 by script). The file was made by copying `RUN4/area-E.md` and editing only the NEW parts;
`target/redesign4/v2-06-area-E/compare_old.py` then checked that every OLD block, every pasted output
and every "means exactly" block of this file is identical to run 4's (output in `compare_old.txt` beside
it). Run 4 checked every OLD and "means exactly" file again with today's
`target/redesign4/bin/ember.exe check` (`target/redesign4/06-area-E/recheck.txt`: the same codes and exit
codes as pasted); none of those files changed since, so no new run was needed, and this step makes no new
claim about what today's Ember accepts. File names inside carried text are relative to
`target/redesign3/12-area-E/`, and "section N" inside carried text means a section of run 3's area report
`RUN3/12-area-E.md`. The "means exactly" programs are written in today's Ember, so they keep today's
`self`, `@sync`, `owned fn` and `Mutex`: that is the language today's compiler accepts, not a NEW
spelling.

## 0. What changed in this area since run 4, answer by answer

| answer | what changed under the v2 rules | meaning changed? |
|---|---|---|
| AN-71 | `with owns names` (decision 19); the messages in the new words (E3040, E7004; E7015 names `shares` and `weakref`) | no |
| AN-72 | nothing | no |
| AN-73 | `with copy of n` (decision 12, row 6's words); the refusal of a task that uses a round's own `n` is E3026 (R12: a function value outliving a place it uses), not run 4's E3060 | no |
| AN-74 | E7010's message names `Array[shares mut Player]` (decisions 4, 19); the owner's-test defaults say `changes this` (decision 6) | no |
| AN-75 | the messages say `with owns log` and `with shares log` (decision 19); `with owns log` of the plain parameter `log` is E3013 at the first capture (a name that does not own the value cannot hand it over, R3), not run 4's E3040 at the second; reading 1 of section 2 stated | no |
| AN-76 | decision 17: every lambda that changes a module variable says so on its first line: the threads are `thread.spawn(fn() changes RESULTS: worker(1))`; the stored handler of the refused loop is `fn() changes LOG: ...`, kept in an `Array[fn() changes LOG]`, and `click` says `changes handlers, LOG`; the loop's refusal now comes from those types, and the same loop over plain `fn()` handlers is accepted | **yes**: under "Meaning changed" in the answer |
| AN-77 | `yield copy of s` (decision 12); decision 16: a stored generator whose list changes before its last use is refused at the change, E3020, with the loop's message ("push after `g`'s last use"), where run 4 refused it at the later use (E3060); run 4's question for the owner is answered by decision 16 | no: the same programs are refused; one refusal moves to the change, under E3020 |
| AN-78 | decisions 4 and 19: the game is `shares mut`; a plain parameter only reads (R4, R5), so `setup` takes one owner of the game, `owns game: shares mut Game`, called `setup(shares mut game)`, and the callback holds it `with owns game`; decision 17: the callback says `changes Game`; E7004, E7015 and `owned fn` messages in the new words | no: C still keeps one more owner that may change the game; the count is now made at `setup`'s call instead of at the capture |
| AN-79 | nothing | no |
| AN-80 | nothing in the code | no |
| AN-81 | nothing | no |
| AN-82 | nothing | no |
| AN-83 | nothing | no |
| AN-84 | decision 3: the `MutSpan` argument says `ref mut` at the call: `place(ref mut a.as_mut_span(), order.as_span())` | no |

No answer of this area opens an `Option` or a `weakref` link, keeps a parent link, or holds a link
several fields share, so `exists as`, parent links and `sharedref mut` appear only where a message names
them. No answer uses `self` outside a carried OLD or "means exactly" block: run 4 already wrote `this`,
and its methods already said what they change after the brackets (`changes words, tags`).

## 1. Terms of this area

Run 3's area terms, kept; every other term is `V2/rules.md` section 0's.

- **thread**: a sequence of instructions the operating system runs alongside others, sharing memory.
  **task**, **scope**, **join**: as R15 (`with scope = thread.scope():`; the threads started in it;
  the end of the block, which waits for every task). An **unscoped thread** (`thread.spawn(...)`) may
  outlive the function that started it.
- **data race**: two threads touch one place in memory at the same time, at least one writes, and
  nothing orders them. In C and Rust it is undefined behaviour; in practice it loses updates or frees
  memory still in use.
- **lock**: an object at most one thread holds at a time; `with x.lock:` waits until it is free, holds
  it for the block, and lets it go at the block's end.
- **atomic**: an operation two threads cannot interleave (on this processor a `lock`-prefixed
  instruction), several times dearer than a plain one.
- **reduction**: per-task results combined after the join (R15).
- **trampoline**: the small C function Ember generates for a C library to call; it finds the Ember
  function and calls it. **user data**: the `void*` a C library hands back to its callback.
- **kind**: the job's outcomes: a readable spelling of Rust's guarantee; a smarter compile-time
  solution; a new mechanism; an understandable limitation kept.
- In the owner's-test tables a name counts as **without a word** when neither column has a word and
  both give the stated default.

## 2. Area E's rules, as the v2 rules state them

Run 3's nine area rules are all in `V2/rules.md`; nothing here adds to them.

| run 3's area rule | v2 rule | note |
|---|---|---|
| 1. scoped tasks need no words; reductions | R15 (first point) | a local of one loop round is given to a task `with copy of n` |
| 2. an unscoped thread or a kept C callback says what it holds in its capture list | R15 (second point), R3 | the capture words are `with owns`, `with thread_shared`, `with copy of`, and, on this thread only, `with shares`, `with shares mut`, `with weakref`; E7004 |
| 3. a lock guards the fields its class names (`guarded by`) | R15 (third point) | inside a method the object is `this`: `with this.lock:` |
| 4. `thread_shared` | R3, R9 | unchanged (decision 20); E7014, E7017 |
| 5. what may go to another thread is inferred from the words | R15 (fourth point) | unchanged; E7015, E7008 |
| 6. module variables, `static mut` with `changes NAME` | R5, R12, R15 (fifth point) | `static mut` kept (decision 8). A lambda that changes a module variable says so on its first line (`fn() changes LOG:`, decision 17); the function that only writes such a lambda names nothing |
| 7. generators walk by position across `yield` | R15 (seventh point), R6, R8 (c) | a generator paused in a list is a walk of it: the list does not change while the generator will still be used (decision 16, E3020) |
| 8. C callbacks through the checked trampoline | R15 (last point), R12 | the callback's first line says what it changes outside itself (`changes Game`, decision 17) |
| 9. `unsafe`: one word, two positions | R16 | unchanged |

**Area E's safety argument** (run 3's, unchanged in substance). A data race needs two threads, one
place, a write, and nothing ordering them. Every place two threads can reach is one of these, and each
is ordered or never written:
1. a place a scoped task reads: the parent and the other tasks may not change it until the join (R15;
   unknown code is taken to do everything its type and first line allow);
2. a place tasks change by a reduction: each task has its own accumulator; the join orders them;
3. an `Atomic`, `Mutex`, `RwLock`, channel end, or a `guarded by` field: ordered by the atomic or the
   lock, held by the thread that touches it (a task starts holding none);
4. a value handed over (`owns`): the giver's name has ended, and nothing reachable from it is a
   one-thread owner (`shares`, `shares mut`), a `weakref` link of a one-thread value, or thread-bound
   (E7015, E7008);
5. a `thread_shared` value: never changed directly by any owner (E7014), its count is atomic;
6. a `static mut`: reached only by the main thread and its scoped tasks (E7012), or `guarded by`;
7. a `sharedref mut` target (R9): a scoped task changes it directly only where no other task and not
   the parent may touch it before the join (E7007); an unscoped thread never holds one (E7015).

Freed while in use: a scoped task's views die at the join, which is before the block ends; an
unscoped thread holds only what its capture list names (an owner, a copy, or a `weakref` link, which is
counted and tested on read, R11); one-thread counts never cross threads (E7015); where scoped tasks
re-point `shares` fields of parent objects, count changes are netted per task and decreases wait until
every task has finished its loop (run 3's attack, 3.1, kept); a C callback holds a checked slot, not an
address.

**Area E's rulings** (run 3's, standing, in the v2 words): `@thread_shared` is not added once
`thread_shared` exists (AN-80); a class object held `shares` uses a one-thread count, and a value created
`shares` cannot later be made `thread_shared` ("create it `thread_shared` from the start: `names =
thread_shared Names()`", E7017); a safe `static mut` with `changes` (AN-76); `@cached` across threads:
on an object several threads reach, the slot is filled under a once-flag, and a `@cached` method whose
`reads` names a field other threads change is refused (E2117: "`radius` reads `verts`, which other
threads change under `lock`; a kept result could be stale; remove `@cached`"); the `unsafe fn` body rule
(AN-79).

**Three readings this area relies on, which `V2/rules.md` does not state in words** (each listed for the
owner in section 4.2, with what this file does meanwhile):
1. **A change made through a `thread_shared` link needs no `changes` word on a function or its type.**
   R12 names the links whose changes a function type must say (`weakref`, `shares mut`,
   `sharedref mut`); a `thread_shared` value is changed only through `Atomic`, a lock type, a channel or
   a `guarded by` field under its lock (R9), which are R12's own "need no word" kinds. So
   `fn() with thread_shared log: log.push("a")` (AN-75) and `fn() with thread_shared st: st.note(3)`
   (AN-80) say nothing more. Memory safety does not need the word: a call through a function value
   that takes the same lock again waits for itself (a deadlock, never a change under a walk), since the
   lock's static re-entry (AN-74) applies only to calls the compiler sees.
2. **What a function given to `thread.spawn`, `scope.spawn` or a C `callback(...)` parameter may
   change.** These take a function of any `changes`: a thread is not a call (R5), its changes are
   checked by the thread rules (R15: E7007, E7012, E7014, E7015), and a task's changes by the scope rule.
   So `thread.spawn(fn() changes RESULTS: worker(1))` (AN-76) and
   `register_cb(fn() with owns game changes Game: game.add_point())` (AN-78) are not E2113.
3. **A foreign call may run the callbacks C keeps.** Run 4's R12 counted "the callbacks C may run on
   this thread during" a foreign call; v2's R12 replaced that rule by function types and does not say it
   for C. This file reads it by types: a foreign call made on a thread a kept callback may run on is
   taken to make every change the types of the callbacks registered there say (`changes Game` for
   AN-78's), so a loop over `game`'s lists across such a call is refused (E3020) as across any call
   that changes them.

## 3. The answers of area E

### AN-71: threads (A15): the loop variable, `total +=` from tasks, E7004 offering `@sync`

**Meaning changed:** no.

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

**3. NEW** (run 4's, with `with owns names`; no method, no call to a `ref mut` or `MutSpan` parameter):

```ember
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
    h = thread.spawn(fn() with owns names: names.items.len())
    println(h.join())

fn share_and_keep():
    names = thread_shared Names()
    h = thread.spawn(fn() with thread_shared names: names.items.len())
    println(names.items.len(), h.join())
```

`count_words` is the OLD text, unchanged: under R15 it compiles (`total +=` in a task is a reduction).
Its task stays a plain `fn()`: what it changes, `total`, is a real variable it captures, which counts
as part of the function itself (R12, decision 17).
`share_plain` hands the object to the thread; `share_and_keep` creates it shared between threads and
keeps reading it. A capture `with thread_shared x` of a plain local is refused (E7017, below).

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `texts` (1) | none (default: a plain parameter reads the caller's list, which does not change while `texts` is used, R4) | none (default: read-only, R5) |
| `total` (2) | none (default: a new local, the function's own) | none (default: the function's own variable; `+=` is the change, written) |
| `scope` (3) | `with` (bound for this block, ended at its end) | none (default: read-only binding) |
| `t` (4) | none (default: a plain loop name, a read-only view of each element of `texts`, R2) | none (default: read-only, R5) |
| `total` (5, in the task) | none (default, row 6: a name not in the capture list is the real variable, R12) | none (default, row 6: the real `total` changes; `+=` written) |
| `t` (5, in the task) | none (default: the same view of the element of `texts`, which lives until the join) | none (default: read-only) |
| `items` (9) | none (default: the object's own list) | none (default: changed only by a method whose first line says `changes items` or `changes this`, R5) |
| `names` (12) | none (default: a new local, the function's own object) | none (default: its own) |
| `h` (13) | none (default: the `JoinHandle` the call returns, the caller's own) | none |
| `names` (13, capture) | `owns` (handed to the thread; the caller's `names` ends here) | `owns` (the thread's own now) |
| `names` (17) | `thread_shared` (a new object with owners on any thread) | `thread_shared` (no owner changes it directly, R9) |
| `h` (18) | none (default as line 13) | none |
| `names` (18, capture) | `thread_shared` (one more owner, on any thread) | `thread_shared` (no owner changes it directly) |
| `names` (19) | `thread_shared` (line 17) | `thread_shared` (line 17: read-only) |

14 names; 10 without a word, each with its default stated. No default changes anything the line does
not show, aliases silently, or copies silently: the per-task accumulator behind line 5 is a
compile-time translation no line can observe (5).

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
written as a struct value moved to the thread, which is what a hand-over of a one-owner object is.

**6. Still refused** (each message to R18):

- A task reads `total` before the join: `scope.spawn(fn(): if total > 10: println("many"))`:
  `error[E7007]: tasks add their parts to \`total\`, which is known only once the \`with\` block (lines
  3 to 5) ends; read \`total\` after it, as line 6 does`.
- A task changes `total` in a way the compiler cannot add up per task: AN-72's message.
- A float place one task adds to more than once: `error[E7007]: each task adds to \`sum\` many times;
  float sums depend on how the adds are grouped, so the result would not be any order of single adds;
  add up in the task first: \`mine = 0.0\`, \`mine += x\` for each, then \`sum += mine\` once`. (With
  one add per task the spawn-order result is one order of single adds.)
- Using `names` after `with owns names`: `error[E3040]: \`names\` was handed to the thread at line 13
  (\`with owns names\`); to keep using it here, create it shared, \`names = thread_shared Names()\`,
  and capture it \`with thread_shared names\``.
- An unscoped thread using a name with no capture word: `thread.spawn(fn() => names.items.len())`:
  `error[E7004]: this thread may outlive \`share_plain\`, so it cannot use \`share_plain\`'s
  \`names\`; say what the thread holds: \`fn() with owns names:\` (handed over), or create it shared,
  \`names = thread_shared Names()\`, and write \`fn() with thread_shared names:\` (one more owner)`. It
  never offers `@sync`.
- `with thread_shared names` on the plain local of line 12: `error[E7017]: \`names\` is
  \`share_plain\`'s own object, which it may change; to share it between threads, create it shared:
  \`names = thread_shared Names()\`` (the same for `keep = thread_shared names`).
- Handing over a value that holds a one-thread owner: if `Names` had `cache: shares Cache` and the
  caller another owner of it, `error[E7015]: \`names\` exists as \`cache\`, which has other owners on this
  thread (line 9), whose counts change without atomics; share it between threads:
  \`cache: thread_shared Cache\``. The same for a class that is the target of a `weakref` field
  anywhere in the build (its `weakref` links stay on this thread): "share it instead: create it
  `names = thread_shared Names()` and capture it `with thread_shared names`".
- Changing a `thread_shared` value directly: `names.items.push(s)` in `share_and_keep`: `error[E7014]:
  \`names\` is shared between threads (line 17), so no owner changes it directly; keep the list in a
  field \`guarded by\` a lock and change it inside \`with names.lock:\``.
- A thread-bound value sent to a task (a guard, a `Scope`, a raw pointer; run 3's attack `x05`,
  `x08`, `x09`): `error[E7008]: \`g\` is the guard of \`m\`'s lock, which belongs to the thread that
  took it; take the lock inside the task: \`with g = m.lock():\``.

**7. Cost.** Scoped tasks and reductions: nothing beyond plain C's per-thread counter. Last run,
`phase3b-threads.md:381-386` (64 MiB of text, 8 tasks; PROVISIONAL): `reduce-0`, plain C with a local
counter per thread summed after the join, 43.548 ms (1.00), which the reduction compiles to; an atomic
add per word (today's `Atomic` spelling) 10.22x; safe Rust's `AtomicU64` 10.35x, its join-sum 1.08x.
`with owns names`: a pointer handed over, no count. `with thread_shared names`: one atomic increment
when the thread's owner is made and one atomic decrement when it ends, Rust's `Arc` price. Because the
word says which values are thread-shared, the count kind is known when compiling: last run's run-time
"shared flag" (and its walk, 5.451 ms per 262,144 objects, `phase3b-threads.md:538`) is not needed.

**8. Kind:** a smarter compile-time solution (the scope rule and reductions), with a readable spelling
of Rust's `move` + `Arc` for unscoped threads (the capture words).

---

### AN-72: errors land on the wrong variable in `a15_threads.em`

**Meaning changed:** no (the help now gives lines that compile, R18).

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
            scope.spawn(fn(): total = total * 2 + t.len())
    return total
```

```text
error[E7007]: two tasks change `total`, and `total = total * 2 + t.len()` uses its old value
  --> count.em:5:31
   |
 5 |             scope.spawn(fn(): total = total * 2 + t.len())
   |                               ^^^^^ every task of this scope runs this line at the same time as the others
   |
   = help: to add each task's part, write `total += t.len()`; to combine the parts in order, write `parts: Array[int] = []` before the `with` block, `scope.spawn(fn(): parts.push(t.len()))` as the task, and `for p in parts: total = total * 2 + p` after the `with` block
```

The rule: a task's conflict is reported once, on the captured name that causes it; errors on other
names of the same function that only follow from it are not shown, and no error ever names a name the
program uses correctly (`t` here). The help's second way gives the same result as the program run on
one thread: `push` in a task is a reduction combined at the join in spawn order (R15), which is the
order of `texts`. (Run 3's help said only "have each task keep its own (`parts.push(t.len())`)", which
does not compile until `parts` is declared; R18 asks the help's lines to compile in the program.)

**4. The owner's test:** `texts` (1), `total` (2), `t` (4), `total` (5), `t` (5): none (defaults as in
AN-71's table, the same lines); `scope` (3): `with`. 6 names, 5 without a word. In the help's lines,
`parts` is a new local, `count_words`'s own (none: its own; `push` is the change, written), and `p` is a plain
loop name (none: a read-only view of each number, which as an `int` is copied, R2).

**5. Means exactly.** The program is refused; the line its help gives first (`total += t.len()`) is
AN-71's program, whose translation is `means/an71_means.em` (`exit=0`, pasted in AN-71).

**6. Still refused:** the NEW program itself, with the message above.

**7. Cost:** none; a compile-time message.

**8. Kind:** a smarter compile-time solution (the error found at its cause).

---

### AN-73: "cannot find `n`" (E1010) inside a lambda that calls an `Atomic` method

**Meaning changed:** no (`with copy of n`, decision 12; one refusal's code corrected, E3026).

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

**3. NEW** (run 4's, with `copy of`, decision 12):

```ember
fn count(texts: Array[String]) -> int:
    total = Atomic(0)
    with scope = thread.scope():
        for t in texts:
            n = t.split_whitespace().count()
            scope.spawn(fn() with copy of n: total.fetch_add(n))
    return total.load()
```

The defect is fixed (a name used as an argument of a method call inside a lambda is looked up like any
other), and the task names what it copies, row 6's words (R15: "a local of one loop round is given to a
task with `with copy of n`"). `t` needs no word: it is a view of an element of `texts`, which lives
until the join; `n` is the round's own number, which does not. `total` is the real variable (row 6);
`fetch_add` changes an `Atomic` through any name (R5's exception by type).

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `texts` (1) | none (default: a plain parameter reads the caller's list) | none (default: read-only) |
| `total` (2) | none (default: a new local, the function's own) | none (default: an `Atomic` changes only through its own methods, R5) |
| `scope` (3) | `with` | none (default: read-only binding) |
| `t` (4) | none (default: a read-only view of each element) | none |
| `n` (5) | none (default: a new local of this round, its own) | none (default: its own) |
| `n` (6, capture) | `copy of` (the task's own copy, row 6) | `copy of` (the task's own) |
| `total` (6, in the task) | none (default, row 6: the real `total`) | none (default: through `fetch_add`, the `Atomic`'s own method, written) |

7 names, 5 without a word.

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

**6. Still refused:** the task without the word, once the lookup is fixed:
`error[E3026]: the task uses \`n\`, which ends at the end of this round of the loop (line 5), while
the task may still be running; give the task its own copy: \`fn() with copy of n:\``. Under v2 this
is E3026, "a function value that may outlive a place it uses and does not own" (R12); run 4 gave it
E3060, whose v2 meaning is a view used after its target changed.

**7. Cost:** one atomic add per task, as the program asks (an `Atomic` is the programmer's choice;
`total += n` on a plain `int` would be AN-71's reduction at plain C's cost). The copy of `n` is one
8-byte store into the task's state.

**8. Kind:** a smarter compile-time solution (a defect fixed, and the real cause named with row 6's
words).

---

### AN-74: a `@sync` class freezes every field after `init`; objects written by several tasks need a lock or a claim

**Meaning changed:** no (E7010's message says `shares mut Player`, decisions 4 and 19).

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

    fn add_all(text: StrView) changes words, tags:
        for w in text.split_whitespace():
            with this.lock:
                this.words += 1
                if w.starts_with("#"):
                    this.tags.push(String(w))

fn count(texts: Array[String]) -> int:
    stats = Stats()
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn(): stats.add_all(t))
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
  `this.lock` held" (run 3's attack; a fact worked out from the body, carried in the function's type,
  and recorded in the package's interface file across a package's edge, R17) and may be called only
  with the lock held. A `with` on a lock this thread is known to hold calls the callee's entry that
  skips taking it again (the attack's static re-entry; no run-time owner). A view of a guarded field
  does not leave the `with` block (row 4's rule, R8 (e)).
- `add_all`'s first line says what it changes, `changes words, tags` (row 3, R5); its object is `this`
  and is not listed among its parameters. `count` calls it on its own local `stats`, so `count`'s first
  line names nothing (R5: a change to an own local needs no clause). The method is called on the object
  with no mark (`stats.add_all(t)`, R4). The task's function changes only `stats`, a real variable it
  captures, which counts as part of the function itself (R12, decision 17), so it is a plain `fn()`.
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
| `this` (8, not listed: the method's object, R5) | `this` (the object `add_all` was called on) | `changes words, tags` (row 3) |
| `text` (8) | `StrView` (a view of someone else's text) | none (default: read-only) |
| `w` (9) | none (default: a read-only view of each word) | none |
| `texts` (15) | none (default: reads the caller's list) | none |
| `stats` (16) | none (default: a new local, the function's own object) | none (default: its own; changed through `add_all`, which says `changes`) |
| `scope` (17) | `with` | none |
| `t` (18) | none (default: a read-only view of each element) | none |
| `stats` (19, task) | none (default, row 6: the real `stats`) | none (default: through `add_all`'s `changes words, tags`, under the lock) |
| `t` (19, task) | none (default: the same view, alive until the join) | none |
| `players` (27) | `ref mut` (the caller's list) | `ref mut` (may change it) |
| `p` (29) | `ref mut` (each element of the caller's list, alone while used, R8 (f)) | `ref mut` |

14 names, 7 without a word.

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
- A guarded field used outside its lock: `stats.words += 1` in `count`: `error[E7013]: \`words\` is
  guarded by \`lock\`; change it inside \`with stats.lock:\``.
- The lock taken on another object: `with a.lock: b.words += 1`: `error[E7013]: \`b.words\` is
  guarded by \`b.lock\`, and this block holds \`a.lock\`; write \`with b.lock:\``.
- A lock held by the parent "protecting" tasks it spawns (the attack's `y14`): the task body holds no
  lock, so `stats.words += 1` in it gets the first message.
- `add_all` without `changes`: `fn add_all(text: StrView):` with the same body: `error[E2105]:
  \`add_all\` changes \`this.words\` and \`this.tags\`; write its first line as \`fn add_all(text:
  StrView) changes words, tags:\``.
- A field written after `init` by two tasks with no lock and no `guarded by`: `error[E7007]: two
  tasks change \`stats.total\` (line 19); guard it: \`total: int guarded by lock\`, and change it inside
  \`with this.lock:\`; or make it \`Atomic[int]\``.
- A list whose objects may have a second owner in a parallel loop: `for ref mut p in players` with
  `players: Array[shares mut Player]`: `error[E7010]: \`players\` exists as \`shares mut Player\` objects,
  and one object may be in the list twice, so two chunks could change it at once; give each object one owner
  (\`Array[Player]\`), or change it under its lock (\`hp: int guarded by lock\`)`.

**7. Cost.** A lock where the program writes one, the lock C takes (last run, `phase3b-threads.md:741`,
`lock-0`, lock-use-unlock in C, 1479.479 ms against safe Rust's `Mutex` 2170.016 ms, PROVISIONAL);
no field is wrapped, so the object is its plain fields plus one lock word. One thread that uses such a
class pays an uncontended lock per `with`, as Python and C pay. The parallel loop over one-owner
elements: plain loads and stores, no count test and no claim: the per-object claim's 1.12x to 1.36x
(the attack, 3.5) is gone because the words prove what the claim checked while running.

**8. Kind:** a readable spelling of Rust's guarantee (`Mutex<Inner>` spelled as a lock field, with the
field saying which lock guards it), plus a smarter compile-time solution for the per-element loop.

---

### AN-75: clone-before-move for shared handles (A34)

**Meaning changed:** no (the messages say `with owns log` and `with shares log`, decision 19; one
refusal's code corrected, E3013).

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

    fn push(s: StrView) changes lines:
        with this.lock:
            this.lines.push(String(s))

fn start(log: thread_shared Log):
    a = thread.spawn(fn() with thread_shared log: log.push("a"))
    b = thread.spawn(fn() with thread_shared log: log.push("b"))
    a.join()
    b.join()

fn main():
    start(thread_shared Log())
```

The capture word does what Rust's `Arc::clone` line does, on the name it is about: each thread gets
one more owner. No second name (`log2`) and no separate clone line. `log: thread_shared Log` reads the
caller's value with no count (R4); `with thread_shared log` makes one more owner from it (R3, allowed
because the value is already `thread_shared`). Each thread changes the log only through `push`, whose
first line says `changes lines`, a field `guarded by lock` changed inside `with this.lock:` (R9: the
one way a `thread_shared` value is changed). The threads' functions change `log` only that way, under
its lock, so their first lines name nothing (section 2, reading 1), and `start`'s read-only `log`
promises only that its other fields do not change while it is used (R4, R8 (g)): `lines` is changed under
its lock, as a `Mutex`'s value is changed through any name.

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `lock` (4) | none (default: the object's own lock) | none (default: changes only by being held) |
| `lines` (5) | none (default: the object's own list) | `guarded by lock` |
| `this` (7, not listed: the method's object, R5) | `this` (the `Log` `push` was called on) | `changes lines` (row 3) |
| `s` (7) | `StrView` (a view of someone else's text) | none (default: read-only) |
| `log` (11) | `thread_shared` (in its type: a log shared between threads; a plain parameter, so a view of the caller's, no count, R4) | `thread_shared` (no owner changes it directly; `push` changes a guarded field) |
| `a` (12) | none (default: the `JoinHandle` the call returns, the caller's own) | none |
| `log` (12, capture) | `thread_shared` (one more owner) | `thread_shared` |
| `b` (13) | none (default as `a`) | none |
| `log` (13, capture) | `thread_shared` | `thread_shared` |

9 names, 3 without a word.

**5. Means exactly** (`means/an75_means.em`, which is phase 1's `a34_clone_dance.em` itself): a
`@sync` class (atomic counts), its list inside a `Mutex`, and `owned fn` retaining the handle once per
thread. `exit=0`.

**6. Still refused:**
- A thread that takes the log over: `with owns log`: `error[E3013]: \`log\` is lent to \`start\`
  (a plain parameter), so the thread cannot take it over (\`with owns log\`, line 12); to give each
  thread one more owner of the log, write \`fn() with thread_shared log:\``. (Run 4 gave this E3040,
  "handed to the first thread"; under R3 and E3013 a name that does not own the value cannot hand it
  over at all, so the refusal comes at the first capture.)
- A one-thread owner sent to a thread: `with shares log`: `error[E7015]: \`with shares log\` makes an
  owner of \`log\` that stays on one thread, and this function runs on a new one; write \`with
  thread_shared log\``.
- No word: `thread.spawn(fn() => log.push("a"))`: AN-71's E7004 message, in this program's names.
- `with thread_shared log` where `log` is a plain parameter `log: Log` (a view of an object whose
  owner may be one-thread): `error[E7017]: \`log\` is lent to \`start\`, and its owner may not be shared
  between threads; take it as \`log: thread_shared Log\``.
- `log.lines.push(String("c"))` in `start`, outside the lock: `error[E7013]: \`lines\` is guarded by
  \`lock\`; change it inside \`with log.lock:\`, or call \`log.push("c")\``.

**7. Cost:** one atomic increment per `with thread_shared` capture and one atomic decrement when the
thread's owner ends: exactly Rust's `Arc::clone` and drop, and what today's `owned fn` does for a
`@sync` handle. The lock is C's.

**8. Kind:** a readable spelling of Rust's guarantee (`Arc::clone` + `move`, written as one word on the
captured name).

---

### AN-76: global mutable state (A19)

**Meaning changed: yes** (decision 17: functions say what they change, and so do lists of stored
functions).
1. **A lambda that changes a module variable says so on its first line, and the list that keeps it says
   so in its type**: `b.on_click(fn() changes LOG: LOG.push(String("clicked")))`, kept in
   `handlers: Array[fn() changes LOG]`; `click`, which runs them, says `fn click() changes handlers,
   LOG:`. Run 4 let the lambda be a plain `fn()` and took every call that may run a function value to
   make every change any such value makes; its E3020 message said "the compiler does not follow which
   handlers each button holds" (G-07). Now the types say it: a loop over `LOG` across `b.click()` is
   refused because `click`'s first line says `changes LOG`, and the same loop is **accepted** when
   `b`'s handlers are plain `fn()`, which change nothing outside themselves.
2. **The functions the threads run say what they change**: `thread.spawn(fn() changes RESULTS:
   worker(1))`, where run 4 wrote `thread.spawn(fn(): worker(1))`.
What stays from run 4 (G-06): `setup`, which only writes the lambda, names nothing on its first line;
`changes LOG` on `setup` would say "a call of `setup` changes `LOG`", and it does not. Nothing this
answer's own program does changes.

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

fn tick() changes FRAME, LOG:
    FRAME += 1
    LOG.push(String("tick"))

static RESULTS_LOCK: Lock = Lock()
static mut RESULTS: Array[int] guarded by RESULTS_LOCK = []

fn worker(x: int) changes RESULTS:
    r = x * x
    with RESULTS_LOCK:
        RESULTS.push(r)

fn main() changes FRAME, LOG:
    tick()
    a = thread.spawn(fn() changes RESULTS: worker(1))
    b = thread.spawn(fn() changes RESULTS: worker(2))
    a.join()
    b.join()
```

and the button of decision 17, whose handlers push to the log:

```ember
class Button:
    handlers: Array[fn() changes LOG] = []

    fn on_click(owns f: fn() changes LOG) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers, LOG:
        for ref mut h in this.handlers:
            h()

fn setup(ref mut b: Button):
    b.on_click(fn() changes LOG: LOG.push(String("clicked")))

fn report(ref mut b: Button) changes LOG:
    for entry in LOG:
        println(entry)
    b.click()
```

The rules (R5, R15, R12):
1. `static mut NAME: T = e` declares a module variable that may change. It needs no `unsafe`; a
   foreign `static mut` (in an `extern` block) keeps `[STA-1]`'s `unsafe` at every use. The
   initialiser follows `[STA-3]`, as today's `static`s.
2. A function whose body changes it, itself or through a call, says so on its first line: `changes
   FRAME` (row 3's word, on a free function; R5). Checked against the body both ways: a change not
   named is refused, a name not changed is refused. Callers name it too, as a method names what the
   methods it calls change: `main` calls `tick()`, so `fn main() changes FRAME, LOG:`; `report` calls
   `b.click()`, so `fn report(ref mut b: Button) changes LOG:` (the change to `b`'s handlers goes
   through its `ref mut` parameter and needs no clause). A function `thread.spawn` starts is not a call
   of `main` (`worker` runs on its own thread; rule 4 and E7012 check it), so `main` does not name
   `RESULTS`. A lambda says what it changes on its own first line (R12): `fn() changes RESULTS:
   worker(1)`, `fn() changes LOG: LOG.push(...)`. Writing a lambda is not changing what it changes:
   `setup` names nothing (R5; G-06).
3. One thread: a loop over `LOG`, or any view into it, across a call that may change `LOG` is refused
   (R8 (c), (g); the agreed limitation: change after the loop). A call may change `LOG` when its
   callee's first line says so, and a call through a function value when the value's type says so
   (R12): `b.click()` changes `LOG` because `click` runs handlers of type `fn() changes LOG` and its
   first line says `changes handlers, LOG`. A list of plain `fn()` handlers changes nothing outside
   them, so clicking such a button inside a loop over `LOG` is allowed. A foreign call is taken to make
   the changes that the types of the kept C callbacks which may run during it say (section 2,
   reading 3; the attack's 5.1, `y16`).
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
| `FRAME` (5) | `static` (line 1) | `changes FRAME` (line 4) |
| `LOG` (6) | `static` (line 2) | `changes LOG` (line 4) |
| `RESULTS_LOCK` (8) | `static` | none (default: a `Lock` changes only by being held) |
| `RESULTS` (9) | `static` | `mut`, `guarded by RESULTS_LOCK` |
| `x` (11) | none (default: a plain parameter reads the caller's value; an `int` is copied in, R4) | none (default: read-only) |
| `r` (12) | none (default: a new local, the function's own) | none (default: its own) |
| `RESULTS` (14) | `static` (line 9) | `guarded by RESULTS_LOCK` (line 9), held by line 13's `with`; `changes RESULTS` on `worker` (line 11) |
| `main` (16) | `fn` | `changes FRAME, LOG` (it calls `tick()`, R5) |
| `a` (18) | none (default: the `JoinHandle` the call returns, the caller's own) | none |
| `b` (19) | none (default as `a`) | none |

The two lambdas of lines 18 and 19 declare no name; each says on its first line what it changes,
`changes RESULTS` (R12). In the button's block:

| name (line) | what it is | may it change it |
|---|---|---|
| `handlers` (2) | none (default: the object's own list) | none (default: changed only by a method whose first line says `changes handlers` or `changes this`); its type says what the functions in it change, `fn() changes LOG` |
| `this` (4, not listed: the method's object, R5) | `this` (the `Button` `on_click` was called on) | `changes handlers` |
| `f` (4) | `owns` (the function is handed to `on_click`) | `owns` (its own); what `f` itself changes: `changes LOG`, in its type |
| `this` (7, not listed) | `this` (the `Button` `click` was called on) | `changes handlers, LOG` |
| `h` (8) | `ref mut` (each stored function, the list's own) | `ref mut` (a call may change the function's own state, R12) |
| `b` (11) | `ref mut` (the caller's button) | `ref mut` |
| `b` (14) | `ref mut` (the caller's button) | `ref mut`; `report` also says `changes LOG` |
| `entry` (15) | none (default: a read-only view of each line of `LOG`, R2) | none (default: read-only; `LOG` does not change while the loop walks it, R8 (c)) |

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

**6. Still refused:**
- A change not named: `fn reset(): FRAME = 0`: `error[E2105]: \`reset\` changes \`FRAME\`; write its
  first line as \`fn reset() changes FRAME:\``.
- `changes LOG` on a function that does not change it: `error[E2105]: \`tick\` says \`changes LOG\`
  and never changes it; remove \`LOG\` from its first line: \`fn tick() changes FRAME:\``.
- A loop over `LOG` calling `tick()`: `for entry in LOG:` with `tick()` in its body:
  `error[E3020]: this loop walks \`LOG\`, and \`tick()\` changes \`LOG\` (\`fn tick() changes FRAME,
  LOG:\`, line 4); call \`tick()\` after the loop`.
- A worker thread using an unguarded one: `thread.spawn(fn() changes FRAME, LOG: tick())`:
  `error[E7012]: \`tick\`
  changes \`FRAME\`, a module variable of the main thread, and this thread runs beside it; guard it:
  \`static mut FRAME: int guarded by FRAME_LOCK\` and change it inside \`with FRAME_LOCK:\`, or make it
  \`Atomic[int]\``.
- `FRAME = 0` at module level (Python's spelling): `error[E1030]: \`FRAME = 0\` at module level is a
  statement, and this file has \`fn main\`; declare a module variable: \`static mut FRAME: int = 0\``.
- `global FRAME`: today's E0100 help, kept ("`global` is Python's"), pointing at `changes FRAME`.
- A thread's function that does not say what it changes: `thread.spawn(fn(): worker(1))`:
  `error[E2105]: this function calls \`worker\`, which changes \`RESULTS\`; say so on its first line:
  \`fn() changes RESULTS: worker(1)\``.
- The handler written as a plain `fn()`: `b.on_click(fn(): LOG.push(String("clicked")))` (line 12):
  `error[E2105]: this function pushes to \`LOG\`; say so on its first line: \`fn() changes LOG:
  LOG.push(String("clicked"))\``.
- A handler that changes `LOG` kept in a list of plain functions: `handlers: Array[fn()]` and
  `fn on_click(owns f: fn()) changes handlers:` with line 12 as written: `error[E2113]: this function
  changes \`LOG\` (\`fn() changes LOG\`), and \`on_click\` takes \`fn()\`, functions that change nothing
  outside themselves; keep it in a list that says so: \`handlers: Array[fn() changes LOG]\`, with
  \`fn on_click(owns f: fn() changes LOG) changes handlers:\``.
- `click` without `LOG`: `fn click() changes handlers:`: `error[E2105]: \`click\` runs the handlers in
  \`this.handlers\`, whose type says \`changes LOG\`; add it to its first line: \`fn click() changes
  handlers, LOG:\``.
- A loop over `LOG` across a call that runs those handlers (run 3's `20-re-attacker.md`, F-04's
  program, in this block's names: `b.click()` inside `report`'s loop, line 16, instead of after it):
  `error[E3020]: this loop walks \`LOG\` (line 15), and \`b.click()\` (line 16) changes \`LOG\`: \`click\`
  runs the handlers of \`b\`, an \`Array[fn() changes LOG]\` (its first line says \`changes handlers,
  LOG\`); click after the loop` (R8 (c), R12; `V2/rules.md` section 3's message, in this program's
  lines). The program that message leads to (line 17's order) is
  `RUN3/21-fixer/c/f4_fnvalue_static_loop.c` mode 2: clean under AddressSanitizer (run 3's proof, not
  run again).

No longer refused: the same loop over `LOG` with `b.click()` inside it, when `b`'s handlers are plain
`fn()` (`handlers: Array[fn()]`, `fn click() changes handlers:`): no function in the list may change
`LOG` (E2113 keeps such a function out), so the walk is safe (decision 17).

**7. Cost:** plain C globals: nothing counted, nothing checked while running. Last run measured the
same work (`phase3b-threads.md:854-863`, `tick()` called 100,000,000 times, PROVISIONAL): plain C
globals 192.219 ms (1.00); today's Ember workaround (`Atomic` + `Mutex`) 3.10x on one thread, 7.15x
once a second thread has started; safe Rust's `AtomicU64` + `Mutex` 7.44x; Rust's `thread_local!`
1.04x. The guarded list costs the lock C takes.

**8. Kind:** a smarter compile-time solution (Rust needs `unsafe` or atomics for this; here the
function's first line says what it changes and the compiler proves only one thread group reaches it).

---

### AN-77: async and generators (A14)

**Meaning changed:** no, since run 4. Run 4 asked the owner whether a generator kept in a variable
may see its list change; decision 16 ("a, same rule for every loop") answered it the way run 4 had
read it: a generator paused in the middle of a list is a loop over that list (R6, R8 (c)). What that
settles, as run 4 had it:
1. **No "walk by position" way out.** A generator that walks by position (`while i <
   len(this.log):`) is a loop over the list too, so `g = d.replay()`, `d.log.push(...)`, `g.next()` is
   refused whichever way `replay` walks. A program that wants to add to the log between resumes
   collects the additions and pushes them after the generator's last use: the agreed limitation ("a
   list may not change while a loop walks it"), now the owner's for every loop.
2. **`from log` is optional.** A generator method's result views its whole object without it, so
   `gen fn replay() -> Generator[int]:` compiles; `from log` narrows it to `d.log`, so that changing
   another field of `d` stays allowed while `g` is used. This answer keeps `from log`, which says on
   the line what the generator walks.
What changes since run 4 is where the refusal of a stored generator lands, and its code: at the change,
E3020, with the loop's own message ("push after `g`'s last use"), where run 4 gave E3060 at the
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
(c), decision 16). The position kept in the frame stays: it is what lets the frame move.

`async` stays reserved in this version (understandable limitation: no async runtime exists). Its
design is run 2's (`phase3b-threads.md` 7.3 rule 3, the attack's 6.3): task groups, which are scopes;
frames that never move; the scope rule in place of `Send`.

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `buf` (2) | none (default: a new local, the generator's own) | none (default: its own) |
| `s` (3) | none (default: a read-only view of each element) | none |
| `s` (4, yielded) | `copy of` (row 9: a new, independent `String` is given out) | `copy of` (the receiver's own) |
| `angle` (7) | none (default: the object's own field) | none (default: changed only by a method whose first line says `changes angle` or `changes this`, R5) |
| `log` (8) | none (default: the object's own list) | none (default as `angle`) |
| `this` (10, not listed: the method's object, R5) | `this` (the `Door` `replay` was called on) | none (default: the first line says neither `changes` nor `owns this`, so `replay` only reads it, R5) |
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
- A reference to the frame's own inline local across `yield` (`r = ref n`, then `yield`, then `r`):
  `error[E2221]: \`r\` points into this generator's own frame, which may move while it is suspended;
  keep the value instead (\`m = n\`), or its position`.
- The in-place consumer changing the walked field: `for n in d.replay(): d.log.clear()`:
  `error[E3020]: this loop walks \`d.log\` through \`d.replay()\` (\`from log\`), and \`d.log.clear()\`
  changes it; write \`d.log.clear()\` after the loop`.
- A stored generator and a change to its list before its last use: `g = d.replay()` (line 1),
  `d.log.push(String("x"))` (line 2), `println(g.next())` (line 3): `error[E3020]:
  \`d.log.push(String("x"))\` (line 2) changes \`d.log\`, which \`g\` walks (\`g = d.replay()\`, line
  1), and \`g\` is used again on line 3; push after \`g\`'s last use` (R8 (c), decision 16; the
  message of `V2/rules.md` section 3, in this program's lines; no "walk by position" way out, since a
  position walk is refused the same way).
- A walk with no position (a linked structure held by views): E2221 as today, with "walk it by an
  index or a `Handle`".
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

**Meaning changed:** no. C still keeps one more owner of the game that may change it. The words
change (decisions 4, 17, 19): the game is `shares mut`; `setup` takes one owner of it (`owns game:
shares mut Game`, called `setup(shares mut game)`) instead of a read-only view, because an owner made
from a read-only name only reads (R3, E3023); the callback holds that owner (`with owns game`) and says
what it changes, `changes Game`. The count is now made at `setup`'s call, not at the capture.

**1. Intent:** a C event library calls back into the game; each call adds a point to that game.

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

    fn add_point() changes score:
        this.score += 1

unsafe extern "C":
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)

fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:
    return register_cb(fn() with owns game changes Game: game.add_point())

fn main():
    game = shares mut Game()
    token = setup(shares mut game)
    println(game.score)
```

The rules:
- The capture list says what the callback C keeps holds (row 6, R15): here `setup`'s owner of the
  game, handed to the callback (`with owns game`). A one-thread owner (`shares mut`) may go to it
  because the contract says `threads(creator)`, which the trampoline checks; a `threads(any)` callback
  needs a `thread_shared` value. A plain capture (the real variable) is allowed only for
  `callback(borrowed)`, which C calls only during the call.
- **Checked trampoline** (run 2's 8.3 with the attack's fixes, 7.1 and 7.2): C holds a tagged
  `{slot, generation}` number, never an Ember address; the top 16 bits are `0x4080`, not an address,
  so a C library that wrongly writes through it faults before writing anything. The trampoline checks
  the slot is inside a table that never moves, then the generation and the function's type in one
  compare (a call after `unregister` stops with the callback's name), then the calling thread for
  `threads(creator)`/`threads(main)`. A `threads(any)` callback that can be unregistered while it runs
  announces the running slot, and unregistering waits.
- The contract may be read from the header when its shape is a function pointer plus a `void*` it
  hands back (run 2's 8.4); it is listed by `ember tcb` as an asserted fact, like a written one.
- **Who may change the game** (R3, R4, R9). `main`'s `game` is the first owner and may change it.
  `setup(shares mut game)` makes one more owner that may change it and hands it to `setup`'s `owns`
  parameter (R4: the caller writes `shares mut game` for an `owns x: shares mut T` parameter); `with
  owns game` hands that owner on to the callback. A plain parameter `game: shares mut Game` would be a
  read-only view of `main`'s owner (R4, R5), and an owner made from it may do no more than it, so
  `with shares mut game` from it is refused (E3023, below); that is why `setup` takes an owner.
- **What the callback changes** (R12, decision 17). It changes the game through an owner it shares
  with `main`, a link other names reach, so its first line names the type: `changes Game` (checked
  against its body, E2105: `add_point` says `changes score`). A foreign call made on the creator thread
  may run it, so such a call is taken to make that change (section 2, reading 3): no view of a game's
  part and no walk of a game's list lives across it (R8).

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `score` (2) | none (default: the object's own number) | none (default: changed only by a method whose first line says `changes score`) |
| `this` (4, not listed: the method's object, R5) | `this` (the `Game` `add_point` was called on) | `changes score` (row 3) |
| `cb` (8-10) | `extern "C" fn` (a C function pointer, its type) | none (default: a parameter only reads) |
| `ud` (8-10) | `*mut void` (C's raw pointer; the contract names it user data) | none (default: Ember never reads it here) |
| `game` (12) | `owns` (one owner of the game, handed to `setup` by the call) | `shares mut` in its type: an owner that may change the game (R4, R9) |
| `game` (13, capture) | `owns` (`setup`'s owner, handed to the callback C keeps; `setup`'s `game` ends here) | `shares mut` (its type), and the callback's `changes Game` (R12) |
| `game` (16) | `shares mut` (a new object with several owners; this is the first) | `shares mut` (this owner may change it) |
| `token` (17) | none (default: the `Retained` token the call returns, the caller's own; dropping it unregisters) | none |

8 names, 2 without a word.

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

**6. Still refused:**
- No capture word for a kept callback: `register_cb(fn() changes Game: game.add_point())`:
  `error[E7004]: C keeps this function after \`setup\` returns (\`callback(retained)\`), so it cannot
  use \`setup\`'s \`game\`; say what it holds: \`fn() with owns game changes Game:\` (\`setup\`'s owner
  of the game), or \`fn() with weakref game changes Game:\` with \`if game holds ref mut g:
  g.add_point()\` as its body (a link that lets the game end)`.
- The callback without `changes Game`: `register_cb(fn() with owns game: game.add_point())`:
  `error[E2105]: this function changes \`game\`, a \`Game\` other owners share, through \`add_point\`
  (\`changes score\`); say so on its first line: \`fn() with owns game changes Game:\`` (R12).
- A changing owner made from a read-only view: `fn setup(game: shares mut Game)` with `with shares mut
  game`: `error[E3023]: \`game\` only reads the \`Game\` that \`main\` lends to \`setup\`, so an owner
  made from it only reads too; take one owner of it: \`fn setup(owns game: shares mut Game)\`, call it
  \`setup(shares mut game)\`, and capture it \`with owns game\``.
- The game kept by a `threads(any)` callback: `error[E7015]: C may call this function from any thread
  (\`threads(any)\`), and \`shares\` owners stay on one thread; share the game between threads: write
  \`game = thread_shared Game()\` and \`token = setup(game)\` in \`main\`, \`fn setup(game:
  thread_shared Game)\` and \`fn() with thread_shared game: game.add_point()\` in \`setup\`, and, since
  \`add_point\` changes \`score\`, \`score: Atomic[i32] = Atomic(0)\`, \`fn add_point():\` with
  \`this.score.fetch_add(1)\`, and \`println(game.score.load())\`` (R9, R15; `V2/rules.md` section 3's
  message, completed with every line it replaces, R18: a `thread_shared` value is changed only through
  `Atomic` or a lock, so `add_point` changes nothing its first line names, and the function value needs
  no `changes`, section 2, reading 1).
- A capturing lambda where C wants a bare function pointer: E5040, today's message (it already gives
  the line to write).
- Today's `owned fn(...)`, as `register_cb(owned fn(): game.add_point())`: `error[E1010]: \`owned
  fn\` is not a word of Ember; name what the function takes or copies on each captured name: \`fn()
  with owns game changes Game:\`; a name not listed is the real variable` (and `owns fn(...)`: E0109,
  the same advice).

**7. Cost:** one plain count add at `setup(shares mut game)` and one subtract when the callback is
unregistered (Rust's `Rc::clone` and drop); the capture hands the owner over with no count. The checked
trampoline, 1.05x plain C (the attack, `phase3b-threads-attack.md:902`,
164.575 against 156.179 ms; 1.02x in its second batch; the tag adds no instruction, 163.755 ms,
line 960), against today's trampoline 1.47x (`phase3b-threads.md:1133`). The announced
`threads(any)` version that may be unregistered while running costs 1.42x to 1.58x on an empty
callback (the attack, 7.1): above C's cost, and only on that path; Rust's workaround (0.97x,
`phase3b-threads.md:1137`) checks none of these facts.

**8. Kind:** a new mechanism (the checked trampoline), safer than Rust's: a stale, forged or
wrong-thread call is caught, where Rust's workaround is an unbounded `unsafe` reference.

---

### AN-79: `unsafe` points two ways, and an `unsafe fn` body is unchecked with no mark per operation

**Meaning changed:** no.

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
own note (Rust 2024's `unsafe_op_in_unsafe_fn`, made an error, E3100).

Today's compiler asks for a `# SAFETY:` note on the `unsafe fn` line too (`[UNS-8]`): the NEW text
without it gets W3012 there (`target/redesign3/15-vocabulary-review/an79_no_fn_note.em`, checked again
in this run as `target/redesign4/06-area-E/old/an79_no_fn_note.em`, the same output):

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
  \`read_u32_le\` is an \`unsafe fn\`, which says its callers must vouch; the read at line 4 is this
  function's own unsafe step: put it in an \`unsafe:\` block with a \`# SAFETY:\` note saying why it
  holds here`.
- `unsafe fn` with no `@safety` on any function, not only `pub` ones (today `L3015` for `pub`):
  `warning[L3015]: say what callers of \`read_u32_le\` must make true:
  \`@safety("p points to four readable bytes")\``.
- An `unsafe:` block with no note: `W3012`, today's message (it already gives the line).
- A `# SAFETY:` note on the `unsafe fn` line itself (the OLD's line 2): `warning[W3013]:
  \`read_u32_le\` says what its callers must make true with \`@safety\`; a \`# SAFETY:\` note says why
  one block is sound and goes above each \`unsafe:\` block inside it: remove this line`.
- Calling an `unsafe fn` outside `unsafe`: E3100, today's.

**7. Cost:** none; compile-time only.

**8. Kind:** a readable spelling of Rust's guarantee (Rust 2024's rule, with the two directions told
apart by where `unsafe` sits and by their two notes).

---

### AN-80: `Sync` names three things; `@sync` reads "synchronised"; `@nosync` reads "not `@sync`"

**Meaning changed:** no.

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

    fn note(n: int) changes words, longest:
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
    a = thread.spawn(fn() with thread_shared st: st.note(3))
    st.note(5)
    a.join()
```

The three meanings get three spellings (run 3's ruling, standing):
- the **effect** keeps `Sync` and **`@nosync`** keeps its spelling: "no synchronisation", which is what
  it forbids (locks, atomics stronger than relaxed, channels, spawn and join, spec 3184-3186);
- the **property** "several threads may hold and read one value" is **`ThreadShared`**, written only in
  bounds (`fn keep[T: ThreadShared](x: T)`) and messages, inferred for every type as Rust infers
  `Sync` (R15);
- the **marker** `@sync` is **removed**: what it said on the class (handles cross threads, atomic
  counts, fields frozen) is said on the value where it is shared, `thread_shared Stats()`, and which
  fields may change is said on each field (`guarded by`, `Atomic`). With the marker gone, `@nosync` can
  no longer be read as "not `@sync`".

The read and the write of `longest` now sit under one lock, so `longest` ends as 5 whichever thread
runs first (the OLD's race, both reading 0 and the one with 3 storing last, cannot happen). The
thread's function changes `st` only through `note`, under its lock, so its first line names nothing
(section 2, reading 1).

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `lock` (2) | none (default: the object's own lock) | none (default: changes only by being held) |
| `words` (3) | none (default: the object's own number) | `guarded by lock` |
| `longest` (4) | none (default: the object's own number) | `guarded by lock` |
| `this` (6, not listed: the method's object, R5) | `this` (the `Stats` `note` was called on) | `changes words, longest` (row 3) |
| `n` (6) | none (default: a plain parameter reads the caller's value) | none |
| `xs` (13) | none (default: reads the caller's numbers; `Span` is a view) | none |
| `s` (14) | none (default: a new local, the function's own) | none (default: its own) |
| `x` (15) | none (default: a read-only view of each number, copied as an `int`, R2) | none |
| `st` (20) | `thread_shared` (one of several owners, on any thread) | `thread_shared` (changed only through `note`, under the lock) |
| `a` (21) | none (default: the `JoinHandle` the call returns, the caller's own) | none |
| `st` (21, capture) | `thread_shared` (one more owner) | `thread_shared` |

11 names, 6 without a word.

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
- `@sync class Stats`: `error[E0104]: \`@sync\` is gone: a class needs no mark to be shared between
  threads; share the object where it is shared, \`st = thread_shared Stats()\`, and say which lock
  guards each field that changes (\`words: int guarded by lock\`) or make it \`Atomic[int]\``.
- `T: Sync` in a bound: `error[E1010]: \`Sync\` is the effect \`@nosync\` forbids, not a property of
  a type; for "threads may share it" write \`T: ThreadShared\``.
- `st.longest = n` outside the lock: `error[E7013]: \`longest\` is guarded by \`lock\`; change it
  inside \`with st.lock:\`` (AN-74's message, in this program's names).
- `@nosync` reaching a lock: `E4002`, today's, which names the chain (`[EFF-6]`).

**7. Cost:** none from the words; the lock is C's (AN-74).

**8. Kind:** a readable spelling of Rust's guarantee (Rust's `Sync`, inferred, named `ThreadShared`;
the class marker replaced by words on the value and the fields).

---

### AN-81: `@static_safe` forbids only aliasing checks

**Meaning changed:** no.

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
- A run-time alias check inside a `@no_alias_checks` function, written out: AN-03's `step` marked
  `@no_alias_checks`, whose `collide(ref mut bodies[i], ref mut bodies[j])` (line 3) is compared once
  while running (R4): `error[E4030]: \`step\` is \`@no_alias_checks\`, and \`collide(ref mut
  bodies[i], ref mut bodies[j])\` (line 3) compares \`i\` and \`j\` while running; prove them different
  (\`if i != j:\` around the call), or remove \`@no_alias_checks\`` (`RUN4/rules.md` section 3). The
  run-time alias checks left in the model are R4's index compare and `assert_disjoint`.

**7. Cost:** none; the contract is checked when compiling.

**8. Kind:** a readable spelling of Rust's guarantee (a rename: the contract keeps its meaning and is
named for it).

---

### AN-82: `@no_runtime_checks` is reserved, promising a switch that will never exist

**Meaning changed:** no.

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

**Meaning changed:** no.

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

**Meaning changed:** no (the `MutSpan` argument says `ref mut` at the call, decision 3).

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

**3. NEW:** one meaning per code (`RUN4/rules.md` section 4):

| code | means | was |
|---|---|---|
| `E5015` | this type may not cross into C as written (an owning Ember value in an exported signature; a type with no C form) | the same, plus the variadic case |
| **`E5013`** (new) | C would change this further argument of a variadic call (a default promotion, or an untyped literal); pass it as C reads it | E5015 |
| `E7010` | a parallel loop's chunks may write one place | the same, plus the export case (spec only) and the loop-shape case |
| **`E7012`** (new) | a module variable of the main thread (`static mut` not `guarded by` a lock) is reached by code that runs on another thread (an unscoped thread, an `any`/`creator` export, a C callback not `threads(main)`) | spec 4966's E7010 |
| **`E7016`** (new) | a `@parallel` loop is over something it cannot split into chunks: neither a range (`for i in 0..n:`) nor a list walked by element (`for ref mut p in xs:`), for example a `while` loop | the compiler's E7010 at `for p in players` |

Messages under the new codes:

```text
error[E5013]: C reads the further argument `narrow` as an `int`, and an `i16` would be promoted on the way
  --> an84_e5015_variadic.em:10:31
   = help: pass it as C reads it: `narrow as c_int`
```

```text
error[E7012]: `bump` may be called from any thread (`@export(threads=any)`), and it changes `HITS`, a module variable of the main thread
   = help: guard it: `static mut HITS: int guarded by HITS_LOCK`, and change it inside `with HITS_LOCK:`; or make it `Atomic[int]`
```

E7016's message, on the refused program written out (`place` with its loop as a `while` loop):

```ember
fn place(xs: MutSpan[int], idx: Span[int]):
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

The programs the new codes' messages lead the programmer to, in NEW spelling (run 4's, with one change,
decision 3: the argument given to `place`'s `MutSpan` parameter says `ref mut` at the call, R4):

```ember
import c "./variadic.c" with (overlay = "overlays/variadic.em") as v
from std.ffi import c_int
from std.sync import Lock

pub extern "C" fn read_text(text: *u8) -> i32:         # E5015's help: a C type in the export
    return 0

fn place(xs: MutSpan[int], idx: Span[int]):             # E7010's help: indexed by its own `i`
    @parallel
    for i in 0..idx.len():
        xs[i] = idx[i]

static HITS_LOCK: Lock = Lock()
static mut HITS: int guarded by HITS_LOCK = 0           # E7012's help

@export(threads=any)
fn bump() changes HITS:
    with HITS_LOCK:
        HITS += 1

fn main():
    narrow: i16 = 3
    unsafe:  # SAFETY(ffi): the further argument is passed as the `int` C reads
        println(v.sum_wide(1, narrow as c_int))         # E5013's help
    a = [0, 0, 0]
    order = [2, 0, 1]
    place(ref mut a.as_mut_span(), order.as_span())
    println(a)
```

`place(ref mut a.as_mut_span(), order.as_span())`: `ref mut` says the call may change `a`'s elements
(before a `MutSpan` the word names its elements, so it stands before the span made in the call, R4;
without it, E3028: "`place` changes the elements of `a` (its parameter `xs` is a `MutSpan`); say so at
the call: `place(ref mut a.as_mut_span(), order.as_span())`"). The two arguments are views of two
different lists, `a` and `order`, so R4's "arguments do not overlap a changeable one" is proved when
compiling, with nothing checked while running, and `xs` is compiled with noalias (R4).

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `v` (1) | `import ... as v`: the C module's name | none (default: a module's name changes nothing) |
| `text` (5) | `*u8` (its type): a raw pointer to bytes C owns | `*u8` without `mut`: reads only |
| `xs` (8) | `MutSpan` (its type): a view of the caller's numbers, the only name for them while `place` runs (R8 (f)) | `MutSpan`: its elements may be written (R5's exception by type); the caller says `ref mut` (line 27) |
| `idx` (8) | `Span` (its type): a view of the caller's numbers | `Span`: read only |
| `i` (10) | none (default: the loop's own number) | none (default: only the loop changes it) |
| `HITS_LOCK` (13) | `static`: the module's own lock | none (default: a `Lock` changes only by being held with `with`) |
| `HITS` (14) | `static`: the module's own number | `mut`, `guarded by HITS_LOCK` |
| `HITS` (19) | `static` (line 14) | `changes HITS` on `bump` (line 17), inside `with HITS_LOCK:` (line 18) |
| `narrow` (22) | none (default: `main`'s own number) | none (default: its own) |
| `a`, `order` (25, 26) | none (default: `main`'s own new lists) | none (default: its own; `place` writes `a`'s elements, said at the call: `ref mut a.as_mut_span()`, line 27) |

11 names, 4 without a word (`i`, `narrow`, `a`, `order`), each with its default stated; none of the
defaults changes, aliases or copies anything the line does not show.

**5. Means exactly** (`means/an84_means.em`): the programs after following each help (the refusals
stay; only the codes change): `narrow as c_int`, an export taking `*u8`, a parallel loop indexed by
its own `i`:

```text
exit=0
```

**6. Still refused:** everything refused today stays refused, under the codes above.

**7. Cost:** none.

**8. Kind:** a readable spelling of Rust's guarantee (the same refusals; each code now names one
thing, as Rust's codes do).

## 4. Words needed, and for the owner

### 4.1 Words needed

None. Every word this area's NEW blocks and messages use has a row in `V2/vocabulary.md`. Checked by
`target/redesign4/v2-06-area-E/check_words.py` (output in `check_words.txt` beside it): it lists every
name in the NEW code blocks (part 3 of each answer) and in the inline code of parts 3 and 6 that does
not occur inside a `` `...` `` span of the vocabulary. Of the 173 names in the NEW blocks, those it lists
are all the programs' own names (`count_words`, `Names`, `Button`, `Game`, `HITS`, `texts`, `token`,
`report`, ...). Of the inline code, it lists the programs' own names (`FRAME_LOCK`, `collide`, `bodies`,
`parts`, `mine`, `log2`, `cache`), error codes, plain English inside quoted messages, `global` (the
Python spelling a message refuses, AN-76), Rust's lint name `unsafe_op_in_unsafe_fn` and the tool
command `ember tcb` (AN-78), none of which is a word of an Ember program. The five rows run 4 asked for
(`count`; a raw pointer's `read` and `add`; `*T`; `i16`; `user_data`) are now in the vocabulary
(`V2/changes.md` section 5, "E words"). Run 4's AN-76 named `jobs.submit`, which has no row; this file
says "an unscoped job of the job system" instead.

### 4.2 For the owner

Run 4's question (AN-77, a generator kept while its list changes) is answered by decision 16 and
applied. Four new points, one at a time, each with what this file does meanwhile:

1. **Does a function that changes a value shared between threads say so?** (AN-75, AN-80; section 2,
   reading 1.) `fn() with thread_shared log: log.push("a")` changes the log, under its lock. The v2
   rules say a function type names what it changes through a `weakref`, `shares mut` or `sharedref
   mut` link, and that changes through `Mutex`, `RwLock`, `Atomic` or a channel need no word; they do
   not say which of the two a `thread_shared` value changed under a `guarded by` lock is. Memory safety
   does not need the word (taking the same lock twice on one thread waits, it never changes a list under
   a walk). Meanwhile this file writes no word, as for a `Mutex`. The other choice is
   `fn() with thread_shared log changes Log: log.push("a")`, which says more on the line and costs
   nothing while running.
2. **What a thread's function and a C callback may change** (AN-76, AN-78; section 2, reading 2).
   Decision 17 makes `thread.spawn(fn() changes RESULTS: worker(1))` and
   `register_cb(fn() with owns game changes Game: game.add_point())` say what they change. The rules do
   not say what function type `thread.spawn`, `scope.spawn` and a C `callback(...)` parameter accept;
   this file takes it to be a function of any `changes` (a thread is not a call, and its changes are
   checked by the thread rules), so neither is E2113. This is the library's first line to write
   (for example `fn spawn[C](owns f: fn() changes C) -> JoinHandle`), next to `V2/changes.md` section 11,
   question 1 (`for_each` and friends).
3. **A foreign call may run a callback C keeps** (AN-76 rule 3, AN-78; section 2, reading 3). Run 4's
   R12 counted "the callbacks C may run on this thread during" a foreign call; v2's R12 replaced that
   rule with function types and no longer says it for C. Without it, a loop over a game's list across a
   call such as `poll_events()` would not see that a kept callback may push to that list. This file
   reads it by types: a foreign call on a thread where kept callbacks may run makes every change their
   types say. This is a rule, so the rule writer or the lead should settle it in R12 or R15.
4. **A plain parameter of a `shares mut` type cannot pass on an owner that may change** (AN-78). Run 4
   gave `setup` a plain parameter and made the callback's owner from it in the capture. Under R4 and
   R5 a plain parameter only reads, and R3 refuses a changing owner made from a reading name (E3023),
   so the v2 program is `fn setup(owns game: shares mut Game)`, called `setup(shares mut game)`, with
   `with owns game`: the call says the callee gets one more owner that may change the game, which the
   line before did not say. This file takes that as the rules' intent (the line now says who may
   change the game). The other reading lets a plain `x: shares mut T` parameter make `shares mut x`
   owners, which is shorter but lets a read-only name create a writer.
