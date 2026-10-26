# Area E, v3: threads, locks across threads, `unsafe` (AN-71 to AN-84)

Agent 06 (area-E), run 4, step 2 (v3). Every answer of area E in the eight-part form, brought to the v3
rules and words, which apply all of the owner's verdicts 1 to 34 (`inputs/JOB.md`, "Owner verdicts on
run 4's open questions"; 27 replaces 25): `V3/rules.md` (rules R1 to R18, the codes), `V3/vocabulary.md`
(the words) and `V3/changes.md` (what changed from v2). `RUN3` means `tasks/96/redesign/run3/`, `RUN4`
means `tasks/96/redesign/run4/`, `V2` means `tasks/96/redesign/run4/v2/`, `V3` means
`tasks/96/redesign/run4/v3/`. "R1" .. "R18" are the v3 rules; "row N" is the owner's agreed row N and
"verdict N" the owner's verdict N in `inputs/JOB.md` (v2 wrote "decision N" for verdicts 1 to 20; this
file says "verdict N" throughout). Terms are those of `V3/rules.md` section 0; the few this area adds are
in section 1 below.

No compiler change, no git command, no Ember program run, and no new `ember check`: this file makes no
new claim about what today's Ember accepts or refuses, and where it cites today's compiler on a new
spelling it cites the rule writer's probes (`V3/changes.md` section 7, `r1` .. `r8`). No power reading
was taken: this step ran in a cloud container.

**How the OLD code and its output were carried.** This file is phase 0's copy of `V2/area-E.md`
(`V3/spelling-changes.txt` lists its three replaced lines), edited in place. No OLD program of this area
changes, so every OLD block, its output and every "means exactly" block is v2's, and so run 4's, byte for
byte: `target/redesign4/v3-06-area-E/scripts/check_carried.py` compares every fenced block of parts 2 and
5 of each answer, and every pasted output of today's compiler elsewhere in the file, with `V2/area-E.md`
(output in `target/redesign4/v3-06-area-E/check_carried.txt`: 54 blocks, all identical). Run 4 checked
every OLD and "means exactly" file again with today's compiler (`target/redesign4/06-area-E/recheck.txt`,
on the owner's laptop: the same codes and exit codes as pasted); none of those files changed since. The
probe files the answers cite (`old/...`, `means/...`, `c/...`) were in the laptop's ignored `target/` and
are not in this checkout; their pasted outputs carry over unchanged. File names inside carried text are
relative to `target/redesign3/12-area-E/`, and "section N" inside carried text means a section of run 3's
area report `RUN3/12-area-E.md`. The "means exactly" programs are written in today's Ember, so they keep
today's `self`, `@sync`, `owned fn`, `Mutex` and a plain `MutSpan` parameter that writes (`mut ps:
MutSpan[Player]`): that is the language today's compiler accepts, not a NEW spelling.

Each answer has the eight parts: 1 intent; 2 OLD with today's output; 3 NEW; 4 the owner's test for every
declared name; 5 means exactly (today's Ember or C doing what NEW means); 6 still refused, each with its
code and message; 7 cost; 8 kind. In the owner's-test tables "none (default: ...)" means no word in the
line answers that question and the stated default does (allowed only where the default never changes
anything, never aliases silently and never copies silently); a name counts as **without a word** when
neither column has a word. A function's own name has a row where its first line says what it changes.

## 0. What v3 changed in each answer

"Meaning changed" compares with `V2/area-E.md`; what changed from run 4 to v2 is `V2/area-E.md` section
0. The verdicts that land in area E are 22 (a task or a thread's function names the captured variables it
changes: AN-71 to AN-74), 29 (a change made under a lock or through an `Atomic` is named: AN-73, AN-75,
AN-80), 30 (starting a thread, or handing C a callback to keep, counts as making what its function
changes: AN-75, AN-76, AN-78, AN-80), 31 (every C call counts as possibly running the callbacks C keeps,
unless its declaration says `@ffi(no_callbacks)`: AN-78), 32 (the word in front of a parameter's name
sets what the function may do, for every type: AN-78, which v2 already wrote that way) and 34 (a
`MutSpan` parameter that writes says `ref mut`: AN-84). They settle the four points v2 left for the owner
(`V2/area-E.md` section 4.2): point 1 by verdict 29, point 2 by verdict 30, point 3 by verdict 31, point 4
by verdict 32 (section 2 below). Verdict 26 was done in phase 0, which also turned two English "holds"
inside messages into `exists as` and missed one `holds` in code inside a message; this step put all three
right (AN-71, AN-74, AN-78). Verdicts 21, 24, 27, 28 and 33 change no line of area E: no answer hands a
function to a library function that runs it, makes a value in an arena, tests a value's kind, walks a
tree, or uses `.copied()`; verdict 23 is cited where AN-78's `main` keeps using its game after sharing it.

| answer | what v3 changed | meaning changed? |
|---|---|---|
| AN-71 | the task says what it changes, `fn() changes total:` (verdict 22); starting it counts as making that change (verdict 30), and `total` is `count_words`'s own, so `count_words` names nothing; one refusal added (the OLD's task line, E2105); phase 0's wrong replacement in an E7015 message put right | **yes**, for one line: v2's task `fn(): total += ...` (the OLD's text) is refused (E2105); the program does what it did |
| AN-72 | the refused program's task and the help's second way say what they change (`changes total`, `changes parts`, verdict 22), so the program's one mistake is the conflict and the help's lines compile | no |
| AN-73 | the task says `changes total` (verdicts 22 and 29: a change through an `Atomic` is named, a captured variable by its name); E3026's line and the prose follow; one refusal added (E2105) | **yes**, for one line: v2's `fn() with copy of n: total.fetch_add(n)` is refused (E2105); the program does what it did |
| AN-74 | the task says `changes stats` (verdict 22); one refusal added (E2105); phase 0's wrong replacement in E7010's message put right | **yes**, for one line: v2's `fn(): stats.add_all(t)` is refused (E2105); the program does what it did |
| AN-75 | each thread's function says `changes Log` (verdict 29), and `start` and `main` say `changes Log` (verdict 30); the messages give the v3 lines; refusals added: a thread's function without `changes Log` and `start` without it (E2105), and a call through a `fn() changes Log` value inside a walk of `log.lines` under its lock (E3020, verdict 29's gain); v2's reading 1 replaced by verdict 29 | **yes**, for four lines: v2's two thread functions, `fn start(log: thread_shared Log):` and `fn main():` are refused (E2105); the program does what it did. A walk that v2 accepted, and that waited for its own lock while running, is now refused when compiling |
| AN-76 | `main` says `changes FRAME, LOG, RESULTS` (verdict 30); a `spawn` inside a loop over `RESULTS` under its lock is allowed and a join inside the lock is refused (E7018, verdict 52); rule 3 states verdict 31; v2's readings 2 and 3 replaced by verdicts 30 and 31 | **yes**, for one line: v2's `fn main() changes FRAME, LOG:` is refused (E2105); the program does what it did |
| AN-77 | the v3 line only | no |
| AN-78 | `setup` and `main` say `changes Game` (verdict 30); the NEW program walks `game.names` across a C function marked `@ffi(no_callbacks)` and calls `poll_events()` after the walk (verdict 31); refusals added: `poll_events()` inside the walk and an unmarked `sqrt` inside it (E3020), the mark on `register_cb` (E5069), `setup` and `main` without `changes Game` (E2105), `setup(...)` inside the walk (E3020); E3023's message is the rules' (verdict 32), E7015's gives the v3 lines; phase 0's missed `holds` in E7004's message put right | **yes**, for two lines: v2's `fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:` and `fn main():` are refused (E2105); and a call of a C function marked `@ffi(no_callbacks)` is allowed inside a walk of the game's lists, where v2's reading 3 refused every foreign call there; what the program does with the game is unchanged |
| AN-79 | the v3 line only | no |
| AN-80 | the thread's function says `changes Stats` (verdict 29) and `main` says `changes Stats` (verdict 30); two refusals added (E2105) | **yes**, for two lines: v2's `fn() with thread_shared st: st.note(3)` and `fn main():` are refused (E2105); the program does what it did |
| AN-81 | the v3 line only; E4030's message cited from `V3/rules.md` | no |
| AN-82 | the v3 line only | no |
| AN-83 | the v3 line only | no |
| AN-84 | `place`'s parameter says `ref mut xs: MutSpan[int]` (verdict 34), in the accepted program and in E7016's refused one; E3028's message names it; one refusal added (E3023 for the plain parameter) | **yes**, for one line: v2's `fn place(xs: MutSpan[int], idx: Span[int]):`, which writes `xs[i]`, is refused (E3023); the program does what it did |

No answer of this area opens an `Option` or a `weakref` link, keeps a parent link, or holds a link
several fields share, so `exists as`, parent links and `sharedref mut` appear only where a message names
them. No answer uses `self` outside a carried OLD or "means exactly" block: inside a method the object is
`this`, and a method says what it changes after the brackets (`changes words, tags`).

## 1. Terms of this area

Run 3's area terms, kept; every other term is `V3/rules.md` section 0's (among them **change list**,
**kept callback** and **unknown call**).

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

## 2. Area E's rules, as the v3 rules state them

Run 3's nine area rules are all in `V3/rules.md`; nothing here adds to them.

| run 3's area rule | v3 rule | note |
|---|---|---|
| 1. scoped tasks need no words; reductions | R15 (second point) | no capture words: a task uses the real variables and names on its first line the ones it changes (`fn() changes total:`, verdict 22); a local of one loop round is given to a task `with copy of n`; starting a task counts as making what its function changes (R15, first point; verdict 30), which names nothing on the caller's line when the task changes only the caller's own locals (R5) |
| 2. an unscoped thread or a kept C callback says what it holds in its capture list | R15 (third point), R3 | the capture words are `with owns`, `with thread_shared`, `with copy of`, and, on this thread only, `with shares`, `with shares mut`, `with weakref`; E7004 |
| 3. a lock guards the fields its class names (`guarded by`) | R15 (fourth point) | inside a method the object is `this`: `with this.lock:`; a function that changes a guarded field names it like any change: the field on a method's first line (`changes words, tags`), the module variable for a `static mut` (`changes RESULTS`), the object's type through a link (`changes Log`; R12, verdict 29) |
| 4. `thread_shared` | R3, R9 | spelling kept (verdict 20); E7014, E7017; what a function changes in one, under its lock, is on its first line (`fn() with thread_shared log changes Log:`, verdict 29) |
| 5. what may go to another thread is inferred from the words | R15 (fifth point) | unchanged; E7015, E7008 |
| 6. module variables, `static mut` with `changes NAME` | R5, R12, R15 (sixth point) | `static mut` kept (verdict 8). A lambda that changes a module variable says so on its first line (`fn() changes LOG:`, verdict 17); storing it changes nothing (AN-76's `setup`, G-06), and starting a thread that runs it counts (`main` names `RESULTS`, verdict 30) |
| 7. generators walk by position across `yield` | R15 (last point), R6, R8 (c) | a generator paused in a list is a walk of it: the list does not change while the generator will still be used (verdict 16, E3020) |
| 8. C callbacks through the checked trampoline | R15 (last point), R12 | the callback's first line says what it changes (`changes Game`, verdicts 17 and 29); handing it to C counts as making that change (verdict 30); every foreign call counts as possibly running it, unless the foreign function's declaration says `@ffi(no_callbacks)` (verdict 31) |
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

The words verdicts 29 to 31 add are not needed by these points: they put on first lines and in types what
a thread, a kept callback or a function value changes, so that a walk across such a change is refused
when compiling (R8 (c)): a call through a `fn() changes Log` inside a walk of the log's guarded list
(AN-75, which v2 let wait for its own lock while running), a C call inside a walk of a list a kept
callback may change (AN-78), a `spawn` inside a walk of what its thread changes (AN-76).

Freed while in use: a scoped task's views die at the join, which is before the block ends; an
unscoped thread holds only what its capture list names (an owner, a copy, or a `weakref` link, which is
counted and tested on read, R11); one-thread counts never cross threads (E7015); where scoped tasks
re-point `shares` fields of parent objects, count changes are netted per task and decreases wait until
every task has finished its loop (run 3's attack, 3.1, kept); a C callback holds a checked slot, not an
address.

**Area E's rulings** (run 3's, standing, in the v3 words): `@thread_shared` is not added once
`thread_shared` exists (AN-80); a class object held `shares` uses a one-thread count, and a value created
`shares` cannot later be made `thread_shared` ("create it `thread_shared` from the start: `names =
thread_shared Names()`", E7017); a safe `static mut` with `changes` (AN-76); `@cached` across threads:
on an object several threads reach, the slot is filled under a once-flag, and a `@cached` method whose
`reads` names a field other threads change is refused (E2117: "`radius` reads `verts`, which other
threads change under `lock`; a kept result could be stale; remove `@cached`"); the `unsafe fn` body rule
(AN-79).

**v2's four readings, now the owner's verdicts** (`V2/area-E.md` section 2 and its section 4.2):
1. **A change made under a lock or through an `Atomic` is named** (verdict 29, R12): `fn() with
   thread_shared log changes Log: log.push("a")` (AN-75) and `fn() with thread_shared st changes Stats:
   st.note(3)` (AN-80) name the type of what changes, through a link; a task that changes a captured
   `Atomic` names the variable, `fn() with copy of n changes total: total.fetch_add(n)` (AN-73). The
   call-site rule is unchanged: a `Mutex`, `RwLock`, `Atomic` or channel end takes no word at a call
   (verdict 3). The gain: a call through such a function value inside a walk of the guarded list is
   refused when compiling (E3020, AN-75).
2. **Starting a thread, or handing C a callback to keep, counts as making what its function changes**
   (verdict 30, R12, R15): the library writes `pub fn spawn[R, CHANGED](owns f: fn() changes CHANGED ->
   R) changes CHANGED -> JoinHandle[R]:`, and `scope.spawn` the same; so `main` says `changes FRAME,
   LOG, RESULTS` (AN-76), `start` and `main` say `changes Log` (AN-75), `setup` and `main` say `changes
   Game` (AN-78), and a function whose tasks change only its own locals names nothing (AN-71 to AN-74).
3. **A foreign call counts as possibly running every callback C keeps**, and makes the changes their
   types list, unless the foreign function's declaration says `@ffi(no_callbacks)` (verdict 31, R12):
   AN-78's walk of `game.names` across `sqrt` (marked) is allowed, across `poll_events()` refused (E3020).
4. **A plain `game: shares mut Game` parameter only reads the game** (verdict 32, R4), so `setup` takes
   `owns game: shares mut Game`, called `setup(shares mut game)`, with `with owns game`, as v2 wrote it.

**One reading this file relies on, which `V3/rules.md` does not state in words** (section 4.2, point 1,
for the owner): **a `guarded by` field is its lock's value, changed under the lock through any name that
reaches its object, as a `Mutex`'s value is.** AN-74 calls the lock and its fields "`Mutex<Inner>`
spelled as a lock field"; R5 lets a `Mutex` be used through any name by its own operations, but R4 and R5 list only
`Mutex`, `RwLock`, `Atomic` and channel ends. AN-75 relies on it three times: `start`'s plain `log:
thread_shared Log` (the vocabulary's own example of a `thread_shared` parameter) makes `with thread_shared
log` owners that change `lines` under the lock, which R3's "an owner made from another may do no more than
it" allows only if `log` may do the same; `start`'s read-only `log` makes no promise about `lines`, which
is read and changed only under its lock (R8 (g)); and E7013's way out, `log.push("c")` in `start`. What a
function changes there is still on its first line (verdict 29).

## 3. The answers of area E

### AN-71: threads (A15): the loop variable, `total +=` from tasks, E7004 offering `@sync`

**v3:** the task says what it changes, `fn() changes total:`: a plain `fn()` now changes nothing at all,
captured variables included (verdict 22). Starting the task counts as making that change (verdict 30),
and `total` is `count_words`'s own local, so `count_words`'s first line names nothing (R5). **Meaning
changed:** yes, for one line: v2's task, `scope.spawn(fn(): total += t.split_whitespace().count())`,
which was the OLD's text, is refused (E2105, part 6); the program does what it did: every task adds its
part to the real total.

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

**3. NEW** (v2's, with the task's `changes total`; no method, no call to a `ref mut` or `MutSpan`
parameter):

```ember
fn count_words(texts: Array[String]) -> int:
    total = 0
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn() changes total: total += t.split_whitespace().count())
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

`count_words` is the OLD text with one addition: under R15 it compiles (`total +=` in a task is a
reduction). The task uses the real `total` (row 6), and says on its first line that it changes it,
`changes total` (R12, R15, verdict 22); `scope.spawn` counts as making that change (R12, verdict 30), and
a change of a variable that a lambda written in `count_words` captured is `count_words`'s own, which needs
no clause (R5). The task reads `t` and needs no word for it.
`share_plain` hands the object to the thread; `share_and_keep` creates it shared between threads and
keeps reading it. A capture `with thread_shared x` of a plain local is refused (E7017, below).

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `texts` (1) | none (default: a plain parameter reads the caller's list, which does not change while `texts` is used, R4) | none (default: read-only, R5) |
| `total` (2) | none (default: a new local, the function's own) | none (default: the function's own variable; `+=` is the change, written) |
| `scope` (3) | `with` (bound for this block, ended at its end) | none (default: read-only binding) |
| `t` (4) | none (default: a plain loop name, a read-only view of each element of `texts`, R2) | none (default: read-only, R5) |
| `total` (5, in the task) | none (default, row 6: a name not in the capture list is the real variable, R12) | `changes total` (the task's first line: the real `total` changes, verdict 22; `+=` written) |
| `t` (5, in the task) | none (default: the same view of the element of `texts`, which lives until the join) | none (default: read-only) |
| `items` (9) | none (default: the object's own list) | none (default: changed only by a method whose first line says `changes items` or `changes this`, R5) |
| `names` (12) | none (default: a new local, the function's own object) | none (default: its own) |
| `h` (13) | none (default: the `JoinHandle` the call returns, the caller's own) | none |
| `names` (13, capture) | `owns` (handed to the thread; the caller's `names` ends here) | `owns` (the thread's own now) |
| `names` (17) | `thread_shared` (a new object with owners on any thread) | `thread_shared` (no owner changes it directly, R9) |
| `h` (18) | none (default as line 13) | none |
| `names` (18, capture) | `thread_shared` (one more owner, on any thread) | `thread_shared` (no owner changes it directly) |
| `names` (19) | `thread_shared` (line 17) | `thread_shared` (line 17: read-only) |

14 names; 8 without a word (`texts`, `total` (2), `t` (4), `t` (5), `items`, `names` (12), `h` (13),
`h` (18)), each with its default stated. No default changes anything the line does not show, aliases
silently, or copies silently: the per-task accumulator behind line 5 is a compile-time translation no
line can observe (5).

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

- The task without its word, the OLD's line (v2's NEW): `scope.spawn(fn(): total +=
  t.split_whitespace().count())`: `error[E2105]: this function changes \`total\`, a variable it uses
  from \`count_words\`; say so on its first line: \`scope.spawn(fn() changes total: total +=
  t.split_whitespace().count())\`` (R12, verdict 22; the rules' E2105 row, in this program's names).
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
  caller another owner of it, `error[E7015]: \`names\` holds \`cache\`, which has other owners on this
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

**v3:** the refused program's task says what it changes, `changes total`, and so does the task in the
help's second way, `changes parts` (verdict 22): without the word the program would have a second
mistake (E2105) beside the one this answer is about, and the help's lines would not compile under v3
(R18). **Meaning changed:** no: the same program is refused, at the same name, for the same reason.

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
            scope.spawn(fn() changes total: total = total * 2 + t.len())
    return total
```

```text
error[E7007]: two tasks change `total`, and `total = total * 2 + t.len()` uses its old value
  --> count.em:5:45
   |
 5 |             scope.spawn(fn() changes total: total = total * 2 + t.len())
   |                                             ^^^^^ every task of this scope runs this line at the same time as the others
   |
   = help: to add each task's part, write `total += t.len()`; to combine the parts in order, write `parts: Array[int] = []` before the `with` block, `scope.spawn(fn() changes parts: parts.push(t.len()))` as the task, and `for p in parts: total = total * 2 + p` after the `with` block
```

The rule: a task's conflict is reported once, on the captured name that causes it; errors on other
names of the same function that only follow from it are not shown, and no error ever names a name the
program uses correctly (`t` here). The help's second way gives the same result as the program run on
one thread: `push` in a task is a reduction combined at the join in spawn order (R15), which is the
order of `texts`. (Run 3's help said only "have each task keep its own (`parts.push(t.len())`)", which
does not compile until `parts` is declared; R18 asks the help's lines to compile in the program.) The
help's task says `changes parts`, since a plain `fn()` changes nothing at all (verdict 22). A program
that also leaves out the task's `changes total` has two mistakes and gets two messages: E2105 on the
task's first line (AN-71's), then this one.

**4. The owner's test:** `texts` (1), `total` (2), `t` (4), `t` (5): none (defaults as in AN-71's table,
the same lines); `total` (5): none for what it is (the real variable, row 6), `changes total` for whether
it may change it (the task's first line); `scope` (3): `with`. 6 names, 4 without a word. In the help's
lines, `parts` is a new local, `count_words`'s own (none: its own; `push` is the change, written, and the
task that pushes says `changes parts`), and `p` is a plain loop name (none: a read-only view of each
number, which as an `int` is copied, R2).

**5. Means exactly.** The program is refused; the line its help gives first (`total += t.len()`) is
AN-71's program, whose translation is `means/an71_means.em` (`exit=0`, pasted in AN-71).

**6. Still refused:** the NEW program itself, with the message above.

**7. Cost:** none; a compile-time message.

**8. Kind:** a smarter compile-time solution (the error found at its cause).

---

### AN-73: "cannot find `n`" (E1010) inside a lambda that calls an `Atomic` method

**v3:** the task says what it changes, `changes total`: a change made through an `Atomic` is named like
every change (verdict 29), and a captured variable is named by its name (verdict 22). Starting the task
counts as making that change (verdict 30); `total` is `count`'s own local, so `count`'s first line names
nothing (R5). **Meaning changed:** yes, for one line: v2's task, `scope.spawn(fn() with copy of n:
total.fetch_add(n))`, is refused (E2105, part 6); the program does what it did.

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

**3. NEW** (v2's, with the task's `changes total`):

```ember
fn count(texts: Array[String]) -> int:
    total = Atomic(0)
    with scope = thread.scope():
        for t in texts:
            n = t.split_whitespace().count()
            scope.spawn(fn() with copy of n changes total: total.fetch_add(n))
    return total.load()
```

The defect is fixed (a name used as an argument of a method call inside a lambda is looked up like any
other), and the task names what it copies, row 6's words (R15: "a local of one loop round is given to a
task with `with copy of n`"). `t` needs no word: it is a view of an element of `texts`, which lives
until the join; `n` is the round's own number, which does not. `total` is the real variable (row 6),
changed by `fetch_add`, the `Atomic`'s own operation, which any name may use (R5); the task says so after
its capture list, `changes total` (R12: a lambda names the real variables it changes, a change through an
`Atomic` included; verdicts 22 and 29). Two tasks changing `total` this way is not E7007: the atomic
instruction orders each change (R5, R9).

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `texts` (1) | none (default: a plain parameter reads the caller's list) | none (default: read-only) |
| `total` (2) | none (default: a new local, the function's own) | none (default: an `Atomic` changes only through its own methods, R5) |
| `scope` (3) | `with` | none (default: read-only binding) |
| `t` (4) | none (default: a read-only view of each element) | none |
| `n` (5) | none (default: a new local of this round, its own) | none (default: its own) |
| `n` (6, capture) | `copy of` (the task's own copy, row 6) | `copy of` (the task's own) |
| `total` (6, in the task) | none (default, row 6: the real `total`) | `changes total` (the task's first line, verdicts 22 and 29), through `fetch_add`, the `Atomic`'s own operation, written |

7 names, 4 without a word (`texts`, `total` (2), `t`, `n` (5)).

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
- The task without the copy word, once the lookup is fixed: `scope.spawn(fn() changes total:
  total.fetch_add(n))`: `error[E3026]: the task uses \`n\`, which ends at the end of this round of the
  loop (line 5), while the task may still be running; give the task its own copy: \`fn() with copy of n
  changes total:\``. This is E3026, "a function value that may outlive a place it uses and does not own"
  (R12); run 4 gave it E3060, whose meaning since v2 is a view used after its target changed.
- The task without `changes total`, v2's line: `scope.spawn(fn() with copy of n: total.fetch_add(n))`:
  `error[E2105]: this function changes \`total\` through \`fetch_add\`, a variable it uses from
  \`count\`; say so on its first line: \`scope.spawn(fn() with copy of n changes total:
  total.fetch_add(n))\`` (R12, verdicts 22 and 29).

**7. Cost:** one atomic add per task, as the program asks (an `Atomic` is the programmer's choice;
`total += n` on a plain `int` would be AN-71's reduction at plain C's cost). The copy of `n` is one
8-byte store into the task's state.

**8. Kind:** a smarter compile-time solution (a defect fixed, and the real cause named with row 6's
words).

---

### AN-74: a `@sync` class freezes every field after `init`; objects written by several tasks need a lock or a claim

**v3:** the task says what it changes, `fn() changes stats:` (verdict 22): it changes the real `stats`
through `add_all`. Starting it counts as making that change (verdict 30), and `stats` is `count`'s own
local, so `count`'s first line names nothing (R5). Phase 0 had turned an English "holds" in E7010's
message into `exists as`; it reads "holds" again. **Meaning changed:** yes, for one line: v2's task,
`scope.spawn(fn(): stats.add_all(t))`, is refused (E2105, part 6); the program does what it did.

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
            scope.spawn(fn() changes stats: stats.add_all(t))
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
  with no mark (`stats.add_all(t)`, R4). The task changes `stats`, a real variable it captures, through
  `add_all`, so its first line says so, `fn() changes stats:` (R12, verdict 22); the tasks change it
  only under its lock, inside `add_all`, so two of them changing it is not E7007 (R15). Starting a task
  counts as making that change (verdict 30), a change of `count`'s own local, which needs no clause (R5).
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
| `stats` (19, task) | none (default, row 6: the real `stats`) | `changes stats` (the task's first line, verdict 22), through `add_all`'s `changes words, tags`, under the lock |
| `t` (19, task) | none (default: the same view, alive until the join) | none |
| `players` (27) | `ref mut` (the caller's list) | `ref mut` (may change it) |
| `p` (29) | `ref mut` (each element of the caller's list, alone while used, R8 (f)) | `ref mut` |

14 names, 6 without a word (`lock`, `w`, `texts`, `stats` (16), `t` (18), `t` (19)).

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
- The task without its word, v2's line: `scope.spawn(fn(): stats.add_all(t))`: `error[E2105]: this
  function changes \`stats\`, a variable it uses from \`count\`, through \`add_all\` (\`changes words,
  tags\`); say so on its first line: \`scope.spawn(fn() changes stats: stats.add_all(t))\`` (R12,
  verdict 22).
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
  `players: Array[shares mut Player]`: `error[E7010]: \`players\` holds \`shares mut Player\` objects,
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

**v3:** each thread's function says what it changes, `changes Log`: a change made under a lock is named
like every change, by the type of what changes (verdict 29; v2's reading 1, which named nothing, is
withdrawn). Starting a thread counts as making what its function changes (verdict 30), so `start` says
`changes Log`, and so does `main`, which calls `start`. **Meaning changed:** yes, for four lines: v2's
`fn() with thread_shared log: log.push("a")` (and its `"b"` twin), `fn start(log: thread_shared Log):`
and `fn main():` are refused (E2105, part 6); the program does what it did. And a call through a
`fn() changes Log` value inside a walk of `log.lines` under the log's lock, which v2 accepted and which
waited for its own lock while running, is now refused when compiling (E3020, part 6: verdict 29's gain).

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

fn start(log: thread_shared Log) changes Log:
    a = thread.spawn(fn() with thread_shared log changes Log: log.push("a"))
    b = thread.spawn(fn() with thread_shared log changes Log: log.push("b"))
    a.join()
    b.join()

fn main() changes Log:
    start(thread_shared Log())
```

The capture word does what Rust's `Arc::clone` line does, on the name it is about: each thread gets
one more owner. No second name (`log2`) and no separate clone line. `log: thread_shared Log` reads the
caller's value with no count (R4); `with thread_shared log` makes one more owner from it (R3, allowed
because the value is already `thread_shared`). Each thread changes the log only through `push`, whose
first line says `changes lines`, a field `guarded by lock` changed inside `with this.lock:` (R9: the
one way a `thread_shared` value is changed). What the threads' functions change is on their first lines,
`changes Log`: they change a `Log` through a `thread_shared` link, under its lock, and such a change is
named by the type of what changes (R12, verdict 29), exactly the verdict's own line. `thread.spawn`
counts as making what its function changes (R12, verdict 30; the library's first line is `pub fn
spawn[R, CHANGED](owns f: fn() changes CHANGED -> R) changes CHANGED -> JoinHandle[R]:`), so `start`
says `changes Log`, and `main`, which calls `start`, says it too (R5). Nothing more is checked while
running: the words are checked when compiling. `start`'s read-only `log` promises that what it reads
does not change while it is used (R4, R8 (g)); `lines` is read and changed only under its lock, as a
`Mutex`'s value is, which is the reading section 2 states (`guarded by` as the lock's value).

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `lock` (4) | none (default: the object's own lock) | none (default: changes only by being held) |
| `lines` (5) | none (default: the object's own list) | `guarded by lock` |
| `this` (7, not listed: the method's object, R5) | `this` (the `Log` `push` was called on) | `changes lines` (row 3) |
| `s` (7) | `StrView` (a view of someone else's text) | none (default: read-only) |
| `start` (11) | `fn` | `changes Log` (it starts threads whose functions change a `Log`, R5, R12, verdict 30) |
| `log` (11) | `thread_shared` (in its type: a log shared between threads; a plain parameter, so a view of the caller's, no count, R4) | `thread_shared` (no owner changes it directly; `push` changes a guarded field, under its lock) |
| `a` (12) | none (default: the `JoinHandle` the call returns, the caller's own) | none |
| `log` (12, capture) | `thread_shared` (one more owner) | `thread_shared`, and the function's `changes Log` (verdict 29) |
| `b` (13) | none (default as `a`) | none |
| `log` (13, capture) | `thread_shared` | `thread_shared`, and `changes Log` |
| `main` (17) | `fn` | `changes Log` (it calls `start`, R5) |

11 names, 3 without a word (`lock`, `a`, `b`).

**5. Means exactly** (`means/an75_means.em`, which is phase 1's `a34_clone_dance.em` itself): a
`@sync` class (atomic counts), its list inside a `Mutex`, and `owned fn` retaining the handle once per
thread. `exit=0`.

**6. Still refused:**
- A thread's function that does not say what it changes, v2's line: `fn() with thread_shared log:
  log.push("a")`: `error[E2105]: this function changes \`log\` through \`push\` (\`changes lines\`, under
  \`log\`'s lock); say so on its first line: \`fn() with thread_shared log changes Log:
  log.push("a")\`` (R12, verdict 29; the rules' E2105 row).
- `start` without the word, v2's line `fn start(log: thread_shared Log):`: `error[E2105]: \`start\` starts
  threads whose functions change \`Log\` (\`fn() with thread_shared log changes Log: log.push("a")\`,
  line 12); add it to \`start\`'s first line: \`fn start(log: thread_shared Log) changes Log:\`` (R5,
  R12, verdict 30); and `main` without it: `error[E2105]: \`main\` calls \`start\`, which changes
  \`Log\` (line 11); add it to \`main\`'s first line: \`fn main() changes Log:\``.
- A call through a function value that may change the log, inside a walk of the log's list under its
  lock (verdict 29's gain): in `fn dump(log: thread_shared Log, note: fn() changes Log) changes Log:`
  (line 1), `with log.lock:` (line 2), `for line in log.lines:` (line 3), `println(line)` (line 4),
  `note()` (line 5), called as `dump(log, fn() with thread_shared log changes Log: log.push("seen"))`:
  `error[E3020]: this loop walks \`log.lines\` (line 3) under \`log\`'s lock, and \`note()\` (line 5)
  may change any \`Log\`, this one included (its type says \`changes Log\`), so it would change the list
  during the walk and wait for the lock this thread holds; call \`note()\` after the \`with log.lock:\`
  block` (R8 (c), R12; the rules' E3020 row). v2 accepted it (its `note: fn()` named nothing), and the
  program waited for ever at line 5.
- A thread that takes the log over: `with owns log`: `error[E3013]: \`log\` is lent to \`start\`
  (a plain parameter), so the thread cannot take it over (\`with owns log\`, line 12); to give each
  thread one more owner of the log, write \`fn() with thread_shared log changes Log: log.push("a")\``.
  (Run 4 gave this E3040, "handed to the first thread"; under R3 and E3013 a name that does not own the
  value cannot hand it over at all, so the refusal comes at the first capture.)
- A one-thread owner sent to a thread: `with shares log`: `error[E7015]: \`with shares log\` makes an
  owner of \`log\` that stays on one thread, and this function runs on a new one; write \`fn() with
  thread_shared log changes Log: log.push("a")\``.
- No capture word: `thread.spawn(fn() changes Log: log.push("a"))`: `error[E7004]: this thread may
  outlive \`start\`, so it cannot use \`start\`'s \`log\`; say what the thread holds: \`fn() with
  thread_shared log changes Log: log.push("a")\` (one more owner of the log)` (AN-71's E7004, in this
  program's names; `with owns log` is not offered, since `start` does not own the log).
- `with thread_shared log` where `log` is a plain parameter `log: Log` (a view of an object whose
  owner may be one-thread), with `start(Log())` in `main`: `error[E7017]: \`log\` is lent to \`start\`,
  and its owner may not be shared between threads; take it as \`fn start(log: thread_shared Log) changes
  Log:\`, and create the log shared in \`main\`: \`start(thread_shared Log())\``.
- `log.lines.push(String("c"))` in `start`, outside the lock: `error[E7013]: \`lines\` is guarded by
  \`lock\`; change it inside \`with log.lock:\`, or call \`log.push("c")\``.

`with log.lock:` around `note()` with no walk, where `note` pushes to the same log, would wait for ever
while running (`push` takes the lock its own thread holds; AN-74's static re-entry covers only calls
whose body the compiler sees). It is refused when compiling (R15, verdict 43), E7018: "`note()` may change any `Log`, this one included (its type says `changes Log`), and this thread holds `log`'s lock, so taking it again inside `note` would wait for ever; call `note()` after the `with log.lock:` block".
C and Rust hang there (Rust's `Mutex::lock` does not return when the thread already holds it).

**7. Cost:** one atomic increment per `with thread_shared` capture and one atomic decrement when the
thread's owner ends: exactly Rust's `Arc::clone` and drop, and what today's `owned fn` does for a
`@sync` handle. The lock is C's. `changes Log` on the functions' first lines costs nothing while running:
it is checked when compiling.

**8. Kind:** a readable spelling of Rust's guarantee (`Arc::clone` + `move`, written as one word on the
captured name).

---

### AN-76: global mutable state (A19)

**v3:** starting a thread counts as making what its function changes, whenever that happens (verdict
30: "who said a change needs to be bound by a moment?"): `main` starts threads running `fn() changes
RESULTS: worker(1)`, so it says `fn main() changes FRAME, LOG, RESULTS:`, the verdict's own line. A
`spawn` inside a loop over `RESULTS` under `RESULTS_LOCK` is allowed, the thread waiting for the lock, and
joining it inside the lock is refused (E7018; verdict 52). A foreign call counts as possibly running the
callbacks C keeps, unless marked `@ffi(no_callbacks)` (rule 3 below; verdict 31). **Meaning changed:**
yes, for one line: v2's `fn main() changes FRAME, LOG:` is refused (E2105, part 6); the program does what
it did. What stays from v2: every lambda that changes a module variable says so on its first line, and
the list that keeps it says so in its type (verdict 17): `b.on_click(fn() changes LOG:
LOG.push(String("clicked")))`, kept in `handlers: Array[fn() changes LOG]`, run by `fn click() changes
handlers, LOG:`; a loop over `LOG` across `b.click()` is refused because `click`'s first line says
`changes LOG`, and the same loop is accepted when `b`'s handlers are plain `fn()`, which change nothing.
And `setup`, which only hands the lambda to `on_click` to keep, names nothing (G-06): storing a function
is not a change; running it, or handing it to a thread or to C to run, is (R12).

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

fn main() changes FRAME, LOG, RESULTS:
    tick()
    a = thread.spawn(fn() changes RESULTS: worker(1))
    b = thread.spawn(fn() changes RESULTS: worker(2))
    a.join()
    b.join()
```

and the button of verdict 17, whose handlers push to the log:

```ember
type Handler = fn() changes LOG

class Button:
    handlers: Array[Handler] = []

    fn on_click(owns f: Handler) changes handlers:
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
   methods it calls change: `main` calls `tick()`, so `main` names `FRAME, LOG`; `report` calls
   `b.click()`, so `fn report(ref mut b: Button) changes LOG:` (the change to `b`'s handlers goes
   through its `ref mut` parameter and needs no clause). **Starting a thread counts as making what its
   function changes, now or later** (R5, R12, R15; verdict 30): `thread.spawn` takes `owns f: fn()
   changes CHANGED -> R` and says `changes CHANGED`, so `main`, which starts threads running `fn()
   changes RESULTS: worker(1)`, says `fn main() changes FRAME, LOG, RESULTS:`. That says what a call of
   `main` causes to change; rule 4 and E7012 still keep the threads apart. A lambda says what it
   changes on its own first line (R12): `fn() changes RESULTS: worker(1)`, `fn() changes LOG:
   LOG.push(...)`. **Storing a lambda is not changing what it changes**: `setup` hands the handler to
   `on_click`, which keeps it for `click` to run, so `setup` names nothing (R12: the line falls at who
   runs the function; G-06).
3. One thread: a loop over `LOG`, or any view into it, across a call that may change `LOG` is refused
   (R8 (c), (g); the agreed limitation: change after the loop). A call may change `LOG` when its
   callee's first line says so, and a call through a function value when the value's type says so
   (R12): `b.click()` changes `LOG` because `click` runs handlers of type `fn() changes LOG` and its
   first line says `changes handlers, LOG`. A list of plain `fn()` handlers changes nothing, so
   clicking such a button inside a loop over `LOG` is allowed. A `spawn` inside a loop over what its
   thread changes under the loop's lock is allowed, and joining that thread inside the lock is refused
   (E7018; verdict 52). A foreign call
   counts as possibly running every callback C keeps, and makes the changes their types list, unless
   the foreign function's declaration says `@ffi(no_callbacks)` (R12, verdict 31; the attack's 5.1,
   `y16`; AN-78).
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
| `main` (16) | `fn` | `changes FRAME, LOG, RESULTS` (it calls `tick()`, and starts threads whose functions change `RESULTS`, R5, R12, verdict 30) |
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
- `main` without `RESULTS`, v2's line `fn main() changes FRAME, LOG:`: `error[E2105]: \`main\` starts a
  thread whose function changes \`RESULTS\` (\`fn() changes RESULTS: worker(1)\`, line 18); add it to
  \`main\`'s first line: \`fn main() changes FRAME, LOG, RESULTS:\`` (R5, R12, verdict 30; the rules'
  E2105 row).
- A thread started inside a walk of what it changes, then joined inside the lock: after line 21,
  `with RESULTS_LOCK:` (line 22), `for r in RESULTS:` (line 23), `c = thread.spawn(fn() changes RESULTS:
  worker(r))` (line 24) and `c.join()` (line 25): `error[E7018]: \`c.join()\` (line 25) waits for the
  thread started on line 24, whose function changes \`RESULTS\` under \`RESULTS_LOCK\`, and this thread
  holds that lock, so each would wait for the other for ever; join after the \`with RESULTS_LOCK:\` block`
  (R15, verdict 52). Without line 25 the program is accepted: the new thread waits for the lock until the
  walk ends, as Rust allows (`probes/rust_spawn_in_loop.txt`: rustc compiles both; the joining one, run,
  was still waiting after 5 s).
- The handler written as a plain `fn()`: `b.on_click(fn(): LOG.push(String("clicked")))` (line 12):
  `error[E2105]: this function pushes to \`LOG\`; say so on its first line: \`fn() changes LOG:
  LOG.push(String("clicked"))\``.
- A handler that changes `LOG` kept in a list of plain functions: `handlers: Array[fn()]` and
  `fn on_click(owns f: fn()) changes handlers:` with line 12 as written: `error[E2113]: this function
  changes \`LOG\` (\`fn() changes LOG\`), and \`on_click\` takes \`fn()\`, functions that change
  nothing; keep it in a list that says so: name the type once, \`type Handler = fn() changes LOG\`,
  and write \`handlers: Array[Handler]\` and \`fn on_click(owns f: Handler) changes handlers:\``.
- `click` without `LOG`: `fn click() changes handlers:`: `error[E2105]: \`click\` runs the handlers in
  \`this.handlers\`, whose type says \`changes LOG\`; add it to its first line: \`fn click() changes
  handlers, LOG:\``.
- A loop over `LOG` across a call that runs those handlers (run 3's `20-re-attacker.md`, F-04's
  program, in this block's names: `b.click()` inside `report`'s loop, line 16, instead of after it):
  `error[E3020]: this loop walks \`LOG\` (line 15), and \`b.click()\` (line 16) changes \`LOG\`: \`click\`
  runs the handlers of \`b\`, an \`Array[fn() changes LOG]\` (its first line says \`changes handlers,
  LOG\`); click after the loop` (R8 (c), R12; `V3/rules.md` section 3's message, in this program's
  lines). The program that message leads to (line 17's order) is
  `RUN3/21-fixer/c/f4_fnvalue_static_loop.c` mode 2: clean under AddressSanitizer (run 3's proof, not
  run again).

Accepted (since v2): the same loop over `LOG` with `b.click()` inside it, when `b`'s handlers are plain
`fn()` (`handlers: Array[fn()]`, `fn click() changes handlers:`): a plain `fn()` changes nothing at all
(verdicts 17 and 22), no function in the list may change `LOG` (E2113 keeps such a function out), so the
walk is safe.

**7. Cost:** plain C globals: nothing counted, nothing checked while running. Last run measured the
same work (`phase3b-threads.md:854-863`, `tick()` called 100,000,000 times, PROVISIONAL): plain C
globals 192.219 ms (1.00); today's Ember workaround (`Atomic` + `Mutex`) 3.10x on one thread, 7.15x
once a second thread has started; safe Rust's `AtomicU64` + `Mutex` 7.44x; Rust's `thread_local!`
1.04x. The guarded list costs the lock C takes.

**8. Kind:** a smarter compile-time solution (Rust needs `unsafe` or atomics for this; here the
function's first line says what it changes and the compiler proves only one thread group reaches it).

---

### AN-77: async and generators (A14)

**v3:** no verdict of 21 to 34 changes a line of this answer: its generators change nothing outside
themselves and start no thread. **Meaning changed:** no. What v2 settled stays: run 4 asked the owner
whether a generator kept in a variable may see its list change; verdict 16 ("a, same rule for every
loop") answered it the way run 4 had read it: a generator paused in the middle of a list is a loop over
that list (R6, R8 (c)). What that settles, as run 4 had it:
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
  1), and \`g\` is used again on line 3; push after \`g\`'s last use` (R8 (c), verdict 16; the
  message of `V3/rules.md` section 3, in this program's lines; no "walk by position" way out, since a
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

**v3:** handing C a callback to keep counts as making what the callback changes, now and whenever C runs
it (verdict 30): `setup` says `changes Game`, and so does `main`, which calls `setup`. Every C call
counts as possibly running the callbacks C keeps, unless the C function's declaration says it never calls
back (verdict 31): the NEW program now shows it, with a walk of the game's `names` across `sqrt`, marked
`@ffi(no_callbacks)` (allowed), and `poll_events()`, which runs the callback, after the walk (inside it,
refused, E3020). What a parameter may do is set by the word in front of its name (verdict 32): v2's `owns
game: shares mut Game`, called `setup(shares mut game)`, is that rule's own example. **Meaning changed:**
yes, for two lines: v2's `fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:` and `fn
main():` are refused (E2105, part 6); and a C function marked `@ffi(no_callbacks)` may be called inside a
walk of the game's lists, where v2's reading 3 counted every foreign call as running the callback. What
the program does with the game is unchanged: C keeps one more owner of the game that may change it, and
each callback adds a point.

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

    fn add_point() changes score:
        this.score += 1

unsafe extern "C":
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)
    safe fn poll_events()
    @ffi(no_callbacks)
    safe fn sqrt(x: f64) -> f64

fn setup(owns game: shares mut Game) changes Game -> Retained[ForeignCallback]:
    return register_cb(fn() with owns game changes Game: game.add_point())

fn main() changes Game:
    game = shares mut Game(names=[String("ann"), String("bo")])
    token = setup(shares mut game)
    for name in game.names:
        println(name, sqrt(name.len() as f64))
    poll_events()
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
- **Who may change the game** (R3, R4, R9). `main`'s `game` is the first owner and may change it, and
  it keeps what it could do after sharing it (verdict 23). `setup(shares mut game)` makes one more owner
  that may change it and hands it to `setup`'s `owns` parameter (R4: the caller writes `shares mut game`
  for an `owns x: shares mut T` parameter); `with owns game` hands that owner on to the callback. What a
  function may do with a parameter is set by the word in front of its name, for every type (R4, verdict
  32): a plain parameter `game: shares mut Game` would be a read-only view of `main`'s owner (the
  `shares mut` after the colon says what `main`'s game is, not what `setup` may do), and an owner made
  from it may do no more than it, so `with shares mut game` from it is refused (E3023, below); that is
  why `setup` takes an owner.
- **What the callback changes** (R12, verdicts 17 and 29). It changes the game through an owner it
  shares with `main`, a link other names reach, so its first line names the type: `changes Game`
  (checked against its body, E2105: `add_point` says `changes score`).
- **Handing it to C counts** (R5, R12, verdict 30). `register_cb`'s parameter says
  `callback(retained)`: C keeps the function and runs it during later foreign calls, so the call that
  hands it over counts as making what it changes, now and later. `setup` says `changes Game`, and so
  does `main`, which calls `setup`. (Storing a function for its caller to run is not a change; handing it
  to something that runs it on its own, a thread or C, is.)
- **Every foreign call may run it** (R12, R15, verdict 31). The compiler cannot see inside C, so a call
  of a foreign function counts as possibly running every callback C keeps, on the threads their
  contracts allow (here the creator thread), and makes the changes their types list: `poll_events()` on
  line 24 makes `changes Game`, so no view of a game's part and no walk of a game's list lives across it
  (R8). That is why it is called after the walk of `game.names`. `sqrt` says on its declaration that it
  never calls back into Ember, `@ffi(no_callbacks)` (line 13), so its call inside the walk runs no
  callback and is allowed. The mark is a fact the programmer vouches for, listed by `ember tcb` with the
  other asserted facts (`[TCB-1]`); on a function whose own contract takes a callback it is refused
  (E5069). The callbacks' changes are on the first lines of the functions that hand them to C (`setup`,
  and `main` through it) and of the functions that make an unmarked foreign call while they are kept
  (`main`, which calls `poll_events()`, says `changes Game` already; a `fn tick(): poll_events()` would
  say `fn tick() changes Game:`), and the messages name the mark (R12; verdict 39).

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `score` (2) | none (default: the object's own number) | none (default: changed only by a method whose first line says `changes score` or `changes this`) |
| `names` (3) | none (default: the object's own list) | none (default as `score`) |
| `this` (5, not listed: the method's object, R5) | `this` (the `Game` `add_point` was called on) | `changes score` (row 3) |
| `cb` (9-11) | `extern "C" fn` (a C function pointer, its type) | none (default: a parameter only reads) |
| `ud` (9-11) | `*mut void` (C's raw pointer; the contract names it user data) | none (default: Ember never reads it here) |
| `x` (14) | none (default: a plain parameter reads the caller's value; an `f64` is copied in, R4) | none (default: read-only) |
| `setup` (16) | `fn` | `changes Game` (it hands C a callback that changes `Game`, R12, verdict 30) |
| `game` (16) | `owns` (one owner of the game, handed to `setup` by the call) | `shares mut` in its type: an owner that may change the game (R4, R9) |
| `game` (17, capture) | `owns` (`setup`'s owner, handed to the callback C keeps; `setup`'s `game` ends here) | `shares mut` (its type), and the callback's `changes Game` (R12) |
| `main` (19) | `fn` | `changes Game` (it calls `setup`, R5) |
| `game` (20) | `shares mut` (a new object with several owners; this is the first) | `shares mut` (this owner may change it, and keeps that after line 21, verdict 23) |
| `token` (21) | none (default: the `Retained` token the call returns, the caller's own; dropping it unregisters) | none |
| `name` (22) | none (default: a read-only view of each name, R2) | none (default: read-only; `game.names` does not change while the loop walks it, R8 (c)) |

13 names, 5 without a word (`score`, `names`, `x`, `token`, `name`), each with its default stated.

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

That program is the callback part of NEW. The walk of `game.names` and the two C calls v3 adds have no
translation run here: today's Ember has no `@ffi(no_callbacks)` (E0104, the rule writer's
`r4_no_callbacks.em`, `V3/changes.md` section 7), and they do what they say: print each name with the
square root of its length, then let C deliver its events, each of which runs the callback and adds a
point.

**6. Still refused:**
- The walk with `poll_events()` inside it (line 24 moved into the loop's body, after line 23):
  `error[E3020]: this loop walks \`game.names\` (line 22), and \`poll_events()\` (line 24) is a C
  function, which may run the callback C keeps from \`register_cb\` (line 17), and that callback changes
  \`Game\` (\`fn() with owns game changes Game\`), so it may change \`game.names\`; call
  \`poll_events()\` before or after the loop; if \`poll_events\` never calls back into Ember, say so on
  its declaration: \`@ffi(no_callbacks) safe fn poll_events()\`` (R8 (c), R12, verdict 31; the rules'
  E3020 row, in this program's lines). With a kept callback that changes nothing
  (`register_cb(fn(): println("event"))`) the same walk is allowed.
- `sqrt` declared without the mark (line 13 removed, so the lines after it move up by one):
  `error[E3020]: this loop walks \`game.names\` (line 21), and \`sqrt(name.len() as f64)\` (line 22) is
  a C function, which may run the callback C keeps from \`register_cb\` (line 16), and that callback
  changes \`Game\`; if \`sqrt\` never calls back into Ember, say so on its declaration:
  \`@ffi(no_callbacks) safe fn sqrt(x: f64) -> f64\`; otherwise call it before or after the loop` (the
  rules' E3020 row: a missing mark refuses only such a walk, and the message names the mark).
- The mark on a function whose own contract takes a callback, `@ffi(no_callbacks, param(cb,
  callback(retained, user_data=ud), threads(creator)), destroys(unregister))` on line 10:
  `error[E5069]: \`register_cb\` takes a callback (\`param(cb, callback(retained, user_data=ud))\`), so
  it may call back into Ember; remove \`no_callbacks\` from its \`@ffi(...)\`` (R12).
- `setup` without the word, v2's line: `fn setup(owns game: shares mut Game) ->
  Retained[ForeignCallback]:`: `error[E2105]: \`setup\` hands \`register_cb\` a callback that C keeps
  and that changes \`Game\`; add it to \`setup\`'s first line: \`fn setup(owns game: shares mut Game)
  changes Game -> Retained[ForeignCallback]:\`` (R5, R12, verdict 30; the rules' E2105 row); and `main`
  without it, v2's `fn main():`: `error[E2105]: \`main\` calls \`setup\`, which changes \`Game\` (line
  16: it hands C a callback that changes \`Game\`); add it to \`main\`'s first line: \`fn main() changes
  Game:\``.
- The callback handed to C inside the walk (`token = setup(shares mut game)` moved from line 21 into the
  loop's body, so the loop is on line 21 and the call on line 22): `error[E3020]: this loop walks
  \`game.names\` (line 21), and \`setup(shares mut game)\` (line 22) changes \`Game\` (its first line
  says \`changes Game\`: it hands C a callback that changes \`Game\`); call it before the loop: write
  \`token = setup(shares mut game)\` above the \`for\` line` (R8 (c), R12, verdict 30).
- No capture word for a kept callback: `register_cb(fn() changes Game: game.add_point())`:
  `error[E7004]: C keeps this function after \`setup\` returns (\`callback(retained)\`), so it cannot
  use \`setup\`'s \`game\`; say what it holds: \`fn() with owns game changes Game:\` (\`setup\`'s owner
  of the game), or \`fn() with weakref game changes Game:\` with \`if game exists as ref mut g:
  g.add_point()\` as its body (a link that lets the game end)`.
- The callback without `changes Game`: `register_cb(fn() with owns game: game.add_point())`:
  `error[E2105]: this function changes \`game\`, a \`Game\` other owners share, through \`add_point\`
  (\`changes score\`); say so on its first line: \`fn() with owns game changes Game:\`` (R12).
- A changing owner made from a read-only view: `fn setup(game: shares mut Game) changes Game ->
  Retained[ForeignCallback]:` with `with shares mut game`: `error[E3023]: \`game\` only reads the
  \`Game\` that \`main\` lends to \`setup\` (\`shares mut Game\` says what \`main\`'s game is, not what
  \`setup\` may do with it), so an owner made from it only reads too; to hand on an owner that may
  change the game, take one: write \`fn setup(owns game: shares mut Game) changes Game ->
  Retained[ForeignCallback]:\`, call it \`setup(shares mut game)\`, and capture it \`with owns game\``
  (R4, verdict 32; the rules' E3023 row).
- The game kept by a `threads(any)` callback (line 10 with `threads(any)`): `error[E7015]: C may call
  this function from any thread (\`threads(any)\`), and \`shares\` owners stay on one thread; share the
  game between threads: write \`game = thread_shared Game(names=[String("ann"), String("bo")])\` and
  \`token = setup(game)\` in \`main\`, \`fn setup(game: thread_shared Game) changes Game ->
  Retained[ForeignCallback]:\` and \`return register_cb(fn() with thread_shared game changes Game:
  game.add_point())\` in \`setup\`, and, since \`add_point\` changes \`score\`, \`score: Atomic[i32] =
  Atomic(0)\`, \`this.score.fetch_add(1)\` as \`add_point\`'s body, and \`println(game.score.load())\``
  (R9, R15; the rules' E7015 row, completed with every line it replaces, R18). `add_point` keeps its
  first line, `fn add_point() changes score:`: a change through an `Atomic` is named like every change
  (verdict 29), and an `Atomic` is changed through any name by its own operations (R5).
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
`phase3b-threads.md:1137`) checks none of these facts. `changes Game` on the first lines, the
foreign-call rule and `@ffi(no_callbacks)` cost nothing while running: they are checked when compiling,
and `sqrt` is called as C calls it.

**8. Kind:** a new mechanism (the checked trampoline), safer than Rust's: a stale, forged or
wrong-thread call is caught, where Rust's workaround is an unbounded `unsafe` reference. The walk across
a C call is a smarter compile-time solution: the kept callback's type says what a C call may change,
and `@ffi(no_callbacks)` exempts a C function that never calls back, a fact vouched for and listed.

---

### AN-79: `unsafe` points two ways, and an `unsafe fn` body is unchecked with no mark per operation

**v3:** no verdict of 21 to 34 changes a line of this answer. **Meaning changed:** no.

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

**v3:** the thread's function says what it changes, `changes Stats`: a change made under a lock is named
like every change, by the type of what changes (verdict 29; v2's reading 1, which named nothing, is
withdrawn). Starting the thread counts as making that change (verdict 30), so `main` says `changes
Stats`. **Meaning changed:** yes, for two lines: v2's `fn() with thread_shared st: st.note(3)` and `fn
main():` are refused (E2105, part 6); the program does what it did: `longest` ends as 5 whichever
thread runs first.

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

fn main() changes Stats:
    st = thread_shared Stats()
    a = thread.spawn(fn() with thread_shared st changes Stats: st.note(3))
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
thread's function changes `st` through `note`, under its lock, so its first line says `changes Stats`
(R12, verdict 29: a change through a `thread_shared` link is named by the type of what changes).
`thread.spawn` counts as making what its function changes (R12, verdict 30), so `main` says `changes
Stats`; `main`'s own `st.note(5)`, a change of its own local, needs no clause (R5). Nothing more runs:
the words are checked when compiling.

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
| `main` (19) | `fn` | `changes Stats` (it starts a thread whose function changes a `Stats`, R5, R12, verdict 30) |
| `st` (20) | `thread_shared` (one of several owners, on any thread) | `thread_shared` (changed only through `note`, under the lock) |
| `a` (21) | none (default: the `JoinHandle` the call returns, the caller's own) | none |
| `st` (21, capture) | `thread_shared` (one more owner) | `thread_shared`, and the function's `changes Stats` (verdict 29) |

12 names, 6 without a word (`lock`, `n`, `xs`, `s`, `x`, `a`).

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
- The thread's function without the word, v2's line: `thread.spawn(fn() with thread_shared st:
  st.note(3))`: `error[E2105]: this function changes \`st\` through \`note\` (\`changes words,
  longest\`, under \`st\`'s lock); say so on its first line: \`fn() with thread_shared st changes Stats:
  st.note(3)\`` (R12, verdict 29); and `main` without it, v2's `fn main():`: `error[E2105]: \`main\`
  starts a thread whose function changes \`Stats\` (\`fn() with thread_shared st changes Stats:
  st.note(3)\`, line 21); add it to \`main\`'s first line: \`fn main() changes Stats:\`` (R5, R12,
  verdict 30).
- `@sync class Stats`: `error[E0104]: \`@sync\` is gone: a class needs no mark to be shared between
  threads; share the object where it is shared, \`st = thread_shared Stats()\`, and say which lock
  guards each field that changes (\`words: int guarded by lock\`) or make it \`Atomic[int]\``.
- `T: Sync` in a bound: `error[E1010]: \`Sync\` is the effect \`@nosync\` forbids, not a property of
  a type; for "threads may share it" write \`T: ThreadShared\``.
- `st.longest = n` outside the lock: `error[E7013]: \`longest\` is guarded by \`lock\`; change it
  inside \`with st.lock:\`` (AN-74's message, in this program's names).
- `@nosync` reaching a lock: `E4002`, today's, which names the chain (`[EFF-6]`).

**7. Cost:** none from the words, `changes Stats` included (checked when compiling); the lock is C's
(AN-74).

**8. Kind:** a readable spelling of Rust's guarantee (Rust's `Sync`, inferred, named `ThreadShared`;
the class marker replaced by words on the value and the fields).

---

### AN-81: `@static_safe` forbids only aliasing checks

**v3:** no verdict of 21 to 34 changes a line of this answer; E4030's message is cited from `V3/rules.md`.
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
  (\`if i != j:\` around the call), or remove \`@no_alias_checks\`` (`V3/rules.md` section 3, E4030
  row). The run-time alias checks left in the model are R4's index compare and `assert_disjoint`.

**7. Cost:** none; the contract is checked when compiling.

**8. Kind:** a readable spelling of Rust's guarantee (a rename: the contract keeps its meaning and is
named for it).

---

### AN-82: `@no_runtime_checks` is reserved, promising a switch that will never exist

**v3:** no verdict of 21 to 34 changes a line of this answer. **Meaning changed:** no.

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

**v3:** no verdict of 21 to 34 changes a line of this answer. **Meaning changed:** no.

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

**v3:** `place`'s `MutSpan` parameter, which writes the caller's numbers, says `ref mut` in front of
its name, like every name that may change something it does not own (verdict 34; R5's exception by type
is withdrawn), in the accepted program and in E7016's refused one. The call already said `ref mut`
(verdict 3). **Meaning changed:** yes, for one line: v2's `fn place(xs: MutSpan[int], idx:
Span[int]):`, which writes `xs[i]`, is refused (E3023, part 6); the program does what it did.

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

The programs the new codes' messages lead the programmer to, in NEW spelling (run 4's, with two
changes: the argument given to `place`'s `MutSpan` parameter says `ref mut` at the call, verdict 3, and
the parameter says `ref mut` in front of its name, verdict 34; R4):

```ember
import c "./variadic.c" with (overlay = "overlays/variadic.em") as v
from std.ffi import c_int
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
without it, E3028: "`place` changes the elements of `a` (its first line says `ref mut xs:
MutSpan[int]`); say so at the call: `place(ref mut a.as_mut_span(), order.as_span())`"). The parameter
says the same words, `ref mut xs: MutSpan[int]` (R4, verdict 34): a plain `xs: MutSpan[int]` only reads
the caller's numbers, and `xs[i] = idx[i]` through it is refused (E3023, part 6). The two arguments are
views of two different lists, `a` and `order`, so R4's "arguments do not overlap a changeable one" is
proved when compiling, with nothing checked while running, and `xs` is compiled with noalias (R4).

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `v` (1) | `import ... as v`: the C module's name | none (default: a module's name changes nothing) |
| `text` (5) | `*u8` (its type): a raw pointer to bytes C owns | `*u8` without `mut`: reads only |
| `xs` (8) | `ref mut` (the caller's numbers, through its `MutSpan`: the only name for them while `place` runs, R8 (f)) | `ref mut` (it may write them, R4, verdict 34); the caller says `ref mut` too (line 27) |
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

**6. Still refused:** everything refused today stays refused, under the codes above. And, by verdict 34,
v2's plain parameter: `fn place(xs: MutSpan[int], idx: Span[int]):` with `xs[i] = idx[i]` (line 11):
`error[E3023]: \`xs[i] = idx[i]\` writes an element of the caller's span, but \`xs\` only reads it;
write \`fn place(ref mut xs: MutSpan[int], idx: Span[int]):\`; the call already says
\`place(ref mut a.as_mut_span(), order.as_span())\`` (R4, R5; the rules' E3023 row, in this program's
names).

**7. Cost:** none.

**8. Kind:** a readable spelling of Rust's guarantee (the same refusals; each code now names one
thing, as Rust's codes do).

## 4. Words needed, and for the owner

### 4.1 Words needed

None. Every word this area's NEW blocks and messages use has a row in `V3/vocabulary.md`. Checked by
`target/redesign4/v3-06-area-E/scripts/check_words.py` (output in
`target/redesign4/v3-06-area-E/check_words.txt`): it lists every name in the NEW code blocks (part 3 of
each answer), in the inline code of parts 3 and 6 and in the code inside their quoted messages that does
not occur inside a `` `...` `` span of the vocabulary. The 66 names it lists are the programs' own names
(`count_words`, `Names`, `Button`, `Game`, `poll_events`, `sqrt`, `dump`, `HITS`, `texts`, `token`,
`report`, ...), error codes, file and test names carried from run 3 (`f4_fnvalue_static_loop`, `y14`,
`x05`), plain words of the trampoline's description (`slot`, `generation`, `0x4080`), `global` (the
Python spelling a message refuses, AN-76) and Rust's lint name `unsafe_op_in_unsafe_fn` (AN-79), none of
which is a word of an Ember program. The words v3 adds to this area are all rows of the vocabulary:
`changes` with a captured variable (`changes total`, `changes stats`), with a type (`changes Log`,
`changes Stats`, `changes Game`) and on `main` (`changes FRAME, LOG, RESULTS`); `@ffi(no_callbacks)` (row
`@ffi`); `ref mut` on a `MutSpan` parameter (rows `ref mut`, `MutSpan`); `as f64` (rows `as`, `f64`).

### 4.2 For the owner

v2's four points (`V2/area-E.md` section 4.2) are answered by verdicts 29 to 32 and applied (section 2).
Three new points, one at a time, each with what this file does meanwhile.

1. **Answered (verdict 44, the owner: "Keep guarded by and the same way as mutex"): yes; `guarded by`
   stays and follows the `Mutex`'s rules, so AN-75's `fn start(log: thread_shared Log) changes Log:`
   stands.** The question as it was put: **may a `guarded by` field be changed under its lock through a
   plain name, as a `Mutex`'s value is?** (AN-75; section 2.) R4 says "Only a `Mutex`, `RwLock`, `Atomic` or channel end may be changed
   through a plain name, by its own operations", and does not name a `guarded by` field, which is this
   area's readable spelling of a `Mutex` around the fields (AN-74). AN-75's `fn start(log: thread_shared
   Log) changes Log:`, the vocabulary's own example of a `thread_shared` parameter, gives each thread
   `with thread_shared log`, an owner that changes `lines` under the lock. If a plain name may not do
   that, R3 ("an owner made from another may do no more than it") refuses the capture, and `start` must
   take an owner, `fn start(owns log: thread_shared Log) changes Log:`, called `start(thread_shared
   Log())`; E7013's way out "or call `log.push("c")`" in `start` would be refused too. Meanwhile this
   file reads a `guarded by` field as its lock's value, one rule for `Mutex`, `Lock` and `guarded by`
   (verdict 3: their "only me" is the lock's block), and keeps v2's `start`. Either costs nothing while
   running.
2. **Answered (verdict 43, the owner: "B"): refused when compiling, E7018.** The question as it was
   put: **verdict 29's gain covers a walk, not every call made while a lock is held.** `with log.lock: for
   line in log.lines: note()`, where `note` is a `fn() changes Log`, is refused when compiling (E3020,
   AN-75). `with log.lock: note()`, with no walk, where `note` pushes to the same log, is accepted and
   waits for ever while running: `push` takes the lock its own thread holds, and AN-74's static re-entry
   covers only calls whose body the compiler sees. That is a hang, never a memory error, and C and Rust
   do the same (Rust's `Mutex::lock` does not return when the thread already holds the lock). The
   choices: (a) leave it, as C and Rust do (this file); (b) refuse a call through a function value whose
   type says `changes Log` while this thread holds a `Log`'s lock, which also refuses a function that
   changes another log; (c) keep in each lock the thread that holds it and stop the program, naming the
   function, instead of hanging: one store when the lock is taken and one compare before, not measured.
3. **Answered (verdict 52, the owner: "B"): allowed under the loop's lock, as Rust allows it; the join
   inside the lock is refused instead (E7018).** The note as it was put: **verdict 30's refusal of a
   `spawn` inside a walk of what its thread changes guards nothing that
   memory safety needs in this area's programs.** A list another thread may change is reached only under
   its lock (`guarded by`, `Mutex`, E7012, E7014), and the walk holds that lock, so the new thread's
   change waits until the walk ends (AN-76's `RESULTS`); a scoped task's change to a list its parent
   walks is already refused by E7007 (nothing reads a task's place before the join, R15). No question:
   this file applies the verdict as written ("start the thread after the loop" moves one line), and notes
   it so that the owner can judge whether it is an understandable limitation or an annoyance. For the
   rule writer: R15 does not say which of E3020 and E7007 a scoped task that changes the list its parent
   walks gets (R18: one mistake, one message).
