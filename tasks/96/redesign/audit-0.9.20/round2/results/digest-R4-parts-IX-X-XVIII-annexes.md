# R4-parts-IX-X-XVIII-annexes

could_not_cover: None of the nine clusters touches verdicts 164 to 166 (a `sharedref mut` link crossing threads, `thread_shared` refused at the making line for a non-`ThreadShared` content, a scoped task making or ending a `shares` owner); K54's program uses a `static mut` under a `static` `Lock` and OS threads, none of those three cases, and K14's `thread_shared Stats()` has only an `int` field, so verdict 165's check does not fire. I read the current rules, the round-1 record (`round1/applied.md`, `G1.json`, `G4.json` for the `String using` decision), `decisions.md` §12 and §13 (N54, N58, N74, N78) and the verdicts 19, 24, 47, 107, 145, 152, 157, 159, 162, 163; I did not re-read every Nondet mention of Part XI beyond `[THR-21]` and the job-system line, nor the whole of Annex B beyond HR-1 to HR-3b, HR-12 to HR-16, HR-34 to HR-37 and HR-43 to HR-48, so a rule elsewhere that already classifies the `runtime` test (K58) would make that cluster "answered"; a grep for `runtime` across Part X and the registry found none.

## K34-which-types-take-using  reader=answered_differently  refuter=narrowed  should_be=answered_differently  safety=none

**answered_by**: TYP-46: "`using A` after a type whose value has heap memory of its own names the kind of memory manager that memory comes from, a type implementing `Allocator` (`[ALC-2]`): `mesh: owns Mesh using LevelArena`, `mesh: shares Mesh using LevelArena`, `names: Array[String] using LevelArena`. It may follow an `owns`, `shares` or `shares mut` type, a class type, or an `Array`, a `Map`, a `Set` or a `String` (`name: String using LevelArena = String("") using current`); after any other type, a `thread_shared` type included (`[OBJ-4]`), it is `E2114` ("an `int` has no memory of its own to take from `LevelArena`; remove `using`")."; ALC-1: "`using A` after a type whose value has heap memory of its own (an `owns`, `shares` or `shares mut` value, a class object, an `Array`, `Map`, `Set` or `String`; never a `thread_shared` value, whose block carries its making thread's record, `[OBJ-4]`, `E2114`) names the kind of memory manager that memory comes from"; ALC-1: "a value that grows (an `Array`, `Map`, `Set` or `String`) made in it keeps the word, since its growth goes back to the instance"; registry row E2114: "`using` after a type with no heap memory of its own"; STD-7: "Fixed-capacity containers with no heap allocation: `FixedArray[T, N]`, `FixedString[N]` and `RingBuffer[T, N]`; ... Also `Deque[T]`, `SmallArray[T, N]` (inline up to `N`, then heap), `BitSet`."; std.collections layer table (Part XII): "layer: **alloc**, but `FixedArray`, `FixedString`, `RingBuffer`, `BitSet`, `Pool`, `Handle` and `SmallArray`'s inline part, which are **core**"

**program**: 

- READER EDIT rule=TYP-46
  - old: It may follow an `owns`,
  `shares` or `shares mut` type, a class type, or an `Array`, a `Map`, a `Set` or a `String` (`name:
  String using LevelArena = String("") using current`); after any other type, a `thread_shared` type
  included (`[OBJ-4]`), it is `E2114`
  - new: It may follow an `owns`,
  `shares` or `shares mut` type, a class type, or a collection of `std.collections` whose elements
  live on the heap: an `Array`, a `Map`, a `Set`, a `String` (`name: String using LevelArena =
  String("") using current`), a `Deque`, and a `SmallArray`, whose elements beyond `N` do; after any
  other type, a type with no heap memory of its own (an `int`, a struct held inline, a `FixedArray`,
  a `RingBuffer`, a `BitSet`) and a `thread_shared` type included (`[OBJ-4]`), it is `E2114`
  - why: The rule opens with the property (heap memory of its own) and the registry row states the same property, yet the list refuses `Deque[int] using A` with a message ("has no memory of its own") that is untrue for a `Deque`. The owner's rule 7 wants the property, not a list; the collections with no heap memory stay refused by the property itself.

- READER EDIT rule=ALC-1
  - old: (an `owns`,
  `shares` or `shares mut` value, a class object, an `Array`, `Map`, `Set` or `String`; never a
  `thread_shared` value
  - new: (an `owns`,
  `shares` or `shares mut` value, a class object, a collection whose elements live on the heap: an
  `Array`, `Map`, `Set`, `String`, `Deque`, or a `SmallArray` beyond its `N`; never a
  `thread_shared` value
  - why: The parallel list in `[ALC-1]` says the same as `[TYP-46]`; the two are kept word for word.

- READER EDIT rule=ALC-1
  - old: a
  value that grows (an `Array`, `Map`, `Set` or `String`) made in it keeps the word, since its
  growth goes back to the instance
  - new: a
  value that grows (an `Array`, `Map`, `Set`, `String`, `Deque` or `SmallArray`) made in it keeps
  the word, since its growth goes back to the instance
  - why: Every collection `using` may follow is one that grows; the exception of verdict 107 (b) must name them all.

**remaining_doubt**: The current text decides every case: `String using A` compiles (the finder's J5-09 and the checker's option A for String), and `Deque`, `SmallArray`, `BitSet` and `RingBuffer` are `E2114` by "after any other type" (J2-02's option B, not its recommendation A). That answer is safe (it only refuses). The defect left is one of consistency: the E2114 words and the registry row claim "no heap memory of its own", which is false for a `Deque` or a `SmallArray`'s spill. The alternative to the edits above is to keep the closed list and reword the row and the message to "a type not among those `[TYP-46]` lists"; that keeps a list a reader must memorise, so the property is recommended. The lead's round-1 ruling (G1-13, G4-14) added only `String`, without a stated reason to leave `Deque` out, so this is not an owner verdict (verdicts 19, 24, 47, 107 name `Array`, `String`, lists and values that grow only). A `Deque.new_in` or `[..] using current` making line for a `Deque` is library detail (`Array.new_in(current)` is the model).

**REFUTER reason**: The status stands: the text decides every case and refuses safely. `[TYP-46]`: "after any other type, a `thread_shared` type included (`[OBJ-4]`), it is `E2114` (\"an `int` has no memory of its own to take from `LevelArena`; remove `using`\")", so `Deque[int] using A` and `SmallArray[int, 4] using A` are refused. The defect is one of consistency, as the reader says: the rule opens with the property, "`using A` after a type whose value has heap memory of its own", the registry row says "`using` after a type with no heap memory of its own", and the message says "has no memory of its own", yet `[STD-7]` gives `Deque[T]` and `SmallArray[T, N]` ("inline up to `N`, then heap") heap memory, and the Part XII layer table puts `Deque` in **alloc**. No verdict fixes the list (19, 24 and 47 name `Array`, `String` and lists; 107 (b) speaks of "a value that grows"), and round 1's G1-13/G4-14 (lead, not owner) added `String` by the same property argument. The reader's edits are at the cause and open no hole: a `Deque` or `SmallArray` made in an instance is bound to it by `[ALC-1]`'s "a value made in an instance, and every value that holds one, lives no longer than that instance (`E3063`)", and the third edit keeps verdict 107 (b)'s word for every value that grows; no new cost row is needed, since the `[COST-3]` row "A value made `using` a memory manager" already prices the word. The edits are narrowed in one point: they state the property ("a collection ... whose elements live on the heap") and then give a list that omits `SoA`, which `[SOA-1]` makes "a growable container holding one column per field of `T`", heap storage of its own; a property followed by a list that falls short of it is the shape this cluster complains of. The better text names the property as the deciding clause and lists what it covers in `std`, `SoA` included, and names the refused shapes by the property too.

- BETTER EDIT rule=TYP-46
  - old: It may follow an `owns`,
  `shares` or `shares mut` type, a class type, or an `Array`, a `Map`, a `Set` or a `String` (`name:
  String using LevelArena = String("") using current`); after any other type, a `thread_shared` type
  included (`[OBJ-4]`), it is `E2114`
  - new: It may follow an `owns`,
  `shares` or `shares mut` type, a class type, or a standard collection whose storage is on the heap:
  an `Array`, a `Map`, a `Set`, a `String` (`name: String using LevelArena = String("") using
  current`), a `Deque`, an `SoA` (`[SOA-1]`), and a `SmallArray`, whose elements beyond `N` are
  (`[STD-7]`); after any other type, one with no heap memory of its own (an `int`, a struct held
  inline, a `FixedArray`, a `RingBuffer`, a `BitSet`, an arena container, `[ARN-5]`) and a
  `thread_shared` type included (`[OBJ-4]`), it is `E2114`
  - why: The deciding clause is the property the rule opens with; the list names every standard type the property covers, so the message "has no memory of its own" is true of everything refused.

- BETTER EDIT rule=ALC-1
  - old: (an `owns`,
  `shares` or `shares mut` value, a class object, an `Array`, `Map`, `Set` or `String`; never a
  `thread_shared` value
  - new: (an `owns`,
  `shares` or `shares mut` value, a class object, a standard collection whose storage is on the heap:
  an `Array`, `Map`, `Set`, `String`, `Deque`, `SoA`, or a `SmallArray` beyond its `N`; never a
  `thread_shared` value
  - why: The parallel list says the same as `[TYP-46]`.

- BETTER EDIT rule=ALC-1
  - old: a
  value that grows (an `Array`, `Map`, `Set` or `String`) made in it keeps the word, since its
  growth goes back to the instance
  - new: a
  value that grows (an `Array`, `Map`, `Set`, `String`, `Deque`, `SoA` or `SmallArray`) made in it
  keeps the word, since its growth goes back to the instance
  - why: Verdict 107 (b): "A value that grows cannot be made in it this way" without the word; every collection `using` may follow grows, so the exception names them all.

## K54-nondet-contested-lock  reader=answered_differently  refuter=confirmed  should_be=answered_differently  safety=none

**answered_by**: DET-2: "a read whose value depends on which thread acted first: a receive on a channel more than one thread may send to (one sender on one thread gives the sent order), `try_lock`'s and `try_recv`'s answer, and a lock's view or an `Atomic` read of a place another thread, or a task not yet joined, may change (`[THR-11]`; after the join the place holds what the joined tasks left, in spawn order for a reduction, `[THR-21]`), `E4070` naming the senders"; DET-2: "A reduction of scoped tasks, combined in spawn order, is not `Nondet` either (`[THR-21]`)."; DET-1: "`@deterministic` on a function or module is a contract: `Nondet` MUST NOT appear in its effect set (`E4070`, naming the operation and the chain)."; THR-20: "a function that may run on another thread (reachable from `thread.spawn`, from a job submitted outside a scope, from a callback C may run on a thread other than the main one, or from an `@unsafe(export("name", threads=any))` or `@unsafe(export("name", threads=creator))` function)"; EFF-1: "An operation's effects are its own and those of everything it runs where it runs: the methods and function values it calls"; EFF-2: "A parameter of a function type is the function given, per instance, as a type parameter bounded by a function type is (`[COST-3]`): the package that gives the function compiles that instance and sees the function."; EFF-26: "A call that runs a function by itself counts as making everything that function changes, now or later: `thread.spawn`, `scope.spawn`, `jobs.submit` and a job scope's `submit` (Part XI)"; THR-21: "A field of a `thread_shared` value is not a reduction place: its owners never change it directly (`E7014`, `[THR-1]`)"

**program**: 

**remaining_doubt**: The current text names the operation: not `spawn` or `join`, but "a lock's view ... of a place another thread ... may change" (and a receive on a multi-sender channel, `try_lock`, an `Atomic` read). In the finder's program `with ORDER_LOCK: ORDER.push(owns 1)` in `push_one` is a lock's view of `ORDER`, a place `push_two`'s thread may change (both are reachable from `thread.spawn`, `[THR-20]`'s test for "may run on another thread"), so `push_one` and `push_two` are `Nondet`; `race` reaches them through `thread.spawn(push_one)` (`[EFF-2]`: the function given is the instance's function) and is `E4070`, the chain `race -> thread.spawn -> push_one -> with ORDER_LOCK`. Its own `with ORDER_LOCK: return copy of ORDER` is a lock's view too; the text tracks joins only for scoped tasks (`[THR-11]`), so for OS threads "another thread may change" stands and that line is `Nondet` as well; either way `race` is refused, which is the safe answer. This is the finder's option A refined by the "another thread may change" condition (a lock one thread alone takes is not `Nondet`), not its recommendation B (the spawn as the line); `[THR-21]`'s reductions are untouched. The only loose end is cosmetic: the registry row for `E4070` lists "a receive on a channel several threads send to, `try_lock`, an `Atomic` read of a place an unjoined task may change" and omits the lock's view, the case `[DET-2]` itself names; "thread and job completion order" still stands in the list beside the concrete clause that gives it an operation.

**REFUTER reason**: The text decides the program, and refuses it. `[DET-2]`: `Nondet` is introduced by "a read whose value depends on which thread acted first: ... a lock's view or an `Atomic` read of a place another thread, or a task not yet joined, may change (`[THR-11]`; after the join the place holds what the joined tasks left ...)". `race`'s `with ORDER_LOCK: return copy of ORDER` is a lock's view (`[THR-18]`: "`guarded by lock` ... names the `Lock` that must be held for every read and change of that field") of a place `push_one` and `push_two` change, and both "may run on another thread" by `[THR-20]`'s test ("reachable from `thread.spawn`"); the join exception is written for scoped tasks only (`[THR-11]`), and `[THR-10]` gives `h.join()` of an OS thread no such clause, so for the compiler "another thread ... may change" stands and the read is `Nondet`; `[DET-1]`: "`Nondet` MUST NOT appear in its effect set (`E4070`, naming the operation and the chain)". Whether `push_one`'s own `ORDER.push(owns 1)` is itself `Nondet` (a change, not "a read whose value depends") does not change the outcome: `race` is refused at its own line at least, which is the safe answer, and `[THR-21]`'s reductions ("combined in spawn order, is not `Nondet`") are untouched. The reader's reading of the operation (the lock's view, not the spawn) is the one the text gives. One consistency gap remains, which the reader called cosmetic and left unedited: the registry row for `E4070` lists "a receive on a channel several threads send to, `try_lock`, an `Atomic` read of a place an unjoined task may change" and omits the lock's view and `try_recv`, both named by `[DET-2]`; it is listed under missed with its edit.

## K58-runtime-check-effect-kind  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=none

**answered_by**: EFF-9: "`Aliasing` covers the compare of `assert_disjoint` and `check_disjoint` (`[DSJ-1]`), the one run-time alias check there is"; EFF-12: "it forbids only `RuntimeCheck(Aliasing)`, the run-time compare of `assert_disjoint` or `check_disjoint`, because a data-oriented inner loop can always do without it: spans the compiler proves apart need no compare, and a copy needs none"; DSJ-6: "They are the one run-time alias check there is, so inside `@no_alias_checks` each is `E4030`"; EFF-16: "`@nopanic(explicit)` forbids only the panics the programmer writes and allocation failure. Arithmetic, bounds, aliasing and stale checks, and a count's compare with its limit, are `RuntimeCheck` kinds and are allowed under it ... A function that is `@nopanic(explicit)` **and** free of `RuntimeCheck` — two facts `ember inspect --safety` reports together — has no panic the effect model knows of"; effects table, Part X.1: "| `Panic(Explicit)` | `panic`, `assert*` (`assert_disjoint` included), `unwrap`, `expect`, `todo`, `unreachable`, `cast x as T`, allocation failure |"; LT-48: "at each later line that changes a place the result may point into for one of those kinds, while the result will still be used, tests the fact's bit for that place and stops the program when it is set ... stops at `xs.push(owns 3)`: "panic at main.em:17:5: `xs` changes while `r` looks at it: `p` is a `Yours`, whose `pick` gives back the list it was handed""; COST-3 row: "| A `runtime` name's check | guaranteed present | one load of the fact from the called method's entry in the value's kind's table at the call ... and one bit test at each line the result guards"; registry row E4030: "`assert_disjoint`, the one alias check left to run time, inside `@no_alias_checks`"

**program**: Implementer question the text cannot answer: does this compile, and what does `ember inspect --safety` report for `present`?

```
interface Renderer:
    fn draw(scratch: Array[u8]) -> Span[u8]

struct Gpu:
    pixels: Array[u8]

extend Gpu implements Renderer:
    fn draw(scratch: Array[u8]) -> Span[u8]:
        return this.pixels

struct Software:
    unused: int

extend Software implements Renderer:
    fn draw(scratch: Array[u8]) -> Span[u8]:
        return scratch

@nopanic(explicit)
@no_alias_checks
fn present(r: runtime dyn Renderer, ref mut scratch: Array[u8]):
    frame = r.draw(scratch)
    scratch.push(owns 1)          # tested while running; stops for a Software
    println(frame.len())

fn main():
    scratch = [0, 0]
    r: dyn Renderer = Gpu(pixels=[1, 2]) if input() == "gpu" else Software(unused=0)
    present(r, ref mut scratch)
```

By the letter of `[EFF-9]`, `[EFF-12]` and `[DSJ-6]` the `runtime` test is not `RuntimeCheck(Aliasing)` (`assert_disjoint`/`check_disjoint` are "the one run-time alias check there is"), it is not in the `Panic(Explicit)` row, and `[COST-6]` has no row for its stop; so the text accepts `present` under both contracts, `ember inspect --safety` reports it free of `RuntimeCheck`, and `[EFF-16]`'s promise ("has no panic the effect model knows of") is broken when a `Software` reaches it and the program stops at line 21. `@no_alias_checks` then admits a function whose only run-time check is an alias check, against its own words in `[EFF-12]`.

- READER EDIT rule=EFF-9
  - old: `Aliasing` covers the compare of
  `assert_disjoint` and `check_disjoint` (`[DSJ-1]`), the one run-time alias check there is; `Stale`
  - new: `Aliasing` covers the compare of
  `assert_disjoint` and `check_disjoint` (`[DSJ-1]`) and the test of a `runtime` name's fact at a
  line that changes what the call's result may point into (`[LT-48]`), the two run-time alias
  checks there are; `Stale`
  - why: The `runtime` test is a check the compiler emits where its proof fails and that guards a view against a change: an alias check by what it tests. Naming it here gives it an effect kind, which no rule does today.

- READER EDIT rule=EFF-12
  - old: it forbids only `RuntimeCheck(Aliasing)`, the run-time compare of `assert_disjoint`
  or `check_disjoint`, because a data-oriented inner loop can always do without it: spans the
  compiler proves apart need no compare, and a copy needs none (`E4030`: "`mix` is
  - new: it forbids only `RuntimeCheck(Aliasing)`, the run-time compare of `assert_disjoint`
  or `check_disjoint` and the test of a `runtime` name (`[LT-48]`), because a data-oriented inner
  loop can always do without it: spans the compiler proves apart need no compare, a copy needs none,
  and a kind known when compiling is checked when compiling (`E4030` at the word: "`present` is
  `@no_alias_checks`, and `runtime` on `r` (line 19) tests while running whether `scratch` changes
  while `frame` looks at it; give `present` a kind known when compiling, `fn present(r: Gpu, ref
  mut scratch: Array[u8]):`, or remove `@no_alias_checks`"; `E4030`: "`mix` is
  - why: `@no_alias_checks` means no alias check runs in the function; the message names the line to write, as the attribute's existing message does.

- READER EDIT rule=DSJ-6
  - old: They are the one run-time alias check there is, so inside `@no_alias_checks`
  each is `E4030`
  - new: They are, with a `runtime` name's test (`[LT-48]`), the run-time alias checks there
  are, so inside `@no_alias_checks` each is `E4030`
  - why: The sentence "the one run-time alias check there is" became false when verdict 152 added the `runtime` test; `[DSJ-6]` must claim what `[EFF-9]` says.

- READER EDIT rule=effects table, Part X.1, row Panic(Explicit)
  - old: | `Panic(Explicit)` | `panic`, `assert*` (`assert_disjoint` included), `unwrap`, `expect`, `todo`, `unreachable`, `cast x as T`, allocation failure |
  - new: | `Panic(Explicit)` | `panic`, `assert*` (`assert_disjoint` included), `unwrap`, `expect`, `todo`, `unreachable`, `cast x as T`, the test a `runtime` name asks for (`[LT-48]`), allocation failure |
  - why: `runtime` is a word the programmer writes to ask for a stop (verdict 152: "a way to check it at runtime but with a word"), as `cast x as T` and `assert_disjoint` are; the same thing gets the same classification, and `[EFF-16]`'s promise holds for a function free of `Panic(Explicit)` and `RuntimeCheck`.

- READER EDIT rule=EFF-16
  - old: Arithmetic, bounds, aliasing and stale checks, and a
  count's compare with its limit, are `RuntimeCheck` kinds and are allowed under it;
  - new: Arithmetic, bounds, aliasing and stale checks, and a
  count's compare with its limit, are `RuntimeCheck` kinds and are allowed under it, except the two
  a written word asks for, `assert_disjoint` and a `runtime` name's test, which are also panics the
  programmer writes (`E4040`: "`present` is `@nopanic(explicit)`, and `runtime` on `r` (line 19) may
  stop the program at `scratch.push(owns 1)` (line 21); give `present` a kind known when compiling,
  or remove `@nopanic(explicit)`");
  - why: Without this the sentence "aliasing checks ... are allowed under it" contradicts the `Panic(Explicit)` row for `assert_disjoint` today and for `runtime` after the edit; the message names the line to write.

- READER EDIT rule=LT-48
  - old: it costs
  one load at the call and one bit test at each line it guards, and nothing where no line needs them
  (§X.4).
  - new: it costs
  one load at the call and one bit test at each line it guards, and nothing where no line needs them
  (§X.4). Where a test is emitted it enters the function's effect set as `RuntimeCheck(Aliasing)`
  and, since the word asks for the stop, as `Panic(Explicit)` (`[EFF-9]`, `[EFF-16]`), exactly as
  `assert_disjoint` does: inside `@no_alias_checks` the word is `E4030` (`[EFF-12]`) and inside
  `@nopanic(explicit)` it is `E4040` (`[EFF-16]`), each naming the kind known when compiling that
  needs no test; a `runtime` name whose every kind is known when compiling, where nothing is tested,
  carries neither, and `ember inspect --safety` lists each test with its line (`[EFF-10]`).
  - why: The rule that defines the test says what it costs and must say what it carries; the defining rule is where an implementer looks.

- READER EDIT rule=COST-6
  - old:   | `assert_disjoint` (`[DSJ-1]`) | "`assert_disjoint(d, s)` on line 11: the two spans overlap (`dst_name` and `src_name` name one track)" |
  - new:   | `assert_disjoint` (`[DSJ-1]`) | "`assert_disjoint(d, s)` on line 11: the two spans overlap (`dst_name` and `src_name` name one track)" |
  | a `runtime` name's test (`[LT-48]`) | "`xs` changes while `r` looks at it: `p` is a `Yours`, whose `pick` gives back the list it was handed" |
  - why: `[COST-6]` promises to list every cost of X.4 that may stop the program; the `runtime` row of `[COST-3]` has no entry, though `[LT-48]` gives the stop's approved text (verdict 157).

- READER EDIT rule=registry row E4030
  - old: | `E4030` | `assert_disjoint`, the one alias check left to run time, inside `@no_alias_checks` | `[DSJ-6]`, `[EFF-12]` |
  - new: | `E4030` | `assert_disjoint`, `check_disjoint` or a `runtime` name, the alias checks left to run time, inside `@no_alias_checks` | `[DSJ-6]`, `[EFF-12]`, `[LT-48]` |
  - why: The row must claim what the rules say.

- READER EDIT rule=registry row E4040
  - old: | `E4040` | `@nopanic(explicit)` function reaches a panic; a making line `using x` whose `allocate` may return null | `[EFF-17]`, `[ALC-6]` |
  - new: | `E4040` | `@nopanic(explicit)` function reaches a panic, a `runtime` name whose test may stop it included; a making line `using x` whose `allocate` may return null | `[EFF-17]`, `[EFF-16]`, `[ALC-6]`, `[LT-48]` |
  - why: The row must claim what the rules say.

**remaining_doubt**: The `Aliasing` half follows from the text's own purpose for `@no_alias_checks` (`[EFF-12]`: a loop can always do without a run-time alias check, since a kind known when compiling needs none) and is not in doubt. The `Panic(Explicit)` half is a classification choice safety does not decide: option B (an alias check only, like a bounds check) keeps `[EFF-16]`'s promise too, since such a function is not free of `RuntimeCheck`. The edits take A, as all three finders did, because the text's own principle for `Panic(Explicit)` is a stop a written word asks for (`panic`, `assert*`, `cast x as T`, `assert_disjoint`), and `runtime` is exactly such a word (verdict 152). If the lead prefers B, drop the `Panic(Explicit)` row edit, the `[EFF-16]` edit and the `E4040` row edit, and in the `[LT-48]` edit keep only `RuntimeCheck(Aliasing)` and `E4030`. Verdicts 152, 157, 162 and 163 are met either way; none of them speaks of contracts.

**REFUTER reason**: The hole is real. `[LT-48]` makes the `runtime` test a check the compiler emits where its proof fails ("is not refused when compiling where which kind it is is known only while the program runs. The function then reads, at the call, ... tests the fact's bit for that place and stops the program when it is set"), yet no rule gives it an effect kind: `[EFF-9]` limits `Aliasing` to "the compare of `assert_disjoint` and `check_disjoint` (`[DSJ-1]`), the one run-time alias check there is", `[DSJ-6]` repeats "They are the one run-time alias check there is", `[EFF-12]` says `@no_alias_checks` "forbids only `RuntimeCheck(Aliasing)`, the run-time compare of `assert_disjoint` or `check_disjoint`", and the `Panic(Explicit)` row does not name it. So by the letter `present` compiles under `@no_alias_checks` with a run-time alias check in it, against the attribute's own words (`[EFF-12]`'s `E0104` text: "it forbids run-time alias checks only"), and two exhaustive lists are false: `[COST-6]` ("Of the costs above, these may stop the program") has no row for the `runtime` check though `[COST-3]` lists it and `[LT-48]` gives its stop text, and `[PRF-4]` ("The checks that run while a program runs are the same in every profile ... bounds checks; the arithmetic checks ...; `assert_disjoint`'s compare of two spans; ...") omits it. `[EFF-6]` does not rescue the program: `Renderer` is not `pub`, so the `dyn` call "counts the implementations and overrides the function's own package sees". Confirmed as not_answered. The edits are narrowed: the `Aliasing` half (EFF-9, EFF-12, DSJ-6, COST-6, the E4030 row) is right and at the cause; the `Panic(Explicit)` half (the effects-table row, the EFF-16 edit, the E4040 row, and the `E4040` clause of the LT-48 edit) should be dropped, for the text's own reasons: `[EFF-9]` defines `RuntimeCheck(k)` as what "enters a function's effect set when a check of kind `k` survives the compiler's profile-independent proofs" and the table as "a safety check the compiler emitted rather than proved away", which is exactly what the `runtime` test is; `[EFF-11]` has its reason ready, `not_provable_in_principle`, "the property depends on run-time data"; `[EFF-16]`'s principle for `Panic(Explicit)` is "the panics the programmer writes", and `runtime` writes a check, not a panic, as `a[i]` and `pool[h]` do (`RuntimeCheck` only, though the programmer chose the type, `[EFF-11]` `requested_by_type`), unlike `assert_disjoint`, which is an `assert*`. Option A would also make `@realtime`'s default set (`[EFF-19]`: "`@noalloc @nolock @noblock @nopanic(explicit)`") refuse every `runtime` name, a loss with no safety gain, and would turn a `runtime` test inside `migrate_from` into `E2225` under `[HR-35]` ("may not contain an explicit panic") instead of an `Err` return ("every run-time check inside it ... returns `Err(ReloadError.Migration(...))`"). Under B `[EFF-16]`'s promise holds as written: `present` is not "free of `RuntimeCheck`". Verdicts 152, 157, 162 and 163 are met either way; none speaks of contracts. Two lists the reader did not reach need the same name added: `[PRF-4]` and `[HR-35]`.

- BETTER EDIT rule=EFF-9
  - old: `Aliasing` covers the compare of
  `assert_disjoint` and `check_disjoint` (`[DSJ-1]`), the one run-time alias check there is; `Stale`
  - new: `Aliasing` covers the compare of
  `assert_disjoint` and `check_disjoint` (`[DSJ-1]`) and the test of a `runtime` name's fact at a
  line that changes what the call's result may point into (`[LT-48]`), the two run-time alias
  checks there are; `Stale`
  - why: The `runtime` test is a safety check the compiler emits rather than proves, `[EFF-9]`'s own definition of `RuntimeCheck`; this is the rule that assigns kinds.

- BETTER EDIT rule=EFF-12
  - old: it forbids only `RuntimeCheck(Aliasing)`, the run-time compare of `assert_disjoint`
  or `check_disjoint`, because a data-oriented inner loop can always do without it: spans the
  compiler proves apart need no compare, and a copy needs none (`E4030`: "`mix` is
  - new: it forbids only `RuntimeCheck(Aliasing)`, the run-time compare of `assert_disjoint`
  or `check_disjoint` and the test of a `runtime` name (`[LT-48]`), because a data-oriented inner
  loop can always do without it: spans the compiler proves apart need no compare, a copy needs none,
  and a kind known when compiling is checked when compiling (`E4030` at the word: "`present` is
  `@no_alias_checks`, and `runtime` on `r` (line 19) tests while running whether `scratch` changes
  while `frame` looks at it; give `present` a kind known when compiling, `fn present(r: Gpu, ref
  mut scratch: Array[u8]):`, or remove `@no_alias_checks`"; `E4030`: "`mix` is
  - why: `@no_alias_checks` means no alias check runs in the function; the message gives the line to write.

- BETTER EDIT rule=DSJ-6
  - old: They are the one run-time alias check there is, so inside `@no_alias_checks`
  each is `E4030`
  - new: They are, with a `runtime` name's test (`[LT-48]`), the run-time alias checks there
  are, so inside `@no_alias_checks` each is `E4030`
  - why: The sentence became false when verdict 152 added the `runtime` test; `[DSJ-6]` must claim what `[EFF-9]` says.

- BETTER EDIT rule=LT-48
  - old: it costs
  one load at the call and one bit test at each line it guards, and nothing where no line needs them
  (§X.4).
  - new: it costs
  one load at the call and one bit test at each line it guards, and nothing where no line needs them
  (§X.4). Where a test is emitted it enters the function's effect set as `RuntimeCheck(Aliasing)`
  (`[EFF-9]`), with the reason `not_provable_in_principle` (`[EFF-11]`), as the compare of
  `assert_disjoint` does: inside `@no_alias_checks` the word is `E4030` (`[EFF-12]`), naming the kind
  known when compiling that needs no test; `@nopanic(explicit)` admits it, as it admits every
  `RuntimeCheck` (`[EFF-16]`). A `runtime` name whose every kind is known when compiling, where
  nothing is tested, carries nothing, and `ember inspect --safety` lists each test with its line
  (`[EFF-10]`).
  - why: The rule that defines the test says what it costs and must say what it carries; the classification follows `[EFF-9]`'s definition and `[EFF-16]`'s principle (a check, not a panic the programmer writes).

- BETTER EDIT rule=COST-6
  - old:   | `assert_disjoint` (`[DSJ-1]`) | "`assert_disjoint(d, s)` on line 11: the two spans overlap (`dst_name` and `src_name` name one track)" |
  - new:   | `assert_disjoint` (`[DSJ-1]`) | "`assert_disjoint(d, s)` on line 11: the two spans overlap (`dst_name` and `src_name` name one track)" |
  | a `runtime` name's test (`[LT-48]`) | "`xs` changes while `r` looks at it: `p` is a `Yours`, whose `pick` gives back the list it was handed" |
  - why: `[COST-6]` lists every cost of X.4 that may stop the program; the `runtime` row of `[COST-3]` has no entry, though `[LT-48]` gives the stop's text.

- BETTER EDIT rule=PRF-4
  - old: `assert_disjoint`'s compare of two spans;
  - new: `assert_disjoint`'s compare of two spans and the test of a `runtime` name's fact at a line
  that changes what a call's result may point into (`[LT-48]`);
  - why: `[PRF-4]`'s list of the checks that run in every profile is exhaustive and omits the `runtime` test.

- BETTER EDIT rule=HR-35
  - old: every run-time check inside it — overflow, bounds, a cast, a count
  at its maximum, a stale handle — returns
  - new: every run-time check inside it — overflow, bounds, a cast, a count
  at its maximum, a stale handle, a `runtime` name's test (`[LT-48]`) — returns
  - why: Once the test is a run-time check, `[HR-35]` must say that inside `migrate_from` it returns `Err(ReloadError.Migration(...))` rather than stopping the process.

- BETTER EDIT rule=registry row E4030
  - old: | `E4030` | `assert_disjoint`, the one alias check left to run time, inside `@no_alias_checks` | `[DSJ-6]`, `[EFF-12]` |
  - new: | `E4030` | `assert_disjoint`, `check_disjoint` or a `runtime` name, the alias checks left to run time, inside `@no_alias_checks` | `[DSJ-6]`, `[EFF-12]`, `[LT-48]` |
  - why: The row must claim what the rules say; `[DSJ-6]` already makes `check_disjoint` `E4030` too.

## K61-arena-scope-views  reader=answered_differently  refuter=confirmed  should_be=answered_differently  safety=none

**answered_by**: ARN-1: "`reset()` and the arena's end reuse or free that memory: each ends every view into the arena, and a later use of one is `E3060`. A scope (`[ARN-6]`) ends no view made before it: while it is live the parent, and every view into it, is not used (`E3096`, since the scope holds a `ref mut` view of the arena, `[LT-46]`), and its end reuses only the scope's own bytes, ending the views into the scope, which live no longer than it in any case (`[LT-1]`)."; ARN-6: "`with s = frame.scope():` allocates from `s` and releases everything at the block's end, last in first out. Using the parent while a scope is live is `E3096`, whose help is to allocate from the scope. Scopes nest: `s.scope()` opens one inside another."; ARN-7: "No operation lowers an arena's allocation pointer, reuses its bytes or moves what it has given out while a view into it is used: every such operation ends every view into the arena (`[ARN-1]`)"; LT-46: "`ref mut T` may change, and it is alone: while its holder is still used, the place is not read or changed through any other name (`[BRW-1]`)"

**program**: 

**remaining_doubt**: The question is decided: a scope ends no view made before it, so the finder's `return root.kind` after the block compiles (not `E3060`), which is the finder's recommendation A on the point asked. It differs from A's outcome in one respect: `root.kind += toks[0].kind` inside the scope is `E3096`, since every earlier view into the arena is unusable while the scope is live. That is safe (over-restrictive) and is `[LT-46]`'s holding rule applied to the `ref mut` the `ScopedArena` holds, but it is a chosen restriction, not a forced one: `[ARN-1]` itself says each `alloc*` result is "a place of its own" that the next `alloc*` (also through a `ref mut` of the arena) neither ends nor overlaps, so a read or write of `root` during the scope touches no byte the scope can reuse. Relaxing it (a scope excludes the parent's `alloc*`, `reset()` and `scope()`, not the use of earlier views) would accept the parser layout the finder wanted; a design choice for the lead, not a defect of safety. No verdict covers scopes.

**REFUTER reason**: Decided, and safely. `[ARN-1]`: "A scope (`[ARN-6]`) ends no view made before it: while it is live the parent, and every view into it, is not used (`E3096`, since the scope holds a `ref mut` view of the arena, `[LT-46]`), and its end reuses only the scope's own bytes, ending the views into the scope, which live no longer than it in any case (`[LT-1]`)"; `[ARN-6]`: "the `ScopedArena` holds a `ref mut` view of it for the scope's life (`[LT-46]`) ... Using the parent while a scope is live is `E3096`, whose help is to allocate from the scope"; `[LT-46]`: "`ref mut T` may change, and it is alone: while its holder is still used, the place is not read or changed through any other name". So `return root.kind` after the block compiles (the view was made before the scope and the scope "ends no view made before it"), and `root.kind += toks[0].kind` inside the block is `E3096` ("every view into it, is not used"). Both outcomes refuse or admit on the safe side: a view into the parent is never used while the scope's `ref mut` holder is used. The reader is right that `[ARN-1]`'s "each a place of its own" would make a relaxation sound (a scope's `alloc*` touches no byte an earlier view reaches), but the text as written is consistent with `[LT-46]` and compromises nothing; a design choice for the lead, not a defect, and no verdict covers scopes.

## K14-reload-shared-block  reader=answered  refuter=confirmed  should_be=answered  safety=none

**answered_by**: HR-12: "A `thread_shared` value's 32-byte header (`[OBJ-1]`) is 48 bytes in a reloadable build. Registering costs two stores when an object is made and two when it ends, and for an object that is not `thread_shared` touches nothing another thread touches (`[RT-11]`); a `thread_shared` object's registration is ordered by the runtime, one lock or one atomic exchange, on whichever thread makes or ends it. ... A value made shared in a block of its own (`[RC-1]`) is registered as a class instance is, with its type's schema, and migrated once, whatever the number of owners and links that reach it."; RC-1: "A value made shared lives in a block of its own with the header of `[OBJ-1]` (a class object already does)"; COST-3 row: "| Registering an object in a reloadable build (`[HR-12]`) | guaranteed present | two stores when it is made and two when it ends; a lock or an atomic exchange for a `thread_shared` object; a `thread_shared` header of 48 bytes |"; HR-15: "commit rewrites every owner, link and view of it that the runtime can enumerate: the one-owner fields and elements that hold it (`T`, `owns T`), its `shares`, `shares mut` and `thread_shared` owners, its `weakref` links, and the kept views into it ... in live instances and registered containers, in statics, and inside tuples, `owns` blocks and function values' state held by any of these"; HR-14 table: "| field of a migrated type | migrated depth-first |"; HR-3b: "an instance a frame's view points into is not changed in place while that frame lives: a layout change of its type is a dependence of the frame, and is refused naming the function, the line of its call and the view"

**program**: 

**remaining_doubt**: The finder's program (`s = thread_shared Stats()`, `Stats` gains a field) migrates: the block is registered as a class instance is, with its header 48 bytes in a reload build, the finder's option A, and `[COST-3]` carries the row. The side question of an `owns Stats()` box is covered differently: an `owns` block is reached through its holder (`[HR-15]`'s "inside ... `owns` blocks", `[HR-14]`'s depth-first row) when the holder is a registered instance, container or static, and one held only by a live frame's local is a dependence of that frame, refused at a checkpoint (`[HR-3b]`); at a poll no frame is live (`[HR-3]`). Nothing is left unregistered and unrefused.

**REFUTER reason**: Decided. `[RC-1]`: "A value made shared lives in a block of its own with the header of `[OBJ-1]` (a class object already does)", and `thread_shared Stats()` makes such a value (`[THR-1]`: "it is written before a new value (`st = thread_shared Stats()`)"). `[HR-12]`: "A value made shared in a block of its own (`[RC-1]`) is registered as a class instance is, with its type's schema, and migrated once, whatever the number of owners and links that reach it", and "A `thread_shared` value's 32-byte header (`[OBJ-1]`) is 48 bytes in a reloadable build"; the `[COST-3]` row "Registering an object in a reloadable build (`[HR-12]`)" prices it ("a lock or an atomic exchange for a `thread_shared` object; a `thread_shared` header of 48 bytes"). The `owns Stats()` side question is decided too: an `owns` block reached from a registered instance, container or static is rewritten and migrated with its holder (`[HR-15]`: "inside tuples, `owns` blocks and function values' state held by any of these"; `[HR-16]`: "through links, `owns` blocks, containers and tuples alike"), and one held only by a live frame's local exists only at a checkpoint, where `[HR-3b]` makes "those a value it still uses is of, holds, or views (... through links and `owns` blocks alike ...)" a dependence of the frame, refused; at a poll `[HR-3]` requires "each thread's Ember-depth counter is zero", so no frame holds one. Nothing is left unregistered and unrefused, and the answer adds no run-time work beyond the priced row. Verdict 165 does not fire (an `int` field is `ThreadShared`).

## K48-migrate-from-ownership  reader=answered_differently  refuter=confirmed  should_be=answered_differently  safety=none

**answered_by**: HR-16: "`OldSelf` is the previous schema as a generated struct `Old<Name>`, visible only in that function; `old` is a read-only view of the old instance. ... A `shares`, `shares mut`, `thread_shared` or `weakref` link read from `old` may be stored into the new instance's field of the new type (`log=shares old.log`): it names the old instance until commit, and commit's map (`[HR-2a]`) repoints it to the migrated one; so `migrate_from` never reads the new instance of another type, and never stores a view of an `Old` value (`E3063`, since the old block is freed in reclaim)."; HR-2a: "Prepare ... copies into the arena the old values of the fields that will not be carried over by name (removed fields, the old value of each converted field, and every field of an instance rebuilt by `migrate_from`), runs every `migrate_from` against a read-only view of the old instance ... Reclaim ends those saved old values, running their `drop`s and never the type's own `drop`, since the instance lives on in its new form (a field carried over by name has been handed over and is not ended), then frees each old block whose instance moved."; HR-37: "A reload discarded after prepare first ends every instance prepare built, field by field and without the type's own `drop` (as reclaim ends an old version, `[HR-2a]`), before the arena is released: a count raised through `old` (`shares old.log`) is lowered"; OWN-11 (VI.2): "a field of a class object, which other names may reach and would see the hole, and anything reached through a name that does not own it, are `E3013` (`[OWN-11]`). The value inside an `owns T` place belongs to that place: handing it out on its own (`m: Mesh = b` for `b: owns Mesh`) is `E3013` too"; Annex B intro: "Hot reload turns a source edit into running behaviour in a live process, keeping existing objects, statics and open resources."

**program**: 

**remaining_doubt**: Both halves of the question are decided. Old fields: every field of an instance rebuilt by `migrate_from` is saved in prepare and ended in reclaim, with its `drop` and never the type's own, so `target=shares old.target` leaves no count too many (the new owner adds one, reclaim's end of the saved field subtracts one; `[HR-37]` does the same on a discarded reload) and an old socket is closed. The socket: `sock=old.sock` and `sock=owns old.sock` are `E3013` (a hand-out through a name that does not own, `old` being a read-only view), so a non-`Clone` owned resource cannot be carried; the type may still declare `migrate_from` and must give the new instance a new `Socket` (the checker's option B is not what the text does: nothing refuses the declaration). This is safe. It differs from the finder's recommendation A (`static fn migrate_from(owns old: OldSelf)`, leftovers ended at reclaim) and leaves Annex B's "keeping ... open resources" true only for field-wise migration, not for a type with `migrate_from`; if the lead wants that promise kept for such types, option A (written `Session(sock=old.sock, target=old.target, hp=Health(value=old.hp))` by verdict 159, with `owns old` on the first line) is the design to add, and `[HR-2a]`'s "every field of an instance rebuilt by `migrate_from`" would become "every field `migrate_from` did not take". A design choice, not a hole.

**REFUTER reason**: Both halves are decided as the reader says, and the current shape is forced, not a free design choice. `[HR-16]`: "`old` is a read-only view of the old instance"; `[OWN-11]`: "A hand-over out of a name or place that does not own the value is `E3013` too", so `sock=old.sock` and `sock=owns old.sock` (which by verdict 159 "works as `=` does" for a non-`Copy` value) are `E3013`, while `target=shares old.target` is admitted (`[HR-16]`: "A `shares`, `shares mut`, `thread_shared` or `weakref` link read from `old` may be stored into the new instance's field ... it names the old instance until commit, and commit's map (`[HR-2a]`) repoints it"). The old fields leave no count behind: `[HR-2a]` saves "every field of an instance rebuilt by `migrate_from`" and "Reclaim ends those saved old values, running their `drop`s and never the type's own `drop`", and `[HR-37]` does the same on a discarded reload ("a count raised through `old` (`shares old.log`) is lowered"). Safe. The reader's residual option A (`owns old: OldSelf`, the socket moved out) is not open to the lead: `[HR-2]` promises "A failure in plan or prepare leaves the process running the old image, with no instance changed, handed over or ended", and `[HR-37]` "ends every instance prepare built, field by field" on a discard, which would close a socket moved out of the old instance and leave the old instance without it; `[HR-2a]`'s "it writes nothing into an old block" is the same invariant. The read-only `old` is what keeps reload transactional, so the only promise left short, Annex B's intro "keeping ... open resources" for a type that rebuilds itself with `migrate_from`, is the price of `[HR-2]`, and the program's remedy is field-wise migration of the changed field where it applies.

## K51-ref-to-ptr-spelling  reader=answered_differently  refuter=confirmed  should_be=answered_differently  safety=none

**answered_by**: §IV.5 Raw types: "Creating one from a reference is safe (ODR-075): `ref_to_ptr(ref x) -> *T` and `ref_to_ptr(ref mut x) -> *mut T` are one function the compiler knows, not two of one name (`[TYP-26]`): its argument is a view written with its word (a plain parameter, `ref x`, or `ref mut x`), and the result's type follows that word. An explicit `r as *T` is also safe when the pointee type is unchanged and the cast does not strengthen mutability: `ref T` to `*T`, or `ref mut T` to `*T` or `*mut T`."; TYP-26: "There is no overloading: two functions of one name in one scope are `E1030`, except operator-interface implementations and `extend` blocks for different types."; Appendix H.52 (N74): "`ref_to_ptr` is one function the compiler knows, whose result follows the word its argument is written with, not two functions of one name, which `[TYP-26]` forbids (§IV.5)."

**program**: 

**remaining_doubt**: Decided by the lead's ruling N74: one compiler-known function whose result type follows the argument's word, beside the safe cast; neither the finder's A (two names) nor its recommendation B (the cast alone) nor C (overloading by word). Safe: the program's own `addr_of` pair stays `E1030`, the library has no overload (the function is the compiler's, as `[TYP-26]`'s exceptions are), and `[HR-13b]` and Part XVI keep citing `ref_to_ptr`, which still exists. Small residue: `ref_to_ptr(x)` on a local `x` is accepted through `[TYP-5]` rule 7 (`ref T` from a place) and gives `*T`; the help for the refused `addr_of` pair is `E1030`'s general one.

**REFUTER reason**: Decided by §IV.5: "`ref_to_ptr(ref x) -> *T` and `ref_to_ptr(ref mut x) -> *mut T` are one function the compiler knows, not two of one name (`[TYP-26]`): its argument is a view written with its word (a plain parameter, `ref x`, or `ref mut x`), and the result's type follows that word", beside the safe cast "`r as *T` is also safe when the pointee type is unchanged and the cast does not strengthen mutability". `[TYP-26]`: "There is no overloading: two functions of one name in one scope are `E1030`", so a program's own `addr_of` pair stays refused, and the compiler-known function is outside the library, where no overload exists. Ruled by the lead as N74 (decisions.md §12), not an owner verdict, and nothing departs from one. Safe: a `ref T` gives `*T` only, `ref mut T` the `*mut T`, no mutability is gained, and dereference still needs `unsafe` ("Dereferencing, offsetting, reading and writing through it need `unsafe`"). The reader's residue (the help of `E1030` for an `addr_of` pair) is general help, not a hole.

## K60-gpu-frame-early-exit  reader=answered  refuter=confirmed  should_be=answered  safety=none

**answered_by**: GPU-2: "A `Frame` has one owner and is not `Copy`, and `end_frame(owns frame: Frame)` takes it over (`device.end_frame(owns frame)` ends the name `frame`), so ending a frame that never began, or ending one twice, is refused when compiling. A `Frame` whose owner ends without `end_frame` is ended by its `drop`, which ends the frame as `end_frame` does with nothing recorded: the device submits nothing for it and marks its fence observed, so no later `begin_frame` waits for it, and what was made in its arena ends with it (`[GPU-3]`)."; GPU-3: "`frame.arena()` is the frame's transient memory manager, an instance that lives as long as the `Frame`: a value made `using` it (`[ALC-1]`) lives no longer than the frame (`E3063`), so nothing made in it outlives `end_frame`."; THR-6: "The standard library applies it to `Scope` and `JobScope` only."

**program**: 

**remaining_doubt**: The finder's program compiles; on the `?` the `Frame`'s `drop` closes the frame, submits nothing and marks the fence observed, the finder's recommendation A. `Frame` rightly stays off `[THR-6]`'s `@must_drop` list (no safety guarantee depends on its `drop`). Nothing left.

**REFUTER reason**: Decided by `[GPU-2]`: "A `Frame` whose owner ends without `end_frame` is ended by its `drop`, which ends the frame as `end_frame` does with nothing recorded: the device submits nothing for it and marks its fence observed, so no later `begin_frame` waits for it, and what was made in its arena ends with it (`[GPU-3]`)". On the `?` the `Frame`'s owner ends and its `drop` runs; `[GPU-3]` keeps the arena's values inside the frame's life ("a value made `using` it (`[ALC-1]`) lives no longer than the frame (`E3063`)"), and since nothing is submitted, nothing on the GPU reads the arena after it ends. `[THR-6]` rightly leaves `Frame` off `@must_drop` ("The standard library applies it to `Scope` and `JobScope` only"): no safety guarantee depends on the `drop` running at a particular point, only resource hygiene, which `[OWN-2]`'s every-value-ends rule already gives. Nothing left.

## K67-bool-char-casts  reader=answered_differently  refuter=confirmed  should_be=answered_differently  safety=none

**answered_by**: TYP-6: "`char → u32` and `u8 → char`; other integers become a `char` only through `char.from_u32(x) -> Option[char]`."; TYP-6: "`bool → integer`: `true` is 1 and `false` 0 (Python's `int(True)`, `[PHIL-14]`); nothing converts to `bool` (`x != 0` is the test);"; TYP-6: "`as` between any other pair of types is `E2020`, naming both."; TYP-5: "Nothing converts implicitly to or from `bool` or `char`."; ENM-3: "A unit-only enum is `Copy`, `Eq`, `Ord` (declaration order), `Hash` and `Debug` automatically, converts to its representation with `as`, and converts back with `Mode.from_repr(x) -> Option[Mode]`."

**program**: 

**remaining_doubt**: The list is now closed ("`as` between any other pair of types is `E2020`"), so every case is decided: `visible as u8` and `lit as u8` compile (the finder's A for `bool`); `c as int` and `c as u8` are `E2020`, and the program writes `(c as u32) as int` (the finder's B for `char`, not its recommendation A). Safe, and consistent with `[TYP-5]`. The `E2020` text "naming both" does not spell the help for `c as int`; `(c as u32) as int` is the one line to write, which the lead may add to the message if it wants the help explicit. No verdict covers it (verdict 145 is about `owns` on a `bool`).

**REFUTER reason**: Decided by `[TYP-6]`'s closed list: "`bool → integer`: `true` is 1 and `false` 0 (Python's `int(True)`, `[PHIL-14]`); nothing converts to `bool`", so `visible as u8` compiles; "`char → u32` and `u8 → char`; other integers become a `char` only through `char.from_u32(x) -> Option[char]`" and "`as` between any other pair of types is `E2020`, naming both", so `c as int` and `c as u8` are `E2020` and the program writes `(c as u32) as int`. Consistent with `[TYP-5]`: "Nothing converts implicitly to or from `bool` or `char`", and with `[ENM-3]` for enums. Safe: `char → u32` is total, `u8 → char` is total (every byte is a scalar value), and the one lossy direction goes through an `Option`. No verdict covers it (145 is `owns` on a `bool`). The one gap the reader noted, that `E2020`'s "naming both" gives no line to write for `c as int`, is listed under missed with its edit; it is help, not a hole.

# MISSED

## MISSED: The registry row for E4070 omits the lock's view and try_recv, which [DET-2] names

**rules**: `[DET-2]`: "a read whose value depends on which thread acted first: a receive on a channel more than one thread may send to (one sender on one thread gives the sent order), `try_lock`'s and `try_recv`'s answer, and a lock's view or an `Atomic` read of a place another thread, or a task not yet joined, may change". Registry row: "a read whose value depends on which thread acted first (a receive on a channel several threads send to, `try_lock`, an `Atomic` read of a place an unjoined task may change)".

**program**:
```
The finder's K54 program: `race` is `@deterministic` and reads `ORDER` under `ORDER_LOCK` while `push_one`, started by `thread.spawn`, may change it. `[DET-2]` makes the read `Nondet`; a reader of the registry row, which names only channels, `try_lock` and `Atomic` reads of a place an unjoined task may change, finds no clause for it, and the row's "unjoined task" leaves out "another thread".
```

- MISSED EDIT rule=registry row E4070
  - old: a read whose value depends on which thread acted first (a receive on a channel several threads send to, `try_lock`, an `Atomic` read of a place an unjoined task may change)
  - new: a read whose value depends on which thread acted first (a receive on a channel several threads send to, `try_lock` and `try_recv`, a lock's view or an `Atomic` read of a place another thread or an unjoined task may change)
  - why: Every cross-reference claims what its rule says; the row left out the two cases `[DET-2]` added in 0.9.20.

## MISSED: The registry row for E3096 names only the parent arena, not the earlier views [ARN-1] also refuses

**rules**: `[ARN-1]`: "while it is live the parent, and every view into it, is not used (`E3096`, since the scope holds a `ref mut` view of the arena, `[LT-46]`)". `[ARN-6]`: "Using the parent while a scope is live is `E3096`". Registry row: "| `E3096` | arena is scoped here | `[ARN-6]` |".

**program**:
```
The finder's K61 parser: `root = frame.alloc(owns Node(kind=0))`, then `with s = frame.scope():` and inside it `root.kind += toks[0].kind`. `[ARN-1]` makes the line `E3096`; the registry row, which says only "arena is scoped here" and cites `[ARN-6]`, describes a use of the parent, not of a view made before the scope, so a reader of the registry cannot tell which rule the code comes from.
```

- MISSED EDIT rule=registry row E3096
  - old: | `E3096` | arena is scoped here | `[ARN-6]` |
  - new: | `E3096` | the parent arena, or a view into it made before the scope, used while a scope of it is live | `[ARN-6]`, `[ARN-1]` |
  - why: One code, one defining rule, and the row claims what both rules say.

## MISSED: E2020 for `c as int` gives no line to write, though one exists

**rules**: `[TYP-6]`: "`char → u32` and `u8 → char`; other integers become a `char` only through `char.from_u32(x) -> Option[char]`" and "`as` between any other pair of types is `E2020`, naming both." The owner's rule 2 (the brief): "the message gives the line to write".

**program**:
```
```
fn main():
    c = 'a'
    n = c as int          # E2020 naming `char` and `int`; the line to write is `(c as u32) as int`
```
The text refuses it, safely, but the message names the two types and no help, while every other refused conversion in `[TYP-6]` gives its line (`Color.from_repr(7)`, `char.from_u32(x)`, `T.try_from(x)`).
```

- MISSED EDIT rule=TYP-6
  - old: `as` between any other pair of types is `E2020`, naming both.
  - new: `as` between any other pair of types is `E2020`, naming both, and where two of the conversions above join them its help gives the chain (`c as int` for a `char`: "a `char` converts to a `u32` only; write `(c as u32) as int`").
  - why: The message gives the line to write, as the rule's other refusals do; nothing new is accepted.
