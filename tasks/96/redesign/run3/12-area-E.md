# 12 area E: threads, locks across threads, `unsafe` (run 3, phase 2)

Agent 12 of run 3. Duty: answer every annoyance of area E (AN-71 .. AN-84 in `RUN3/annoyances.md`)
with the words of `RUN3/vocabulary.md` and `RUN3/planned-words.md`, repairing last run's threads
answers (`tasks/96/redesign/phase3b-threads.md`, called "last run" below, and its attack
`phase3b-threads-attack.md`, called "the attack"). Area E has no agreed row of its own; the rows it
uses (3, 5, 6, 9) are used exactly as agreed (section 6).

Power: `PowerLineStatus` printed `Online` at the start and at the end of this work.

Every claim about today's Ember is the real output of `target/redesign3/bin/ember.exe check`, pasted.
Files, all under `C:\Users\ism19\Code\ember\target\redesign3\12-area-E\`:
- `old/`: today's programs (the OLD of each answer), copied from phase 1
  (`tasks/96/redesign/artifacts/phase1/sharing/`) or written here;
- `new/`: every NEW block of this report, as a file (proposed syntax; today refuses it);
- `means/`: every "means exactly" translation in today's Ember (each checked, `exit=0`);
- `c/an76_static_mut.c`: the one translation today's Ember cannot say, in C (built and run);
- `check_summary.txt`: the exit code of `ember check` on every file in `old/` and `means/`.

No Ember program was run. No compiler, std, spec or test file was changed.

## 0. Terms used here

- **thread**: a sequence of instructions the operating system runs alongside others, sharing memory.
  **task**: a function handed to another thread. **scope**: the block `with scope = thread.scope():`;
  its tasks may use the parent's variables because the block does not end until every task has
  finished (the **join**). An **unscoped thread** (`thread.spawn(...)`) may outlive the function that
  started it.
- **data race**: two threads touch one place in memory at the same time, at least one writes, and
  nothing orders them. In C and Rust it is undefined behaviour; in practice it loses updates or frees
  memory still in use.
- **lock**: an object at most one thread holds at a time; `with self.lock:` waits until it is free,
  holds it for the block, and lets it go at the block's end.
- **atomic**: an operation two threads cannot interleave (on this processor a `lock`-prefixed
  instruction), several times dearer than a plain one.
- **count**: the number of owners of a value that has several (`shared`, `thread_shared`); the value
  is freed when it reaches zero. A **one-thread count** changes with a plain add, an **atomic count**
  with an atomic add.
- **effect summary**: what the compiler records about a function from its body: which places it reads,
  changes and **reshapes** (changes in a way that can free or move memory: replace, `push`, `clear`).
  Ember already computes effect sets (`[EFF-1]`) and stores them in package interfaces (`[EFF-4]`).
- **view**: a value that points into something else and may not outlive it (`ref T`, `Span[T]`,
  `StrView`).
- **reduction**: per-task results combined after the join.
- **trampoline**: the small C function Ember generates for a C library to call; it finds the Ember
  function and calls it. **user data**: the `void*` a C library hands back to its callback.
- **the owner's test** (brief): for every name in a line, the word that says what it is (the owner's
  own thing, a copy, or handed over) and the word that says whether it may change it. In the tables, a
  name counts as **without a word** when neither column has a word and both say "none (default: ...)".
- **kind**: the job's four outcomes: a readable spelling of Rust's guarantee; a smarter compile-time
  solution; a new mechanism; an understandable limitation kept.

## 1. The model for area E in one page

What changes from last run, in one sentence: last run decided everything from effect summaries and
had programmers write no thread words at all; this run keeps every compile-time check last run and
its attack saved, and adds the words the owner asks for wherever the program's meaning would
otherwise be decided by the compiler: what a thread holds (the capture list, row 6), which lock guards
which field, and which module variable a function changes.

1. **Scoped tasks need no words** (last run's scope rule, kept with the attack's fixes). Inside
   `with scope = thread.scope():`, every task's effect summary is checked against every other task's
   and against the parent's lines until the join. A name a task uses is the real variable (row 6),
   which lives until the join; a view (`for t in texts`) is fine; a local of one loop round is not,
   and gets row 6's `with copied n`. Two tasks that change one place are refused, except:
   - **reductions**: `total += e`, `-=`, `*=`, `|=`, `&=`, `^=`, `max`/`min` on a number, `push`,
     `extend`, `add` on a list or set, and `m[k] op= e` on a map or array, written in the task's own
     body, compile to one private accumulator per task, combined at the join in spawn order (last run
     4.4; the attack's element reductions, 3.2). No task and no parent line may read the place before
     the join, so no line of the program can tell this from every task changing the real variable;
   - changes through `Atomic`, `Mutex`, `RwLock`, a channel, or a field `guarded by` a lock while
     holding it (3 below);
   - each task's own element: `for ref mut p in players:` hands out one element per task.
2. **What an unscoped thread or a kept C callback holds is written in its capture list** (row 6's
   words, 07's capture positions): `with owned names` (handed over; the caller's `names` ends),
   `with thread_shared log` (one more owner, on any thread; atomic count), `with copied n` (its own
   copy), `with shared game` / `with weak game` (only where the function stays on this thread, which a
   C contract `threads(creator)` or `threads(main)` guarantees). A name not in the list is the real
   variable, which an unscoped thread may outlive: refused, naming the words.
3. **A lock guards the fields its class names** (new words, section 7): `words: int guarded by lock`.
   Every read or change of `words` must sit inside `with x.lock:` on the same object, on the thread
   that holds it (the attack's "held is per thread": a task starts holding no lock). Fields not
   written after `init` need no lock. This replaces last run's inference of which lock guards which
   field, and the `@sync` rule that froze every field.
4. **`thread_shared` says a value has owners on several threads** (07's word). Its count is atomic
   because the word says so; no owner changes it directly, only through `Atomic`, `Mutex`, `RwLock`, a
   channel, or a `guarded by` field under its lock. `@sync` (06's `@thread_shared`) is removed: what it
   said on the class is now said on the value at the place it is shared. The property `ThreadShared`
   (06) stays for bounds and messages, inferred as Rust infers `Send` and `Sync`.
5. **What may go to another thread is inferred from the words**, not written as `Send`: a value
   handed over (`owned`) may go when nothing reachable from it is `shared` (a one-thread count), a
   `weak` link into or out of a one-thread value, or **thread-bound** (the attack's fact: a lock's
   guard, a `Scope`, a raw pointer, a C handle whose contract says `threads(creator)`). The `unsafe`
   author of a raw-pointer wrapper lifts it with today's `unsafe extend Window implements Send:`.
6. **Module variables**: `static mut FRAME: int = 0`, changed only by functions whose first line says
   `changes FRAME` (row 3's word on a free function, 07). Used only by the main thread and its scoped
   tasks; for other threads, `static mut RESULTS: Array[int] guarded by RESULTS_LOCK`.
7. **Generators walk by position** across `yield` (the attack's 6.1, 6.2); `async` stays reserved.
8. **C callbacks** go through the checked trampoline (last run 8.3 with the attack's fixes and tag).
9. **`unsafe`**: one word; on a declaration it says "callers must vouch" (with `@safety`), on a block
   "this code vouches" (with `# SAFETY:`); inside an `unsafe fn` body every unsafe operation sits in
   its own `unsafe:` block (Rust 2024's rule, made an error).

What runs while the program runs, compared with C: nothing for scoped tasks and reductions (plain C's
per-thread counter); a lock where the program takes one; an atomic count change where a
`thread_shared` owner is made or ends (Rust's `Arc`); netted counts where scoped tasks re-point
`shared` fields of parent objects (the attack, 1.05x); the checked trampoline per C callback (1.02x
to 1.05x). Each answer below gives its own numbers and their sources.

---

## 2. The answers

### AN-71: threads (A15): the loop variable, `total +=` from tasks, E7004 offering `@sync`

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

**3. NEW** (`new/an71_new.em`):

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
    h = thread.spawn(fn() with owned names: names.items.len())
    println(h.join())

fn share_and_keep():
    names = Names()
    h = thread.spawn(fn() with thread_shared names: names.items.len())
    println(names.items.len(), h.join())
```

`count_words` is the OLD text, unchanged: it now compiles. `share_plain` hands the object to the
thread; `share_and_keep` shares it and keeps reading it.

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `texts` (1) | none (default: a parameter reads the caller's list; vocabulary `ref`/`mut` rows) | none (default: read-only) |
| `total` (2) | none (default: a new local, the function's own) | none (default: the function's own variable; `+=` is the change, written) |
| `scope` (3) | `with` (bound for this block, ended at its end) | none (default: read-only binding) |
| `t` (4) | none (default: a read-only view of each element of `texts`; vocabulary `for` row) | none (default: read-only) |
| `total` (5, in the task) | none (default, row 6: a name not in the capture list is the real variable) | none (default, row 6: the real `total` changes; `+=` written) |
| `t` (5, in the task) | none (default: the same view of the element of `texts`, which lives until the join) | none (default: read-only) |
| `items` (9) | none (default: the object's own list) | none (default: changed only through `changes items` or `mut self`, row 3) |
| `names` (12) | none (default: a new local, the function's own object) | none (default: its own) |
| `h` (13) | none (default: the `JoinHandle` the call returns, the caller's own; 07's same default) | none |
| `names` (13, capture) | `owned` (handed to the thread; the caller's `names` ends here) | `owned` (the thread's own now) |
| `names` (17) | none (default: a new local, the function's own object) | none |
| `h` (18) | none (default as line 13) | none |
| `names` (18, capture) | `thread_shared` (one more owner, on any thread) | `thread_shared` (no owner changes it directly) |
| `names` (19) | `thread_shared` (line 18's capture made the caller's name one of the owners, as row 9's `shared items` does to `items`) | `thread_shared` (read-only from now on) |

14 names; 10 without a word, each with its default stated. No default changes anything the line does
not show, aliases silently, or copies silently: the per-task accumulator behind line 5 is a
compile-time translation no line can observe (see 5).

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

**6. Still refused** (codes in section 8):

- A task reads `total` before the join: `scope.spawn(fn(): if total > 10: println("many"))`:
  `error[E7007]: tasks add their parts to \`total\`, which is known only after the scope ends at line
  7; read \`total\` after the \`with\` block`.
- A task changes `total` in a way the compiler cannot add up per task (AN-72's message).
- A float place one task adds to more than once: `error[E7007]: each task adds to \`sum\` many times;
  float sums depend on how the adds are grouped, so the result would not be any order of single adds;
  add up in the task first: \`mine = 0.0\`, \`mine += x\` for each, then \`sum += mine\` once`. (With
  one add per task the spawn-order result is one order of single adds.)
- Using `names` after `with owned names`: `error[E3040]: \`names\` was handed to the thread at line 13
  (\`with owned names\`); to keep using it here, write \`with thread_shared names\``.
- An unscoped thread using a name with no capture word: `thread.spawn(fn() => names.items.len())`:
  `error[E7004]: this thread may outlive \`share_plain\`, so it cannot use \`share_plain\`'s
  \`names\`; say what the thread holds: \`fn() with owned names:\` (handed over) or \`fn() with
  thread_shared names:\` (one more owner)`. It never offers `@sync`.
- Handing over a value that holds a one-thread owner: if `Names` had `cache: shared Cache` and the
  caller another owner of it, `error[E7015]: \`names\` holds \`cache\`, which has other owners on this
  thread (line 9), whose counts change without atomics; share it between threads:
  \`cache: thread_shared Cache\``. The same for a class that is the target of a `weak` field anywhere
  in the build (its weak links stay on this thread): "share it instead: `with thread_shared names`".
- Changing a `thread_shared` value directly: `names.items.push(s)` after line 18: `error[E7014]:
  \`names\` is shared between threads since line 18, so no owner changes it directly; keep the list in
  a field \`guarded by\` a lock and change it inside \`with names.lock:\``.
- A thread-bound value sent to a task (a guard, a `Scope`, a raw pointer; the attack's `x05`, `x08`,
  `x09`): `error[E7008]: \`g\` is the guard of \`m\`'s lock, which belongs to the thread that took it;
  take the lock inside the task: \`with g = m.lock():\``.

**7. Cost.** Scoped tasks and reductions: nothing beyond plain C's per-thread counter. Last run,
`phase3b-threads.md:381-386` (64 MiB of text, 8 tasks; PROVISIONAL): `reduce-0`, plain C with a local
counter per thread summed after the join, 43.548 ms (1.00), which the reduction compiles to; an atomic
add per word (today's `Atomic` spelling) 10.22x; safe Rust's `AtomicU64` 10.35x, its join-sum 1.08x.
`with owned names`: a pointer handed over, no count. `with thread_shared names`: one atomic increment
when the thread's owner is made and one atomic decrement when it ends, Rust's `Arc` price. Because the
word says which values are thread-shared, the count kind is known when compiling: last run's run-time
"shared flag" (and its walk, 5.451 ms per 262,144 objects, `phase3b-threads.md:538`) is not needed.

**8. Kind:** a smarter compile-time solution (the scope rule and reductions), with a readable spelling
of Rust's `move` + `Arc` for unscoped threads (the capture words).

---

### AN-72: errors land on the wrong variable in `a15_threads.em`

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

**3. NEW** (`new/an72_new.em`): a program that stays refused, and the one error it gets:

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
   = help: to add each task's part, write `total += t.len()`; to combine the parts another way, have each task keep its own (`parts.push(t.len())`) and combine `parts` after the `with` block
```

The rule: a task's conflict is reported once, on the captured name that causes it; errors on other
names of the same function that only follow from it are not shown, and no error ever names a name the
program uses correctly (`t` here).

**4. The owner's test:** `texts`, `total` (2), `t` (4), `total` (5), `t` (5): none (defaults as in
AN-71's table, the same lines); `scope`: `with`. 6 names, 5 without a word.

**5. Means exactly.** The program is refused; the line its help gives (`total += t.len()`) is AN-71's
program, whose translation is `means/an71_means.em` (`exit=0`, pasted in AN-71).

**6. Still refused:** the NEW program itself, with the message above.

**7. Cost:** none; a compile-time message.

**8. Kind:** a smarter compile-time solution (the error found at its cause).

---

### AN-73: "cannot find `n`" (E1010) inside a lambda that calls an `Atomic` method

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

**3. NEW** (`new/an73_new.em`):

```ember
fn count(texts: Array[String]) -> int:
    total = Atomic(0)
    with scope = thread.scope():
        for t in texts:
            n = t.split_whitespace().count()
            scope.spawn(fn() with copied n: total.fetch_add(n))
    return total.load()
```

The defect is fixed (a name used as an argument of a method call inside a lambda is looked up like any
other), and the task names what it copies, row 6's words. `t` needs no word: it is a view of an
element of `texts`, which lives until the join; `n` is the round's own number, which does not.

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `texts` (1) | none (default: a parameter reads the caller's list) | none (default: read-only) |
| `total` (2) | none (default: a new local, the function's own) | none (default: an `Atomic` changes only through its own methods) |
| `scope` (3) | `with` | none (default: read-only binding) |
| `t` (4) | none (default: a read-only view of each element) | none |
| `n` (5) | none (default: a new local of this round, its own) | none (default: its own) |
| `n` (6, capture) | `copied` (the task's own copy, row 6) | `copied` (the task's own) |
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
`error[E3060]: the task uses \`n\`, which ends at the end of this round of the loop (line 5), while
the task may still be running; give the task its own copy: \`fn() with copied n:\``.

**7. Cost:** one atomic add per task, as the program asks (an `Atomic` is the programmer's choice;
`total += n` on a plain `int` would be AN-71's reduction at plain C's cost). The copy of `n` is one
8-byte store into the task's state.

**8. Kind:** a smarter compile-time solution (a defect fixed, and the real cause named with row 6's
words).

---

### AN-74: a `@sync` class freezes every field after `init`; objects written by several tasks need a lock or a claim

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

**3. NEW** (`new/an74_new.em`):

```ember
from std.sync import Lock

class Stats:
    lock: Lock = Lock()
    words: int guarded by lock = 0
    tags: Array[String] guarded by lock = []

    fn add_all(self, text: StrView) changes words, tags:
        for w in text.split_whitespace():
            with self.lock:
                self.words += 1
                if w.starts_with("#"):
                    self.tags.push(String(w))

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
- **`guarded by lock`** (new, section 7) on a field: the field is read and changed only inside
  `with x.lock:` on the same object `x` (the same name, not re-pointed in the block; `self` in
  methods), on the thread that holds it, except in `init`. Several fields may name one lock; a class
  may have several locks. A helper whose body uses a guarded field without taking the lock is
  compiled as "needs `self.lock` held" (last run's attack, inferred and carried in function types) and
  may be called only with the lock held. A `with` on a lock this thread is known to hold calls the
  callee's entry that skips taking it again (the attack's static re-entry; no run-time owner). A view
  of a guarded field does not leave the `with` block (row 4's rule, area B).
- No class mark: `@sync` is gone (AN-80). The fields that are never written after `init` need nothing.
- **Each task its own object**: `for ref mut p in players` in a `@parallel` loop hands each chunk its
  own elements. An element of `Array[Player]` has one owner, the list, because a second owner of a
  class object needs a word (06's `class` row, C-3: `p2 = player` hands over or is refused; a list
  holding one object twice would be `Array[shared Player]`). So no two chunks can reach one object,
  and no run-time claim is needed. This depends on area C ruling the `class` row's condition as 06
  wrote it; if area C keeps silent sharing of class objects, the claim (the attack's 3.5) is the
  fallback (section 9).

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `lock` (4) | none (default: the object's own lock) | none (default: a `Lock` changes only by being held with `with`) |
| `words` (5) | none (default: the object's own number) | `guarded by lock` (only while holding `lock`) |
| `tags` (6) | none (default: the object's own list) | `guarded by lock` |
| `self` (8) | `self` (the value the method was called on) | `changes words, tags` (row 3) |
| `text` (8) | `StrView` (a view of someone else's text) | none (default: read-only) |
| `w` (9) | none (default: a read-only view of each word) | none |
| `texts` (15) | none (default: reads the caller's list) | none |
| `stats` (16) | none (default: a new local, the function's own object) | none (default: its own; changed through `add_all`, which says `changes`) |
| `scope` (17) | `with` | none |
| `t` (18) | none (default: a read-only view of each element) | none |
| `stats` (19, task) | none (default, row 6: the real `stats`) | none (default: through `add_all`'s `changes words, tags`, under the lock) |
| `t` (19, task) | none (default: the same view, alive until the join) | none |
| `players` (27) | `ref mut` (the caller's list) | `ref mut` (may change it; 06's C-1) |
| `p` (29) | `ref mut` (each element of the caller's list) | `ref mut` |

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
  lock, so `stats.words += 1` in it is the first message.
- A field written after `init` by two tasks with no lock and no `guarded by`: `error[E7007]: two
  tasks change \`stats.total\` (line 19); guard it: \`total: int guarded by lock\`, and change it inside
  \`with self.lock:\`; or make it \`Atomic[int]\``.
- A list whose objects may have a second owner in a parallel loop: `for ref mut p in players` with
  `players: Array[shared Player]`: `error[E7010]: \`players\` holds \`shared Player\` objects, and one
  object may be in the list twice, so two chunks could change it at once; give each object one owner
  (\`Array[Player]\`), or change it under its lock (\`hp: int guarded by lock\`)`.

**7. Cost.** A lock where the program writes one, the lock C takes (last run, `phase3b-threads.md:741`,
`lock-0`, lock-use-unlock in C, 1479.479 ms against safe Rust's `Mutex` 2170.016 ms, PROVISIONAL);
no field is wrapped, so the object is its plain fields plus one lock word. One thread that uses such a
class pays an uncontended lock per `with`, as Python and C pay. The parallel loop over one-owner
elements: plain loads and stores, no count test and no claim: the per-object claim's 1.12x to 1.36x
(the attack, 3.5) is gone because the words prove what the claim checked while running.

**8. Kind:** a readable spelling of Rust's guarantee (`Mutex<Inner>` spelled as Python's lock field,
with the field saying which lock guards it), plus a smarter compile-time solution for the per-element
loop.

---

### AN-75: clone-before-move for shared handles (A34)

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

**3. NEW** (`new/an75_new.em`):

```ember
from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(self, s: StrView) changes lines:
        with self.lock:
            self.lines.push(String(s))

fn start(log: thread_shared Log):
    a = thread.spawn(fn() with thread_shared log: log.push("a"))
    b = thread.spawn(fn() with thread_shared log: log.push("b"))
    a.join()
    b.join()

fn main():
    start(thread_shared Log())
```

The capture word does what Rust's `Arc::clone` line does, on the name it is about: each thread gets
one more owner. No second name (`log2`) and no separate clone line.

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `lock` (4) | none (default: the object's own lock) | none (default: changes only by being held) |
| `lines` (5) | none (default: the object's own list) | `guarded by lock` |
| `self` (7) | `self` | `changes lines` (row 3) |
| `s` (7) | `StrView` (a view of someone else's text) | none (default: read-only) |
| `log` (11) | `thread_shared` (in its type: a log shared between threads) | `thread_shared` (no owner changes it directly; `push` changes a guarded field) |
| `a` (12) | none (default: the `JoinHandle` the call returns, the caller's own) | none |
| `log` (12, capture) | `thread_shared` (one more owner) | `thread_shared` |
| `b` (13) | none (default as `a`) | none |
| `log` (13, capture) | `thread_shared` | `thread_shared` |

9 names, 3 without a word.

**5. Means exactly** (`means/an75_means.em`, which is phase 1's `a34_clone_dance.em` itself): a
`@sync` class (atomic counts), its list inside a `Mutex`, and `owned fn` retaining the handle once per
thread. `exit=0`.

**6. Still refused:**
- One thread hands over and the other uses: `with owned log` in both: `error[E3040]: \`log\` was
  handed to the first thread at line 12 (\`with owned log\`); to give both threads the log, write
  \`with thread_shared log\` in both`.
- A one-thread owner sent to a thread: `with shared log`: `error[E7015]: \`shared\` owners stay on
  one thread, and this function runs on a new one; write \`with thread_shared log\``.
- No word: `thread.spawn(fn() => log.push("a"))`: AN-71's E7004 message.
- `with thread_shared log` where `log` is a plain parameter `log: Log` (a view of an object whose
  owner may be one-thread): `error[E7015]: \`log\` is lent to \`start\`, and its owner may not be shared
  between threads; take it as \`log: thread_shared Log\``.

**7. Cost:** one atomic increment per `with thread_shared` capture and one atomic decrement when the
thread's owner ends: exactly Rust's `Arc::clone` and drop, and what today's `owned fn` does for a
`@sync` handle. The lock is C's.

**8. Kind:** a readable spelling of Rust's guarantee (`Arc::clone` + `move`, written as one word on the
captured name).

---

### AN-76: global mutable state (A19)

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

**3. NEW** (`new/an76_new.em`):

```ember
static mut FRAME: int = 0
static mut LOG: Array[String] = []

fn tick() changes FRAME, LOG:
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

The rules (07 asked area E to rule on a safe `static mut`):
1. `static mut NAME: T = e` declares a module variable that may change. It needs no `unsafe`; a
   foreign `static mut` (in an `extern` block) keeps `[STA-1]`'s `unsafe` at every use. The
   initialiser follows `[STA-3]`, as today's `static`s.
2. A function whose own body changes it says so on its first line: `changes FRAME` (row 3's word,
   07's position on a free function). Checked against the body both ways: a change not named is
   refused, a name not changed is refused. Callers write nothing; the compiler carries the change in
   the caller's effect summary, so every check below sees it through any chain of calls, and its
   messages name the chain.
3. One thread: a loop over `LOG`, or any view into it, across a call whose summary changes `LOG` is
   refused (the agreed limitation: change after the loop). A foreign call's summary includes the
   callbacks C may run on this thread during it (the attack's 5.1, `y16`), so a C call that runs an
   Ember callback which pushes to `LOG` counts as a change.
4. Threads: a `static mut` is used only by the main thread and the tasks of scopes it opens (where
   tasks' changes follow the scope rule and reductions, AN-71). A function reachable from
   `thread.spawn`, `jobs.submit`, a C callback whose contract is not `threads(main)`, or an
   `@export(threads=any|creator)` may not read or change it (E7012) unless it is
   `guarded by` a `static` lock, inside `with` that lock.
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
| `x` (11) | none (default: a parameter reads the caller's value) | none (default: read-only) |
| `r` (12) | none (default: a new local, the function's own) | none (default: its own) |
| `RESULTS` (14) | `static` (line 9) | `guarded by RESULTS_LOCK` (line 9), held by line 13's `with` |
| `a` (18) | none (default: the `JoinHandle` the call returns, the caller's own) | none |
| `b` (19) | none (default as `a`) | none |

11 names, 4 without a word.

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
  and never changes it; remove \`LOG\` from its first line`.
- A loop over `LOG` calling `tick()`: `error[E3020]: this loop walks \`LOG\`, and \`tick\` (line 15)
  changes \`LOG\` (\`fn tick() changes FRAME, LOG:\`, line 4); collect what to add in a local list and
  push it after the loop`.
- A worker thread using an unguarded one: `thread.spawn(fn(): tick())`: `error[E7012]: \`tick\`
  changes \`FRAME\`, a module variable of the main thread, and this thread runs beside it; guard it:
  \`static mut FRAME: int guarded by FRAME_LOCK\` and change it inside \`with FRAME_LOCK:\`, or make it
  \`Atomic[int]\``.
- `FRAME = 0` at module level (Python's spelling): `error[E1030]: \`FRAME = 0\` at module level is a
  statement, and this file has \`fn main\`; declare a module variable: \`static mut FRAME: int = 0\``.
- `global FRAME`: today's E0100 help, kept ("`global` is Python's"), pointing at `changes FRAME`.

**7. Cost:** plain C globals: nothing counted, nothing checked while running. Last run measured the
same work (`phase3b-threads.md:854-863`, `tick()` called 100,000,000 times, PROVISIONAL): plain C
globals 192.219 ms (1.00); today's Ember workaround (`Atomic` + `Mutex`) 3.10x on one thread, 7.15x
once a second thread has started; safe Rust's `AtomicU64` + `Mutex` 7.44x; Rust's `thread_local!`
1.04x. The guarded list costs the lock C takes.

**8. Kind:** a smarter compile-time solution (Rust needs `unsafe` or atomics for this; here the
function's first line says what it changes and the compiler proves only one thread group reaches it).

---

### AN-77: async and generators (A14)

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

**3. NEW** (`new/an77_new.em`):

```ember
gen fn lines() -> Generator[String]:
    buf = [String("a"), String("b")]
    for s in buf:
        yield copied s

class Door:
    angle: float = 0.0
    log: Array[String] = []

    gen fn replay(self) -> Generator[int]:
        for entry in self.log:
            yield entry.len()

fn main():
    for s in lines():
        println(s)
    d = Door()
    for n in d.replay():
        println(n)
```

The rule (the attack's 6.1 and 6.2, kept): a `for` loop over a list, string, map or set whose body
contains `yield` keeps a position in the generator's frame and, after each resume, reads the
collection's header again from where it lives now (the frame, or the object's field) and checks the
position against the current length. No address is held across the suspension, so the frame may move.
Between two resumes the ordinary loan rule applies. When the generator is consumed in place by a `for`
loop in the same function (`for n in d.replay():`), the consumer's body may not change `Door.log`: the
agreed limitation for any loop (a list does not change while a loop walks it). When the generator is
stored or passed, the walk stays memory-safe by position and sees the list as it is at each resume, as
Python's list iterator does for `append` and `pop`; it differs from Python only when the whole field is
replaced (Python keeps walking the old list), and the reference says so.

`async` stays reserved in this version (understandable limitation: no async runtime exists). Its
design is last run's (`phase3b-threads.md` 7.3 rule 3, the attack's 6.3): task groups, which are
scopes; frames that never move; the scope rule in place of `Send`.

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `buf` (2) | none (default: a new local, the generator's own) | none (default: its own) |
| `s` (3) | none (default: a read-only view of each element) | none |
| `s` (4, yielded) | `copied` (row 9: a new, independent `String` is given out) | `copied` (the receiver's own) |
| `angle` (7) | none (default: the object's own field) | none (default: changed only through `changes angle` or `mut self`) |
| `log` (8) | none (default: the object's own list) | none (default as `angle`) |
| `self` (10) | `self` | none (default: read-only, row 3) |
| `entry` (11) | none (default: a read-only view of each element) | none |
| `s` (15) | none (default: the `String` the generator gives, the loop's own) | none (default: its own) |
| `d` (17) | none (default: a new local, the function's own object) | none |
| `n` (18) | none (default: the `int` the generator gives) | none |

10 names, 8 without a word.

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

**6. Still refused:**
- A reference to the frame's own inline local across `yield` (`r = ref n`, then `yield`, then `r`):
  `error[E2221]: \`r\` points into this generator's own frame, which may move while it is suspended;
  keep the value instead (\`m = n\`), or its position`.
- The in-place consumer changing the walked field: `for n in d.replay(): d.log.clear()`:
  `error[E3020]: \`replay\` walks \`d.log\` while this loop runs; clear it after the loop`.
- A walk with no position (a linked structure held by views): E2221 as today, with "walk it by an
  index or a `Handle`".
- `async`: `error[E0005]: \`async\` is planned for a later version; for work that waits, use a
  generator or threads (\`with scope = thread.scope():\`)`.

**7. Cost:** the measured model's own. The attack (`phase3b-threads-attack.md:766-775`, 104,857,600
values over a 1 MiB string, PROVISIONAL): plain C loop 69.943 ms (1.00); position re-read each resume,
`next()` inlined, 82.222 ms (1.18); an address held across `yield` (what the old rule allowed) 82.020
ms (1.17); a class object's field walked by position 71.308 ms (1.02). The gap to plain C is the
generator protocol, not the walk.

**8. Kind:** a smarter compile-time solution (generators); `async` is an understandable limitation kept.

---

### AN-78: unsafe and FFI (A20): a C callback with user data

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

**3. NEW** (`new/an78_new.em`):

```ember
class Game:
    score: i32 = 0

    fn add_point(self) changes score:
        self.score += 1

unsafe extern "C":
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)

fn setup(game: shared Game) -> Retained[ForeignCallback]:
    return register_cb(fn() with shared game: game.add_point())

fn main():
    game = shared Game()
    token = setup(game)
    println(game.score)
```

The rules:
- The capture list says what the callback C keeps holds (row 6, 07's capture words): here one more
  owner of the game. `with shared` is allowed because the contract says `threads(creator)`, which the
  trampoline checks; a `threads(any)` callback needs `with thread_shared`. A plain capture (the real
  variable) is allowed only for `callback(borrowed)`, which C calls only during the call.
- **Checked trampoline** (last run 8.3 with the attack's fixes, 7.1 and 7.2): C holds a tagged
  `{slot, generation}` number, never an Ember address; the top 16 bits are `0x4080`, not an address, so
  a C library that wrongly writes through it faults before writing anything. The trampoline checks the
  slot is inside a table that never moves, then the generation and the function's type in one compare
  (a call after `unregister` stops with the callback's name), then the calling thread for
  `threads(creator)`/`threads(main)`. A `threads(any)` callback that can be unregistered while it runs
  announces the running slot, and unregistering waits.
- The contract may be read from the header when its shape is a function pointer plus a `void*` it
  hands back (last run 8.4); it is listed by `ember tcb` as an asserted fact, like a written one.
- A foreign call's effect summary includes the callbacks C may run on this thread during it (AN-76
  rule 3).
- Whether `game.add_point()` may change the game through one of its `shared` owners is area C's
  ruling (AN-48); the callback is just one more owner on the same thread, and follows it.

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `score` (2) | none (default: the object's own number) | none (default: changed only through `changes score`) |
| `self` (4) | `self` | `changes score` (row 3) |
| `cb` (8-10) | `extern "C" fn` (a C function pointer, its type) | none (default: a parameter only reads) |
| `ud` (8-10) | `*mut void` (C's raw pointer; the contract names it user data) | none (default: Ember never reads it here) |
| `game` (12) | `shared` (one of several owners, this thread) | none (default: changed only through `add_point`'s `changes score`) |
| `game` (13, capture) | `shared` (one more owner) | as line 12 |
| `game` (16) | `shared` (a new object with several owners) | as line 12 |
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
- No capture word for a kept callback: `register_cb(fn(): game.add_point())`: `error[E7004]: C keeps
  this function after \`setup\` returns (\`callback(retained)\`), so it cannot use \`setup\`'s
  \`game\`; say what it holds: \`fn() with shared game:\` (one more owner) or \`fn() with weak game:\`
  (a link that lets the game end)`.
- `with shared game` under `threads(any)`: `error[E7015]: C may call this function from any thread
  (\`threads(any)\`), and \`shared\` owners stay on one; write \`with thread_shared game\``.
- A capturing lambda where C wants a bare function pointer: E5040, today's message (it already gives
  the line to write).
- `owned fn(...)`: `error[E0100]: name what the function copies or takes: \`fn() with copied score:\``
  (vocabulary, `owned fn` row).

**7. Cost:** the checked trampoline, 1.05x plain C (the attack, `phase3b-threads-attack.md:902`,
164.575 against 156.179 ms; 1.02x in its second batch; the tag adds no instruction, 163.755 ms,
line 960), against today's trampoline 1.47x (`phase3b-threads.md:1133`). The announced
`threads(any)` version that may be unregistered while running costs 1.42x to 1.58x on an empty
callback (the attack, 7.1): above C's cost, and only on that path; Rust's workaround (0.97x,
`phase3b-threads.md:1137`) checks none of these facts.

**8. Kind:** a new mechanism (the checked trampoline), safer than Rust's: a stale, forged or
wrong-thread call is caught, where Rust's workaround is an unbounded `unsafe` reference.

---

### AN-79: `unsafe` points two ways, and an `unsafe fn` body is unchecked with no mark per operation

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

**3. NEW** (`new/an79_new.em`):

```ember
@safety("p points to four readable bytes")
# SAFETY: the caller guarantees `p` points to four readable bytes
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

The rule: one word, two places, each with its own sentence. `unsafe` before a declaration (`unsafe
fn`, `unsafe interface`, an `unsafe fn(...)` type) says "whoever uses this must vouch", and `@safety`
says what they must make true. `unsafe` before a block or an implementation (`unsafe:`, `unsafe
extend`, `unsafe extern`, `unsafe overlay`) says "this code vouches", and `# SAFETY:` says why. Inside
an `unsafe fn` body every operation of `[UNS-1]` must still sit in an `unsafe:` block, with its own
note (Rust 2024's `unsafe_op_in_unsafe_fn`, made an error, E3100). 07 planned no second spelling
(`planned-words.md` section 6) and the position already tells the two apart; this answer agrees.

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `p` (3) | `*u8` (a raw pointer to someone else's bytes) | `*u8` without `mut`: reads only |
| `data` (8) | none (default: a parameter reads the caller's bytes; `Span` is a view) | none (default: read-only) |

2 names, 1 without a word.

**5. Means exactly** (`means/an79_means.em`, which is the NEW text: today accepts it):

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
- Calling an `unsafe fn` outside `unsafe`: E3100, today's.

**7. Cost:** none; compile-time only.

**8. Kind:** a readable spelling of Rust's guarantee (Rust 2024's rule, with the two directions told
apart by where `unsafe` sits and by their two notes).

---

### AN-80: `Sync` names three things; `@sync` reads "synchronised"; `@nosync` reads "not `@sync`"

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

**3. NEW** (`new/an80_new.em`):

```ember
class Stats:
    lock: Lock = Lock()
    words: int guarded by lock = 0
    longest: int guarded by lock = 0

    fn note(self, n: int) changes words, longest:
        with self.lock:
            self.words += n
            if n > self.longest:
                self.longest = n

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

The three meanings get three spellings, as 06 settled (C-4), and area E rules on the marker (07,
section 8):
- the **effect** keeps `Sync` and **`@nosync`** keeps its spelling: "no synchronisation", which is what
  it forbids (locks, atomics stronger than relaxed, channels, spawn and join, spec 3184-3186);
- the **property** "several threads may hold and read one value" is **`ThreadShared`** (06), written
  only in bounds (`fn keep[T: ThreadShared](x: T)`) and messages, inferred for every type as Rust
  infers `Sync`;
- the **marker** `@sync` (06's placeholder `@thread_shared`) is **removed**: what it said on the class
  (handles cross threads, atomic counts, fields frozen) is said on the value where it is shared,
  `thread_shared Stats()`, and which fields may change is said on each field (`guarded by`, `Atomic`).
  With the marker gone, `@nosync` can no longer be read as "not `@sync`".

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `lock` (2) | none (default: the object's own lock) | none (default: changes only by being held) |
| `words` (3) | none (default: the object's own number) | `guarded by lock` |
| `longest` (4) | none (default: the object's own number) | `guarded by lock` |
| `self` (6) | `self` | `changes words, longest` (row 3) |
| `n` (6) | none (default: a parameter reads the caller's value) | none |
| `xs` (13) | none (default: reads the caller's numbers; `Span` is a view) | none |
| `s` (14) | none (default: a new local, the function's own) | none (default: its own) |
| `x` (15) | none (default: a read-only view of each number) | none |
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
- `st.longest = n` outside the lock: AN-74's E7013 message.
- `@nosync` reaching a lock: `E4002`, today's, which names the chain (`[EFF-6]`).

**7. Cost:** none from the words; the lock is C's (AN-74).

**8. Kind:** a readable spelling of Rust's guarantee (Rust's `Sync`, inferred, named `ThreadShared`;
the class marker replaced by words on the value and the fields).

---

### AN-81: `@static_safe` forbids only aliasing checks

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

**3. NEW** (`new/an81_new.em`):

```ember
@no_alias_checks
fn total(xs: Span[int], n: int) -> int:
    s = 0
    for i in 0..n:
        s += xs[i]
    return s
```

The same contract, spelled by what it forbids (vocabulary, 06, and 07 kept it): it reads as the rest of
the `@no...` family (`@noalloc` forbids allocation; `@no_alias_checks` forbids run-time alias checks,
E4030).

**4. The owner's test:** `xs` none (default: reads the caller's numbers), `n` none (default: reads the
caller's value), `s` none (default: the function's own local), `i` none (default: the loop's own
number). 4 names, 4 without a word; none changes anything of the caller's.

**5. Means exactly** (`means/an81_means.em`, the OLD text: `@static_safe` is today's spelling of the
same contract): `exit=0`.

**6. Still refused:**
- `@static_safe`: `error[E0104]: \`@static_safe\` is now \`@no_alias_checks\`: it forbids run-time
  alias checks only; bounds and overflow checks still run`.
- A run-time alias check inside: `error[E4030]: \`total\` is \`@no_alias_checks\`, and
  \`RuntimeChecked\` (line N) checks aliasing while running; restructure the loop so the two views are
  disjoint by construction, or remove \`@no_alias_checks\``.

**7. Cost:** none; the contract is checked when compiling.

**8. Kind:** a readable spelling of Rust's guarantee (a rename: the contract keeps its meaning and is
named for it).

---

### AN-82: `@no_runtime_checks` is reserved, promising a switch that will never exist

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

**3. NEW** (`new/an82_new.em`): no such word. The function the programmer can write:

```ember
fn total(xs: Span[int]) -> int:
    s = 0
    for x in xs:
        s += x
    return s
```

**4. The owner's test:** `xs` none (default: reads the caller's numbers), `s` none (default: its own
local), `x` none (default: a read-only view of each number). 3 names, 3 without a word.

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

**3. NEW** (`new/an83_new.em`, 07's spelling):

```ember
@fp(allow_fma)
fn dot(ax: float, ay: float, bx: float, by: float) -> float:
    return ax * bx + ay * by
```

**4. The owner's test:** `ax`, `ay`, `bx`, `by`: none (default: parameters read the caller's
values). 4 names, 4 without a word.

**5. Means exactly** (`means/an83_means.em`, the OLD text, today's spelling): `exit=0`.

**6. Still refused:** `@fp(contract)`: `error[E0104]: \`@fp(contract)\` is now \`@fp(allow_fma)\`: it
lets the compiler fuse \`a * b + c\` into one fused multiply-add`. `@fp(allow_fma)` inside a
`@deterministic` function: E4072, as `@fastmath` today ("`dot` is `@deterministic`, and a fused
multiply-add rounds once where two operations round twice; remove `@fp(allow_fma)`").

**7. Cost:** none more; one instruction where two were.

**8. Kind:** a readable spelling of Rust's guarantee (a rename of an argument; Rust has no such
attribute, its `f64::mul_add` is explicit).

---

### AN-84: two error codes mean two unrelated things each: E5015 and E7010

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

**3. NEW:** one meaning per code:

| code | means | was |
|---|---|---|
| `E5015` | this type may not cross into C as written (an owning Ember value in an exported signature; a type with no C form) | the same, plus the variadic case |
| **`E5013`** (new) | C would change this further argument of a variadic call (a default promotion, or an untyped literal); pass it as C reads it | E5015 |
| `E7010` | a parallel loop's chunks may write one place | the same, plus the export case (spec only) and the loop-shape case |
| **`E7012`** (new) | a module variable of the main thread (`static mut` not `guarded by` a lock) is reached by code that runs on another thread (an unscoped thread, an `any`/`creator` export, a C callback not `threads(main)`) | spec 4966's E7010 |
| **`E7016`** (new) | a `@parallel` loop is over something it cannot split (it counts over a range, or walks a list with `for ref mut p in xs`) | the compiler's E7010 at `for p in players` |

The other area E codes this report uses are in section 8.

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

**4. The owner's test:** no NEW names (error codes only): 0 names.

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

---

## 3. How the answers keep memory safe (the argument, for 16 and 17 to attack)

A data race needs two threads, one place, a write, and nothing ordering them. Every place two threads
can reach is one of these, and each is ordered or never written:
1. a place a scoped task reads: the parent and the other tasks may not change it until the join
   (scope rule; summaries over-approximate unknown code as "every effect");
2. a place tasks change by a reduction: each task has its own accumulator; the join orders them;
3. an `Atomic`, `Mutex`, `RwLock`, channel end, or a `guarded by` field: ordered by the atomic or the
   lock, held by the thread that touches it (a task starts holding none);
4. a value handed over (`owned`): the giver's name has ended, and nothing reachable from it is a
   one-thread owner, a `weak` link of a one-thread value, or thread-bound (E7015, E7008);
5. a `thread_shared` value: never changed directly by any owner (E7014), its count is atomic;
6. a `static mut`: reached only by the main thread and its scoped tasks (E7012), or `guarded by`.

Freed while in use: a scoped task's views die at the join, which is before the block ends; an
unscoped thread holds only what its capture list names (an owner, a copy, or a `weak` link that is
tested on read); one-thread counts never cross threads (E7015); where scoped tasks re-point `shared`
fields of parent objects, count changes are netted per task and decreases wait until every task has
finished its loop (the attack, 3.1, kept); a C callback holds a checked slot, not an address.

## 4. What changed from last run, and the good parts kept (repairs)

| last run's answer | this run | why | good parts kept |
|---|---|---|---|
| scope rule (4.3), the attack's fixes (thread-bound values, held-per-thread locks, nested spawns, netted counts) | **kept** | it is compile-time and costs nothing | all of it |
| reductions (4.4), element reductions (attack 3.2) | **kept**; float many-adds refused with a message | a float sum grouped per task is not any order of single adds | spawn-order combine; `m[k] op= e` private copies (2.264 ms against 8.019 ms on one thread, the attack, line 357) |
| lock field whose guarded fields are inferred (4.5; attack's lockset rule) | **replaced** by `guarded by` written on each field | the owner's rule: what the programmer means is written; a reader of the class could not tell which lock guards which field | the lockset check (now checking what `guarded by` says), held per thread, "needs the lock" in function types, static re-entry, a wait ending every view of what the lock guards (for `Lock.wait`, when it exists) |
| shared flag (4.6): a bit set at run time when an object first reaches another thread | **not needed**: the count kind follows the word (`shared` one-thread, `thread_shared` atomic) | the words now say which values are shared between threads, so it is known when compiling | the flag and its free test (same instructions, 1.00x to 1.04x, `phase3b-threads.md:542-544`) are the fallback if area C keeps class objects silently shared |
| claims (4.7, saved narrowed by the attack at 1.12x to 1.36x) | **not needed** when list elements have one owner (06's `class` row) | the words prove what the claim checked while running | the claim with atomic take and drop (the attack, 3.5) is the fallback for `Array[shared T]` if area C ever allows changes there; 2.5x cheaper than Rust's lock per object (`phase3b-threads.md:597`) |
| no thread words at all, also for unscoped threads (4.3 rule 5) | **changed**: unscoped threads and kept callbacks name what they hold (row 6's capture list) | the owner's test: a name the thread outlives must say whether it is handed over, shared, copied or a weak link | scoped tasks still need none |
| `@sync` as the class marker, `Send`/`Sync` written | **removed** (`@sync`); `ThreadShared` inferred; `Send` written only by `unsafe` authors | AN-80 | today's `unsafe extend T implements Send:` for raw-pointer wrappers |
| module variables in Python's spelling, `FRAME = 0` and `global FRAME` (6.3) | **replaced** by `static mut FRAME` + `changes FRAME` | 07 planned `changes` on a free function and no `global`; a module-level assignment is a statement of the implicit `main` today (E1030) | loans across calls that change it; scoped tasks and reductions; unscoped threads refused; callbacks in foreign calls' summaries (attack 5.1) |
| thread-local module variables (6.4, discarded) | still discarded | not the meaning of a module variable | free when wanted (0.58x, `phase3b-threads.md:859`); no word planned |
| base-pointer layout for globals (6.3, discarded by the attack) | still discarded | a store-to-load effect on one variable | benchmarks must address globals the same way on both sides |
| generators: heap-contents rule (7.3), replaced by the attack's walk by position | **kept as the attack's**; the in-place consumer may not change the walked field | the agreed limitation: a list does not change while a loop walks it | the heap-contents fact, useful to area B for views into a value's own heap contents |
| stackful coroutines (7.4, discarded) | still discarded | about 5.4 KB per suspended task against about 135 bytes (`phase3b-threads.md:1039`) | no function colouring; blocking code runs unchanged |
| checked trampolines (8.3), inferred contracts (8.4), the attack's tag | **kept** | 1.02x to 1.05x and safer than Rust | all of it |
| `copy(...)` for a list out of a lock (5.3) | `copied` (vocabulary) | one spelling for a copy | — |
| references out of a lock (A33, 5.x) | **area B's** (agreed row 4) | moved by 05 | the guard's region rule; the accessor (`yield` inside `with`) |
| `@cached` slots with a once-flag (section 9) | **ruled here** (section 5) | 07 asked area E | — |

## 5. Rulings 07 asked area E for (`planned-words.md` section 8)

- **Whether `@thread_shared` stays once `thread_shared` exists:** no; removed (AN-80).
- **Whether a class object held `shared` uses a one-thread count:** yes; the count follows the word, for
  every type. A `shared` value never reaches another thread (E7015); a value created `shared` cannot
  later be made `thread_shared` while other one-thread owners exist ("create it `thread_shared` from the
  start: `names = thread_shared Names()`").
- **Safe `static mut` with `changes`:** yes, under AN-76's rules.
- **`@cached` across threads:** on an object several threads reach (a `thread_shared` value, or a
  scope's tasks), the slot is filled under a once-flag (`[STA-3]`'s `Once`: an acquire load and a branch
  when filled). A `@cached` method whose `reads` clause names a field that changes while other threads
  hold the object (`guarded by` or `Atomic`) is refused: "`radius` reads `verts`, which other threads
  change under `lock`; a kept result could be stale; remove `@cached`, or keep the result in a field
  `guarded by lock`".
- **The `unsafe fn` body rule:** every unsafe operation in its own `unsafe:` block (AN-79).

## 6. The agreed rows

No agreed row belongs to area E. The rows the answers use, each exactly as agreed:
- **Row 3** (`changes`): `fn add_all(self, text: StrView) changes words, tags:` (AN-74), `fn note(...)
  changes words, longest:` (AN-80), and 07's new position on a free function, `fn tick() changes
  FRAME, LOG:` (AN-76). The meaning is the row's: the fields (here also module variables) the function
  changes, checked against its body.
- **Row 5** (`String(...)`): `String(w)`, `String("tick")`: the caller's own `String` made from a view.
- **Row 6** (capture list): a name not in it is the real variable (AN-71's `total`, AN-74's `stats`);
  `with copied n` (AN-73) is the row's own form; 07's `with owned`, `with shared`, `with thread_shared`
  follow its place and meaning.
- **Row 9** (`copied`): `yield copied s` (AN-77).
- Rows 4, 7, 8 are not used by these answers and are untouched; AN-74's guarded fields rely on row 4's
  refusal (a view of a guarded field does not leave the `with` block), unchanged.

## 7. New words

| word | meaning | where written | reason (why no word of `vocabulary.md` or `planned-words.md` says it) | example | answers |
|---|---|---|---|---|---|
| `guarded` (in the clause `guarded by <lock>`; `by` is already a contextual word in 07's `set by`) | this field (or module variable) is read and changed only while holding the named lock, on the thread that holds it; outside `init` | after a field's type, or a `static mut`'s type, naming a `Lock` field of the same object or a `static` lock | which lock protects which field is what the programmer means and a reader must see; `changes` says a method changes a field, not under which lock; `Mutex[T]` says it for one value but forces several fields into a separate struct (AN-74's translation); inferring it (last run) leaves it unwritten. Java's `@GuardedBy` and Clang's `GUARDED_BY` say the same on the field | `words: int guarded by lock = 0` | AN-74, AN-75, AN-76, AN-80 |
| `Lock` (a `std.sync` type) | a lock with no value inside; `with x.lock:` holds it for the block (the bare `with` of vocabulary settlement 10: "hold this until the block ends") | in a field or `static` type; `Lock()` | `Mutex[T]` holds a value and is taken with `m.lock()`; a lock that guards several fields of its object has no value of its own, and `with self.guard.lock():` on a `Mutex[void]` reads twice. Python's `threading.Lock`; last run's proposal (`phase3b-threads.md:399`) | `lock: Lock = Lock()` | AN-74, AN-75, AN-76, AN-80 |

Today both are refused: `new/an74_new.em` gives `error[E1061]: module 'std.sync' has no item named
\`Lock\`` and `error[E1010]: cannot find type \`Lock\` in this scope` (first errors; `guarded` is not
reached). `grep -rnw` over `std/src` finds no name `guarded` and no name `Lock`.

No other new word: `thread_shared`, `weak`, the capture positions, `changes` on a free function and
`@fp(allow_fma)` are 07's; `ThreadShared`, `@no_alias_checks`, `static mut`, `unsafe`, `@safety`,
`Send` are the vocabulary's.

## 8. Error codes used by these answers

| code | message shape | new or changed |
|---|---|---|
| E7004 | a thread or a kept callback uses a name without saying what it holds; names `with owned` / `with thread_shared` (or `with shared` / `with weak` for `threads(creator)`) | changed message (never offers `@sync`) |
| E7007 | two tasks of one scope touch one place and one changes it, other than by a sum or push added up at the join, or under a lock | new (today E3021/E3022 on whatever name) |
| E7008 | a value tied to one thread (a lock's guard, a `Scope`, a raw pointer, a `threads(creator)` handle) goes to another thread | new (today E7004/E7005 "not Send/Sync") |
| E7010 | a parallel loop's chunks may write one place | narrowed (AN-84) |
| E7012 | a `static mut` not `guarded by` a lock is reached from another thread | new (spec 4966's E7010) |
| E7013 | a `guarded by` field used without holding its lock | new |
| E7014 | a `thread_shared` value changed directly | new (replaces E7003) |
| E7015 | a one-thread owner (`shared`, a `weak` link of one) or a lent value reaches another thread | new (replaces E7001, E7005's class cases) |
| E7016 | a `@parallel` loop over something it cannot split | new (AN-84) |
| E5013 | a variadic further argument C would change | new (AN-84) |
| E2105 | a function's `changes` clause does not match its body (the code area B gives row 3's check; named here only as a placeholder for that check) | for area B / 14 to number |
| E3060, E3100, E0104, E0005, E1030 | new messages as written in each answer | changed messages |

Retired: E7001, E7003 (with `@sync`), E7005 (its cases go to E7008 and E7015).

## 9. Dependencies on other areas

- **Area C**: the `class` row's condition (06, C-3): a second owner of a class object needs a word.
  AN-74's per-element loop with no claim, AN-71's `with owned names`, and the static count kinds all
  rest on it. If area C keeps silent sharing of class objects, the fallbacks are last run's shared flag
  and the attack's claims (section 4). Whether an owner of a `shared` value may change it (AN-48)
  decides AN-78's `game.add_point()`.
- **Area D**: the capture list (row 6) and 07's capture words; `weak` reads (`with weak game`).
- **Area B**: row 3's `changes` check (code numbering), row 4's guard region (views of guarded fields).
- **Area A**: the agreed limitation (a list does not change while a loop walks it) used by AN-76 and
  AN-77; `for ref mut p in players` in a `@parallel` loop.

## 10. Found in passing (today's compiler; nothing was changed)

1. **A `@parallel` loop cannot change a field of its own struct element**: `ps[i].hp -= 1` with
   `ps: MutSpan[Player]` is `E7010: the parallel loop writes through a place its chunks share`
   (`old/an74_parallel_field_write.em`); writing the whole element `ps[i] = Player(...)` is accepted
   (`means/an74_means_each.em`). The spec's `[PAR-2]` (c) allows a place in a captured `MutSpan`.
2. **E7010 for a loop shape, with an unrelated help**: `@parallel for p in players` gives "counts over a
   range `a..b`" with the help "an `Atomic`, `chunks_mut`, or `@parallel(reduce=…)`"
   (`old/an74_parallel_class.em`).
3. **A list of scoped join handles cannot be kept**: `parts.push(scope.spawn(...))` into
   `Array[ScopedJoinHandle[int]]` is `E3063` ("a view, so it may not be stored in an `Array` unless it is
   `static`"), and a second `E3063` is then reported at the call `count_words([...])` in `main`
   (`old/an71_scoped_handles_list.em`, both errors, `exit=1`). Rust keeps a `Vec` of scoped handles.
4. **The E1010 lambda defect needs the non-`owned` form**: `fn() => total.fetch_add(n)` says "cannot
   find `n`"; `owned fn() => tr.fetch_add(n)` compiles (`means/an73_means.em`).
5. **`std.cell` is not implemented** (`E0900`), so `Cell` in a probe fails before any check of its own.
6. **`Atomic` has no `fetch_max`** (`E2072`, a first version of `means/an80_means.em`).
