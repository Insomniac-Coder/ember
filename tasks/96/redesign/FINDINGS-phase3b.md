# Phase 3, done again on the owner's job: what it found (2026-10-24)

This run followed [JOB.md](JOB.md): speed on par with C and Rust, memory safety the same as Rust's
or better, Python-like. Each annoying part of Rust gets one of three kinds of answer: a Pythonic
spelling of Rust's guarantee, a smarter compile-time solution, or a new mechanism that meets all
three goals. Nothing is called a failure before it has been saved where it can be.
**Nothing is decided and nothing in the compiler was changed.** The full reports are beside this
file, and every number below comes from them.

## 1. The result

The agents answered the annoyances group by group:
- containers and loops;
- returning and keeping references;
- ownership and sharing;
- graphs, callbacks and dynamic types;
- threads, locks and unsafe.

Each group was attacked by an adversary, which then saved whatever it could. The surviving answers were put together into **one memory model**, and that model was attacked again as a whole. The second attack found 6 problems, 4 of them memory-safety holes, and **all 6 were fixed**. Then the model was measured.

**It runs on par with plain C laid out the same way on all four test programs, and faster than safe Rust on all four.** The timings are provisional (section 7).

## 2. The model, in plain words, with real code

1. **Two kinds of values.** A `struct` (and every list, map, string and number) lives in one place. A `class` object lives on the heap and is shared through counted handles, as every Python object is.
2. **`b = a` gives a second name for the same value, as in Python, never a hidden copy.** The compiler remembers each name as a path ("element 3 of `enemies`"). It holds the address only while nothing can change what the name points into, and looks the place up again after something may.
3. **Every function gets an "effect summary"**, worked out from its body: the places it may change in a way that frees or moves memory. The compiler never keeps a pointer across a call that may do that. This one rule replaces Rust's "one writer or many readers", its lifetimes, `RefCell`, and Ember's run-time access checks today.
4. **Links that must not own are written as Python writes them.**

   A parent pointer, from the combined report's tree program:
   ```
   class Node:
       value: i32
       parent: Option[Node] = backlink(children)    # kept in step with the parent's children list
       children: Array[Node] = []

   nodes[new_parent].children.push(nodes[i])       # moves the node: out of the old parent, in here, parent set
   ```
   Reading `parent` is a plain memory load, with no count and no check.

   A reference to another game object, from the game-object program:
   ```
   open class GameObject:
       target: weak GameObject = None             # reads as None once the target is destroyed

   class Turret(GameObject):
       override fn update(self):
           t = self.target
           if t is not None and (t.x - self.x).abs() + (t.y - self.y).abs() < 300:
               t.health -= 1
   ```
   Each read of a `weak` field costs one bit test.
5. **Closures capture variables the way Python's do.** The compiler picks copy, move, pointer or a counted box. A handler can remove itself from inside its own call.
6. **Threads are checked from the same effect summaries.** Nothing called `Send`, `Sync` or `Arc` is ever written. A lock guards what is only touched while holding it, and `+=` from several tasks becomes a total per task, added together at the end.

What still runs while the program runs:
- a count change when a class handle is stored;
- bounds checks the compiler cannot prove away;
- a compare when a list loses an element while a name into it is in use;
- the search-and-move C does when a node changes parent;
- one bit test per `weak` read;
- one counted box per stored closure;
- locks or atomic counts only where threads really share.

## 3. Measured

Timings are medians of 5 runs on the P-cores, on mains power. All 33 programs gave the known checksum on all 390 timed runs. Ratios are to plain C.

| Program | Plain C | The model | Safe Rust |
|---|---|---|---|
| W1, the ECS (EnTT owning group) | 91.8 ms | 88.8 ms (**0.97**) | 106.0 ms (1.15) |
| W2, tree with parent pointers | 622.2 ms | 720.3 ms (**1.16**; 1.01 against plain C laid out the same way) | 1080.6 ms (1.74) |
| W3, game objects referring to each other | 440.9 ms | 423.2 ms (**0.96**) | 574.1 ms (1.30) |
| W4, callbacks that remove themselves | 23.3 ms | 21.3 ms (**0.91**) | 31.9 ms (1.37) |

Notes on the table:
- **W2's memory is 1.40x plain C's.** All of it comes from the data layout: a list of node objects, which is the shape of the Python program. Plain C in the same layout uses the same memory (76,664 KB against 76,724 KB).
- **The W4 Rust figure** is for a handler that removes itself from inside its own call, which is what the workload asks for. Rust written differently, with the handler returning "remove me" instead, takes 16.8 ms.

## 4. Your ECS concern

- **The memory model no longer costs anything on the ECS.** The movement loop takes 88.8 ms, against 91.8 ms for plain C in EnTT's owning-group layout.
- **What's left is EnTT's layout itself.** The same work in plain C with the archetype layout (all of an entity's components side by side in one record) takes 44.0 ms, half the time. That's a choice about how the ECS library stores its data, not a memory-model question.
- **The adversary found one ECS problem, now fixed.** When entities join or leave an EnTT group, the group swaps them around inside the pools, so a running loop could visit some entities twice and skip others. In the adversary's test, 5,000 random loops made 39,602 wrong visits, and W1's loop written as a view over the group gave the wrong result. EnTT's own docs state the same limit. The fix keeps new entities waiting until the loop ends and measures 0.94 to 1.02 of plain C.

## 5. What still doesn't meet all three goals

1. **A loop that removes from its own list while calling code the compiler can't see,** where that code may also change the list, is a compile-time error (Rust's rule). A run-time version runs as fast as C inside the loop, but it costs something elsewhere and can stop the program where Python works.
2. **A struct element's name used after such a call** is a compile-time error. You copy it first or make it a class.
3. **One value in two owning places** needs `copy(x)` or a `class`. Python shares one object.
4. **Cycles of owning links leak,** as in Rust and Swift; Python's collector frees them. A child of a dead parent sees `None`.
5. **Shared class objects that are re-pointed often** run at 1.16x to 1.17x of plain C (Rust's `Rc` is 1.31x to 1.42x). That's above the 15% line, and no fix was found. The C-speed way to write it is structs in an `Array`, referred to by position.
6. **A loop that calls a stored callback whose effects are unknown** pays a count pair per step: 1.27x to 1.32x.
7. **Threads:** a class object written by several tasks needs a lock, or a "claim" costing 1.12x to 1.36x (1.9x to 4.0x when the objects are also held elsewhere).
8. **Generics:** "the most specific implementation wins" is unproven. Python's duck-typed method call and its `acc = 0` sum have proposed rules that you would need to accept.

## 6. Choices waiting for you

1. **How `weak` works.** Either a test on each read, or registering every weak pointer so it can be cleared.

   | | Time | Memory |
   |---|---|---|
   | Test on read | 1.13x to 1.16x of registration | 1x |
   | Registration | 1x | 1.30x of the test |

   The time gap straddles the 15% line.
2. **The loop in item 1 of section 5:** a compile-time error, or the run-time version.
3. **Item 8 of section 5:** whether to accept the proposed generics rules (duck-typed method calls, the `acc = 0` sum).
4. **The ECS layout:** EnTT's owning groups (your choice), or archetypes, which ran in half the time.

## 7. Caveats

- **The timings are provisional.** Other agents were running while the measurer worked, and W2's runs spread 28% to 52%. They need the usual quiet-machine run before anything is decided on them.
- **The model exists on paper and as C programs written to be exactly what its compiler would emit.** Nothing is built in the Ember compiler.
- **Its safety is argued and attacked, not proven.** Every hole found was shown in C with a memory checker (AddressSanitizer), and then fixed.
- **The error-box problem didn't spoil any results.** Every timing line used carries its program's checksum, so every number comes from a program that ran to its end. The salvage numbers were re-run without `cmd`, and all came within 0.10 of the final batches except safe Rust on W2, which came out slower (1.74 to 1.94 against 1.48). That makes Rust look worse, not the designs.
- **Restarting the run re-ran more than intended.** It re-ran the five group adversaries and the step that combines the answers, not just the last two steps, which cost extra time. The earlier reports are kept as `*-first-run.md` and `superseded/`.

## 8. Last night's three designs, salvaged

Final measurements, ratio to plain C:

| Design | W1 | W2 | W3 | W4 | What no fix removes |
|---|---|---|---|---|---|
| copy instead of share | 1.26 | 1.40 | 0.80 | 1.05 | lists are values: `matrix.append(row); row.append(1)` gives `[[]]` |
| checked pointers | 1.21 | 1.37 (1.87 without a whole-program fact) | 0.95 | 0.99 | an object dies with its one owner even while referenced |
| today's rules, but never an error | 1.13 | 1.13 (2.21 without a whole-program fact) | 0.96 | 0.97 | needs an `own` word, or line order decides when objects die |

All three were rated "partly saved". Their good parts went into the combined model:
- objects allocated from fixed-size pools;
- dispatch by class number;
- protecting a stored closure only around the rare call that may remove it;
- accessors that yield a part of an object;
- looking a name up again after a change.

## 9. Files

| Report | What's in it |
|---|---|
| [phase3b-combined.md](phase3b-combined.md) | the model, all four programs beside Python, and the C each compiles to |
| [phase3b-combined-attack.md](phase3b-combined-attack.md) | the 6 problems and their fixes |
| [phase3b-measurements.md](phase3b-measurements.md) | the timings |
| [phase3b-salvage.md](phase3b-salvage.md) | last night's designs |
| `phase3b-<group>.md` and `phase3b-<group>-attack.md` | each group's answers and their attacks |

Code is in `artifacts/redesign2/`, a copy of `target/redesign2/` without the build files.
