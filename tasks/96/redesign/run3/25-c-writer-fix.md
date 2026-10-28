# 25 C writer, second pass: the gaps 24 found, fixed (run 3, phase 4)

Power: `PowerLineStatus` printed `Online` at the start, before the builds, and before writing this report.
No compiler change, no git command, no Ember program run (only `ember check`). Nothing another agent wrote
was edited: I copied 23's files from `target/redesign3/23-c-writer/` into
`target/redesign3/25-c-writer-fix/` with `cp` and changed only the copies. The model is
`RUN3/combined-v4.md`; the first pass is `RUN3/23-c-writer.md`; the gaps are 24's (`RUN3/24-faithfulness.md`).

## 0. Terms (as 23 and 24 use them)

- **check**: code that runs while the program runs and stops it when something is wrong (an index past
  the end, an overflow, a zero divisor, `unwrap` of `None`, a count that would overflow).
- **memory-model work**: what the model itself adds while running: a weak count changed, the "ended"
  (DEAD) test on reading a `weak` link, a count held across a call, a box for a captured variable.
- **range fact** (`[RNG-4]`): a number the compiler knows is inside a range, which removes a check. A
  function's result is not one; the spec does not list "a length is `>= 0`".
- **floor `%`** (`FMOD` in the C): Ember's `%` on signed integers, which rounds like Python's; one compare
  and add more than C's `%` unless the left side is known to be `>= 0`.
- **box**: the model's "small counted heap block holding one captured variable" (combined-v4.md line 7316).
- **noalias** (`restrict` in C): the promise that nothing else reaches what a parameter points to.

## 1. The ten gaps and what I did

| # | workload | gap (24) | what I did | where |
|---|---|---|---|---|
| 1 | W2 | DEAD test on every hop of the walk: R11 says test, AN-62 says no test (for the lead) | Nothing changed in the model's C (R11 as written stays the default build). **Both builds exist and both ran**: `model_w2` (with the test) and `diag_w2_hop_proved` (`w2.c -DHOP_PROVED`, only that test removed, every other fix below included). | `w2.c` 130-134 |
| 2 | W4 | `counter` and `m` in the state block, model says one box each (favours the model) | **The box-per-capture build is now the model's W4** (default build): `counter` and `m` each in a 16-byte counted box from a slab; the state holds the two box pointers; one more load per use. 23's placement stays as `w4.c -DSTATE_BLOCK` (`model_w4_state_block`), for if the lead writes the placement into D1. | `w4.c` 53-61, 103-113, 76-83 |
| 3 | W2 | `h_rand() % i` used C's `%` | `FMOD` at both sites; the zero-divisor proof from `i >= 1` stays. | `w2.c` 102, 118 |
| 4 | W3 | the four `rand_int() %` sites used C's `%` | `FMOD` at all four; at line 133 floor `%` by a positive length is what proves the index below `len`, so that bounds check stays elided, now with the proof true. | `w3.c` 91, 92, 109, 133 |
| 5 | W4 | bounds check of `self.handlers[self.current]` elided | **Added** `[K10]` (no ruling on lengths; the lead may still remove it with one line in `[RNG-4]`). | `w4.c` 142 |
| 6 | W2 | the drop of `n` missing when the build's `nodes[...]` gives `None` | `else node_drop(n);` (never taken in W2). | `w2.c` 122 |
| 7 | W2 | `[K8]` bounds check of `nodes[i]` per iteration; `[OPT-2]` says one entry test | **Loop versioning**: one test `N - 1 < len(nodes)` per pass (`[K8]`, line 140); if it passes, the loop runs with no check of `nodes[i]`; if it fails (never in W2), a second copy of the loop checks `nodes[i]` on each iteration (line 95, `chk`) and so stops at the same iteration the language would. The body is one inlined function, `reparent` (lines 93-107), used by both copies. | `w2.c` 93-107, 137-144 |
| 8 | W4 | `total` in a counted box; D1's own test allows an address (optional) | **Done**: `total` is main's local and each state holds its address; no box, no count on `total` (one count add per handler made and one subtract per handler dropped are gone). | `w4.c` 17-20, 149, 153 |
| 9 | W3, W4 | noalias: the W3 question is for the lead; W4's `ref mut d` lacks noalias (unfair to the model) | W3: no change (24 measured no difference; the lead rules R4/R9). W4: **`restrict` added** on the handler's `ref mut d` and on dispatch's `mut self`, as R4 says. Sound here: inside each, the `Dispatcher` is reached only through `d` (the state and `total` are other objects). | `w4.c` 64, 102, 116, 138 |
| 10 | W1, W3 | no overflow check on `len(self.ents) - 1` and `before - len(objects)` | **Added both** (no ruling on lengths): `[K20]` in W1, `[K19]` in W3. One compare per despawn and per frame. | `w1.c` 56; `w3.c` 127 |

Nothing else changed. The four `.em` files are 23's, unchanged (`cmp` against 23's printed "same" for
each): box placement, `restrict`, `FMOD` and loop versioning are the compiler's, not written in the program.

## 2. Build and run (each built once with clang -O2, run once)

Build: `"/c/Program Files/LLVM/bin/clang.exe" -O2 -I /c/Users/ism19/Code/ember/target/redesign/bench
wN.c -o bin/model_wN.exe`, the same with `-DPLAIN` into `bin/plain_same_wN.exe`, plus
`-DHOP_PROVED w2.c` into `bin/diag_w2_hop_proved.exe` and `-DSTATE_BLOCK w4.c` into
`bin/model_w4_state_block.exe`. No warnings. Each run once, `timeout 300 ./bin/<name>.exe`,
`target/redesign3/25-c-writer-fix/run_output.txt`:

```text
model-w1 ms=95.642 peak_kb=36748 checksum=2111831520
exit=0
plain-same-w1 ms=95.687 peak_kb=36368 checksum=2111831520
exit=0
model-w2 ms=988.720 peak_kb=84924 checksum=647034814
exit=0
diag-w2-hop-proved ms=701.723 peak_kb=84904 checksum=647034814
exit=0
plain-same-w2 ms=760.696 peak_kb=84944 checksum=647034814
exit=0
model-w3 ms=425.456 peak_kb=9180 checksum=9288233970
exit=0
plain-same-w3 ms=422.201 peak_kb=9144 checksum=9288233970
exit=0
model-w4 ms=30.030 peak_kb=5068 checksum=34946993000
exit=0
model-w4-state-block ms=26.136 peak_kb=4804 checksum=34946993000
exit=0
plain-same-w4 ms=21.527 peak_kb=4796 checksum=34946993000
exit=0
```

These are single unpinned runs to show the checksums, not timings; the timer's numbers decide. Every
checksum equals plain C's for its workload (23's `baseline_output.txt`: W1 2111831520, W2 647034814,
W3 9288233970, W4 34946993000). In these single runs the two open questions show the sizes 24 measured:
W2 with the per-hop DEAD test 988.7 ms against 701.7 ms without it; W4 with a box per capture 30.0 ms
and 5,068 KB against 26.1 ms and 4,804 KB with 23's placement.

AddressSanitizer (`clang -O1 -g -fsanitize=address`, the ASan DLL directory first on `PATH`), every
build that is not `-DPLAIN`, `target/redesign3/25-c-writer-fix/asan_output.txt`: no report (no line
starting `ERROR: AddressSanitizer`), the same checksums:

```text
model-w1 ms=236.333 peak_kb=96564 checksum=2111831520
exit=0
model-w2 ms=2971.793 peak_kb=115232 checksum=647034814
exit=0
diag-w2-hop-proved ms=2276.441 peak_kb=115224 checksum=647034814
exit=0
model-w3 ms=679.973 peak_kb=49672 checksum=9288233970
exit=0
model-w4 ms=66.267 peak_kb=9848 checksum=34946993000
exit=0
model-w4-state-block ms=54.444 peak_kb=9512 checksum=34946993000
exit=0
```

`ember check` on the four (unchanged) NEW programs refuses each at its first new word, as for 23
(`target/redesign3/25-c-writer-fix/ember_check_output.txt`): `w1.em` "error[E0110]: `add` is declared
without a body" (83 error lines), `w2.em` and `w3.em` "error[E1010]: cannot find type `weak` in this
scope" (48 and 58), `w4.em` "error[E0110]: `register` is declared without a body" (26); exit 1 each.

## 3. Every check the final C keeps, with its line

Lines are of the files in `target/redesign3/25-c-writer-fix/` as they are now. Rows changed by me are
marked **(25)**. "Per step" means inside the workload's hot loop.

### 3.1 W1 (`w1.c`)

| tag | line | check | how often |
|---|---|---|---|
| K14 | 110 | `[OPT-2]` entry test: `glen <=` the three lengths | once per pass (100) |
| K15 | 117 | `dead.push` growth path | when the buffer grows |
| K1, K2, K3 | 44, 46, 50 | bounds in `Pool.swap` | per swap |
| K4 | 54 | bounds `sparse[e]` in `Pool.remove` | per remove |
| **K20 (25)** | 56 | overflow, `len(self.ents) - 1` | per remove |
| K6, K7, K8 | 57, 60, 62 | bounds in `Pool.remove` | per remove |
| K9 | 69 | overflow, `next_id += 1` | per create |
| K10, K11 | 73, 76 | bounds and overflow in `enter` | per group entry |
| K5, K12, K13 | 84, 86, 87 | bounds and overflow in `destroy` | per despawn |
| K16, K17, K18 | 131, 131, 132 | overflow; float -> int saturation | per entity, once |
| K19 | 134 | overflow | once |
| (no tag) | 37, 39, 40 | `pool_add`'s pushes: growth path | when a buffer grows |

No memory-model test. Nothing per step besides plain C's work.

### 3.2 W2 (`w2.c`)

| tag | line | what | how often |
|---|---|---|---|
| M2 | 133 | DEAD test of the walk's copied link `a` (R11; gap 1, for the lead; absent in `-DHOP_PROVED`) | **per hop** (per step) |
| K7 | 135 | overflow, `depth_sum += 1` | **per hop** (per step) |
| M2 | 127, 120, 104, 148 | null and DEAD test of a `nodes[...]` read: walk, build, re-parent, checksum | per node per pass; per node built; per re-parent; per node once |
| M1, K1 | 114, 117 (`weak_make` 50-56, check 52) | weak count +1 and its overflow check | per node, once |
| M3 | 66, 73 | `set by`: `push` sets `parent`, `remove` clears it | per build and re-parent |
| K2 | 65 | `children.push` growth path | when a buffer grows |
| K3 | 69 | bounds `children.remove(k)` | per re-parent |
| K4 | 87 | `index_of(node).unwrap()` | per re-parent |
| K5 | 117 | `nodes.push` growth path | when the buffer grows |
| K6 | 119 | bounds `nodes[rand() % i]` in the build | per node, once |
| **K8 (25)** | 140 | `[OPT-2]` entry test `N - 1 < len(nodes)`; the per-iteration check at line 95 runs only in the checked copy, never in W2 | **once per pass** (10) |
| K9 | 103 | bounds `nodes[rand() % i]` in the re-parent | per re-parent |
| K10, K11 | 149 | overflow, `*` and `+` in the checksum | per node, once |

Floor `%` **(25)**: 102, 118 (`rand() % i`). New drop **(25)**: line 122, `else node_drop(n)`, never taken.

### 3.3 W3 (`w3.c`)

| tag | line | what | how often |
|---|---|---|---|
| M2 | 73, 83 (`weak_read` 46-48) | null and DEAD test of `self.target` | **per Ship and Turret update** (per step) |
| K3-K6 | 75, 76 | overflow in `Ship.update` | per Ship update |
| K7 | 80 | overflow, `self.x + 1` (then floor `%` by a constant) | per Rock update |
| K8, K9, K2 | 84, 69 | overflow, the `-`s, `abs`, `+` | per Turret update with a live target |
| K10 | 85 | overflow, `t.health -= 1` | per hit |
| M3 | 124 (`obj_end` 59-65) | a dead object's owner ends | per death |
| M1, K1 | 111, 133 (`weak_store` 52-58, check 53) | weak count +1 and its overflow check | per object made |
| K11 | 110 | bounds `objects[rand() % i]` | per object, once |
| K12, K15 | 113, 134 | `objects.push` growth path | when the buffer grows |
| **K19 (25)** | 127 | overflow, `before - len(objects)` | per frame |
| K13 | 128 | overflow, `destroyed += dead` | per frame |
| K14 | 131 | overflow, `next_id += 1` | per object made |
| K16 | 137 | overflow, `destroyed * 1_000_000` | once |
| K17, K18 | 140 | overflow in the checksum | per object, once |

Floor `%`: 80, 93, 95 (23's) and **(25)** 91, 92, 109, 133 (`rand() %`). No `restrict` on `self` (gap 9,
for the lead).

### 3.4 W4 (`w4.c`)

| tag | line | what | how often |
|---|---|---|---|
| K7 | 140 | zero divisor of `k % len(self.handlers)` | **per dispatch** (per step) |
| **K10 (25)** | 142 | bounds of `self.handlers[self.current]` | **per dispatch** (per step) |
| M1 | 143 | {code, state} copied out for the call | per dispatch |
| **M5 (25)** | 118, 126, 135 (`CNT`, `MV`, 58-59) | `counter` and `m` reached through their boxes | per handler call |
| K4, K5 | 118 | overflow, `v * m` and `counter += ...` | per handler call |
| K6 | 119 | overflow, `total += 1` (now through an address) | per handler call |
| M2, K1 | 122, 128 (check 73) | the handler's hold on its own state across `unregister_current` | per self-removal |
| M3 | 93 (`state_release` 76-83) | the removed handler is dropped; at zero its two boxes and its block go back | per self-removal |
| **M4 (25)** | 103-113 | a new handler: one state block and two boxes; no count on `total` any more | per handler made |
| K2 | 88 | bounds `remove(self.current)` | per self-removal |
| K3 | 99 | `handlers.push` growth path | when the buffer grows |
| K8 | 158 | overflow, `s += ...` | per handler, once |
| K9 | 160 | overflow, `total + s` | once |

Floor `%`: 120, 125, 141, 144 (23's). `restrict` **(25)**: 116 (`ref mut d`), 138 (`mut self`).

### 3.5 Counts of checks kept

Each `K` tag, plus the DEAD test `M2` where there is one:

| workload | language checks (K) | memory-model tests | total | per step |
|---|---|---|---|---|
| W1 | 20 (23: 19) | 0 | 20 | none (K14 once per pass) |
| W2 | 11 (K8 now once per pass) | 1 (M2) | 12 | M2 and K7 per hop |
| W3 | 19 (23: 18) | 1 (M2) | 20 | M2 per Ship/Turret; K2-K10 per update |
| W4 | 10 (23: 9) | 0 | 10 | K7 and K10 per dispatch; K4-K6 per call |

## 4. For the timer (26) and the lead

The model's programs are the default builds of `target/redesign3/25-c-writer-fix/wN.c`; plain C in the
model's layout is `wN.c -DPLAIN` (for W1 the baseline `w1_group.c -DLOCAL` is also in the model's layout).
Two more builds to time, labelled, until the lead rules:

| build | what it is | open question |
|---|---|---|
| `w2.c -DHOP_PROVED` (`diag-w2-hop-proved`) | the model's W2 without the DEAD test per hop, i.e. AN-62's C | gap 1: R11 as written (default build) or AN-62's walk |
| `w4.c -DSTATE_BLOCK` (`model-w4-state-block`) | 23's placement of `counter` and `m` in the state block, all other 25 fixes kept | gap 2: the default build follows D1 as written; this one is the model if the lead adds the placement to D1 |

24's cautions stand: W2's `plain_same_w2` is not a floor (a build doing more work ran faster); W3's
`-DPLAIN` keeps the weak counts and DEAD tests, so the model's cost of links shows only against the
baseline `bench/w3.c`. `w2.c -DPLAIN` keeps the one entry compare per pass (both copies of the loop are
then the same code); it is 10 compares per run.

Still for the lead, not changed by me: gap 1 (which W2 is the model's), gap 9's W3 half (R4's noalias on
`mut self` against a `ref mut` view reached through a `weak` link of the same type), and the missing
`[RNG-4]` fact "a container's `len()` is `>= 0`", which would remove `[K20]` in W1, `[K19]` in W3 and
`[K10]` in W4 (W4's also needs the fact to survive the store to `self.current`; a local would do).

## 5. Files

All under `C:\Users\ism19\Code\ember\target\redesign3\25-c-writer-fix\`:

| file | what |
|---|---|
| `w1.em` .. `w4.em` | 23's NEW-syntax programs, copied unchanged |
| `ember_rt.h` | 23's runtime header, copied unchanged |
| `w1.c` .. `w4.c` | the final C; `-DPLAIN` plain C in the model's layout; `w2.c -DHOP_PROVED`, `w4.c -DSTATE_BLOCK` the two builds above |
| `bin/` | `model_w1..w4.exe`, `plain_same_w1..w4.exe`, `diag_w2_hop_proved.exe`, `model_w4_state_block.exe`; `bin/asan/` the AddressSanitizer builds |
| `run_output.txt`, `asan_output.txt`, `ember_check_output.txt` | the outputs quoted above |
