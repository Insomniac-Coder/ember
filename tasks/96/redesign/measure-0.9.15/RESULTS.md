# The measuring phase: results (2026-10-27)

## How these were measured

- **Who ran what.** The programs were written by agents M1 to M4; their reports are `M1.md` to `M4.md`,
  next to this file. The lead built every program again from source and ran every timing.
- **Builds.**
  - C with clang 22.1.8 `-O2`;
  - C with MSVC `cl /O2 /MD /fp:precise /std:c11`;
  - Rust with rustc 1.98.1 `-C opt-level=2`.

  Every build had no errors and no warnings.
- **Running.**
  - One program at a time, pinned to the P-cores (mask `0xC03C03`).
  - Mains power was checked before every round. There was no Kernel-Power 105 event from 08:05Z to
    08:23Z.
  - 5 rounds per workload, the order reversed in even rounds.
  - Workloads with a program over 10% spread got two more sets of 5 rounds. Their numbers below are the
    medians of all 15 runs.
- **Checks.** Every run of every program exited 0 and printed its workload's checksum.
- **Raw data and scripts.** Copied into `lead/` next to this file: `lead/runs/` (every run's line), `lead/tables.md`
  (every workload's table), and the runner, build and summary scripts.

How to read the numbers:
- **"x C"** is the median time divided by the plain C program of the same workload, from the same
  compiler. Rust is set against clang.
- **"Verified"** means the lead read the assembly and it shows the stated cause.
- **"Not settled"** means the cause is not established, and no conclusion rests on it.

## Summary

| cost row (`run4/v5/rules.md` section 2) | result | verdict |
|---|---|---|
| owner count +1/-1 and the limit compare | the compare: +1.2% (retarget), 0% (hotshare, the worst case); the count is faster than Rust's `Rc` (0.75x and 0.48x of its time) | **holds** |
| `thread_shared` atomic count and its limit test | the test: 0%; atomic counts are 10.8x the one-thread ones on the worst case, and 7% faster than Rust's `Arc` | **holds** (atomics are their own cost) |
| a view of a `thread_shared` object through a `weakref` link (one atomic count per view) | 10.2x plain C (Rust `Arc` 13.1x); salvage `plaindata` 1.14x | **falls short; the salvage fixes it** for classes whose end runs and frees nothing |
| a handler's count on its own state | 0% (209.0 against 207.8 ms) | **holds** |
| weak counts on every `weakref` link, parent links included | +13.3% over the same layout; parent links are 7.8 of those points (salvage `parentfree`) | **falls short by 13%; the salvage halves it** |
| the walk up a tree across a call that may change nodes | count moves +3.5% over the free walk, inside the noise (spreads up to 28%); the region form measured 13.5% **faster** than the free walk (not settled) | **holds** within this workload's noise |
| the checked trampoline (C callbacks) | tight callback: 1.57x (clang), 1.17x (MSVC); parser workload: 1.08x, 1.02x; salvage `teb`: 0.99x, 1.11x | **falls short** on tight callbacks; `teb` fixes clang |
| the arena's instance word, and the allocation path | the word: +3% (lists), +26% clang and +45% MSVC (small values), +30% (a pool that frees); memory +12% to +58%; ways (a) and (b) remove it | **falls short**: owner question Q-F |
| `this` never `noalias` | 0% in two loops; +25% clang and +9% MSVC in the float loop; salvage `typeproof` removes it | **falls short in one loop shape; the salvage fixes it** |
| one whole variable under two names: the one-name copy | 3.6x **faster** than plain C (clang), 3.3x (MSVC); 247 bytes of code for the copy | **holds, better than C** |
| the same function's normal copy with `noalias` | +21% over plain C with `restrict` (clang), +8.6% (MSVC); equal to Rust | **falls short; not settled** |
| a `ref mut` span the callee may re-point, across a package | clang 1.008x; MSVC 1.188x (1.016x when the length is not known) | **holds under clang, falls short under MSVC** |

## 1. Owner counts and the limit compare (M1)

**retarget** (run 3's AN-52 (a); 15 runs):

| program | clang ms | x C | MSVC ms | x C | peak KB |
|---|---|---|---|---|---|
| plain C (no count) | 218.5 | 1.0 | 214.0 | 1.0 | 53,280 |
| model (count, limit compare) | 253.7 | 1.161 | 263.3 | 1.231 | 55,332 |
| count, no compare | 250.6 | 1.147 | 260.1 | 1.215 | 55,332 |
| model `thread_shared` | 260.4 | 1.192 | 266.8 | 1.247 | 55,336 |
| Rust `Rc` / `Arc` | 336.5 / 347.0 | 1.540 / 1.588 | | | 61,928 |

**hotshare** (every step makes and ends an owner, all in cache; 5 runs):

| program | clang ms | x C | MSVC ms |
|---|---|---|---|
| plain C | 18.2 | 1.0 | 29.9 |
| model | 60.9 | 3.351 | 61.3 |
| count, no compare | 61.3 | 3.369 | 60.7 |
| model `thread_shared` | 655.7 | 36.06 | 659.6 |
| `thread_shared`, no compare | 658.6 | 36.23 | 659.0 |
| Rust `Rc` / `Arc` | 126.6 / 703.8 | 6.97 / 38.7 | |

- **The compare costs nothing measurable:** 60.9 against 61.3 ms in its worst case.
- `[BEN-6]`'s gate (the model within 1.15x of the count with no compare) holds with room to spare
  (1.012).
- The salvages `fold` and `wide` (M1.md) gain nothing, so they are not needed.
- **Cost-table correction:** run 3's 1.13x to 1.25x already contained a limit compare (M1.md, "a
  finding first").

## 2. A view of a `thread_shared` object through a `weakref` link (M1, weakview; 15 runs)

| program | clang ms | x C | MSVC ms | x C |
|---|---|---|---|---|
| plain C (generational slots) | 53.4 | 1.0 | 54.3 | 1.0 |
| model (CAS open, atomic -1 at the view's end) | 544.6 | 10.19 | 546.1 | 10.06 |
| the same program on one thread (header test only) | 53.2 | 0.996 | 56.5 | 1.04 |
| salvage `plaindata` (an acquire load and a test, no view count) | 61.1 | 1.144 | 64.6 | 1.191 |
| Rust `Arc`, `Weak::upgrade` | 701.0 | 13.12 | | |
| Rust `Rc` | 64.3 | 1.204 | | |

- **Why 10x:** two locked read-modify-writes per opening, against one load and a compare.
  The whole program takes 8.1 ns per opening (544.6 ms over 67,108,864 openings).
- **The salvage** applies to a `thread_shared` class family with no `drop` whose fields own nothing and
  hold no link. Its safety argument is in M1.md, salvage 4.
- **For classes that do have a drop**, epoch-based deferral is the general form. It is not written yet.
- **This is a spec change** (`[RC-5]` point 2), for the owner.

## 3. A handler's count on its own state (M1, selfremove)

The model took 209.0 ms (1.12x C, from the checks and the slab, as run 3's 1.09x). Without the count it
took 207.8 ms. Rust took 274.1 ms.

**The row holds.**

## 4. Weak counts on links, parent links included (M2, tree; 15 runs)

| program | clang ms | x C | MSVC ms | peak KB |
|---|---|---|---|---|
| plain C (array of nodes, parent pointers) | 445.7 | 1.0 | 440.0 | 56,748 |
| the model's layout, no counts | 1,015.9 | 2.279 | 1,021.2 | 87,160 |
| model (every link counted) | 1,151.0 | 2.582 | 1,135.3 | 87,056 |
| salvage `parentfree` (parent links proved by `[WK-17]` uncounted) | 1,067.7 | 2.395 | 1,096.9 | 87,160 |
| Rust `Rc` tree | 1,659.8 | 3.724 | | 118,456 |
| Rust index arena | 311.5 | 0.699 | | 53,564 |

- **The rule** costs 13.3% over the same layout. Parent links are 7.8 points of that (1,151.0 against
  1,067.7), so the salvage removes more than half.
- **The salvage is a spec change** (R11: "every link, with no exception"). Its safety argument is in
  M2.md.
- **The layout itself is outside this row.** Separately made 48-byte objects, a separate index and the
  recursive end of the tree cost 2.28x against C's array of nodes, and Rust's index arena is 0.70x of
  plain C. This is the class-object model of every earlier spec, and run 3 measured the same. It is
  noted for a later decision, not this one.

## 5. The walk across a call that may change nodes (M2, w2v; 15 runs)

| program | clang ms | x C | MSVC ms | x C |
|---|---|---|---|---|
| plain C, with the call | 959.2 | 1.0 | 984.2 | 1.0 |
| plain C, no call (run 3's W2) | 479.2 | 0.500 | 455.5 | 0.463 |
| model's layout, no checks | 1,356.6 | 1.414 | 1,334.8 | 1.356 |
| views (the visitor changes no node: nothing per step) | 1,365.6 | 1.424 | 1,357.3 | 1.379 |
| count moves per step (the rule where no region is allowed) | 1,414.1 | 1.474 | 1,439.9 | 1.463 |
| the same with a drop method (two header tests) | 1,418.3 | 1.479 | 1,409.3 | 1.432 |
| deferred-drop region (header test only) | 1,180.9 | 1.231 | 1,277.7 | 1.298 |
| salvage `freeonly` / `dispatch` / `hoist` | 1,279.4 / 1,360.2 / 1,175.4 | 1.33 / 1.42 / 1.23 | | |
| Rust index arena / Rust `Rc` walk | 1,019.5 / 2,654.4 | 1.063 / 2.767 | | |

- **The walk is memory-bound and noisy.** The 15-run spreads reach 28%, though the medians of the three
  sets agree within about 4%.
- **The count moves** cost 3.5% over views, which is inside that noise.
- **Every form is about 2x faster than Rust's `Rc` walk.**
- **Not settled:** the region form measured 13.5% faster than views, although its inner loop (assembly
  read) is the views loop plus one compare and branch. No conclusion rests on that difference.
- **Cost-table correction:** `rust_w2` is an index arena with no `Weak` (M2.md, doubt 2). `rust_w2v_rc`
  is the real per-step upgrade.

## 6. The checked trampoline (M4)

| workload | plain C | model | no thread check | salvage `fold` | salvage `teb` | Rust |
|---|---|---|---|---|---|---|
| cb_event, clang (15 runs) | 162.3 ms | 1.565 | 1.215 | 1.227 | **0.989** | 0.982 (checks nothing) |
| cb_event, MSVC | 161.7 ms | 1.174 | 1.034 | 1.239 | 1.106 | |
| cb_parse, clang | 83.0 ms | 1.082 | 1.067 | 1.077 | 1.072 | 1.019 |
| cb_parse, MSVC | 101.1 ms | 1.018 | 1.047 | 1.040 | 1.094 | |
| cb_once, clang (15 runs) | 149.5 ms | 1.576 | 1.473 | | genmark 1.573, nomark 1.517 | 7.648 (one `Box` per request) |
| cb_once, MSVC | 143.0 ms | 1.233 | 1.218 | | genmark 1.237, nomark 1.183 | |

- **The design's 1.02x to 1.05x holds only for the parser workload.**
- **On a tight callback** the release thread check alone costs 29% under clang and 14% under MSVC.
- **Cause (M4's reading of clang's assembly, not checked by the lead for MSVC):** the attach check and
  the creator check each reach the thread's TLS through `_tls_index`, the TEB and the module's TLS
  block, four or five loads each. Salvage `teb` compares the OS thread id in the TEB, which is one load.
  It also stands for the attach check, under the runtime duty given in M4.md.
- **`cb_once`'s cost** is mostly registration: the mark is 4% of it, and the thread check 7%.
- **Cost-table correction:** run 3's 1.02x to 1.05x already included a thread compare (M4.md).
- **Not settled:** why `teb` is 0.99x under clang and 1.11x under MSVC.

## 7. Arenas: allocation and the instance word (M3)

**trees** (2^20 small nodes alive per batch; 15 runs):

| program | clang ms | x arena C | MSVC ms | peak KB |
|---|---|---|---|---|
| plain C, malloc | 939.6 | 4.92 | 938.0 | 36,836 |
| **plain C, arena (the C to match)** | 191.0 | 1.0 | 178.3 | 28,212 |
| model: runtime call per allocation, the word, the end walk | 394.0 | 2.06 | 418.0 | 44,732 |
| model, allocation inlined | 352.8 | 1.85 | 258.7 | 44,736 |
| model, inlined, end walk proved away | 240.6 | 1.26 | 260.3 | 44,732 |
| way (a): a kind that frees nothing one by one (no word, no walk) | 191.0 | 1.00 | 179.5 | 28,212 |
| way (b): chunks find their arena (no word; the walk stays) | 253.9 | 1.33 | 187.3 | 28,140 |
| Rust `Box<T, &Bump>` / `&'a T` / index | 375.7 / 209.3 / 185.2 | 1.97 / 1.10 / 0.97 | | 45,108 / 28,584 / 37,672 |

**lists** (2^18 growing lists per frame; 5 runs):

| program | clang ms | x arena C | MSVC ms | peak KB |
|---|---|---|---|---|
| plain C, arena | 216.8 | 1.0 | 241.3 | 17,052 |
| model (runtime calls) / allocation inlined | 278.2 / 247.9 | 1.28 / 1.14 | 272.9 / 250.7 | 19,140 |
| way (a+): the arena known where the list grows / way (b) | 240.1 / 243.1 | 1.11 / 1.12 | 244.0 / 254.0 | 17,112 / 17,100 |
| Rust `Vec<T, &Bump>` | 232.5 | 1.07 | | 19,512 |

**churn** (a pool that frees single values, two instances; 5 runs):

| program | clang ms | x pool C | MSVC ms | peak KB |
|---|---|---|---|---|
| plain C, pool | 289.6 | 1.0 | 355.8 | 13,744 |
| model (the word) | 384.2 | 1.327 | 396.2 | 15,796 |
| way (b) | 295.7 | 1.021 | 355.4 | 13,740 |
| Rust `Box<T, &Pool>` | 335.8 | 1.160 | | 17,564 |

What these show:
- **The word alone** (inlined, no walk, against way (a)) costs:
  - +26% under clang and +45% under MSVC on small values;
  - +3% on lists;
  - +30% on a pool that frees, against way (b), which differs only by the word.

  Its memory cost is +58% on small values, +12% on lists and +15% on the pool.
- **Way (a)** removes all of it where values never grow. **Way (b)** removes it where they do (lists
  +1%) and for kinds that free (+2%).
- **Two implementation costs, not the rule's:**
  - the runtime call per allocation, +12% under clang (trees);
  - the end walk over values that run nothing: +47% under clang, but nothing under MSVC (not settled).
- **These decide owner question Q-F.**

## 8. `this` never `noalias` (M4)

| workload | plain C | plain C `restrict` | model | salvage `typeproof` (model + `restrict`) | salvage `hoist` | Rust |
|---|---|---|---|---|---|---|
| this_tick, clang | 271.4 | 271.6 | 267.3 | 266.0 | **107.6** | 254.3 |
| this_tick, MSVC | 253.2 | 255.1 | 164.1 | 166.2 | 157.7 | |
| this_tally, clang | 289.5 (*) | 51.9 | 91.2 | 96.7 | 81.2 | 85.5 |
| this_tally, MSVC | 288.0 | 81.9 | 114.4 | 114.6 | 119.1 | |
| this_mix, clang | 76.5 | 60.9 | 77.4 | **61.9** | 153.8 | 61.0 |
| this_mix, MSVC | 65.1 | 60.1 | 65.5 | 59.9 | 154.9 | |

- **The missing `noalias`** costs nothing in tick and tally (model against `typeproof`).
- **In mix it costs 25% under clang and 9% under MSVC**, and `typeproof` removes it.
- **`typeproof`** gives `this` noalias for a method when no pointer it uses can lead to an object of its
  class. It is `[SIMD-3]`'s existing permission, widened from a loop to a method. Nothing runs while the
  program runs.
- **`hoist` is not a general fix as written** (verified, assembly). It is 2.5x faster in tick. In mix it
  turns `if v > peak: peak = v` into a `maxsd` on every element, a serial chain, where the model has a
  branch that is almost never taken. So it is 2x slower there.
- (*) **Not settled:** plain C's tally runs 3x slower than the model. Its loop (assembly read) uses
  memory read-modify-write instructions where the model uses separate loads and stores, but that is not
  shown to be the cause. That baseline is left out of the verdicts.

## 9. One whole variable under two names (M4, samevar)

| program | `one` (one span twice), clang / MSVC | `two` (two spans), clang / MSVC |
|---|---|---|
| plain C | 325.8 / 328.7 | 100.1 / 129.0 |
| plain C `restrict` (refused for `one`) | | 90.9 / 110.3 |
| model | **89.6 / 99.3** | 109.9 / 119.8 |
| salvage `generic` (no noalias copy) | 325.7 / 330.9 | |
| Rust (hand-written one-name function; two `&mut`) | 88.8 | 109.9 |

- **Verdict 88's one-name copy is 3.6x faster than plain C under clang and 3.3x under MSVC**, because
  plain C must give up `noalias` to serve both kinds of call.
- **Code (measured with `llvm-objdump`):**
  - the one-name copy is 247 bytes, against 508 for the normal copy;
  - the worst case measured, all 15 copies of a 4-name function, is 8,324 bytes (11.6x).
- **Not settled:** the normal copy is 21% slower than plain C with `restrict` under clang (8.6% under
  MSVC), and equal to Rust. The model's per-span range checks and the entry test are the likely places,
  but they are not verified.

## 10. A `ref mut` span the callee may re-point, across a package (M4, respan; 5 runs)

| program | clang x C | MSVC x C |
|---|---|---|
| model (span by address, read back) | 1.008 | 1.188 |
| `const ref mut` / same package (two words) | 1.007 | 0.995 |
| salvage `flag` | 1.079 | 1.048 |
| length known only at run time: model | 1.005 | 1.016 |
| Rust `&mut &mut [f32]` / `&mut [f32]` | 1.597 / 1.360 | |

- **Under clang the re-pointable span costs nothing.**
- **Under MSVC it costs 19%** when the length is a constant, and 1.6% when it is not. So it is the lost
  constant, not the reload.
- The `flag` salvage helps MSVC (1.048), hurts clang, and is not adopted.
- Verdict 42's `const ref mut` already removes the cost wherever it is written.

## What follows from this

| cost | what changes | whose decision |
|---|---|---|
| a view of a `thread_shared` object (10x) | `plaindata`: no view count for a class whose end runs and frees nothing ([RC-5] point 2); epoch deferral for the rest, still to write | owner |
| weak counts on parent links (+7.8%) | `parentfree`: a parent link `[WK-17]` proves holds no weak count (R11) | owner |
| the instance word | way (a) and/or (b) | owner (Q-F) |
| the trampoline's TLS loads | the thread compare reads the TEB's thread id (`teb`); MSVC still to explain | lead: the rule's compare is unchanged, only how it is read |
| `this` without `noalias` | `typeproof`: noalias for a method when no reachable pointer can lead to its class ([SIMD-3] widened) | lead, under the delegation; only the compiler's freedom grows |
| the arena's runtime call and its end walk | inline the bump; prove the walk away when nothing runs | lead: implementation |

Still to find before any verdict rests on them:
- the region form faster than views;
- the normal copy against `restrict` C;
- MSVC's lost constant in `respan`;
- MSVC's `teb`;
- plain C's tally;
- MSVC's free end walk.
