# Two counters for `thread_shared` objects: safety and speed (the lead's, 2026-10-27)

## The question

The owner asked for this after the measuring phase. Today's rule (0.9.15: `[RC-4]`, `[RC-5]` point 2)
counts every owner of a `thread_shared` object with locked instructions. That includes the +1 and −1
around each opening through a `weakref`. It made the camera program 10x slower than C.

The owner kept the counter and asked for a safe way to make it cheaper. It has to work for the language
as a whole, not for one benchmark.

## The idea: biased reference counting

This is a published technique: Choi, Shull and Torrellas, "Biased Reference Counting", PACT 2018. The
design and every rule below are in `brc.h`.

- Every object has two counters.
  - **The home counter** is written only by the thread that made the object, with plain adds.
  - **The away counter** is used by every other thread, with locked adds.
- An object lives while home + away > 0.
- When a thread that did not make the object drops one of the maker's owners, the away count goes below
  0. The object is then handed to the maker's queue, and the maker merges the two counts at its next
  safe point.
- **Fix 1.** The maker's id is read from the header of the 64 KiB memory chunk the object came from. That
  header is written once and never changes, so reading it never pulls the object's own memory between
  cores.
- **Fix 2.** When a thread stores an owner where other threads can reach it, it merges its count at
  once. Such places are a lock-guarded `thread_shared` container, a channel, or a spawned thread's
  capture, and all of them are visible when compiling. Direct hand-offs then never use the queue.
- **Rule Q.** It was found by the stress test (below). A drop that sends an object to the queue marks it
  queued in the same atomic step as the drop. A queued object is destroyed only by the queue's
  processing.
- **Shortcut.** Once an object is merged, a −1 is a single locked add again, instead of a read and a
  compare-and-swap.

## Safety: the stress test (`stress.c`)

**What it does.** Real threads, 2 to 32 at once, in three waves; a later wave works on objects whose
making thread has ended. Each thread does random operations:
- make, copy and drop objects;
- put objects into shared places and take them out, under locks;
- share copies;
- link objects with `weakref`s and open them;
- nest one object inside another, publish the outer one, then take the inner one out on another thread;
- run safe points.

**What it checks.**
- A read through an open link never sees a destroyed object.
- Each object is destroyed exactly once.
- A block is freed only after its object was destroyed, and only once.
- At the end, every object made was destroyed and every block freed.

Blocks are never reused in this build, so any read of a freed block is caught.

| version | runs | objects made | reads checked | result |
|---|---|---|---|---|
| first version, today's rule and two counters | 90 | 35.3 M each | 33 to 36 M | all passed |
| version 2 (fixes 1 and 2), no nesting | 135 | | | all passed |
| with nesting, before rule Q | 180 | about 41 M each | | **1 failure** (fix 1+2, 32 threads, one shared place, seed 22): a block freed twice |
| with nesting, after rule Q (all four designs) | 260 | 68.6 to 68.8 M each | 39.9 to 47.7 M | all passed |
| the shortcut | 65 | 68.8 M | 40.2 M | all passed |

**The failure was a real race in my design**, present in every two-counter version and rarer without
fix 2:
1. a thread dropped the last owner it held and marked the object queued;
2. before it pushed the object onto the maker's queue, the maker merged, saw a total of 0, and destroyed
   and freed it;
3. the dropping thread then touched freed memory.

Rule Q closes it: the mark and the drop are one atomic step, and only the queue's processing may then
destroy the object.

The test reached every path: 22.8 million queued drops; 26,000 queues drained by another thread after
their thread ended; "merged while queued" 46,882 times.

One guard was never reached: the case where a drop finds a merge half done, but only in the version that
keeps the maker's id in the object. It is impossible by construction, because the merge now writes the
id after its compare-exchange.

## Speed (`perf.c`), 15 runs per cell

**Setup.** Five patterns, 1 to 8 threads, the eight performance cores only, mains power, clang `-O2`.
Medians are taken over 3 sets of 5 rounds.

**Harness bug, found and fixed.** The per-thread counters were first placed side by side in memory.
Depending on the allocator, two threads' counters sometimes shared one cache line, which made whole
2-thread runs 4x slower. Every multi-thread number taken before the fix is withdrawn, including version
1's matrix. The single-thread camera program was not affected.

**Where the cores matter.** The round trip of one cache line between two performance cores is 64 to
87 ns, depending on the pair (`pingpong.c`). That is a 1.4x range, too small to explain the 4x the
harness bug caused.

**Results.** No Kernel-Power 105 event in the window. The columns are:
- `atomic`: today's rule;
- `biased_chunk`: fix 1;
- `biased_pub`: fixes 1 and 2;
- `biased_pubfast`: fixes 1 and 2 plus the shortcut.

Ratios under 1 mean faster than today's rule. The raw runs are in `lead/runs/brcperf4*.txt`.

| pattern | threads | atomic ms (spread) | biased_chunk ms (spread) | biased_pub ms (spread) | biased_pubfast ms (spread) | biased_chunk / atomic | biased_pub / atomic | biased_pubfast / atomic |
|---|---|---|---|---|---|---|---|---|
| own | 1 | 43.6 (5%) | 32.3 (8%) | 32.3 (10%) | 32.6 (11%) | 0.741 | 0.741 | 0.748 |
| own | 2 | 43.4 (3%) | 32.5 (2%) | 32.4 (3%) | 32.7 (7%) | 0.749 | 0.747 | 0.753 |
| own | 4 | 44.3 (46%) | 33.1 (7%) | 32.6 (7%) | 33.0 (5%) | 0.747 | 0.736 | 0.745 |
| own | 8 | 46.6 (57%) | 35.3 (17%) | 34.7 (10%) | 34.8 (17%) | 0.758 | 0.745 | 0.747 |
| read | 1 | 87.6 (3%) | 104.6 (3%) | 102.8 (5%) | 90.1 (7%) | 1.194 | 1.174 | 1.029 |
| read | 2 | 128.4 (3%) | 113.1 (3%) | 113.3 (3%) | 106.3 (3%) | 0.881 | 0.882 | 0.828 |
| read | 4 | 166.0 (5%) | 122.8 (3%) | 123.7 (3%) | 122.9 (2%) | 0.740 | 0.745 | 0.740 |
| read | 8 | 200.3 (9%) | 139.1 (8%) | 138.7 (15%) | 143.5 (14%) | 0.694 | 0.692 | 0.716 |
| pipe | 1 | 98.9 (6%) | 81.9 (5%) | 124.8 (5%) | 125.2 (6%) | 0.828 | 1.262 | 1.266 |
| pipe | 2 | 562.8 (28%) | 458.5 (46%) | 560.4 (34%) | 567.4 (35%) | 0.815 | 0.996 | 1.008 |
| pipe | 4 | 551.4 (25%) | 548.8 (52%) | 606.4 (43%) | 584.3 (15%) | 0.995 | 1.100 | 1.060 |
| pipe | 8 | 691.0 (55%) | 630.2 (62%) | 682.7 (78%) | 673.0 (21%) | 0.912 | 0.988 | 0.974 |
| fanout | 1 | 17.4 (11%) | 17.5 (5%) | 17.6 (6%) | 18.2 (15%) | 1.006 | 1.011 | 1.046 |
| fanout | 2 | 22.9 (9%) | 21.7 (34%) | 21.6 (29%) | 21.2 (7%) | 0.948 | 0.943 | 0.926 |
| fanout | 4 | 30.1 (6%) | 29.8 (4%) | 29.8 (10%) | 28.6 (7%) | 0.990 | 0.990 | 0.950 |
| fanout | 8 | 41.8 (31%) | 44.3 (23%) | 44.1 (11%) | 42.2 (7%) | 1.060 | 1.055 | 1.010 |
| mixed | 1 | 27.3 (11%) | 27.0 (5%) | 29.4 (3%) | 29.7 (10%) | 0.989 | 1.077 | 1.088 |
| mixed | 2 | 33.2 (4%) | 36.3 (8%) | 36.3 (10%) | 36.6 (6%) | 1.093 | 1.093 | 1.102 |
| mixed | 4 | 36.8 (7%) | 40.2 (6%) | 40.3 (6%) | 40.4 (7%) | 1.092 | 1.095 | 1.098 |
| mixed | 8 | 43.5 (52%) | 47.5 (96%) | 47.0 (161%) | 49.2 (161%) | 1.092 | 1.080 | 1.131 |

How to read it, for fixes 1 and 2 plus the shortcut:

| pattern | against today's rule |
|---|---|
| a thread's own objects | **25% faster**, from 1 to 8 threads; the camera program is 6x faster (89 ms against 544 ms, `M1/lead_weakview.c`) |
| reading another thread's objects | 3% slower on 1 thread; **17% to 28% faster** on 2 to 8 threads |
| one thread handing copies to all threads | the same (0.93 to 1.05) |
| a random mix | **9% to 13% slower**, because it does more locked steps |
| a pipeline (made on one thread, dropped on the next) | 27% slower on 1 thread (the hand-over merge, then every step locked), the same on 2 to 8 threads (spreads up to 78%) |

Fix 1 without fix 2 (`biased_chunk`) is faster in the pipeline (0.83 on 1 thread) and as fast in the
mix on 1 thread. The cost is the later destruction of objects dropped on other threads (see "What
changes for programs").

## What was confirmed, and how

| finding | how it was confirmed |
|---|---|
| reading the maker's id from the object makes cross-thread copying 13% to 44% slower (cache-line traffic) | fix 1, the same code with the id in the chunk header, removes it (fanout 0.95 to 1.05) |
| the queue costs locked steps in a pipeline | counted: about 8.0 locked steps per object with two counters, against 7.1 with today's rule |
| the random mix is about 10% slower because it does more locked steps | counted: 0.96 locked steps per operation against 0.81 (18% more) |
| one thread reading others' objects was 20% slower with the same number of locked steps | it was the read-then-compare-and-swap −1: the shortcut takes it from 102.9 ms to 90.5 ms (today's rule: 86.5 ms) |

## What changes for programs

1. **Memory.** A `thread_shared` object's header grows from 16 to 32 bytes in this model.
2. **When an object is destroyed.** Today, the thread that drops the last owner destroys the object at
   once. With two counters, one case is different: the last owner is dropped on a thread that did not
   make the object, and the object was not handed over through a publish point (for example, it was
   reached inside another object). Then the object is destroyed when its maker next reaches a safe point,
   at the latest when that thread ends. Until then, another thread can still open it through a
   `weakref`. That is safe, because the object is intact, but its `drop` runs later. Objects handed over
   through a publish point (fix 2) are destroyed at once, as today.
3. **Safe points.** The compiler places them, for example at loop back-edges and before a thread blocks.
   That is still to design.
