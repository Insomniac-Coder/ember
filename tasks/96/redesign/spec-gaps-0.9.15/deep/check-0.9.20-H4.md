# Check of 0.9.20_Hardened_4 (2026-11-10, solo, attended)

The owner's request: pull the latest, and check the 0.9.20 spec for gaps, memory-safety flaws and
differences from the implementation that hurt readability. No agents, no workflow. In the first two
sittings nothing in the repository was changed but this file; the third read the rest, and
0.9.20_Hardened_5 was cut from this record (the last two sections before the departures).

## What was pulled

* `main` was already level with `origin/main` (`ad9bcdec`).
* `origin/spec-0.9.20-hardening` moved `1a6459e8` to `977d69e7` (8 commits of 2026-11-08: it merged
  `main`, cut Hardened_3 and Hardened_4). It is 16 commits ahead of `main` and 0 behind, so `main`
  can fast-forward to it. **Not merged and not checked out**: the spec and records were read from
  the fetched branch. Development target there: `docs/spec-source/Ember_v0.9.20_Hardened_4.md`
  (17,702 lines).

## What was read, and what was not

Read in full: the branch's handoff top blocks, `docs/AUDIT-0.9.20.md`, ODR-260 to ODR-267 and
ODR-238, the spec's front matter and Part I, §IV.4 to §IV.6a, `[FN-1]` to `[FN-2a]`, `[GRM-15]`,
`[GRM-42]`, `[GRM-43]`, `[GRM-47]`, `[CLO-2]` to `[CLO-4]`, Part VII to `[LT-46]` and §VII.8,
§VIII.2, §VIII.3, `[THR-1]`, `[THR-2]`, §XI.3. **Not read:** Parts II, III (but the rules above),
V, VI (but closures), IX, X, XII to XVIII, the annexes. So "no hole found" below covers the parts
read, not the document.

## Findings

Kinds: TEXT the document is damaged or stale; GATE a check does less than the rule says; GAP a fact
left unsaid; IMPL the compiler differs from the spec; READ a readability cost (the owner's call);
OWNER waits for the owner's word.

| # | Kind | What | Evidence |
|---|---|---|---|
| 1 | TEXT | Two example blocks lost their line breaks in Hardened_2 and are still broken in Hardened_4: the `unsafe import c "./bridge.c"` fragment, and `fn main()` of the `widget_new` example, now one run-on line with the closing fence glued to its end. | H4 lines 9021-9022 and 9626-9628; intact in 0.9.20_Hardened_1; source `tasks/spec-0.9.9/parts/p16-ffi.md:837`. Cause: the second pass's script that re-wraps a bullet at 100 columns. |
| 2 | GATE | Because of 1, `check_examples.py` does not see the fragment, reads the widget block on through `[FFI-56]` to §XVI.7 and uses up that section's opening fence, so the export example is not checked either. Three examples are outside the check. | `check_examples.py` output, block 47: errors on prose lines up to "XVI.7 Exporting Ember to C". |
| 3 | GATE | `[TST-7]` says every example "must pass `ember check`"; the tool runs `ember check --syntax-only` on every block. The quoted "41 of 54 compile" is parse-only. With the whole check, of the 45 complete programs: 21 pass, 13 parse and are refused, 11 do not parse. | H4 line 10299; `check_examples.py` line 93; rerun below. One of the 13 is the conformance example of §XVII.5, which is meant to be refused. |
| 4 | TEXT | The front matter describes Hardened_1, not Hardened_4. "Supersedes" stops at 0.9.15_Hardened_10; "Authority" stops at 0.9.20_Hardened_1, says it changed the memory rules "taking no feature away", and cites §H.51 for it; the table of Parts says "in 0.9.20 (§H.51)". §H.51 is 0.9.15_Hardened_11; 0.9.20 is §H.52 to §H.55. About 350 of the ~1,020 rules carry a 0.9.20 mark (331 changed, 21 new); six codes are new and three retired. | H4 lines 4-7, 27-45, 107; headings at 15367, 15394, 15517, 15900, 16190. |
| 5 | TEXT | Small: `(both reserved).---` (a divider glued to the end of a sentence); a divider directly under text with no blank line at the end of §VII.9 and of the Versioning list, which Markdown may read as a heading underline; `[BRW-1]` cited twice in one bracket of §I.6. | H4 lines 390, 4869-4870, 153-154, 385-387. |
| 6 | GAP | One `owns` where two rules each ask for one. `[GRM-15]` asks for `owns` before an argument for an `owns` parameter (`E3028`), and before a value made for a place that owns it alone (`E2123`). `r.pools.push(owns IntPool(items=[1, 2]))` for `pools: Array[owns dyn Pool]` meets both with one word; no sentence says one word answers both, or what `push(owns owns IntPool(...))` is. An implementer must guess. | `[GRM-15]` (1046), `[OWN-10]` (3817), `[FN-2a]` (2488); `owns owns` appears only as a type (1734). |
| 7 | GAP | **(First half wrong: corrected in the third sitting.)** `Deque[T]`, `SmallArray[T, N]` and `BitSet` are named with no operations. `[BRW-16]` says a name made by indexing "a list" is a position that survives growth, and does not say which containers that covers; a container that grows at the front would make such a name mean another element. | `[STD-7]` (8538), `[BRW-16]` (4239), glossary "position". |
| 8 | IMPL | The compiler builds none of the memory-model words. `owns T`, `weakref`, `shares`, `shares mut`, `sharedref mut`, `thread_shared`, capture words (`fn() owns x:`), `guarded by`, `using` and `is T as d` are all `E0900` or a parse error. The 11 examples that do not parse are these. | rerun below; README on the branch: Phase 2 10%, Phase 3 0%, overall 14% against Hardened_4 (24% on `main` against 0.9.15_Hardened_11). |
| 9 | IMPL | The standard library is still written in the language before 0.9.15: `Box[` 10, `Shared[` 9, `Weak[` 3, `@sync` 3, `mem.forget` 1, `owned fn` 5; `weakref`, `shares`, `thread_shared`, `guarded by` 0. Tests write `owned fn` 203 times, which `[CLO-2]` makes `E1010`. | `grep` over `std/` and `tests/` (`.em` files, comments included). |
| 10 | IMPL, READ | The compiler's messages use the words the spec retired. For the spec's own conformance example in §XVII.5 (`b = a` then `a.push(owns 4)`) the spec's message is "`a` was handed over to `b` on line 4 and is used on line 5; to keep both, write `b = copy of a` ...". The compiler says "`a` is borrowed after it has been moved out of", help "borrow before the move, or clone the value". In the compiler's strings: "borrowed" 80, "moved" 54, "clone" 84, "mutable" 59; "handed over" 4. | block 48; `grep` over `compiler/*/src`. |
| 11 | IMPL | `[MAN-9]` (0.9.20) says "No compile command writes a file the program did not ask for". `ember check x.em` in an empty folder wrote `ember.toml` and `target/`. The compiler follows verdict 135; the spec reversed that half of it (N83, N138). | H4 line 9985; the probe folder after the run. |
| 12 | OWNER | Rulings made by the cloud lead that depart from the owner's verdicts and wait for his word, by the branch's own record: N83/N138 (half of verdict 135: no manifest written); N87/N139 (touch 101 and 114: the kept-by-C mark in the type index); ODR-264 (departs from 155: a list pattern may open a `match` arm); ODR-261 (restores verdict 110's `interface Error: needs (Debug, Display)` after Hardened_3 had added `Send, ThreadShared`; `AnyError` carries the markers). ODR-260, 262, 263, 265, 266, 267, 238 were also ruled by the lead alone and closed. No ODR is open. | `docs/OWNER-QUEUE.md` on the branch, `decisions.md` §13 to §15. |
| 13 | READ | Handing a value over is written two ways. With a word: `take(owns xs)`, `xs.push(owns 3)`, `m.insert(owns (i * 7), owns i)`, `tx.send(owns (i * i))`, `retain(owns fn(p) => p.lifetime > 0.0)`. With none, and the word an error: `b = a`, `return xs`, `Words(names=xs)`, `Emitter(label)`, `xs = [a, b]`. So whether `label` survives `Emitter(label)` depends on `Emitter` being a constructor. These are verdicts 147 to 149 and 158 to 161. | `[OWN-9]`, `[GRM-15]`, `[FN-2a]`. |
| 14 | READ | The same word on the same shape of line is a hand-over in one place and a new heap block in another: `particles.push(owns Particle(...))` makes no block, `e.mesh = owns Mesh(...)` and `pools.push(owns IntPool(...))` make one. Which it is depends on the place's type, not on the line (finding 6 is the rule gap under it). | §I.5, `[OWN-10]`, `[PHIL-2]`. |
| 15 | READ | A span that may be written says so three times: `d = ref mut buf.as_mut_span()`; `fn normalize(ref mut xs: MutSpan[f32])`. A plain name holding a `MutSpan` only reads. | `[FN-1a]`, §VII.7's example. |
| 16 | READ | `total += x` inside a scoped task is not the line's plain meaning: each task adds to a private total joined at the end, and `[[-100], [100, 100]]` into an `i8` stops where adding one by one gives 100 (`[THR-21]`, verdicts 104, 105). `[PHIL-16]` says a rule that makes a program "do something other than what its words say is not part of Ember". | H4 lines 280, 7363-7391. |

## Memory safety

First sitting: no program was found that the parts then read accept and that breaks `[PHIL-10]`
(the second sitting found one: finding 17). Lines of attack tried on
paper, each stopped by a rule: a view of an element kept across a reshape through another name
(`[BRW-16]`, `[BRW-17]`, `[BRW-21]`); a view opened from a `weakref` link while a call ends or
replaces its target (`[RC-5]` point 1); a number changed through a second name under a plain
parameter (`[FN-1]`: never assumed unchanged); a `drop` that changes the list being cleared
(`[DRP-8]`, `[BRW-21]`); a kept view handed to a task with the value it points into (`[OWN-3]`,
`[THR-2]`); the home and away counts with the making thread gone (`[RC-4]` point 3); tasks touching
plain counts (`[THR-11]`).

What stands between that and "memory safe":

* Every pass on 0.9.20 was a paper review by agents of one model. The compiler builds none of the
  changed rules, Phase 3 is at 0%, and the must-refuse programs the rulings name (K07, K10) are not
  written as tests. So about 350 changed rules have never met a compiler.
* The safety of `shares mut`, `sharedref mut` and `weakref` with nothing checked while running rests
  on one unbuilt analysis: every function's summary of what it may change, end and link, for
  function values, `dyn` calls, callbacks C keeps, `drop`s and generic instances alike. One missed
  case there is a use after free.
* One acknowledged leftover: a making thread that waits in one call for its whole life holds the
  memory of values queued to it until that call returns (`[RC-4]` point 5; NG-46/47 "left to a later
  round"). It leaks for a time; it does not break safety.

## Second sitting (2026-11-10, the owner: "Yes, read the rest")

Findings are numbered on from the table above. PAPER means found by reading and not run: the
compiler builds none of these rules.

### 17. MEM, PAPER: what an implementation in another package does to an argument a library gives it

Two facts of a function's summary are about what it does **to an argument**: it stores an owner of it
(`[WK-7]`, `[BLD-14]` item 10) and it makes a `weakref T` link of it (`[OWN-13]`, item 9). Each call
is judged by them. A call through a `dyn`, interface or `virtual` method is judged "by the summaries
of every function the package can put there" (`[WK-7]`). A library cannot see an implementation a
dependant adds, and the rules that cover that case carry other facts only: item 11 (where a result
points), items 12 and 16 and `[MOD-9a]` (walks, views, names opened from links, locks: `E3020`,
`E3021`, `E3022`, `E3031`, `E3060`, `E7018`), `[EFF-2a]` (calls, changes, ends, locks, effects).
None carries an owner stored or a link made. So both go unjudged:

```text
# package lib
pub interface Plugin:
    fn attach(host: shares mut Host)
pub class Host:
    plugins: Array[owns dyn Plugin] = []
pub fn install(owns host: shares mut Host, owns p: owns dyn Plugin):
    p.attach(host)                      # lib sees no implementation that keeps `host`
    host.plugins.push(owns p)           # item 10: "stores p into host", judged at each call

# the program
class MyPlugin implements Plugin:
    host: Option[shares Host] = None
    fn attach(host: shares mut Host):
        this.host = Some(shares host)   # a parameter stored into `this`: "not judged in the body"
fn main():
    h = shares mut Host()
    install(shares mut h, owns MyPlugin())   # proof (1) holds: nothing stored into the new plugin
```

`h` owns the plugin and the plugin owns `h`: a loop of owners nothing opens, accepted, which
`[PHIL-10]` and `[WK-6]` say cannot be. The same route with `this.last = weakref n` in a dependant's
`see(n: Item)`, called by a library's `visit(ref mut v: dyn Visitor, n: Item)` and given
`tree.items[0]` of a `thread_shared` tree, makes a `weakref Item` link, counted with plain adds, to
an object another thread may end: a race on the weak count and a use after free, where `[OWN-13]`
says `E7015`. Round 1's G9-04 covered a library storing a dependant's value; this is the dependant's
method storing the library's.

`[BCK-8]` states the intent: a line whose verdict rests on what the whole program puts in a place is
"checked again, with the same rules, by every package that puts a new ... implementation ... there
... a line checked this way by no build is a defect of the build". The records it cites for that
(`[MOD-9a]`, items 12 and 16) do not hold what these two rules need, so by its own words this is
that defect.

A general fix, costing nothing while running: record these two facts as item 11 records a result's
(what the body needs of the kinds that reach a `dyn`, interface or base-class parameter, or a place
another package can fill), so the package that gives the kind judges the call, and a kind it cannot
know is refused. Not applied: a change of the accepted programs is a new hardening.

### 18. GAP: `shares this` and `thread_shared this` in a method

`[OWN-13]` reads a class's kind from its methods: one-thread when one writes "`shares this` or
`shares mut this` where `[RC-1]` allows them", `thread_shared` when one "writes `thread_shared
this`". `[RC-1]` lists where the words may stand: before "a new value, a local of this function that
owns its value ... or a name that is already a `shares` or `shares mut` owner", and `thread_shared`
"only before a new value or a name that is already a `thread_shared` owner (`E7017`)". `this` is none
of these, so by `[RC-1]` both are refused, and by `[OWN-13]` one of them is what makes a class
`thread_shared`. No rule says when a method may make one more owner of its own object (the common
need: an object that registers itself, `REGISTRY.push(thread_shared this)`).

### 19. GAP, with a memory-safety side, PAPER: `jobs.local_arena()`

`[JOB-5]` is the only rule for it and for `ThreadArena`: it "gives a `ref mut ThreadArena` whose
region is the function the job runs ... and the arena is reset when the job ends". Unsaid: whether
the arena is the thread's or the job's; whether two calls in one job name one place (two `ref mut`
names of one arena, each compiled as the only one, `[BRW-22]`); and what "reset" does to a job that
runs inside another job's wait on the same thread. `[PAR-4]` has that case: "when they are full the
calling thread runs the chunk itself". If the arena is the thread's and the inner job's end resets
it, the outer job's `xs = ref mut a.alloc_zeroed[int](1024)` points at reused memory, and nothing
when compiling ties the two calls to one place. Safe if each job only gives back its own bytes
(`[ARN-6]`'s scope); the text does not say which.

### 20. TEXT: `split_at_mut`

`[SPN-5]`: "There is no separate `split_at_mut`". `[DSJ-11]`: "the two results of one `split_at`,
`split_at_mut`, `get_pair_mut` or `chunks_mut` call" (H4 lines 4792, 6272).

### 21. CON: `debug_assert` and one meaning per program

`[PHIL-13]`: "`debug_assert` is the one check a profile turns off; it may not have side effects, so
it cannot change what a program whose assertions hold computes." §VI.6: its argument may not use
`ref mut` (`E3027`) or hand over (`E3013`), and "a call in it of a function with effects other than
`Panic` is `W2016`", a warning. A function that changes a `static mut` by its name has no effect in
the effect table (§X.1 lists `Alloc`, `Sync`, `Lock`, `Block`, `Io`, `Panic`, `RuntimeCheck`,
`Unsafe`, `FFI`, `Nondet`; a change is a summary fact, `[EFF-24]`). So
`debug_assert(bump())`, with `bump` doing `COUNT += 1` and returning `true`, compiles with no
message, and `COUNT` differs between `debug` and `release`. The rule needs the summary (changes
nothing, ends nothing) and an error, not the effect set and a warning.

### 22. TEXT, small (more of finding 5)

* `[COST-6]`'s table: ten rows (H4 lines 6768 to 6777) lost the two-space indent the rest of the
  table has, which ends the list item and splits the table.
* `[TYP-32]` (line 2284): "whose type cannot be written: A `some` list (...) names ...": two
  sentences run together where the example was moved below.

### 23. TEXT: a help line that does not parse

`[LT-4]`'s `E3063` message tells the programmer to write "`with thread.scope() as scope:`" (H4 line
4487). The grammar is `with_item := [IDENT "="] expression`, and every example writes
`with scope = thread.scope():`. `[DIA-25]` says a message's line compiles.

### Read by the end of the second sitting

Read in full: Parts I to XIV; of Part XVI, §XVI.1, §XVI.2, §XVI.6 and `[FFI-55]`, `[FFI-35a]`;
`[BLD-14]`, `[BLD-16]`, `[MAN-9]`, `[TST-7]`; §XVIII.4 to §XVIII.7; Annex A; Appendix E. Skipped
inside those: the listing of the standard interfaces in §IV.8 (lines 2003 to 2195).
**Still not read:** Part XV (the standard library), §XVI.3 to §XVI.5, §XVI.7, §XVI.8, the rest of
Part XVII (toolchain, diagnostics), §XVIII.1 to §XVIII.3, Annex B (hot reload), Annex C (C++),
Annex D (GPU), the glossary and the change logs.

Attacks tried in the second sitting and stopped by a rule: an arena reset, swapped or handed over
under a value made in it (`[ALC-5]`, `[LT-4]`); a `ref mut` name opened from a `weakref` link and
given to a function that reaches the same object another way (`[BRW-21]`, `[EXC-3]`); a lock's
view kept past its block (`[BRW-20]`); a listener that changes the registry that holds it
(`[CLO-17]`, `[ECS-8]`); a generator's view of its own frame across a `yield` (`[CORO-6]`); a
callback that ends its own token (`[FFI-53]`). Finding 17 is the one that got through.

## Third sitting (2026-11-10, the owner: "read the parts you haven't read and fix them too, before creating a new hardened spec")

Read: Part XV, §XVI.3 to §XVI.5, §XVI.7, §XVI.8, the rest of Part XVII, §XVIII.1 to §XVIII.3, Annexes
B, C and D, the listing and tables of §IV.8, and the glossary (Appendix F). With the first two
sittings that is every hand-written line of the document. **Not read:** the three generated
appendices (G, the resolution of the findings of 2026-09-23; H, the change log, but §H.52 to §H.55;
I, the rule index).

No new memory-safety finding in these parts. Text findings, numbered T:

| # | Kind | What | Evidence |
|---|---|---|---|
| T1 | TEXT | `ember audit` (`[CLI-11]`) is missing from §XVII.1's command listing; `[CLI-15]` calls a command the listing lacks a defect. The compiler's `--help` has it. | H4 lines 9792 to 9821, 9884. |
| T2 | TEXT | `[CLI-4]` says a single file is "a package named after the file"; `[MAN-9]`, and the compiler, name it after the file's folder. In Hardened_1 too. | H4 line 9825. |
| T3 | TEXT | `[CLI-20]` cites §H.51 for what 0.9.20 changed; §H.51 is 0.9.15_Hardened_11 (finding 4's mistake again). | H4 line 9846. |
| T4 | TEXT | Thirteen table rows without their table's indent, which ends the list item and splits the table: the ten of `[COST-6]` (finding 22), one of `[HR-14]`, two in Annex C (`[FFI-44]`, `[FFI-48]`). | `parts/p10-effects.md` 476 to 485, `p20-annex-b.md` 215, `p21-annex-c.md` 57 and 156, as Hardened_4 froze them. |
| T5 | TEXT | The cause of finding 5's dividers: the document is `cat parts/p*.md`, and several Parts end with no blank line, so the next Part's `---` lands on the end of a sentence or directly under text, which Markdown reads as a heading underline. The three generated Parts (the code registry, Appendices G and H) end on a table row. | the last bytes of each `parts-0.9.20-h4/p*.md`. |
| T6 | TEXT | Three glossary lines say other than the rules. A `Copy` type includes "structs made only of these" (`[STR-5]`: `Copy` is never implicit, but for a unit-only enum). A hand-over is "for every type, numbers included: `b = a`" (`[OWN-9]`: `=` copies a number, and `b = owns n` is how `=` hands one over). A position is "a name for a list element" (finding 7's question). | H4 lines 12612, 12626, 12646. |
| T7 | TEXT | A unit-only enum is `Copy` with nothing written (`[ENM-3]`, `[STR-5]`); the marker table of §IV.8, the terms that open Part VII and `[OWN-9]` each list the `Copy` types without it. | H4 lines 1961, 3719, 3808. |
| T8 | TEXT | `[TYP-36]`: `copy of` is `E2116` before "a `weakref` link, a `ref mut T` view, a `MutSpan[T]`, a struct holding one". Read as a struct holding a `weakref` link, that contradicts the rule's next sentence and `[OWN-8]`, which copy such a struct field by field. | H4 lines 2104, 2105. |
| T9 | TEXT | `[HASH-2]`: "`std.collections. RandomState`", a name split in two by a re-wrap. | H4 line 2178. |

**Finding 7, corrected.** Its first half was wrong. `Deque`, `SmallArray` and `BitSet` are plain
structs of `std/src/collections.em`, each with its operations, and Part XV's preamble makes
`std/**/*.em` normative for what the Part leaves open. What stands is its second half: `[BRW-16]`
said "a list" and did not say which containers' indexing makes a position.

## What 0.9.20_Hardened_5 did with each finding

Cut on 2026-11-10 from this record, by the lead solo. Rulings N140 to N146 are in
`tasks/96/redesign/compiler-0.9.15/decisions.md` §16; the owner's words are verdicts 167 to 169 in
`tasks/96/redesign/JOB.md`; the change log is §H.56.

| Finding | In Hardened_5 |
|---|---|
| 1, 2 | Fixed: both blocks restored. The example check sees 56 blocks (54 before: the fragment and the export example are seen again, and the widget example ends where it should). `check_examples.py` now reports a code fence that shares its line with other text and exits non-zero; on Hardened_4 it names lines 9021, 9022 and 9628. |
| 3 | Not changed. The tool still parses only, where `[TST-7]` says `ember check`; the whole check can pass only once the compiler builds the words of 0.9.15 and 0.9.20. Recorded in `docs/AUDIT-0.9.20.md` §10. |
| 4, 5, 20, 22, 23, T1 to T9 | Fixed in the text. T5 also in the three generators, which now end their Part with a blank line. |
| 6 | N140: one `owns` says both, where the parameter's type is an `owns T`; a second is `E0109` (`[GRM-15]`, `[OWN-10]`). |
| 7 | Corrected above; N141: a position is made by indexing an `Array`, and an `SoA` proxy is one; any other container's element is an address (`[BRW-16]`). |
| 8, 9, 10 | Not the text's: the compiler and `std` have not caught up. Each lands with the milestone that owns its rule. |
| 11 | The text now says what the compiler does (N146). |
| 12 | Answered by the owner: see the table below and verdicts 167 and 168. |
| 13, 14, 15, 16 | Not changed: each follows from the owner's own verdicts (147 to 149, 158 to 161; 104, 105). Reported to him as readability costs. Finding 14's rule gap is closed by N140; that one word means a new block in one place and a hand-over in another stays. |
| 17 | N142: what a member another package adds keeps of what a call gives it is judged where the member is seen, at the call that gives its kind (`[BLD-14]` item 11) or at the line that adds it to a place (`[MOD-9a]`); a kind the judging package cannot see counts as keeping it (`[WK-7]`, `[OWN-13]`). |
| 18 | N143: `thread_shared this` is written in a method and pins the class; `shares this` and `shares mut this` are `E2122` (`[RC-1]`). Not added, and the owner's to decide: a pin for one-thread shared classes, which would make `shares this` sound. |
| 19 | N144: the arena is the job's own, a scope of the worker's arena, one place for each job (`[JOB-5]`). |
| 21 | N145: a call in `debug_assert`'s argument that changes or ends anything outliving the argument is `E3023`; `W2016` stays for an effect that changes nothing (§VI.6, `[PHIL-13]`, `[PRF-1]`). |

Checked at the cut: `check_ids` 0 problems (1,020 rules), `check_citations` 0 undefined,
`code_registry --check` 0 problems, `rule_index` and `spec_check` no new problems, `phase_progress
--check` green, `check_examples` 41 of 56 with 0 malformed fences; `ember_diag`, `ember_parser` and
the three milestone tests that read the pinned document green. The compiler is untouched, so
everything above the text is still paper: no rule changed here has met a compiler.

## The four departures from the owner's verdicts, judged

The owner (2026-11-10): "if your judgement says they are the better solutions then yes go for it".

| Ruling | Verdict | Judgement | Why |
|---|---|---|---|
| ODR-264: a list pattern may open a `match` arm (`["quit"]:`) | departs from 155 | **Better: stands.** | `[GRM-13]` gives list patterns a meaning; without this they could be written nested (`Some([first, ..rest]):`) and on a `for` line but never as an arm, an exception with no reason. No ambiguity: at arm level a `match` block holds only arms, and a type-parameter line never ends in `:`. Verdict 155's two cases (a stray `[T: ...]` line, `[10, 20].len()` as a statement) are still `E0100`. |
| ODR-261: `interface Error: needs (Debug, Display):`; `AnyError` holds `owns dyn (Error, Send, ThreadShared)` | restores 110's line (Hardened_3 had added `Send, ThreadShared` to `Error`) | **Better: stands.** | It is the owner's own line again. An error holding a one-thread part stays an `Error` on its thread; it is refused only where it would become an `AnyError`, the one place the property is needed (`E2040` at that `?`). `Result[T]` still crosses threads with nothing written. Rust's shape, nothing while running. The header promise on a `pub open` class is the one way a package can decide it from a declaration it sees. |
| N87/N139: the kept-by-C mark is the top bit of the type index | touches 101 and 114 | **Stands.** It fills a gap, it does not contradict either verdict. | Verdict 114 asked for the stop and `[OBJ-1]` had no bit for the mark; the header keeps verdict 101's size. Owed when it is built: N139's "costs nothing measurable" is not measured. The `and` sits on every kind test, cast and table load. The mark is read only at a hand-over with a link, so only an object that carries a weak count needs it; the weak word's top bit would cost nothing on those paths. Price both then. |
| N83/N138: no compile command writes `ember.toml`; a file with no manifest is compiled with default settings held in memory | reverses 135, and 127's "ember should build a default one for you and then create the overlays file" | **Not better. For the owner.** | (1) `[FFI-55]` now says a single file with no manifest "applies its overlay as at a first build", every time: the record verdict 121 made mandatory is never kept, so a header that changed under an overlay is never caught there (`E5010`), and a stale `safe fn` or count is believed. (2) It treats a program with no manifest apart, where the owner's answer on 2026-10-29 was to give every program the file so that none is. (3) Asked whether a program with no C import gets the file too, the owner said "always". What it buys is no stray file after `ember check`, which the compiler's own tests can have by running in a throw-away folder. The compiler today does what 127 and 135 say. |

**The owner's answer (2026-11-10), asked whether to restore verdicts 127 and 135: "Yes, restore them".**
Done in 0.9.20_Hardened_5 (N146; `JOB.md` verdict 168). The other three
stand on the judgement above (verdict 167).

## To rerun

```text
git show origin/spec-0.9.20-hardening:docs/spec-source/Ember_v0.9.20_Hardened_4.md > H4.md
git show origin/spec-0.9.20-hardening:tasks/spec-0.9.9/tools/check_examples.py > check_examples.py
python -I -X utf8 check_examples.py target/release/ember.exe H4.md
```

The whole-check count came from the same block finder with `ember check` in place of
`ember check --syntax-only` on blocks not marked `fragment`, `ignore` or `overlay`
(compiler: `target/release/ember.exe` of 2026-11-02, built from `main`). Run it in a throw-away
folder: the compiler writes `ember.toml` and `target/` there (finding 11).
