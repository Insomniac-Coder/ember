# R7-part-XVI-a

could_not_cover: Verdicts 164 to 166 touch none of my eight clusters (no callback, handle, export or C++ import rule changes `Send`/`ThreadShared` of a `sharedref mut` link, where `thread_shared` is refused, or what a scoped task may end), so I checked them for contact only. I did not read the full round-2 design files for K07 and K21 beyond their edits, decisions and blockers (`design-K07-method4.md` lines 1-340, `design-K21-method3.md` lines 1-300); the K21 design's nine "holes left" past line 300 were skimmed from their headings and the first (override coercion), which my exact-fit limit addresses, not each in turn. K12's answer (the call word for a function a standard-library function keeps) is another agent's and K31's edit depends on agreeing with it; K47 (overlay names of `operator[]`) and K51 (`ref_to_ptr`'s spelling) likewise touch my examples but were not read.

## K07-raw-pointer-struct-safe-call  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=none

**answered_by**: FFI-10: "`safe fn` (a contextual keyword, `[LEX-15]`), safe to call only when every parameter and result is a scalar, a `@layout(c)` value type, or a pointer whose contract is complete (given with `@ffi(…)` on the declaration), `safe` on any other function being `E5002`, naming the fact that is missing"; FFI-10: "A raw `*T` can be declared for an unsafe call, but complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`."; FFI-56: "An overlay may assert one safe to call with `@ffi(safe)` on the typedef's entry (`@ffi(safe) type PFN_log_message`) when every parameter and result is a scalar or a `@layout(c)` value type (`E5002` otherwise)"; FFI-2: "Supplying the facts, in an `@ffi(…)` attribute or an overlay (§XVI.4), makes the call safe. The safe-to-call fact is never derived, because a scalar can be an address (`void free_addr(uintptr_t)`): it is asserted by `safe fn` in an extern block (`[FFI-10]`), or by an overlay entry for the function"; FFI-1: "Every foreign declaration has a **contract**: for each pointer-typed parameter and result, its ownership, mutability, nullability, count and lifetime"; FFI-30a: "An overlay changes the signatures, safety and names of *functions* and adds wrappers; it never changes a type's identity, layout or fields"; §IV.5: "Creating one from a reference is safe (ODR-075): `ref_to_ptr(ref x) -> *T` and `ref_to_ptr(ref mut x) -> *mut T` are one function the compiler knows"; FFI-8 (type mapping table): "complete `struct`, `union` | `@layout(c) struct` (`Copy` when every field is plain data)"

**program**: Still accepted by the current text, and C writes 4,095 bytes past `x` from safe code (a `[PHIL-10]` out-of-bounds write with no `unsafe` boundary and no false contract: every stated fact is true, the pointer field simply has none):

```
@layout(c)
struct Buf:
    data: *mut u8
    len: usize

unsafe extern "C":
    @ffi(no_callbacks)
    safe fn buf_zero(b: Buf)        # every parameter is "a `@layout(c)` value type": accepted by [FFI-10]'s list

fn main():
    x: u8 = 0
    buf_zero(Buf(data=ref_to_ptr(ref mut x), len=4096))   # safe: ref_to_ptr is safe (§IV.5), Buf is Copy
```

The same hole through the other two spellings of the fact: an overlay entry `fn cp_load(opts: borrowed one)` whose header struct holds `const char* path` (accepted: the entry's contract is complete; no word can reach `opts.path`, `[FFI-30a]`), and `@ffi(safe) type mix_fn` for `void (*)(struct Block)` where `Block` holds `float* samples` (`[FFI-56]`'s list accepts a `@layout(c)` value type). The document's own XVI.4 overlay asserts `vkCreateBuffer(... pCreateInfo: borrowed one ...)` safe while `VkBufferCreateInfo` holds `pNext` and `pQueueFamilyIndices` that safe code fills.

- READER EDIT rule=FFI-2
  - old: requires `unsafe`. Supplying the facts, in an `@ffi(…)` attribute or an overlay (§XVI.4), makes the
  call safe.
  - new: requires `unsafe`. Supplying the facts, in an `@ffi(…)` attribute or an overlay (§XVI.4), makes the
  call safe, within the limit on raw pointers at the end of this rule.
  - why: This sentence states the general claim the limit qualifies; without the pointer forward, the rule promises that complete facts always make a call safe.

- READER EDIT rule=FFI-2
  - old: 
  - new: No assertion of the safe-to-call fact — `safe fn` (`[FFI-10]`), an overlay function entry not written `unsafe fn`, or `@ffi(safe)` on a function-pointer typedef (`[FFI-56]`) — covers a raw pointer (`*T`, `*mut T`) that the call's arguments carry to foreign code: an argument itself, or a field or element, at any depth, of what an argument passes as its Ember type declares it, by value or behind a contracted pointer (`one`, `count(n)`, `owns`), or a raw pointer an Ember function given to the call returns. No contract word reaches a field (`[FFI-11]`, `[FFI-30a]`), and safe code makes such a pointer freely, beside a length it chooses (`ref_to_ptr`, `as_mut_ptr()`, `null[P]()`, §IV.5, `[SPN-8]`), so nothing could check it. A declaration or entry of those three kinds whose calls the other rules make safe, and whose calls would carry one, is `E5002` where it is written, naming where the pointer is held and the lines to write: "`buf_zero` is declared `safe fn`, and `Buf.data` is a raw pointer (`*mut u8`) that C may read or write through, with nothing stating how many bytes; write `@safety("…")` above `unsafe fn buf_zero(b: Buf)`, and give safe callers a wrapper that builds the `Buf` from a checked view inside `unsafe:` (`[FFI-13]`)"; for an overlay entry the line is `@safety("…")` above `unsafe fn` (`[GRM-35]`), for an entry of another package's overlay an `override` entry so written in the importing package's own overlay (`[FFI-30c]`), and for a typedef the entry without `@ffi(safe)`. A raw pointer foreign code hands Ember (a result, an `out` value, a callback's parameter) is outside this limit: it is read through only in unsafe code (`[UNS-1]`). Nothing runs while the program runs for this limit.
  - why: The cause is where the safe-to-call fact is defined: the three spellings that assert it share one hole, so the limit sits here once and the two lists cite it. Tying it to the three named spellings keeps `@ffi(std_function)` and the bridge types (`[FFI-17a]`) as they are (the earlier design was rejected for reaching them).

- READER EDIT rule=FFI-10
  - old: carrier. A raw `*T` can be declared for an unsafe call, but complete facts do not make a
  raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`.
  - new: carrier. A raw `*T` can be declared for an unsafe call, but complete facts do not make a
  raw-pointer parameter or result safe to expose, nor a raw pointer a parameter carries inside a
  `@layout(c)` value, by value or behind a contracted pointer (`[FFI-2]`'s limit); `safe fn` with one
  is `E5002`, naming the slot or the field.
  - why: The list of `[FFI-10]` is where the question showed: a `@layout(c)` value type was accepted whatever its fields hold; a cite, not a second rule.

- READER EDIT rule=FFI-56
  - old: when every parameter and
  result is a scalar or a `@layout(c)` value type (`E5002` otherwise)
  - new: when every parameter and
  result is a scalar or a `@layout(c)` value type that carries no raw pointer (`[FFI-2]`'s limit;
  `E5002` otherwise, naming the field)
  - why: The typedef's list is `[FFI-10]`'s list for C function pointers; without the cite the two would disagree on `@ffi(safe) type mix_fn`.

- READER EDIT rule=XVI.4 example (the Vulkan overlay above [GRM-35])
  - old:     @ffi(effects=[FFI, Alloc], no_callbacks)
    fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed one,
                      pAllocator: nullable borrowed one, pBuffer: out one) -> status

    @ffi(no_callbacks)
    fn vkCmdSetViewport(commandBuffer: borrowed, firstViewport, viewportCount,
                        pViewports: borrowed count(viewportCount))
  - new:     @safety("`pCreateInfo`'s `pNext` chain is null or valid and `pQueueFamilyIndices` points at `queueFamilyIndexCount` indices; `pAllocator`, when given, holds valid callbacks")
    @ffi(effects=[FFI, Alloc], no_callbacks)
    unsafe fn vkCreateBuffer(device, pCreateInfo: borrowed one,
                             pAllocator: nullable borrowed one, pBuffer: out one) -> status

    @ffi(no_callbacks)
    fn vkCmdSetViewport(commandBuffer, firstViewport, viewportCount,
                        pViewports: borrowed count(viewportCount))
  - why: The document's own example must compile under its rules: `VkBufferCreateInfo` and `VkAllocationCallbacks` hold raw pointers, so the safe entry is `E5002` under the limit; verdict 130's spelling. `device` and `commandBuffer` lose `borrowed` for K46 (a handle is passed by value); one rewrite for both clusters.

- READER EDIT rule=XVI.4 (a sentence under the example, before [GRM-35])
  - old:     rename VkPhysicalDeviceFeatures2 as PhysicalDeviceFeatures2
    hide vkAllocationFunction
```
  - new:     rename VkPhysicalDeviceFeatures2 as PhysicalDeviceFeatures2
    hide vkAllocationFunction
```

`vkCreateBuffer` is an `unsafe fn` entry because the structs its call hands Vulkan hold raw pointers (`pNext`, `pQueueFamilyIndices`, `[FFI-2]`); a wrapper in an `extend` block of the overlay (`[FFI-13]`) gives safe callers a create call that builds those structs from checked views. `device` and `commandBuffer` are handles, passed by value with no pointer words (`[FFI-8]`).
  - why: Part XVI opens with "after which calls are safe"; a reader who sees the headline Vulkan call turn unsafe is told here how a binding gives safe callers a create call, with the tool the language already has.

- READER EDIT rule=registry row E5002
  - old: `safe fn` on a function whose contract is incomplete, or two destructors for one foreign type (a call of a function without a verified contract outside `unsafe` is `E3100`)
  - new: `safe fn` on a function whose contract is incomplete; a `safe fn`, overlay function entry or `@ffi(safe)` typedef whose calls would otherwise be safe and would carry foreign code a raw pointer from the caller's arguments (one an argument is or holds at any depth, or one an Ember function given to the call returns); or two destructors for one foreign type (a call of a function without a verified contract outside `unsafe` is `E3100`)
  - why: The registry lists every cause of a code; `[FFI-56]` is added to the row's rule column (`[FFI-2]`, `[FFI-10]`, `[FFI-36a]`, `[FFI-21]`, `[FFI-56]`).

**remaining_doubt**: The edit refuses the document's showcase safe `vkCreateBuffer`; in Vulkan nearly every create call carries `pNext`, so Vulkan bindings get wrappers (as Rust's do). Two neighbours are left as they are and need their own ruling: a `@ffi(std_function)` callable (`[FFI-17a]`) whose parameters hold a raw pointer is still safe to call (the same hole in another spelling; not among my clusters), and field contracts (an overlay stating a field pointer's count) would be a feature `[FFI-30a]` rules out today. `std.ffi`'s `CppVector[S]` with `S` holding a pointer is reached by the words "as its Ember type declares it" only if the bridge type declares its elements; it does not, so it is left outside, which is the earlier design's E2-2 doubt.

**REFUTER reason**: The current text accepts the program. `[FFI-10]`: "`safe fn` (a contextual keyword, `[LEX-15]`), safe to call only when every parameter and result is a scalar, a `@layout(c)` value type, or a pointer whose contract is complete" — `Buf` is a `@layout(c)` value type, so `safe fn buf_zero(b: Buf)` passes the list; the one limit, "complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`", names a raw-pointer *parameter or result*, not a field. Filling the field is safe: §IV.5 "Creating one from a reference is safe (ODR-075): `ref_to_ptr(ref x) -> *T` and `ref_to_ptr(ref mut x) -> *mut T`", and `[UNS-1]`'s closed list of unsafe operations ("dereference, read or write through `*T`/`*mut T`; offset a pointer; cast ...") does not include storing a raw pointer in a struct. No word can reach the field: `[FFI-1]` counts a contract "for each pointer-typed parameter and result", and `[FFI-30a]` "never changes a type's identity, layout or fields". So C writes 4,095 bytes past `x` from a program with no `unsafe:` block. Against the lead's possible objection that `safe fn` is itself a false asserted fact (`[PHIL-11]`: "an inaccurate `asserted` foreign fact"): the text already refuses the identical property for a bare pointer slot (ODR-073's reason, "a safe caller may pass a null, dangling or forged pointer; the contract cannot establish validity"), because no declarer can truthfully assert it, so the same pointer one struct deeper is one property decided two ways; and the document's own XVI.4 overlay asserts `fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed one, …)` safe while `VkBufferCreateInfo` holds `pNext` and `pQueueFamilyIndices`, which shows authors read `safe` as not covering fields. `[FFI-56]`'s list ("when every parameter and result is a scalar or a `@layout(c)` value type") and a plain overlay entry (`[FFI-2]`: "asserted ... by an overlay entry for the function") have the same hole. The reader's edits fix the cause (the safe-to-call fact in `[FFI-2]`), tie the limit to the three spellings that assert the fact (which answers the earlier design's blocking objection E2-1: `@ffi(std_function)` and the `std.ffi` bridge types are no longer reached by the sentence), keep ODR-073/074/075 and verdicts 130, 133 and 136 (the lines given are `@safety("…")` above `unsafe fn`), add nothing that runs (no §X.4 row needed), and the Vulkan example is rewritten so it compiles under the rule. All `old` strings match the current text. What the edits leave open is listed under missed: `@ffi(std_function)` (the same hole in a fourth spelling) and a `@ffi(newtype)` over a pointer typedef (a pointer under another name that the limit's words `*T`, `*mut T` do not reach).

## K08-export-arguments-from-c  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=hole_in_proposed_text

**answered_by**: FFI-31b: "`@layout(c)` values, scalars, opaque handles, `CStrView` and `extern "C" fn` may."; FFI-31b: "`ember build --emit-header` writes above each prototype what the caller must make true, `[UNS-4]`'s words per parameter: a pointer valid for the call, a `T*` reached by no other argument, a `const char*` NUL-terminated, a count that fits its pointer; a C caller that breaks one is outside the contract, as a C library is."; FFI-31b: "that the count is right, the pointer is not null and the memory lives and is not read or changed by C for the call is C's side of the export's contract, an asserted fact `ember tcb` lists (`[TCB-1]`), and the header writes each with that contract as a comment"; FFI-31b: "an `Option` of a pointer-shaped type as a null pointer for `None`, `CStrView` as `const char*`"; FFI-56: "A function pointer that comes from C — a field of an imported struct, a typedef, a result, an `extern static`, an argument C passes to a callback — imports as `unsafe extern "C" fn(…)` (`Option[…]` when nullable, `[FN-6]`), so a call through it is in `unsafe:` (`E3100` …): nothing tells the compiler what the function it points to does with its arguments (`void free_addr(uintptr_t)`, `[FFI-2]`)."; FFI-26: "with a `# SAFETY:` note above saying why nothing else defines that name (`W3012` when missing)"; FN-6: "A function that captures nothing also coerces to `extern "C" fn(…)` when every type is FFI-safe"

**program**: Half answered. The `CStrView` half is decided, the other way than the finder recommended (option B, safe): `msg: CStrView` is the host's stated obligation ("the pointer is not null ... C's side of the export's contract, an asserted fact `ember tcb` lists", written in the header), and a host that may pass NULL is written `msg: Option[CStrView]` ("an `Option` of a pointer-shaped type as a null pointer for `None`"). The function-pointer half is open: `[FFI-31b]` admits `extern "C" fn` in an exported signature and says nothing about it; `[FFI-56]`'s open list ("a function pointer that comes from C — … an argument C passes to a callback") reaches an export's parameter by its reason but not by its words, and the header comment of `[FFI-31b]` has no words for it. Accepted today, and memory corruption from safe code:

```
# SAFETY: the host defines no other symbol named `game_fill`
@unsafe(export("game_fill"))
pub extern "C" fn game_fill(fill: extern "C" fn(*mut u8)) -> i32:
    x: u8 = 0
    fill(ref_to_ptr(ref mut x))     # a safe call ([FN-6]): the host's function writes 4096 bytes past `x`
    return x as i32
```

Nothing states what the host's function does with the pointer (there is no contract slot for it), which is exactly `[FFI-56]`'s reason for `unsafe extern "C" fn`.

- READER EDIT rule=FFI-31b
  - old: `@layout(c)` values, scalars, opaque handles,
  `CStrView` and `extern "C" fn` may.
  - new: `@layout(c)` values, scalars, opaque handles and
  `CStrView` may, and so may an `extern "C" fn` as a result or as a field of an `@export_table` (a
  function Ember made, `[FN-6]`); a function-pointer parameter of an export is a function pointer C
  supplies, so it is written `unsafe extern "C" fn(…)` and called inside `unsafe:` (`[FFI-56]`), the
  safe spelling being `E5015`: "`fill` is a function pointer the host supplies, and nothing says what
  the function does with its arguments (`[FFI-56]`); write `fill: unsafe extern "C" fn(*mut u8)` and
  call it inside `unsafe:` with a `# SAFETY:` note".
  - why: One rule with `[FFI-56]`: a value that comes from C gets the same type whichever way it arrives, and the export's author has no slot to state a contract for the host's function, so no assertion can make it safe.

- READER EDIT rule=FFI-56
  - old: comes from C — a field of an imported struct, a typedef, a result, an `extern static`, an argument
  C passes to a callback — imports as
  - new: comes from C — a field of an imported struct, a typedef, a result, an `extern static`, an argument
  C passes to a callback or to an export (`[FFI-31b]`) — imports as
  - why: The list names each way a function pointer arrives from C; an export's parameter was the one way missing.

- READER EDIT rule=registry row E5015
  - old: type may not cross the boundary; an exhaustive `@repr` enum in an export
  - new: type may not cross the boundary; an exhaustive `@repr` enum in an export; a safe `extern "C" fn` parameter of an export (write `unsafe extern "C" fn`)
  - why: The registry lists every cause of a code.

**remaining_doubt**: Whether the owner wants an export's author able to assert a scalar-only host function pointer safe (as `@ffi(safe)` lets a typedef be, `[FFI-56]`): the edit keeps one rule (always unsafe) since the `# SAFETY:` note of `@unsafe(export)` vouches only for the symbol name (`[FFI-26]`) and the export's signature has no attribute for such an assertion. A host passing NULL for a `CStrView` crashes, as a C library breaking a `count` contract does: the design's answer, `[PHIL-11]`'s "a bug in foreign code reached through a correctly declared boundary".

**REFUTER reason**: The `CStrView` half is answered as the reader says: `[FFI-31b]` "`ember build --emit-header` writes above each prototype what the caller must make true, `[UNS-4]`'s words per parameter: a pointer valid for the call, ... a `const char*` NUL-terminated ...; a C caller that breaks one is outside the contract, as a C library is", `[UNS-4]` "every view is non-null", and "an `Option` of a pointer-shaped type as a null pointer for `None`" gives `Option[CStrView]` for a host that may pass NULL. The function-pointer half is a real hole: `[FFI-31b]` admits "`extern "C" fn` may" in an exported signature; `[FN-6]` makes a call through a safe `extern "C" fn(*mut u8)` a safe call; `[FFI-56]`'s reason, "nothing tells the compiler what the function it points to does with its arguments", holds for the host's pointer but its list ("a field of an imported struct, a typedef, a result, an `extern static`, an argument C passes to a callback") names only imports, and `[FFI-26]`'s note vouches only for the name ("saying why nothing else defines that name"). The header has no `[UNS-4]` word for a function pointer, so an implementer cannot write the host's obligation. The reader's edit is right in direction but smaller than the hole: it covers only "a function-pointer parameter of an export", while the same host-supplied pointer arrives as a field of a `@layout(c)` value an export takes by value, behind `ref mut x: T`, or as a `Span[T]` element (`[FFI-31b]` lowers all three), and an Ember-declared `@layout(c)` struct with an `extern "C" fn` field is written by the export's author as safe, unlike an imported struct's field (`[FFI-56]`). The better edit states the property: a function pointer C hands an export, wherever it sits in the arguments, is `unsafe extern "C" fn`. No cost while running; no verdict touched.

- BETTER EDIT rule=FFI-31b
  - old: `@layout(c)` values, scalars, opaque handles,
  `CStrView` and `extern "C" fn` may.
  - new: `@layout(c)` values, scalars, opaque handles and
  `CStrView` may, and so may an `extern "C" fn` that Ember hands C: a result, or a field of an
  `@export_table` (`[FFI-26]`; a function Ember made, `[FN-6]`). A function pointer C hands an
  export — a parameter of it, or a field, at any depth, of a `@layout(c)` value a parameter passes,
  by value, behind `ref mut` or as a span's element — is one nothing describes (`[FFI-56]`), so its
  type is `unsafe extern "C" fn(…)`, called inside `unsafe:` with a `# SAFETY:` note; a safe
  `extern "C" fn` there is `E5015`: "`fill` is a function pointer the host supplies, and nothing
  says what the function does with its arguments (`[FFI-56]`); write `fill: unsafe extern "C"
  fn(*mut u8)` and call it inside `unsafe:` with a `# SAFETY:` note".
  - why: One property with `[FFI-56]`: a function pointer that comes from C is unsafe to call however it arrives; the reader's text left the struct-field and span-element routes, which `[FFI-31b]` itself lowers, as safe callables.

- BETTER EDIT rule=FFI-56
  - old: comes from C — a field of an imported struct, a typedef, a result, an `extern static`, an argument
  C passes to a callback — imports as
  - new: comes from C — a field of an imported struct, a typedef, a result, an `extern static`, an argument
  C passes to a callback or to an export, itself or in a `@layout(c)` value (`[FFI-31b]`) — imports as
  - why: The list names each way a function pointer arrives from C; an export's arguments were missing.

- BETTER EDIT rule=registry row E5015
  - old: type may not cross the boundary; an exhaustive `@repr` enum in an export
  - new: type may not cross the boundary; an exhaustive `@repr` enum in an export; a safe `extern "C" fn` C hands an export, as a parameter or in a `@layout(c)` value a parameter passes (write `unsafe extern "C" fn`)
  - why: The registry lists every cause of a code.

## K09-cpp-reference-result  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=none

**answered_by**: FFI-40: "a `T&` or `const T&` result is a pointer result (`[FFI-2a]`): its lifetime is never derived, and the overlay's entry gives it a lifetime word, `from(this)`, `from(p)` or `from(static)` (`[FFI-11]`), under which it is a `ref mut T` or `ref T` view into what the word names (`[LT-1a]`); without one the method stays unsafe to call, its result a `*mut T` or `*T`, listed by `ember bind --report` among the result-position pointers with no lifetime word (`[FFI-35a]`), and a call of it outside `unsafe:` is `E3100` ("`child.parent()` returns a C++ reference whose target no contract names; put line 7 in an `unsafe:` block with a `# SAFETY:` note, or give `parent` a lifetime word in the overlay: `fn parent() -> borrowed one from(this)`")"; FFI-2a: "In **result** position a pointer's lifetime is never derived."; FFI-35a: "every result-position pointer or C++ reference with no lifetime word"; FFI-17a: "`std::span<T>`, `std::span<const T>` | `MutSpan[T]`, `Span[T]`, zero-copy; a result needs a lifetime word (`[FFI-11]`)"; FFI-17a: "`std::string_view` | `Span[u8]`; `.to_str()` is the fallible conversion to `StrView` (`[TXT-2]`)"; FFI-17a: "`std::span<T, N>`, `std::span<const T, N>` (a fixed extent) | `ref mut [T; N]`, `ref [T; N]`"; FFI-11f: "A result lifetime derived from `[[clang::lifetimebound]]` imports as an asserted fact: the attribute is not enforced by C++, so it never counts as checked."

**program**: The cluster's question is answered, the way the checker's option B and the finder's recommendation said (Hardened_2 changed `[FFI-40]`: a C++ reference result needs a lifetime word, else the method is unsafe to call, `E3100`, with the entry line given). `n = s.pick(ref mut root)` and `r = u.larger(1, 2)` are therefore refused outside `unsafe:` until the entry says `-> from(start)` / `-> from(static)`. What remains undecidable is the finder's third line, a view type that is not a C++ reference:

```
// util.hpp
// class Scene { public: std::string_view name() const noexcept;
//                       std::span<const float, 3> origin() const noexcept; };
import cpp "util.hpp" with (project="engine") as u

fn main():
    s = u.Scene.new()
    nm = s.name()        # Span[u8] by [FFI-17a]'s row, which names no lifetime word: into `this` by [LT-1] rule 1, or a pointer result with none ([FFI-2a])?
    o = s.origin()       # ref [f32; 3]: the same question
    s = u.Scene.new()    # the first scene ends; `nm` and `o` point into freed memory if the rows derived a lifetime
    println(nm.to_str())
```

The `std::span<T>` row says "a result needs a lifetime word" and the two rows beside it say nothing, which an implementer may read as the others needing none; `[FFI-40]`'s sentence names only `T&` and `const T&`.

- READER EDIT rule=FFI-40
  - old: T` where the type is movable; a `T&` or `const T&` result is a pointer result (`[FFI-2a]`): its
  lifetime is never derived,
  - new: T` where the type is movable; a `T&` or `const T&` result, and a result whose Ember type is a view
  (`std::span`, `std::string_view`, a fixed-extent span, `[FFI-17a]`), of a method or of a function
  outside a class alike, is a pointer result (`[FFI-2a]`): its lifetime is never derived,
  - why: The property is "a foreign result that points somewhere needs a word"; stated once for every result type that is a view, and for free functions, which the sentence's place inside the methods rule left in doubt.

- READER EDIT rule=FFI-17a (the `std::string_view` and fixed-extent rows)
  - old: | `std::string_view` | `Span[u8]`; `.to_str()` is the fallible conversion to `StrView` (`[TXT-2]`) |
  | `std::span<T, N>`, `std::span<const T, N>` (a fixed extent) | `ref mut [T; N]`, `ref [T; N]` |
  - new: | `std::string_view` | `Span[u8]`; `.to_str()` is the fallible conversion to `StrView` (`[TXT-2]`); a result needs a lifetime word (`[FFI-40]`) |
  | `std::span<T, N>`, `std::span<const T, N>` (a fixed extent) | `ref mut [T; N]`, `ref [T; N]`; a result needs a lifetime word (`[FFI-40]`) |
  - why: The span row says it and the two rows beside it did not; the table now says one thing for every view it produces.

**remaining_doubt**: Whether an implementer would already read `[FFI-2a]`'s "pointer" as covering a `Span[u8]` result; the edit removes the doubt rather than relying on it. `[FFI-40]`'s message gives `-> borrowed one from(this)`, so the lifetime word for a view result is written the same way, with `count`/`nul_terminated` words where the C++ type gives no length (none needed here: the span and string_view carry theirs).

**REFUTER reason**: The reference case is answered by `[FFI-40]` as the reader says: "a `T&` or `const T&` result is a pointer result (`[FFI-2a]`): its lifetime is never derived, and the overlay's entry gives it a lifetime word ... without one the method stays unsafe to call, its result a `*mut T` or `*T`". The view case is open: `[FFI-17a]`'s rows give `std::string_view` → "`Span[u8]`" and the fixed extent → "`ref mut [T; N]`, `ref [T; N]`" with no lifetime sentence, while the `std::span` row says "a result needs a lifetime word (`[FFI-11]`)", so an implementer may read the two silent rows as needing none and fall back to `[LT-1]` rule 1 ("if it is a method, the result points into `this`"), which for a C++ body nobody reads is a derived lifetime that is wrong for a method returning a view into a parameter or a static buffer (`[FFI-11]`: "a result that holds only until the library's next call ... has no lifetime word"). `[FFI-2a]`'s "In **result** position a pointer's lifetime is never derived" says "pointer", and whether a `Span[u8]` is one is the question. Confirmed as an implementer question with an unsafe reading. The reader's fix sits in the methods rule and leaves two things undecided: what the result is when the entry has no word (for a reference `[FFI-40]` says `*T`; for a span nothing), and what the entry writes, since `[FFI-11a]` makes "A pointer contract with no count word ... `E5012`" and a span carries its own count. The cause is `[FFI-2a]`'s sentence, which defines the result-position rule for "a pointer" only; the better fix states the property there once, cites it from the rows, and settles both open points.

- BETTER EDIT rule=FFI-2a
  - old: In **result** position a pointer's lifetime is never derived.
  - new: In **result** position a lifetime is never derived: a pointer's, a C++ reference's (`[FFI-40]`),
  or that of a view a C++ type maps to (`std::span`, a fixed-extent span, `std::string_view`,
  `[FFI-17a]`), from a method or from a function outside a class alike. The entry gives it a
  lifetime word (`[FFI-11]`); without one the function stays unsafe to call (`E3100` outside
  `unsafe:`), its result a raw pointer (`*T`, `*mut T`) for a pointer or a reference and, for a view,
  the pointer with the count C++ returned, `(*T, usize)` or `(*mut T, usize)`, read inside `unsafe:`
  by an `extend` wrapper that copies the value (`[FFI-13]`); `ember bind --report` lists each
  (`[FFI-35a]`).
  - why: The property is "a foreign result that points somewhere needs a word"; stated where the result-position rule is defined, for every type that points, and with the raw form a wordless result takes, which the reader's edit left for the implementer.

- BETTER EDIT rule=FFI-17a (the `std::string_view` and fixed-extent rows)
  - old: | `std::string_view` | `Span[u8]`; `.to_str()` is the fallible conversion to `StrView` (`[TXT-2]`) |
  | `std::span<T, N>`, `std::span<const T, N>` (a fixed extent) | `ref mut [T; N]`, `ref [T; N]` |
  - new: | `std::string_view` | `Span[u8]`; `.to_str()` is the fallible conversion to `StrView` (`[TXT-2]`); a result needs a lifetime word (`[FFI-2a]`) |
  | `std::span<T, N>`, `std::span<const T, N>` (a fixed extent) | `ref mut [T; N]`, `ref [T; N]`; a result needs a lifetime word (`[FFI-2a]`) |
  - why: The span row says it and the two rows beside it did not; the table now says one thing for every view it produces, citing the rule that holds it.

- BETTER EDIT rule=FFI-11a
  - old: 
  - new: A result whose C++ type carries its count (a span, a fixed-extent span or `std::string_view`, `[FFI-17a]`) takes its lifetime word alone (`-> from(this)`), with no count word.
  - why: Without this the entry a view result needs (`[FFI-2a]`) is `E5012` for lacking a count the C++ type already gives; the line a message gives must compile (`[DIA-25]`).

## K15-foreign-const-globals  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=none

**answered_by**: FFI-41: "A static data member, and a variable of a namespace, imports as a foreign static (`[FFI-10]`), read and written as a global (`cpp.engine.Grid.made += 1`), in `unsafe` unless it is `const` and holds no pointer, each use going through a thunk that gives its address"; FFI-40a: "`const` is not an aliasing guarantee in C++: a `const` method of a class with a `mutable` member, its own or a base's, may change its object."; FFI-8 (type mapping table, static row): "`static const` global, `extern` global | `extern static`; reading it needs `unsafe` unless `@ffi(immutable)` asserts a `const` C object and an unchanged value (ODR-082). The generated C declaration is `extern const T` with that assertion, or `extern T` without it; a safe read also requires `T: needs ThreadShared`."; STA-1: "A foreign static read only in `unsafe` need not be `ThreadShared`: the programmer answers for sharing it between threads; one read without `unsafe` under `@ffi(immutable)` (`[FFI-10]`) must be (ODR-163)."; GRM-35: "overlay_item  := {attribute} (overlay_fn | "type" IDENT NEWLINE | "enum" IDENT NEWLINE | "rename" IDENT "as" IDENT NEWLINE | "hide" IDENT NEWLINE | extend_decl | overlay_class)"; FFI-2a: "Derived facts need no `unsafe`: `const T*` is a read-only view, `T*` one through which C may write"

**program**: Neither rule changed in 0.9.20. Accepted today, a data race from safe code (`[PHIL-10]`), and C and C++ decided two ways:

```
// limits.h (C):     extern const int MAX_PLAYERS;
// limits.hpp (C++): namespace game { extern const int MAX_PLAYERS;
//                   struct Stats { mutable long hits; long size() const; }; extern const Stats STATS; }
import c "limits.h" as lc
import cpp "limits.hpp" with (project="engine") as lp

fn main():
    println(lp.game.MAX_PLAYERS)   # safe by [FFI-41]: `const`, no pointer
    println(lc.MAX_PLAYERS)        # `unsafe` only: [FFI-8]'s row wants `@ffi(immutable)`, and [GRM-35] has no overlay item for a variable, so a header's global can never carry it
    println(lp.game.STATS.hits)    # safe by [FFI-41], while `STATS.size()` on a library thread writes `hits` legally ([FFI-40a]): a torn read, a data race in safe code
```

`Stats` is standard-layout and trivially copyable, so it imports as a `@layout(c) struct` with plain fields (`ThreadShared` inferred), and `[FFI-41]` asks neither `ThreadShared` nor the absence of `mutable`.

- READER EDIT rule=FFI-41
  - old: 1`), in `unsafe` unless it is `const` and holds no pointer, each use going through a thunk that
  gives its address,
  - new: 1`), in `unsafe` unless the header declares it `const` and its type is `ThreadShared` (`[STA-1]`),
  holds no pointer and has no `mutable` member, its own or a base's (`[FFI-40a]`: a `const` method may
  change such a member on any thread, so the value is not unchanged), each use going through a thunk
  that gives its address; a read outside `unsafe` of one that fails a condition is `E3100` ("`STATS`
  is a C++ `const` object whose type `Stats` has a `mutable` member (`hits`), which C++ may change
  while this reads it; put line 9 in an `unsafe:` block with a `# SAFETY:` note"),
  - why: The cause is the exemption's conditions: `const` alone is not "unchanged" in C++ (the rule two bullets up says so), and a safe read of a global is `[STA-1]`'s `ThreadShared` read; the same two tests the C path already has.

- READER EDIT rule=FFI-8 (type mapping table, static row)
  - old: reading it needs `unsafe` unless `@ffi(immutable)` asserts a `const` C object and an unchanged value (ODR-082).
  - new: reading it needs `unsafe` unless the header declares the object `const`, a derived fact (`[FFI-2a]`: a C `const` object is never changed), read as `[FFI-41]` reads a C++ one (its type `ThreadShared`, no pointer), or, on a hand declaration, which has no header to read, `@ffi(immutable)` asserts a `const` C object and an unchanged value (ODR-082).
  - why: One rule for C and C++: the fact a header shows is derived (`[FFI-2a]`), and the assertion stays for the declaration that has no header; it also closes the usability gap that a header's `const` global could never be read safely (no overlay item carries `@ffi(immutable)`).

- READER EDIT rule=STA-1
  - old: one read without `unsafe` under `@ffi(immutable)`
  (`[FFI-10]`) must be (ODR-163).
  - new: one read without `unsafe`, under `@ffi(immutable)`
  (`[FFI-10]`) or as a header's `const` object (`[FFI-8]`, `[FFI-41]`), must be (ODR-163).
  - why: The cross-reference claims what its rule says: the safe read now has two sources.

**remaining_doubt**: Deriving C's `const` for header imports changes ODR-082 (the lead's ruling of 0.9.9, not an owner verdict) from asserted to derived; a header that declares `extern const int X` over a non-const definition is a lie of the header, outside the contract (`[PHIL-11]`). The equally safe alternative is the finder's B: a `static IDENT` item in `[GRM-35]`'s overlay grammar so `@ffi(immutable)` can be written for a header's global, C and C++ alike, with `[FFI-41]`'s derived exemption removed; it keeps ODR-082 and adds a line to write. Safety does not choose between them; `[FFI-2a]`'s principle (what the declaration shows is derived) does.

**REFUTER reason**: Confirmed as a hole. `[FFI-41]`: "A static data member, and a variable of a namespace, imports as a foreign static (`[FFI-10]`), read and written as a global ..., in `unsafe` unless it is `const` and holds no pointer" asks neither `ThreadShared` nor the absence of `mutable`, while `[FFI-40a]` two bullets up says "`const` is not an aliasing guarantee in C++: a `const` method of a class with a `mutable` member ... may change its object". `Stats` is standard-layout and trivially copyable, so `[FFI-32]` imports it "as a `@layout(c) struct` with `@derive(Copy)`", and `println(lp.game.STATS.hits)` is a safe read that a C++ thread's legal `STATS.size()` writes under: a data race in a program with no `unsafe` boundary and no asserted fact, against `[PHIL-10]`. The C side is the inconsistency the reader names: `[FFI-8]`'s row "reading it needs `unsafe` unless `@ffi(immutable)` asserts a `const` C object" with no overlay item for a variable in `[GRM-35]` ("overlay_item := {attribute} (overlay_fn | "type" IDENT NEWLINE | "enum" IDENT NEWLINE | "rename" ... | "hide" ... | extend_decl | overlay_class)"), so one property is derived for C++ and unwritable for C. The reader's three edits are right in substance and keep ODR-161 and ODR-163 (the owner's: "a foreign static read only in `unsafe` need not be Sync; a C or C++ variable holding a pointer imports as any other"); ODR-082 is the lead's and, by the registry, was "for a hand-declared foreign static", so deriving a header's `const` does not reverse it. One defect: the edited `[FFI-8]` row cites "`[FFI-2a]`: a C `const` object is never changed", which `[FFI-2a]` does not say (its list is "`const T*` is a read-only view, `T*` one through which C may write; the layouts ..., packing and alignment, and the calling convention"), against the brief's "every cross-reference claims what its rule says". The better fix adds the derived fact to `[FFI-2a]`'s list and keeps the reader's edits. A reading outside `unsafe` under the reader's `E3100` also needs `[UNS-1]` to list the operation (missed item below).

- BETTER EDIT rule=FFI-2a
  - old: alignment, and the calling convention.
  - new: alignment, and the calling convention; and the `const` of an object a header declares (`[FFI-8]`,
  `[FFI-41]`): in C it is never changed, in C++ only through a `mutable` member, which is why an
  object whose type has one is read in `unsafe`.
  - why: The derived fact the two edited rules rely on is stated where derived facts are listed, so the cross-references claim what the rule says.

- BETTER EDIT rule=FFI-41
  - old: 1`), in `unsafe` unless it is `const` and holds no pointer, each use going through a thunk that
  gives its address,
  - new: 1`), in `unsafe` unless the header declares it `const` and its type is `ThreadShared` (`[STA-1]`),
  holds no pointer and has no `mutable` member, its own or a base's (`[FFI-40a]`: a `const` method may
  change such a member on any thread, so the value is not unchanged), each use going through a thunk
  that gives its address; a read outside `unsafe` of one that fails a condition is `E3100` ("`STATS`
  is a C++ `const` object whose type `Stats` has a `mutable` member (`hits`), which C++ may change
  while this reads it; put line 9 in an `unsafe:` block with a `# SAFETY:` note"),
  - why: The reader's edit, kept: the exemption's conditions are the cause.

- BETTER EDIT rule=FFI-8 (type mapping table, static row)
  - old: reading it needs `unsafe` unless `@ffi(immutable)` asserts a `const` C object and an unchanged value (ODR-082).
  - new: reading it needs `unsafe` unless the header declares the object `const`, a derived fact (`[FFI-2a]`), read as `[FFI-41]` reads a C++ one (its type `ThreadShared`, no pointer), or, on a hand declaration, which has no header to read, `@ffi(immutable)` asserts a `const` C object and an unchanged value (ODR-082).
  - why: The reader's edit with the cite made true by the `[FFI-2a]` edit: one rule for C and C++, the assertion kept for the declaration with no header.

- BETTER EDIT rule=STA-1
  - old: one read without `unsafe` under `@ffi(immutable)`
  (`[FFI-10]`) must be (ODR-163).
  - new: one read without `unsafe`, under `@ffi(immutable)`
  (`[FFI-10]`) or as a header's `const` object (`[FFI-8]`, `[FFI-41]`), must be (ODR-163).
  - why: The reader's edit, kept: the cross-reference claims what its rule says.

## K16-unsafe-import-cpp  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=none

**answered_by**: FFI-51: "An import that compiles C into the program — a C source file (`import c "./bridge.c"`, `[BLD-FFI-1]`), or a header with `implementation = [...]` (`[FFI-29b]`) — runs C code that nothing checks, a constructor in it even before `main`, so it is written `unsafe import c`, with a `# SAFETY:` note above it saying why that code is sound (`W3012` when missing)."; FFI-51: "An import of a header alone needs no word: its functions run only when they are called, and those calls are fenced by `[FFI-2]`. A library linked by name or path (`link = [...]`, `[link] libs`) is listed by `ember tcb` as an external library (`[TCB-4]`)."; FFI-3: "The importer generates `extern "C"` thunks, compiled by the project's own C++ compiler with the project's flags, and Ember calls the thunks."; FFI-17: "parses the header with libclang in the configuration of the named `[cpp.<project>]` section and generates one thunk per imported function, method, constructor and destructor"; §III.1 grammar: "import_decl   := … | ["pub"] ["unsafe"] "import" ("c" | "cpp") STRING [ffi_opts] ["as" IDENT] NEWLINE"; TCB-4: "The unsafe section lists each `unsafe:` block, `unsafe fn`, `unsafe import c` (`[FFI-51]`) and `@unsafe(export(…))` or `@unsafe(export_table(…))` (`[FFI-26]`) with its note; … and the C++ bridge section each `@ffi(const)` (`[FFI-40a]`)."; TCB-3: "Every assumption a guarantee relies on appears in the report with its source and grade."; TIER-1: "an `unsafe import c` of C source compiled into the program (`[FFI-51]`)"

**program**: Unchanged since Hardened_11, and the implementer cannot decide the import line: the grammar accepts both `import cpp` and `unsafe import cpp`, `[FFI-51]`'s words reach `import c` only, and its reason for exempting a header ("its functions run only when they are called") is false for a C++ header, whose thunk translation unit (`[FFI-3]`, `[FFI-17]`) compiles the header's definitions into the program:

```
// json.hpp (header-only): inline std::string g_default = load_default();   // a dynamic initialiser, run before main by the thunk unit
// #include <iostream>                                                      // its `ios_base::Init` object too
import cpp "json.hpp" with (project="engine") as js     # accepted with no word; `unsafe import cpp` also parses and no rule says what it means

fn main():
    println("start")
```

`load_default()` runs before `main`, unchecked, inside no `[TIER-1]` boundary, and `ember tcb` lists nothing for it (`[TCB-4]` lists `unsafe import c` and `@ffi(const)` only), against `[TCB-3]`.

- READER EDIT rule=FFI-51
  - old: An import that compiles C
  into the program — a C source file (`import c "./bridge.c"`, `[BLD-FFI-1]`), or a header with
  `implementation = [...]` (`[FFI-29b]`) — runs C code that nothing checks, a constructor in it even
  before `main`, so it is written `unsafe import c`, with a `# SAFETY:` note above it saying why that
  code is sound (`W3012` when missing).
  - new: An import that compiles foreign
  code into the program — a C source file (`import c "./bridge.c"`, `[BLD-FFI-1]`), a header with
  `implementation = [...]` (`[FFI-29b]`), or a C++ header, whose thunk translation unit compiles the
  header's definitions (`[FFI-3]`, `[FFI-17]`: an `inline` or namespace-scope variable's initialiser,
  or the standard library's, runs before `main`) — runs code that nothing checks, a constructor in
  it even before `main`, so it is written `unsafe import c` or `unsafe import cpp`, with a `# SAFETY:`
  note above it saying why that code is sound (`W3012` when missing).
  - why: The rule's own test ("compiles code into the program") decides a C++ import; stating it as the property, not the language, makes the exemption sentence true again: a C header alone is parsed, a C++ header is compiled.

- READER EDIT rule=FFI-51
  - old: Without `unsafe` it is `E3100`: "`import c "./bridge.c"`
  compiles `bridge.c` into the program, where its code runs unchecked, before `main` too; say that
  you vouch for it: write `unsafe import c "./bridge.c" as b` with a `# SAFETY:` note above it". An
  import of a header alone needs no word:
  - new: Without `unsafe` it is `E3100`: "`import c "./bridge.c"`
  compiles `bridge.c` into the program, where its code runs unchecked, before `main` too; say that
  you vouch for it: write `unsafe import c "./bridge.c" as b` with a `# SAFETY:` note above it"; for
  C++, "`import cpp "json.hpp"` compiles `json.hpp` into the program through its thunk unit, where
  its code runs unchecked, before `main` too; write `unsafe import cpp "json.hpp" with
  (project="engine") as js` with a `# SAFETY:` note above it". An import of a C header alone needs
  no word:
  - why: The message gives the line to write for the C++ case too.

- READER EDIT rule=TCB-4
  - old: `unsafe fn`, `unsafe import c` (`[FFI-51]`) and `@unsafe(export(…))` or `@unsafe(export_table(…))`
  (`[FFI-26]`) with its note;
  - new: `unsafe fn`, `unsafe import c` and `unsafe import cpp` (`[FFI-51]`) and `@unsafe(export(…))` or
  `@unsafe(export_table(…))` (`[FFI-26]`) with its note;
  - why: The report lists every boundary (`[TCB-3]`).

- READER EDIT rule=LEX-23
  - old: an `unsafe import c` of C
  source (`[FFI-51]`)
  - new: an `unsafe import c` or
  `unsafe import cpp` of code compiled into the program (`[FFI-51]`)
  - why: The safety note's places follow the boundaries.

- READER EDIT rule=TIER-1
  - old: an
  `unsafe import c` of C source compiled into the program (`[FFI-51]`);
  - new: an
  `unsafe import c` or `unsafe import cpp` of foreign code compiled into the program (`[FFI-51]`);
  - why: The list of boundaries is closed ("No other attribute ... introduces another"), so the new spelling must be in it.

**remaining_doubt**: Safety does not choose here: the pre-`main` code runs under every option, and a library linked by name already runs constructors with no word and is only listed in the report; the owner may prefer that shape (list each `import cpp` in `[TCB-4]`'s C++ bridge section, no `unsafe`), which is less to write but leaves `[FFI-51]`'s C rule and the C++ case decided two ways for one property. A per-header detection (the finder's B) would mark nearly every header that includes `<iostream>` anyway. Noted in passing (not this cluster): a C header defining a `static __attribute__((constructor))` function is compiled into the shim (`[FFI-29]`) and also runs before `main` with no word; the edited sentence keeps "a C header alone needs no word" and does not close that edge.

**REFUTER reason**: The implementer question is real: §III.1's grammar admits the word for both languages, "import_decl := ... | ["pub"] ["unsafe"] "import" ("c" | "cpp") STRING [ffi_opts] ["as" IDENT] NEWLINE", and no rule says what `unsafe import cpp` means; `[FFI-51]`'s words reach "`unsafe import c`" only, and its exemption's reason, "An import of a header alone needs no word: its functions run only when they are called", is false for a C++ header, whose definitions the thunk unit compiles (`[FFI-3]` "The importer generates `extern "C"` thunks, compiled by the project's own C++ compiler", `[FFI-17]` "generates one thunk per imported function"), so an `inline` variable's dynamic initialiser runs before `main` inside no `[TIER-1]` boundary and `[TCB-4]` lists nothing for it. Safety does not choose the fix (a library linked by name runs its constructors with no word today), so the brief's rule 7 does: a rule that states the property. The reader's edit states a language ("a C++ header, whose thunk translation unit compiles the header's definitions") as a proxy for the property (code that runs with no call from Ember), which (a) puts `unsafe` on every `import cpp`, a header of plain classes included, where the word describes nothing, against "a word on the thing it describes"; (b) keeps "a C header alone needs no word" although `[FFI-29]`'s shim compiles the header too ("an external wrapper for every `static inline` function") and a `__attribute__((constructor))` function in it runs before `main`, the edge the reader itself notes it does not close; and (c) its message cannot name what runs. The importer parses the header with libclang (`[FFI-6]`, `[FFI-17]`) and reads each namespace-scope, `inline` or static-member variable's initialiser, so the property is a fact the compiler reads (brief rule 2); `[FFI-38]` gives the rule for one it cannot decide ("rejects rather than guesses"). The better fix states that property for both languages, so the exemption sentence becomes true as written, the word appears exactly where unchecked code runs uncalled, and the message names the declaration. The reader's `[TCB-4]`, `[LEX-23]` and `[TIER-1]` edits stand.

- BETTER EDIT rule=FFI-51
  - old: An import that compiles C
  into the program — a C source file (`import c "./bridge.c"`, `[BLD-FFI-1]`), or a header with
  `implementation = [...]` (`[FFI-29b]`) — runs C code that nothing checks, a constructor in it even
  before `main`, so it is written `unsafe import c`, with a `# SAFETY:` note above it saying why that
  code is sound (`W3012` when missing).
  - new: An import that compiles into the program foreign code that runs with no call from Ember — a C
  source file (`import c "./bridge.c"`, `[BLD-FFI-1]`), a header with `implementation = [...]`
  (`[FFI-29b]`), or a C or C++ header that declares, in it or in a header it includes, a function
  with a constructor attribute or a variable at namespace scope, `inline` or a static member, whose
  initialiser is not a constant expression (the shim, `[FFI-29]`, or the thunk unit, `[FFI-3]`,
  `[FFI-17]`, compiles it, and it runs before `main`), as the importer reads the header, one it cannot
  decide counting as running (`[FFI-38]`) — runs code that nothing checks, so it is written `unsafe
  import c` or `unsafe import cpp`, with a `# SAFETY:` note above it saying why that code is sound
  (`W3012` when missing).
  - why: The property, not the language: the word goes exactly where unchecked code runs uncalled, for a C constructor function and a C++ dynamic initialiser alike, and a header of plain declarations keeps `import` with no word.

- BETTER EDIT rule=FFI-51
  - old: Without `unsafe` it is `E3100`: "`import c "./bridge.c"`
  compiles `bridge.c` into the program, where its code runs unchecked, before `main` too; say that
  you vouch for it: write `unsafe import c "./bridge.c" as b` with a `# SAFETY:` note above it". An
  import of a header alone needs no word:
  - new: Without `unsafe` it is `E3100`: "`import c "./bridge.c"`
  compiles `bridge.c` into the program, where its code runs unchecked, before `main` too; say that
  you vouch for it: write `unsafe import c "./bridge.c" as b` with a `# SAFETY:` note above it"; for
  a header the message names what runs: "`import cpp "json.hpp"` compiles the initialiser of
  `g_default` (json.hpp, line 12) into the program, where it runs unchecked before `main`; write
  `unsafe import cpp "json.hpp" with (project="engine") as js` with a `# SAFETY:` note above it". An
  import of a header that declares nothing that runs uncalled needs no word:
  - why: The message gives the line to write and the declaration that makes it necessary; the exemption sentence that follows ("its functions run only when they are called") is true again.

- BETTER EDIT rule=TCB-4
  - old: `unsafe fn`, `unsafe import c` (`[FFI-51]`) and `@unsafe(export(…))` or `@unsafe(export_table(…))`
  (`[FFI-26]`) with its note;
  - new: `unsafe fn`, `unsafe import c` and `unsafe import cpp` (`[FFI-51]`) and `@unsafe(export(…))` or
  `@unsafe(export_table(…))` (`[FFI-26]`) with its note;
  - why: The report lists every boundary (`[TCB-3]`).

- BETTER EDIT rule=LEX-23
  - old: an `unsafe import c` of C
  source (`[FFI-51]`)
  - new: an `unsafe import c` or
  `unsafe import cpp` of code compiled into the program (`[FFI-51]`)
  - why: The safety note's places follow the boundaries.

- BETTER EDIT rule=TIER-1
  - old: an
  `unsafe import c` of C source compiled into the program (`[FFI-51]`);
  - new: an
  `unsafe import c` or `unsafe import cpp` of foreign code compiled into the program that runs uncalled (`[FFI-51]`);
  - why: The list of boundaries is closed, so the new spelling must be in it.

## K21-cpp-overloads  reader=needs_owner  refuter=narrowed  should_be=needs_owner  safety=none

**answered_by**: TYP-26: "There is no overloading: two functions of one name in one scope are `E1030`, except operator-interface implementations and `extend` blocks for different types."; FFI-48: "| overloads differing only in return type | Ember has no return-type overloading | rename one in the overlay |"; FFI-39b: "`super.init(…)` selects the C++ base constructor by arity and argument types and runs it at `super.init(…)` (`[CLS-11]`); a base with no default constructor and no declared `init` is `E5058`."; FFI-17: "generates one thunk per imported function, method, constructor and destructor"; FFI-12: "An overlay entry whose name, parameter count or parameter names do not match the header is `E5010`, so overlays cannot drift."; GRM-35: "overlay_fn    := ["unsafe"] "fn" IDENT "(" [overlay_param {"," overlay_param} [","]] ")" ["->" contract {contract}] NEWLINE"; FFI-44: "| other classes | automatic, opaque behind a pointer; methods through thunks |"

**program**: No rule changed in 0.9.20 (`[FFI-48]` and `[FFI-39b]` keep their 0.9.10 text), so the implementer question stands: `[FFI-48]` refuses only overloads differing in return type, which implies the others import, `[FFI-17]` gives each a thunk, `[TYP-26]` makes two functions of one name `E1030`, and `[FFI-12]` matches entries by name, count and parameter names, so `[FFI-48]`'s own way out ("rename one in the overlay") cannot name one overload.

```
// renderer.hpp
// class Renderer { public: Renderer(); Renderer(int w, int h);
//   void draw(const Mesh& m); void draw(const Sprite& s); };
import cpp "renderer.hpp" with (project="engine") as eng

fn frame(ref mut r: eng.Renderer, mesh: eng.Mesh, hud: eng.Sprite):
    r.draw(mesh)     # one `draw`, two, E1030, E5034? the text does not say
    r.draw(hud)

fn main():
    r = eng.Renderer.new(1280, 720)   # which `new`?
```

Options, each safe (nothing runs; a wrong choice is a compile-time refusal): (A) one Ember name per C++ name; each use picks the one function its own words fit exactly (the arguments' number, names, types and `ref mut`/`owns` words, the object, a literal typed as C++ types it: `int`, `double`), as `super.init` already does; none or two fitting is an error naming each; an overlay entry is told apart by its parameter names and, where those coincide, by the Ember types it may write (`value: f32`), the form `@ffi(macro_fn)` entries already have. (B) each overload under a name built from its C++ types (`draw_Mesh`, `new_int_int`): names the program never wrote, broken by a header's rename. (C) overloads import only through an overlay that renames each; a header's constructors unusable until then. Recommendation: A, limited to exact fits (no widening or narrowing conversions, so an `override fn on(timer_id: i16)` cannot fit `on(int)`, the hole the earlier design left), since it keeps the C++ program's own lines, reuses `[FFI-39b]`'s selection and `[TYP-24]`'s ranking, and `[TYP-26]`'s rule only fails to describe names a header, not the program, declared.

- READER EDIT rule=TYP-26
  - old: There is no overloading: two functions of one name in one scope are
  `E1030`, except operator-interface implementations and `extend` blocks for different types.
  - new: There is no overloading: two functions of one name in one scope are
  `E1030`, except operator-interface implementations and `extend` blocks for different types; a C++
  name of several functions imports as one name (`[FFI-39b]`), since the header, not the program,
  declares them.
  - why: For option A: the exception says why the rule does not reach a header (refusing would refuse correct programs over code they cannot change).

- READER EDIT rule=FFI-39b
  - old: `super.init(…)` selects the C++ base constructor by arity and
  argument types and runs it at `super.init(…)` (`[CLS-11]`); a base with no default constructor and
  no declared `init` is `E5058`.
  - new: Each use of a C++ name of several functions (as `[FFI-30]` counts them) stands for the one it
  fits exactly: a call (`new`, `super.init(…)`, a method, a function of a namespace) by its object
  (`ref` through a name that only reads it or under `@ffi(const)`, else `ref mut`) and its
  arguments' number, names, types and words, an untyped literal counting as the type C++ gives it
  (`c_int`, `f64`); an `override fn` by its parameters' types; a function value by its expected
  callable type; an overlay entry by what it writes (`[GRM-35]`). No conversion but a literal's
  typing takes part, so a use fits one function or none, as C++ would pick it or refuse. What the
  code around the use permits (`unsafe`, effects, hand-overs, threads, views) is checked after, on
  the one chosen, so a line runs the same function wherever it stands. A use that none fits, or that
  two fit, is `E5070`, naming each C++ declaration ("`r.draw(x)`: `Renderer::draw(const Mesh&)` and
  `Renderer::draw(const Sprite&)` both take `x` as written; give `x` the type of one"); an entry so
  placed is `E5010`. `super.init(…)` runs the constructor it selects at `super.init(…)` (`[CLS-11]`);
  a base with no default constructor and no declared `init` is `E5058`.
  - why: For option A: the rule that already picks a constructor by its arguments becomes the one property for every use of an overloaded C++ name; exact fits only, so C++'s widening never changes which function an override or call means.

- READER EDIT rule=GRM-35
  - old: In an entry marked `@ffi(macro_fn)` (`[FFI-6b]`) a parameter's and the result's words are one Ember
  type each (`IDENT ":" type`, `"->" type`), since a macro has no C declaration to read (ODR-147).
  - new: In an entry marked `@ffi(macro_fn)` (`[FFI-6b]`) a parameter's and the result's words are one Ember
  type each (`IDENT ":" type`, `"->" type`), since a macro has no C declaration to read (ODR-147). Any
  entry may write a parameter's Ember type as the import maps it, before its contract words
  (`value: f32 borrowed one`), checked against the header; it is needed only where an entry would
  otherwise fit two of a C++ name's functions (`[FFI-39b]`), and `[FFI-30c]`, `[FFI-12]` and
  `[FFI-55]` count entries by function.
  - why: For option A: the entry needs a way to name one overload where parameter names coincide; Ember's own spelling of a parameter, already in the grammar for macro entries, is reused rather than a C++ signature in a string.

- READER EDIT rule=FFI-48 (the overloads row)
  - old: | overloads differing only in return type | Ember has no return-type overloading | rename one in the overlay |
  - new: | overloads differing only in return type | no call's arguments can tell them apart (`[FFI-39b]`) | a C++ wrapper with another name |
  - why: `rename` takes one name (`[GRM-35]`) and cannot name one overload; the way out must be one the grammar has.

**new_code**: `E5070` | a use of an imported C++ name of several functions that none fits exactly, or that two fit, naming each declaration | `[FFI-39b]`

**remaining_doubt**: A design choice no rule or verdict decides: B and C are as safe as A. The earlier design run on this cluster (round-2 inputs, `design-K21-method3.md`, unresolved) found nine holes in a version of A that admitted conversions; the exact-fit limit above is meant to close them, and an owner who prefers C++'s full ranking would reopen them. K47's `operator[]` const/non-const pair goes through `Index`/`IndexMut` (`[FFI-42]`), not this selection.

**REFUTER reason**: The gap is real and no verdict decides it: `[TYP-26]` "There is no overloading: two functions of one name in one scope are `E1030`"; `[FFI-17]` "generates one thunk per imported function, method, constructor and destructor"; `[FFI-48]` refuses only "overloads differing only in return type", which implies the others import; `[FFI-12]` matches an entry by "name, parameter count or parameter names", so `[FFI-48]`'s "rename one in the overlay" cannot name one overload (`[GRM-35]`: "rename" IDENT "as" IDENT). Options A, B and C are each safe (a wrong choice is a compile-time refusal), so the owner chooses: `needs_owner` is right. The reader's text for A, though, does not close the design run's holes it says it does. Its object-selection clause, "by its object (`ref` through a name that only reads it or under `@ffi(const)`, else `ref mut`)", decides a `const`/non-`const` pair (`const Entry& at(size_t) const` / `Entry& at(size_t)`) by whether the name only reads the object; for `this.cfg.at(i)` inside a method that is worked out from the body (`[FN-2a]`: "what the method does to it is worked out from its body (`[EXC-15]`)"), which includes this very call, so two implementations may pick differently (the design file's fourth hole). Such a pair also has identical parameter names and types, so the reader's `[GRM-35]` edit cannot write an entry that names one of them (the third hole). The better text keeps the reader's exact-fit limit (which does close the `override` hole: `override fn on(e: KeyEvent)` then fits no base method and is `E5056`, `[FFI-39]`), and decides the `const` pair by the nearest rule the text already has, `[FFI-42]` ("`operator[]` to `Index`/`IndexMut`"): the pair is one Ember method chosen by how the result is used, never by what the object's name permits, and one entry covers both. `[FFI-55]`'s record keys a fingerprint by name ("SDL_FreeSurface = "blake3:…""), so a name of several functions needs one line per function; added. Nothing runs; `E5070` is a new code (the reader's row).

- BETTER EDIT rule=TYP-26
  - old: There is no overloading: two functions of one name in one scope are
  `E1030`, except operator-interface implementations and `extend` blocks for different types.
  - new: There is no overloading: two functions of one name in one scope are
  `E1030`, except operator-interface implementations and `extend` blocks for different types; a C++
  name of several functions imports as one name (`[FFI-39b]`), since the header, not the program,
  declares them.
  - why: The reader's edit, kept: the exception says why the rule does not reach a header.

- BETTER EDIT rule=FFI-39b
  - old: `super.init(…)` selects the C++ base constructor by arity and
  argument types and runs it at `super.init(…)` (`[CLS-11]`); a base with no default constructor and
  no declared `init` is `E5058`.
  - new: Each use of a C++ name of several functions (as `[FFI-30]` counts them) stands for the one it
  fits exactly: a call (`new`, `super.init(…)`, a method, a function of a namespace) by its
  arguments' number, names, types and words, an untyped literal counting as the type C++ gives it
  (`c_int`, `f64`); an `override fn` by its parameters' types; a function value by its expected
  callable type; an overlay entry by what it writes (`[GRM-35]`). No conversion but a literal's
  typing takes part, so a use fits one function or none, as C++ would pick it or refuse. Two methods
  alike but for the method's own `const` are one Ember method, as `operator[]`'s pair is `Index` and
  `IndexMut` (`[FFI-42]`): a call runs the `const` one, and the other where the call's result is
  assigned through, given `ref mut`, or has a method called on it that changes its object
  (`[FFI-40]`), so the choice is read from the use and never from what the object's name permits
  (such a use through a name that only reads the object is `E3023`); one overlay entry covers the
  pair, its `@ffi(const)` speaking of the `const` one (`[FFI-40a]`). What the code around the use
  permits (`unsafe`, effects, hand-overs, threads, views) is checked after, on the one chosen, so a
  line runs the same function wherever it stands. A use that none fits, or that two fit, is `E5070`,
  naming each C++ declaration ("`r.draw(x)`: `Renderer::draw(const Mesh&)` and
  `Renderer::draw(const Sprite&)` both take `x` as written; give `x` the type of one"); an entry so
  placed is `E5010`. `super.init(…)` runs the constructor it selects at `super.init(…)` (`[CLS-11]`);
  a base with no default constructor and no declared `init` is `E5058`.
  - why: The reader's rule with the `const` pair decided by the result's use (the nearest rule, `[FFI-42]`), which removes the circularity through `this` and lets one entry vouch for the pair; exact fits only, so C++'s widening never changes which function an override or call means.

- BETTER EDIT rule=GRM-35
  - old: In an entry marked `@ffi(macro_fn)` (`[FFI-6b]`) a parameter's and the result's words are one Ember
  type each (`IDENT ":" type`, `"->" type`), since a macro has no C declaration to read (ODR-147).
  - new: In an entry marked `@ffi(macro_fn)` (`[FFI-6b]`) a parameter's and the result's words are one Ember
  type each (`IDENT ":" type`, `"->" type`), since a macro has no C declaration to read (ODR-147). Any
  entry may write a parameter's Ember type as the import maps it, before its contract words
  (`value: f32 borrowed one`), checked against the header; it is needed only where an entry would
  otherwise fit two of a C++ name's functions (`[FFI-39b]`), and `[FFI-30c]`, `[FFI-12]` and
  `[FFI-55]` count entries by function.
  - why: The reader's edit, kept: Ember's own spelling of a parameter, already in the grammar for macro entries.

- BETTER EDIT rule=FFI-55 (the `ember.overlays` example)
  - old:   SDL_FreeSurface = "blake3:…"
  - new:   SDL_FreeSurface = "blake3:…"
  "Renderer::draw(const Mesh&)" = "blake3:…"   # a C++ name of several functions: one line per function ([FFI-39b])
  - why: The record keys a fingerprint by name; a name of several functions needs a line per function or the check of `[FFI-55]` cannot tell which changed.

- BETTER EDIT rule=FFI-48 (the overloads row)
  - old: | overloads differing only in return type | Ember has no return-type overloading | rename one in the overlay |
  - new: | overloads differing only in return type | no call's arguments can tell them apart (`[FFI-39b]`) | a C++ wrapper with another name |
  - why: The reader's edit, kept: `rename` takes one name and cannot name one overload.

## K31-c-kept-callback-declaration  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=none

**answered_by**: FFI-10: "A `user_data=p` parameter of a `callback(…)` contract is an ABI witness as `count(n)`'s `n` is: `p` is written as a raw pointer, or an integer wide enough for the tagged number (`E5002` otherwise, since `[FFI-53]`'s number is 64 bits), at its C ABI position, is absent from the callable signature, and the call supplies the callback's tagged number for it; a `safe fn` may name it (`[FFI-21]`)."; FFI-21: "The callable type of a `callback(…)` parameter is `fn(P…) -> R`: the C function pointer's parameters mapped by `[FFI-8]` with the user-data one removed, a raw pointer among them staying raw (a lambda's body uses it in `unsafe:`); `thread_shared fn(P…) -> R` under `threads(any)` (`[FFI-22]`), and `once fn(P…) -> R` under `callback(once)`; a function given to it is written as any function value is (`fn() owns game: …`)."; FFI-21: "`retained` and `once` return a `Retained[ForeignCallback]` token (`[FFI-23]`) whose end unregisters the callback through the declared `destroys(f)`"; FFI-21: "A function that captures anything is `E5040` unless the parameter's contract names a `user_data` parameter, in which case the importer generates the checked trampoline (`[FFI-53]`): the function value, its code and its state, is moved into a block of its own"; FFI-52 (example): "safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void) … return register_cb(fn() owns game: game.add_point())"; FN-2a: "A parameter of a callable type (`owns f: fn(int)`) keeps what it is given and takes `owns` as any `owns` parameter does."; FN-2a: "`owns h` before such an argument is `E0109` ("`apply` runs `h` and keeps nothing of it; remove `owns`: `apply(h)`"); a function written in the call is given with `owns` (verdict 158)"; FFI-10: "A `safe fn` writes the safe Ember types and the words in front of its parameters' names (`ref mut`, `owns`) produced by those words; the compiler checks that mapping and lowers it to the C ABI carrier."

**program**: Three of the finder's four points are now decided: the declaration compiles (`ud: *mut void` is the user-data witness of `[FFI-10]`, absent from the callable signature, so the one-argument call is right), the callable type is `fn()` (`[FFI-21]`), and the call's value is the `Retained[ForeignCallback]` token. Left undecided, and decided against `[FN-2a]`'s own property: the hand-over word. `[FN-2a]` says a parameter of a callable type that keeps what it is given is an `owns` parameter and takes `owns` at the call, a function written in the call included (verdicts 147, 158); `[FFI-21]` moves the function into a block C keeps, a hand-over, yet its callable parameter is plain and the example calls `register_cb(fn() owns game: …)` with no word. For a named function the text says nothing an implementer can act on:

```
fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:
    fn add() owns game:
        game.add_point()
    t = register_cb(add)      # [FFI-21]: `add`'s state "is moved into a block of its own" (a hand-over with no word);
                              # [FN-2a]: a function from a variable is lent and `owns add` is E0109 for a non-`owns` parameter;
                              # so is `add()` on the next line E3040, allowed, or is this call refused?
    add()
    return t
```

- READER EDIT rule=FFI-21
  - old: The callable type of a `callback(…)` parameter is `fn(P…) -> R`: the C
  function pointer's parameters mapped by `[FFI-8]` with the user-data one removed, a raw pointer
  among them staying raw (a lambda's body uses it in `unsafe:`);
  - new: The callable type of a `callback(…)` parameter is `fn(P…) -> R`: the C
  function pointer's parameters mapped by `[FFI-8]` with the user-data one removed, a raw pointer
  among them staying raw (a lambda's body uses it in `unsafe:`); under `callback(retained)` or
  `callback(once)` C keeps the function, so the parameter is an `owns` parameter, `owns cb: fn(P…)
  -> R`, and its argument says `owns` at the call as every argument for an `owns` parameter does
  (`[FN-2a]`, verdicts 147 and 158), a function written in the call (`register_cb(owns fn() owns
  game: …)`) and a named one (`register_cb(owns add)`, after which `add` is handed over, `E3040` at a
  later use) alike, `E3028` without it; under `callback(borrowed)` the parameter is plain and the
  function is lent for the call with no word, as to any call that runs it then;
  - why: The cause is the parameter's word: once the parameter says what C does with the function (`owns`: kept), `[FN-2a]`'s one rule decides both the written and the named function, and nothing is handed over silently.

- READER EDIT rule=FFI-10
  - old: A
  `safe fn` writes the safe Ember types and the words in front of its parameters' names (`ref mut`,
  `owns`) produced by those words; the compiler checks that mapping and lowers it to the C ABI
  carrier.
  - new: A
  `safe fn` writes the safe Ember types and the words in front of its parameters' names (`ref mut`,
  `owns`) produced by those words; the compiler checks that mapping and lowers it to the C ABI
  carrier. A `callback(…)` parameter is written as the C function pointer it is, its callable type
  being `[FFI-21]`'s, with `owns` in front of its name under `retained` or `once` (`owns cb: extern
  "C" fn(*mut void)`), as `exclusive` puts `ref mut` there; the word missing is `E5002` ("`register_cb`
  keeps `cb` for C (`callback(retained)`), so it is handed over: write `owns cb: extern "C" fn(*mut
  void)`").
  - why: The rule says a safe declaration writes the produced words in front of the names; the example writes the C pointer type, which this sentence makes the stated exception, and the `owns` word follows the rule's own property.

- READER EDIT rule=FFI-52 (example)
  - old:       safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)
  - new:       safe fn register_cb(owns cb: extern "C" fn(*mut void), ud: *mut void)
  - why: The document's declaration follows the edited `[FFI-10]`.

- READER EDIT rule=FFI-52 (example)
  - old:       return register_cb(fn() owns game: game.add_point())
  - new:       return register_cb(owns fn() owns game: game.add_point())
  - why: The call says `owns` for an `owns` parameter (verdicts 147, 158).

- READER EDIT rule=FFI-21 (the E7004 message)
  - old: say what it holds: write `return register_cb(fn() owns game: game.add_point())` in place
  of line 17
  - new: say what it holds: write `return register_cb(owns fn() owns game: game.add_point())` in place
  of line 17
  - why: The line a message gives compiles.

- READER EDIT rule=FFI-22 (the E7015 message)
  - old: `return register_cb(fn() thread_shared game: game.add_point())` in
  `setup`
  - new: `return register_cb(owns fn() thread_shared game: game.add_point())` in
  `setup`
  - why: The line a message gives compiles.

**remaining_doubt**: K12 (another agent's cluster) must agree: a named function given to a callee that keeps it is handed over with `owns`, which is what `[FN-2a]`'s callable-type sentence already says for `owns f: fn(int)`. Small residue an implementer may still ask: when the C function returns a value of its own (`int listen(cb, ud)`), `[FFI-11]` says the token keeps it for `destroys(f)`, but no sentence says the call returns the token alone and not a tuple as `[FFI-11b]` returns for a counted output; one clause there would settle it.

**REFUTER reason**: Confirmed as an implementer question. `[FN-2a]`: "A parameter of a callable type (`owns f: fn(int)`) keeps what it is given and takes `owns` as any `owns` parameter does", and "a function written in the call is given with `owns` (verdict 158)"; `[FFI-21]` under `callback(retained)`: "the function value, its code and its state, is moved into a block of its own" — a hand-over — yet the parameter in `[FFI-52]`'s example is plain, `safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)`, and the call `register_cb(fn() owns game: game.add_point())` writes no word. For `register_cb(add)` with a named `add` holding `owns game`, nothing says whether `add` ends (`E3040` at the next use) or is usable, and an implementer who lets it be used reads a moved value. The reader's `[FFI-21]` edit (an `owns` parameter under `retained`/`once`, `owns` at the call, `E3028` without it) is at the cause and follows verdicts 147 and 158. Its `[FFI-10]` edit, though, makes the C pointer type the written type of a `callback(…)` parameter ("written as the C function pointer it is ... `owns cb: extern "C" fn(*mut void)`"), an exception to the rule's own sentence that a `safe fn` "writes the safe Ember types and the words in front of its parameters' names (`ref mut`, `owns`) produced by those words; the compiler checks that mapping and lowers it to the C ABI carrier", while `[FFI-21]` already names the safe type ("The callable type of a `callback(…)` parameter is `fn(P…) -> R`"). A `Span[T]` parameter is written as `Span[T]` with its witness `n` in C form (ODR-074); the callback is the same shape: the callable type written, the user-data witness in C form. So the better fix writes `owns cb: fn()` (the finder's A) and keeps the reader's `[FFI-21]` and message edits. No cost while running.

- BETTER EDIT rule=FFI-21
  - old: The callable type of a `callback(…)` parameter is `fn(P…) -> R`: the C
  function pointer's parameters mapped by `[FFI-8]` with the user-data one removed, a raw pointer
  among them staying raw (a lambda's body uses it in `unsafe:`);
  - new: The callable type of a `callback(…)` parameter is `fn(P…) -> R`: the C
  function pointer's parameters mapped by `[FFI-8]` with the user-data one removed, a raw pointer
  among them staying raw (a lambda's body uses it in `unsafe:`); under `callback(retained)` or
  `callback(once)` C keeps the function, so the parameter is an `owns` parameter, `owns cb: fn(P…)
  -> R`, and its argument says `owns` at the call as every argument for an `owns` parameter does
  (`[FN-2a]`, verdicts 147 and 158), a function written in the call (`register_cb(owns fn() owns
  game: …)`) and a named one (`register_cb(owns add)`, after which `add` is handed over, `E3040` at a
  later use) alike, `E3028` without it; under `callback(borrowed)` the parameter is plain and the
  function is lent for the call with no word, as to any call that runs it then;
  - why: The reader's edit, kept: once the parameter says what C does with the function, `[FN-2a]`'s one rule decides the written and the named function.

- BETTER EDIT rule=FFI-10
  - old: A
  `safe fn` writes the safe Ember types and the words in front of its parameters' names (`ref mut`,
  `owns`) produced by those words; the compiler checks that mapping and lowers it to the C ABI
  carrier.
  - new: A
  `safe fn` writes the safe Ember types and the words in front of its parameters' names (`ref mut`,
  `owns`) produced by those words; the compiler checks that mapping and lowers it to the C ABI
  carrier. A `callback(…)` parameter is so written too: its type is `[FFI-21]`'s callable type
  (`cb: fn()`; `thread_shared fn()` under `threads(any)`, `once fn()` under `callback(once)`), with
  `owns` in front of its name under `retained` or `once` (`owns cb: fn()`), as `exclusive` puts
  `ref mut` there, and the compiler lowers it to the C function pointer, giving C the trampoline
  (`[FFI-53]`) and the witness; the C pointer type written there, or the word missing, is `E5002`
  ("`register_cb` keeps `cb` for C (`callback(retained)`), so it is handed over, and a `safe fn`
  writes the Ember type: write `owns cb: fn()`").
  - why: The rule's own principle applied, not an exception to it: the safe type is written and the compiler lowers it, as it does for a `Span[T]` and its `count(n)` witness.

- BETTER EDIT rule=FFI-52 (example)
  - old:       safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)
  - new:       safe fn register_cb(owns cb: fn(), ud: *mut void)
  - why: The document's declaration follows the edited `[FFI-10]`.

- BETTER EDIT rule=FFI-52 (example)
  - old:       return register_cb(fn() owns game: game.add_point())
  - new:       return register_cb(owns fn() owns game: game.add_point())
  - why: The call says `owns` for an `owns` parameter (verdicts 147, 158).

- BETTER EDIT rule=FFI-21 (the E7004 message)
  - old: say what it holds: write `return register_cb(fn() owns game: game.add_point())` in place
  of line 17
  - new: say what it holds: write `return register_cb(owns fn() owns game: game.add_point())` in place
  of line 17
  - why: The line a message gives compiles (`[DIA-25]`).

- BETTER EDIT rule=FFI-22 (the E7015 message)
  - old: `return register_cb(fn() thread_shared game: game.add_point())` in
  `setup`
  - new: `return register_cb(owns fn() thread_shared game: game.add_point())` in
  `setup`
  - why: The line a message gives compiles (`[DIA-25]`).

## K46-ffi-handle-meaning  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=none

**answered_by**: FFI-8: "A `handle` is an opaque pointer-sized `Copy` value with no destructor of its own (`destroys(f)` applies to an `owns T` of its type, never to the handle), `nullable` as an `Option` of it, and bound to its thread under `threads(creator)` (`[FFI-33b]`, `E7008`)."; FFI-8: "An `extern type`, an opaque C++ class (`[FFI-32]`) and a `@ffi(handle)` type hold what the compiler cannot see, so neither marker is inferred for them (`[THR-8]`, `[THR-9]`): each is `Send` unless its contract says `threads(creator)` or `threads(main)`"; FFI-11: "`handle` (an opaque pointer-sized `Copy` value, `[FFI-8]`)"; FFI-1: "`threads(creator)` and `handle` are words of a type's or a result's contract, not a function's: on a function each is a contract error (`E5002`)."; FFI-11a: "A pointer contract with no count word is `E5012`, which lists the five count words and names any sibling parameter that looks like a length"; XVI.4 example: "fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed one, … fn vkCmdSetViewport(commandBuffer: borrowed, firstViewport, viewportCount, …"; Appendix F: "**handle** | a token that names something elsewhere: `Handle[T]`, a generational index into a `Pool` (`[HND-1]`), a `JoinHandle`, a C object's `@ffi(handle)`; never a class object"

**program**: Hardened_2 answered most of the question (the finder's A): a handle is an opaque `Copy` value, passed by value, no destructor, `nullable` is an `Option` of it, `threads(creator)` binds it, no marker is inferred. Two lines stay undecidable. First, the document's own overlay writes pointer words on handle parameters, and no rule says whether a handle parameter is a pointer contract:

```
unsafe overlay c "vulkan/vulkan.h":
    @ffi(handle)
    type VkDevice
    @ffi(no_callbacks)
    fn vkDestroyDevice(device: borrowed, pAllocator: nullable borrowed one)
    # `device: borrowed` with no count word: E5012 by [FFI-11a]? accepted, as the XVI.4 example assumes? E5002?
```

Second, "pointer-sized" is false for Vulkan's non-dispatchable handles (`VkBuffer` is a `uint64_t` on 32-bit targets, `VK_DEFINE_NON_DISPATCHABLE_HANDLE`), so an implementer on a 32-bit target cannot lay the type out from the rule.

- READER EDIT rule=FFI-8
  - old: A `handle` is an opaque pointer-sized `Copy` value with
  no destructor of its own (`destroys(f)` applies to an `owns T` of its type, never to the handle),
  `nullable` as an `Option` of it, and bound to its thread under `threads(creator)` (`[FFI-33b]`,
  `E7008`).
  - new: A `handle` is an opaque `Copy` value with the C type's representation (a pointer or an integer:
  Vulkan's `VkBuffer` is a `uint64_t` on 32-bit targets), passed and returned by value and never
  read through, with no destructor of its own (`destroys(f)` applies to an `owns T` of its type,
  never to the handle), `nullable` as an `Option` of it (the null pointer or zero is `None`), and
  bound to its thread under `threads(creator)` (`[FFI-33b]`, `E7008`). A parameter or result of a
  handle type is no pointer contract: it takes no word but `nullable`, and a pointer word on it is
  `E5002` ("`device` is a `VkDevice`, a `handle` passed by value; remove `borrowed`").
  - why: The cause is the definition: it said what a handle is but not that a handle slot is outside `[FFI-11]`'s pointer contract, and its size was a fact of one representation.

- READER EDIT rule=FFI-11
  - old: `handle` (an
  opaque pointer-sized `Copy` value, `[FFI-8]`)
  - new: `handle` (an
  opaque `Copy` value passed by value, taking no pointer words, `[FFI-8]`)
  - why: The cross-reference claims what its rule says.

- READER EDIT rule=XVI.4 example (the Vulkan overlay above [GRM-35])
  - old:     fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed one,
  - new:     fn vkCreateBuffer(device, pCreateInfo: borrowed one,
  - why: The document's example compiles under its rule; merged with K07's rewrite of the same entry (which also makes it `unsafe fn` with `@safety`), and `commandBuffer: borrowed` → `commandBuffer` on `vkCmdSetViewport` the same way.

**remaining_doubt**: Whether `@ffi(handle)` on a `type` in a hand-written `unsafe extern "C":` block is meant as well as on an overlay `type` entry: `[FFI-54]`'s example writes `@ffi(destroys(widget_free)) type Widget` in such a block, so by analogy yes, but no sentence says a hand-declared `type` takes `handle`. `nullable` on an integer handle maps `None` to zero, which is `VK_NULL_HANDLE`; a C API whose null handle is another value would need a word the text does not have.

**REFUTER reason**: The two lines are undecided as the reader says. `[FFI-8]`: "A `handle` is an opaque pointer-sized `Copy` value with no destructor of its own ..., `nullable` as an `Option` of it" — nothing says whether a parameter of a handle type is a pointer contract, so `device: borrowed` in the document's own overlay is either accepted or `E5012` by `[FFI-11a]` ("A pointer contract with no count word is `E5012`"), since `[FFI-1]` counts "each pointer-typed parameter" and `VkDevice` is a pointer typedef in C; and "pointer-sized" is false for `VkBuffer` on a 32-bit target (`VK_DEFINE_NON_DISPATCHABLE_HANDLE`, a `uint64_t`), so the layout cannot be read from the rule (it is verified by `[FFI-5]`, which only makes the sentence a wrong fact). The reader's edit is right on both, but its clause "it takes no word but `nullable`, and a pointer word on it is `E5002`" removes the ownership words from handle slots, which the text relies on: `[FFI-8]` itself says "`destroys(f)` applies to an `owns T` of its type", and `[FFI-33b]` "`returns_owned` results and callbacks may carry `threads(creator)`: an `owns T` foreign value of such a type is two words ... the rule that makes GPU, GL-context and COM handles safe to hold in ordinary values" — an `owns T` of a handle type comes from `returns_owned(destructor=f)` on a handle result and goes to `owns` on a handle parameter. The better text refuses the count, lifetime, aliasing and `borrowed` words (a handle is passed by value) and keeps `nullable`, `owns` and `returns_owned(destructor=f)`. The example and `[FFI-11]` edits stand.

- BETTER EDIT rule=FFI-8
  - old: A `handle` is an opaque pointer-sized `Copy` value with
  no destructor of its own (`destroys(f)` applies to an `owns T` of its type, never to the handle),
  `nullable` as an `Option` of it, and bound to its thread under `threads(creator)` (`[FFI-33b]`,
  `E7008`).
  - new: A `handle` is an opaque `Copy` value with the C type's representation (a pointer or an integer:
  Vulkan's `VkBuffer` is a `uint64_t` on 32-bit targets), passed and returned by value and never
  read through, with no destructor of its own (`destroys(f)` applies to an `owns T` of its type,
  never to the handle), `nullable` as an `Option` of it (the null pointer or zero is `None`), and
  bound to its thread under `threads(creator)` (`[FFI-33b]`, `E7008`). A parameter or result of a
  handle type is no pointer contract: it is passed by value, so a count, lifetime or aliasing word
  or `borrowed` on it is `E5002` ("`device` is a `VkDevice`, a `handle` passed by value; remove
  `borrowed`"); it takes `nullable`, and the ownership words where the call transfers the handle's
  object, `returns_owned(destructor=f)` on a result and `owns` on a parameter, which give and take an
  `owns T` of the handle type (`[FFI-33b]`, two words under `threads(creator)`).
  - why: The reader's definition with the ownership words kept: `[FFI-8]`'s own `destroys(f)` sentence and `[FFI-33b]`'s GPU handles need an `owns T` of a handle type, which `returns_owned` and `owns` produce.

- BETTER EDIT rule=FFI-11
  - old: `handle` (an
  opaque pointer-sized `Copy` value, `[FFI-8]`)
  - new: `handle` (an
  opaque `Copy` value passed by value, taking no pointer words, `[FFI-8]`)
  - why: The reader's edit, kept: the cross-reference claims what its rule says.

- BETTER EDIT rule=XVI.4 example (the Vulkan overlay above [GRM-35])
  - old:     fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed one,
  - new:     fn vkCreateBuffer(device, pCreateInfo: borrowed one,
  - why: The reader's edit, kept and merged with K07's rewrite of the same entry (`unsafe fn` with `@safety`); `commandBuffer: borrowed` becomes `commandBuffer` on `vkCmdSetViewport` the same way.

# MISSED

## MISSED: A `@ffi(std_function)` callable whose parameters hold a raw pointer is safe to call (K07's hole in a fourth spelling)

**rules**: `[FFI-17a]` (the `std::function` row): "with `@ffi(std_function)` on its overlay entry, a function value of type `fn(A…) -> R` (`[CLO-3]`) which owns its target, a foreign function"; `[FFI-56]`: "A C function pointer is unsafe to call ... nothing tells the compiler what the function it points to does with its arguments"; `[FFI-2]` (as edited for K07) limits only `safe fn`, an overlay function entry and `@ffi(safe)`.

**program**:
```
// fill.hpp
// namespace px { struct Buf { std::uint8_t* data; std::size_t len; };
//   using Fill = std::function<void(Buf)>;
//   inline Fill make_filler() noexcept { return [](Buf b) { for (size_t i = 0; i < b.len; ++i) b.data[i] = 0; }; } }
# overlays/fill.em
unsafe overlay cpp "fill.hpp":
    @ffi(std_function)
    type Fill
    @ffi(no_callbacks)
    fn make_filler()
# main.em
import cpp "fill.hpp" with (project="fill", overlay="overlays/fill.em") as f

fn main():
    fill = f.px.make_filler()
    x: u8 = 7
    fill(f.px.Buf(data=ref_to_ptr(ref mut x), len=4096))   # a safe call of a `fn(Buf)`: C++ writes 4,095 bytes past `x`

```

- MISSED EDIT rule=FFI-17a (the `std::function` row)
  - old: a function value of type `fn(A…) -> R` (`[CLO-3]`) which owns its target, a foreign function:
  - new: a function value of type `fn(A…) -> R` (`[CLO-3]`), or of `unsafe fn(A…) -> R` (`[FN-6]`) when a parameter is or holds, at any depth, a raw pointer (nothing states what the target does with it, `[FFI-56]`, `[FFI-2]`'s limit; a call is then in `unsafe:`, `E3100`), which owns its target, a foreign function:
  - why: The same property as `[FFI-2]`'s limit, in the one spelling it does not reach; the earlier design left it to the owner, but the hole is memory corruption from safe code and the fix is the callable type the language already has.

## MISSED: A `@ffi(newtype)` over a pointer typedef is a pointer under another name (the K07 limit's words `*T`, `*mut T` do not reach it)

**rules**: `[FFI-8]` (typedef row): "with `@ffi(newtype)` a nominal type with the typedef's representation, passed to C as the typedef is, converted with `as` only"; §IV.5: "Raw-pointer-to-raw-pointer and pointer/integer casts still require `unsafe` (`[TYP-7]`)" names raw pointer types only; `[FFI-10]`'s and `[FFI-2]`'s (edited) limits name "a raw pointer (`*T`, `*mut T`)".

**program**:
```
// tex.h: typedef struct tex* tex_ref;  struct draw_cmd { tex_ref tex; float x, y; };  void tex_draw(struct draw_cmd);
import c "tex.h" with (overlay="tex.em") as tx      # overlay: `@ffi(newtype) type tex_ref`, `fn tex_draw(cmd)`

fn main():
    bogus = null[*mut tx.tex]() as tx.tex_ref          # "converted with `as` only": safe or unsafe? the text does not say
    tx.tex_draw(tx.draw_cmd(tex=bogus, x=0.0, y=0.0))   # a `@layout(c)` value holding no `*T` field: outside the limit as worded; C reads through null

```

- MISSED EDIT rule=FFI-8 (type mapping table, typedef row)
  - old: | `typedef` | a type alias; with `@ffi(newtype)` a nominal type with the typedef's representation, passed to C as the typedef is, converted with `as` only |
  - new: | `typedef` | a type alias; with `@ffi(newtype)` a nominal type with the typedef's representation, passed to C as the typedef is, converted with `as` only; one whose representation is a pointer is a raw pointer for `[THR-8]`, `[UNS-1]` and `[FFI-2]`'s limit, and `as` into it from a pointer or an integer is unsafe (`[TYP-7]`), so safe code never makes one |
  - why: A type whose values safe code could turn into any address is a raw pointer whatever its name; stated on the row that makes the type.

## MISSED: `[UNS-1]` does not list reading a foreign static that `[FFI-8]` and `[FFI-41]` say needs `unsafe`

**rules**: `[UNS-1]`: "An `unsafe` context is required to: ... call an `extern` function without a verified contract (Part XVI); access a foreign `static mut` (`[STA-1]`); ..."; `[UNS-2]`: "`unsafe` permits exactly those operations"; `[FFI-8]` (static row): "reading it needs `unsafe` unless `@ffi(immutable)` ..."; `[FFI-41]`: "in `unsafe` unless it is `const` and holds no pointer".

**program**:
```
// limits.h: extern int CURRENT_PLAYERS;
import c "limits.h" as lc

fn main():
    println(lc.CURRENT_PLAYERS)   # `[FFI-8]`: "reading it needs `unsafe`"; `[UNS-1]`'s closed list names a foreign `static mut` only, so which code, if any, refuses this line?

```

- MISSED EDIT rule=UNS-1
  - old: access a foreign `static mut` (`[STA-1]`);
  - new: access a foreign `static mut`, or read a foreign static outside the safe reads of `[FFI-8]`'s static row and `[FFI-41]` (`[STA-1]`);
  - why: `[UNS-2]` makes `[UNS-1]`'s list the whole of what `unsafe` permits, so the operation the FFI rules fence must be in it, with its `E3100`.

## MISSED: What a registering call returns when the C function has a result of its own (K31's residue)

**rules**: `[FFI-21]`: "`retained` and `once` return a `Retained[ForeignCallback]` token (`[FFI-23]`) whose end unregisters the callback through the declared `destroys(f)`"; `[FFI-11]`: "`destroys(f)` (`f`'s parameters may be the registering call's own and its result, `id = listen(…)` then `unlisten(id)`: the token keeps them)"; `[FFI-11b]`: "When the C function has a result of its own, the Ember function returns a tuple: that result first, then each pointer output".

**program**:
```
unsafe extern "C":
    @safety("`id` is a listener `listen` returned, not yet removed")
    unsafe fn unlisten(id: c_int)
    @ffi(param(cb, callback(retained, user_data=ud)), destroys(unlisten))
    safe fn listen(owns cb: fn(), ud: *mut void) -> c_int

fn main():
    t = listen(owns fn(): println("tick"))   # the token alone (`[FFI-21]`), or `(c_int, Retained[ForeignCallback])` (`[FFI-11b]`'s order)? the text says both

```

- MISSED EDIT rule=FFI-21
  - old: 
  - new: A registering call whose C function has a result of its own returns that result before the token, as `[FFI-11b]` orders a tuple, unless `destroys(f)` names the result among `f`'s parameters (`[FFI-11]`), in which case the token keeps it and the call returns the token alone.
  - why: Two rules each describe the call's value; one sentence says which holds when.

## MISSED: Whether a hand-declared `type` in an `unsafe extern "C":` block takes `@ffi(handle)` (K46's doubt)

**rules**: `[FFI-10]`: "An `unsafe extern "C":` block declares foreign functions, statics and opaque types"; `[FFI-8]`: "An `extern type`, an opaque C++ class (`[FFI-32]`) and a `@ffi(handle)` type hold what the compiler cannot see"; `[FFI-54]`'s example writes `@ffi(destroys(widget_free)) type Widget` in such a block; no sentence says which type words a hand `type` takes.

**program**:
```
unsafe extern "C":
    @ffi(handle)
    type VkDevice                      # accepted as an overlay `type` entry is, or `E0104` (an attribute the block's `type` does not take)? the text does not say
    @ffi(no_callbacks)
    safe fn vkDeviceWaitIdle(device: VkDevice) -> c_int

```

- MISSED EDIT rule=FFI-10
  - old: An `unsafe extern
  "C":` block declares foreign functions, statics and opaque types;
  - new: An `unsafe extern
  "C":` block declares foreign functions, statics and opaque types (`type Widget`, which takes the words an overlay `type` entry takes: `@ffi(destroys(f))`, `@ffi(handle)`, `@ffi(threads(creator))`, `[FFI-54]`, `[FFI-8]`);
  - why: The hand declaration and the overlay entry are one contract system (`[FFI-11]`: "that attribute does not create a separate contract system"), so the type words of one are the other's; said once where the block is defined.
