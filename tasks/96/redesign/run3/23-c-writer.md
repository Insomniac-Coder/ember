# 23 C writer: W1-W4 in the final model, and the C it compiles to (run 3, phase 4)

Power: `PowerLineStatus` printed `Online` at the start, before the builds and runs, and before writing
this report. No compiler change, no git command, no Ember program run (only `ember check`, section 6).
Files written: this report and the files under `target/redesign3/23-c-writer/` (section 7). Nothing
another agent wrote was edited. The model is `RUN3/combined-v4.md` ("the model").

## 0. Terms

- **check**: code that runs while the program runs and stops it with a panic when something is wrong
  (an index past the end, an integer overflow, a zero divisor, `unwrap` of `None`, a count that would
  overflow). A check costs a compare and a branch that is never taken in these programs.
- **memory-model work**: what the model itself adds while the program runs: a weak count changed, the
  "ended" (DEAD) flag tested when a `weak` link is read, a count held across a call, a list of
  entities to despawn after a loop. Some of it is a test (the DEAD test), the rest is plain work.
- **language check**: a check the language had before this rework and the model does not remove:
  bounds ([TYP-31]), integer overflow ([TYP-8], "panics in every profile"), zero divisor ([TYP-28]),
  `unwrap`. The spec removes one only with a **range fact** ([RNG-4]: a value the compiler knows is
  inside a range, from a loop's range, a comparison, a length, arithmetic on known ranges). I take
  range facts as per function: a `bool` returned by a call (`hp = self.pos.has(e)`) proves nothing in
  the caller.
- **floor `%`**: Ember's `%` on signed integers is floor modulo (Python's), one compare and adjust over
  C's `%` unless the dividend is known to be `>= 0` ([TYP-28], [COST-3]). It is the language's meaning,
  not a check; I list its sites separately.
- **plain C in the model's layout** (the duty's words): each `wN.c` built with `-DPLAIN`. Same object
  sizes, same Array header {ptr, int64 len, int64 cap} growing 4, 8, 16, same slab; every language check
  gone, C's `%`; and the memory-model work removed wherever plain C would not need it (each file's header
  comment says exactly what).
- **tag**: every check site in the C carries a comment `[K1]`, `[K2]`, ... (language checks) or `[M1]`, ...
  (memory-model work). The tables below give the line of each.

## 1. What I did

1. Read the brief, JOB.md, my duty, the model's sections 0-9 (rules R1-R17, the run-time table 1.2),
   area C's rules C1-C10, area D's rules D1-D8 and answers AN-53, AN-60 to AN-65, area A's AN-04 to
   AN-06, and 22's open problems G-01 to G-05 (none touches these four programs: G-05 is the `for` walk
   over handlers, which W4 does not use; G-01 is a copied link whose target ended before a region, and no
   object dies in W2's walk).
2. Wrote each workload in the model's NEW syntax (`wN.em`), from the Python twins in
   `target/redesign2/combined/python/` and the plain C baselines in `target/redesign2/bench/`, with the
   same random source (`harness.h`'s `h_rand`, seeds 42 for W2, 7 for W3) and the same work.
3. Wrote the C the model's compiler would emit for each (`wN.c`, with the shared `ember_rt.h`), every
   check tagged; `-DPLAIN` gives plain C in the model's layout.
4. Built each once with `clang -O2 -I target/redesign/bench` and ran it once through the harness
   (section 5); built the plain C baselines from `target/redesign2/bench/` and ran each once (their
   binaries in `target/redesign2/bench/bin/` no longer exist); ran all four models under AddressSanitizer.

## 2. Choices where the model is silent or self-contradicting (for 24)

Each is a reading of the model's text; 24 should judge each.

| # | where | what the model says | what the C does |
|---|---|---|---|
| C1 | W2, W3: where the weak count and the DEAD flag live | R9: "A class object has an 8-byte header (a 32-bit count and a 32-bit type index)". R2: a copied link adds "to the target's weak count (the count that keeps the target's block ...)"; R11: "one test of the target's header per read"; 21 and 22 call the weak count "the number in an object's header". No place for a weak count in 8 bytes. | DEAD is bit 31 of the count word; the weak count is a **third 32-bit header word, only in classes that some `weak` link targets** (`Node`, `GameObject`). With packing, both objects keep the sizes of the 8-byte-header layout: `Node` 48 bytes (AN-62's walk node is 48 too), `GameObject` 40 (last run's model was 40). The model should say where the weak count lives. |
| C2 | W4: `counter` and `m` | D1: "a box" when the function value is returned; table 1.2: "one allocation per captured variable instance"; D4: "its state is in its own block"; D1: "Its place ... is chosen when compiling and never changes what the program does". | `counter` and `m` live in the handler's own state block (32 bytes, one allocation per handler), not in a box each: only this one handler uses them after `make_handler` returns, so the place is unobservable. One box per variable would be three allocations per handler. |
| C3 | W4: `total` | D1's address case: "the compiler proves the function value ends before the variable does: it is kept only in locals, local lists, or objects that never leave the function, and every method it is handed to says what it changes"; box case: "the function value is returned, ...". | **A box** {count, type, value}, one count per handler state. The handlers are returned by `make_handler`, which D1 lists as a box case; and the handler itself hands new handlers to `d.register` on its parameter `d`, which D1's address case (an object that stays in the function) does not cover. Last run proved an address here (phase3b-combined.md 6.4); if 24 rules that the model allows that proof, the box goes (it is one more load per handler call and one count per handler). |
| C4 | W1: despawn and spawn inside the group loop | R8 (c), the owner's agreed limitation: "A list may not change while a `for` loop walks it ...; the change goes after the loop (`retain`, `extend`, `Commands`)"; AN-06: "last run's in-loop destroy with a cursor is not built". | The loop pushes each entity to despawn into a local `dead` list; after the loop, for each: `destroy`, `create`, two `add`s. Same entity ids (a counter, as the baseline and the Python twin), so the same checksum. No cursor. I wrote a small registry in Ember (two pools and the group, as the Python twin) rather than `std.ecs`'s `Commands`, whose `create` reuses freed indexes (AN-06) and so would not do the baseline's work. |
| C5 | W2: who owns the nodes | D7.1: a `set by` holder holds one-owner elements; R1: one owner. Python keeps every node in `nodes` and in its parent's `children`. | `children: Array[Node]` owns; `parent: weak Node set by children`; `nodes: Array[weak Node]` is an index of links (AN-62's `leaves: Array[weak Node]`). So every `nodes[...]` read is a `weak` read (DEAD test), and every node carries one weak count from `nodes`. |
| C6 | W2: the walk's copied link | AN-62's own spelling, `a = copied n.parent` then `while Some(p) = a:`; R11: "one test of the target's header per read"; R2/R10: the copy takes no weak count when nothing in its life can end the target. | No weak count on `a` (nothing in the walk can end a `Node`), but **the DEAD test on every hop** (w2.c line 110), since R11 says every read of a `weak` link tests. The model's text has no rule that removes it, though the target is provably alive (copied from a `set by` link, whose target lives while it holds the node, D7 rule 5, and no line of the walk can end a `Node`). AN-62's own C (`11-area-D/c/an62_walk_bench.c`) has no such test. Section 4 measures what it costs. |
| C7 | W2: `remove` | Python's `list.remove` keeps order; `Array.remove(i)` keeps order, `swap_remove(i)` does not ([STD-15]). | `old.children.remove(old.children.index_of(node).unwrap())`, order kept (memmove), as Python and last run's same-layout C. The baseline `bench/w2.c` swaps instead; the checksum does not depend on the order. |
| C8 | W3: `noalias` | R4: "Every `ref mut` parameter is compiled with noalias." `update(mut self)` is called on each element while `Turret.update` changes another object through `Some(ref mut t) = self.target`, which R8 (b) says may be the same object (both `GameObject`, one reached through a `weak` link). | No `restrict` on `self` in `update`: a noalias promise there would be false if a turret ever targeted itself (it never does in W3). The model should say that a `ref mut` view reached through a link of the parameter's own type cancels the noalias promise. |
| C9 | all: language checks | [TYP-8]: overflow "panics in every profile"; [COST-3]: bounds, overflow, zero-divisor checks are "conditionally elided" by range facts. Last run's models left overflow and zero-divisor checks out of the C. | **Kept** wherever no range fact proves them (tables below), since the model does not remove them and leaving them out would favour the model. Safe Rust at `-C opt-level=2` checks bounds and zero divisors but not overflow; plain C checks nothing. |
| C10 | all: the drops at the end of `main` | Ember drops `main`'s locals after the last line (`println`). | Not emitted: `h_report` stands where `println` is, so the timed work is the same as plain C's, which does not free at exit. |

## 3. The four programs: layout, memory-model work, every check kept

Line numbers are of the files as they are now. "Per step" means inside the workload's hot loop.

### 3.1 W1, the ECS (`w1.em`, `w1.c`)

Layout: the baseline's (`bench/w1_group.c`): one sparse set per component, the group <Position,
Velocity> owning slots [0, glen) of both pools, walked backwards. Ember's Array header; `sparse` grows by
one push per new id (as `w1.em` writes it) instead of jumping to a power of two from 1,024. Nothing is a
class object: no header, no count, no weak test. So the plain C in the same layout is the baseline
`w1_group.c -DLOCAL` (its loop keeps the three array pointers in registers, as the model's does);
`w1.c -DPLAIN` is the same work as the model (the despawns after the loop) without the checks.

Memory-model work: the `dead` list, one push per despawn (w1.c line 116), and the despawn and spawn after
the loop (lines 118-125) instead of inside it. Nothing per step besides plain C's work.

| tag | line | check | how often |
|---|---|---|---|
| K14 | 109 | loop versioning ([OPT-2]): `glen <= len` of `pos.dense`, `vel.dense`, `pos.ents`; then no check inside the loop | once per pass (100) |
| K15 | 116 | `dead.push`: growth path, capacity and out of memory (`ember_rt.h` 53, 55) | when the buffer grows |
| K1 | 43 | bounds `ents[i]`, `ents[j]` in `Pool.swap` | per swap (spawn and despawn) |
| K2 | 45 | bounds `dense[i]`, `dense[j]` in `Pool.swap` | per swap |
| K3 | 49 | bounds `sparse[b]`, `sparse[a]` in `Pool.swap` | per swap |
| K4 | 53 | bounds `sparse[e]` in `Pool.remove` | per remove |
| K6 | 56 | bounds `ents[last]`, `ents[i]` (also proves `pop` finds an element) | per remove |
| K7 | 59 | bounds `dense[last]`, `dense[i]` | per remove |
| K8 | 61 | bounds `sparse[moved]` | per remove |
| K9 | 68 | overflow, `next_id += 1` (u32) | per create |
| K10 | 72 | bounds `pos.sparse[e]`, `vel.sparse[e]` in `enter` | per group entry |
| K11 | 75 | overflow, `glen += 1` | per group entry |
| K5 | 83 | bounds `pos.sparse[e]` in `destroy` | per despawn |
| K12 | 85 | overflow, `glen -= 1` | per despawn |
| K13 | 86 | bounds `vel.sparse[e]` in `destroy` | per despawn |
| K16 | 130 | overflow, i64 `+` in the checksum | per entity, once |
| K17 | 130 | float -> int saturation and NaN ([TYP-6]; `ember_rt.h` 42-44) | per entity, once |
| K18 | 131 | overflow, u64 `+` | per entity, once |
| K19 | 133 | overflow, u64 `+` | once |
| (no tag) | 38-40 | `pool_add`'s three `PUSH`es: growth path (`ember_rt.h` 53, 55) | when a buffer grows |

Proved, so not emitted: `sparse[e]` in `has` and `add` (the comparison before it), every index in the
group loop (K14 instead), `len - 1` (a length is `>= 0`), `% 1000`, `// 1000`, `% 2`, `% 7`, `% 5` on the
loop's `i` (`>= 0`), `e % 100` (u32, constant divisor).

### 3.2 W2, the tree with parent links (`w2.em`, `w2.c`)

Layout: `Node` = {count (DEAD bit 31), type index, weak count, value, parent, children} = 48 bytes from a
slab (choice C1); `children` and `nodes` are Ember Arrays of pointers. The baseline (`bench/w2.c`) keeps
32-byte nodes in one array and swaps on removal. Plain C in the model's layout: `w2.c -DPLAIN` (48-byte
nodes, same slab, same Arrays, order-keeping remove; no weak counts, no DEAD tests, no checks).

Memory-model work and checks:

| tag | line | what | how often |
|---|---|---|---|
| M2 | 110 (`weak_read`, 53-58) | DEAD test of the walk's copied link `a` (choice C6) | **per hop of the walk** (per step) |
| K7 | 112 | overflow, `depth_sum += 1` | **per hop of the walk** (per step) |
| M2 | 104 | null and DEAD test, `if Some(n) = w` over `nodes` | per node per pass |
| M2 | 98, 117, 125 | null and DEAD test, `nodes[...]` in the build and the re-parent | per node built, per re-parent (twice) |
| M2 | 132 | null and DEAD test in the checksum loop | per node, once |
| M1 | 92, 95 (`weak_make`, 46-50) | `weak n` into `nodes`: weak count +1 | per node, once |
| K1 | 48 | weak count overflow | per node, once |
| M3 | 62, 69 | `set by`: `push` sets `parent`, `remove` clears it (one store each) | per build and re-parent |
| K2 | 61 | `children.push`: growth path (`ember_rt.h` 53, 55) | when a buffer grows |
| K3 | 65 | bounds `children.remove(k)` | per re-parent |
| K4 | 83 | `index_of(node).unwrap()`: panic if not found | per re-parent |
| K5 | 95 | `nodes.push`: growth path | when the buffer grows |
| K6 | 97 | bounds `nodes[rand() % i]` in the build (`len(nodes) == i` is not a fact the compiler keeps) | per node, once |
| K8 | 116 | bounds `nodes[i]` | per re-parent |
| K9 | 124 | bounds `nodes[rand() % i]` | per re-parent |
| K10 | 133 | overflow, `n.value as int * len(n.children)` | per node, once |
| K11 | 133 | overflow, `checksum +=` | per node, once |

No strong count changes anywhere (one owner per node; a move between `children` lists is a hand-over),
no count pair and no deferred-drop region (no line in the walk or the re-parent can end a `Node`, R10
point 1). Proved, not emitted: the zero divisor of `rand() % i` (`i >= 1`), `(i + p) % 100` (both in known
ranges), `% 1000`. `node_drop` (lines 74-79) is the drop of a node taken out of its parent and not
pushed anywhere: never reached in W2, emitted because the model would emit it.

### 3.3 W3, game objects referring to each other (`w3.em`, `w3.c`)

Layout: `GameObject` = {count (DEAD bit 31), type index, weak count, x, y, health, id, target} = 40 bytes
from a slab (choice C1); the type index picks `update` from the method table, as plain C's kind table
does. The baseline (`bench/w3.c`) has 56-byte objects with watcher lists. Plain C in the model's layout:
`w3.c -DPLAIN`; it keeps the weak count and the DEAD test (in this layout plain C needs some way to know
its target ended), and drops only the checks.

| tag | line | what | how often |
|---|---|---|---|
| M2 | 71, 81 (`weak_read`, 44-46) | null and DEAD test, `if Some(t) = self.target` | **per Ship and Turret update** (per step) |
| K3, K4 | 73 | overflow, `t.x - self.x`, `self.x += ...` (i32) | per Ship update |
| K5, K6 | 74 | the same for `y` | per Ship update |
| K7 | 78 | overflow, `self.x + 1`; then floor `%` by a positive constant | per Rock update |
| K8 | 82 | overflow, the two `-` | per Turret update with a live target |
| K2 | 67 | overflow, `abs` of the minimum `i32` (two calls) | per Turret update with a live target |
| K9 | 82 | overflow, `abs(...) + abs(...)` | per Turret update with a live target |
| K10 | 83 | overflow, `t.health -= 1` | per hit |
| M3 | 122 (`obj_end`, 57-63) | a dead object's one owner ends: DEAD set, its own link gives back its weak count, block freed or kept for its weak links | per death |
| M1 | 109, 131 (`weak_store`, 50-56) | `o.target = weak x`: weak count +1 (and -1 on the old target, none here) | per object made |
| K1 | 51 | weak count overflow | per object made |
| K11 | 108 | bounds `objects[rand() % i]` in the build | per object, once |
| K12, K15 | 111, 132 | `objects.push`: growth path | when the buffer grows |
| K13 | 126 | overflow, `destroyed += dead` | per frame |
| K14 | 129 | overflow, `next_id += 1` | per object made |
| K16 | 135 | overflow, `destroyed * 1_000_000` | once |
| K17, K18 | 138 | overflow, `o.x + o.y + o.health` (i32) and `total +=` | per object, once |

Floor `%` (language meaning, not a check): 78 (`% 1000` on a field), 91 and 93 (`id % 50`, `id % 3`;
`id` is a parameter of `make`, its sign unknown there). Proved, not emitted: `rand() % 1000` (`rand()`
is in [0, 2^31)), the zero divisor and the bounds of `objects[rand() % len(objects)]` (inside
`if len(objects) > 0`, floor `%` by a positive length is below it), `before - len(objects)`. No count on
`t`: no line in its life can end a `GameObject` (R10 point 1); the update loop holds nothing.

### 3.4 W4, callbacks that remove themselves (`w4.em`, `w4.c`)

Layout: a stored handler is {code, state}, 16 bytes, in Ember's Array; each state is a 32-byte counted
block from a slab {count, type, counter, m, total's box} (choice C2); `total` is a 16-byte box
{count, type, value} (choice C3); the `Dispatcher` is a class object in its own block. The baseline
(`bench/w4.c`) mallocs a 24-byte environment per handler and holds `&total`. Plain C in the model's
layout: `w4.c -DPLAIN` (32-byte states from the same slab, the same Array; `total` by address, no counts,
the handler frees its own state as its last act, as the baseline does; no checks).

| tag | line | what | how often |
|---|---|---|---|
| K7 | 120 | zero divisor of `k % len(self.handlers)` | **per dispatch** (per step) |
| M1 | 122 | {code, state} copied out of the list for the call; nothing counted | per dispatch |
| K4, K5 | 98 | overflow, `v * m` and `counter += ...` | per handler call |
| K6 | 99 | overflow, `total += 1` (through the box) | per handler call |
| M2 | 102, 108 | the handler holds a count on its own state across `d.unregister_current()` and releases it after its last use | per self-removal (once per 100,000 events) |
| K1 | 58 | count overflow, in `count_add` (used by M2 and M4) | per self-removal, per handler made |
| M3 | 75 (`state_release`, 61-65) | the removed handler is dropped: its state's count -1; at zero its count on `total`'s box -1 and the block goes back to the slab | per self-removal |
| M4 | 89 | a new handler: one state block, `total`'s box count +1 | per handler made |
| K2 | 70 | bounds `remove(self.current)` | per self-removal |
| K3 | 81 | `handlers.push`: growth path | when the buffer grows |
| K8 | 143 | overflow, `s += ...` | per handler, once |
| K9 | 149 | overflow, `total + s` | once |

Floor `%`: 100 (`k % 100_000` in the handler), 105 (`k % 17`), 121 (`k % len`), 123 (`k % 1000`); `k` is
a parameter there. Proved, not emitted: the bounds of `handlers[current]` (current is `k % len`, just
stored, below `len`), the bounds of `d.handlers[i]` in the final `while` (the loop's test), `i % 13 + 1`.

### 3.5 Counts of checks kept

Counting each tag that is a check (a `K` tag, plus the DEAD test `M2` where there is one):

| workload | language checks (K) | memory-model tests | total | inside the hot loop, per step |
|---|---|---|---|---|
| W1 | 19 | 0 | 19 | none (K14 once per pass) |
| W2 | 11 | 1 (M2, at 6 sites) | 12 | M2 (DEAD test) and K7 (overflow), per hop |
| W3 | 18 | 1 (M2, at 2 sites) | 19 | M2 per Ship/Turret; K2-K10 per update |
| W4 | 9 | 0 | 9 | K7 (zero divisor) per dispatch; K4-K6 per call |

## 4. One finding for 24, 26 and the lead: W2's per-hop DEAD test

Single runs, not timings (the timer's numbers decide): `model-w2` ran 938.867 to 958.404 ms and
`plain-same-w2` 746.492 to 814.427 ms over three rounds (`target/redesign3/23-c-writer/diag_w2_output.txt`).
A diagnostic build that only removes the DEAD test of the walk's copied link (`w2.c -DHOP_PROVED`, lines
107-111; **not the model**, overflow checks kept) ran 677.731 to 706.611 ms in the same rounds. So in
these runs the per-hop DEAD test of choice C6 accounts for the whole gap between the model and plain C in
its layout. It can be saved without a word: a rule that a `weak` copy of a `set by` link, over a life in
which no line can end its target's type (R10 point 1), reads its target with no test, because the target
was alive when copied (D7 rule 5) and nothing can end it since. AN-62's walk already assumes this.

Two smaller notes for the lead (not memory errors):
- W2: `np.children.push(moved)`, with `np` reached through `nodes` (a `weak` link), could put `moved`
  into its own subtree in another program; the compiler cannot see it (AN-64's "handing a node into its
  own subtree ... is refused" covers only a view derived from the node). It would be a leak, not a
  memory error. In W2 the new parent always has a smaller number, so it never happens.
- W3: choice C8 (the noalias promise of `mut self` against a `ref mut` view reached through `weak`).

## 5. Build and run output (each built once with clang -O2, run once)

Build: `"/c/Program Files/LLVM/bin/clang.exe" -O2 -I /c/Users/ism19/Code/ember/target/redesign/bench
wN.c -o bin/model_wN.exe` and the same with `-DPLAIN` into `bin/plain_same_wN.exe`.
`target/redesign3/23-c-writer/run_output.txt`:

```text
model-w1 ms=83.825 peak_kb=36724 checksum=2111831520
exit=0
plain-same-w1 ms=80.913 peak_kb=36440 checksum=2111831520
exit=0
model-w2 ms=1013.914 peak_kb=84972 checksum=647034814
exit=0
plain-same-w2 ms=778.406 peak_kb=84948 checksum=647034814
exit=0
model-w3 ms=431.161 peak_kb=9212 checksum=9288233970
exit=0
plain-same-w3 ms=444.403 peak_kb=9196 checksum=9288233970
exit=0
model-w4 ms=26.146 peak_kb=4812 checksum=34946993000
exit=0
plain-same-w4 ms=21.809 peak_kb=4792 checksum=34946993000
exit=0
```

The plain C baselines, built from `target/redesign2/bench/` the same way (`w1_group.c` with and without
`-DLOCAL`), `target/redesign3/23-c-writer/baseline_output.txt`:

```text
plain-w1-group ms=85.761 peak_kb=39696 checksum=2111831520
plain-w1-group-local ms=83.158 peak_kb=39692 checksum=2111831520
plain-w2 ms=609.366 peak_kb=55512 checksum=647034814
plain-w3 ms=444.883 peak_kb=10340 checksum=9288233970
plain-w4 ms=23.291 peak_kb=4672 checksum=34946993000
```

Every checksum equals plain C's for the same workload, and the known checksums of
`phase3b-measurements.md` section 3.

AddressSanitizer (`clang -O1 -g -fsanitize=address`, the ASan DLL directory first on `PATH`), the four
models, `target/redesign3/23-c-writer/asan_output.txt`: no report, the same checksums:

```text
model-w1 ms=213.508 peak_kb=96576 checksum=2111831520
exit=0
model-w2 ms=2678.010 peak_kb=115240 checksum=647034814
exit=0
model-w3 ms=646.544 peak_kb=15888 checksum=9288233970
exit=0
model-w4 ms=61.064 peak_kb=9528 checksum=34946993000
exit=0
```

(The `w2.c` lines 107-111 `#ifdef HOP_PROVED` and lines 135-136 were added after `run_output.txt`;
the default build's code is unchanged by them, and the rebuilt `model-w2` printed the same checksum,
`diag_w2_output.txt`.)

## 6. What today's checker says about the NEW programs

The NEW syntax is not today's Ember; `target/redesign3/bin/ember.exe check wN.em` refuses each at its
first new word (`changes`, `weak`). First lines of each, `target/redesign3/23-c-writer/ember_check_output.txt`:

```text
=== w1.em
error[E0110]: `add` is declared without a body
  --> w1.em:29:8
   |
29 |     fn add(self, e: u32, value: T) changes sparse, ents, dense:
...
(first lines of 83 errors)
exit=1
=== w2.em
error[E1010]: cannot find type `weak` in this scope
  --> w2.em:13:13
...
(first lines of 48 errors)
exit=1
=== w3.em
error[E1010]: cannot find type `weak` in this scope
  --> w3.em:16:13
...
(first lines of 58 errors)
exit=1
=== w4.em
error[E0110]: `register` is declared without a body
  --> w4.em:14:8
...
(first lines of 26 errors)
exit=1
```

## 7. Files

All under `C:\Users\ism19\Code\ember\target\redesign3\23-c-writer\`:

| file | what |
|---|---|
| `w1.em` .. `w4.em` | the four workloads in the model's NEW syntax |
| `ember_rt.h` | the runtime pieces they share: `KEEP`, `ADD`/`SUB`/`MUL` (overflow), `FMOD` (floor `%`), float -> int, Ember's Array, the slab; `-DPLAIN` switches the checks off |
| `w1.c` .. `w4.c` | the C the model compiles each to; `-DPLAIN` = plain C in the model's layout; `w2.c -DHOP_PROVED` = the diagnostic of section 4 (not the model) |
| `bin/model_wN.exe`, `bin/plain_same_wN.exe`, `bin/diag_w2_hop_proved.exe`, `bin/base_*.exe` | the builds (the baselines from `target/redesign2/bench/`) |
| `run_output.txt`, `baseline_output.txt`, `asan_output.txt`, `diag_w2_output.txt`, `ember_check_output.txt` | the outputs pasted above |

For the timer: the model's programs are `wN.c` (default build); the plain C in the model's layout is
`wN.c -DPLAIN`; for W1 the baseline `w1_group.c -DLOCAL` is already in the model's layout.
