# Phase 3b: the final measurement

Written 2026-10-24 for the owner, under the job in [JOB.md](JOB.md). It times the combined memory model
([phase3b-combined.md](phase3b-combined.md), called "the combined report"), the amended W3 from its attack
([phase3b-combined-attack.md](phase3b-combined-attack.md), "the attack"), and the three salvaged designs
([phase3b-salvage.md](phase3b-salvage.md), "the salvage report"), against plain C and safe Rust doing the
same work, on the four workloads.

**All timings are PROVISIONAL**: other agents were running on the laptop while I timed. A conclusion that
rests on a difference under 15% is marked as such.

---

## 0. The short version

The combined model, in the C its compiler would emit, against plain C in the same data layout (my runs,
medians of 5, two or three batches each):

| Workload | Combined model, time against plain C in the same layout | Against safe Rust | Memory against plain C in the same layout |
|---|---|---|---|
| W1, ECS (EnTT owning group) | **0.99 and 1.04** | 0.84 and 0.78 | 1.00 |
| W2, tree with parent links | **1.01, 1.04, 1.03** | 0.67, 0.62, 0.68 | 1.00 |
| W3, game objects (as amended by the attack) | **0.96 and 0.94** (no separate same-layout C; against plain C) | 0.74 and 0.72 | 0.94 |
| W4, callbacks | **0.98 and 1.02** | 0.67 and 0.67 | 1.05 |

In plain words:

1. **Speed: on par with C on all four workloads**, and faster than safe Rust on all four when Rust does the
   work the way the workload says. Every gap is under 5%, which is inside the noise of these runs.
2. **Memory: on par with C written in the same layout, but W2 needs 1.40x the memory of plain C's tightest
   layout** (76,724 KB against 54,940 KB). That gap is the layout (Ember's list header, one heap block per
   node, a list of handles to the nodes), not the memory model: plain C in the same layout uses the same
   memory (76,664 KB). It is still a cost of the language as specified, and safe Rust's W2 uses 0.95x.
3. **Every program printed the known checksum on every run**: 33 programs, 390 timed runs, no mismatch, every
   exit code 0 (`runs/SUMMARY.txt`, column "checksum ok").
4. **One baseline was weaker than it should be, and I fixed it.** The salvage's plain C for W1 reloads its
   three array pointers from global variables on every step of the loop. With the pointers kept in local
   variables it is 2% and 13% faster (section 2.1). The combined model is compared with that faster version.
5. **The three salvaged designs, re-measured**: their numbers from the salvage report hold within noise,
   except that safe Rust's W2 is slower in my runs (1.74x to 1.94x of plain C, the salvage had 1.48x).
   Two W2 results depend on a whole-program fact (no node is ever removed from `nodes`): without it,
   "checked pointers" is 1.63x to 1.83x and "today's rules, but never an error" 1.92x to 1.98x of plain C in
   the same layout.
6. **The `weak` decision the combined report left for me** (its section 7, item 1): the model's W3 with
   `weak` as a test on read is 1.13x the time of W3 with `weak` cleared by registration, in both batches;
   under the 15% line, so the combined report's choice stands. With the attack's amendment (weak counts
   beside the objects) it is 1.16x and 1.14x: one batch over the line, one under. That is too close for
   these runs to decide; registration also needs 1.30x the memory (section 4).

**The owner's question, "what about the agents that already ran with that problem?"**: the salvage
report's timings were taken with a runner that starts each program through `cmd /c start`
(`salvage/runs/ab.ps1` line 12). I re-ran every salvaged program with a runner that never uses `cmd`
(section 1.2). The salvage numbers are reproduced within noise (section 5), so nothing that report concluded
came from that runner. My own runs opened no window: every program was started directly by PowerShell, and
every C program calls `SetErrorMode` first.

---

## 1. How this was measured

### 1.1 Words used

- **Plain C**: C written for speed with no safety checks. **Safe Rust**: Rust with no `unsafe` in the
  workload (the one `unsafe` call is in the harness, section 1.3).
- **Same layout**: plain C that keeps the data the way the model does (same object sizes, same list header,
  same allocator), so the ratio shows the memory model's own cost, not the data layout's.
- **Median**: the middle of the 5 times. **Spread**: (slowest - fastest) / fastest over the 5 runs.
- **Peak memory**: the process's peak working set (the most physical memory it held at once), as the
  harness prints it in `peak_kb`.
- **Batch**: one run of 5 rounds; in each round every program of the workload runs once, one at a time, in
  forward order in odd rounds and reverse order in even rounds (so two programs being compared alternate).
  Batch b starts from the other end of the list.
- **EnTT owning group**: EnTT keeps one array set per component type (a "pool"); an owning group of
  Position and Velocity keeps the entities that have both at the front of both pools, in the same order, so
  the loop reads both arrays straight through. **Archetype layout**: each entity's components side by side
  in one table (plain C only, for reference).
- **PROVISIONAL**: measured while other agents ran. **UNVERIFIED**: believed, not checked.

### 1.2 Method

- Machine: the owner's laptop. `(Get-CimInstance Win32_Battery).BatteryStatus` returned 2 (mains) before I
  started and before every round of every batch (`bench/run.ps1` checks and aborts otherwise; no batch
  aborted).
- Compilers: `clang version 22.1.8`, `-O2`; `rustc 1.98.1 (48a229cea 2026-09-01)`,
  `-C opt-level=2 --edition 2021`. Every build command is in `bench/build.sh`.
- Runner: `bench/run.ps1`. It sets its own processor affinity to the performance cores (mask 0xC03C03)
  and then starts each program directly (`& $exe`), so each program inherits the mask from its first
  instruction. No `cmd`, no `start`, no new window, one program at a time.
- Commands (PowerShell, in `target/redesign2/bench/bin/`):

  ```text
  powershell -NoProfile -ExecutionPolicy Bypass -File ..\run.ps1 -Out ..\runs\w1.txt -Rounds 5 -Exes plain_w1_group.exe,rust_w1.exe,combined_w1.exe,plain_w1_group_local.exe,d1_w1.exe,d2_w1.exe,d3_w1.exe,plain_w1_arch.exe
  powershell -NoProfile -ExecutionPolicy Bypass -File ..\run.ps1 -Out ..\runs\w2.txt -Rounds 5 -Exes plain_w2.exe,rust_w2.exe,combined_w2.exe,plain_w2_same.exe,plain_w2_emberlist.exe,d1_w2.exe,d2_w2.exe,d2_w2_checked.exe,d3_w2.exe,d3_w2_unproven.exe
  powershell -NoProfile -ExecutionPolicy Bypass -File ..\run.ps1 -Out ..\runs\w3.txt -Rounds 5 -Exes plain_w3.exe,rust_w3.exe,combined_w3.exe,combined_w3_beside.exe,combined_w3_registration.exe,d1_w3.exe,d2_w3.exe,d3_w3.exe
  powershell -NoProfile -ExecutionPolicy Bypass -File ..\run.ps1 -Out ..\runs\w4.txt -Rounds 5 -Exes plain_w4.exe,rust_w4.exe,combined_w4.exe,plain_w4_same.exe,d1_w4.exe,d2_w4.exe,d3_w4.exe,rust_w4_return.exe
  ```

  and the same lists reversed into `w1b.txt` .. `w4b.txt`; W2 a third time into `w2c.txt`. Medians:
  `C:\Python314\python.exe -I medians.py [b|c]`, collected in `runs/SUMMARY.txt`.
- **Noise.** W3 and W4 spreads were mostly 2% to 6%. W1 spreads were 6% to 23%. W2 was badly disturbed in
  batch a (spreads 28% to 52%), less in batches b and c (5% to 26%). W2 walks a million-node tree by
  following parent links, so it waits on memory and is the most sensitive to other programs' memory
  traffic. I ran W2 three times; the three batches agree on every ratio to within 0.10, and the model's
  ratio to its same-layout C to within 0.03.
- The first two W1 batches, before I added the faster plain C (section 2.1), are kept as
  `runs/w1_before_local.txt` and `runs/w1b_before_local.txt`; they are not used in the tables.

### 1.3 The programs

All under `C:/Users/ism19/Code/ember/target/redesign2/`:

| Program (binary in `bench/bin/`) | Source | What it is |
|---|---|---|
| `plain_w1_group` | `bench/w1_group.c` (the salvage's `plain/w1_group.c`, harness include changed) | plain C, W1 in the EnTT owning group: the baseline the task names |
| `plain_w1_group_local` | `bench/w1_group.c -DLOCAL` | the same, with the loop's array pointers in local variables (section 2.1) |
| `plain_w1_arch` | `bench/w1_arch.c` (phase 3's `plain/w1.c`) | plain C, W1 in the archetype layout, for reference |
| `plain_w2`, `plain_w3`, `plain_w4` | `bench/w2.c`, `w3.c`, `w4.c` (phase 3's `plain/`, include changed) | plain C baselines |
| `plain_w2_same` | `graphs-callbacks/bench/w2/w2_plain_malloc.c -DSLAB -DARRAY24` | plain C in the combined model's W2 layout (combined report section 7) |
| `plain_w2_emberlist` | `salvage/plain/w2_emberlist.c` | plain C with Ember's list header only (the salvage's same-layout W2) |
| `plain_w4_same` | `graphs-callbacks-attack2/bench/w4_plain_same.c -DSLAB` | plain C doing exactly the combined model's W4 work with the same slab (combined report section 7) |
| `rust_w1` .. `rust_w3` | `bench/rust_w1.rs` .. `rust_w3.rs` (the salvage's `rust/w1_group.rs`, `w2.rs`, `w3.rs`) | safe Rust baselines |
| `rust_w4` | `bench/rust_w4.rs` (the graphs group's `w4/w4_rust.rs`) | safe Rust W4 with the workload's shape: the handler unregisters itself from inside its own call |
| `rust_w4_return` | `bench/rust_w4_return.rs` (the salvage's `rust/w4.rs`) | safe Rust W4 shaped differently: the handler returns "remove me", for reference |
| `combined_w1` .. `combined_w4` | `combined/w1.c` .. `w4.c` | the combined model |
| `combined_w3_beside` | `combined-attack/w3_beside.c` | W3 as the attack amends the model (weak counts beside the objects) |
| `combined_w3_registration` | `combined/w3_registration.c` | W3 with `weak` cleared by registration (the alternative the combined report left for me) |
| `d1_wN`, `d2_wN`, `d3_wN` | `salvage/d1/`, `d2/`, `d3/` (W1 from `w1_group.c`) | the salvaged "copy instead of share", "checked pointers" and "today's rules, but never an error", default builds |
| `d2_w2_checked`, `d3_w2_unproven` | `salvage/d2/w2.c -DCHECKED`, `salvage/d3/w2.c -DUNPROVEN` | the same two designs' W2 without the whole-program fact their default builds assume |

The salvage report names all three designs "partly saved" and the default builds as the saved versions;
its attribution builds (`-DD1_PROVEN`, `-DNOINLINE`, `-DMALLOCBOX` and so on) price single causes and were
not re-timed.

**The Rust harness** (`bench/rharness.rs`) has exactly one `unsafe` block: the call to
`K32GetProcessMemoryInfo` that reads the peak working set (the task allowed it). It passes `-1`, the
current-process pseudo-handle, so `GetCurrentProcess` is not called. Unlike the salvage's Rust harness it
does not call `SetErrorMode`: safe Rust ends a failure with a panic message and exit code 101, which opens
no box. The C programs use the shared `target/redesign/bench/harness.h`, which calls `SetErrorMode` first.

---

## 2. Two findings about the baselines

### 2.1 Plain C for W1 was 2% to 13% slower than it needs to be

The combined report noticed it (its section 2.2): "plain C also reloads its three array pointers from its
globals on every step of the loop, the model keeps them in registers". The pools are global variables, and
the loop body contains a call (`destroy`) on its rare path, so the compiler loads `pos.ents`, `pos.data`
and `vel.data` again on every step. A C programmer who cares about speed keeps them in locals and fetches
them again only after the rare call. I added that as `-DLOCAL` (`bench/w1_group.c`):

```c
/* the salvage's loop (default build) */
for (size_t k = glen; k-- > 0;) {
    uint32_t e = pos.ents[k];
    V2 *p = &pos.data[k], *v = &vel.data[k];
    p->x += v->x; p->y += v->y;
    if (e % 100 == s) { destroy(e); uint32_t n = next++; emplace_pos(n, (V2){0, 0}); emplace_vel(n, (V2){1, 1}); }
}

/* -DLOCAL: the pointers in locals, fetched again only after the despawn and spawn */
uint32_t *ents = pos.ents; V2 *pd = pos.data, *vd = vel.data;
for (size_t k = glen; k-- > 0;) {
    uint32_t e = ents[k];
    V2 *p = &pd[k], *v = &vd[k];
    p->x += v->x; p->y += v->y;
    if (e % 100 == s) {
        destroy(e); uint32_t n = next++; emplace_pos(n, (V2){0, 0}); emplace_vel(n, (V2){1, 1});
        ents = pos.ents; pd = pos.data; vd = vel.data;
    }
}
```

Measured: 89.671 ms against 91.787 ms (batch a), 83.548 against 95.812 (batch b) (`runs/w1.txt`,
`runs/w1b.txt`). Without this fix the combined model looked faster than C (0.968 and 0.909); against the
faster C it is 0.990 and 1.043: on par, not faster.

The combined model's C does the same thing (`combined/w1.c` line 246 takes the three pointers before the
loop, line 260 fetches them again after the despawn and spawn): its compiler knows which calls may move the
pools, from their effect summaries.

### 2.2 The W3 and W4 "plain C" baselines are not the fastest plain C

- **W3**: `plain/w3.c` clears targets through an intrusive list of watchers on every object (48-byte
  objects, two more links per object). Objects that test their target's liveness on read are faster: the
  salvaged "copy instead of share" runs at 0.80 and 0.83 of it. A ratio under 1 in W3 means "this plain C
  chose a slower mechanism", not "faster than any C". No faster plain C was written for this report; the
  salvage report also did not write one.
- **W4**: `plain/w4.c` mallocs each closure environment; with a slab (`plain_w4_same`) the same work is 0.93
  and 0.91 of it. So W4's fair baseline is `plain_w4_same`, and the tables give both.

---

## 3. The tables

Ratios: "x plain C" is against the plain C baseline the task names (`plain_w1_group`, `plain_w2`,
`plain_w3`, `plain_w4`); "x same layout" is against plain C in the model's layout (`plain_w1_group_local`,
`plain_w2_same`, `plain_w4_same`); "x safe Rust" is against `rust_w1` .. `rust_w4`. Times in ms, memory in
KB. Each cell is batch a; the batch b (and for W2 batch c) value follows after a slash. Source:
`target/redesign2/bench/runs/SUMMARY.txt`, computed from `runs/w1.txt` .. `w4c.txt`.

### 3.1 W1, ECS (EnTT owning group); known checksum 2111831520

| Program | Median ms | x plain C | x same layout | x safe Rust | Peak KB | Memory x plain C | Checksum |
|---|---|---|---|---|---|---|---|
| plain C (`plain_w1_group`) | 91.787 / 95.812 | 1.000 | 1.024 / 1.147 | 0.866 / 0.860 | 39,120 | 1.000 | ok |
| plain C, pointers in locals (`plain_w1_group_local`) | 89.671 / 83.548 | 0.977 / 0.872 | 1.000 | 0.846 / 0.750 | 39,168 | 1.001 | ok |
| safe Rust (`rust_w1`) | 105.957 / 111.452 | 1.154 / 1.163 | 1.182 / 1.334 | 1.000 | 39,504 | 1.010 | ok |
| **combined model** (`combined_w1`) | 88.811 / 87.123 | 0.968 / 0.909 | **0.990 / 1.043** | 0.838 / 0.782 | 39,120 | 1.000 | ok |
| copy instead of share (`d1_w1`) | 115.646 / 113.968 | 1.260 / 1.189 | 1.290 / 1.364 | 1.091 / 1.023 | 37,320 | 0.954 | ok |
| checked pointers (`d2_w1`) | 110.755 / 109.173 | 1.207 / 1.139 | 1.235 / 1.307 | 1.045 / 0.980 | 37,872 | 0.968 | ok |
| today's rules, but never an error (`d3_w1`) | 103.560 / 102.326 | 1.128 / 1.068 | 1.155 / 1.225 | 0.977 / 0.918 | 36,224 | 0.926 | ok |
| plain C, archetype layout, reference (`plain_w1_arch`) | 43.965 / 43.977 | 0.479 / 0.459 | 0.490 / 0.526 | 0.415 / 0.395 | 17,116 | 0.438 | ok |

The EnTT layout itself costs about 2x the archetype layout in plain C (89.671 against 43.965 ms) and 2.3x
the memory (39,168 against 17,116 KB), before any memory model. The salvage report found the same (88.5
against 42.5 ms); the owner chose EnTT, and this is its price on this workload.

### 3.2 W2, tree with parent links; known checksum 647034814

| Program | Median ms (a / b / c) | x plain C | x same layout | x safe Rust | Peak KB | Memory x plain C | Checksum |
|---|---|---|---|---|---|---|---|
| plain C (`plain_w2`) | 622.214 / 619.543 / 622.295 | 1.000 | 0.870 / 0.858 / 0.870 | 0.576 / 0.516 / 0.572 | 54,940 | 1.000 | ok |
| plain C, the model's layout (`plain_w2_same`) | 715.065 / 721.916 / 715.276 | 1.149 / 1.165 / 1.149 | 1.000 | 0.662 / 0.601 / 0.657 | 76,664 | 1.395 | ok |
| plain C, Ember's list header only (`plain_w2_emberlist`) | 743.626 / 738.010 / 685.205 | 1.195 / 1.191 / 1.101 | 1.040 / 1.022 / 0.958 | 0.688 / 0.614 / 0.630 | 68,664 | 1.250 | ok |
| safe Rust (`rust_w2`) | 1080.578 / 1201.366 / 1087.976 | 1.737 / 1.939 / 1.748 | 1.511 / 1.664 / 1.521 | 1.000 | 52,288 | 0.952 | ok |
| **combined model** (`combined_w2`) | 720.306 / 749.978 / 737.778 | 1.158 / 1.211 / 1.186 | **1.007 / 1.039 / 1.031** | 0.667 / 0.624 / 0.678 | 76,724 | 1.397 | ok |
| copy instead of share (`d1_w2`) | 873.448 / 936.567 / 874.714 | 1.404 / 1.512 / 1.406 | 1.221 / 1.297 / 1.223 | 0.808 / 0.780 / 0.804 | 77,112 | 1.404 | ok |
| checked pointers (`d2_w2`) | 850.547 / 899.041 / 866.305 | 1.367 / 1.451 / 1.392 | 1.189 / 1.245 / 1.211 | 0.787 / 0.748 / 0.796 | 76,580 | 1.394 | ok |
| checked pointers, without the whole-program fact (`d2_w2_checked`) | 1165.614 / 1318.682 / 1175.798 | 1.873 / 2.128 / 1.889 | 1.630 / 1.827 / 1.644 | 1.079 / 1.098 / 1.081 | 76,580 | 1.394 | ok |
| today's rules, but never an error (`d3_w2`) | 705.206 / 722.579 / 738.374 | 1.133 / 1.166 / 1.187 | 0.986 / 1.001 / 1.032 | 0.653 / 0.601 / 0.679 | 76,612 | 1.394 | ok |
| the same, without the whole-program fact (`d3_w2_unproven`) | 1371.966 / 1401.853 / 1414.161 | 2.205 / 2.263 / 2.272 | 1.919 / 1.942 / 1.977 | 1.270 / 1.167 / 1.300 | 102,364 | 1.863 | ok |

What the model's layout costs against plain C's tightest one: 15% to 17% of time and 40% of memory
(`plain_w2_same` against `plain_w2`). The pieces, from the files' own comments: Ember's list header is
pointer + 64-bit length + 64-bit capacity, growing from 4 (plain C: 32-bit count and capacity, growing from
2), which makes the node 40 bytes instead of 32 (`w2_plain_malloc.c`'s `_Static_assert`); the nodes come one
by one from a slab instead of one array; and the program keeps a list of handles to the nodes, as the
Python program does (8 MB for 1,048,576 slots, salvage report 1.3). Ember's list header alone is 1.10x to
1.20x and 1.25x the memory (`plain_w2_emberlist`). The memory model adds nothing on top (1.00).

Why safe Rust is slow here: it keeps nodes in one `Vec` and links them by `u32` index, so each step of the
parent walk computes `base + index * 32` before the next load, on a chain where each load waits for the
previous one. The salvage report measured the same effect in C (index form +64% against a raw pointer, its
section 0). That is the price of Rust's usual answer to parent pointers (an arena with indices), not a
weakness of this Rust program. My Rust W2 ratio (1.74 to 1.94) is higher than the salvage's (1.48); the
batches here were noisier and I did not find the cause of the difference (UNVERIFIED which is closer).

### 3.3 W3, game objects referring to each other; known checksum 9288233970

| Program | Median ms (a / b) | x plain C | x safe Rust | Peak KB | Memory x plain C | Checksum |
|---|---|---|---|---|---|---|
| plain C (`plain_w3`, watcher lists) | 440.911 / 449.446 | 1.000 | 0.768 / 0.760 | 9,756 | 1.000 | ok |
| safe Rust (`rust_w3`, generational slots) | 574.123 / 591.473 | 1.302 / 1.316 | 1.000 | 11,544 | 1.183 | ok |
| combined model as written (`combined_w3`, side records) | 411.637 / 421.582 | 0.934 / 0.938 | 0.717 / 0.713 | 9,488 | 0.973 | ok |
| **combined model as amended by the attack** (`combined_w3_beside`) | 423.238 / 423.269 | **0.960 / 0.942** | 0.737 / 0.716 | 9,200 | 0.943 | ok |
| combined model with `weak` by registration (`combined_w3_registration`) | 364.586 / 372.867 | 0.827 / 0.830 | 0.635 / 0.630 | 11,948 | 1.225 | ok |
| copy instead of share (`d1_w3`) | 352.080 / 373.666 | 0.799 / 0.831 | 0.613 / 0.632 | 8,364 | 0.857 | ok |
| checked pointers (`d2_w3`) | 419.823 / 438.348 | 0.952 / 0.975 | 0.731 / 0.741 | 9,272 | 0.950 | ok |
| today's rules, but never an error (`d3_w3`) | 424.036 / 438.366 | 0.962 / 0.975 | 0.739 / 0.741 | 8,268 | 0.847 | ok |

(No same-layout plain C exists for W3; see section 2.2 for why ratios under 1 appear.)

### 3.4 W4, callbacks that remove themselves; known checksum 34946993000

| Program | Median ms (a / b) | x plain C | x same work, same slab | x safe Rust | Peak KB | Memory x plain C | Checksum |
|---|---|---|---|---|---|---|---|
| plain C (`plain_w4`, malloc per closure) | 23.285 / 24.238 | 1.000 | 1.077 / 1.103 | 0.731 / 0.728 | 4,076 | 1.000 | ok |
| plain C, same work, slab (`plain_w4_same`) | 21.614 / 21.974 | 0.928 / 0.907 | 1.000 | 0.678 / 0.660 | 4,080 | 1.001 | ok |
| safe Rust, the workload's shape (`rust_w4`) | 31.864 / 33.289 | 1.368 / 1.373 | 1.474 / 1.515 | 1.000 | 4,480 | 1.099 | ok |
| **combined model** (`combined_w4`) | 21.251 / 22.299 | 0.913 / 0.920 | **0.983 / 1.015** | 0.667 / 0.670 | 4,268 | 1.047 | ok |
| copy instead of share (`d1_w4`) | 24.384 / 25.934 | 1.047 / 1.070 | 1.128 / 1.180 | 0.765 / 0.779 | 4,548 | 1.116 | ok |
| checked pointers (`d2_w4`) | 23.032 / 22.758 | 0.989 / 0.939 | 1.066 / 1.036 | 0.723 / 0.684 | 4,344 | 1.066 | ok |
| today's rules, but never an error (`d3_w4`) | 22.641 / 23.724 | 0.972 / 0.979 | 1.048 / 1.080 | 0.711 / 0.713 | 4,260 | 1.045 | ok |
| safe Rust, differently shaped, reference (`rust_w4_return`) | 16.817 / 17.351 | 0.722 / 0.716 | 0.778 / 0.790 | 0.528 / 0.521 | 4,476 | 1.098 | ok |

The two Rust programs differ in shape, and the difference is the speed. The workload (Python twin,
`combined/python/w4.py`) has the handler change the dispatcher's list from inside its own call:

```python
def handler(d, k, v):
    nonlocal counter, total
    if v >= 0:
        counter += v * m
        total += 1
        if k % 100_000 == 99_999:
            d.unregister_current()                 # from inside its own call
            d.register(make_handler(k % 17 + 1))
    return counter
```

Safe Rust refuses a handler that changes the list it is called from while the list is borrowed for the call,
so `rust_w4.rs` takes the closure out of its slot for the call and puts it back after (an `Option` per slot):

```rust
let mut h = self.handlers[i].take().expect("handler re-entered");
h(self, k, k % 1000);
if i < self.handlers.len() && self.handlers[i].is_none() { self.handlers[i] = Some(h); }
```

`rust_w4_return.rs` instead has the handler return "remove me" and the dispatcher do it after the call; then
the handler cannot reach the list at all, and the loop is 0.72x of plain C. (The salvage report, its 1.2,
read both loops' assembly: in this shape the list's length stays in a register across the call; that this
accounts for the whole gap is UNVERIFIED there and here.) It produces the same checksum but is not the program the workload describes: the handler no longer
unregisters itself. I use the first as the Rust baseline.

---

## 4. The decisions the combined report left for the final measurer

From the combined report's section 7 and the attack's section 6:

1. **`weak` as a test on read against `weak` cleared by registration** (rule: "if `combined-w3` is more than
   15% slower than `combined-w3-registration`, the choice is open again").
   - As written: 411.637 / 364.586 = **1.129** (batch a), 421.582 / 372.867 = **1.131** (batch b). Under
     15%: the choice stands.
   - As amended by the attack (the version the model now emits): 423.238 / 364.586 = **1.161**, 423.269 /
     372.867 = **1.135**. One batch over the line, one under. These runs cannot settle a 15% line that the
     result straddles.
   - What else weighs: registration uses 11,948 KB against 9,200 KB (1.30x), and the combined report
     (its 0.4 item 5) records that registration needed three fixes for lists of weak elements and costs
     1.09x to 1.27x of plain C on churn (from the second graphs attack, not re-measured here).
   - So: the test on read stays the choice on these numbers, with a time cost of 13% to 16% in W3 against
     registration, and both are faster than plain C's watcher lists. Whether the owner wants that 13% to 16%
     back for 30% more memory and a more complex mechanism is a choice for the owner, not a measurement.
2. **Whether the 4-byte header removes W2's header cost** (combined report 4.4): yes. The model is 1.007,
   1.039 and 1.031 of plain C in the same layout with no header at all (`plain_w2_same` has none), and uses
   the same memory (76,724 against 76,664 KB).
3. **The lean class from a slab** (`ownership-attack2/bench/share_slab.c` against `share_slab0.c`), **the
   claim's count test** and **the `threads(any)` compare** (combined report section 7, items 3 to 5), and the
   attack's `share_weak.c` and `ecs_view_cost.c`: not timed here. They are not W1 to W4 programs, and this
   task named only those. Their only numbers are still the PROVISIONAL ones in their own reports.

---

## 5. Against the salvage report's numbers (taken through `cmd /c start`)

Same programs, ratio to the same baseline; salvage numbers from its section 0 and 1.2 (7-round medians in
`salvage/runs/SUMMARY.txt`), mine from section 3:

| | Salvage report | My runs (batch a / b [/ c]) |
|---|---|---|
| copy instead of share, W1 / owning-group plain C | 1.21 | 1.260 / 1.189 |
| checked pointers, W1 | 1.10 | 1.207 / 1.139 |
| today's rules, W1 | 1.06 | 1.128 / 1.068 |
| copy instead of share, W2 / `plain_w2_emberlist` | 1.24 | 873.448 / 743.626 = 1.17; 1.27; 1.28 |
| checked pointers, W2 | 1.24 | 1.14; 1.22; 1.26 |
| today's rules, W2 | 1.07 | 0.95; 0.98; 1.08 |
| copy instead of share, W3 / plain C | 0.82 | 0.799 / 0.831 |
| checked pointers, W3 | 0.94 | 0.952 / 0.975 |
| today's rules, W3 | 0.97 | 0.962 / 0.975 |
| copy instead of share, W4 / plain C | 1.04 | 1.047 / 1.070 |
| checked pointers, W4 | 0.96 | 0.989 / 0.939 |
| today's rules, W4 | 0.96 | 0.972 / 0.979 |
| safe Rust W1 / W2 / W3 / W4 (returning shape) | 1.17 / 1.48 / 1.28 / 0.71 | 1.154 / 1.737, 1.939, 1.748 / 1.302 / 0.722 |

(The W2 rows against `plain_w2_emberlist` are computed from the medians in `runs/SUMMARY.txt`, e.g. batch
b: `d1_w2` 936.567 / 738.010.) Every salvaged ratio is within 0.10 of at least one of my batches; the largest
differences are "checked pointers" on W1 (1.10 there, 1.14 and 1.21 here) and safe Rust's W2 (1.48 there,
1.74 to 1.94 here, slower in every batch). **So the runner that used `cmd /c start` did not change the
salvage report's conclusions**; it changes one baseline number (Rust W2), and that change makes Rust look
worse, not the designs.

What the salvage verdicts look like with these numbers, against plain C in the same layout: "copy instead of
share" is 1.29x to 1.36x on W1 and 1.22x to 1.30x on W2, so not on par on those two; "checked pointers"
1.24x to 1.31x on W1 and, without its whole-program fact, 1.63x to 1.83x on W2; "today's rules, but never an
error" 1.16x to 1.23x on W1 and 1.92x to 1.98x on W2 without its fact. The combined model is on par on all
four; it is the only one of the four models measured here that is.

---

## 6. What the attack's problems and the reported shortcuts mean for these numbers

The attack found six problems, fixed all six, and checked the C models for shortcuts (its sections 3 and 4).
Each, and what it does to the table:

| Attack item | What it is | What it means for the numbers |
|---|---|---|
| Problem 1, narrowing a `weak` field across a drop | compile-time rule: a statement that may drop a `T` ends narrowing of every `weak T` place | none; no workload narrows a weak field across a drop, and the fix emits nothing when running |
| Problem 2, netted counts and side records in threads | weak counts move beside the objects; slabs per thread | W3 should be read from `combined_w3_beside`: 0.960 / 0.942 of plain C (as written: 0.934 / 0.938), 9,200 KB (as written: 9,488). The per-thread slab with a list for frees from other threads is **not built**; every model here uses one global free list per size. W3 and W4 allocate only on their rare paths (W3: 9,173 objects die in all, combined report 2.2), so the effect should be small, but it is UNVERIFIED |
| Problem 3, a `view` over a pool a group owns | the view walks a pending list or a copy of ids | none for W1: the model's W1 loop is a group loop, which the problem does not touch. A user who writes W1's body as a `view` gets the attack's costs (0.94 and 1.02 with the pending move, 1.11 and 1.09 with the copy; attack runs, not re-measured) |
| Problem 4, a closure capturing a list element's name | handle for class elements, refusal or path for structs | none; W4's closures capture `counter`, `m` and `total`, not element names |
| Problem 5, counting a weakly referenced object through a side record | fixed by problem 2's fix | not in W1 to W4 (W3 changes no strong count in its loop); the sharing workload was not timed here |
| Problem 6, a back link's removal inside the loop filter | compile-time rule, no running cost | none; W2's re-parent is not inside a loop over the same list |
| Shortcut: W1's `Pool.leave` cursor compare left out | one compare per destroy, never taken in W1 | the model's W1 is missing at most one compare and branch per despawn. W1 has at least 500,000 despawns (attack section 4). At an assumed one nanosecond each (UNVERIFIED, not measured) that is at most about 0.5 ms of about 88 ms, under 1%; it cannot change "on par" |
| Shortcut: W1's registry type-table lookup left out | the lookup that finds a component's pool | it runs once per `add` of a component, on the rare spawn path, not per entity in the loop; small, UNVERIFIED by measurement |
| W2's `nodes` list grows by doubling where the baseline allocates once | the model does a little more work | makes the model look slightly slower, not faster |
| Zero-divisor checks (`k % len`) left out of every C model and C baseline | not a shortcut against C | safe Rust does check (and bounds-checks), so Rust does slightly more work in W4; this is Rust's real cost |

None of these moves a number in a way that changes a verdict in this report.

---

## 7. What is left open

- **W2's memory**: 1.40x plain C, all of it from the layout the language specifies (Ember's list header,
  a slab block per node, a list of handles). Not a memory-model cost, but not on par with plain C either;
  the 24-byte list header alone is 1.25x. A smaller list header is a standard-library question, outside the
  memory model.
- **`weak`**: 1.13x to 1.16x of registration in W3; on the 15% line (section 4).
- **The per-thread slab** the attack's fix needs is not built; its cost is UNVERIFIED.
- **Programs not timed** (section 4, item 3).
- **Noise**: every number here is PROVISIONAL. Differences under 15% that a conclusion rests on: the
  combined model against same-layout C on all four workloads (0.98 to 1.04), and the `weak` comparison.
  For the combined model the conclusion is "on par", which needs the difference to be small, and every
  batch agrees on that.

---

## 8. Files and commands

All under `C:/Users/ism19/Code/ember/target/redesign2/bench/`:

| File | What it is |
|---|---|
| `w1_group.c` (`-DLOCAL`), `w1_arch.c`, `w2.c`, `w3.c`, `w4.c` | the plain C baselines (copied, include line changed; `-DLOCAL` added) |
| `rharness.rs`, `rust_w1.rs` .. `rust_w4.rs`, `rust_w4_return.rs` | the safe Rust baselines and their harness |
| `build.sh` | every build command (Git Bash: `bash build.sh`) |
| `run.ps1` | the timing runner |
| `medians.py` | medians, spreads, ratios, checksum check (`C:\Python314\python.exe -I medians.py [b|c]`) |
| `runs/w1.txt` .. `w4.txt`, `w1b.txt` .. `w4b.txt`, `w2c.txt`, `SUMMARY.txt` | every run line and the summary |
| `runs/w1_before_local.txt`, `runs/w1b_before_local.txt` | the first two W1 batches, before `plain_w1_group_local` existed (not used) |
| `bin/` | the 33 binaries |

Nothing outside `tasks/96/redesign/phase3b-measurements.md` and `target/redesign2/bench/` was written. No
Ember program was built or run. No git command was run. Nothing was downloaded or installed.
