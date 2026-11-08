# Report: pilot (method 4; nothing here is in the specification)

| question | status | agents | decisions | open objections | tests kept |
|---|---|---:|---:|---:|---:|
| K07-raw-pointer-struct-safe-call | rejected | 9 | 4 | 8 | 61 |
| K20-anyerror-send | rejected | 9 | 3 | 5 | 66 |
| K10-queued-drop-check-points | rejected | 11 | 5 | 5 | 59 |

## K07-raw-pointer-struct-safe-call (P04; J8-08): **rejected**, 9 agents

Outcome: `decided_by_parameters`. Intent: Have a C routine zero a pixel buffer the program owns, where the buffer reaches C as a pointer and a length inside a struct.

Where the cause is: FFI-2, p16-ffi.md 33-42: the safe-to-call fact. Three spellings assert it for a call: `safe fn` (FFI-10), an overlay function entry (FFI-2 36-38; GRM-35 277-281: 'a function entry, which asserts the function safe to call') and `@ffi(safe)` on a function-pointer typedef (FFI-56, 596-598). The fact has no limit on raw pointers. The only limit is FFI-10's sentence at 82-84, and it covers one spelling and one position: a raw-pointer parameter or result of a hand declaration. A contract (FFI-1, 26-32) counts only pointer-typed parameters and results; contract words attach only to those slots (FFI-11, 284-286), and an overlay never changes a type's fields (FFI-30a, 210-212). So a raw pointer in a struct field, in an array or span element, in a C++ method's object, or returned by a callback the call gives C, is in no contract and under no limit. (reported at: FFI-10 (p16-ffi.md 59-61, 82-84), FFI-11 (283-309), FFI-30a (210-212), FFI-56 (596-598)). Why there: ODR-073's reason (p26-appx-h.md 361; OWNER-QUEUE.md 3672-3673: 'a safe caller may pass a null, dangling or forged pointer; the contract cannot establish validity') is a property of what any assertion of safety can cover. It is not specific to hand declarations or to parameter slots. Safe code makes such a pointer: `null[P]()`, `ref_to_ptr`, `as_mut_ptr()` (p04-types.md 340-348, p07-ownership.md 837-838), a pointer-cast constant (FFI-6a, p16 150-151). It can then put that pointer in a struct field or a span element. So the limit belongs where the fact is defined, and every spelling that asserts the fact shares it. FFI-11 and FFI-30a are right as written: they explain why a field pointer can never be contracted, which is why no assertion may cover it. FFI-10's and FFI-56's lists accept 'a `@layout(c)` value type' only because no limit sat above them, so each list gets a cite, not a rule of its own. A fourth assertion, `@ffi(std_function)` (p21-annex-c.md 67), has no limit at all, not even on a raw-pointer parameter, and its way out needs a spelling the language lacks. It is decision 4, for the owner, and its text is left as it is.

**In short: A C function declared safe takes a struct holding a raw `uint8_t*` and a length that safe code filled in. The change refuses such safe declarations, but its refusal sentence also catches imported `std::function` callables it meant to leave for you.**

step | who | what happens | result
1 | binding author | declares `clear_pixels(PixelBuf)` safe | refused, as intended; a wrapper over a checked span is the way
2 | overlay writer | imports a `std::function<void(Buf)>` (Buf holds `uint8_t*`) as a callable | read literally: refused with no line that compiles, though the change says it stays as today
3 | program | passes a `std::vector<Slice>`, or a C++ object whose override returns a dangling pointer | unclear whether refused; the change keeps them open for reasons that do not hold
4 | binding author | follows an existing message's 'or declare it safe' line | the next build refuses that line

Today: All of these compile, and safe code can hand C or C++ a null, dangling or too-short pointer.

* Send the change back for one revision: tie the refusal to the three spellings it names (blocking), and also fix the vector, override, message and tool points
* Pass it after only the blocking wording fix, with the vector and override routes listed for later
* Reject it and keep today (memory corruption from safe code stays)

Recommendation: Send it back for one revision: the main decision is sound, but one sentence reaches further than intended, and four smaller points are cheap to fix now.

Arbiter: The core of the change holds: a safe C declaration, overlay entry or `@ffi(safe)` typedef whose call would hand C a raw pointer the caller supplied, inside a struct, span, object or callback result, is refused, and a wrapper builds the struct from a checked view. It closes J8-08's memory corruption from safe code at compile time with no run-time cost. One objection blocks (E2-1, a contradiction). FFI-2's new consequence sentence, 'a declaration or entry whose calls the other rules make safe ... is E5002', is not tied to the three spellings sentence 1 names. Read literally, it refuses `@ffi(std_function)` entries, and arguably std.ffi bridge methods, with no compiling line. The unchanged FFI-17a, the edited codes row and the impact report all say these compile. Tying the consequence to the named assertions resolves it and leaves decision 4 with the owner. I1-3 makes the same point, non-blocking. Real but not blocking: E2-2 (`std::vector<S>` elements are reached or not depending on how 'as its Ember type declares it' is read; span reached, vector kept), E2-3 (a C++ override on an object argument returning a dangling pointer is excluded for a reason that is untrue there), E2-4 (FFI-10's plain-fn message and missing-fact messages offer lines the limit then refuses, against verdict 73 and DIA-25), E2-5 (bind --check/--report do not know the limit), E2-6 (the override message omits the import line), E2-7 (the codes row omits a method's object; line 248 not 249; a hard CLI-5 sentence). The holes in E2-2 and E2-3 already exist today; what the change gets wrong there is its reach and its report.

### Edit: FFI-2 (its second sentence) (add, tasks/spec-0.9.9/parts/p16-ffi.md 34-35)

**Before:**

> Supplying the facts, in an `@ffi(…)` attribute or an overlay (§XVI.4), makes the
>   call safe.

**After:**

> Supplying the facts, in an `@ffi(…)` attribute or an overlay (§XVI.4), makes the
>   call safe, within the limit on raw pointers below.

Why here: This is the sentence the limit qualifies. Without the cite, the paragraph would state a general claim that its own later sentences contradict.

Why general: Read alone, the sentence would still promise that complete facts always make a call safe. The cite keeps it true for every declaration.

### Edit: FFI-2 (the limit on raw pointers) (add, tasks/spec-0.9.9/parts/p16-ffi.md 41-42 (added after 42))

**Before:**

> stays unsafe to call. The diagnostic,
>   `E5002`, names each missing fact and the overlay line that supplies it (`[DIA-18]`).

**After:**

> stays unsafe to call. The diagnostic,
>   `E5002`, names each missing fact and the overlay line that supplies it (`[DIA-18]`).
>   No assertion of the fact — `safe fn`, an overlay function entry not written `unsafe fn`, or
>   `@ffi(safe)` on a function-pointer typedef (`[FFI-56]`) — covers a raw pointer (`*T`, `*mut T`) that
>   the caller's arguments carry to foreign code: an argument itself; a field or an element, at any
>   depth, of what an argument passes, by value or behind a contracted pointer, as its Ember type
>   declares it, a method's object included; or what an Ember function given to the call, as an
>   argument or in one, returns, itself or in its result. Safe code can make a raw pointer that is null
>   (`null[P]()`), that outlives what it points at or points at fewer elements than a length beside it
>   counts (`ref_to_ptr`, `as_ptr()`, §IV.5, `[SPN-8]`), or that holds a constant address (`[FFI-6a]`),
>   and nothing checks one. So a declaration or entry whose calls the other rules make safe, and whose
>   calls would carry such a pointer, is `E5002` where it is written. Its message names where the
>   pointer is held (`buf.data`) and gives the lines that leave the calls unsafe: `@safety("…")` above
>   the declaration or entry written `unsafe fn` (a hand declaration with its C ABI parameters,
>   `[FFI-10]`), or the typedef's entry without `@ffi(safe)`; and each call outside unsafe code in an
>   `unsafe:` block with a `# SAFETY:` note (`[DIA-25]`). For an entry in another package's overlay, it
>   gives that entry as an `override` in the importing package's own overlay (`[FFI-30c]`). A raw pointer
>   that foreign code hands Ember (a result, an `out` value, a callback's parameter) is outside this
>   limit; `[FFI-10]` decides a hand declaration's raw-pointer result.

Why here: This is where the safe-to-call fact is defined, and the spellings that assert it refer to it. A limit placed here applies to all of them at once. Placed after the DIA-18 sentence, it leaves the paragraph's 'An entry written `unsafe fn` asserts nothing of the kind' next to what it refers to. FFI-10's sentence at 82-84 stays true as the hand-declaration case of the same property.

Why general: It is one property of the safe-to-call fact: a raw pointer the caller supplied never reaches foreign code under an assertion of safety. Its reason is ODR-073's. It names no type and no construct, and it reaches every type that carries a pointer (a struct, a union, a fixed array, a span's elements, a nested struct) and every route by which the caller's arguments reach C (by value, behind `one`, `count(n)`, `exclusive` or `owns`, a C++ method's object, a callback's result). The reach is judged by the value's Ember type, so opaque classes, `extern type`s and bridge types, which declare no fields, are not reached. The trigger, 'whose calls the other rules make safe', is a property of what a declaration says, so an entry whose own words already leave its calls unsafe (an unknown fact, an uncontracted raw-pointer slot, `retained` on a plain pointer) keeps today's unsafe calls. The direction sentence states scope only. The message is given for each spelling, and each message line compiles (DIA-25).

### Edit: FFI-10 (add, tasks/spec-0.9.9/parts/p16-ffi.md 61-62)

**Before:**

> `@ffi(…)` on the declaration), `safe` on any other function being `E5002`, naming the fact that is
>   missing; or `unsafe fn`

**After:**

> `@ffi(…)` on the declaration) and within `[FFI-2]`'s limit on raw pointers, `safe` on any other function being `E5002`, naming the fact that is
>   missing or, under that limit, where the raw pointer is held; or `unsafe fn`

Why here: The question showed in this list: 'a `@layout(c)` value type' let `PixelBuf` through. For J8-08 no fact is missing, so 'naming the fact that is missing' alone would describe the wrong message.

Why general: A cite, not a rule of its own. Read alone, the list keeps naming every requirement a `safe fn` must meet, and the E5002 clause names what the message says in both cases.

### Edit: FFI-56 (add, tasks/spec-0.9.9/parts/p16-ffi.md 598)

**Before:**

> scalar or a `@layout(c)` value type (`E5002` otherwise)

**After:**

> scalar or a `@layout(c)` value type, within `[FFI-2]`'s limit on raw pointers (`E5002` otherwise)

Why here: FFI-56 grants a permission ('may assert ... when every parameter ... is ... a `@layout(c)` value type'). Without the cite it would contradict FFI-2's refusal of `@ffi(safe) type mix_fn`, whose struct parameter holds `float* samples`.

Why general: The typedef's list is FFI-10's list for C function pointers. The same cite makes both lists share FFI-2's limit, so they cannot disagree.

### Edit: XVI.4 example (the Vulkan overlay under GRM-35) (change, tasks/spec-0.9.9/parts/p16-ffi.md 239-241)

**Before:**

>     @ffi(effects=[FFI, Alloc], no_callbacks)
>     fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed one,
>                       pAllocator: nullable borrowed one, pBuffer: out one) -> status

**After:**

>     @safety("`pCreateInfo`'s `pNext` chain is null or valid and its `pQueueFamilyIndices` points at `queueFamilyIndexCount` indices; `pAllocator`, when given, holds valid callbacks and user data")
>     @ffi(effects=[FFI, Alloc], no_callbacks)
>     unsafe fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed one,
>                              pAllocator: nullable borrowed one, pBuffer: out one) -> status

Why here: Without this edit, the overlay that FFI-2 points readers to would itself be E5002. K46 (J8-14) changes `device: borrowed` on the same lines; the two edits touch different words and merge into one rewrite of the entry.

Why general: The spec's own example must compile under its rules. `VkBufferCreateInfo` holds `pNext` and `pQueueFamilyIndices`. `VkAllocationCallbacks` holds `pUserData`, and function-pointer fields whose results are raw pointers. So FFI-2's limit refuses the safe entry. The example uses verdict 130's spelling for a function unsafe to call.

### Edit: XVI.4 (a sentence under the example) (add, tasks/spec-0.9.9/parts/p16-ffi.md 249-251)

**Before:**

>     hide vkAllocationFunction
> ```
> 
> * `[GRM-35]`

**After:**

>     hide vkAllocationFunction
> ```
> 
> `vkCreateBuffer` is an `unsafe fn` entry because the structs its call hands Vulkan hold raw pointers
> (`pNext`, `pQueueFamilyIndices`, `pUserData`, `[FFI-2]`); a wrapper in an `extend` block of the overlay
> (`[FFI-13]`) gives safe callers a create call, building those structs from checked views.
> 
> * `[GRM-35]`

Why here: Part XVI opens with 'after which calls are safe' (p16 8-9). A reader who sees the section's headline Vulkan call turn unsafe learns here how a binding gives safe callers a create call.

Why general: One sentence, no second example. It names the tool the language already has for this case (FFI-13, p16 356-357: 'this is how an idiomatic API is layered over a raw one').

### Edit: E5002 (the codes table) (change, tasks/spec-0.9.9/parts/p17a-codes.md 164)

**Before:**

> `safe fn` on a function whose contract is incomplete, or two destructors for one foreign type (a call of a function without a verified contract outside `unsafe` is `E3100`) | `[FFI-2]`, `[FFI-10]`, `[FFI-36a]` |

**After:**

> `safe fn` on a function whose contract is incomplete; a `safe fn`, overlay function entry or `@ffi(safe)` typedef whose calls would otherwise be safe and would carry foreign code a raw pointer from the caller's arguments (one an argument is or holds at any depth, or one an Ember function given to the call returns); or two destructors for one foreign type (a call of a function without a verified contract outside `unsafe` is `E3100`) | `[FFI-2]`, `[FFI-10]`, `[FFI-36a]`, `[FFI-56]` |

Why here: The row already cites FFI-2. Without the reach and the 'would otherwise be safe' words, the row would refuse FFI-6b's `PAIR_FIRST(p: *Pair)`, which FFI-2 does not reach. FFI-56's own E5002 was missing from the rule column.

Why general: The registry lists every cause of the code (p17a 3-4), with the rule's own reach and trigger, so a compiler written from the registry agrees with the rule.

### Edit: CLI-5 (add, tasks/spec-0.9.9/parts/p17-toolchain.md 115-116)

**Before:**

> A declaration it leaves
>   unsafe to call is written `unsafe fn` (verdict 130).

**After:**

> A declaration it leaves
>   unsafe to call, one whose calls `[FFI-2]`'s limit on raw pointers keeps from being safe among them, is
>   written `unsafe fn` (verdict 130).

Why here: Without the cite, `ember bind --init pixels.h` would write `fn clear_pixels(buf)`, which FFI-2 refuses, and CLI-5's 'compiles unchanged' would contradict FFI-2.

Why general: The starter overlay's promise that it 'compiles unchanged' must hold for every header. The limit adds a way a declaration with every fact derived can be kept from being safe.

### Impact

Rules changed: FFI-2 (p16 34-35, add): its second sentence points to the limit. FFI-2 (p16 after 42, add): no `safe fn`, overlay function entry or `@ffi(safe)` typedef covers a raw pointer that the caller's arguments carry to foreign code: an argument itself, a field or element at any depth of what an argument passes (as its Ember type declares it, a method's object included), or what an Ember function given to the call returns. A declaration or entry whose calls the other rules make safe and would carry one is E5002 where it is written. The message is given per spelling, with an `override` line for another package's overlay. Pointers foreign code hands Ember are outside the limit. FFI-10 (p16 61-62, add): the list and the E5002 clause cite the limit. FFI-56 (p16 598, add): its list cites it. XVI.4 example (p16 239-241, change): `vkCreateBuffer` becomes `@safety("…")` above `unsafe fn`. XVI.4 (p16 after 249, add): one sentence on `extend` wrappers. E5002 row (p17a 164, change): the new cause, with its reach, and `[FFI-56]` added to the rule column. CLI-5 (p17 115-116, add): `--init` writes such a declaration `unsafe fn`. Text unchanged: FFI-1, FFI-11 (its `retained` row keeps its outcome; two-call enumerations whose items hold raw pointers are newly refused through FFI-2), FFI-11b, FFI-11c, FFI-30a, FFI-31b, FFI-17a (`@ffi(std_function)`: decision 4, for the owner), IV.5 (`ref_to_ptr` stays safe, ODR-075), FFI-10 91-92 and 109-110 (ODR-074).

Readability: The ordinary program that calls a binding gets no longer and no harder. A binding of a C API whose structs carry pointers writes one wrapper for each such function, which is the price Rust binding authors pay too. That price is heaviest for Vulkan, where every create-info has `pNext`. Hand bindings now refused also write their raw form (the pointer and the length by hand), because an unsafe hand declaration keeps its C ABI parameters; the fix for that is listed for the owner. Field contracts would let such structs become safe types, but that is a whole feature, which FFI-30a rules out today; it is listed for the owner.

Cost: Nothing runs: the limit is checked when compiling. A wrapper fills the struct from the span's pointer and length, as C code fills it, and makes the same single direct C call (FFI-9: one `call` instruction). To measure: the wrapper `clear` against a direct C call with the same pointer and length. The expectation is identical instructions after inlining, as in C. Report both times and both code sizes.

* verdict 31: a forgotten mark must refuse, not crash: kept (The change does not touch `@ffi(no_callbacks)`. A safe declaration whose call would hand C a raw pointer is refused, never let through.)
* verdict 60: anything that can break safety only inside unsafe: kept (It closes a route by which safe code broke memory safety. Such a call now happens only inside `unsafe:`, under its `@safety`.)
* verdict 118, 119: an unsafe fn's body is unsafe; anything inside unsafe is unsafe: kept (Unchanged. The wrapper's `unsafe:` block is ordinary unsafe code.)
* verdict 120: a C function in unsafe extern carries @safety: kept (A refused `safe fn` becomes `unsafe fn` with `@safety`, and the message gives that line.)
* verdict 121: every overlay is checked, other packages' too: kept (An entry the limit refuses fails every build that applies its overlay, as a drifted entry does. The message gives the line the importing package can write: an `override` in its own overlay (FFI-30c).)
* verdict 125: imported C and C++ functions say what callers must make true: kept (A refused overlay entry becomes `@safety("…")` above `unsafe fn`, C++ members included.)
* verdict 130: overlay unsafe fn with @safety: kept (The XVI.4 rewrite and every E5002 message use exactly that spelling.)
* verdict 132: an unwritten attribute never falls back to an inferred fact: kept (Decision 3: a plain overlay line keeps its written meaning, 'safe to call'. A line that cannot be made safe is refused where it is written, rather than made unsafe by a header field the reader of the overlay does not see.)
* verdict 133: hand-written declarations spell unsafe the same way: kept (Hand declarations use `@safety` above `unsafe fn`, and `@ffi(…)` keeps its other words (`no_callbacks`, `callback(…)`). No new contract word is added.)
* verdict 136: safe fn or unsafe fn, never plain fn: kept (Unchanged. Decision 3 follows its lesson: no line silently means unsafe.)
* verdict 137: a result's view is read from the callee's code (C has none): kept (Not touched. Results are outside the limit.)
* verdict 165: thread_shared needs ThreadShared: kept (Not touched.)
* verdict ODR-073 (delegated): a safe fn exposes the mapped safe type, never a raw pointer: kept (Its reason ('a safe caller may pass a null, dangling or forged pointer') is applied to a pointer in a field, an element or a callback result. The raw-pointer slot rule is kept word for word. Listed for the owner: its result half can only be about the API's shape (E1-3).)
* verdict ODR-074 (delegated): only a mapped safe fn erases the count witness: kept (Kept as written. The E5002 message gives a hand declaration's unsafe line in the raw form that this rule decides. A single mapping for safe and unsafe declarations is listed for the owner (E1-1).)
* verdict ODR-075 (delegated): `ref_to_ptr` is safe: kept (Making a raw pointer stays safe. Only handing one to C under an assertion of safety is refused.)
* verdict ODR-199 (delegated): `@ffi(std_function)` makes an owned callable value: kept (Not touched. Decision 4 asks the owner whether such a callable should be unsafe to call, as a C function pointer is (FFI-56).)
* a program changes: Compiles. C writes 4,096 bytes from the address of the one-byte `x`: memory corruption in safe code. -> E5002 at the `safe fn` line, naming `buf.data`. The message gives `@safety("…")` above `unsafe fn clear_pixels(buf: PixelBuf)` and line 12 in an `unsafe:` block with a `# SAFETY:` note. With a wrapper `clear(ref mut bytes: MutSpan[u8])`, the program calls `clear(ref mut frame)` and prints `0 0 4`.
* a program changes: Asserted safe. Safe code can give Vulkan a forged `pNext` chain, or a `pQueueFamilyIndices` that points at 1 index with a count of 1,000. -> Written as it stands, the entry is E5002 naming `pCreateInfo.pNext`, `pCreateInfo.pQueueFamilyIndices` and `pAllocator.pUserData`. With the example edit it is an `unsafe fn` entry with `@safety`, called in `unsafe:` or through an `extend` wrapper.
* a program changes: Compile. Safe code can hand C a null, dangling or short pointer. -> E5002 naming the field or the elements. The binding writes `unsafe fn` with `@safety` (a hand declaration in its raw form) and one wrapper, which builds the struct from checked views or keeps the foreign struct in a private field (MOD-2).
* a program changes: Compile. Safe code can put a null or destroyed window in the field (C++: a null `HWND` handed to a renderer). -> E5002 naming the field (`cfg.window`). K46 (J8-14) decides whether `@ffi(handle)` or `@ffi(newtype)` makes such a field something other than a raw pointer. That applies to every handle field, not only `vkCmdBindVertexBuffers`' `pBuffers`.
* a program changes: Compile. The driver reads each item's `pNext`, which safe code set, and writes through it. -> E5002 naming the items. The binding writes `unsafe fn` with `@safety` and a wrapper.
* a program changes: Asserted safe. -> E5002 at the typedef's entry, naming `blk.samples`. Without `@ffi(safe)`, calls through it stay in `unsafe:` (FFI-56's default).
* a program changes: Compile. A safe Ember function returning `ref_to_ptr(ref mut local)` makes C write into a dead stack slot. -> E5002 naming `get_buf`'s result.
* a program changes: Compile. -> E5002 at the entry, even in a program that never calls the function, as a drifted overlay already is (E5010). The message gives the entry as an `override` in the importing package's own overlay. The dependency's own build also fails until it writes `unsafe fn`. No `ember fmt --migrate` rewrite can make the program compile, since the calls then need `unsafe:`.
* a program changes: Writes `fn clear_pixels(buf)`. -> Writes `unsafe fn clear_pixels(buf)` (CLI-5), so the starter still compiles unchanged.
* a program changes: Compile as described. -> Compile exactly as before.

**Decision (made by the owner's parameters): What happens to a safe C declaration whose argument's struct holds a raw pointer: A C function declared safe takes `struct { uint8_t* data; size_t len; }` by value, and safe code can fill it with any address and length: may the safe declaration stand?**

step | who | what happens | result
1 | binding author | declares `clear_pixels(PixelBuf)` safe | refused: the struct carries a raw pointer
2 | binding author | declares it unsafe, with what callers must make true, and writes `clear(ref mut bytes: MutSpan[u8])` | compiles
3 | program | calls `clear(ref mut frame)` | only bytes it owns reach C
4 | program runs | one direct C call | the same cost as C

Today: The safe declaration compiles, and safe code can hand C a pointer to one byte with a length of 4,096; C writes past it.

* A: refuse a safe C declaration that hands C a raw pointer inside a struct; a wrapper builds the struct from a checked view
* B: keep today: the declarer's word covers the struct's pointers too (memory corruption from safe code)
* C: make writing a raw pointer into a struct unsafe instead (still a hole: the length beside it is a plain number safe code changes)
* D: let overlays give struct fields contracts (not in the language: overlays may not change a type's fields)

Recommendation: A: the only choice that closes the hole when compiling, with no cost while running, and callers get a shorter line.

Chosen: A. By: 1. The three goals: A keeps one direct C call, closes the hole, and gives callers a shorter line. B gives up safety. C gives up readability, since every line that puts a pointer into a struct needs `unsafe:`, and it still leaves the hole. D is not in the language. 2. Safety, never given up: only A is safe. Under B, safe code corrupts memory. Under C, `buf.len = 4096` is safe code and C writes past the buffer. 3. Compile time first: A is decided when compiling, and nothing runs. 4. A written word only for meaning: A's `unsafe fn` and `@safety` say who vouches and for what (verdicts 120, 133). 5. The ordinary program no longer or harder: callers write `clear(ref mut frame)`, which is shorter than building the struct by hand; a binding author writes one wrapper. 6. Consistency with the nearest rule and the verdicts: FFI-10's 'complete facts do not make a raw-pointer parameter ... safe to expose' and ODR-073's reason hold exactly for a pointer in a field; verdicts 130, 133 and 136 give the spelling.

**Decision (made by the owner's parameters): Which ways a raw pointer the caller supplied can reach C are covered: Besides struct arguments, which other ways can a call hand C a pointer the caller made, e.g. an allocator callback `uint8_t* (*get_buf)(size_t)` returning the address of a dead local?**

step | who | what happens | result
1 | binding author | declares `stream_read(want, get_buf)` safe; C writes `want` bytes at what `get_buf` returns | refused: the callback's result is a raw pointer
2 | binding author | declares `camera_latest_frame() -> FrameInfo`, holding C's pixel pointer | compiles: C hands it to Ember
3 | program | reads `f.width` | allowed
4 | program | reads through `f.pixels` | only in unsafe code, as today

Today: A callback that returns a raw pointer passes, and so does any struct C returns.

* i: only struct arguments (a callback returning a dangling pointer still reaches C from safe code)
* ii: every pointer the call's arguments carry to C: fields, elements, a C++ object, what a callback given to the call returns
* iii: ii plus what C hands back (refuses programs that only read C's struct, with no safety gained)

Recommendation: ii: covers every pointer a call hands C, and nothing C hands back, which is already unsafe to read through.

Chosen: ii. By: 1. The three goals: ii keeps every correct program that only reads what C hands back, and costs nothing while running. 2. Safety: i leaves a callback's dangling pointer reaching C from safe code; ii and iii close it. 3. Compile time first: all three are decided when compiling. 4. A written word only for meaning: none adds a word. 5. The ordinary program: iii refuses programs that only read C's struct (a camera frame, a display mode, an image loader); ii refuses none of them. 6. Consistency: ii follows the direction the value flows, as FFI-21a does for raw pointers given back to C, and leaves to IV.5 what IV.5 already fences.

**Decision (made by the owner's parameters): What a plain overlay line, or an @ffi(safe) typedef, that would hand C such a pointer becomes: An overlay line `fn cp_load(opts: borrowed one)` looks complete, but the header's struct holds a `const char*`: refuse the line, or quietly make its calls unsafe?**

step | who | what happens | result
1 | overlay writer | writes `fn cp_load(opts: borrowed one)` | refused at this line, naming `opts.path`
2 | overlay writer | writes it unsafe, with what callers must make true | compiles
3 | program | calls `cp_load` in unsafe code with a note | compiles
4 | another package using this overlay | builds before the fix | refused, told the line to write in its own overlay

Today: The line declares the call safe and is accepted.

* a: refuse the line, with the unsafe line to write
* b: accept it, and make its calls unsafe, as for a fact not yet written

Recommendation: a: a line means what it says; a plain overlay line means safe to call, so one that cannot be made safe is refused where it is written.

Chosen: a. By: 1. The three goals: both are safe and free at run time; a reads better, with one error at the line to change. 2. Safety: both refuse a safe call. 3. Compile time first: both. 4. A written word only for meaning: under b, a plain overlay line, which says 'safe to call' (GRM-35), would have unsafe calls because of a header field the overlay's reader never sees (verdict 132's lesson). 5. The ordinary program: a gives one error at the overlay line with the lines to write; b gives an error at every call, then L3015. Under a, a dependant that never calls the entry also fails until it writes the override or the dependency is fixed, as with a drifted entry today (FFI-55). 6. Consistency: a matches FFI-10's E5002 for `safe fn` and verdict 136 (no line silently means unsafe). The trigger is what the entry itself says ('whose calls the other rules make safe'), so FFI-11's `retained` row and FFI-11c's TODO words keep deciding their entries.

**Decision (for the owner): Whether a C++ std::function imported as a callable is limited the same way: A C++ `std::function<void(Buf)>`, where `Buf` holds a `uint8_t*`, imported as a callable: safe code can call it with any address. Should calling it be unsafe, as calling a C function pointer is?**

step | who | what happens | result
1 | overlay writer | maps `std::function<void(Buf)>` to a callable | today: safe to call
2 | program | calls it with a one-byte address and a length of 4,096 | today: C++ writes past the byte
3 | under a | the call goes in unsafe code with a note, unless the overlay vouches (it cannot vouch here: the struct holds a raw pointer) | the caller vouches
4 | under c | the overlay line is refused | Ember cannot call it at all

Today: A mapped `std::function` is safe to call whatever its parameters hold.

* a: unsafe to call unless the overlay says it is safe, which it cannot say for raw pointers (the C function pointer's rule)
* b: keep today (C++ can be handed a dangling pointer from safe code)
* c: refuse such an overlay line (Ember can then never call it)

Recommendation: a: the same rule as a C function pointer: unsafe to call unless someone vouches, and never vouched for raw pointers.

### What blocks it (1)

* E2-1 (X, contradiction): Two compilers built from the words split on every `@ffi(std_function)` entry whose parameters hold a raw pointer, and possibly on bridge-container methods. One reading refuses correct programs with no line that compiles. The other matches the impact report. Decision 4 is meant to be the owner's, but the words already decide it, the way the designer rejected.
  Arbiter: Walk: (1) FFI-2 as added, sentence 3: 'So a declaration or entry whose calls the other rules make safe, and whose calls would carry such a pointer, is `E5002` where it is written.' Its subject is any declaration or entry, not the three assertions sentence 1 names. (2) FFI-17a (p21 67): with `@ffi(std_function)` the value is 'a function value of type `fn(A…) -> R` ..., called as any other', so `type Fill` is an entry whose calls the other rules make safe. FFI-2 (p16 35-38) says the safe-to-call fact 'is never derived' and is asserted 'by an overlay entry for the function', so this entry is an assertion. (3) `fill(f.px.Buf(data=ref_to_ptr(ref mut x), len=1))` carries `Buf.data` by value, which is 'a field ... of what an argument passes, by value'. (4) So FFI-2 gives E5002 at overlays/fill.em line 3. The program is correct (len=1, one byte), and without the change it compiles and prints `0`. (5) The unchanged FFI-17a, the edited codes row (p17a 164: 'a `safe fn`, overlay function entry or `@ffi(safe)` typedef whose calls would otherwise be safe') and the impact report ('`@ffi(std_function)` entries (decision 4) ... Compile exactly as before') accept it. Two rules decide one program differently. The designer's own FFI-56 edit admits the pattern: 'Without the cite it would contradict FFI-2's refusal'. FFI-17a has no such cite. (6) The message clause gives no line for this spelling, only '`@safety("…")` above the declaration or entry written `unsafe fn` ... or the typedef's entry without `@ffi(safe)`', so DIA-25 ('The line a message gives compiles', p17 474) cannot be met. In effect the words apply decision 4's option c, which the designer rejected and left to the owner. Caused by the change: X.

```
// fill.hpp
#pragma once
#include <cstddef>
#include <cstdint>
#include <functional>
namespace px {
struct Buf { std::uint8_t* data; std::size_t len; };
using Fill = std::function<void(Buf)>;
inline Fill make_filler() noexcept {
    return [](Buf b) { for (std::size_t i = 0; i < b.len; ++i) b.data[i] = 0; };
}
}

# overlays/fill.em
# SAFETY: the entries below match fill.hpp
unsafe overlay cpp "fill.hpp":
    @ffi(std_function)
    type Fill
    @ffi(no_callbacks)
    fn make_filler()

# main.em (ember.toml has a [cpp.fill] section)
import cpp "fill.hpp" with (project="fill", overlay="overlays/fill.em") as f

fn main():
    fill = f.px.make_filler()
    x: u8 = 7
    fill(f.px.Buf(data=ref_to_ptr(ref mut x), len=1))
    println(x)
```

Arbiter's rulings on the open objections: 8 real, 0 refuted, 0 already in today's specification

Objections closed in the debate: 13 (12 fixed, 1 withdrawn after a rebuttal)

Gaps found on the way that exist without the change: 10

## K20-anyerror-send (P01; J6-16): **rejected**, 9 agents

Outcome: `decided_by_parameters`. Intent: Load a level file on a worker thread and give the main thread either its length or the reason it failed, using the language's default error type (`Result[int]`).

Where the cause is: Three places, one property each. (1) The definition of `Error`: IV.8's declaration (p04-types.md 676, `interface Error: needs (Debug, Display):`), which ERR-3 describes (p13-errors.md 87-88) and TYP-17 quotes (p04 459). (2) What an object of an `open` or `abstract` class holds: THR-8's list sentence (p11-concurrency.md 47-48, "class objects are `Send` when every component is") and the marker table (p04 577, "a class object is when its fields are") count only the fields of the object's declared class. That is narrower than THR-8's own definition (p11 43-45, "nothing reachable from it ... is a one-thread owner"), because an object of an open class may be an object of any class derived from it (TYP-5 rule 8, p04 86). THR-9 (p11 63-64) has the same gap, and it has no words for a `dyn` either. (3) IFC-3 (p05 411-413) says that implementers meet an interface's parents, but not that an implementation meets them for every type it covers, which takes in each instance of a generic implementation and each derived class. (reported at: ERR-3, ERR-8, THR-2, THR-8 (it shows through THR-10's "`R` MUST be `Send`", p11 175-176)). Why there: Every error is erased through `Error`: `dyn Error`, the box inside `AnyError` (ERR-8), and so the default error type of every `Result[T]` (ERR-9). THR-8's dyn clause is right as it stands (`dyn I` is Send only when `I` requires it). `Error` requires no thread markers, so every erased error is a one-thread value, and the default `Result[T]` cannot leave its thread by any route. ERR-8's `From`, THR-2's routes and THR-10 are each right as written. The missing words belong in what an error is. Once `Error` requires the markers, the requirement is only as sound as the markers of what an error holds. For a class, THR-8's list and the p04 row read the markers from the declared class alone. So a `GameError` that implements `Error` would be judged Send while one of its objects is a `DocError` holding a `shares` owner. That gap already exists for a plain open-class value handed to a thread. The change would carry it into every AnyError. So the class wording is part of the cause. It has to say what THR-8's definition already says. IFC-3 is where the language says what an implementation promises about its interface's parents, so the rule for generic implementations and derived classes goes there, once for every interface, and not in ERR-3 as a case of `Error`.

**In short: Making every error type safe for any thread (as Swift's errors are) lets the catch-all error (like C++'s `std::exception_ptr`) come back from a worker thread. But as written, an engine library's error that holds a `Base*` can still carry a game's `Derived` object, holding a non-atomic refcount, to another thread.**

step | who | what happens | result
1 | engine library | declares an error that keeps the failing `Script` object; checked alone it is thread-safe | allowed
2 | game (another package) | declares `Enemy`, a kind of `Script`, holding a one-thread shared owner | allowed: nothing checks it against the engine's error
3 | game | puts an `Enemy` into that error, then the error into the catch-all, and hands it to a thread | allowed, though today it is refused
4 | both threads | change the same one-thread count at once | race: freed memory read or freed twice

Today: A worker returning `Result[int]` is refused at `thread.spawn` without a word of explanation, so the race above cannot happen.

* Repair and keep D: a class derived later, in another package or by a reload, must keep the thread-safety the library relied on, and is refused at its own line with the fix
* D as written: unsafe across libraries, so not acceptable
* A: check each error where `?` puts it into the catch-all (the game's own build sees the Enemy, so it is safe), but generic error code and wrappers must write thread words
* B: keep today: the catch-all stays on its thread, and workers name their own error type

Recommendation: Repair and keep D. Ordinary worker code still compiles with nothing written, and the one-thread object is refused at the line of the game's class that holds it.

Arbiter: Not yet. The change answers J6-16 (direction D: every error is safe to hand to and read from any thread, so `Result[int]` comes back from a worker with nothing written). But it is blocked by one safety hole, reported twice (S2-1 and E2-1). Take a library error struct that holds an object of the library's own open class (an engine's `LoadError` holding the failing `Script`). It is judged thread-safe in the library's build, which cannot see the game's classes. A game in another package then derives `Enemy(Script)` holding a one-thread shared owner. The changed IFC-3 checks a derived class only when it derives from a class that implements Error, and Script implements nothing, so no line of the game is checked. AnyError is Send from Error's requirement alone (p11 51), so the owner crosses threads inside an AnyError and its plain count races (use after free or double free). Today the same program is refused at the spawn, so the change causes it. The repair is one property, not a case. Every thread fact a package worked out from one of its open classes and relied on becomes a fact of that package's interface (MOD-9, EFF-4). A class derived later, in another package or by a reload, must keep it, and is refused at its own line, with the message naming the line that relied on it and giving a copy line. The same property closes the generic-derived-class route, and to_verify item 6 (a library's own send of a base value) should move into rule text. Three objections do not block. S2-2: CLS-12 decides it, but the cost report scopes the once-flag too narrowly. E2-2: decision 3's short text hides that option (i) has the far-break property C was turned down for. E2-3: TYP-43's prose needs one clause. All three are real, and two must go to the owner.

### Edit: IV.8 (standard interfaces, the definition ERR-3 cites) (change, p04-types.md 676)

**Before:**

> interface Error: needs (Debug, Display):

**After:**

> interface Error: needs (Debug, Display, Send, ThreadShared):

Why here: This is the cause. `Error` is the interface that every error is erased through, and it was the only one of ERR-3, ERR-8 and THR-8 that said nothing about threads. std/src/core.em line 80 follows it.

Why general: IV.8 holds "the definitions" of the standard interfaces (p04 600). It is the declaration that every bound, every `dyn Error` and every implementer reads.

### Edit: ERR-3 (change, p13-errors.md 87-88)

**Before:**

> `Error` is the interface of error types:
>   `Display + Debug` with `fn source() -> Option[ref dyn Error]` (default `None`) (§IV.8).

**After:**

> `Error` is the interface of error types, `interface Error: needs (Debug, Display, Send, ThreadShared):` with `fn source() -> Option[ref dyn Error]` (default `None`) (§IV.8). Any thread may have and read an error, so `dyn Error` and `AnyError` are `Send` and `ThreadShared` (`[THR-8]`, `[THR-9]`). A type that holds a value that stays on one thread is not an error, and is refused where it implements `Error` or where it is derived from a class that does (`E2040`, `[IFC-3]`; `E2080` at `@derive(Error)`, `[DRV-1]`). It may still be a `Result`'s error that its `Display` prints (`[ERR-1]`, `[ERR-12]`).

Why here: ERR-3 describes the declaration of p04 676. The two must say the same thing, or the two texts would decide one program differently.

Why general: It states one property of what an error is: any thread may have and read it. The general rules then decide the rest. IFC-3 says where an implementation that does not meet the parents is refused and what its message gives. THR-8 and THR-9 say what each part of a type holds. ERR-3 lists no cases and no fixes. ERR-1's "any `E: needs Error`", ERR-8's `From` and ERR-9's "any error" stay true as written. The parents are spelled with `needs`, as verdict 110 spells them, and the old `Display + Debug` spelling goes.

### Edit: TYP-17 (its example of an interface's parents) (change, p04-types.md 458-459)

**Before:**

> an interface's parents
>   (`interface Error: needs (Debug, Display):`)

**After:**

> an interface's parents
>   (`interface Error: needs (Debug, Display, Send, ThreadShared):`)

Why here: Left as it was, a reader of the generics chapter would find the old parents, and objection E1-4's `report_later` would compile under one text and be E2040 under the other.

Why general: It is the third copy of the declaration. It still serves as the example of several needs in round brackets.

### Edit: THR-8 (change, p11-concurrency.md 47-48)

**Before:**

> Structs, enums, tuples, fixed arrays, collections and class objects are `Send` when every component is;

**After:**

> Structs, enums, tuples, fixed arrays, collections and class objects are `Send` when every component is, and an object of an `open` or `abstract` class may be an object of any class the program derives from it, a library's included and each instance of a generic one, whose fields are then among its components (as a call through a `virtual` method counts every override the program has, `[EFF-24]`);

Why here: This is where the class markers are stated. Without it, the change's requirement on `Error` would be checked against the declared class alone, and objection S1-1's `DocError` (and an error struct holding a field of a plain open class whose derived object holds a `shares` owner) would cross threads inside an AnyError.

Why general: It makes THR-8's list say what its own definition says ("nothing reachable from it"). This holds for every class hierarchy, error or not: a game's `Script` with an `Enemy` holding a `weakref`, std's ECS `World` holding `Map[u64, AnyStorage]` (std/src/ecs.em 101, 201) whose `[T] AosStorage` holds the program's components, and an error's `GameError` with a `DocError`. Nothing is written: the markers are still inferred from the words of types (TYP-43), here the declarations of the derived classes. A line that needs the marker is refused with E7015, whose message names the derived class and its field (THR-2's "whose message gives the words to write").

### Edit: THR-9 (change, p11-concurrency.md 63-64)

**Before:**

> Structural types are `ThreadShared` when every component is.

**After:**

> Structural types and class objects are `ThreadShared` when every component is (a class object's components as `[THR-8]` counts them), and `dyn I` when `I` requires it (`interface I: needs (Send, ThreadShared):`), as for `Send` (`[THR-8]`, ODR-255).

Why here: Without it, `ref r` of a `Result[int]` given to a scoped task (a `ref T` is Send when `T` is ThreadShared, p11 49) and an AnyError inside a thread_shared object's `RwLock` (verdict 165) stay undecided after the ERR-3 change.

Why general: Every interface gets the ThreadShared half of THR-8's dyn clause, a program's own `interface Command: needs (Send, ThreadShared):` as well as `Error`. ODR-255 already ruled it, and the text never said it. Class objects are counted as for Send.

### Edit: Marker table, Send and ThreadShared rows (TYP-43) (change, p04-types.md 577-578)

**Before:**

> Send row: "a class object is when its fields are;" / ThreadShared row: "every component is `ThreadShared`;"

**After:**

> Send row: "a class object is when its fields are, and those of every class the program derives from its class (`[THR-8]`); `dyn I` when `I` requires it;" / ThreadShared row: "every component is `ThreadShared`, a class object's counted as for `Send`; `dyn I` when `I` requires it;"

Why here: The row "a class object is when its fields are" is the narrow reading that lets a base-class value holding a one-thread derived object through. Leaving it would be a contradiction with the edited THR-8.

Why general: TYP-43 says the table answers the markers (p04 583). It must give the same answer as THR-8 and THR-9.

### Edit: IFC-3 (add, p05-declarations.md 411-413 (after "(ODR-042)."))

**Before:**

> `interface Ord: needs Eq` requires implementers of `Ord` to implement `Eq`. So a bound brings its parents: `T: needs IndexMut[int]` provides `Index[int]`'s methods too, and a binding it writes (`Output = f32`) names the parent's associated type (ODR-042).

**After:**

> (unchanged, then:) An implementation meets its interface's parents for every type it covers: each instance of a generic implementation, and each class derived from a class it covers, whose objects are values of that class (`[TYP-5]` rule 8), each instance of a generic one included. One that does not is `E2040`, either at the generic implementation (`extend[T] Tagged[T] implements Ord:` over `extend[T: needs Eq] Tagged[T] implements Eq:` gives `extend[T: needs Eq] Tagged[T] implements Ord:`) or where the derived class is declared, which its own package checks, whichever package declares the base. The message names the part that does not meet a parent and gives a line that compiles. That line either makes the part meet the parent by the rule that decides it (a type parameter's needs, an interface's needs), or holds in its place a copy of what the type uses from it.

Why here: Decision 2 (a generic error implementation) and objection S1-1 (a derived error class) are both about what an implementation covers. Stating it in ERR-3 would make it a case of `Error` (objection I1-1), and `Command`, `Ord` and every later interface with marker parents would stay undecided.

Why general: It is one property of every implementation of every interface with parents: `Ord` over `Eq`, `IndexMut` over `Index`, a program's own `interface Command: needs (Send, ThreadShared):`, and `Error`. For a parent the base implements (Eq, Debug), a derived class meets it through the base's implementation, so nothing changes there. For a marker parent, which is inferred from contents, the check falls on the derived class's own fields, at the derived class's own line. A derived class may be in another package, but the check is still local. Rust also requires a supertrait to hold for every type an impl covers. The message rule is a property, not a list: it gives a needs line where one exists and a copy line otherwise. That is principle 12, and verdict 73's "how to fix an issue not how to run into another", so `thread_shared` is offered only where it compiles.

### Edit: DRV-1 (add, p14-comptime.md 138 (after "naming the field."))

**Before:**

> A derive whose requirement a field does not meet is `E2080`, naming the field.

**After:**

> (unchanged, then:) On a generic type, a requirement that a field meets only for some arguments of the type's parameters is written as their needs in the generated `extend`, so the derive implements the interface for those instances (`@derive(Copy)` above `[T]` and `pub struct Range:` is `Copy` when `T` is, ODR-027; `[STR-5]`).

Why here: With Error's new parents, `@derive(Error)` on a generic error type (`SpawnError[T]` holding `rejected: T`) must be decided. Read the other way it would be refused even when only `SpawnError[int]` is used, unlike every other derive (objection E1-3).

Why general: Every derive works this way, which is how STR-5 ("when every field implements the interface"), ODR-027 and std's own `@derive(Copy)` above `[T] pub struct Range` (std/src/core.em 1850-1853) already read it. E2080 stays for a field whose own written type fails. The generated code stays what `ember inspect --expand` prints.

### Edit: Code table, E2080 row (change, p17a-codes.md 71)

**Before:**

> | `E2080` | `@derive(Copy)` on a type with a field that is not Copy | `[STR-3]` |

**After:**

> | `E2080` | a derive whose requirement a field does not meet, naming the field (`@derive(Copy)` with a field that is not `Copy`; `@derive(Error)` on a type holding a value that stays on one thread) | `[STR-3]`, `[DRV-1]`, `[ERR-3]` |

Why here: ERR-3 now names E2080 for `@derive(Error)`.

Why general: The row says what DRV-1 already says for every derive.

### Edit: Code table, E2040 row (change, p17a-codes.md 59)

**Before:**

> | `E2040` | unsatisfied interface bound; for a `pub` generic whose bounds are not written, or a generic that writes some of its bounds and not all the body needs, one message on the first line of its body that needs a missing bound, giving the whole line of what it needs | `[IFC-6]`, `[TYP-17]`, `[BLD-14]` |

**After:**

> | `E2040` | unsatisfied interface bound; for a `pub` generic whose bounds are not written, or a generic that writes some of its bounds and not all the body needs, one message on the first line of its body that needs a missing bound, giving the whole line of what it needs; an implementation that does not meet its interface's parents for every type it covers, at the generic implementation or at the derived class | `[IFC-6]`, `[TYP-17]`, `[BLD-14]`, `[IFC-3]`, `[ERR-3]` |

Why here: Objection I1-4.

Why general: It lists the refusal that IFC-3 now states.

### Impact

Rules changed: IV.8 (p04 676), ERR-3 (p13 87-88) and TYP-17's example (p04 459): Error requires Debug, Display, Send and ThreadShared, and a type holding a one-thread part is not an error. THR-8 (p11 47-48), THR-9 (p11 63-64) and the marker rows (p04 577-578): an object of an open or abstract class counts the fields of every class the program derives from it, and THR-9 gains the dyn clause. IFC-3 (p05 411-413): an implementation meets its interface's parents for every type it covers (generic instances, derived classes), with the refusal placed and the message's line stated as a property. DRV-1 (p14 138): a derive on a generic type holds per instance and writes those needs. p17a 59 and 71: the code rows. ERR-1, ERR-8, ERR-9, ERR-12, THR-1, THR-2, THR-10 and CLS-12 are unchanged and stay true.

Readability: Better. No word is added to ordinary code. The worker loses an import, a concrete error type and, with two kinds of failure, a whole error enum. Generic error code (`type Failure: needs Error`), wrappers holding `owns dyn Error`, and class hierarchies whose derived classes hold plain data stay exactly as they are today. Words are added in two places only. A hand-written generic error implementation whose parameter holds the caller's value writes `T: needs (Send, ThreadShared)` (a derive writes it itself). An interface whose values an error holds as `owns dyn I` gets `Send` and `ThreadShared` in its needs, which then holds every implementer of that interface to them, or else the error holds a copy of the data. Under option A, the bound would be written instead in every pub generic function that uses `?` on such an error. Refusals fall on the type that holds the one-thread part: an error type at its declaration, a derived error class at its own line, and a plain open-class value at the line that sends it.

Cost: Compile time: Send and ThreadShared are inferred as today. For each open or abstract class, the program build collects the classes derived from it (a library's included, and each generic instance made). That is one walk of the class declarations, which EFF-24 already makes for virtual calls. Each implementation of an interface with parents is checked once per instance and once per derived class. A `?` into AnyError is checked as today. Run time: the class rule adds nothing. AnyError keeps its layout (the boxed error's data and table pointers, a 16-byte identity, the context list) and its one allocation on the failure path (ERR-11). The success path costs nothing, as in C, and handing an AnyError to another thread moves it, as C passes an error pointer. Two costs are stated. (1) CLS-12 (p05 380): a `@cached` method of a class that implements Error is filled under the once-flag, because its objects may now be read by several threads. Per call, the flag test becomes an acquire load (on x86-64 the same `mov` as a plain load; on ARM64 an `ldar`), and the first call fills under the flag with one compare-and-swap. This is already the duty today for any ThreadShared class given by `ref` to scoped tasks. (2) THR-2's hand-over check (verdict 114, "every object the handed object owns") reaches objects the boxed error owns whose class has weakref links, through the error's table: one indirect call per AnyError handed over. Nothing runs in a program where no error type converted into AnyError owns such an object. To measure: AnyError's size and its failure-path time against today's build; the `@cached` once-flag against a plain flag on ARM64 and x86-64; and the hand-over check's time.

* verdict 61 (`thread_shared fn()`: the type says which threads may run it, not a whole-program search): kept (The Error interface says which threads may have an error, and option C (a search through the `?` lines of the whole program) is turned down for verdict 61's reason. The class rule reads the declarations of derived classes, which are the words of types that TYP-43 infers markers from. It does not read bodies to find which functions a field is given, which is what 61 turned down. Listed for the owner as the nearest tension (decision 3).)
* verdict 86 and 122 (what a function changes is worked out from every function the program can put in a place, a far edit naming the chain): kept (An open class's markers are worked out as its virtual calls already are (EFF-24, p10 94: "from every function the program can put there"). A refusal names the derived class and its field.)
* verdict 94 (thread.spawn takes a thread_shared fn(); R must be Send): kept (spawn's line is unchanged. AnyError now meets `R: Send`.)
* verdict 110 (an interface's parents written with `needs`): kept (`interface Error: needs (Debug, Display, Send, ThreadShared):`, in all three copies.)
* verdict 108, 111 (pub generics and extends write their needs with `needs`): kept (A hand-written generic implementation writes `T: needs (Send, ThreadShared)`: a library's promise is in its line.)
* verdict 113 (a private generic's needs worked out): kept (Unchanged. A derive writes its generated needs itself (DRV-1), as the compiler works out what it can.)
* verdict 132 (a declaration means what it says): kept (`extend[T] X[T] implements Error` is never quietly read as "only for some T". It is refused, with the bounded line given. A derive's generated `extend` says its bounds.)
* verdict 164 (a sharedref mut link never goes to another thread): kept (An error holding one is not Send, so it is refused where it implements Error.)
* verdict 165 (thread_shared refused where the value is made when not ThreadShared): kept (The same 'refused where it is made' shape. A thread_shared holder of results is now accepted, because AnyError is ThreadShared.)
* verdict 23 (a shares value is never used from two threads): kept (No shares owner can be inside an error, inside a derived error class, or inside a base-class value that crosses threads.)
* verdict 114 (a handed-over object's weakref links say gone): kept (THR-2's words already reach objects owned through the error's box. The cost is stated above.)
* verdict ODR-255 (dyn I is Send/Sync when I requires it): kept (Its ThreadShared half is now written in THR-9.)
* verdict ODR-050 (Result's error parameter is unbounded): kept (A one-thread error type is still a valid `Result` error, printed by main through `Display` (ERR-12). It is just not an `Error`.)
* verdict ODR-027 (a range is `Copy` when its bound is): kept (DRV-1 now states that reading for every derive on a generic type.)
* a program changes: E7015 at the spawn line, which no text explains -> compiles; main prints the file's length, or `error: <the system's message>` with status 1
* a program changes: E7015 at the spawn line -> prints `8080` then `error: the text is not a value of this type`
* a program changes: E7015 -> prints 2
* a program changes: E7015 at that line (verdict 165) -> compiles; a thread counts the loaded sizes and prints 1
* a program changes: E7015 at the spawn line -> prints 3; no thread word written
* a program changes: compiles -> E2080 at `@derive(Error)` (E2040 at a written `extend ... implements Error:`), naming the field, with a line that compiles: a copy of what the error uses (`path: String` with `copy of doc.path`). `thread_shared` is offered only where no line then changes the value directly (objection E1-2's `shares mut Doc` gets the copy line). Such a type can still be a `Result`'s error shown with Display.
* a program changes: compiles -> E2080 at the derive, naming the field. The message gives `interface Shape: needs (Debug, Send, ThreadShared):` (which then holds every implementer of `Shape` to the markers, so a `Sprite` holding `shares Texture` becomes E2040 at its `implements Shape`), or a copy of the data (`TooLarge(area: int)`). Under option A, a one-thread renderer that matches `Result[int, DrawError]` would compile.
* a program changes: compiles -> E2040 at its `implements Error`, naming `retry`. The message gives a copy line (the data the retry would read). It does not offer `thread_shared fn()`, whose ThreadShared the text does not yet decide (an existing gap, in notes).
* a program changes: compiles (the Error case); undecided (the others) -> E2040 at that line, giving the bounded line (`extend[T: needs (Send, ThreadShared)] PoolFull[T] implements Error:`)
* a program changes: compiles -> compiles; the derive implements Error for the instances whose `T` is Debug, Send and ThreadShared; a `?` of a `SpawnError[shares Enemy]` into AnyError is E2040 at the `?`
* a program changes: compiles (it cannot reach another thread inside an AnyError today, because AnyError is not Send) -> E2040 at `class DocError(GameError):`, naming `doc`, checked in DocError's package even when GameError is a library's
* a program changes: compiles, and the derived object's plain count races (an existing hole) -> E7015 at that line, naming the derived class and its field, and giving a copy line (`entity = s.entity` and `fn() copy of entity: entity`)
* a program changes: runs -> runs, unchanged (prints 91 in the test)
* a program changes: always allowed, and races when a component holds a one-thread part -> allowed when every component type the program stores is Send (and ThreadShared for reads), and E7015 at the line otherwise, naming the component and its field
* a program changes: refused (AnyError not ThreadShared) -> prints 68; CLS-12's once-flag applies, since every object of a class that implements Error is an object several threads may reach
* a program changes: may be an Error -> refused where it implements Error, until ThreadShared can be vouched for (open question P04/J7-06)

**Decision (made by the owner's parameters): Where the thread markers of an erased error come from: Should every error type be safe to hand to and read from any thread (as Swift's errors are), so the catch-all error (like C++'s `std::exception_ptr`; Ember's `AnyError`) can come back from a worker thread?**

step | who | what happens | result
1 | error type's author | declares an error of plain values (a code, a text) | allowed: it may go to any thread
2 | worker thread | `fs.read_to_string(path)?` puts the file error into the catch-all | allowed, nothing written
3 | worker thread | returns `Result[int]` (a count or the catch-all error) | allowed: every error may cross threads
4 | main thread | `n = h.join()?` | gets the count, or passes the error on
5 | error type's author | declares an error holding a one-thread shared owner, or an object of an interface that does not say threads may have it | refused at that declaration, with the line to write

Today: The worker returning `Result[int]` is refused at `thread.spawn`, the text never says so, and the worker must name its own error type.

* A: the catch-all checks each error where `?` puts it in; generic error code and error wrappers must add thread words
* B: keep today: the catch-all stays on its thread; every worker names its own error type
* C: the compiler searches the whole program; one one-thread error anywhere breaks every thread hand-off, far from that line
* D: every error type is safe for every thread, checked where the type is declared an error; nothing written anywhere else
* E: the catch-all may be sent but not read by two threads at once; a shared table of results is refused
* F: check while running and stop the program when a one-thread error crosses

Recommendation: D: ordinary worker, server and library code compile with nothing written; only an error holding a one-thread part is refused, at its own line, with the fix.

Chosen: D. By: 1. The three goals at once. B fails 'easy to write': every worker grows its own error type, and an enum when it can fail two ways. A and E fail it for generic error code and wrappers, which grow thread words. C, D and F keep ordinary code as it is today. 2. Safety: A to E all refuse before a one-thread value crosses, and F stops the program. D is sound only with the class components counted as THR-8's definition reads them (decision 3) and with derived classes checked (IFC-3), both in this change. 3. Compile time first: A to E are decided when compiling. F's run-time check cannot make the refused program work: a shares owner inside the error cannot be moved away from its other owners, so F can only stop it. A compile-time refusal with a line to write would become a crash on the least-tested path (checklist 8), so F is out. 4. A written word only for meaning: A and E write markers in bounds and after `dyn`, and B writes a concrete error type. D writes a bound only on a hand-written generic error implementation (the library's promise, verdict 108), and the markers on an interface only when an error holds its values, where they state which threads may have those values. 5. The ordinary program no longer or harder: C lets an edit anywhere break a spawn far away. D refuses only an error type holding a one-thread part, at its own declaration, even in a one-thread program, and the fix is to hold a copy. A refuses the same errors at their `?`, which nearly every error reaches because main returns `Result[void]`. A accepts an error that is only matched, as in objection E1-1's renderer, which D refuses. 6. Consistency: D uses THR-8's own dyn clause, IFC-3, ODR-255, verdict 110's spelling, verdict 61 (the type says which threads may have it) and verdict 165 (refused where it is made). It matches Swift (`Error: Sendable`) and Rust's anyhow (`Send + Sync`). So D.

**Decision (made by the owner's parameters): A generic implementation whose interface has parents (a generic error that holds the caller's value): A generic error that hands the caller's value back (like Rust's `SendError<T>`): is it an error for every `T`, or only for the `T`s any thread may have, and who writes that?**

step | who | what happens | result
1 | library author | writes `extend[T] PoolFull[T] implements Error:` | refused, with the line to write
2 | library author | writes `extend[T: needs (Send, ThreadShared)] PoolFull[T] implements Error:` | allowed
3 | library author | writes `@derive(Error)` on a generic enum | allowed; the derive writes the bound itself
4 | program | `insert(...)?` with `PoolFull[int]` into the catch-all | allowed
5 | one-thread game | `insert(...)?` with `PoolFull[shares Enemy]` into the catch-all | refused at that line: name the error type instead

Today: Nothing says; the implementation covers every `T`, because errors need no thread markers today.

* (i) a hand-written implementation writes the bound, or is refused with the line to write; a derive writes it itself
* (ii) the implementation quietly holds only for the safe `T`s
* (iii) a generic error type must bound its parameter on the type itself

Recommendation: (i): the line says what it means, the compiler gives the line to write, and derives keep working per instance as they already do.

Chosen: i. By: 1. The three goals: all are equal for ordinary code, because generic error types holding a caller's value are rare. (iii) is harder: a type used only as `SpawnError[int]` is refused. 2. Safety: all three are safe. 3. Compile time: all three. 4. A written word only for meaning: the bound on a hand-written implementation is the library's promise of which values it is an error for (verdicts 108, 111). A derive is the compiler writing the implementation, so it writes the bound itself and `ember inspect --expand` shows it (principle 1). 5. The ordinary program: unchanged under (i). 6. Consistency: IFC-3 says implementers implement the parents, Rust requires a supertrait to hold for every type an impl covers, verdict 132 says a declaration's meaning comes only from what it says, and STR-5, ODR-027 and std's `@derive(Copy)` on `[T] pub struct Range` make derives per instance. So (i), stated once in IFC-3 for every interface and in DRV-1 for every derive.

**Decision (made by the owner's parameters): What an object of an open class may hold, for the thread markers: When a base-class object (C++'s `Base*` that may point at a `Derived`) is handed to another thread, should the compiler look at every derived class the program declares?**

step | who | what happens | result
1 | game author | `open class Script`, and `class Door(Script)` with plain fields | a Script may go to another thread
2 | game author | adds `class Enemy(Script)` holding a one-thread shared owner | allowed
3 | worker | `thread.spawn(owns fn() owns s: ...)` with `s: Script` | refused at that line, naming Enemy's field
4 | error author | `class DocError(GameError)` holding a one-thread shared owner, GameError being an error | refused at DocError's line

Today: The compiler looks only at the base class's own fields, so a Script that is an Enemy goes to another thread and its shared count races.

* (i) look at every derived class the program declares; nothing written
* (ii) a base class's objects never go to another thread unless the base promises it through an interface (Swift)
* (iii) every derived class must be as thread-safe as its base's own fields
* (iv) keep today

Recommendation: (i): correct hierarchies keep working with nothing written, and only the class that holds the one-thread part is named.

Chosen: i. By: 1. The three goals: (iii) refuses ordinary class hierarchies. (ii) refuses every open-class value crossing threads or read by tasks unless an interface is written only to carry the markers, and it breaks std's own ECS World. (i) and (iv) keep ordinary code. 2. Safety: (iv) lets a one-thread owner cross, so it is out. 3. Compile time: (i), (ii) and (iii) are all decided when compiling. 4. A written word only for meaning: (i) writes nothing, and (ii) writes an interface that says nothing a declaration does not already say. 5. The ordinary program: (i) accepts every program whose derived classes are thread-safe. Its refusal is at the line that sends the value and names the derived class, the cost verdict 86 accepted for effects. 6. Consistency: (i) is what THR-8's own definition says ("nothing reachable from it"). It is how the language already treats an open class's virtual calls (EFF-24, p10 94: "from every function the program can put there"; verdict 122) and its views (p07 713). And it infers markers from the words of types, as TYP-43 says. (ii) matches THR-8's dyn clause and Swift, but a class has no needs list, and (ii) loses on parameters 1 and 5 first. Where a base does promise the markers, by implementing an interface that needs them (Error), each derived class is checked at its own line (IFC-3, decision 2's property). So (i).

### What blocks it (2)

* S2-1 (S, unsafe_allowed): A use after free and a double free of a one-thread shared value, through the default error type. Nothing is written beyond `thread.spawn`, in the ordinary engine/game split: an engine library with an open `Script` (or `Component`, `Node`, `Widget`) and an error type that keeps the failing object, and a game whose derived class holds a `shares` owner, a `weakref` target or a `Cell`. The ThreadShared half breaks the same way: a derived class's `Cell` read by two scoped tasks through `ref r`, or under a thread_shared RwLock of results. Hot reload reaches the same race by adding a field to a derived class. Every soundness argument for AnyError rests on 'every implementation of Error meets Send and ThreadShared in the final program', and across a package's edge or a reload nothing in the changed text makes that true.
  Arbiter: Walked through the changed text. (1) gamekit's build. THR-8 after (p11 47-48) counts "any class the program derives from it, a library's included". gamekit derives nothing from Script, so LoadError (`script: Script`, `entity: int`) is Send and ThreadShared, and `extend LoadError implements Error:` meets IFC-3 after ("An implementation meets its interface's parents for every type it covers"). (2) game's build. IFC-3 after places the refusal only "either at the generic implementation ... or where the derived class is declared", and the derived-class check reaches only "each class derived from a class it covers". The Error implementation covers LoadError, a struct. It does not cover Script, and Script implements nothing, so `class Enemy(Script):` is checked against nothing. ERR-3 after says the same: "or where it is derived from a class that does". (3) MOD-9 (p05 38-46) says what crosses a package's edge: the written words, plus the facts it lists ("what each function may change, read and end ... the locks it may take; the function values it stores and the callbacks it hands to C"). No edited word records that gamekit relied on Script's markers. No edited word makes game's build check gamekit's implementation again either. (4) The `?` into AnyError needs `E: needs Error` (ERR-8, p13 95), and gamekit's written implementation meets it. AnyError's `Box[dyn Error]` is Send by p11 51 ("`dyn I` when `I` requires it"), which reads the interface and never what the box holds. So the spawn compiles. The worker ends the Enemy's `shares` owner while main changes the same count with plain adds (RC-4 / THR-2 p11 100-101): a race on a non-atomic count, then a use after free or a double free. Today the same program is E7015 at the spawn: p04 676 and p13 88 give Error only Debug and Display, so by p11 51 `dyn Error` is not Send. So the change causes it. The other reading (game's build checks gamekit's line 20 again) is refused at a dependency's line that the text places nowhere. One of the two readings lets a race through, so this is S. The reload route (HR-44 compares only functions' facts) and the generic-derived-class route come from the same missing rule. I did not walk them separately, and the first route settles the ruling.

```
# file gamekit/ember.toml
#   [package]
#   name = "gamekit"
#   version = "0.1.0"
#   language = "0.9.15"
#   kind = "lib"
# file gamekit/src/lib.em (line numbers below count from its first line)
from std.fmt import Formatter, FmtError

pub open class Script:
    let entity: int

    pub fn init(entity: int):
        this.entity = entity

pub struct LoadError:
    pub script: Script

extend LoadError implements Display:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("the script could not load")

extend LoadError implements Debug:
    fn fmt_debug(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("LoadError")

extend LoadError implements Error:
    pass
# file game/ember.toml
#   [package]
#   name = "game"
#   version = "0.1.0"
#   language = "0.9.15"
#   [dependencies]
#   gamekit = { path = "../gamekit" }
# file game/src/main.em (line numbers below count from its first line)
import std.thread
from gamekit import Script, LoadError

class Level:
    name: String = String("forest")

class Enemy(Script):
    level: shares Level

    fn init(entity: int, owns level: shares Level):
        this.level = level
        super.init(entity)

fn make_enemy(entity: int, owns level: shares Level) -> Result[int, LoadError]:
    return Err(LoadError(script=Enemy(entity, owns level)))   # an Enemy, upcast to Script

fn load(entity: int, owns level: shares Level) -> Result[int]:
    return Ok(make_enemy(entity, owns level)?)                # LoadError -> AnyError

fn main():
    level = shares Level()
    r = load(1, shares level)                                  # r's error holds a second owner of the Level
    h = thread.spawn(owns fn() owns r: r.is_err())             # r goes to another thread, which ends it
    for i in 0..100000:
        other = shares level                                   # +1, then -1 at the end of the round: plain adds
        println(i, other.name.len())
    println(h.join())
```

* E2-1 (S, unsafe_allowed): Decision D is sound only if every error's markers hold in the build that uses it. For an error type in one package that holds an object of an open class (a field, an element, an `owns` part), the markers were judged without the classes another package derives later, and every AnyError, channel of results, scoped `ref` and thread_shared holder of results then carries a one-thread owner across threads. Library error types holding a base-class object are ordinary: a game engine's ScriptError holding the failing Script, a UI toolkit's error holding the Widget. The impact report says the class rule closes the open-class hole, but across a package edge it does not, and it gives the library-line placement only in to_verify.
  Arbiter: Same cause as S2-1, a second program. In engine's build, THR-8 after (p11 47-48) sees no derived class, so ScriptError meets IFC-3 after at its `implements Error`. EFF-4 (p10 52-56) says "a fact for which a word exists counts only as written", and the implementation is written. In game's build, IFC-3 after's two refusal places ("at the generic implementation ... or where the derived class is declared" for a class "derived from a class it covers") miss `class Enemy(Script):` and `fail(owns s)?`. `Sender[Result[int]]` is Send because AnyError's `owns dyn Error` is Send by requirement (p11 51, with p04 676 after). So the spawn's `owns tx` compiles, and main ending `e` takes a plain -1 from `level`'s count while the worker's loop changes it: a race, then a double free or a leak. Today it is E7015 at the spawn, because Error does not require Send. So the change causes it. The objection also points out something true: the concrete route `channel[Result[int, ScriptError]]` is refused after the change, because game's build infers ScriptError's Send with Enemy counted. Only the erased route trusts a requirement that was checked where Enemy could not be seen. One repair closes both S2-1 and E2-1. to_verify item 6 ("reported in the program's build, at the line that passes the value") is in no edited rule.

```
# package engine (a library): engine/src/lib.em
from std.fmt import Formatter, FmtError

pub open class Script:
    pub entity: int

    pub fn init(entity: int):
        this.entity = entity

pub struct ScriptError:
    pub script: Script

extend ScriptError implements Display:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("the script failed")

extend ScriptError implements Debug:
    fn fmt_debug(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("ScriptError")

extend ScriptError implements Error:
    pass

pub fn fail(owns s: Script) -> Result[int, ScriptError]:
    return Err(ScriptError(script=s))

# package game (uses engine): game/src/main.em
import std.thread
from std.sync import Sender, channel
from engine import Script, fail

class Level:
    name: String = String("forest")

class Enemy(Script):
    level: shares Level

    fn init(entity: int, owns level: shares Level):
        this.level = level
        super.init(entity)

fn tick(owns s: Script) -> Result[int]:
    return Ok(fail(owns s)?)                         # ScriptError -> AnyError

fn work(tx: Sender[Result[int]]):
    level = shares Level()
    tx.send(owns tick(owns Enemy(1, shares level))).expect("receiver is gone")
    n = 0
    for _ in 0..1000000:
        extra = shares level                         # plain add and sub on this thread
        n += extra.name.len()
    println(n)

fn main():
    tx, rx = channel[Result[int]](capacity=1)
    h = thread.spawn(owns fn() owns tx: work(tx))
    for r in rx:
        match r:
            Ok(n) => println(n)
            Err(e) => println(e)                     # the Enemy ends here: a plain sub on the main thread
    h.join()
```

Arbiter's rulings on the open objections: 5 real, 0 refuted, 0 already in today's specification

Objections closed in the debate: 11 (11 fixed, 0 withdrawn after a rebuttal)

Gaps found on the way that exist without the change: 4

## K10-queued-drop-check-points (P05; J7-01, J7-02): **rejected**, 11 agents

Outcome: `decided_by_parameters`. Intent: A thing one thread made and another let go of (a pooled connection, a GPU texture, a packet buffer) is ended soon by its drop, at a point where running that drop breaks no rule, and is never left undone because its making thread sleeps in the job pool.

Where the cause is: RC-4 point 3 (tasks/spec-0.9.9/parts/p08-classes.md 169-172), with RC-4 point 4 (p08 173-178), THR-20's headline 'A `drop` runs where the last owner ends' (p11-concurrency.md 296-301) and DRP-8 (p07-ownership.md 865-873). (reported at: BRW-18, EFF-24, FFI-22, HR-3a, JOB-2, RC-3, RC-4, THR-16, THR-19, THR-20, WK-3). Why there: A check point is where a queued value is ended, so it is where its drop runs (RC-4 point 2, p08 166-168; WK-3, p08 365-367). DRP-8 makes every end a call of the drop 'for every rule that needs them'. Point 3 places those ends by position only. So they land inside a lock the drop takes, inside a walk or view of what it changes, inside a scope whose tasks touch it, or inside a no-waiting function (J7-01). It also ties the last end to the thread's end, which a pool worker reaches only with the process (THR-16) and a host thread never shows (FFI-31a) (J7-02). The debate showed the same cause three more ways. (1) A value whose maker can never run its drop safely must not be queued at all, and that is judged by where its making line runs, not by where it is written (point 4). (2) A by-type view rule is untrue for what the queue ends only for a view that holds a count or reads through an owner. A `weakref` view of an object that is not `thread_shared` does neither. (3) A thread's last point must come before its join is let go. Then THR-20's count (the making thread may run the drop) stays true and THR-19 judges the join. The reporting rules are right as written: each one sees only what point 3 and THR-20 hand it.

**In short: A deleter owed to a scoped worker thread (a shared_ptr whose destructor must run on the thread that made it) needs a mutex the parent holds while it waits for the scope to finish.**

step | who | what happens | result
1 | scoped worker thread | opens a connection and puts it in an inbox | about to finish
2 | closer thread | lets go of the connection | its destructor is owed to the scoped thread
3 | main | holds the pool's mutex and waits for the scope to finish | waiting
4 | scoped thread | runs the owed destructor before finishing; it needs the pool's mutex | each waits for the other
5 | the changed rules | one says refuse main's wait, another says compile it | two answers for one program

Today: Not said whether a scoped thread's last owed destructors run before or after the scope's wait ends.

* a: count a thread's last owed destructors as part of what that thread does, scoped threads too; the wait under the mutex is refused with a one-line fix (wait after the mutex block)
* b: run a scoped thread's last owed destructors after the scope's wait ends; it compiles, but the scope's end no longer means they are done
* c: leave the words as they are: one rule refuses it, another compiles it and it freezes

Recommendation: a: one answer, decided when compiling, nothing added while running, the same as a joined thread already gets.

Arbiter: One open objection blocks: E3-1, a contradiction. The change says a thread's last pending destructors run before its join or its scope's end lets go, and that they count as part of what the thread does. But the only rule it cites for that count covers spawned threads and unscoped jobs, not scoped tasks, and it must stay that way, or the change's own loader tests become errors. So a scope's end inside a lock, whose task may end such a value at its last point, is refused by one rule and compiled by another, and when compiled it hangs. The other safety claim, S3-1 (a use after free through the point after a chunk main runs itself), is refused. Read one way, the placement rule keeps that point out. Read the other way, the base destroy-counts-as-a-call rule refuses the program. Either way nothing reads freed memory or hangs. The wording should still say which. E3-2 (prebuilt libraries) is refused: the rule is a whole-program property, and the per-thread number already listed covers it. E3-3 and E3-4 are real but do not block. The cost figures shown to the owner misread the measured table, and decision 4's option b needs its scope and cost stated. The change goes back to the designer for one more round on E3-1. J7-01 and J7-02 themselves stay decided.

### Edit: RC-4 point 3 (change, tasks/spec-0.9.9/parts/p08-classes.md 169-172)

**Before:**

>   3. **Check points.** The compiler places them: at every loop's back edge, before every call that may
>      block the thread (waiting for a lock, `join`, a channel's receive, a sleep) and when the thread
>      ends. A value whose making thread has ended is ended at once by the thread that ends its last
>      owner.

**After:**

>   3. **Check points.** The compiler places them: at every loop's back edge, before every call that may
>      block the thread (waiting for a lock, `join`, a channel's receive, a sleep), when a call C made
>      into Ember returns to C, after each job a thread runs (a `@parallel` loop's chunks included) once
>      its end is recorded for what waits for it, and when the thread ends, before what waits for it (its
>      `join`, its scope's end) is let go; a job worker waiting for work waits at one, and a value put in
>      its queue wakes it. Emptying the queue ends, on this thread, the values whose counts have all
>      ended (point 2), and their `drop`s run there: it is a call of the `drop`s of every value that may
>      be in this thread's queue (`[DRP-8]`), with the queue's own count changes. A check point in a
>      function's lines is placed only where that call would compile if it were written there, judged
>      for each place the function runs from (its callers; a task's body against its parent and the
>      other tasks until its join, `[THR-11]`), and where this thread leaves the last place that keeps
>      one out (the end of a lock's `with` block or guard, of a walk, of a view, of a `with scope` block,
>      the return from a function with a contract) is a check point. A thread's last check point is
>      always placed: for a thread other than the main one, what it may end there is counted in its
>      function (`[THR-20]`), so every wait for the thread is judged with it (`[THR-19]`, `[THR-11]`).
>      What the queue ends, the values and what only they own, has no owner left, so a view that reads
>      its object through an owner or a count of this thread (`[RC-5]` point 2) cannot be reading it,
>      and a rule that judges by type what a call may end (a view's, `[RC-5]` point 1) does not keep a
>      check point out for such a view. A view opened from a `weakref` link to an object that is not
>      `thread_shared` holds neither, so it, and every view made through it, keeps check points out for
>      its life as that rule says. A value whose making thread has ended is ended at once by the thread
>      that ends its last owner.

Why here: Point 3 is the only place where hidden ends are placed. The same words decide J7-01 (no point inside `with pool.lock:`, and the block's end is one) and J7-02 (an idle worker is woken). They give the bound the conflict note asks for, and they decide the debate's scope-task, walk, view, weakref-view and join programs.

Why general: One property covers every hidden end: the queue is emptied only where a call of the queued drops would compile if it were written there. So every rule that judges a call judges this one (THR-19 in full, BRW-18, RC-5, THR-11 against a scope's tasks, every contract, THR-20), with no list of rules. Where the thread leaves the last place that keeps a point out, there is a point. That one phrase is the bound for every kind of place. The view exception is now true for what it covers. What the queue ends can be reached by this thread only through a `weakref` link. A view opened from one holds a count when its target is `thread_shared` (RC-5 point 2, p08 215-219). Every other view reads through an owner. So the exception holds for those views, and a `weakref` view of a plain class object, which holds no count, stays under RC-5 point 1 as written. The new points cover every kind of thread: spawned, main, job worker, and a thread C calls in on. A thread's last point comes before its join is let go, so `join` means done, and THR-20's count lets THR-19 judge the join. A job's point comes after its end is recorded, because a worker's queue holds values from any earlier job. No type or pattern is named.

### Edit: RC-4 point 4 (add, tasks/spec-0.9.9/parts/p08-classes.md 176-178 (sentences added after 178))

**Before:**

>      first adds its own count of the value into the away count and marks the value handed over; from
>      then on every thread, the making thread included, counts in the away count, and a later end on
>      another thread never goes through the queue.

**After:**

>      first adds its own count of the value into the away count and marks the value handed over; from
>      then on every thread, the making thread included, counts in the away count, and a later end on
>      another thread never goes through the queue. A thread also counts a value this way from the line
>      that makes it when its own check points may not run that value's `drop` until a time that may
>      never come, judged for each place that line runs from, through any chain of calls (`[EFF-24]`):
>      the line runs inside a function whose contract forbids an effect of the `drop`, which may never
>      return; or it runs on the thread of a task or a job of a scope, or of a chunk of a `@parallel`
>      loop, which may go on beside the main thread after the scope, and the `drop` reaches a module
>      variable of the main thread, which no other thread may reach (`[THR-20]`). Its `drop` then runs
>      where its last owner ends.

Why here: Point 4 is the rule that moves the making thread's count into the away count, so a value that must not wait for its maker belongs there.

Why general: One property: a value whose maker's own check points may never run its drop is counted for every thread from the start. Its drop then runs where its last owner ends, which is where THR-20 and DRP-8 already judge it. It is judged by where the making line runs, through any chain of calls and for each place, as point 3 judges its points. So a constructor helper called from a job (`load_texture(path)`) is covered just as a making line written in the job's body is. The parenthesis names the two kinds of place, as point 4's own parenthesis does. They are the only places that can hold a drop back for good when another thread could run it. A lock, a walk, a view or a scope would hold up any thread just the same.

### Edit: THR-20 (add, tasks/spec-0.9.9/parts/p11-concurrency.md 296-297)

**Before:**

> **A `drop` runs where
>   the last owner ends.** The last owner of a `thread_shared` value may end on any thread that holds a

**After:**

> **A `drop` runs where
>   the last owner ends**, or, for a value queued back to the thread that made it (`[RC-4]` point 2), at
>   a check point of that thread, placed where the `drop` may run (`[RC-4]` points 3 and 4): the function
>   of the thread that makes such a value holds a count of it from the making line to the thread's last
>   check point. The last owner of a `thread_shared` value may end on any thread that holds a

Why here: This rule judges which thread a drop runs on, so it must name both places where a drop runs. Here E2-1's contradiction is closed: the joined thread's function counts the drop, and nothing elsewhere takes that away.

Why general: Without the clause, the headline is untrue for every queued value. The clause also says plainly what the next sentence ('a function that may run on another thread and holds such a count counts as possibly running that value's `drop`', p11 298-301) needs: the maker holds its home count until its last point. So E7012, THR-13's module-variable bullet, THR-11 and THR-19's join and scope-end clause all see the drops a thread may run before it is joined.

### Edit: DRP-8 (add, tasks/spec-0.9.9/parts/p07-ownership.md 873 (a sentence added after it))

**Before:**

>   A `drop` may change things outside its object (a log line, leaving a list).

**After:**

>   A `drop` may change things outside its object (a log line, leaving a list). The ends at a check
>   point placed in a function's lines (`[RC-4]` point 3) are judged by that rule where it places them,
>   which is only where they break no rule; they are not part of that function's effects (`[EFF-1]`) or
>   summary (`[EFF-24]`) beyond what `[THR-20]` counts for a function that holds a count of the value,
>   which also covers a thread's last check point.

Why here: EFF-1 (p10 26-27), EFF-24 (p10 96), BRW-18 (p07 508) and RC-5 point 1 (p08 202) all route drops through DRP-8.

Why general: Without it, EFF-1's 'the drops it performs' would put every queued drop into the effects and summary of every function that loops. Every `@noblock` loop would be E4003, every walk across a call to a looping function E3020, and every task with a loop E7007. That would be untrue, because placement already keeps those ends where they break no rule (principle 6). The sentence takes away nothing that THR-20 counts, so the two cannot decide one program in different ways. BRW-18, RC-5 point 1, EFF-1 and EFF-24 all reach a drop through DRP-8, so this one sentence reaches all of them.

### Edit: COST-3 (the row 'A `thread_shared` value's two counts') (add, tasks/spec-0.9.9/parts/p10-effects.md 304)

**Before:**

> a test of the thread's queue at each check point (`[RC-4]`), and the queue's work for a value whose last owner ends on another thread without a hand-over.

**After:**

> a test of the thread's queue at each check point (`[RC-4]`), and the queue's work for a value whose last owner ends on another thread without a hand-over, with one wake of its making thread when that is a job worker waiting for work; atomic counts from the start for a value made where its making thread's check points may not run its `drop` (`[RC-4]` point 4); and, for a function whose check points or making lines are judged differently for the places it runs from, one more copy of its code or, where the compiler keeps one instead, a plain add and subtract of a per-thread number at each such place, which its check points and making lines test (`[RC-4]` points 3 and 4).

Why here: This row lists the run-time costs of the two-count design.

Why general: COST-3 lists everything that runs, and COST-5 (p10 353) asks for a row for each new implicit cost. The new costs are the wake, the hand-over at the making line, and the copy or the per-thread number, which now also serves making lines. The copy is named beside the number, as p10 343 names BRW-11's copy.

### Impact

Rules changed: RC-4 point 3 (p08 169-172), changed. Points are placed only where a call of the queued drops would compile. Where a thread leaves the last place that keeps a point out, there is a point. There are new points after each job, at a callback's return to C, and for a worker waiting for work. A thread's last point comes before its join is let go. The view exception now covers only views that read through an owner or a count. RC-4 point 4 (p08 178), added: a value is counted for every thread from its making line when its maker's points may never run its drop. This is judged by where that line runs. THR-20 (p11 296-297), added: a queued value's drop runs at its maker's check point, and the maker's function holds a count until its last point. DRP-8 (p07 873), added: ends at points inside a function are judged where placed and are not in the function's effects or summary, beyond what THR-20 counts. COST-3 (p10 304), added. Not changed: RC-4 points 1, 2 and 5; WK-3; RC-5; THR-11; THR-13; THR-19; BRW-18; EFF-1; EFF-24; RC-3; JOB-2; HR-3a; FFI-22; THR-16.

Readability: Better, with nothing added. The one visible difference: a queued drop's output may appear a little later, where the thread leaves the place the drop would have broken.

Cost: Compared with C++ `shared_ptr` and today's Ember (C has no automatic end). Nothing changes per count operation for a value whose maker can run its drop. A check point is still one thread-local load and a branch. New tests come after each job, at each return to C (beside FFI-22's attach check), and where a thread leaves each place that keeps points out; back edges inside those places lose theirs. A sleeping worker is woken (one system call) only for the first value put in its queue. Atomic counts from the making line, at most the measured 9% to 13% of objects passed between threads, apply only to values made under a contract that forbids their drop's effect, or in a scope's task, job or chunk, with a drop that reaches main's module variables. A function judged differently at different call places costs a copy (code size, COST-2) or one plain add and subtract of a per-thread number per place. Ordering a thread's last point before its join costs nothing. To measure on the laptop's P-core, for speed and memory: C backend with MSVC and with Clang, and the LLVM backend.

* verdict 101: the making thread deletes at its next check point, placed by the compiler, always before the thread ends; or the other thread deletes at once if the maker has ended: kept (A queued value is still ended only by its maker, at a point the compiler places, always before the thread ends, and now also before its join is let go. The ended-thread path is word for word the same. Counting a value in the away count from its making line is fix 2's step, taken where the compiler sees the maker can never end it. The arbiter should judge whether that stays within fix 2 (decision 3). Decision 4's option b would widen the ended-thread path, so it goes to the owner and is not applied.)
* verdict 43: a call that may take a lock this thread holds is refused when compiling, nothing checked while running: kept (A hidden end never waits for a lock its own thread holds; placement decides that when compiling. A join under a lock that the joined thread's drops take is refused (E7018), as verdict 43 refuses a call.)
* verdict 57: destroying an object counts as calling its drop: kept (Emptying the queue is a call of the drops, judged at every place it is put. It stays out of a function's summary only because there it would be untrue: the copy called inside a walk of `LOG` runs no such drop. THR-20's count is kept. For the arbiter: this is the one place where verdict 57's 'a function that may destroy an Enemy changes LOG' is not carried into a summary.)
* verdict 104: a scoped task's claims end at its join: kept (A point inside a scope, or in a task's body, is judged against the tasks not yet joined. The task's last point comes before its join, and is judged with the task.)
* verdict 56: an object counts as ended before its drop runs: kept (Unchanged.)
* verdict 114: a moved object's drop runs once, on the other thread: kept (A moved one-thread object is never queued.)
* verdict 86: what a function changes is worked out, never written: kept (Points, and which values are counted from their making line, are worked out from the drops' code.)
* verdict 88: the compiler makes a copy of a function where callers differ: kept (Used for a function whose points or making lines are judged differently at different call places.)
* verdict 122: by field, not by type: kept (For what the queue ends, a view's by-type rule gives way to the object only where the view reads through an owner or a count. Where it cannot, the by-type rule stands.)
* a program changes: Not decided. The inner walk's back edge may run Conn's drop there: it would wait for the lock main holds, or push into the walked list. -> Compiles. The drop runs where the `with` block ends. Later frames print 1, and the join prints 1.
* a program changes: The Textures are never freed while the game runs. -> Each worker is woken and prints 'texture N freed' once, right after the clear.
* a program changes: E3060 at `p.data[i]`, or not decided (the last round's text: a use after free). -> Compiles. No point inside the view's life. The Conn is ended after `p`'s last use. Prints `1 1 24`.
* a program changes: The Textures wait for ever in the idle workers' queues: prints 1000, or more if the joiner ran a job (the last round's text: a race between the woken worker and main). -> Counted for every thread from the making line, judged for the job's place. `bundles.clear()` ends the Textures on main. Prints `1002`.
* a program changes: E7018 (THR-19 with THR-20). Under the last round's text it would also compile and print 0. -> E7018 at `h.join()`, as today, with the fix 'join after the `with pool.lock:` block'.
* a program changes: Not decided whether the drop has run when `join` returns: prints 0 or 1. -> Prints `1`.
* a program changes: E7007 or E3020 if DRP-8 counts main's back edges; otherwise a race or a push into the walked list. -> `499999500000 1001`; `10 5`; `start`, `enemy gone`, `2`; `499500 2`.
* a program changes: Prints 1000, 1001 or 1002; a race once a later job runs on that worker. -> Prints `1002`.
* a program changes: E4003, or the drop runs inside `@noblock poll`. -> Prints `1000 buffers back`.
* a program changes: Not decided: E4003, or the drop runs inside `checksum`. -> Prints 6, 1, 1.
* a program changes: Never ends. -> Prints 7, then `1 mesh freed`.
* a program changes: Not decided (RC-5 point 1 read by type bars every back edge). -> Prints `eu-1 1`, then `1`.
* a program changes: Values queued to main wait until the C loop returns. -> They are ended at each return of `on_frame`, unless an Ember frame below keeps the point out.

**Decision (made by the owner's parameters): Where may a queued drop run while its thread holds a lock the drop takes, walks or views what it changes, waits for a scope whose tasks touch it, or is in a contract function?: A destructor sent back to the thread that made the object (a shared_ptr whose deleter runs at home) must never run inside a lock_guard it also takes, a range-for over the vector it pushes to, or while a view into what it frees is still read.**

step | who | what happens | result
1 | worker | lets go of the last owner of a connection made on main | it goes into main's queue
2 | main | holds the pool's lock and walks the free list | a loop's back edge comes up
3 | main, today | may run its queue there | the destructor waits for the lock main holds: hang
4 | main, after | runs the queue only where calling the destructor would compile, and as the lock's block ends | the id is given back

Today: The destructor may run inside the lock it needs, the loop it changes, or under a reader of what it frees.

* A: refuse such loops; you copy the list under the lock and walk the copy
* B: the compiler runs the queue only where calling the destructor would compile, and as soon as the thread leaves the place that keeps it out
* C: such objects use atomic counts everywhere (9% to 13% slower for them)

Recommendation: B: nothing to write, nothing refused, no speed lost.

Chosen: B. By: 1. Goals: A costs readability, C costs speed, B costs neither. 2. Safety: all three are safe. 3. Compile time first: A refuses correct programs, B decides when compiling and refuses none. 4. No written word under B. 5. The ordinary program is unchanged under B and longer under A. 6. Consistency: B is verdict 101's 'placed by the compiler' with verdict 57 applied where the call is placed. C undoes verdict 101's two counts.

**Decision (made by the owner's parameters): When is a value queued to an idle job worker, or to a thread between C's calls into Ember, ended?: A destructor sent back to a sleeping thread-pool worker (a deleter that must run on the thread that made the object) must still run.**

step | who | what happens | result
1 | worker job | makes a texture in a bundle for main | the worker sleeps
2 | main | clears the bundles | each texture goes to its worker's queue
3 | worker, today | sleeps; no point ever comes | never freed
4 | worker, after | is woken by its queue, frees it, sleeps again | freed at once

Today: A texture sent back to a sleeping worker is never freed while the game runs.

* A: wake the worker to run its queue; every job's end and every callback's return also run it
* B: everything a worker or a C thread makes uses atomic counts (slower for all of it)
* C: keep today: it waits for that worker's next job, maybe never

Recommendation: A: freed within one job, or at once if the worker sleeps.

Chosen: A. By: 1. Goals: B gives up the two-count speed for worker-made values. C leaves a correct program not doing what it says. 2. Safety: all three are safe. 3. Run time is needed, since no compile-time point exists here. A is the cheapest: one queue test where the thread already stops, and one wake only for a sleeping worker. A job's point comes after its end is recorded, because a worker's queue holds values from any earlier job. 4. No written word. 5. The ordinary program is unchanged. 6. A is verdict 101's 'at its next check point' for every kind of thread.

**Decision (made by the owner's parameters): A value whose maker's own check points may never run its drop: A destructor that must not be sent back to its maker: it writes main's own log with no mutex, or its maker spins for ever in a loop that promises never to wait.**

step | who | what happens | result
1 | loader job (or a helper it calls) | makes a texture whose destructor writes main's log | it goes to main in a bundle
2 | main | clears the bundles | the texture would go back to the pool thread
3 | without a fix | the pool thread runs the destructor beside main | two threads write the log: a race
4 | after | such an object is counted for every thread from the moment it is made | main runs the destructor during the clear

Today: Never freed, or, once that pool thread wakes, a race with main.

* a: count such objects for every thread from the moment they are made, wherever the making line is called from
* b: refuse the loader (main's log needs a mutex) and let the busy-poll wait for ever
* c: the pool thread passes the object on to main

Recommendation: a: nothing refused, nothing leaked, a small cost only for those objects.

Chosen: a. By: 1. Goals: b costs ease and leaves the leak. c adds a hidden path and leaves the leak. a costs atomics for those values only. 2. Safety: all three are safe, and a is safe for helpers too when judged by where the line runs. 3. b refuses a program every visible rule accepts. a reuses point 4's hand-over, and c is a second path. 4. No written word. 5. The ordinary program is unchanged. 6. a is verdict 101's fix 2 step, taken at the making line and judged per place, as point 3 judges its points.

**Decision (for the owner): A value queued to a host thread after its call into Ember has returned to C: A destructor sent back to a C library's thread that has gone back into C.**

step | who | what happens | result
1 | C loader thread | our callback makes a texture for main | the callback returns to C
2 | main | lets it go | it goes to that C thread's queue
3 | option a | freed at that thread's next callback | maybe never
4 | option b | main frees it at once, as for a finished thread | freed right away

Today: It waits until C calls into Ember on that thread again, possibly never.

* a: keep: freed at that thread's next call into Ember
* b: while a thread runs only C, whoever lets go of the last owner frees it at once; one atomic step per call from C
* c: everything made on a C thread uses atomic counts

Recommendation: b: nothing waits on a thread that may never come back.

**Decision (made by the owner's parameters): Does a thread's last check point come before or after its join is let go?: A thread's pending destructors (deleters owed to it) must have run when join() returns, as in C++.**

step | who | what happens | result
1 | worker thread | makes a connection, sends it to main | about to end
2 | main | lets it go | it goes back to the worker's queue
3 | main | joins the worker while holding the pool's lock | the worker's last step needs that lock
4 | after | refused when compiling: join after the lock's block | join always means done

Today: Not said whether a thread's last queued destructors run before or after join returns.

* A: before join returns; a join inside a lock they take is refused with a one-line fix
* B: after join returns; it compiles, but code after join may not see them done

Recommendation: A: join means done; the only refusal moves one line.

Chosen: A. By: 1. Goals: B makes `join` not mean done (ease), and results vary from run to run. 2. Safety: both are safe. 3. A decides when compiling with today's rules (THR-19 with THR-20). Its one refusal moves one line. 5. The ordinary join-then-read program is right under A and varies under B. 6. Consistency: A is verdict 43's way and THR-19's join clause as written. B narrows THR-20.

### What blocks it (1)

* E3-1 (X, contradiction): The promise of verdict 43 and THR-19, 'refused when compiling, nothing checked while running', fails for a `with scope` block inside a lock whenever a task makes a value another thread lets go of. A server that opens connections in scoped tasks and closes them on a closer thread can hang, with no message. This is a hang, not memory unsafety, so it is not S. E2-1's spawned-thread case is fine only because `thread.spawn` is on THR-20's list.
  Arbiter: Line 33 makes the Conn on the task's thread. The Conn travels inside a plain Batch, so it is not handed over (RC-4 point 4, p08 173-178; point 5, 'it travelled inside another object'). Point 4's addition does not apply either: there is no contract, and the Conn's drop reaches no module variable of main. The closer's clear puts the Conn in the task thread's queue (point 2, p08 163-168). Changed point 3 runs the last point 'before what waits for it (its `join`, its scope's end) is let go'. It also says 'for a thread other than the main one, what it may end there is counted in its function ([THR-20]), so every wait for the thread is judged with it ([THR-19], [THR-11])'. Read that way, the task changes `pool.free` under `pool.lock`, so the scope's end inside `with pool.lock:` is E7018 (THR-19, p11 272-273: 'the end of a `with scope` block, whose task changes what the held lock guards'; THR-5, p11 384-386). But the new DRP-8 sentence says these ends are 'not part of that function's effects or summary beyond what [THR-20] counts'. THR-20 counts a drop only for 'a function that may run on another thread and holds such a count' (p11 298-301). The rule's first sentence defines that phrase by a list: 'reachable from `thread.spawn`, from a job submitted outside a scope, from a callback C may run ..., or from an `@unsafe(export ...)` function' (p11 287-290). Scoped tasks are not on it. The narrow reading is forced. Read widely, THR-20's 'a module variable of the main thread they reach is `E7012`' would clash with the same rule's 'reached only by the main thread and the tasks of scopes it opens' (p11 285-286). It would also make the change's own tests scoped_job_value_freed_on_main and scoped_job_helper_value_freed_on_main E7012 instead of `1002`, because their jobs hold an away count of the Texture from the making line. So THR-20 counts nothing for this task, and by DRP-8 the scope's end compiles. Two rules give two outcomes for line 40: E7018, or compile and hang. Without the change, the order of the last point against the scope's end is not stated. The definite hang, and both sentences in conflict, come from the change.

```
import std.thread
from std.sync import Atomic, Lock

static DONE: Atomic[int] = Atomic(0)

class Pool:                                   # a connection pool: a free list behind a lock (std::mutex + std::vector)
    lock: Lock = Lock()
    free: Array[int] guarded by lock = []

class Conn:                                   # one connection, made on a scoped task's thread
    id: int
    pool: thread_shared Pool
    fn drop():                                # gives its id back to the pool
        with this.pool.lock:
            this.pool.free.push(owns copy of this.id)

class Batch:                                  # a plain object: connections to close
    conns: Array[thread_shared Conn] = []

class Inbox:                                  # batches waiting for the closer thread
    lock: Lock = Lock()
    items: Array[Batch] guarded by lock = []

fn close_all(inbox: thread_shared Inbox):     # the closer thread: lets the batches go
    while DONE.load() == 0:
        with inbox.lock:
            inbox.items.clear()               # ends each Batch and its Conn: the Conn goes back to the queue of the thread that made it
    with inbox.lock:
        inbox.items.clear()

fn open_one(pool: thread_shared Pool, inbox: thread_shared Inbox):
    with inbox.lock:
        inbox.items.push(owns Batch(conns=[thread_shared Conn(id=1, pool=thread_shared pool)]))

fn main():
    pool = thread_shared Pool()
    inbox = thread_shared Inbox()
    closer = thread.spawn(owns fn() thread_shared inbox: close_all(inbox))
    with pool.lock:
        with s = thread.scope():
            s.spawn(owns fn(): open_one(pool, inbox))   # danger in C++ terms: a scope's join waiting on a thread whose pending destructor needs the mutex this thread holds
        println(pool.free.len())
    DONE.store(1)
    closer.join()
    with pool.lock:
        println(pool.free.len())
```

Arbiter's rulings on the open objections: 3 real, 2 refuted, 0 already in today's specification

Objections closed in the debate: 14 (14 fixed, 0 withdrawn after a rebuttal)

Gaps found on the way that exist without the change: 2

