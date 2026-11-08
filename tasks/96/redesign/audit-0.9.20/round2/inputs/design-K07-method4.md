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

