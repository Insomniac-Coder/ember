# The owner questions of the 0.9.15_Hardened_11 gap sweep, in the order to ask them

From `phase2.json` (the drafts) and `phase3.json` (the critic: groups, order, clashes). Status: open unless marked.

| # | Group | Severity | Items | Question | Status |
|---:|---|---|---|---|---|
| 1 | `K01-sharedref-send` | high | J2-06 | May a struct holding a sharedref mut link be sent to another thread? | verdict 164 |
| 2 | `K02-thread-shared-not-threadshared` | high | J7-03 | May a type that is not safe to read from two threads be made thread_shared? | verdict 165 |
| 3 | `K03-task-ends-shares-owners` | high | J7-04 | May a scoped task end shares owners, whose count has no atomics? | verdict 166 |
| 4 | `K04-weakref-outlives-arena` | high | J5-02 | Can a weakref link to an arena-made object outlive the arena? | open |
| 5 | `K05-thread-shared-using` | high | J5-05 | May a thread_shared value take its memory from an arena or other manager? | open |
| 6 | `K06-sharing-a-local` | high | J5-06, J4-05 | What happens to a local made shared after it is made: do its views survive, and may the word be thread_shared? | open |
| 7 | `K07-raw-pointer-struct-safe-call` | high | J8-08 | May a safe C call take a struct that holds raw pointers? | open |
| 8 | `K08-export-arguments-from-c` | high | J8-09 | What may an exported function trust about the arguments its C host passes? | open |
| 9 | `K09-cpp-reference-result` | high | J8-06 | Must a C++ function returning a reference say where the reference points? | open |
| 10 | `K10-queued-drop-check-points` | high | J7-01, J7-02 | Where and when does a queued thread_shared value's drop run (inside a held lock, a walk, an idle worker, a C thread)? | open |
| 11 | `K11-unsafe-drop-override` | high | J5-16 | May a class's drop, or an override of a safe method, be an unsafe fn? | open |
| 12 | `K12-function-lent-to-std-and-kept` | high | J3-02, J6-12 | A function from a variable given to std that keeps it (thread.spawn, push, a returned adapter): lent, handed over, or owns? | open |
| 13 | `K13-weakref-to-static` | medium | J6-06 | May a weakref link point at a class object held in a static? | open |
| 14 | `K14-reload-shared-block` | medium | J7-16 | Does a reload migrate a struct that lives in a shared block? | open |
| 15 | `K15-foreign-const-globals` | medium | J8-07 | Is reading a foreign const global safe, the same way for C and C++? | open |
| 16 | `K16-unsafe-import-cpp` | medium | J8-11 | Does import cpp need unsafe, given that the header's code runs before main? | open |
| 17 | `K17-read-only-parameter-promise` | medium | J5-07, J4-14 | Can a function change what its read-only parameter names, through a weakref made from it? | open |
| 18 | `K18-string-plus` | high | J2-04, J6-14 | Does String + read its left side and make a new String, or hand it over with a word? | open |
| 19 | `K19-written-init-call-words` | high | J1-07 | Does a call of a class's own init say owns before an owns parameter's argument? | open |
| 20 | `K20-anyerror-send` | high | J6-16 | Can a worker thread return Result[int], whose error is AnyError? | open |
| 21 | `K21-cpp-overloads` | high | J8-04 | How do C++ overloaded functions and constructors import into Ember? | open |
| 22 | `K22-opened-name-owns-call-result` | medium | J1-09 | Does a name opened from pop()'s result own the popped value? | open |
| 23 | `K23-match-subject-temporaries` | medium | J3-07 | Does a lock taken in a match subject stay held while the arms run? | open |
| 24 | `K24-owning-names-changeable` | medium | J4-03 | May a name that owns its value, like for job in owns jobs, be changed? | open |
| 25 | `K25-owns-takes-a-place-out` | medium | J4-04, J2-10, J5-12 | Does owns before a part or a downcast hand it over (body = owns req.body, owns cast s as Circle)? | open |
| 26 | `K26-copying-a-view` | medium | J4-08, J2-01 | What does copying a view give: w = this.rest of a StrView field, and copy of a StrView? | open |
| 27 | `K27-for-over-named-iterator` | medium | J3-09 | Does for t in toks: over a named iterator use it up, or continue after break? | open |
| 28 | `K28-untyped-list-of-text` | medium | J3-13 | With no type written, is ["ann", "bob"] a list of Strings, as {"ann", "bob"} is? | open |
| 29 | `K29-text-view-in-list` | medium | J6-13 | Can a text view be looked for in a list of Strings with in? | open |
| 30 | `K30-shared-owner-equality-and-identity` | medium | J2-09, J3-12 | On shared owners and links, does == compare contents and is compare identity? | open |
| 31 | `K31-c-kept-callback-declaration` | high | J8-01 | How is a C function that keeps an Ember callback declared and called? | open |
| 32 | `K32-super-init-memberwise` | medium | J3-05 | When a base class has no init, what does super.init(…) call? | open |
| 33 | `K33-init-and-field-defaults` | medium | J3-06 | Does a struct's init make a field's default first, even when it sets that field itself? | open |
| 34 | `K34-which-types-take-using` | medium | J2-02, J5-09 | Which heap-owning types may name a memory manager with using (String, Deque, …)? | open |
| 35 | `K35-weakref-dyn` | medium | J5-13 | Is weakref dyn Listener, a list of observers that does not keep them alive, a type? | open |
| 36 | `K36-unsafe-extend-vouching` | medium | J7-06, J2-12 | What may a program vouch for with unsafe extend: Send for a class, ThreadShared for a C wrapper? | open |
| 37 | `K37-parallel-array-elements` | medium | J7-11 | May a @parallel loop over a range write the elements of an Array? | open |
| 38 | `K38-parallel-reductions` | medium | J7-10, J7-09 | Must a @parallel loop declare its totals, and how is a max reduction written? | open |
| 39 | `K39-reduction-start-values` | medium | J7-08 | What does a task's private total start from, for -= on an unsigned field? | open |
| 40 | `K40-function-value-inline-state` | medium | J3-15 | When does a stored function value allocate: is the one-word state rule fixed or optional? | open |
| 41 | `K41-some-after-conversions` | medium | J2-05 | Does a value become Some after any other conversion, or only after four? | open |
| 42 | `K42-split-mutspan-halves` | medium | J4-09 | How does a program write through both halves of a split MutSpan? | open |
| 43 | `K43-weakref-into-cached` | medium | J6-02 | May a weakref link point into what a @cached method reads, and change it? | open |
| 44 | `K44-drop-changes-let-field` | medium | J3-10 | May a class's drop change what one of its let fields holds? | open |
| 45 | `K45-dyn-with-self-parameter` | medium | J2-08 | Can an interface whose method takes another Self be used as dyn? | open |
| 46 | `K46-ffi-handle-meaning` | medium | J8-14 | What does @ffi(handle) make a C type, and does a handle parameter need pointer words? | open |
| 47 | `K47-overlay-operator-names` | medium | J8-05 | How does an overlay name a C++ operator[] so that indexing can be safe? | open |
| 48 | `K48-migrate-from-ownership` | medium | J7-15 | How does migrate_from keep an open socket, and what happens to old fields? | open |
| 49 | `K49-ser-skip-field-value` | medium | J6-15 | What does a @ser(skip) field hold after a save file is read back? | open |
| 50 | `K50-adopt-ends-by-own-destructor` | medium | J8-13, J8-16 | What does adopt end an object with: taking back into_raw(), and a C++ object from a raw pointer? | open |
| 51 | `K51-ref-to-ptr-spelling` | low | J2-11 | How is a pointer made from a reference: one library name, two, or the cast? | open |
| 52 | `K52-c-keeps-address-ember-owns` | medium | J8-12 | How does code give C an object's address while Ember keeps owning it? | open |
| 53 | `K53-once-callback-token` | medium | J8-03 | Does a one-shot C callback give a token, and must it name an unregister function? | open |
| 54 | `K54-nondet-contested-lock` | medium | J6-07 | Which line makes a function nondeterministic when threads race for a lock? | open |
| 55 | `K55-nondet-math-functions` | medium | J6-08 | Which math functions count as nondeterministic in @deterministic code? | open |
| 56 | `K56-unsafe-at-compile-time` | medium | J6-04 | Can code with an unsafe block run at compile time? | open |
| 57 | `K57-export-threads-main-creator` | medium | J8-10 | May an export marked creator change a module variable, as one marked main may? | open |
| 58 | `K58-runtime-check-effect-kind` | medium | J1-02, J5-10, J6-01 | Does a runtime parameter's check count as a written panic, an alias check, or both? | open |
| 59 | `K59-runtime-placement-beyond-verdicts` | medium | J1-11 | May a field say runtime, so its methods' lines are checked while running (and a lambda parameter)? | open |
| 60 | `K60-gpu-frame-early-exit` | low | J7-19 | What happens when a GPU frame ends without end_frame, on an error? | open |
| 61 | `K61-arena-scope-views` | low | J5-19 | Does opening an arena scope end views made from the arena before it? | open |
| 62 | `K62-copy-of-on-loop-names` | low | J1-10 | May a loop name or a match pattern name say copy of, as exists as does? | open |
| 63 | `K63-copied-array-tuple-changed` | low | J4-11 | Is changing a copied fixed array or tuple refused, as for a copied struct? | open |
| 64 | `K64-list-pattern-arms` | low | J1-05 | May a match arm's pattern be a list pattern, a line starting with [? | open |
| 65 | `K65-several-interfaces-after-dyn` | low | J1-13 | Are several interfaces after dyn or some written in brackets, as needs are? | open |
| 66 | `K66-constructor-visibility` | low | J3-18 | May another module of the package build a struct whose fields are pub(package)? | open |
| 67 | `K67-bool-char-casts` | low | J2-14 | May as convert a bool or a char to any integer type? | open |
| 68 | `K68-pack-tuple-sizes` | medium | J2-07 | What is a tuple of a pack's types when the pack holds one type, or none? | open |
| 69 | `K69-soa-flattened-field` | low | J7-13 | What is ps.pos when pos is split into x, y, z columns? | open |
| 70 | `K70-raw-identifier-reserved` | low | J1-14 | Can a C field named owned or self be reached as r#owned? | open |
| 71 | `K71-nested-tuple-access` | low | J1-15 | Does t.0.1, element 1 of a nested tuple's element 0, compile? | open |
| 72 | `K72-comptime-read-file` | low | J6-05 | What does comptime.read_file give, and can its function run at run time? | open |
| 73 | `K73-det-ln-name` | low | J6-10 | Is the deterministic natural logarithm det.ln, like std.math's ln? | open |
| 74 | `K74-anyerror-core-layer` | low | J6-11 | Can firmware with no heap write Result[int], whose error is AnyError? | open |
| 75 | `K75-overlay-header-mismatch` | low | J8-15 | What happens when an overlay's first line names a different header from the import? | open |
| 76 | `K76-callback-table-full` | low | J8-18 | What happens when a program registers more C callbacks than the table holds? | open |
