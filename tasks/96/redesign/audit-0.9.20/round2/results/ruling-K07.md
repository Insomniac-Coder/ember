# K07: May a safe C or C++ call take a struct that holds raw pointers, and what limits raw pointers under the safe-to-call fact?

## Design 1

**question**: K07: May a safe C or C++ call take a struct that holds raw pointers, and what limits raw pointers under the safe-to-call fact? (J8-08; angle: speed first, then safety.)

**summary**: No. The safe-to-call fact ([FFI-2]) gets one limit, stated where the fact is defined and shared by every spelling that asserts it (`safe fn`, a plain overlay function entry, `@ffi(safe)` on a function-pointer typedef, and `safe` beside `std_function`): no assertion covers a raw pointer that the call carries to foreign code, whether it is the argument itself, is held at any depth by what an argument passes (a field, a union field, an element, a C++ method's object, read from the Ember type), or is the result of a function-pointer type among them (what an Ember callback returns to C); such an assertion is `E5002` where it is written, and the function is `unsafe fn` with `@safety`, with an `extend` wrapper that builds the struct from a checked view inside `unsafe:`. The std_function contradiction that blocked the method-4 team is closed by giving the imported `std::function` the exact shape of [FFI-56]'s C function pointer: with `@ffi(std_function)` alone its value is an `unsafe fn(A…) -> R`, and `@ffi(std_function, safe)` asserts the target safe to call under the same limit. Nothing runs while the program runs: the limit is a walk over declared types at compile time, and the wrapper emits the same stores and the same single call as the C it replaces.

- EDIT rule=[FFI-11] table, row ownership (the `retained` outcome)
  - old: (`[FFI-23]`, `[FFI-54]`): the call stays `unsafe`, with a raw pointer |
  - new: (`[FFI-23]`, `[FFI-54]`): the entry is written `unsafe fn`, with a raw pointer (`[FFI-2]`'s limit) |
  - why: The row said a plain entry's call 'stays unsafe' although [GRM-35] says a function entry 'asserts the function safe to call'; with the limit in [FFI-2], an entry whose call carries a raw pointer is refused unless written `unsafe fn`, so the row must say the written form (verdicts 130, 136: no line silently means unsafe).

- EDIT rule=[FFI-11] other contract words
  - old: and on a
  function-pointer typedef, `safe` (`[FFI-56]`). `@ffi(effects=[…])` states effects.
  - new: and on a
  function-pointer typedef or a `std_function` type entry, `safe` (`[FFI-56]`, `[FFI-17a]`). `@ffi(effects=[…])` states effects.
  - why: The vocabulary table must list the one new place the existing word `safe` may stand; no new word is added (verdict 133: `unsafe` stays no contract word).

- EDIT rule=[FFI-2] second sentence
  - old: Supplying the facts, in an `@ffi(…)` attribute or an overlay (§XVI.4), makes the
  call safe.
  - new: Supplying the facts, in an `@ffi(…)` attribute or an overlay (§XVI.4), makes the
  call safe, within the limit on raw pointers below.
  - why: Read alone the sentence promises that complete facts always make a call safe; the limit qualifies exactly this promise.

- EDIT rule=[FFI-2] (append: the limit on raw pointers)
  - old: 
  - new: No assertion of the fact — `safe fn` (`[FFI-10]`), an overlay function entry not written `unsafe fn` (`[GRM-35]`), `@ffi(safe)` on a function-pointer typedef (`[FFI-56]`) or `@ffi(std_function, safe)` on a `std::function` type (`[FFI-17a]`) — covers a raw pointer (`*T`, `*mut T`) that a call carries to foreign code: an argument itself; one held, at any depth, by what an argument passes, by value or behind a contracted pointer (`one`, `count(n)`, `fixed(N)`, `exclusive`, `owns`), a C++ method's object included, as its Ember type declares it (a field, a union's field, an element of a `[T; N]`, a `Span`, a `MutSpan`, an `Option` or a `CppVector`; an `extern type`, an opaque class, a `@ffi(handle)` type and a bridge type declare none, `[FFI-8]`, `[FFI-17e]`); and the result, itself or at any depth in it, of a function-pointer type among them, since that is what an Ember function given to the call returns to foreign code. Safe code can make a raw pointer that is null (`null[P]()`), that points at fewer elements than a count beside it says or at a value already ended (`ref_to_ptr`, `as_ptr()`, §IV.5, `[SPN-8]`), or that holds a constant address (`[FFI-6a]`), and no contract reaches it (`[FFI-1]`, `[FFI-30a]`), so nothing checks one. An assertion of the fact for a function whose call would carry one is `E5002` where it is written, naming the path to the pointer (`buf.data`, `pCreateInfo.pNext`, `get_buf`'s result) and giving the lines that leave the call unsafe: `@safety("…")` above the declaration or entry written `unsafe fn` (a hand declaration keeping its C ABI parameters, `[FFI-10]`), or the typedef's or type's entry without `safe`; and each call in an `unsafe:` block with a `# SAFETY:` note, or a wrapper in an `extend` block (`[FFI-13]`) that builds the value from checked views inside `unsafe:`; for an entry of another package's overlay, that entry as an `override` in the importing package's own overlay (`[FFI-30c]`). A message elsewhere that offers an assertion of the fact (`safe fn lib_step`, a `callback(…)` word, `@ffi(safe)`) offers it only where this limit allows it (`[DIA-25]`). A raw pointer foreign code hands Ember (a result, an `out` value, a callback's or an export's parameter) is outside the limit: reading or writing through it is unsafe code (§IV.5). Nothing runs while the program runs for this rule: the limit is read from the declared types.
  - why: This is where the safe-to-call fact is defined and where every spelling that asserts it points; one limit here reaches all of them, so no list elsewhere can disagree. The reach is a property of types in the Ember-to-foreign direction, never of a program's shape, and it is decided by the package that compiles the declaration from the header, the overlay and the types it sees ([BLD-16]).

- EDIT rule=[FFI-10] the `safe fn` list
  - old: `@layout(c)` value type, or a pointer whose contract is complete (given with `@ffi(…)` on the
  declaration), `safe` on any other function being `E5002`, naming the fact that is missing; or
  - new: `@layout(c)` value type, or a pointer whose contract is complete (given with `@ffi(…)` on the
  declaration), within `[FFI-2]`'s limit on raw pointers, `safe` on any other function being `E5002`, naming the fact that is missing or where the raw pointer is held; or
  - why: The hole showed in this list: 'a `@layout(c)` value type' let `PixelBuf` through, and for J8-08 no fact is missing, so the message clause must name the other cause.

- EDIT rule=[FFI-10] the raw-pointer sentence
  - old: A raw `*T` can be declared for an unsafe call, but complete facts do not make a
  raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`.
  - new: A raw `*T` can be declared for an unsafe call, but complete facts do not make a
  raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002` (`[FFI-2]`'s limit, for a pointer that is the parameter or result itself).
  - why: Makes the existing sentence the hand-declaration case of the one property instead of a second rule of its own.

- EDIT rule=[FFI-56]
  - old: scalar or a `@layout(c)` value type (`E5002` otherwise): an asserted fact, listed by
  - new: scalar or a `@layout(c)` value type, within `[FFI-2]`'s limit on raw pointers (`E5002` otherwise): an asserted fact, listed by
  - why: The typedef's list grants a permission that would otherwise contradict [FFI-2]'s refusal of `@ffi(safe) type mix_fn` whose struct parameter holds `float* samples`.

- EDIT rule=[FFI-17a] table, row `std::function<R(A…)>`
  - old: a function value of type `fn(A…) -> R` (`[CLO-3]`) which owns its target, a foreign function: for `[EFF-2]` and `[CLO-17]`
  - new: a function value which owns its target, a foreign function: of type `unsafe fn(A…) -> R` (`[FN-6]`; a call of it is in `unsafe:`, `E3100`, as a call through a C function pointer is, `[FFI-56]`, since nothing tells the compiler what the target does with its arguments), or of type `fn(A…) -> R` (`[CLO-3]`) with `@ffi(std_function, safe)` on the entry, which asserts the target safe to call within `[FFI-2]`'s limit on raw pointers (`E5002` otherwise), an asserted fact `ember tcb` lists (`[TCB-4]`); for `[EFF-2]` and `[CLO-17]`
  - why: The one contradiction left by the method-4 fix: today the row derives the safe-to-call fact for a foreign target, against [FFI-2] ('never derived, because a scalar can be an address'), and a limit on raw pointers would then leave a `std::function<void(Buf)>` with no line that compiles. [FFI-56]'s shape (unsafe to call until `safe` is written; `safe` refused for raw pointers) keeps the feature, gives every program a compiling line, and uses words the language already has (`safe` in `@ffi(…)`, `no_callbacks` beside `std_function`).

- EDIT rule=[TCB-4]
  - old: and the C++ bridge section each
  `@ffi(const)` (`[FFI-40a]`).
  - new: and the C++ bridge section each
  `@ffi(const)` (`[FFI-40a]`) and `@ffi(std_function, safe)` (`[FFI-17a]`).
  - why: Every asserted fact is listed by `ember tcb` ([TCB-3]); the new assertion needs its row.

- EDIT rule=[CLI-5]
  - old: A declaration it leaves
  unsafe to call is written `unsafe fn` (verdict 130).
  - new: A declaration it leaves
  unsafe to call, one that `[FFI-2]`'s limit on raw pointers keeps from being safe among them, is
  written `unsafe fn` (verdict 130).
  - why: `ember bind --init pixels.h` would otherwise write `fn clear_pixels(buf)`, which [FFI-2] refuses, against CLI-5's 'compiles unchanged'.

- EDIT rule=registry row E5002
  - old: | `E5002` | a foreign declaration whose contract cannot stand: a hand declaration that says neither `safe fn` nor `unsafe fn`, or both; `safe fn` on a function whose contract is incomplete, or two destructors for one foreign type (a call of a function without a verified contract outside `unsafe` is `E3100`); a function-pointer parameter of a `safe fn` with no `callback(…)` word; a `user_data` witness too narrow for the tagged number | `[FFI-2]`, `[FFI-10]`, `[FFI-36a]`, `[FFI-21]` |
  - new: | `E5002` | a foreign declaration whose contract cannot stand: a hand declaration that says neither `safe fn` nor `unsafe fn`, or both; `safe fn` on a function whose contract is incomplete; an assertion of the safe-to-call fact (`safe fn`, a plain overlay function entry, `@ffi(safe)`, `@ffi(std_function, safe)`) for a function whose call carries a raw pointer to foreign code, as an argument, held at any depth by what an argument passes, or returned by a function-pointer type among them (`[FFI-2]`'s limit); or two destructors for one foreign type (a call of a function without a verified contract outside `unsafe` is `E3100`); a function-pointer parameter of a `safe fn` with no `callback(…)` word; a `user_data` witness too narrow for the tagged number | `[FFI-2]`, `[FFI-10]`, `[FFI-56]`, `[FFI-17a]`, `[FFI-36a]`, `[FFI-21]` |
  - why: One code, one defining rule: the registry lists every cause of the code with its reach, so a compiler built from it agrees with [FFI-2]; [FFI-56]'s own `E5002` was missing from the rule column.

- EDIT rule=§XVI.4 the Vulkan overlay example (above [GRM-35])
  - old:     @ffi(effects=[FFI, Alloc], no_callbacks)
    fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed one,
                      pAllocator: nullable borrowed one, pBuffer: out one) -> status
  - new:     @safety("`pCreateInfo.pNext` is null or a valid chain and `pCreateInfo.pQueueFamilyIndices` points at `queueFamilyIndexCount` indices; `pAllocator`, when given, holds valid callbacks and user data")
    @ffi(effects=[FFI, Alloc], no_callbacks)
    unsafe fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed one,
                             pAllocator: nullable borrowed one, pBuffer: out one) -> status
  - why: `VkBufferCreateInfo` holds `pNext` and `pQueueFamilyIndices`, and `VkAllocationCallbacks` holds `pUserData` and function pointers returning `void*`, so the limit refuses the safe entry; the document's own example must compile under its rules. K46 (J8-14) edits `device: borrowed` on the same lines: the two edits touch different words and are applied as one rewrite.

- EDIT rule=§XVI.4, after the example's closing fence (before [GRM-35])
  - old:     hide vkAllocationFunction
```

  - new:     hide vkAllocationFunction
```

`vkCreateBuffer` is an `unsafe fn` entry because the structs its call hands Vulkan hold raw pointers
(`pNext`, `pQueueFamilyIndices`, `pUserData`; `[FFI-2]`'s limit), while `vkCmdSetViewport` is safe: a
`VkViewport` holds only floats, and `commandBuffer` is a `handle`, which safe code cannot forge
(`[FFI-8]`). A wrapper in an `extend` block of the overlay (`[FFI-13]`) gives safe callers a create
call, building those structs from checked views inside `unsafe:`.

  - why: Part XVI's opening promises to say 'after which calls are safe'; a reader who sees the headline Vulkan call turn unsafe learns here why, why the neighbour stays safe, and the tool the language already has for the safe surface.

**refused_program**: // pixels.c, linked into the program
// #include <stddef.h>
// #include <stdint.h>
// typedef struct { uint8_t* data; size_t len; } PixelBuf;
// void clear_pixels(PixelBuf buf) { for (size_t i = 0; i < buf.len; i++) buf.data[i] = 0; }

#! language 0.9.20

@layout(c)
struct PixelBuf:
    data: *mut u8
    len: usize

unsafe extern "C":
    @ffi(no_callbacks)
    safe fn clear_pixels(buf: PixelBuf)

fn main():
    x: u8 = 7
    clear_pixels(PixelBuf(data=ref_to_ptr(ref mut x), len=4096))   # every line here is safe code
    println(x)

E5002, line 10: "`clear_pixels` is declared `safe fn`, but a call of it hands C a raw pointer, `buf.data` (`PixelBuf`, line 5), and nothing checks where it points or for how many bytes (`buf.len` is a number safe code chooses); write `unsafe fn clear_pixels(buf: PixelBuf)` with `@safety(\"…\")` above it saying what `buf.data` must point to, and put line 15 in an `unsafe:` block with a `# SAFETY:` note, or give safe callers a wrapper that builds `PixelBuf` from a `MutSpan[u8]` inside `unsafe:`"

The same limit refuses, with the same code: an overlay line `fn cp_load(opts: borrowed one)` whose header struct holds `const char* path` (naming `opts.path`); `@ffi(safe) type mix_fn` whose block parameter holds `float* samples` (naming `blk.samples`); `safe fn stream_read(want: usize, get_buf: extern "C" fn(usize) -> *mut u8)` (naming `get_buf`'s result); and `@ffi(std_function, safe) type Fill` for `std::function<void(Buf)>` with `Buf { uint8_t* data; size_t len; }` (naming `Buf.data`), whose line without `safe` compiles and whose calls then go in `unsafe:`.

**accepted_program**: // pixels.c as above: void clear_pixels(PixelBuf buf) zeroes buf.len bytes at buf.data

#! language 0.9.20

@layout(c)
struct PixelBuf:
    data: *mut u8
    len: usize

unsafe extern "C":
    @safety("`buf.data` points at `buf.len` writable bytes that live for the call")
    @ffi(no_callbacks)
    unsafe fn clear_pixels(buf: PixelBuf)

fn clear(ref mut bytes: MutSpan[u8]):
    # SAFETY: the struct describes exactly the bytes of `bytes`, a checked view that lives for the call
    unsafe:
        clear_pixels(PixelBuf(data=bytes.as_mut_ptr(), len=bytes.len()))

fn main():
    frame: Array[u8] = [7, 7, 7, 7]
    clear(ref mut frame)        # an Array converts to MutSpan[u8] for a `ref mut` parameter ([SPN-1])
    println(frame)              # [0, 0, 0, 0]

Safe callers can only clear bytes they own; `clear(ref mut frame)` is shorter than building `PixelBuf` by hand. The overlay form of the same feature: `@safety("…") unsafe fn clear_pixels(buf)` as the entry and `clear` in an `extend` block of the overlay ([FFI-13]). A C++ `std::function<void(Buf)>` is called from `unsafe:` with a `# SAFETY:` note under `@ffi(std_function)` alone, or safely under `@ffi(std_function, safe)` when `Buf` holds no raw pointer.

**runtime_cost**: Nothing. The limit is a walk over declared types when the declaration is compiled; no instruction is emitted for it, and the §X.4 row "FFI call | guaranteed present | one direct call for an ABI-direct C function" is unchanged. The wrapper fills the struct from the span's pointer and length, the stores C itself writes to build the struct, and makes the same single call ([FFI-9]); after inlining it is the same instructions as the direct C call (a cross-package wrapper not inlined costs one extra call, as [FFI-29c] already prices a shim, removed by LTO, [BLD-FFI-4]). A `std::function` value's call keeps its thunk; `unsafe fn(A…) -> R` against `fn(A…) -> R` changes no code. To measure: `clear(ref mut frame)` against a hand-written C call with the same pointer and length, both compilers, same flags; expected identical time and code size.

**implementer_steps**: 1. Add one predicate over Ember types, memoized per type: `carries_raw_pointer_to_foreign(T)`: true for `*T`/`*mut T`; for a `@layout(c)` struct or union, true if any field is; for `[T; N]`, `Span[T]`, `MutSpan[T]`, `Option[T]`, `CppVector[T]`, `CppShared[T]` and any other type argument that names what a value holds or points at, true if the argument is; for a function-pointer type (`extern "C" fn(P…) -> R`, `unsafe extern "C" fn(…)`), true if `R` is (the parameters `P…` are the C-to-Ember direction and are not walked); false for scalars, `@repr` enums, `CStrView`, `extern type`, opaque C++ classes, `@ffi(handle)` types, `CppString` and every type that declares no fields (cycles through pointers end at the pointer itself).
2. At each site that asserts the safe-to-call fact — a hand `safe fn` ([FFI-10]), an overlay function entry not written `unsafe fn` (a class member's entry includes its object type, [FFI-40]), `@ffi(safe)` on a function-pointer typedef ([FFI-56]), `@ffi(std_function, safe)` on a type entry ([FFI-17a]) — run the predicate over each parameter's Ember type as the contract maps it (the pointee for `one`, `count(n)`, `fixed(N)`, `exclusive`, `owns`, `nullable`), and, for a C++ method, over the object's type; on a hit, emit `E5002` at the assertion with the first path found (`buf.data`, `pCreateInfo.pNext`, `get_buf`'s result) and the per-spelling fix lines. Keep [FFI-10]'s existing raw-pointer-slot refusal as the degenerate case (depth zero).
3. Import `std::function` with `@ffi(std_function)` as a value of type `unsafe fn(A…) -> R` (the existing owned-callable lowering, destructor thunk and [EFF-2]/[FFI-52] treatment unchanged); with `safe` in the same `@ffi(…)` run step 2 and give `fn(A…) -> R`. Record the `safe` assertion for `ember tcb`'s C++ bridge section.
4. The registry row of `E5002` gets the new cause; `ember bind --init` and `--check/--report` call the same predicate so a declaration the limit keeps from being safe is written and counted `unsafe fn`; messages that offer `safe fn`, a `callback(…)` word or `@ffi(safe)` consult the predicate first and omit the offer when it would be refused ([DIA-25]).
5. Tests: the refused program above (break-test: remove the predicate call, see it compile); the accepted program printing `[0, 0, 0, 0]`; the overlay, `@ffi(safe)` typedef, callback-result and `std_function` variants; `vkCmdSetViewport` and a `count(n)` span of `handle` values still safe; a C-returned struct with a pointer field read in `unsafe:` still accepted; a `std_function` without `safe` called outside `unsafe:` is `E3100`.
Separate compilation and hot reload need nothing new: the predicate reads only the header, the overlay and the declared types the compiling package sees ([BLD-16]); a changed overlay entry changes only Ember code ([HR-23]), and the call sites it turns unsafe are recompiled as any changed Ember code is.

**departs_from_verdict**: None. Verdict 60 (anything that can break a safety promise only inside `unsafe`): applied, the route is closed. Verdicts 120, 125, 130, 133, 136: every refused assertion becomes `@safety("…")` above `unsafe fn`, in a hand declaration, an overlay entry or a C++ member's entry, and no plain line silently means unsafe (the plain overlay entry is refused where it is written, not quietly made unsafe). Verdict 132's lesson (an unwritten attribute never falls back to an inferred fact): a plain overlay line keeps its written meaning and is refused when that meaning cannot hold. Verdict 137: results are outside the limit, not touched. Verdict 121: an entry of another package's overlay that the limit refuses fails every build that applies it, with the `override` line for the importing package. Verdict 73 / [DIA-25]: every fix line the message gives compiles, and messages elsewhere stop offering an assertion the limit refuses. Delegated rulings kept: ODR-073 (its reason, 'a safe caller may pass a null, dangling or forged pointer', is the limit's reason, applied to a pointer in a field, an element or a callback result), ODR-074 (an unsafe hand declaration keeps its C ABI parameters), ODR-075 (`ref_to_ptr` stays safe; only handing the pointer to foreign code under an assertion of safety is refused), ODR-199 (`@ffi(std_function)` still makes an owned callable value; only its call's safety axis is now written, as [FFI-56] writes a C function pointer's). The std_function change touches no owner verdict; it is reported as the one change that refuses programs 0.9.20_Hardened_2 accepts (a call of a `std_function` value outside `unsafe:` without `safe` on the entry), in the spirit of §H.52 (every change of 0.9.20 is a refusal).

**trade_offs**: Speed: nothing is given up; the limit costs no instruction and the wrapper is the C code's own stores and call, which is why this design is preferred over any run-time check (none could establish a pointer's validity anyway). Safety: closed at compile time for every type that carries a pointer and every route by which a call's arguments reach foreign code, including a callback's result and a `std::function` target; the dual route, an Ember export or an `override fn` of a C++ virtual that returns a raw pointer made in safe code, is outside the safe-to-call fact and is named under remaining doubt. Readability: a program that calls a binding gets a shorter line (`clear(ref mut frame)`); a binding author writes one `unsafe fn` entry with `@safety` and one wrapper per function whose structs carry pointers, the price Rust binding authors pay too, heaviest for Vulkan where every create-info has `pNext`. A program that only passes back to C a struct C itself returned (a camera frame with its pixel pointer) is refused as a safe call too, since the limit reads types, not who filled the field; the binding keeps such a struct in a private field of an Ember type behind a wrapper. Field contracts (an overlay stating `pQueueFamilyIndices: count(queueFamilyIndexCount)`) would let some such structs become safe types, but [FFI-30a] rules out an overlay touching fields, `pNext` chains could never be contracted, and it is a whole feature; not proposed. Consistency: one property in one rule, cited by the four lists that assert the fact; the `retained` row stops contradicting [GRM-35]; `std::function` follows the nearest rule ([FFI-56]) with existing words. Cost of the std_function change: existing programs calling a `std_function` value outside `unsafe:` must add `safe` to the entry (refused by the limit when the arguments carry a raw pointer) or move the call into `unsafe:`; `ember fmt --migrate` rewrites nothing, as for every 0.9.20 refusal.

**remaining_doubt**: 1. The dual route is not closed by this ruling and is the same class of hole: an Ember function that foreign code calls — an `@unsafe(export(…))` function ([FFI-26], [FFI-31b] lists no refusal of a raw-pointer result), an `override fn` of a C++ virtual ([FFI-39], an `uint8_t* get_buf(size_t)` virtual) — may return `ref_to_ptr(ref mut local)` in safe code, and C or C++ writes through it. [FFI-21a] covers only a pointer into a copied parameter (`E5042`). The general shape, for K08's lead: a function foreign code calls whose result carries a raw pointer is written `unsafe fn` with `@safety`, so its body is unsafe code (verdict 118) and the author vouches for what the pointer points to and how long. 2. [FFI-11]'s `nullable` `Option[T]` input and `borrowed one nullable` carriers: the walk into `T` is stated in the limit; the implementer must apply the contract mapping before the walk, not the raw C type, so a `handle` behind `nullable` stays safe. 3. Whether `@ffi(std_function, safe)` should also carry [FFI-56]'s list ('every parameter and result is a scalar or a `@layout(c)` value type') or allow bridge types (`CppString`, `CppVector[T]` without pointers) as this ruling does; the limit refuses the raw pointers either way, and the narrower list would only refuse more correct programs. 4. The `vkCreateBuffer` rewrite must be merged with K46's edit of `device: borrowed` on the same lines; the `@safety` text given here is a proposal for the lead to shorten.

## Design 2

**question**: K07 (J8-08): May a safe C or C++ call take a struct that holds raw pointers, and what limits raw pointers under the safe-to-call fact? Including the one contradiction the method-4 fix left: how an `@ffi(std_function)` value is read.

**summary**: No. The safe-to-call fact ([FFI-2]: "The safe-to-call fact is never derived ... it is asserted by `safe fn` in an extern block (`[FFI-10]`), or by an overlay entry for the function") gets one limit, written where the fact is defined and shared by every spelling that asserts it: no assertion covers a raw pointer or C function pointer that the caller supplies and the call carries to foreign code, as an argument, inside what an argument passes (read from its declared Ember type at any depth), or as what an Ember function given to the call returns; a declaration or entry that asserts the fact and would carry one is `E5002` where it is written, with the `unsafe fn` + `@safety` line and the `unsafe:` block as the way out, and a wrapper over a checked span as the safe interface. The contradiction on `std::function` is closed by giving it the C function pointer's rule ([FFI-56]): `@ffi(std_function)` makes an `unsafe fn(A…) -> R` (its target is "a foreign function", [FFI-17a], and nothing describes it), and `@ffi(std_function, safe)` the `fn(A…) -> R` of today, within the same limit. Nothing runs for any of this.

- EDIT rule=FFI-2 (second sentence, the cite)
  - old: call safe. The safe-to-call fact is never derived
  - new: call safe, within the limit on raw pointers below. The safe-to-call fact is never derived
  - why: Read alone, the sentence promises that complete facts always make a call safe; the limit added at the end of the rule qualifies exactly this sentence, so it points there.

- EDIT rule=FFI-10 (the `safe fn` list)
  - old: `safe` on any other function being `E5002`, naming the fact that is missing;
  - new: and within `[FFI-2]`'s limit on raw pointers, `safe` on any other function being `E5002`, naming the fact that is missing or, under that limit, where the pointer is held;
  - why: The hole showed in this list: 'a `@layout(c)` value type' let `PixelBuf` through. For J8-08 no fact is missing, so the E5002 clause must also name what the message says in the new case. A cite, not a second rule: the hand-declaration sentence 'complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`' stays as the parameter-slot case of the same property.

- EDIT rule=FFI-56 (the `@ffi(safe)` typedef list)
  - old: result is a scalar or a `@layout(c)` value type (`E5002` otherwise)
  - new: result is a scalar or a `@layout(c)` value type within `[FFI-2]`'s limit on raw pointers (`E5002` otherwise, naming the missing fact or where the pointer is held)
  - why: FFI-56 grants a permission with FFI-10's list; without the cite it would accept `@ffi(safe) type mix_fn` whose struct parameter holds `float* samples`, which [FFI-2] refuses, and two rules would decide one entry differently.

- EDIT rule=FFI-11 (other contract words, the `safe` word)
  - old: function-pointer typedef, `safe` (`[FFI-56]`).
  - new: function-pointer typedef or a `std_function` type entry, `safe` (`[FFI-56]`, `[FFI-17a]`), within `[FFI-2]`'s limit on raw pointers.
  - why: The vocabulary names every contract word and where it may stand; `safe` now also stands on a `std_function` type's entry, and both uses share the one limit.

- EDIT rule=CLI-5 (`ember bind --init`)
  - old: unsafe to call is written `unsafe fn` (verdict 130).
  - new: unsafe to call, one whose calls `[FFI-2]`'s limit on raw pointers keeps from being safe among them, is written `unsafe fn` (verdict 130).
  - why: Without it `--init pixels.h` would write `fn clear_pixels(buf)`, which [FFI-2] refuses, and CLI-5's 'It compiles unchanged' would be false for every header whose structs hold pointers.

- EDIT rule=§XVII registry row E5002 (rule column)
  - old: | `[FFI-2]`, `[FFI-10]`, `[FFI-36a]`, `[FFI-21]` |
  - new: | `[FFI-2]`, `[FFI-10]`, `[FFI-36a]`, `[FFI-21]`, `[FFI-56]`, `[FFI-17a]` |
  - why: One code, one defining rule per cause: FFI-56's own E5002 was missing from the column, and FFI-17a now gives the code too.

- EDIT rule=§XVII registry row E5002 (the causes)
  - old: `safe fn` on a function whose contract is incomplete, or two destructors for one foreign type
  - new: `safe fn` on a function whose contract is incomplete; a `safe fn`, an overlay function entry not written `unsafe fn`, or `safe` on a function-pointer typedef's or a `std_function` type's entry whose calls would carry foreign code a raw pointer or C function pointer the caller supplies (an argument, one held at any depth by what an argument passes, or one an Ember function given to the call returns, `[FFI-2]`); or two destructors for one foreign type
  - why: The registry lists every cause of a code with the rule's own reach and trigger ('whose calls would carry'), so a compiler written from the registry agrees with [FFI-2] and does not refuse `PAIR_FIRST(p: *Pair)` under an `unsafe fn` entry, which the limit never reaches.

- EDIT rule=§XVI.4 example (the Vulkan overlay above [GRM-35])
  - old:     @ffi(effects=[FFI, Alloc], no_callbacks)
    fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed one,
                      pAllocator: nullable borrowed one, pBuffer: out one) -> status
  - new:     @safety("`pCreateInfo.pNext` is null or a valid chain and `pCreateInfo.pQueueFamilyIndices` points at `queueFamilyIndexCount` indices; `pAllocator`, when given, holds valid callbacks and user data")
    @ffi(effects=[FFI, Alloc], no_callbacks)
    unsafe fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed one,
                             pAllocator: nullable borrowed one, pBuffer: out one) -> status
  - why: `VkBufferCreateInfo` holds `pNext` and `pQueueFamilyIndices`, and `VkAllocationCallbacks` holds `pUserData` and function pointers, so the document's own safe entry is E5002 under the limit; it takes verdict 130's spelling. K46's edit of `device: borrowed` on the same lines touches different words and merges into one rewrite.

- EDIT rule=§XVI.4 (one sentence under the example)
  - old:     hide vkAllocationFunction
```

* `[GRM-35]`
  - new:     hide vkAllocationFunction
```

`vkCreateBuffer` is an `unsafe fn` entry because the structs its call hands Vulkan hold raw pointers (`pNext`, `pQueueFamilyIndices`, `pUserData`; `[FFI-2]`); an `extend` wrapper in the overlay (`[FFI-13]`) gives safe callers a create call, building those structs from checked views.

* `[GRM-35]`
  - why: Part XVI opens by promising 'after which calls are safe'; a reader who sees the headline Vulkan call turn unsafe learns here, in one sentence, how a binding gives safe callers a safe call with the tool the language already has ([FFI-13]: 'this is how an idiomatic API is layered over a raw one').

- EDIT rule=FFI-17a (table row `std::function<R(A…)>`)
  - old: with `@ffi(std_function)` on its overlay entry, a function value of type `fn(A…) -> R` (`[CLO-3]`) which owns its target, a foreign function:
  - new: with `@ffi(std_function)` on its overlay entry, a function value which owns its target, a foreign function, of type `unsafe fn(A…) -> R` (`[FN-6]`), since nothing describes what the target does with its arguments, as for a C function pointer (`[FFI-56]`; a call of it outside unsafe code is `E3100`: "`fill` holds a `std::function` from `fill.hpp` whose target nothing describes; put line 6 in an `unsafe:` block with a `# SAFETY:` note saying why the call is safe", and, where the entry may say `safe`, "or, if a call of it within its signature cannot break memory safety, write `@ffi(std_function, safe)` on `type Fill`"), or of type `fn(A…) -> R` (`[CLO-3]`) with `@ffi(std_function, safe)` on the entry, an asserted fact `ember tcb` lists (`[TCB-1]`), allowed when every parameter and the result is a scalar, a `@layout(c)` value type or a bridge type (`[FFI-17e]`) within `[FFI-2]`'s limit on raw pointers (`E5002` otherwise, naming where the pointer is held):
  - why: This is the one spelling that asserted the fact with no limit at all, and the contradiction the method-4 fix left: read literally, its limit refused `std_function` entries with no line that compiles, while FFI-17a accepted them. Hardened_2 made the value's target 'a foreign function', and [FFI-2] says that fact 'is never derived, because a scalar can be an address': so the value is unsafe to call until the entry vouches, exactly as a C function pointer typedef is ([FFI-56]), with FFI-56's own word `safe`; `@ffi(…)` already takes several words on one entry ('`@ffi(no_callbacks)` beside `std_function`'). No feature goes: every `std::function` stays callable, in `unsafe:` or vouched with one word, and ODR-199 ('an owned callable value') stands, an `unsafe fn(A…) -> R` being one.

- EDIT rule=FFI-2 (the limit on raw pointers, appended)
  - old: 
  - new:  No assertion of the fact — `safe fn` (`[FFI-10]`), an overlay function entry not written `unsafe fn` (`[GRM-35]`), or `safe` on a function-pointer typedef's or a `std_function` type's entry (`[FFI-56]`, `[FFI-17a]`) — covers a value the caller supplies that names memory or code nothing checks: a raw pointer (`*T`, `*mut T`, an `ObjectPtr[C]`, `[FFI-54]`) or a C function pointer (`unsafe extern "C" fn(…)`, `[FFI-56]`), alone or inside an `Option`, that the call's arguments carry to foreign code: an argument itself; one held, at any depth, by what an argument passes by value or behind a contracted pointer, read from the argument's Ember type as it is declared, its fields, its elements and its type arguments, a method's object included (a type that declares no fields, an `extern type`, an opaque class or a `handle`, holds none; of a callable type only the result is read, its parameters being what foreign code hands the function); or one that an Ember function given to the call, as an argument or inside one, returns, itself or inside its result. Safe code makes such a value null (`null[P]()`, `None`), one that outlives what it points at or counts fewer elements than a length beside it says (`ref_to_ptr`, `as_mut_ptr()`, §IV.5, `[SPN-8]`), or one holding any address (a pointer-cast constant, `[FFI-6a]`; a function pointer C handed back), and no contract word reaches it: words attach to parameters and results only (`[FFI-11]`), and no overlay touches a field (`[FFI-30a]`). So a declaration or entry that asserts the fact and whose calls would carry one is `E5002` where it is written, naming where the value is held: "`clear_pixels` is declared `safe fn`, but `buf.data` is a raw pointer (`*mut u8`) that its call hands C, and safe code can put any address in it (line 12); write `@safety("…")` above `unsafe fn clear_pixels(buf: PixelBuf)`, saying what `buf.data` must point to, and put line 12 in an `unsafe:` block with a `# SAFETY:` note, or give safe callers a wrapper that fills `buf` from a `MutSpan[u8]`". The lines that leave the calls unsafe are `@safety("…")` above the declaration or entry written `unsafe fn` (a hand declaration with its C ABI parameters, `[FFI-10]`), or the typedef's or type's entry without `safe`, and each call outside unsafe code in an `unsafe:` block with a `# SAFETY:` note (`[UNS-1]`, `[DIA-25]`); for an entry in another package's overlay, the message gives that entry as an `override` in the importing package's own overlay (`[FFI-30c]`). The check reads the declaration and the header's types, which every package importing the header sees alike (`[FFI-30a]`), never the program (`[BLD-16]`), and nothing runs for it. A value foreign code hands Ember (a result, an `out` value, a callback's parameter, a field of what it returns) is outside this limit: reading through it is unsafe (`[UNS-1]`), and `[FFI-10]` decides a hand declaration's raw-pointer result.
  - why: This is where the safe-to-call fact is defined and where every spelling that asserts it points, so one limit here reaches all of them at once; placed after the DIA-18 sentence it keeps 'An entry written `unsafe fn` asserts nothing of the kind' beside what it refers to. It states a property (a value the caller supplies that names memory or code nothing checks, and the direction it flows), never a type or a program's shape: it reaches a struct, a union, a fixed array, a span's or `CppVector`'s elements, a nested struct, a C++ method's object and a callback's result alike, and stops at types that declare no fields, so opaque classes, `extern type`s and handles are untouched. The reason is ODR-073's ('a safe caller may pass a null, dangling or forged pointer; the contract cannot establish validity'), which is about any assertion of safety, not about parameter slots. The trigger 'asserts the fact' keeps every entry whose own words already leave its calls unsafe (an unknown fact, an uncontracted raw-pointer slot, `retained` on a plain pointer, a `TODO(…)` word) exactly as today. C function pointers are in the property for the same reason as data pointers (NG-16/NG-18 of the gap sweep): a hook table whose `release` field safe code filled from a constant or from another library is code C calls with arguments nothing checks, while a safe `extern "C" fn` made from an Ember function is checked and never matches the limit's types.

**refused_program**: ```ember
@layout(c)
struct PixelBuf:
    data: *mut u8      # C writes `len` bytes here
    len: usize

unsafe extern "C":
    @ffi(no_callbacks)
    safe fn clear_pixels(buf: PixelBuf)

fn main():
    x: u8 = 7
    clear_pixels(PixelBuf(data=ref_to_ptr(ref mut x), len=4096))
    println(x)
```

Today: compiles ([FFI-10]'s list accepts "a `@layout(c)` value type"), and C writes 4,096 bytes over one byte of `main`'s frame from safe code.

With the edits, at line 8:

E5002: `clear_pixels` is declared `safe fn`, but `buf.data` is a raw pointer (`*mut u8`) that its call hands C, and safe code can put any address in it (line 12); write `@safety("…")` above `unsafe fn clear_pixels(buf: PixelBuf)`, saying what `buf.data` must point to, and put line 12 in an `unsafe:` block with a `# SAFETY:` note, or give safe callers a wrapper that fills `buf` from a `MutSpan[u8]`

The same limit on the other spellings. An overlay entry `fn cp_load(opts: borrowed one)` whose header struct holds `const char* path`: E5002 at the entry, naming `opts.path`. `@ffi(safe) type mix_fn` whose block parameter holds `float* samples`: E5002 at the typedef's entry, naming `blk.samples`. A `safe fn stream_read(want: usize, get_buf)` with `param(get_buf, callback(borrowed))` whose C signature is `uint8_t* (*)(size_t)`: E5002, naming `get_buf`'s result. And the `std::function` case that split the method-4 text:

```cpp
// fill.hpp
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
```
```ember
# overlays/fill.em
unsafe overlay cpp "fill.hpp":
    @ffi(std_function, safe)
    type Fill
    @ffi(no_callbacks)
    fn make_filler()
```
E5002 at `type Fill`'s entry: "`Fill` is declared `safe`, but `Buf.data`, a field of its first parameter, is a raw pointer (`*mut u8`) that a call hands C++, and safe code can put any address in it; write `@ffi(std_function)` without `safe`, and put each call of a `Fill` value in an `unsafe:` block with a `# SAFETY:` note". Without `safe` the entry compiles and `fill(f.px.Buf(data=ref_to_ptr(ref mut x), len=4096))` outside `unsafe:` is E3100 ("`fill` holds a `std::function` from `fill.hpp` whose target nothing describes; put line 6 in an `unsafe:` block with a `# SAFETY:` note saying why the call is safe").

**accepted_program**: ```ember
@layout(c)
struct PixelBuf:
    data: *mut u8
    len: usize

unsafe extern "C":
    @safety("`buf.data` points at `buf.len` writable bytes")
    @ffi(no_callbacks)
    unsafe fn clear_pixels(buf: PixelBuf)

fn clear(ref mut bytes: MutSpan[u8]):
    # SAFETY: `data` and `len` are the span's own pointer and length, so C writes exactly the bytes `bytes` views
    unsafe:
        clear_pixels(PixelBuf(data=bytes.as_mut_ptr(), len=bytes.len()))

fn main():
    frame: [u8; 4] = [1, 2, 3, 4]
    clear(ref mut frame)           # a `[u8; 4]` given with `ref mut` converts to `MutSpan[u8]` ([SPN-1])
    println(frame[0], frame[3])    # 0 0
```

Safe callers can only clear bytes they own: `clear(ref mut frame)` is shorter than filling the struct by hand, and the one `unsafe:` block sits in the binding under its `# SAFETY:` note. The hand declaration keeps its C ABI parameters (an unsafe declaration is never mapped, [FFI-10]: "An unsafe raw-pointer declaration keeps its written C ABI parameters"), `ember tcb` lists the `unsafe:` block, and `L3015` never fires because `@safety` is written (verdict 120).

The `std::function` of the refused program, with the C function pointer's rule:

```ember
# overlays/fill.em
unsafe overlay cpp "fill.hpp":
    @ffi(std_function)
    type Fill
    @ffi(no_callbacks)
    fn make_filler()

# main.em
import cpp "fill.hpp" with (project="fill", overlay="overlays/fill.em") as f

fn zero(ref mut bytes: MutSpan[u8], fill: f.px.Fill):
    # SAFETY: the struct describes exactly the bytes `bytes` views, and the target writes `len` bytes from `data`
    unsafe:
        fill(f.px.Buf(data=bytes.as_mut_ptr(), len=bytes.len()))

fn main():
    filler = f.px.make_filler()
    frame: [u8; 2] = [7, 7]
    zero(ref mut frame, filler)
    println(frame[0])              # 0
```

A `std::function<int(int)>` whose entry says `@ffi(std_function, safe)` stays a plain `fn(int) -> int`, called anywhere, as today.

**runtime_cost**: Nothing. The limit is decided when the declaration, overlay entry or typedef is compiled, from types the compiler already has; no instruction is emitted for it (§X.4 row "Contract and range-type annotations | not observable | compile time only"). The call that replaces the refused safe call is the same one direct call (§X.4 row "FFI call | guaranteed present | one direct call for an ABI-direct C function"); the wrapper fills the struct from the span's pointer and length, as C code fills it, and inlines to the same instructions. A `std_function` value's call is unchanged (one call through the owned target and the thunk FFI-17f already reports); only the word `safe` on its entry changes where the call may be written.

**implementer_steps**: 1. One predicate over Ember types, `carries_unchecked_pointer(ty) -> Option<path>`: a raw pointer (`*T`, `*mut T`, `ObjectPtr[C]`) or an `unsafe extern "C" fn(…)` type is a hit (path = the place); `Option[X]`, `[X; N]`, `Span[X]`, `MutSpan[X]`, a tuple, a `@layout(c)` struct or union, a generic bridge type (`CppVector[X]`) walk their fields, elements and type arguments (memoised per type; a pointer is a leaf, so the walk ends); a callable type walks its result only; an `extern type`, an opaque C++ class, a `@ffi(handle)` type, a `@ffi(newtype)` of a scalar and every scalar are misses. The path is a dotted name from the parameter (`buf.data`, `pCreateInfo.pNext`, `get_buf`'s result, `Buf.data` of parameter 1).
2. In the hand-declaration checker ([FFI-10]): for a `safe fn`, after the existing slot checks, run the predicate on every parameter's mapped Ember type (the value type behind `one`, `count(n)`, `owns`, `fixed(N)`; a `callback(…)` parameter's `fn(P…) -> R`); a hit is E5002 at the declaration with the new message (the `@safety` + `unsafe fn` line with the declaration's C ABI parameters, the call lines that need `unsafe:`, and the wrapper hint when the hit is a data pointer).
3. In the importer's overlay binder ([GRM-35], [FFI-2]): for every function entry not written `unsafe fn` (a class section's methods included, with the class type as the object), run the predicate on each parameter's produced Ember type and on the receiver; E5002 at the entry, with the `override` form in the message when the overlay belongs to another package ([FFI-30c]). For a typedef entry with `safe` ([FFI-56]) and a type entry with `std_function, safe` ([FFI-17a]): run it on each parameter of the pointed-to or wrapped signature.
4. `@ffi(std_function)`: the type of the value is `unsafe fn(A…) -> R` unless the entry carries `safe`; the existing `unsafe` callable machinery ([FN-6], ODR-115, [UNS-1]) then refuses calls outside `unsafe:` with E3100, whose text FFI-17a gives, adding the `safe` line only when step 3's predicate would pass for the entry (so the message's line compiles, [DIA-25], and `unsafe` is not suggested while a fix without it compiles, [DIA-9]).
5. `ember bind --init` ([CLI-5]): run the predicate while writing each entry; a hit writes `unsafe fn` with a `@safety("TODO")` line, so the starter compiles unchanged. `ember tcb` lists `safe` on a `std_function` entry beside `@ffi(safe)` typedefs ([TCB-1]).
6. Registry and tests: the new E5002 cause and the FFI-17a E3100 text in the code tables; a conformance test per spelling (hand `safe fn` with a struct field, an overlay entry, an `@ffi(safe)` typedef, a `std_function, safe` entry, a callback result, a C++ method on a `@layout(c)` class, a `CppVector[Buf]` argument), each failing without the predicate; a positive test that an `extern type` or `handle` field is not reached, and that the Vulkan example compiles as edited.
7. Nothing in code generation, the runtime, Annex B or the interface files changes: the check is per declaration from the header's types, which [FFI-30a] makes identical across packages, so a package compiled once decides the same in every program ([BLD-16]).

**departs_from_verdict**: None. Verdicts touched, each kept: 31 (`@ffi(no_callbacks)` untouched; a forgotten word refuses, never crashes); 60 (closes a route by which safe code broke memory safety; the call now happens only inside `unsafe:`); 73 and [DIA-25] (every message gives both lines, the declaration's and the call's, so its fix runs into no second error); 118, 119 (the wrapper's `unsafe:` block is ordinary unsafe code); 120, 125 (the refused declaration or entry becomes `@safety` above `unsafe fn`, C++ members included); 121 (an entry refused in another package's overlay fails every build that applies it, as a drifted one does, and the message gives the `override` line, [FFI-30c]); 130, 133 (that spelling, and no new contract word: `safe` is FFI-56's existing word, `unsafe` stays no word of `@ffi(…)`); 132 and 136 (a plain overlay line keeps its meaning 'safe to call' and is refused where it cannot be made safe, never made silently unsafe by a header field the reader does not see; a `std_function` entry's safety is written, never inferred from the header); 137 (results stay outside the limit, read from the callee's code where there is one; a C function has none); 165 (untouched). Delegated rulings: ODR-073 (its reason is the limit's reason; the slot sentence kept word for word), ODR-074 (an unsafe hand declaration keeps its C ABI parameters, as the message shows), ODR-075 and ODR-076 (`ref_to_ptr` and `null[P]()` stay safe; only handing the result to foreign code under an assertion of safety is refused), ODR-115 (`unsafe fn(…) -> R` is the type a `std_function` value takes), ODR-199 (an `unsafe fn(A…) -> R` is still 'an owned callable value'; the default changes from safe to unsafe to call, which Hardened_2's FFI-17a already implied by making the target 'a foreign function' under FFI-2's 'never derived').

**trade_offs**: Safety is kept whole: a pointer the caller supplied never reaches foreign code under an assertion of safety, by any route the call's arguments give it, and the same words decide every spelling, so there is no struct-shaped, element-shaped, object-shaped or callback-shaped way around [FFI-10]'s slot rule. Speed: nothing runs, and the wrapper is the same struct fill and one direct call C writes. Readability: the ordinary caller's line gets shorter (`clear(ref mut frame)`); the price falls on binding authors, who write one `unsafe fn` entry with `@safety` and one wrapper per function whose structs carry pointers (the price Rust binding authors pay), heaviest for Vulkan, where every create-info has `pNext`, and for hook tables and ops structs, whose function-pointer fields now count too; a hand binding also writes its raw form by hand, since an unsafe hand declaration is not mapped (ODR-074). `std::function` values are callable in safe code only when their entry says `safe`, one word, and C++ callables whose parameters carry pointers are callable only in `unsafe:`, never vouched; a `std::function` taking `std::span` cannot be vouched either (the `safe` list is FFI-56's plus the bridge types; a span is not one), a conservative choice that keeps the unsafe call. Given up: today's acceptance of such declarations and the safe-by-default `std_function` (both accepted programs that broke a promise, or derived a fact [FFI-2] says is never derived). Not taken: field contracts in overlays (a whole feature [FFI-30a] rules out), making a write of a pointer into a struct unsafe (the length beside it stays a plain number safe code changes, so the hole stays), and refusing `std_function` entries outright (drops a feature, against the owner's condition on the delegation).

**remaining_doubt**: 1. Exports: an `@export` function, or a function coerced to `extern "C" fn`, that returns a raw pointer it made in safe code (`ref_to_ptr(ref mut local)`) hands C a dangling address with no `unsafe` anywhere; [FFI-21a] covers only a pointer into a copied parameter (E5042) and [FFI-31b] lists no raw-pointer result among the refused ones. The limit here reaches a callback's result given to a foreign call, not an export C calls on its own; that is K08's question (export arguments and results from C) and should take the same property (a value safe code supplies that names memory nothing checks). 2. Handles: FFI-8 makes a `handle` an opaque `Copy` value, so a handle field is outside the limit; if a pointer-cast macro constant of a handle type (`((HANDLE)-1)`, [FFI-6a]) can be written in safe code, a forged handle still reaches C through a struct, which K46 should settle (a constant of a handle type is a value of it, or needs `unsafe`). 3. A union field (`@layout(c, union) struct`) holding a pointer under one alternative is reached whatever alternative the caller means, which is conservative and right for safety but refuses a safe entry for a tagged-union API; a wrapper is the way out. 4. An integer field C treats as an address (`uintptr_t`) stays the declarer's assertion, as [FFI-2]'s own `free_addr(uintptr_t)` example is for a parameter; the limit does not and cannot reach it. 5. `ember bind --check` and `--report` ([CLI-6], [CLI-7]) count declarations as safe or unsafe without the limit; the compile refuses the entry, so no program is accepted, but the tools' counts may name an entry safe that the build refuses; a one-line cite there is cheap if the lead wants it. 6. The `@safety` text for `vkCreateBuffer` names Vulkan's own validity rules in prose; a reader of the example may want it shorter.

## Design 3

**question**: K07: May a safe C or C++ call take a struct that holds raw pointers, and what limits raw pointers under the safe-to-call fact?

**summary**: The rule a newcomer holds in one sentence: safe code never hands C an address it made up, so a C or C++ call is safe to make only when nothing the call gives foreign code holds a raw pointer, alone, in a struct, in a span's elements, in a C++ object, or returned by an Ember function the call gives C; a raw pointer reaches foreign code only from an `unsafe:` block, under a `@safety` someone wrote. The limit is stated once, in `[FFI-2]`, where the safe-to-call fact is defined, and every spelling that asserts the fact shares it: `safe fn`, a plain overlay function entry, `@ffi(safe)` on a function-pointer typedef, and `safe` beside `std_function`; a `std::function` value is otherwise `unsafe fn(A…) -> R`, as a C function pointer already is under `[FFI-56]`, which removes the two-ways reading of `[FFI-17a]`. A refused declaration is `E5002` where it is written, naming the field that holds the pointer; the binding writes `@safety("…")` above `unsafe fn` and one `extend` wrapper that fills the struct from a checked `MutSpan`, so callers write a shorter safe line and nothing runs while the program runs.

- EDIT rule=FFI-2 (second sentence)
  - old: Supplying the facts, in an `@ffi(…)` attribute or an overlay (§XVI.4), makes the
  call safe.
  - new: Supplying the facts, in an `@ffi(…)` attribute or an overlay (§XVI.4), makes the
  call safe, within the limit on raw pointers below.
  - why: Read alone, the sentence promises that complete facts always make a call safe; the cite keeps it true for a struct whose field no contract reaches ([FFI-1]: a contract is 'for each pointer-typed parameter and result'; [FFI-30a]: an overlay 'never changes a type's identity, layout or fields').

- EDIT rule=FFI-2 (append: the limit on raw pointers)
  - old: 
  - new: Every spelling that asserts the fact — `safe fn`, an overlay function entry not written `unsafe fn`, and `safe` on a callable type's entry (`@ffi(safe)` on a function-pointer typedef, `[FFI-56]`; `@ffi(std_function, safe)` on a `std::function`'s, `[FFI-17a]`) — asserts it for calls that hand foreign code no raw pointer the caller supplied. A contract reaches a pointer-typed parameter or result and nothing else (`[FFI-1]`; an overlay never touches a type's fields, `[FFI-30a]`), and safe code may make a raw pointer that is null (`null[P]()`), that outlives what it points at or points at fewer elements than a count beside it says (`ref_to_ptr`, `as_mut_ptr()`, §IV.5, `[SPN-8]`), or that is a constant address (`[FFI-6a]`); so no assertion covers one. The raw pointers a call hands foreign code are read from the Ember types of what it passes: an argument that is one (`*T`, `*mut T`, `ObjectPtr[C]`); one held, at any depth, by a value an argument passes by value or behind a contracted pointer (`one`, `count(n)`, `owns`), in a field, an element, a union member or a type argument, a C++ method's object included; and one that an Ember function the call gives foreign code, as an argument or held by one, returns, itself or inside its result (a function value is not a raw pointer; what it returns is read). A declaration or entry in one of those spellings whose calls would hand one is `E5002` where it is written, naming where the pointer is held (`buf.data`, `pCreateInfo.pNext`, `get_buf`'s result) and the line that leaves the calls unsafe: `@safety("…")` above it written `unsafe fn` (a hand declaration with its C ABI parameters, `[FFI-10]`), or the type's entry without `safe`; each call then goes in an `unsafe:` block with a `# SAFETY:` note (`[UNS-1]`), or behind an `extend` wrapper that fills the value from a checked view (`[FFI-13]`); for an entry in another package's overlay the line is an `override` in the importing package's own overlay (`[FFI-30c]`). A raw pointer foreign code hands Ember (a result, an `out` value, a callback's parameter, a field of a value it returns) is outside this limit: reading or writing through it is unsafe code's already (`[UNS-1]`), and `[FFI-10]` decides a hand declaration's raw-pointer result. A `@ffi(handle)` value (`[FFI-8]`), a `CStrView`, a `Span[T]` or `MutSpan[T]`, and an opaque type, which declares no field, are not raw pointers. The limit is read from the declarations the compiling package sees (`[BLD-16]`), and nothing runs for it while the program runs.
  - why: This is where the safe-to-call fact is defined ('it is asserted by `safe fn` in an extern block (`[FFI-10]`), or by an overlay entry for the function, which asserts that a call within its contract cannot break memory safety'), and the fact has no limit: a raw pointer in a field is 'within its contract' because the contract never mentions it. ODR-073's reason (a safe caller may pass a null, dangling or forged pointer) is a property of every assertion of the fact, so the limit sits above all four spellings instead of in each list. Naming the spellings closes the method-4 team's blocking objection (E2-1): the consequence sentence reaches exactly the spellings the first sentence names, and each has a line that compiles (DIA-25).

- EDIT rule=FFI-10 (the `safe fn` list)
  - old: `@layout(c)` value type, or a pointer whose contract is complete (given with `@ffi(…)` on the
  declaration), `safe` on any other function being `E5002`, naming the fact that is missing; or
  - new: `@layout(c)` value type, or a pointer whose contract is complete (given with `@ffi(…)` on the
  declaration), and none of them carries a raw pointer to C (`[FFI-2]`'s limit), `safe` on any other function being `E5002`, naming the fact that is missing or where the raw pointer is held; or
  - why: The hole showed in this list: 'a `@layout(c)` value type' let `PixelBuf` through, and for it no fact is missing, so 'naming the fact that is missing' alone would describe the wrong message. A cite, not a second rule.

- EDIT rule=FFI-10 (the raw-pointer sentence)
  - old: complete facts do not make a
  raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`.
  - new: complete facts do not make a
  raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`, the argument's own case of `[FFI-2]`'s limit.
  - why: The existing sentence is the hand-declaration, top-level case of the same property; the cite says so, so a reader does not take it for the whole rule.

- EDIT rule=FFI-11 (other contract words)
  - old: and on a
  function-pointer typedef, `safe` (`[FFI-56]`).
  - new: and on a
  function-pointer typedef, or beside `std_function` on a `std::function` type's entry, `safe` (`[FFI-56]`, `[FFI-17a]`).
  - why: `safe` is already the contract word that asserts a foreign callable safe to call; the vocabulary row names its second place so no new word is added (verdict 133: `@ffi(…)` keeps its other words, `unsafe` is never one).

- EDIT rule=FFI-56
  - old: when every parameter and
  result is a scalar or a `@layout(c)` value type (`E5002` otherwise): an asserted fact, listed by
  `ember tcb`.
  - new: when every parameter and
  result is a scalar or a `@layout(c)` value type, within `[FFI-2]`'s limit on raw pointers (`E5002` otherwise): an asserted fact, listed by
  `ember tcb`. A `std::function` value is unsafe to call for the same reason, and `safe` beside `std_function` on its entry asserts it so under the same limit (`[FFI-17a]`).
  - why: FFI-56 grants a permission ('may assert … when every parameter … is … a `@layout(c)` value type'); without the cite it would contradict FFI-2's refusal of a typedef whose struct parameter holds `float* samples`. The second sentence makes FFI-56 the one rule for every callable that comes from foreign code: 'nothing tells the compiler what the function it points to does with its arguments' holds for a `std::function`'s target exactly as for a function pointer.

- EDIT rule=FFI-17a (the `std::function<R(A…)>` row)
  - old: an opaque object Ember owns; with `@ffi(std_function)` on its overlay entry, a function value of type `fn(A…) -> R` (`[CLO-3]`) which owns its target, a foreign function:
  - new: an opaque object Ember owns; with `@ffi(std_function)` on its overlay entry, a function value which owns its target, a foreign function, of type `unsafe fn(A…) -> R` (`[FN-6]`), since nothing tells the compiler what the target does with its arguments (`[FFI-56]`), or of type `fn(A…) -> R` (`[CLO-3]`) with `@ffi(std_function, safe)`, which asserts its calls safe within `[FFI-2]`'s limit on raw pointers (`E5002` otherwise, naming where the pointer is held and giving the entry without `safe`; an asserted fact `ember tcb` lists):
  - why: Hardened_2 made the value's target 'a foreign function', so a call through it is a foreign call, whose safe-to-call fact [FFI-2] says 'is never derived'; yet the row handed out the safe type `fn(A…) -> R` with no assertion a reader could see and no way to withhold it. Splitting the two meanings into the words the language has (`std_function`: a callable; `safe`: its calls are safe, [FFI-56]'s word) gives the limit a line that compiles and keeps ODR-199's owned callable value. The value stays one Ember function value for [EFF-2], [CLO-17], [FFI-52], [FFI-17e] and [FFI-17f].

- EDIT rule=§XVI.4 example (the Vulkan overlay above GRM-35)
  - old:     @ffi(effects=[FFI, Alloc], no_callbacks)
    fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed one,
                      pAllocator: nullable borrowed one, pBuffer: out one) -> status
  - new:     @safety("`pCreateInfo`'s `pNext` chain is null or valid and `pQueueFamilyIndices` points at `queueFamilyIndexCount` indices; `pAllocator`, when given, holds valid callbacks and user data")
    @ffi(effects=[FFI, Alloc], no_callbacks)
    unsafe fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed one,
                             pAllocator: nullable borrowed one, pBuffer: out one) -> status
  - why: The document's own example must compile under its rules: `VkBufferCreateInfo` holds `pNext` and `pQueueFamilyIndices`, and `VkAllocationCallbacks` holds `pUserData` and function pointers returning raw pointers, so the plain entry is `E5002`. Verdict 130's spelling. `vkCmdSetViewport` stays a safe entry (`VkViewport` holds floats only), so the example now shows both kinds. K46 (J8-14) edits `device: borrowed` on the same line; the two edits touch different words and merge into one rewrite.

- EDIT rule=§XVI.4 (one sentence under the example)
  - old:     hide vkAllocationFunction
```

* `[GRM-35]`
  - new:     hide vkAllocationFunction
```

`vkCreateBuffer` is an `unsafe fn` entry because the structs its call hands Vulkan hold raw pointers (`pNext`, `pQueueFamilyIndices`, `pUserData`; `[FFI-2]`'s limit); an `extend` wrapper in the overlay (`[FFI-13]`) gives safe callers a create call that fills those structs from checked views. `vkCmdSetViewport` is safe to call: a `VkViewport` holds only numbers.

* `[GRM-35]`
  - why: Part XVI opens with 'after which calls are safe'; a reader who sees the section's headline call turn unsafe is told why in one sentence, and where the safe call comes from. One sentence, no second example.

- EDIT rule=§XVII codes table, row E5002 (the cause)
  - old: `safe fn` on a function whose contract is incomplete, or two destructors for one foreign type
  - new: `safe fn` on a function whose contract is incomplete; a `safe fn`, an overlay function entry, an `@ffi(safe)` typedef or an `@ffi(std_function, safe)` type whose calls would hand foreign code a raw pointer from the caller's arguments, held at any depth or returned by an Ember function the call gives it (`[FFI-2]`'s limit); or two destructors for one foreign type
  - why: The registry lists every cause of a code, so a compiler written from the registry agrees with the rule; the reach is stated so the row does not refuse `[FFI-6b]`'s `PAIR_FIRST(p: *Pair)`, an unsafe macro entry FFI-2 does not reach.

- EDIT rule=§XVII codes table, row E5002 (the rule column)
  - old: | `[FFI-2]`, `[FFI-10]`, `[FFI-36a]`, `[FFI-21]` |
  - new: | `[FFI-2]`, `[FFI-10]`, `[FFI-36a]`, `[FFI-21]`, `[FFI-56]`, `[FFI-17a]` |
  - why: FFI-56's own `E5002` was missing from the column, and FFI-17a now gives the code too.

- EDIT rule=CLI-5
  - old: A declaration it leaves
  unsafe to call is written `unsafe fn` (verdict 130).
  - new: A declaration it leaves
  unsafe to call, one whose calls `[FFI-2]`'s limit on raw pointers keeps from being safe among them, is written `unsafe fn` (verdict 130).
  - why: Without it `ember bind --init pixels.h` would write `fn clear_pixels(buf)`, which FFI-2 refuses, and CLI-5's 'It compiles unchanged' would be false for every header whose structs hold pointers.

- EDIT rule=TCB-4
  - old: the C FFI section lists each `safe fn`, `@ffi(no_callbacks)`
  (`[FFI-52]`), `@ffi(safe)` (`[FFI-56]`) and thread contract, and the C++ bridge section each
  `@ffi(const)` (`[FFI-40a]`).
  - new: the C FFI section lists each `safe fn`, `@ffi(no_callbacks)`
  (`[FFI-52]`), `@ffi(safe)` (`[FFI-56]`) and thread contract, and the C++ bridge section each
  `@ffi(const)` (`[FFI-40a]`) and `@ffi(std_function, safe)` (`[FFI-17a]`).
  - why: Every asserted fact appears in the report ([TCB-3]); the new spelling is one.

**refused_program**: ```ember
# pixels.em — C: void clear_pixels(struct PixelBuf buf) writes buf.len zero bytes at buf.data
@layout(c)
struct PixelBuf:
    data: *mut u8
    len: usize

unsafe extern "C":
    @ffi(no_callbacks)
    safe fn clear_pixels(buf: PixelBuf)

fn main():
    x: u8 = 7
    clear_pixels(PixelBuf(data=ref_to_ptr(ref mut x), len=4096))
    println(x)
```

Today every line is safe code and C writes 4,096 bytes over one. Under the ruling the program is refused at line 9:

`E5002`: "`clear_pixels` is declared `safe fn`, but `buf.data` is a raw pointer (`*mut u8`), which safe code may fill with any address, and `buf.len` with any count, and no contract reaches a field; write `@safety("…")` above `unsafe fn clear_pixels(buf: PixelBuf)` saying what `buf.data` must point at, and put each call in an `unsafe:` block with a `# SAFETY:` note, or give safe callers an `extend` wrapper that fills `PixelBuf` from a `MutSpan[u8]` inside `unsafe:`".

The same code and shape of message refuse: an overlay entry `fn cp_load(opts: borrowed one)` whose header struct holds `const char* path` (naming `opts.path`, giving `@safety("…")` above `unsafe fn cp_load(opts: borrowed one)`, or an `override` of it in the importing package's overlay); `@ffi(safe) type mix_fn` whose struct parameter holds `float* samples` (giving the entry without `@ffi(safe)`); `@ffi(std_function, safe) type Fill` for `std::function<void(Buf)>` with `uint8_t* data` in `Buf` (giving `@ffi(std_function) type Fill`, whose value is `unsafe fn(Buf)`); and `safe fn stream_read(want: usize, get_buf: fn(usize) -> *mut u8)` with `callback(borrowed)`, naming `get_buf`'s result.

**accepted_program**: ```ember
# pixels.em — C: void clear_pixels(struct PixelBuf buf); void clear_bytes(uint8_t* data, size_t len)
@layout(c)
struct PixelBuf:
    data: *mut u8
    len: usize

unsafe extern "C":
    @safety("`buf.data` points at `buf.len` bytes C may write during the call")
    @ffi(no_callbacks)
    unsafe fn clear_pixels(buf: PixelBuf)

    # the same work with the pointer and count as parameters: a complete contract, safe as today ([FFI-10], ODR-074)
    @ffi(param(data, borrowed, count(len), exclusive), no_callbacks)
    safe fn clear_bytes(ref mut data: MutSpan[u8], len: usize)

fn clear(ref mut bytes: MutSpan[u8]):
    # SAFETY: the struct describes exactly the bytes of `bytes`, which this call views `ref mut`, so C writes only them
    unsafe:
        clear_pixels(PixelBuf(data=bytes.as_mut_ptr(), len=bytes.len()))

fn main():
    frame: Array[u8] = [7, 7, 7, 7]
    clear(ref mut frame)          # the struct is filled from a checked view; prints [0, 0, 0, 0]
    frame = [7, 7, 7, 7]
    clear_bytes(ref mut frame)    # `len` is the witness the compiler supplies; prints [0, 0, 0, 0]
    println(frame)
```

Safe callers can only clear bytes they own; `clear(ref mut frame)` is shorter than building the struct by hand. A C++ `std::function<void(int)>` mapped `@ffi(std_function, safe) type OnTick` stays a plain `fn(int)` value called from safe code, since an `int` is not a raw pointer; one mapped `@ffi(std_function)` alone is an `unsafe fn(…)` value, called inside `unsafe:` as a C function pointer is today.

**runtime_cost**: Nothing. The limit is decided when compiling, from the declaration's types. The wrapper fills the struct from the span's pointer and length, as C code fills it, and makes the one direct call that §X.4's row "FFI call | guaranteed present | one direct call for an ABI-direct C function" already lists ([FFI-9]: "An ABI-direct call is one `call` instruction and nothing else"); after inlining the instructions are those of the C program. A `std::function` value's call and drop cost what [FFI-17f] already reports, whether the entry says `safe` or not. No row of §X.4 changes.

**implementer_steps**: 1. In the FFI front end, after contract resolution and the mapping to Ember types ([FFI-10], [FFI-11], [FFI-8]), compute for each foreign callable declaration whether it asserts the safe-to-call fact: a hand `safe fn`; an overlay function entry (in or outside a class section) not written `unsafe fn`; a typedef entry carrying `@ffi(safe)`; a type entry carrying `@ffi(std_function, safe)`. Parse `safe` as a second word of `@ffi(std_function, …)` (the `@ffi(…)` word list already takes several words; `unsafe` stays `E0104`).
2. For an asserting declaration, walk the Ember-side type of everything the call hands foreign code: each parameter's mapped type (`x: T`, `ref mut x: T`, `Span[T]`/`MutSpan[T]`, `Option[…]`, `owns T`, `[T; N]`), a C++ method's class for `this`, and for a `callback(…)` parameter or an `extern "C" fn`-typed slot the callable type's result (never its parameters, which C supplies). The walk descends through `@layout(c)` struct and union fields, fixed-array and span elements, `Option` and tuple components, type arguments of generic and bridge types (`CppVector[Slice]`), and callable results, with a visited set over nominal types; it stops at scalars, `@ffi(handle)` types, `CStrView`, `extern type`s and opaque C++ classes (no declared fields), and bridge strings. Field types come from the `.embind`'s recorded declarations ([FFI-14]) or the extern block, so the package compiling the line sees everything it reads ([BLD-16]); no whole-program fact enters.
3. On reaching `*T`, `*mut T` or `ObjectPtr[C]`, record the access path (`buf.data`, `pCreateInfo.pNext`, `items[].pNext`, `get_buf()`'s result) and report `E5002` at the declaration or entry, with the fix line chosen by spelling: `@safety("…")` above the declaration written `unsafe fn` with its C ABI parameters (hand declaration; ODR-074's raw form), the same for an overlay entry, the typedef entry without `@ffi(safe)`, or the type entry without `safe`; for an entry applied from another package's overlay, the `override` line in the importing package's overlay ([FFI-30c]). One message per declaration, listing every path ([DIA-26]).
4. Type the `@ffi(std_function)` value `unsafe fn(A…) -> R` unless the entry says `safe`; the existing `unsafe fn(…)` callable machinery ([FN-6], [UNS-1]: `E3100` outside unsafe code) does the rest. `ember tcb` lists `@ffi(std_function, safe)` in the C++ bridge section.
5. `ember bind --init` runs the same walk and writes such declarations `unsafe fn` ([CLI-5]); `--check` and `--report` count them among the declarations still unsafe ([CLI-6], [CLI-7]). `ember fmt --migrate` rewrites nothing for this: the calls then need `unsafe:` (every 0.9.20 change is a refusal, N75).
6. Separate compilation: the check runs where the declaration is written and where an overlay is applied (as [FFI-55]'s drift check does); the interface file already carries each foreign function's safety, so dependants see the result. Hot reload: a reload image recompiles the package's declarations the same way ([BLD-16]), and a header change refuses the reload anyway ([HR-23]).
7. Tests: the refused program (E5002 at the `safe fn` line naming `buf.data`), the overlay, typedef, `std_function` and callback-result cases, the accepted wrapper program printing `[0, 0, 0, 0]`, the XVI.4 example compiling, and a break test with the walk disabled. Then the twin: time the wrapper `clear` against the direct C call with the same pointer and length under MSVC, clang and gcc; the expectation is identical instructions after inlining, reported with both code sizes.

**departs_from_verdict**: None. Verdict 60 ("anything that can break a safety promise is possible only inside an `unsafe` block") is what the ruling enforces; verdicts 120, 125, 130 and 133 give the spelling of every fix line (`@safety("…")` above `unsafe fn`; `unsafe` never a word of `@ffi(…)`; `safe` is an existing contract word, [FFI-11]); verdict 132's lesson is kept (a plain overlay line is refused where it is written, never quietly made unsafe by a header field its reader does not see); verdict 136 is kept (no line silently means unsafe); verdict 137 is untouched (results are outside the limit; C has no code to read); verdicts 31, 118, 119, 121 and 165 are kept as the method-4 impact report says. Delegated rulings: ODR-073's reason is applied to a pointer in a field, an element or a callback result and its slot rule is kept word for word; ODR-074 and ODR-075 are kept (`ref_to_ptr` stays safe: making a raw pointer is safe, handing one to C under an assertion of safety is not). ODR-199 (the lead's, "makes a `std::function` an owned callable value") keeps its substance, but the value's default type becomes `unsafe fn(A…) -> R`, with `safe` on the entry giving `fn(A…) -> R`: a change of a delegated ruling's shape, not of an owner verdict, and it is the method-4 team's decision 4, option (a), which that team left for the owner; the owner may instead keep `fn(A…) -> R` as the default, in which case the limit refuses such an entry with no callable line to write (their option c) unless a new spelling is added.

**trade_offs**: Safety: the only route by which safe code handed C a null, dangling, short or forged address under an assertion of safety is closed at compile time, for every carrier (field, element, union member, type argument, C++ object, callback or function-pointer result) and every spelling; C's view of memory comes from a span the compiler checked. Speed: nothing runs; the wrapper is the same single call C makes. Readability: the ordinary program's line gets shorter (`clear(ref mut frame)` instead of building a struct); the price falls on binding authors, who write `@safety` above `unsafe fn` and one `extend` wrapper per C function whose structs hold pointers, heaviest for Vulkan, where every create-info has `pNext` (the price Rust binding authors pay too); a hand binding also keeps its raw C ABI form for the unsafe declaration (ODR-074). A `std::function` entry whose parameters are plain data needs one word, `safe`, to stay callable from safe code; programs calling one today need that word or `unsafe:`. Consistency: one rule for raw pointers in both directions, judged by where the value flows (as [FFI-21a] judges pointers given back to C), one word `safe` for every foreign callable ([FFI-56] and [FFI-17a] agree), and FFI-11 and FFI-30a stay as written. Given up: option B (today: memory corruption from safe code); option C (make writing a pointer into a struct unsafe: breaks ODR-075, costs an `unsafe:` on every such line, and still leaves `buf.len = 4096` as safe code); option D (field contracts in overlays: a whole feature, barred by [FFI-30a] and verdict 132's reading). Over-reach accepted: an entry taking a `CppVector[Slice]` or a struct whose pointer C only reads is refused too, since a dangling read pointer is as unsafe as a write; the wrapper is the same.

**remaining_doubt**: (1) [FFI-8]'s table has no row for `void*`/`const void*`; `pNext`'s refusal assumes it maps to some raw pointer type, which any mapping of `void*` must be, but the row should be checked or added by whoever owns K-questions on FFI-8. (2) Whether the owner wants the `@ffi(std_function)` default flipped to `unsafe fn(A…) -> R` (method-4's decision 4); the alternative that keeps today's default has no compiling line for a refused entry without a new spelling such as `unsafe type`. (3) Reaching type arguments (`CppVector[Slice]`) is the general reading of "as its Ember type declares it"; it refuses entries where C++ only reads the vector's length, which is acceptable but should be stated in a test. (4) K46 (J8-14) must decide `@ffi(handle)` so that `VkDevice` and `VkBuffer` fields are handles, not `*mut VkDevice_T`; until then every Vulkan struct holding a handle is also refused by this limit, which is the correct reading of the current [FFI-8] ("A `handle` is an opaque pointer-sized `Copy` value") only once the handle entry exists. (5) A `@layout(c, union)` member: [FFI-8] says union fields "are read only in `unsafe`" but not whether they are written in safe code; the limit reaches them either way, which is the safe reading. (6) The exact E5002 wording is the lead's to word (verdict 157); the lines it gives compile under DIA-25 in every spelling.

## Judgement 1

**winner**: 2

**scores**: [
 {
  "design": 1,
  "safety": 7,
  "speed": 10,
  "readability": 8,
  "consistency": 8,
  "generality": 7,
  "holes": "(1) A struct holding a C function pointer passes its limit, which names only `*T`, `*mut T`: \"covers a raw pointer (`*T`, `*mut T`) that a call carries to foreign code\". Program: `// hooks.h: typedef void (*release_fn)(void*); typedef struct { release_fn release; } Hooks; void set_hooks(Hooks h); #define NO_RELEASE ((release_fn)-1)` then `unsafe extern \"C\": @ffi(no_callbacks) safe fn set_hooks(h: Hooks)` and `fn main(): set_hooks(Hooks(release=NO_RELEASE))`. [FFI-6a]: \"A null-pointer or pointer-cast macro (`((void*)0)`, `((HANDLE)-1)`) is a `const` of that pointer type holding the address as an integer\", so `NO_RELEASE` is a safe constant of type `unsafe extern \"C\" fn(\u2026)` ([FFI-8]: \"function pointer | `unsafe extern \"C\" fn(\u2026) -> R`\"); the entry is accepted and C later calls address -1 from safe code. The same for a nullable function-pointer field given `None`. (2) Its carrier list is a list of shapes (\"a field, a union's field, an element of a `[T; N]`, a `Span`, a `MutSpan`, an `Option` or a `CppVector`\"): `Volatile[*T]` ([FFI-8]: \"`volatile T*` | `Volatile[*T]`\") and a `@ffi(newtype)` of a pointer typedef (\"a nominal type with the typedef's representation\") are reached only by charity. (3) Its own sentence contradicts itself: it lists `CppVector` among carriers and then says \"a bridge type declare[s] none, `[FFI-8]`, `[FFI-17e]`\", and [FFI-17e] calls `CppVector[T]` a bridge type. (4) Its E5002 registry row drops the `[FFI-56]`-first ordering but is otherwise right; its FFI-11 `retained` row edit is the one thing the other two lack."
 },
 {
  "design": 2,
  "safety": 9,
  "speed": 10,
  "readability": 7,
  "consistency": 8,
  "generality": 9,
  "holes": "(1) The `retained` row of [FFI-11] still reads \"the call stays `unsafe`, with a raw pointer\" while its new limit says \"No assertion of the fact \u2014 \u2026 an overlay function entry not written `unsafe fn` (`[GRM-35]`) \u2026 covers \u2026 a raw pointer\" and [GRM-35] says a plain entry \"asserts the function safe to call\": a plain entry `fn keep(p: retained)` is refused by the limit and accepted by the row, two rules on one program (its 'why' claims the trigger spares it, but the limit's words do not). (2) A `@ffi(newtype)` of a pointer typedef is not reached by \"its fields, its elements and its type arguments\" (it has none); only the implementer step 1 says a newtype of a scalar is a miss, implying the pointer case is a hit, which the rule text never says. (3) [TCB-4] is not edited although [TCB-3] says \"Every assumption a guarantee relies on appears in the report\" and [TCB-4] lists the spellings by section; the E3100 registry row gets no `[FFI-17a]` though the row's own text is added to FFI-17a. (4) Its `safe` list for `std_function` (\"a scalar, a `@layout(c)` value type or a bridge type\") refuses `std::function<void(std::span<uint8_t>)>`, though [FFI-17a] maps `std::span<T>` to `MutSpan[T]`, a checked view; conservative, not unsafe. No accepted program found that breaks a promise."
 },
 {
  "design": 3,
  "safety": 8,
  "speed": 10,
  "readability": 8,
  "consistency": 8,
  "generality": 8,
  "holes": "(1) The same hook-table program as design 1's hole (1) is accepted: its limit reads \"a raw pointer (`*T`, `*mut T`, `ObjectPtr[C]`)\" and says \"a function value is not a raw pointer; what it returns is read\", so a struct field of type `unsafe extern \"C\" fn(\u2026)` filled from a pointer-cast constant ([FFI-6a]) or given `None` reaches C under a safe entry, and C calls it. (2) The `retained` row of [FFI-11] is left as \"the call stays `unsafe`, with a raw pointer\" against its own \"Every spelling that asserts the fact \u2014 `safe fn`, an overlay function entry not written `unsafe fn` \u2026\": the plain entry is refused by one rule and left unsafe by the other. (3) Its FFI-56 sentence \"A `std::function` value is unsafe to call for the same reason, and `safe` beside `std_function` on its entry asserts it so\" restates FFI-17a's rule in a second place (one rule should hold it; a cite is enough). (4) Its registry row names the spellings but not the C function pointer, consistent with its narrower limit. Otherwise sound; its doubt about `void*` having no row in [FFI-8] is real (`pNext` is `const void*`) and is K46's or a table question, not this one's."
 }
]

**grafts**: From design 1: the edit of [FFI-11]'s `retained` row (\"the call stays `unsafe`, with a raw pointer\" becomes \"the entry or declaration is written `unsafe fn`\"), which removes the last line in the text whose calls are unsafe without the word, as verdicts 130 and 136 require, and makes the row agree with the limit; and its [TCB-4] row for `@ffi(std_function, safe)`. From design 3: the clarification that `Span[T]`, `MutSpan[T]`, `CStrView` and a `handle` are not raw pointers (a reader's first question), the cite in [FFI-10]'s raw-pointer sentence making it the slot case of one property, a one-sentence cross-reference in [FFI-56] to [FFI-17a] (a cite, not a second rule), and the `vkCmdSetViewport` half of the sentence under the Vulkan example. Added by the judge: a `@ffi(newtype)` is read as the type it represents; a `TODO(…)` word is decided by [FFI-2]'s first sentence ([FFI-11c] unchanged); the E3100 registry row gains `[FFI-17a]`; the E3100 message for a `std_function` call gives the `safe` entry line where the limit allows it and the `unsafe:` block only otherwise ([DIA-9]); `std::function<…(std::span<T>)>` may be vouched, since the span is a checked view of this table. Design 2's property (fields, elements, type arguments; a callable's result only; a C function pointer counted with data pointers) is kept as the core.

- FINAL EDIT rule=[FFI-2] second sentence
  - old: Supplying the facts, in an `@ffi(…)` attribute or an overlay (§XVI.4), makes the
  call safe.
  - new: Supplying the facts, in an `@ffi(…)` attribute or an overlay (§XVI.4), makes the
  call safe, within the limit on raw pointers below.
  - why: Read alone the sentence promises that complete facts always make a call safe; the limit appended to this rule qualifies exactly this sentence, so it points there.

- FINAL EDIT rule=[FFI-2] (append: the limit on raw pointers, after the DIA-18 sentence)
  - old: stays unsafe to call. The diagnostic,
  `E5002`, names each missing fact and the overlay line that supplies it (`[DIA-18]`).
  - new: stays unsafe to call. The diagnostic,
  `E5002`, names each missing fact and the overlay line that supplies it (`[DIA-18]`). **The limit on
  raw pointers.** No assertion of the fact — `safe fn` (`[FFI-10]`), an overlay function entry not
  written `unsafe fn` (`[GRM-35]`), or `safe` on a function-pointer typedef's or a `std_function`
  type's entry (`[FFI-56]`, `[FFI-17a]`) — covers a value the caller supplies that names memory or
  code nothing checks: a raw pointer (`*T`, `*mut T`, an `ObjectPtr[C]`, `[FFI-23]`) or a C function
  pointer (`unsafe extern "C" fn(…)`, `[FFI-56]`) that a call carries to foreign code: as an argument
  itself; held, at any depth, by what an argument passes by value or behind a contracted pointer
  (`one`, `count(n)`, `fixed(N)`, `exclusive`, `owns`, `nullable`), read from the argument's Ember type
  as it is declared, through its fields, its elements, its type arguments (`Option`, `[T; N]`, `Span`,
  `MutSpan`, `CppVector`, `Volatile`) and the representation of a `@ffi(newtype)` (`[FFI-8]`), a C++
  method's object included; or returned, itself or inside its result, by an Ember function the call
  gives foreign code, as an argument or held by one (of a callable type only the result is read: its
  parameters are what foreign code hands the function). A type that declares no field — an `extern
  type`, an opaque class, a `@ffi(handle)` type (`[FFI-8]`), `CppString` — holds none; a `Span[T]`, a
  `MutSpan[T]`, a `CStrView` and a safe `extern "C" fn` made from an Ember function (`[FN-6]`) are
  checked views and code, not raw pointers. Safe code makes such a value null (`null[P]()`, `None`),
  one that outlives what it points at or counts fewer elements than a length beside it says
  (`ref_to_ptr`, `as_mut_ptr()`, §IV.5, `[SPN-8]`), or one holding any address (a pointer-cast
  constant, `[FFI-6a]`), and no contract word reaches it: words attach to parameters and results only
  (`[FFI-1]`, `[FFI-11]`), and no overlay touches a field (`[FFI-30a]`). So a declaration or entry in
  one of those spellings whose calls would carry one is `E5002` where it is written, naming the path
  to the value (`buf.data`, `pCreateInfo.pNext`, `hooks.release`, `get_buf`'s result) and the calls
  the program has: "`clear_pixels` is declared `safe fn`, but `buf.data` is a raw pointer (`*mut u8`)
  that its call hands C, and safe code can put any address in it; write `@safety("…")` above
  `unsafe fn clear_pixels(buf: PixelBuf)`, saying what `buf.data` must point to, and put each call of
  it (line 12) in an `unsafe:` block with a `# SAFETY:` note, or give safe callers a wrapper that fills
  `PixelBuf` from a `MutSpan[u8]` inside `unsafe:`". The lines that leave the calls unsafe are
  `@safety("…")` above the declaration or entry written `unsafe fn` (a hand declaration with its C ABI
  parameters, `[FFI-10]`, ODR-074), or the typedef's or type's entry without `safe`; each call outside
  unsafe code then goes in an `unsafe:` block with a `# SAFETY:` note (`[UNS-1]`), or behind a wrapper
  in an `extend` block (`[FFI-13]`) that fills the value from a checked view inside `unsafe:`; for an
  entry in another package's overlay, the message gives that entry as an `override` in the importing
  package's own overlay (`[FFI-30c]`). A message elsewhere that offers an assertion of the fact (`safe
  fn lib_step`, a `callback(…)` word, `@ffi(safe)`, `@ffi(std_function, safe)`) offers it only where
  this limit allows it (`[DIA-25]`, `[DIA-9]`). A contract with a `TODO(…)` word holds an unknown fact,
  and the first sentence of this rule decides its calls (`[FFI-11c]`). A value foreign code hands
  Ember (a result, an `out` value, a callback's or an export's parameter, a field of what it returns)
  is outside the limit: reading or writing through it is unsafe code (`[UNS-1]`), and `[FFI-10]`
  decides a hand declaration's raw-pointer result. The check reads the declaration, the entry and the
  header's types, which every package importing the header sees alike (`[FFI-30a]`, `[BLD-16]`), and
  nothing runs for it while the program runs (§X.4, "Contract and range-type annotations").
  - why: This is where the safe-to-call fact is defined and where every spelling that asserts it points, so one limit here reaches all four spellings at once and no list elsewhere can disagree. It states a property (a value the caller supplies that names memory or code nothing checks, and the direction it flows), not a shape: fields, elements, type arguments, a newtype's representation, a C++ object and a callable's result are reached alike, and types that declare no field are not. C function pointers are in the property for the same reason as data pointers: [FFI-6a] makes a pointer-cast constant of a function-pointer type a safe value, and a nullable field takes `None`, so a hook table filled by safe code is code C calls at an address nothing checks. ODR-073's reason ('a safe caller may pass a null, dangling or forged pointer') is about any assertion of safety, not about parameter slots. Naming the spellings closes the method-4 team's blocking objection (E2-1); the `TODO(…)` sentence keeps [FFI-11c] as written; the `out` sentence keeps every program that only reads what C returns.

- FINAL EDIT rule=[FFI-10] the `safe fn` list
  - old: declaration), `safe` on any other function being `E5002`, naming the fact that is missing; or
  - new: declaration), within `[FFI-2]`'s limit on raw pointers, `safe` on any other function being `E5002`, naming the fact that is missing or, under that limit, where the raw pointer is held; or
  - why: The hole showed in this list: 'a `@layout(c)` value type' let `PixelBuf` through, and for it no fact is missing, so 'naming the fact that is missing' alone described the wrong message. A cite, not a second rule.

- FINAL EDIT rule=[FFI-10] the raw-pointer sentence
  - old: raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`. Missing facts or a
  - new: raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`, the slot's own case of `[FFI-2]`'s limit, which reaches a raw pointer at any depth of what a parameter passes. Missing facts or a
  - why: The existing sentence is the hand-declaration, top-level case of the one property; the cite says so, so a reader does not take it for the whole rule.

- FINAL EDIT rule=[FFI-11] table, row ownership (the `retained` outcome)
  - old: (`[FFI-23]`, `[FFI-54]`): the call stays `unsafe`, with a raw pointer |
  - new: (`[FFI-23]`, `[FFI-54]`): the parameter stays a raw pointer, so the entry or declaration is written `unsafe fn` (`[FFI-2]`'s limit) |
  - why: The row was the last line in the text whose calls were unsafe without the word, against [GRM-35] ('a function entry, which asserts the function safe to call') and verdicts 130 and 136 (no line silently means unsafe); under the limit a plain entry whose call carries that raw pointer is refused where it is written, so the row must say the written form.

- FINAL EDIT rule=[FFI-11] other contract words
  - old: and on a
  function-pointer typedef, `safe` (`[FFI-56]`). `@ffi(effects=[…])` states effects.
  - new: and on a
  function-pointer typedef or beside `std_function` on a `std::function` type's entry, `safe`
  (`[FFI-56]`, `[FFI-17a]`), within `[FFI-2]`'s limit on raw pointers. `@ffi(effects=[…])` states effects.
  - why: The vocabulary names every contract word and where it may stand; `safe` now also stands on a `std_function` type's entry, and both uses share the one limit. No new word is added (verdict 133: `unsafe` stays no word of `@ffi(…)`).

- FINAL EDIT rule=[FFI-56]
  - old: result is a scalar or a `@layout(c)` value type (`E5002` otherwise): an asserted fact, listed by
  `ember tcb`.
  - new: result is a scalar or a `@layout(c)` value type, within `[FFI-2]`'s limit on raw pointers (`E5002`
  otherwise, naming where the pointer is held and giving the entry without `@ffi(safe)`): an asserted
  fact, listed by `ember tcb`. A `std::function`'s target is a foreign function too, and its value
  follows this rule (`[FFI-17a]`).
  - why: FFI-56 grants a permission ('may assert … when every parameter … is … a `@layout(c)` value type'); without the cite it would accept `@ffi(safe) type mix_fn` whose struct parameter holds `float* samples`, which [FFI-2] refuses, and two rules would decide one entry differently. The last sentence is a cite so that a reader of 'A C function pointer is unsafe to call' finds the `std::function` case; the rule itself is in [FFI-17a].

- FINAL EDIT rule=[FFI-17a] table, row `std::function<R(A…)>`
  - old: with `@ffi(std_function)` on its overlay entry, a function value of type `fn(A…) -> R` (`[CLO-3]`) which owns its target, a foreign function: for `[EFF-2]`
  - new: with `@ffi(std_function)` on its overlay entry, a function value which owns its target, a foreign function, of type `unsafe fn(A…) -> R` (`[FN-6]`), since nothing tells the compiler what the target does with its arguments, as for a C function pointer (`[FFI-56]`): a call of it outside unsafe code is `E3100`, "`fill` holds a `std::function` from `fill.hpp` whose target nothing describes; put line 6 in an `unsafe:` block with a `# SAFETY:` note saying why the call is safe", or, where `safe` may stand on the entry under `[FFI-2]`'s limit, "`fill` holds a `std::function` from `fill.hpp` whose target nothing vouches for; write `@ffi(std_function, safe)` above `type Fill` in `overlays/fill.em` if a call of it within its signature cannot break memory safety" (`[DIA-9]`); or of type `fn(A…) -> R` (`[CLO-3]`) with `@ffi(std_function, safe)` on the entry, which asserts the target safe to call, allowed when every parameter and the result is a scalar, a `@layout(c)` value type, a bridge type (`[FFI-17e]`) or a span of this table, within `[FFI-2]`'s limit on raw pointers (`E5002` otherwise, naming where the pointer is held and giving the entry without `safe`), an asserted fact `ember tcb` lists (`[TCB-1]`, `[TCB-4]`): for `[EFF-2]`
  - why: This was the one spelling that asserted the fact with no limit at all, and the contradiction the method-4 fix left: Hardened_2 made the value's target 'a foreign function', and [FFI-2] says the safe-to-call fact 'is never derived, because a scalar can be an address', yet the row handed out the safe type `fn(A…) -> R` with no written assertion and no way to withhold it, so a limit on raw pointers left a `std::function<void(Buf)>` with no line that compiles. The C function pointer's shape ([FFI-56]: unsafe to call until `safe` is written, `safe` refused for raw pointers) keeps the feature, gives every program a compiling line, and uses words the language has (`@ffi(…)` already takes several words: '`@ffi(no_callbacks)` beside `std_function`'). ODR-199's 'owned callable value' stands: an `unsafe fn(A…) -> R` is one. The two E3100 texts follow [DIA-9] (never `unsafe` while a fix without it compiles).

- FINAL EDIT rule=[TCB-4]
  - old: and the C++ bridge section each
  `@ffi(const)` (`[FFI-40a]`).
  - new: and the C++ bridge section each
  `@ffi(const)` (`[FFI-40a]`) and `@ffi(std_function, safe)` (`[FFI-17a]`).
  - why: [TCB-3]: 'Every assumption a guarantee relies on appears in the report'; [TCB-4] lists the asserted spellings by section, and the new one needs its place.

- FINAL EDIT rule=[CLI-5]
  - old: A declaration it leaves
  unsafe to call is written `unsafe fn` (verdict 130).
  - new: A declaration it leaves
  unsafe to call, one that `[FFI-2]`'s limit on raw pointers keeps from being safe among them, is
  written `unsafe fn` (verdict 130).
  - why: Without it `ember bind --init pixels.h` would write `fn clear_pixels(buf)`, which [FFI-2] refuses, and CLI-5's 'It compiles unchanged' would be false for every header whose structs hold pointers.

- FINAL EDIT rule=§XVII.9 registry row E5002
  - old: | `E5002` | a foreign declaration whose contract cannot stand: a hand declaration that says neither `safe fn` nor `unsafe fn`, or both; `safe fn` on a function whose contract is incomplete, or two destructors for one foreign type (a call of a function without a verified contract outside `unsafe` is `E3100`); a function-pointer parameter of a `safe fn` with no `callback(…)` word; a `user_data` witness too narrow for the tagged number | `[FFI-2]`, `[FFI-10]`, `[FFI-36a]`, `[FFI-21]` |
  - new: | `E5002` | a foreign declaration whose contract cannot stand: a hand declaration that says neither `safe fn` nor `unsafe fn`, or both; `safe fn` on a function whose contract is incomplete; an assertion of the safe-to-call fact (`safe fn`, an overlay function entry not written `unsafe fn`, `@ffi(safe)` on a function-pointer typedef, `@ffi(std_function, safe)` on a `std::function` type) for a function whose calls would carry foreign code a raw pointer or C function pointer the caller supplies, as an argument, held at any depth by what an argument passes, or returned by an Ember function given to the call (`[FFI-2]`'s limit); or two destructors for one foreign type (a call of a function without a verified contract outside `unsafe` is `E3100`); a function-pointer parameter of a `safe fn` with no `callback(…)` word; a `user_data` witness too narrow for the tagged number | `[FFI-2]`, `[FFI-10]`, `[FFI-56]`, `[FFI-17a]`, `[FFI-36a]`, `[FFI-21]` |
  - why: One code, one defining rule per cause: the registry lists every cause with the rule's own reach and trigger ('an assertion … whose calls would carry'), so a compiler written from it agrees with [FFI-2] and does not refuse an `unsafe fn` entry taking a raw pointer, which the limit never reaches; [FFI-56]'s own `E5002` was missing from the column, and [FFI-17a] now gives the code too.

- FINAL EDIT rule=§XVII.9 registry row E3100
  - old: a call of a C++ method whose reference result has no lifetime word outside unsafe code; `ThreadShared` declared for an `extern type` without `unsafe` | `[UNS-1]`, `[UNS-12]`, `[ALC-2]`, `[ARN-8]`, `[FFI-51]`, `[FFI-26]`, `[FFI-54]`, `[FFI-56]`, `[UNS-5]`, `[FFI-40]`, `[FFI-8]` |
  - new: a call of a C++ method whose reference result has no lifetime word outside unsafe code; a call of a `std_function` value whose entry does not say `safe` outside unsafe code; `ThreadShared` declared for an `extern type` without `unsafe` | `[UNS-1]`, `[UNS-12]`, `[ALC-2]`, `[ARN-8]`, `[FFI-51]`, `[FFI-26]`, `[FFI-54]`, `[FFI-56]`, `[FFI-17a]`, `[UNS-5]`, `[FFI-40]`, `[FFI-8]` |
  - why: [FFI-17a] now gives `E3100` with its own text; the registry row lists every cause of the code and its rules.

- FINAL EDIT rule=§XVI.4 the Vulkan overlay example (above [GRM-35])
  - old:     @ffi(effects=[FFI, Alloc], no_callbacks)
    fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed one,
                      pAllocator: nullable borrowed one, pBuffer: out one) -> status
  - new:     @safety("`pCreateInfo.pNext` is null or a valid chain, `pCreateInfo.pQueueFamilyIndices` points at `queueFamilyIndexCount` indices, and `pAllocator`, when given, holds valid callbacks and user data")
    @ffi(effects=[FFI, Alloc], no_callbacks)
    unsafe fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed one,
                             pAllocator: nullable borrowed one, pBuffer: out one) -> status
  - why: The document's own example must compile under its rules: `VkBufferCreateInfo` holds `pNext` and `pQueueFamilyIndices`, and `VkAllocationCallbacks` holds `pUserData` and function pointers, so the plain entry is `E5002` under the limit; it takes verdict 130's spelling. K46 (J8-14) edits `device: borrowed` on the same lines; the two edits touch different words and are applied as one rewrite.

- FINAL EDIT rule=§XVI.4, after the example's closing fence (before [GRM-35])
  - old:     hide vkAllocationFunction
```

* `[GRM-35]`
  - new:     hide vkAllocationFunction
```

`vkCreateBuffer` is an `unsafe fn` entry because the structs its call hands Vulkan hold raw pointers
(`pNext`, `pQueueFamilyIndices`, `pUserData`; `[FFI-2]`'s limit), while `vkCmdSetViewport` is safe to
call: a `VkViewport` holds only numbers, and `pViewports` is a counted view. A wrapper in an `extend`
block of the overlay (`[FFI-13]`) gives safe callers a create call that fills those structs from
checked views inside `unsafe:`.

* `[GRM-35]`
  - why: Part XVI opens by promising to say 'after which calls are safe'; a reader who sees the headline Vulkan call turn unsafe learns here why, why its neighbour stays safe, and the tool the language already has for the safe surface ([FFI-13]: 'this is how an idiomatic API is layered over a raw one').

**final_cost_row**: No row of §X.4 changes. The limit is decided when the declaration, entry or typedef is compiled, from the header's types and the entry's words, and emits no instruction: it falls under the existing row \"| Contract and range-type annotations | not observable | compile time only |\". The call that replaces a refused safe call is the one the existing row already prices: \"| FFI call | guaranteed present | one direct call for an ABI-direct C function |\" ([FFI-9]: \"An ABI-direct call is one `call` instruction and nothing else\"); the wrapper fills the struct from the span's pointer and length, the stores C itself writes to build the struct, and makes that same single call, so after inlining it is the C program's instructions (a cross-package wrapper not inlined costs one call, as [FFI-29c] already prices a shim, removed by LTO under [BLD-FFI-4]). A `std_function` value's call and drop cost what [FFI-17f] already reports, whether its entry says `safe` or not; `unsafe fn(A…) -> R` against `fn(A…) -> R` changes no code. If the lead wants the limit named in the table, the contract row becomes: \"| Contract and range-type annotations | not observable | compile time only; `[FFI-2]`'s limit on raw pointers is read from declared types and emits nothing |\". To measure: `clear(ref mut frame)` against a hand-written C call with the same pointer and length, MSVC, clang and gcc, same flags; the expectation is identical time and code size.

**owner_verdicts_touched**: None overridden. Kept and applied: 60 (anything that can break a safety promise only inside `unsafe`: the route by which safe code handed C a null, dangling, short or forged address under an assertion of safety is closed; such a call now happens only in `unsafe:`); 120, 125 (every refused declaration or entry becomes `@safety(\"…\")` above `unsafe fn`, C++ members included); 130, 133 (that spelling everywhere, `unsafe` never a word of `@ffi(…)`; `safe` is [FFI-56]'s existing word, now also beside `std_function`); 132 and 136 (a plain overlay line keeps its written meaning 'safe to call' and is refused where it cannot hold, never quietly made unsafe by a header field its reader does not see; the `retained` row's last silent-unsafe line is written `unsafe fn`); 137 (results stay outside the limit: a C function has no code to read); 121 (an entry refused in another package's overlay fails every build that applies it, as a drifted entry does, and the message gives the `override` line, [FFI-30c]); 73 and [DIA-25]/[DIA-9] (every message gives a line that compiles, and offers `unsafe:` only where `safe` cannot stand); 31, 118, 119, 165 untouched. Delegated rulings: ODR-073 (its reason applied to a pointer in a field, an element, a type argument or a callback's result; the slot sentence kept), ODR-074 (an unsafe hand declaration keeps its C ABI parameters, as the message shows), ODR-075 and ODR-076 (`ref_to_ptr` and `null[P]()` stay safe; only handing the value to foreign code under an assertion of safety is refused), ODR-115 (`unsafe fn(…) -> R` is the type a `std_function` value takes), ODR-199 (the lead's: 'an owned callable value' stands, but its default type becomes `unsafe fn(A…) -> R` with `safe` on the entry giving `fn(A…) -> R`: a change of a delegated ruling's shape, not of an owner verdict, and the method-4 team's decision 4 option (a), which it left for the owner; reported as the one change that refuses programs 0.9.20_Hardened_2 accepts, a call of a `std_function` value outside `unsafe:` without `safe` on its entry, in the spirit of N75: every change of 0.9.20 is a refusal, `ember fmt --migrate` rewrites nothing).

**remaining_doubt**: (1) The dual route is not closed here and is the same class of hole, for K08's lead: an `@unsafe(export(…))` function ([FFI-26]; [FFI-31b] lists no refusal of a raw-pointer result) or an `override fn` of a C++ virtual ([FFI-39]) that returns `ref_to_ptr(ref mut local)` in safe code hands C a dangling address with no `unsafe` anywhere; [FFI-21a] covers only a pointer into a copied parameter (`E5042`). The same property (a value safe code supplies that names memory nothing checks) should decide it. (2) [FFI-8]'s table has no row for `void*`/`const void*`; `pNext`'s refusal assumes it maps to a raw pointer type, which any mapping must be; K46 or the table's owner should add the row. (3) K46 (J8-14) decides `@ffi(handle)`: until `VkDevice` and `VkBuffer` fields are handles rather than `*mut VkDevice_T`, every Vulkan struct holding a handle is also refused by this limit, which is the correct reading of today's [FFI-8]; and if a pointer-cast macro constant of a handle type (`((HANDLE)-1)`, [FFI-6a]) is a safe value, a forged handle still reaches C through a struct, which K46 should settle. (4) The `@safety` text for `vkCreateBuffer` is a proposal the lead may shorten; the two E3100 texts and the E5002 text are worded under verdict 157 as examples, and the lead words them finally. (5) `ember bind --check` and `--report` ([CLI-6], [CLI-7]) count declarations as safe or unsafe; the compile refuses the entry so no program is accepted, but a one-line cite there would keep the counts honest. (6) A union member holding a pointer is reached whatever alternative the caller means (conservative and right); an integer field C treats as an address (`uintptr_t`) is the declarer's assertion, as [FFI-2]'s own `free_addr(uintptr_t)` example is for a parameter, and the limit cannot reach it. (7) Hook tables and ops structs filled only with Ember functions are refused as safe entries too, since the limit reads the field's type (`unsafe extern \"C\" fn`), not who filled it; the wrapper is the way, and the price is stated in the trade-off rather than hidden. (8) [DIA-18] and the E5002 registry row disagree today on whether a call with unknown facts outside `unsafe` is `E5002` or `E3100`; pre-existing, not touched here, for the lead's consistency pass.

## Judgement 2

**winner**: 2

**scores**: [
 {
  "design": 1,
  "safety": 7,
  "speed": 10,
  "readability": 7,
  "consistency": 6,
  "generality": 8,
  "holes": "(a) No trigger qualifier: its words 'An assertion of the fact for a function whose call would carry one is E5002' reach a plain overlay entry with a TODO word, which [FFI-11c] keeps as written: \"the declaration stays uncontracted and its calls stay `unsafe`. A header can be adopted on the first day with everything behind `unsafe:`\". Program: `unsafe overlay c \"cp.h\": fn cp_load(opts: TODO(count))` is accepted by FFI-11c and E5002 by the new FFI-2, so two rules decide one line. It edits the `retained` row of [FFI-11] towards verdict 130 but leaves FFI-11c, so the pair is inconsistent. (b) It names only `*T` and `*mut T`, while [FFI-54] says \"An `ObjectPtr[C]` is a raw pointer for `[THR-8]`, `[THR-9]` and `[UNS-1]`\": a hand `@layout(c) struct Cfg` with a field `obj: ObjectPtr[Widget]` given to a `safe fn` passes its words. (c) The structural route through a C function pointer stays open: an imported struct's function-pointer field is `unsafe extern \"C\" fn(\u2026)` ([FFI-8] table, [FFI-56]), to which a pointer C handed back converts, so `safe fn set_hooks(h: Hooks)` may be given `Hooks(release=a_hooks.release)` read from another library's returned table, and C calls it on this library's objects; a direct function-pointer parameter of a `safe fn` takes no such value ([FFI-21]: \"accepts a capture-free Ember function (as an `extern \"C\" fn`) or an `extern \"C\" fn` value\"; [FN-6]: \"an `unsafe fn` coerces only to an `unsafe` callable type\"). (d) No message is quoted for the std_function E3100, and [UNS-1]'s text (\"whose callers must make true what its `@safety` says\") fits no value that has no `@safety`. (e) The registry row it writes drops [FFI-56]'s own E5002 from no cause but lists the rule; fine. Nothing accepted under it breaks memory safety through a raw data pointer; the open route is (c)."
 },
 {
  "design": 2,
  "safety": 9,
  "speed": 10,
  "readability": 6,
  "consistency": 7,
  "generality": 9,
  "holes": "(a) Its 'why' claims \"The trigger 'asserts the fact' keeps every entry whose own words already leave its calls unsafe (an unknown fact, an uncontracted raw-pointer slot, `retained` on a plain pointer, a `TODO(\u2026)` word) exactly as today\", but its rule text says \"a declaration or entry that asserts the fact and whose calls would carry one is `E5002`\": a plain entry with `retained` on a plain pointer asserts the fact ([GRM-35]: \"a function entry, which asserts the function safe to call\") and its call carries a raw pointer ([FFI-11]: \"the call stays `unsafe`, with a raw pointer\"), so the text refuses what the 'why' keeps. The method-4 qualifier (\"whose calls the other rules make safe\") is missing from the rule. (b) No [TCB-4] edit, so the new asserted fact `@ffi(std_function, safe)` has no section in the report that [TCB-3] promises (\"Every assumption a guarantee relies on appears in the report\"). (c) Its `safe` list for std_function (\"a scalar, a `@layout(c)` value type or a bridge type\") refuses a `std::function<void(std::span<float>)>` from being vouched, though [FFI-17a] maps the span to a checked `MutSpan[T]` and nothing in it is unchecked; a conservative refusal of correct programs the owner's condition on the delegation counts against. (d) It quotes an E3100 message in [FFI-17a] without adding [FFI-17a] to the E3100 row's rule column, as [FFI-56] is listed there for its variant. (e) The rule paragraph is the longest of the three and a newcomer cannot hold it in one sentence. Safety: no program found that breaks a promise; including C function pointers closes design 1's hole (c)."
 },
 {
  "design": 3,
  "safety": 7,
  "speed": 10,
  "readability": 8,
  "consistency": 6,
  "generality": 8,
  "holes": "(a) The same trigger gap as design 1: \"A declaration or entry in one of those spellings whose calls would hand one is `E5002`\" refuses a plain entry with `retained` on a plain pointer while the unedited [FFI-11] row says \"the call stays `unsafe`, with a raw pointer\", and refuses a `TODO(count)` entry [FFI-11c] accepts; neither row is edited or qualified, so the contradiction stands in the text. (b) The C function-pointer route of design 1's hole (c) stays open: \"a function value is not a raw pointer; what it returns is read\", so a `Hooks` struct filled from another library's returned table passes a safe entry. (c) Its registry 'why' cites \"`[FFI-6b]`'s `PAIR_FIRST(p: *Pair)`\", which the current text does not contain (grep finds no `PAIR_FIRST`); harmless in a 'why' but a false cite. (d) Its [FFI-17a] edit cites no E3100 message either (as design 1). Strengths: names `ObjectPtr[C]`, adds the [FFI-56] bridging sentence and the [TCB-4] row, and its one-sentence summary (\"safe code never hands C an address it made up\") is the most readable statement of the property."
 }
]

**grafts**: Winner: design 2 (the second JSON object, counting from 1), for safety and generality: its property, 'a value the caller supplies that names memory or code nothing checks', is the only one that also closes the structural route through a C function pointer in a field, which the current text already closes for a direct parameter ([FFI-21] with [FN-6]: a `safe fn`'s callback parameter has the safe type `fn(P…) -> R`, to which a C-origin `unsafe extern "C" fn` never converts). Grafted from design 1: the trigger qualifier the method-4 team had ('whose calls the other rules make safe'), restated so [FFI-11]'s `retained` row and [FFI-11c]'s TODO entries keep today's outcome and the text and the 'why' agree; the [DIA-25] sentence (a message elsewhere offers `safe fn`, a `callback(…)` word or `@ffi(safe)` only where the limit allows it); the FFI-10 raw-pointer-sentence cite; the fuller §XVI.4 paragraph that also says why `vkCmdSetViewport` stays safe; the registry row with the spellings and the reach spelled out; `[TCB-4]` listing the new fact. Grafted from design 3: `ObjectPtr[C]` named among raw pointers ([FFI-54]); the [FFI-56] sentence that makes it the one rule for every callable from foreign code; the one-sentence summary as the paragraph's opening clause; the note that a type declaring no field (an `extern type`, an opaque class, a `@ffi(handle)` type) holds none. Dropped from design 2: the extra `safe` list for `std_function` (a mapped `std::span` is a checked view; the limit alone refuses the raw pointers). Dropped from design 1: the edit of [FFI-11]'s `retained` row (with the trigger qualifier it is not needed for K07, and editing it without [FFI-11c] would leave a half-done verdict-130 alignment; listed under remaining doubt). Added by the judge: a cite in [FFI-21] for a `callback(…)` parameter whose `R` is a raw pointer, and `[FFI-17a]` in the E3100 row's rule column beside `[FFI-56]`, since the row now carries a std_function message.

- FINAL EDIT rule=[FFI-2] (second sentence)
  - old: call safe. The safe-to-call fact is never derived
  - new: call safe, within the limit on raw pointers below. The safe-to-call fact is never derived
  - why: Read alone, the sentence promises that complete facts always make a call safe; the limit appended to the rule qualifies exactly this sentence, so it points there (all three designs).

- FINAL EDIT rule=[FFI-2] (append: the limit on raw pointers)
  - old: 
  - new:  Safe code never hands foreign code an address it made up. No assertion of the fact — `safe fn` (`[FFI-10]`), an overlay function entry not written `unsafe fn` (`[GRM-35]`), `@ffi(safe)` on a function-pointer typedef (`[FFI-56]`) or `@ffi(std_function, safe)` on a `std::function` type (`[FFI-17a]`) — covers a value the caller supplies that names memory or code nothing checks: a raw pointer (`*T`, `*mut T`, an `ObjectPtr[C]`, `[FFI-54]`) or a C function pointer (`unsafe extern "C" fn(…)`, `[FFI-56]`), alone or inside an `Option`, that the call carries to foreign code: an argument itself; one held, at any depth, by what an argument passes, by value or behind a contracted pointer (`one`, `count(n)`, `fixed(N)`, `exclusive`, `owns`, `nullable`), a C++ method's object included, read from the argument's Ember type as it is declared, its fields, a union's fields, its elements and its type arguments (`[T; N]`, `Span[T]`, `MutSpan[T]`, `Option[T]`, `CppVector[T]`); and the result, itself or at any depth in it, of a safe callable type among them (a `callback(…)` parameter's `fn(P…) -> R`, `[FFI-21]`; a field of a safe `extern "C" fn` type, which only an Ember function fills, `[FFI-56]`), since that is what an Ember function given to the call returns to foreign code, its parameters being what foreign code hands the function. A type that declares no field holds none: an `extern type`, an opaque class, a `@ffi(handle)` type (`[FFI-8]`), `CppString`; a `Span`, a `MutSpan` and a `CStrView` are checked views, not raw pointers, and only a span's elements are read. Safe code can make such a value null (`null[P]()`, `None`), pointing at fewer elements than a count beside it says or at a value already ended (`ref_to_ptr`, `as_mut_ptr()`, §IV.5, `[SPN-8]`), or holding any address (a pointer-cast constant, `[FFI-6a]`; a function pointer C handed back, whose target nothing describes), and no contract reaches it: words attach to parameters and results only (`[FFI-1]`, `[FFI-11]`), no overlay touches a field (`[FFI-30a]`), and a `safe fn`'s own function-pointer parameter, a safe `fn(P…) -> R`, takes no such value (`[FFI-21]`, `[FN-6]`). So an assertion of the fact for a function whose calls the other rules make safe, and whose calls would carry one, is `E5002` where it is written, naming where the value is held: "`clear_pixels` is declared `safe fn`, but `buf.data` is a raw pointer (`*mut u8`, line 3) that its call hands C, and safe code can put any address in it and any count in `buf.len` (line 12); write `@safety("…")` above `unsafe fn clear_pixels(buf: PixelBuf)`, saying what `buf.data` must point to, and put line 12 in an `unsafe:` block with a `# SAFETY:` note, or give safe callers a wrapper that fills `PixelBuf` from a `MutSpan[u8]` inside `unsafe:`". The lines that leave the calls unsafe are `@safety("…")` above the declaration or entry written `unsafe fn` (a hand declaration keeping its C ABI parameters, `[FFI-10]`), or the typedef's or type's entry without `safe`; each call then goes in an `unsafe:` block with a `# SAFETY:` note (`[UNS-1]`), or behind a wrapper in an `extend` block (`[FFI-13]`) that fills the value from a checked view inside `unsafe:`; for an entry in another package's overlay, the message gives that entry as an `override` in the importing package's own overlay (`[FFI-30c]`). A message elsewhere that offers an assertion of the fact (`safe fn lib_step`, a `callback(…)` word, `@ffi(safe)`) offers it only where this limit allows it (`[DIA-25]`). An entry whose own words already leave its calls unsafe (an unknown fact, a `TODO(…)` word, `retained` on a plain pointer; `[FFI-11]`, `[FFI-11c]`) is as it was. A value foreign code hands Ember (a result, an `out` value, a callback's parameter, a field of what it returns) is outside this limit: reading or writing through it is unsafe code (`[UNS-1]`), and `[FFI-10]` decides a hand declaration's raw-pointer result. The limit is read from the declaration and the header's types, which every package importing the header sees alike (`[FFI-30a]`, `[BLD-16]`), and nothing runs for it while the program runs.
  - why: This is where the safe-to-call fact is defined and where every spelling that asserts it points, so one limit here reaches all four at once (the method-4 team's placement, kept by all three designs). The property is design 2's (a value the caller supplies that names memory or code nothing checks), with design 3's one-sentence opening and `ObjectPtr[C]`, design 1's DIA-25 sentence, and the method-4 trigger ('whose calls the other rules make safe'), which keeps [FFI-11]'s `retained` row and [FFI-11c] deciding their own entries, so the text no longer refuses what its 'why' keeps. C function pointers are in the property because the current text already closes that route for a direct parameter ([FFI-21], [FN-6]) and a field is the one structural-typed way round it; a field of a safe `extern "C" fn` type, which only an Ember function fills, is read for its result only, so a hand binding keeps a safe ops table. ODR-073's reason ('a safe caller may pass a null, dangling or forged pointer; the contract cannot establish validity') is a property of every assertion of safety, not of parameter slots.

- FINAL EDIT rule=[FFI-10] (the `safe fn` list)
  - old: `safe` on any other function being `E5002`, naming the fact that is missing;
  - new: and within `[FFI-2]`'s limit on raw pointers, `safe` on any other function being `E5002`, naming the fact that is missing or, under that limit, where the pointer is held;
  - why: The hole showed in this list: 'a `@layout(c)` value type' let `PixelBuf` through, and for it no fact is missing, so the E5002 clause must name the other message. A cite, not a second rule.

- FINAL EDIT rule=[FFI-10] (the raw-pointer sentence)
  - old: raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`.
  - new: raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`, the argument's own case of `[FFI-2]`'s limit.
  - why: The existing sentence is the hand-declaration, depth-zero case of the one property; the cite says so, so a reader does not take it for the whole rule (designs 1 and 3).

- FINAL EDIT rule=[FFI-11] (other contract words, the `safe` word)
  - old: function-pointer typedef, `safe` (`[FFI-56]`).
  - new: function-pointer typedef or beside `std_function` on a `std::function` type's entry, `safe` (`[FFI-56]`, `[FFI-17a]`), within `[FFI-2]`'s limit on raw pointers.
  - why: The vocabulary names every contract word and where it may stand; `safe` now also stands on a `std_function` type's entry, and both places share the one limit. No new word is added (verdict 133: `unsafe` stays no word of `@ffi(…)`).

- FINAL EDIT rule=[FFI-21] (the callable type of a `callback(…)` parameter)
  - old: among them staying raw (a lambda's body uses it in `unsafe:`);
  - new: among them staying raw (a lambda's body uses it in `unsafe:`), while a raw pointer as `R` is what the function returns to C, which `[FFI-2]`'s limit refuses on a `safe fn` or a plain overlay entry (`E5002`, naming the parameter's result);
  - why: The rule lets a raw pointer stay raw among the parameters (the C-to-Ember direction) without saying that the result is the other direction; without the cite a reader of [FFI-21] alone would accept `safe fn stream_read(want: usize, get_buf)` whose `get_buf` returns `*mut u8`, which [FFI-2] refuses.

- FINAL EDIT rule=[FFI-56]
  - old: result is a scalar or a `@layout(c)` value type (`E5002` otherwise): an asserted fact, listed by
  `ember tcb`. A function pointer made from an Ember function is a safe `extern "C" fn` (`[FN-6]`).
  - new: result is a scalar or a `@layout(c)` value type, within `[FFI-2]`'s limit on raw pointers (`E5002` otherwise, naming where the pointer is held): an asserted fact, listed by
  `ember tcb`; a `std::function` value is unsafe to call for the same reason, and `safe` beside `std_function` on its type's entry asserts it so under the same limit (`[FFI-17a]`). A function pointer made from an Ember function is a safe `extern "C" fn` (`[FN-6]`).
  - why: FFI-56 grants a permission with FFI-10's list; without the cite it would accept `@ffi(safe) type mix_fn` whose struct parameter holds `float* samples`, which [FFI-2] refuses, and two rules would decide one entry differently. The second sentence (design 3) makes FFI-56 the one rule for every callable that comes from foreign code: 'nothing tells the compiler what the function it points to does with its arguments' holds for a `std::function`'s target exactly as for a function pointer.

- FINAL EDIT rule=[FFI-17a] (table row `std::function<R(A…)>`)
  - old: a function value of type `fn(A…) -> R` (`[CLO-3]`) which owns its target, a foreign function: for `[EFF-2]` and `[CLO-17]`
  - new: a function value which owns its target, a foreign function: of type `unsafe fn(A…) -> R` (`[FN-6]`), since nothing tells the compiler what the target does with its arguments, as for a C function pointer (`[FFI-56]`; a call of it outside unsafe code is `E3100`: "`fill` holds a `std::function` from `fill.hpp` whose target nothing describes; put line 6 in an `unsafe:` block with a `# SAFETY:` note saying why the call is safe", and, only where `[FFI-2]`'s limit allows it (`[DIA-25]`), "or, if a call of it within its signature cannot break memory safety, write `@ffi(std_function, safe)` on `type Fill` in `overlays/fill.em`"), or of type `fn(A…) -> R` (`[CLO-3]`) with `@ffi(std_function, safe)` on the entry, which asserts the target safe to call within `[FFI-2]`'s limit on raw pointers (`E5002` otherwise, naming where the pointer is held and giving the entry without `safe`), an asserted fact `ember tcb` lists (`[TCB-1]`, `[TCB-4]`); for `[EFF-2]` and `[CLO-17]`
  - why: The one contradiction the method-4 fix left: 0.9.20_Hardened_2 made the value's target 'a foreign function', and [FFI-2] says the safe-to-call fact 'is never derived, because a scalar can be an address', yet the row handed out the safe type `fn(A…) -> R` with no assertion a reader could see and no way to withhold it, so a limit on raw pointers left a `std::function<void(Buf)>` with no line that compiles (E2-1). Giving it [FFI-56]'s shape (unsafe to call until `safe` is written; `safe` refused for raw pointers) keeps the feature, gives every program a compiling line, and uses words the language has (`safe` in `@ffi(…)`; `@ffi(no_callbacks)` already stands beside `std_function`). The message gives the `unsafe:` line first and the `safe` line conditionally, as [FFI-10]'s plain-fn message and [FFI-40]'s E3100 do. ODR-199 (delegated) keeps its substance: an `unsafe fn(A…) -> R` is still an owned callable value.

- FINAL EDIT rule=§XVI.4 the Vulkan overlay example (above [GRM-35])
  - old:     @ffi(effects=[FFI, Alloc], no_callbacks)
    fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed one,
                      pAllocator: nullable borrowed one, pBuffer: out one) -> status
  - new:     @safety("`pCreateInfo.pNext` is null or a valid chain, `pCreateInfo.pQueueFamilyIndices` points at `queueFamilyIndexCount` indices, and `pAllocator`, when given, holds valid callbacks and user data")
    @ffi(effects=[FFI, Alloc], no_callbacks)
    unsafe fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed one,
                             pAllocator: nullable borrowed one, pBuffer: out one) -> status
  - why: The document's own example must compile under its rules: `VkBufferCreateInfo` holds `pNext` and `pQueueFamilyIndices`, and `VkAllocationCallbacks` holds `pUserData` and function pointers, so the plain entry is E5002 under the limit; it takes verdict 130's spelling. K46 (J8-14) edits `device: borrowed` on the same lines; the two edits touch different words and are applied as one rewrite.

- FINAL EDIT rule=§XVI.4 (after the example's closing fence, before [GRM-35])
  - old:     hide vkAllocationFunction
```

* `[GRM-35]`
  - new:     hide vkAllocationFunction
```

`vkCreateBuffer` is an `unsafe fn` entry because the structs its call hands Vulkan hold raw pointers
(`pNext`, `pQueueFamilyIndices`, `pUserData`; `[FFI-2]`'s limit), while `vkCmdSetViewport` is safe to
call: a `VkViewport` holds only numbers, and `commandBuffer` is a `handle`, which declares no field
(`[FFI-8]`). A wrapper in an `extend` block of the overlay (`[FFI-13]`) gives safe callers a create call,
building those structs from checked views inside `unsafe:`.

* `[GRM-35]`
  - why: Part XVI opens by promising 'after which calls are safe'; a reader who sees the section's headline call turn unsafe learns here why, why its neighbour stays safe, and the tool the language already has ([FFI-13]: 'this is how an idiomatic API is layered over a raw one'). One paragraph, no second example.

- FINAL EDIT rule=[CLI-5]
  - old: unsafe to call is written `unsafe fn` (verdict 130).
  - new: unsafe to call, one whose calls `[FFI-2]`'s limit on raw pointers keeps from being safe among them, is written `unsafe fn` (verdict 130).
  - why: Without it `ember bind --init pixels.h` would write `fn clear_pixels(buf)`, which [FFI-2] refuses, and CLI-5's 'It compiles unchanged' would be false for every header whose structs hold pointers.

- FINAL EDIT rule=[TCB-4]
  - old: `@ffi(const)` (`[FFI-40a]`).
  - new: `@ffi(const)` (`[FFI-40a]`) and `@ffi(std_function, safe)` (`[FFI-17a]`).
  - why: Every asserted fact appears in the report ([TCB-3]: 'Every assumption a guarantee relies on appears in the report with its source and grade'); the new spelling is one and belongs to the C++ bridge section.

- FINAL EDIT rule=§XVII registry row E5002
  - old: | `E5002` | a foreign declaration whose contract cannot stand: a hand declaration that says neither `safe fn` nor `unsafe fn`, or both; `safe fn` on a function whose contract is incomplete, or two destructors for one foreign type (a call of a function without a verified contract outside `unsafe` is `E3100`); a function-pointer parameter of a `safe fn` with no `callback(…)` word; a `user_data` witness too narrow for the tagged number | `[FFI-2]`, `[FFI-10]`, `[FFI-36a]`, `[FFI-21]` |
  - new: | `E5002` | a foreign declaration whose contract cannot stand: a hand declaration that says neither `safe fn` nor `unsafe fn`, or both; `safe fn` on a function whose contract is incomplete; an assertion of the safe-to-call fact (`safe fn`, an overlay function entry not written `unsafe fn`, `@ffi(safe)` on a function-pointer typedef, `@ffi(std_function, safe)` on a `std::function` type) for a function whose calls the other rules make safe and would carry foreign code a raw pointer or C function pointer the caller supplies, as an argument, held at any depth by what an argument passes, or returned by an Ember function given to the call (`[FFI-2]`'s limit); or two destructors for one foreign type (a call of a function without a verified contract outside `unsafe` is `E3100`); a function-pointer parameter of a `safe fn` with no `callback(…)` word; a `user_data` witness too narrow for the tagged number | `[FFI-2]`, `[FFI-10]`, `[FFI-56]`, `[FFI-17a]`, `[FFI-36a]`, `[FFI-21]` |
  - why: One code, one defining rule per cause: the registry lists every cause with the rule's own reach and trigger, so a compiler written from the registry agrees with [FFI-2] and does not refuse an entry whose own words already leave its calls unsafe; [FFI-56]'s own E5002 was missing from the column, and [FFI-17a] now gives the code too.

- FINAL EDIT rule=§XVII registry row E3100 (the causes)
  - old: `ThreadShared` declared for an `extern type` without `unsafe` |
  - new: `ThreadShared` declared for an `extern type` without `unsafe`; a call of a `std::function` value whose type's entry does not say `safe` outside unsafe code |
  - why: The row names the message variants the code has ([FFI-56]'s C function pointer, [FFI-40]'s reference result); the std_function call is a new one with its own message in [FFI-17a].

- FINAL EDIT rule=§XVII registry row E3100 (the rule column)
  - old: `[FFI-40]`, `[FFI-8]` |
  - new: `[FFI-40]`, `[FFI-8]`, `[FFI-17a]` |
  - why: The row lists [FFI-56] for the C function pointer's message; the std_function message in [FFI-17a] is listed the same way.

**final_cost_row**: No row of §X.4 changes and none is added. The limit is decided when the declaration, overlay entry or typedef is compiled, from types the compiler already has, under the existing row "Contract and range-type annotations | not observable | compile time only"; the call that replaces a refused safe call is the one direct call of the existing row "FFI call | guaranteed present | one direct call for an ABI-direct C function" ([FFI-9]: "`extern \"C\"` is the platform C ABI"), and the `extend` wrapper fills the struct from the span's pointer and length, the stores C itself writes to build the struct, and inlines to the same instructions (a cross-package wrapper not inlined costs the one extra call [FFI-29c] already prices for a shim, removed by LTO). A `std_function` value's call keeps the thunk [FFI-17f] already reports; `unsafe fn(A…) -> R` against `fn(A…) -> R` changes no instruction. To measure: `clear(ref mut frame)` against a hand-written C call with the same pointer and length, both compilers, same flags; expected identical time and code size.

**owner_verdicts_touched**: None overridden. Verdict 60 (anything that can break a safety promise only inside `unsafe`): applied; the route is closed and the call happens only in `unsafe:`. Verdicts 120, 125, 130, 133, 136: every refused assertion becomes `@safety("…")` above `unsafe fn`, in a hand declaration, an overlay entry or a C++ member's entry; `unsafe` stays no word of `@ffi(…)`; `safe` is [FFI-11]'s existing word; no plain line silently means unsafe, since a plain entry that cannot be made safe is refused where it is written (the method-4 team's decision 3a), and an entry whose written words already leave its calls unsafe (`TODO(…)`, `retained` on a plain pointer) is kept as the current text has it. Verdict 132's lesson: a plain overlay line keeps its written meaning and is never quietly made unsafe by a header field its reader does not see. Verdict 137: results are outside the limit, untouched. Verdict 121: an entry refused in another package's overlay fails every build that applies it, with the `override` line ([FFI-30c]). Verdict 73 and [DIA-25]: every line a message gives compiles, and messages elsewhere offer an assertion only where the limit allows it; [DIA-9] is met by giving the `safe` line conditionally beside the `unsafe:` block, as [FFI-10] and [FFI-40] already do. Verdicts 31, 118, 119, 165: untouched. Delegated rulings: ODR-073 (its reason is the limit's reason; the slot sentence kept word for word), ODR-074 (an unsafe hand declaration keeps its C ABI parameters), ODR-075 and ODR-076 (`ref_to_ptr` and `null[P]()` stay safe; only handing the value to foreign code under an assertion of safety is refused), ODR-115 (`unsafe fn(…) -> R` is the type a `std_function` value takes), ODR-199 (kept in substance, changed in shape: the value's default type becomes `unsafe fn(A…) -> R`, with `safe` on the entry giving `fn(A…) -> R`; a delegated ruling, not an owner verdict, and the method-4 team's decision 4 option (a), which it left for the owner; reported as the one change that refuses programs 0.9.20_Hardened_2 accepts, a call of a `std_function` value outside `unsafe:` without `safe` on its entry, in the spirit of N75: every change of 0.9.20 is a refusal, and `ember fmt --migrate` rewrites nothing).

**remaining_doubt**: 1. The dual route is not K07's and stays open for K08's lead: an `@unsafe(export(…))` function ([FFI-26]; [FFI-31b] lists no raw-pointer result among the refused) or an `override fn` of a C++ virtual ([FFI-39]) may return `ref_to_ptr(ref mut local)` in safe code, and C writes through it; [FFI-21a] covers only a pointer into a copied parameter (E5042). The same property applies: a function foreign code calls whose result carries a raw pointer is written `unsafe fn` with `@safety`. 2. Including C function pointers in the limit refuses a plain entry taking an imported ops table even when the program fills it with Ember functions, since an imported field is typed `unsafe extern "C" fn` ([FFI-8]) and types cannot tell the origin; a hand-declared `@layout(c) struct` with safe `extern "C" fn` fields keeps a safe entry. Most such tables also hold a `void* user_data`, refused anyway. If the lead judges the structural route too exotic, dropping the clause "or a C function pointer (`unsafe extern "C" fn(…)`, `[FFI-56]`)" and the words "or C function pointer" in the registry row leaves designs 1 and 3's reach, with the route named here as a gap. 3. [FFI-11]'s `retained` row ("the call stays `unsafe`, with a raw pointer") and [FFI-11c] (a plain entry with `TODO(…)` whose "calls stay `unsafe`") are verdict-130 leftovers: a function unsafe to call written as a plain entry. The trigger qualifier keeps them as they are; aligning them (design 1's row edit, plus FFI-11c) is a question of its own, not K07's, since `ember bind --init` already writes such entries `unsafe fn` ([CLI-5]). 4. [FFI-8]'s table has no row for `void*`/`const void*`; the refusal of `pNext` assumes it maps to a raw pointer type, as any mapping of `void*` must. 5. `@ffi(handle)` values are outside the limit, so a pointer-cast macro constant of a handle type (`((HANDLE)-1)`, [FFI-6a]) still reaches C through a struct in safe code; K46 (J8-14) decides what a handle constant is. 6. [CLI-6] and [CLI-7] count declarations safe or unsafe without naming the limit; the compile refuses the entry, so no program is accepted, but a one-line cite there is cheap if the lead wants the counts to agree. 7. The `@safety` text for `vkCreateBuffer` and the quoted messages are the lead's to word (verdict 157); the lines they give compile under [DIA-25] in every spelling. 8. The E5002 message's line numbers (3, 12) refer to the refused program as design 2 lays it out (struct field on line 3, the call on line 12); the lead may renumber if the example in the text differs.
