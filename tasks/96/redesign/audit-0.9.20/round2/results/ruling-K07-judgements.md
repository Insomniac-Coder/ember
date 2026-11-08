# K07: May a safe C or C++ call take a struct that holds raw pointers, and what limits raw pointers under the safe-to-call fact?

### Design 1 summary
No. The safe-to-call fact ([FFI-2]) gets one limit, stated where the fact is defined and shared by every spelling that asserts it (`safe fn`, a plain overlay function entry, `@ffi(safe)` on a function-pointer typedef, and `safe` beside `std_function`): no assertion covers a raw pointer that the call carries to foreign code, whether it is the argument itself, is held at any depth by what an argument passes (a field, a union field, an element, a C++ method's object, read from the Ember type), or is the result of a function-pointer type among them (what an Ember callback returns to C); such an assertion is `E5002` where it is written, and the function is `unsafe fn` with `@safety`, with an `extend` wrapper that builds the struct from a checked view inside `unsafe:`. The std_function contradiction that blocked the method-4 team is closed by giving the imported `std::function` the exact shape of [FFI-56]'s C function pointer: with `@ffi(std_function)` alone its value is an `unsafe fn(A…) -> R`, and `@ffi(std_function, safe)` asserts the target safe to call under the same limit. Nothing runs while the program runs: the limit is a walk over declared types at compile time, and the wrapper emits the same stores and the same single call as the C it replaces.

### Design 2 summary
No. The safe-to-call fact ([FFI-2]: "The safe-to-call fact is never derived ... it is asserted by `safe fn` in an extern block (`[FFI-10]`), or by an overlay entry for the function") gets one limit, written where the fact is defined and shared by every spelling that asserts it: no assertion covers a raw pointer or C function pointer that the caller supplies and the call carries to foreign code, as an argument, inside what an argument passes (read from its declared Ember type at any depth), or as what an Ember function given to the call returns; a declaration or entry that asserts the fact and would carry one is `E5002` where it is written, with the `unsafe fn` + `@safety` line and the `unsafe:` block as the way out, and a wrapper over a checked span as the safe interface. The contradiction on `std::function` is closed by giving it the C function pointer's rule ([FFI-56]): `@ffi(std_function)` makes an `unsafe fn(A…) -> R` (its target is "a foreign function", [FFI-17a], and nothing describes it), and `@ffi(std_function, safe)` the `fn(A…) -> R` of today, within the same limit. Nothing runs for any of this.

### Design 3 summary
The rule a newcomer holds in one sentence: safe code never hands C an address it made up, so a C or C++ call is safe to make only when nothing the call gives foreign code holds a raw pointer, alone, in a struct, in a span's elements, in a C++ object, or returned by an Ember function the call gives C; a raw pointer reaches foreign code only from an `unsafe:` block, under a `@safety` someone wrote. The limit is stated once, in `[FFI-2]`, where the safe-to-call fact is defined, and every spelling that asserts the fact shares it: `safe fn`, a plain overlay function entry, `@ffi(safe)` on a function-pointer typedef, and `safe` beside `std_function`; a `std::function` value is otherwise `unsafe fn(A…) -> R`, as a C function pointer already is under `[FFI-56]`, which removes the two-ways reading of `[FFI-17a]`. A refused declaration is `E5002` where it is written, naming the field that holds the pointer; the binding writes `@safety("…")` above `unsafe fn` and one `extend` wrapper that fills the struct from a checked `MutSpan`, so callers write a shorter safe line and nothing runs while the program runs.


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
