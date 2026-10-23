# Phase 3: measurements (plain C against D1, D2, D3)

Date: 2026-10-24. Machine: the owner's laptop, on mains power (`(Get-CimInstance Win32_Battery).BatteryStatus`
returned 2 before the first build and before every one of the 5 rounds; the run script aborts otherwise).
Compiler: `clang version 22.1.8` (`"C:/Program Files/LLVM/bin/clang.exe" --version`).

## 1. Method

- Plain C baselines written for this phase: `target/redesign/bench/plain/w1.c` .. `w4.c` (raw pointers,
  no checks, `../harness.h`, report `plain-w1` .. `plain-w4`).
- Every program (plain, D1, D2, D3; w1..w4) was built with the same command:
  `"C:/Program Files/LLVM/bin/clang.exe" -O2 <file> -o <exe>`. All 16 built; no warnings or errors were printed.
- Runner: `target/redesign/bench/runs/run.ps1`. 5 rounds; in each round every program runs once, one at a
  time (`Start-Process ... -PassThru`, then `$p.ProcessorAffinity = 0xC03C03`, then `$p.WaitForExit()`).
  Raw lines: `target/redesign/bench/runs/results.txt` (95 runs, all with empty stderr).
- The tables take the median of the 5 runs for `ms` and, separately, for `peak_kb` (the harness's
  peak working set). Ratios are model median / plain median.
- Not covered: the affinity is set right after the process starts, so the first few milliseconds run
  unpinned (the method the task gives). Peak working set is what the harness reports, not committed memory.

## 2. Results (from `runs/results.txt`)

| workload | model | median ms | ratio ms | median peak KB | ratio mem | checksum ok |
|---|---|---|---|---|---|---|
| W1 ECS | plain | 42.362 | 1.00 | 17,120 | 1.00 | yes (2111831520) |
| W1 ECS | D1 | 129.089 | 3.05 | 37,424 | 2.19 | yes |
| W1 ECS | D2 | 223.037 | 5.27 | 41,848 | 2.44 | yes |
| W1 ECS | D3 | 167.447 | 3.95 | 37,128 | 2.17 | yes |
| W2 tree | plain | 535.624 | 1.00 | 54,940 | 1.00 | yes (647034814) |
| W2 tree | D1 | 1053.126 | 1.97 | 92,928 | 1.69 | yes |
| W2 tree | D2 | 912.371 | 1.70 | 76,572 | 1.39 | yes |
| W2 tree | D3 | 604.246 | 1.13 | 76,580 | 1.39 | yes |
| W3 game objects | plain | 435.462 | 1.00 | 9,756 | 1.00 | yes (9288233970) |
| W3 game objects | D1 | 506.416 | 1.16 | 11,160 | 1.14 | yes |
| W3 game objects | D2 | 458.012 | 1.05 | 9,252 | 0.95 | yes |
| W3 game objects | D3 | 424.518 | 0.97 | 8,216 | 0.84 | yes |
| W4 callbacks | plain | 23.145 | 1.00 | 4,076 | 1.00 | yes (34946993000) |
| W4 callbacks | D1 | 33.008 | 1.43 | 4,532 | 1.11 | yes |
| W4 callbacks | D2 | 25.605 | 1.11 | 4,352 | 1.07 | yes |
| W4 callbacks | D3 | 24.719 | 1.07 | 4,280 | 1.05 | yes |

Checksums: every run of every model of a workload printed the same checksum as plain C (all 5 runs,
all 4 models). No mismatch. The values also equal the design reports' and the breakers' tables
(`phase3-break-D1.md` section 3.1, `phase3-break-D2.md` section 3).

Spread (max minus min of the 5 runs, over min; `runs/results.txt`): W1 is the noisiest (D3 15%, D2 12%,
D1 8.5%, plain 7.6%); W2 6 to 8%; W4 2 to 6%; W3 at most 2.3%. So on W1 and W2 a difference under
about 10% between two rows is not meaningful; on W3 and W4 about 3%.

## 3. The plain baselines, and what each ratio contains

### W1: the plain baseline uses a different layout from the models

`plain/w1.c` keeps the entities that have a Velocity in one dense array and the Position-only
entities in a second array that never changes (archetype tables, the layout flecs-style ECSs use):

```c
typedef struct { uint32_t id; float x, y, dx, dy; } Mover;
for (size_t j = nmov; j-- > 0;) {          /* backwards: a swap-removed slot is already visited */
    Mover *m = &mov[j];
    m->x += m->dx; m->y += m->dy;
    if (m->id % 100 == s) { *m = mov[--nmov]; mov[nmov++] = (Mover){next++, 0, 0, 1, 1}; }
}
```

The three models use EnTT's layout (one sparse set per component, a view that probes the other pool).
So the W1 ratios above mix two things: the layout and the model's own costs. To separate them I wrote
`plain/w1_sparse.c`: the EnTT layout in plain C with no checks (walk the Velocity pool backwards,
probe Position's `sparse`, swap-remove from both pools, append spawns). 5 runs, pinned, alternating with
`plain/w1` and `D1/w1` (`runs/results_w1_sparse.txt`): median 96.567 ms, 39,176 KB, same checksum.

| W1 against | D1 | D2 | D3 |
|---|---|---|---|
| plain archetype layout (`w1.c`, 42.362 ms, 17,120 KB) | 3.05x time, 2.19x mem | 5.27x, 2.44x | 3.95x, 2.17x |
| plain EnTT layout (`w1_sparse.c`, 96.567 ms, 39,176 KB) | 1.34x time, 0.96x mem | 2.31x, 1.07x | 1.73x, 0.95x |

Reading: of D1's 3.05x, the layout alone accounts for 2.28x (96.567 / 42.362); the model's own
checks and counts are the remaining 1.34x. The memory difference is all layout: plain C in the EnTT
layout uses about as much as the models (my `w1_sparse.c` grows `sparse` by doubling, so its peak is
not the minimum for that layout). A language whose ECS library uses the EnTT layout is compared fairly
with the second row; the first row is what a C programmer gets by choosing the layout for this job.

### W3: the first plain baseline was slower than every model, so it was replaced

| plain W3 version (file) | median ms | ratio to final plain | median peak KB |
|---|---|---|---|
| malloc per object, sweep all live objects each frame to clear dead targets (`w3_sweep.c`) | 706.660 | 1.62 | 10,640 |
| malloc per object, observer list per object (`w3_malloc.c`) | 548.448 | 1.26 | 12,220 |
| pool + free list, observer list, vtable pointer per object (`w3_pool.c`) | 453.440 | 1.04 | 10,520 |
| pool + free list, observer list, table indexed by kind (`w3.c`, the baseline) | 435.462 | 1.00 | 9,756 |

All four in `runs/results.txt`, same 5 rounds, same checksum. The baseline keeps a raw target pointer
and, per object, an intrusive list of the objects that target it, so a destroy clears exactly its
watchers:

```c
static void destroy(Obj *o) {
    for (Obj *w = o->watchers, *n; w; w = n) { n = w->wnext; w->target = NULL; }
    /* unlink o from its own target's watcher list, then return o to the pool */
}
```

D3 is still 3% faster than this baseline (424.518 against 435.462 ms) and uses 16% less memory.
Measured facts: plain's object is 56 bytes (8 id, 4 x 4 ints, 4 pointers); D3's is 40 bytes
(`_Static_assert(sizeof(Obj) == 40)` in `D3/w3.c`), with one generation compare per target read
instead of three link pointers. That the 16-byte size difference is the cause of the 3% is
UNVERIFIED (not isolated). W3's spread is under 2.5%, so D3 (0.97x) is measurably a little faster
than this plain C, D2 (1.05x) a little slower, and D1 16% slower.

### W2 and W4

`plain/w2.c`: all nodes in one `calloc`, `Node *parent`, a growable `Node **` child array per node,
removal by search and swap with the last child. `plain/w4.c`: a closure is `{function pointer,
environment pointer}`; each environment is `malloc`'d (`counter`, `m`, `int64_t *total`); the
dispatcher records the slot it is calling, and the handler removes itself with `memmove`, appends its
replacement and frees its own environment as its last act. Plain is the fastest program on both.

## 4. What the breakers' findings mean for each model's numbers

### D1 (`phase3-break-D1.md`)

- The breaker found the C models faithful (every check the design names is present, section 3.2).
- **M1 (major), W1: the number is a best case.** The model assumes the view generator is inlined into
  the loop, so the two pool projections are fetched again about 1.7 million times; without that
  inlining (not among the design's listed analyses) D1-6 needs them fetched on every resume, about
  50 million times. UNMEASURED by the breaker and by me, so D1's W1 3.05x (1.34x against the EnTT
  layout) holds only if generator inlining becomes a required analysis.
- M2 (uniqueness tests skipped after `destroy`) touches 856,653 spawns; minor.
- **The safety fixes are not in these numbers.** D1 as written is not memory-safe (B1 to B9). The
  breaker's cost table (section 4) puts most fixes at zero or one compare, but B6 (refetch after calls
  through `dyn`, virtual methods or closures in functions with `mut` parameters) would add 856,653
  refetches to W1's `destroy`, and M5 says no workload has the program shapes where the fixes cost
  most. So D1's four ratios are lower bounds for a safe D1.

### D2 (`phase3-break-D2.md`)

The breaker isolated three shortcuts and wrote the faithful versions. I rebuilt and ran them with
the same command and method (5 runs, pinned, `runs/results_d2_variants.txt`; the `_orig.c` files differ
from `bench/D2/*.c` only in the `#include` path):

| workload | D2 model as written | faithful version | change | ratio to plain, faithful |
|---|---|---|---|---|
| W2, child list as its own object (`w2_listobj.c`) | 931.014 ms, 76,552 KB | 776.197 ms, 92,252 KB | -17% time, +21% mem | 1.45x time, 1.68x mem |
| W3, `objects.remove(o)` as Python's search (`w3_scan.c`) | 465.481 ms | 502.842 ms, 9,280 KB | +8% time | 1.15x time, 0.95x mem |
| W4, handler list as its own object (`w4_listobj.c`) | 25.710 ms | 32.357 ms, 4,356 KB | +26% time | 1.40x time, 1.07x mem |

These match the breaker's figures (+26% on W4, +5% to +9% on W3, -16% time and +20% memory on W2).
So D2's honest numbers are W2 1.45x / 1.68x (not 1.70x / 1.39x), W3 1.15x, W4 1.40x (not 1.11x).
W1 is not affected by these (the breaker found only a sparse-list sizing difference of "up to a few
MB", unmeasured).

The two critical holes (H1, references to stack values; H2, references across threads) have no
priced fix, and the breaker says each way to close H1 gives up one goal; none of W1 to W4 contains
those shapes, so their cost cannot show here. H7's generation-checked `destroy` adds one compare per
destroy (rare path in all four workloads).

### D3 (`phase3-break-D3.md`)

- **W1: the model is the best case of a mechanism the design does not define.** One pin counter and
  one tombstone list for both of a pool's arrays, and a settle that repairs `sparse`, which an
  Ember-written `Pool[T]` could not express with the stated rules (break section 3.1). The cost of the
  real mechanism is UNVERIFIED. D3's W1 3.95x (1.73x against the EnTT layout) is therefore a lower bound.
- **W2: only the proven case is modelled.** 1.13x needs the whole-program fact that nothing ever
  pops from `nodes`; one `nodes.pop()` anywhere, separate compilation or hot reload adds a generation
  per node and a check per step of the parent walk (break 2.8, 3.2). Break 2.1 shows that swapping two
  lines of the design's own W2 program turns it into a panic. The fallback was not modelled, so W2's
  number for a program outside the proven case is UNMEASURED.
- **W3: the same O(1) remove shortcut as D2.** The D2 breaker points out that `D3/w3.c` also removes
  the loop's own element at the cursor (line 143 there) where the program says `objects.remove(o)`;
  the D3 breaker calls the model faithful and notes only the reused `destroyed` buffer. If D3 must
  search like D2's `w3_scan.c`, expect a similar +8% (UNVERIFIED for D3; measured for D2 only), which
  would take D3 from 0.97x to about 1.05x of plain.
- **W4: the 3-word inline closure state** assumes the compiler merges `make_handler`'s capture of
  `total`; without it, one heap allocation per handler and one more dependent load per call (break
  3.4, cost UNVERIFIED). D2's `w4_listobj.c` shows one extra dependent load per call cost +26% on this
  workload, so this is not negligible.
- **The safety fixes are not in these numbers.** D3 as written is not memory-safe (1.1 to 1.9). The fix
  for 1.1, a generation check on every non-home reference that lives across a destroying call, does
  not touch W3's update loop if the design's effect fact (no update destroys) holds; UNVERIFIED.

### Every model

Integer overflow checks (`[TYP-8]`) and zero-divisor checks are left out of all three models (D3 break
3.5). Plain C has none either, so against plain C every design is flattered by the same work in the hot
loops (W2's `depth_sum += 1`, W4's `counter += v * m` and `total += 1`). Teardown is skipped in all
models and in plain C alike.

## 5. Summary in one table (time ratio to plain; memory ratio in brackets)

| | W1 vs archetype C | W1 vs EnTT-layout C | W2 | W3 | W4 |
|---|---|---|---|---|---|
| D1 as modelled | 3.05 (2.19) | 1.34 (0.96) | 1.97 (1.69) | 1.16 (1.14) | 1.43 (1.11) |
| D2 as modelled | 5.27 (2.44) | 2.31 (1.07) | 1.70 (1.39) | 1.05 (0.95) | 1.11 (1.07) |
| D2 with the breaker's faithful versions | same | same | 1.45 (1.68) | 1.15 (0.95) | 1.40 (1.07) |
| D3 as modelled | 3.95 (2.17) | 1.73 (0.95) | 1.13 (1.39) | 0.97 (0.84) | 1.07 (1.05) |

Every model row is a best case: none includes the fixes for its breaker's memory-safety holes, and
D1 W1, D3 W1, D3 W2 and D3 W4 rest on compiler analyses the designs do not yet define.

## Files

- Plain baselines: `target/redesign/bench/plain/w1.c`, `w2.c`, `w3.c`, `w4.c`.
- Plain variants (measured, not baselines): `plain/w1_sparse.c`, `plain/w3_sweep.c`, `plain/w3_malloc.c`, `plain/w3_pool.c`.
- Runner: `target/redesign/bench/runs/run.ps1`. Raw results: `runs/results.txt`,
  `runs/results_w1_sparse.txt`, `runs/results_d2_variants.txt` (the D2 variants were built from
  `target/redesign/break-D2/*.c` into `runs/d2v_*.exe`).
- A first full run, made before the plain W3 baseline was replaced, was overwritten by the run above;
  its numbers for the unchanged programs were within the spread in section 2 (not kept).
