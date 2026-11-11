# Rules changed or new from 0.9.15_Hardened_1 to 0.9.20_Hardened_12, gap area 06

`[[old words]]` were removed and `{{new words}}` added; each line starts with six words of context.

### FFI-2a (p16-ffi.md; 141 words changed)
  - …packing and alignment, and the calling [[convention.]]
  + …packing and alignment, and the calling {{convention; and the `const` of an object a header declares (`[FFI-8]`, `[FFI-41]`): in C it is never changed, in C++ only through a `mutable` member, which is why an object whose type has one is read in `unsafe`.}}
  - …calling convention. In **result** position a [[pointer's]]
  - …position a pointer's lifetime is never [[derived.]]
  + …position a pointer's lifetime is never {{derived: a pointer's, a C++ reference's (`[FFI-40]`), or that of a view a C++ type maps to (`std::span`, a fixed-extent span, `std::string_view`, `[FFI-17a]`), from a method or from a function outside a class alike. The entry gives it a lifetime word (`[FFI-11]`); without one the function stays unsafe to call (`E3100` outside `unsafe:`), its result a raw pointer (`*T`, `*mut T`) for a pointer or a reference and, for a view, the pointer with the count C++ returned, `(*T, usize)` or `(*mut T, usize)`, read inside `unsafe:` by an `extend` wrapper that copies the value (`[FFI-13]`); `ember bind --report` lists each (`[FFI-35a]`).}}

### FFI-10 (p16-ffi.md; 584 words changed)
  - …declares foreign functions, statics and opaque [[types;]]
  + …declares foreign functions, statics and opaque {{types (`type Widget`, which takes the words an overlay `type` entry takes: `@ffi(destroys(f))`, `@ffi(handle)`, `@ffi(threads(creator))`, `[FFI-54]`, `[FFI-8]`);}}
  - …are items of the enclosing module. [[A]]
  + …are items of the enclosing module. {{Each}}
  - …module. A function in the block [[is **safe to call** only when]]
  + …module. A function in the block {{says whether}}
  - …to call** only when it is [[declared]]
  + …to call** only when it is {{safe to call (verdict 136):}}
  - …declared `safe fn` (a contextual keyword, [[`[LEX-15]`) and]]
  + …declared `safe fn` (a contextual keyword, {{`[LEX-15]`), safe to call only when}}
  - …complete (given with `@ffi(…)` on the [[declaration);]]
  + …complete (given with `@ffi(…)` on the {{declaration), and within `[FFI-2]`'s limit on raw pointers,}}
  - …declaration); `safe` on any other function [[is]]
  + …declaration); `safe` on any other function {{being}}
  - …`E5002`, naming the fact that is [[missing. Every]]
  + …`E5002`, naming the fact that is {{missing or, under that limit, where the pointer is held; or `unsafe fn`, whose calls are in unsafe code, with `@safety("…")` above it saying what its callers must make true (`[UNS-7]`), as an overlay entry is (`[GRM-35]`, verdict 133). A plain `fn`, which everywhere else in Ember is safe to call, says neither and is `E5002`: "`lib_step` does not say whether it is safe to call: write `unsafe fn lib_step`, with `@safety("…")` above it saying what its callers must make true, or `safe fn lib_step` if a call within its contract cannot break memory safety", which `ember fmt --migrate` rewrites to `unsafe fn` (`[CLI-20]`); `safe unsafe fn` is `E5002` too ("`lib_step` is declared both `safe` and `unsafe`"). A call of an `unsafe fn` outside unsafe code is `E3100`, as any `unsafe fn`'s: "`raw_abs(-3)` (line 11) calls an `unsafe fn`, whose callers must make true what its `@safety` says". `unsafe` is no word of `@ffi(…)`: `@ffi(unsafe)`, `param(p, unsafe)` and `result(unsafe)` are `E0104` ("`unsafe` is no contract word; write the declaration `unsafe fn peek(…)`, with `@safety("…")` above it saying what its callers must make true"), which `ember fmt --migrate` rewrites (`[CLI-20]`); `@ffi(…)` keeps its}}
  - …fact that is missing. Every other [[call is in `unsafe`.]]
  + …fact that is missing. Every other {{words.}}
  + …to the C ABI carrier. A {{`callback(…)` parameter is so written too: its type is `[FFI-21]`'s callable type (`cb: fn()`; `thread_shared fn()` under `threads(any)`, `once fn()` under `callback(once)`), with `owns` in front of its name under `retained` or `once` (`owns cb: fn()`), as `exclusive` puts `ref mut` there, and the compiler lowers it to the C function pointer, giving C the trampoline (`[FFI-53]`) and the witness; the C pointer type written there, or the word missing, is `E5002` ("`register_cb` keeps `cb` for C (`callback(retained)`), so it is handed over, and a `safe fn` writes the Ember type: write `owns cb: fn()`"). A}}
  - …expose; `safe fn` with one is [[`E5002`.]]
  + …expose; `safe fn` with one is {{`E5002`, the argument's own case of `[FFI-2]`'s limit, which reaches a raw pointer at any depth of what a parameter passes (a field of a `@layout(c)` value, an element of a fixed array or of a bridge type, `CppVector[T]`, the payload of an `Option`, the pointee of a contracted pointer, a value passed by a C++ reference, `[FFI-40]`).}}
  + …mapped `safe fn` erases the witness. {{A `user_data=p` parameter of a `callback(…)` contract is an ABI witness as `count(n)`'s `n` is: `p` is written as a raw pointer, or an integer wide enough for the tagged number (`E5002` otherwise, since `[FFI-53]`'s number is a pointer's width: 64 bits on a 64-bit target, 32 on a 32-bit one, `[FFI-53]`), at its C ABI position, is absent from the callable signature, and the call supplies the callback's tagged number for it; a `safe fn` may name it (`[FFI-21]`).}}
  + …a null pointer and integer zero. {{A `borrowed one nullable` input is a plain `x: Option[T]`, or `ref mut x: Option[T]` with `exclusive`, written with `ref mut` in front of its name and with its argument marked `ref mut` at the call (`set_name(ref mut maybe_name)`): `None` sends a null pointer, and `Some` the address of its value, which C reads or, with `exclusive`, writes in place; the `Option`'s own tag never crosses.}}
  - …is null, the wrapper supplies a [[statically stored,]]
  + …supplies a statically stored, properly aligned {{sentinel, one}}
  - …a statically stored, properly aligned `T` [[sentinel.]]
  + …a statically stored, properly aligned `T` {{in read-only memory for a `Span[T]` and one in the call's own frame for a `MutSpan[T]`, so a C function that touches an element against a count of zero faults or writes a slot of this call and nothing shared.}}

### FFI-51 (p16-ffi.md; 1029 words changed)
  - …vouched for.** An import that compiles [[C]]
  + …that compiles C into the program {{foreign code that runs with no call from Ember}}
  - …source file (`import c "./bridge.c"`, `[BLD-FFI-1]`), [[or]]
  - …a header with `implementation = [...]` [[(`[FFI-29b]`)]]
  + …a header with `implementation = [...]` {{(`[FFI-29b]`), or a C or C++ header that does more than declare, in it or in a header it includes (the shim, `[FFI-29]`, or the thunk unit, `[FFI-3]`, `[FFI-17]`, compiles what it holds)}}
  - …`implementation = [...]` (`[FFI-29b]`) — runs [[C]]
  - …runs C code that nothing checks, [[a constructor in it even before `main`,]]
  - …so it is written `unsafe import [[c`,]]
  + …so it is written `unsafe import {{c` or `unsafe import cpp`,}}
  - …with a `# SAFETY:` note above [[it". An import of]]
  + …with a `# SAFETY:` note above {{it"; for}}
  - …it". An import of a header [[alone]]
  + …it". An import of a header {{the message names what runs: "`import cpp "json.hpp"` compiles the initialiser of `g_default` (json.hpp, line 12) into the program, where it runs unchecked before `main`; write `unsafe import cpp "json.hpp" with (project="engine") as js` with a `# SAFETY:` note above it"; "`import c "lib.h"` compiles `lib_run_setup` (lib.h, line 7) into the start-up section `.CRT$XCU`, so `lib_setup` runs unchecked before `main`; write `unsafe import c "lib.h" as lib` with a `# SAFETY:` note above it". **A header that only declares}}
  - …of a header alone needs no [[word:]]
  + …of a header alone needs no {{word.** It holds nothing but types, macros, declarations of functions and variables, definitions of `static` functions, and of `inline` functions the C compiler compiles only where they are called, with no assembly in their bodies, and variables that are `static`, `inline` or `constexpr` with a constant initialiser, no destructor to run and no section named for them:}}
  + …those calls are fenced by `[FFI-2]`. {{Anything else a header holds counts as running, as the importer reads the header, one it cannot decide counting as running (`[FFI-38]`): a function the loader or the C runtime calls (a constructor or a destructor attribute, an `ifunc` resolver); a variable at namespace scope, `inline` or a static member, whose initialiser is not a constant expression or whose type has a destructor to run; a function or a variable whose definition the linker sees whether or not anything calls it (one defined with external linkage and not `inline`, and an `inline` one the C compiler emits in every file that includes it, as Microsoft's `__inline` is), which the linker may give to code that never meant it; anything the header places in a section it names (`__attribute__((section(…)))`, `__declspec(allocate(…))`, `#pragma section`, `data_seg`, `code_seg`, `init_seg`), which is how a start-up table is filled with no initialiser to see; assembly, at file scope or in the body of a function the shim compiles (an `asm` label whose text, after preprocessing, is one plain symbol name, made of letters, digits, `_`, `$`, `.` and `@` and not starting with a digit, only renames the declaration it is on and is judged as that declaration, as glibc's `__isoc23_fscanf` and Apple's `fopen$DARWIN_EXTSN` are; any other label is assembly); and an order to the linker, a library asked for by name included (`#pragma comment(linker, …)`, `#pragma comment(lib, "uuid.lib")`), since what is linked brings code of its own; only a check that can do nothing but stop the link (`#pragma detect_mismatch`) is none of these. **No header is exempt** (verdict 171: any code that cannot be guaranteed safe is written with `unsafe`): the C or C++ compiler's own headers, its standard libraries' and the platform's are judged as every other header is, by what they hold. So `import c "windows.h"`, which has the linker add `uuid.lib`, and with MSVC an `import cpp` of a header that includes `<string>`, which asks for the C++ library and gives the linker an order, are written `unsafe import`, the note saying what is vouched for ("`import c "windows.h"` has the linker add `uuid.lib`, asked for by a header it includes, which brings code nothing checks; write `unsafe import c "windows.h" as win` with a `# SAFETY:` note above it"), and a header that declares its own functions and types, and includes only headers that do the same, needs no word. What decides is what the compiled header leaves in the program, on the target it is compiled for: with the Windows C runtime `<stdio.h>` defines `sprintf` and its family in every file that includes it, so an import that reaches it is written `unsafe import c` there, and one that reaches only `<stdlib.h>`, `<string.h>` or `<math.h>` is not. And where the C compiler writes assembly text and runs an assembler on it (GCC), the build reads the text of the shim and of the thunk unit before it is assembled, as `[MAN-10]` reads a dependency's, since a string a header gives GCC reaches that text as it is written (a `weakref` target, an `#ident` line): a directive there, outside a quoted name, that reads a file or makes directives (`.incbin`, `.include`, `.macro`, `.irp`, `.irpc`, `.rept`, `.altmacro`) is assembly the header holds, and the import without `unsafe` is `E3100` ("`import c "lib.h"` has GCC read `/home/ci/.aws/credentials` into the program with `.incbin`, from a string the header gives it; write `unsafe import c "lib.h" as lib` with a `# SAFETY:` note above it"). Clang writes such strings quoted, and its text needs no such read.}}
  + …are fenced by `[FFI-2]`. A library {{a header asks for is named in the manifest too, under `[link] unsafe_libs` or the list of the system the build is for (`[link.windows]`, `[BLD-FFI-3]`), as every}}
  - …fenced by `[FFI-2]`. A library linked [[by]]
  + …fenced by `[FFI-2]`. A library linked {{library is (verdict 173); where the manifest does not}}
  + …`[FFI-2]`. A library linked by name {{it the import is `E9001` ("`windows.h` has the linker add `uuid.lib`, and `ember.toml` does not name it; add `unsafe_libs = [\"uuid\"]` under `[link.windows]`,}}
  - …A library linked by name or [[path (`link]]
  + …A library linked by name or {{under `[link]` if every system needs it"). `ember tcb` lists it as an external library with the header and line that asks for it (`[TCB-4]`). **A linked library is named in the manifest and vouched for in the code** (verdict 172). The manifest names it under `[link] unsafe_libs` (`[BLD-FFI-3]`), and the import of a header that declares a function or a variable the library defines is written `unsafe import`, with its note: `unsafe_libs}}
  - …by name or path (`link = [[[...]`, `[link] libs`)]]
  + …by name or path (`link = {{["vulkan-1"]` in `ember.toml`, and `# SAFETY: vulkan-1 is the official Vulkan library` above `unsafe import c "vulkan/vulkan.h" as vk`. The library's code is in the program, with whatever it runs when it loads, and nothing checks it. The build reads from each named library the names it defines, so it knows which imports those are, and nothing more is written to tie the two; an import of such a header without the word is `E3100` ("`vkCreateInstance` of `import c "vulkan/vulkan.h"` (line 3) comes from `vulkan-1` (`ember.toml`, line 7), whose code nothing checks; write `unsafe import c "vulkan/vulkan.h" as vk` with a `# SAFETY:` note above it"). Each library}}

### FFI-8 (p16-ffi.md; 761 words changed)
  + …* `[FFI-8]` **Type mapping.** {{An `extern type`, an opaque C++ class (`[FFI-32]`) and a `@ffi(handle)` type hold what the compiler cannot see, so neither marker is inferred for them (`[THR-8]`, `[THR-9]`): each is `Send` unless its contract says `threads(creator)` or `threads(main)`, so one thread at a time uses an object of it, and `ThreadShared` only where the program declares it, `unsafe extend Ctx implements ThreadShared:` with a `# SAFETY:` note saying that the library reads one object from several threads at once (as `Send` is declared for a type holding a raw pointer, `[THR-8]`; `E3100` without `unsafe`), an asserted fact `ember tcb` lists, and never for a type whose contract says `threads(creator)` or `threads(main)`, which binds every object to one thread: the declaration is `E7008` at its line ("`Ctx` is used by the thread that made it (`threads(creator)`), so no other thread may read one; remove the `unsafe extend`, or change the contract if the library allows any thread"); C's `const` says what the pointer's holder may do, not what the callee does (`[FFI-40a]` for C++), so reading one object from several threads is a fact only the library's documentation gives. A view of one reaching another thread otherwise is `E7015` ("`ctx` is a C object (`Ctx`, line 4), and two tasks read it at once (lines 14 and 15); C's `const` does not promise that `ctx_lookup` writes nothing: give each task a context of its own, or, if the library allows it, declare `unsafe extend Ctx implements ThreadShared:` with a `# SAFETY:` note"). A `handle` is an opaque `Copy` value with the C type's representation (a pointer or an integer: Vulkan's `VkBuffer` is a `uint64_t` on 32-bit targets), passed and returned by value and never read through, with no destructor of its own (`destroys(f)` applies to an `owns T` of its type, never to the handle), `nullable` as an `Option` of it (the null pointer or zero is `None`), and bound to its thread under `threads(creator)` (`[FFI-33b]`, `E7008`). A parameter or result of a handle type is no pointer contract: it is passed by value, so a count, lifetime or aliasing word or `borrowed` on it is `E5002` ("`device` is a `VkDevice`, a `handle` passed by value; remove `borrowed`"); it takes `nullable`, and the ownership words where the call transfers the handle's object, `returns_owned(destructor=f)` on a result and `owns` on a parameter, which give and take an `owns T` of the handle type (`[FFI-33b]`, two words under `threads(creator)`). A handle read out of an `owns T` (`dev` given where a `VkDevice` is taken, `copy of dev`) is a view of that owner for the lifetime rules (`[LT-1]`, `[LT-45]`): the owner's end destroys the object the handle names, so the handle lives no longer than the owner, a call that gives it reads the owner for the call, and keeping or returning it where it would outlive the owner is `E3063` ("`q` is `dev`'s handle (`VkDevice`, line 7), and `dev` ends when `open` returns, destroying the device; return `dev` itself, `-> owns VkDevice`, or use `q` while `dev` lives"); a handle no `owns T` gave (a result with no ownership word, the devices `vkEnumeratePhysicalDevices` fills in, a callback's argument) is the plain `Copy` value above, whose life the library's own documentation governs (`[TCB-1]`).}}
  - …whose fields are read only in [[`unsafe`]]
  + …whose fields are read only in {{`unsafe`; a field of a raw-pointer type or of a C function-pointer type (`[FFI-56]`) is read in safe code and written only in `unsafe:`, and so is the whole value of an aggregate that holds one, written into a place foreign code reads (through a `ref mut` to it, `mem.swap`, `mem.replace`) (`E3100`: "`second.data = first.data` (line 14) writes a raw pointer into a `blob`, which C reads later (`blob_free`, its destructor, `[FFI-36a]`); put line 14 in an `unsafe:` block with a `# SAFETY:` note saying what C may do with that address"), since an imported aggregate is memory foreign code reads, and what it does with the address nothing checks}}
  - …the typedef is, converted with `as` [[only]]
  + …the typedef is, converted with `as` {{only; one whose representation is a pointer is a raw pointer for `[THR-8]`, `[UNS-1]` and `[FFI-2]`'s limit, and `as` into it from a pointer or an integer is unsafe (`[TYP-7]`), so safe code never makes one}}
  + …static`; reading it needs `unsafe` unless {{the header declares the object `const`, a derived fact (`[FFI-2a]`), read as `[FFI-41]` reads a C++ one (its type `ThreadShared`, no pointer), or, on a hand declaration, which has no header to read,}}
  + …a safe read also requires `T: {{needs}}
  - …the C function reads is in [[C:]]
  + …the C function reads is in {{C (verdict 123):}}
  - …in C: "C passes `narrow` as [[an]]
  + …in C: "C passes `narrow` as {{a 4-byte}}
  - …`narrow` as an `int` through `...`; [[if]]
  + …`narrow` as an `int` through `...`; {{write the size}}
  - …an `int` through `...`; if `sum_wide` [[reads another type, convert it first: `v.sum_wide(1, narrow]]
  + …an `int` through `...`; if `sum_wide` {{expects: `narrow}}
  - …convert it first: `v.sum_wide(1, narrow as [[i64)`"]]
  + …convert it first: `v.sum_wide(1, narrow as {{c_int` for 4 bytes, or `narrow as i64` for 8"; for an `f32`, "C passes `single` as an 8-byte `double` through `...`; write `single as f64`"}}

### FFI-30c (p16-ffi.md; 13 words changed)
  + …replaces an earlier file's for that {{function (for a name declared several times, the member its words}}
  + …an earlier file's for that name, {{`[FFI-57]`),}}
  - …any other second entry for one [[name]]
  + …any other second entry for one {{function}}

### FFI-11 (p16-ffi.md; 219 words changed)
  - …| `borrowed` (valid for the call), [[`owned`]]
  + …| `borrowed` (valid for the call), {{`owns`}}
  - …for the call), `owned` (callee takes [[it),]]
  + …for the call), `owned` (callee takes {{it; verdict 102),}}
  - …only when it is handed over [[(`owned`)]]
  + …only when it is handed over {{(`owns`)}}
  + …parameter only reads what it views. {{A lifetime word asserts two things of the memory the result points into: it lives at least as long as the word says, and nothing writes it while the view may be used, neither this function on a later call nor any other; a result that holds only until the library's next call (C's `localtime`, `strerror`, `getenv`) has no lifetime word and its function stays `unsafe fn`, with an `extend` wrapper that copies the value inside `unsafe:` (`[FFI-13]`). `ember bind --report` lists each `from(static)` and `from(p)` result as the assertion it is (`[FFI-35a]`).}}
  - …contract words: `out` (an out-parameter, returned [[instead);]]
  + …contract words: `out` (an out-parameter, returned {{instead; it asserts that C writes the whole value before it returns, on success for a `-> status` function, `[FFI-16]`, an asserted fact `ember bind --report` lists beside the retention assumptions, `[FFI-35a]`; a function that may leave its out-parameter unwritten is declared `unsafe fn`, with `@safety` saying when the value is written);}}
  - …ok=X, error=N)` on the enum, `[FFI-16]`); [[`handle`;]]
  + …ok=X, error=N)` on the enum, `[FFI-16]`); {{`handle` (an opaque `Copy` value passed by value, taking no pointer words, `[FFI-8]`);}}
  - …then `unlisten(id)`: the token keeps them); [[`unsafe` (leave the call unsafe);]]
  - …Ember, `[FFI-52]`), and on a function-pointer [[typedef,]]
  + …Ember, `[FFI-52]`), and on a function-pointer {{typedef or beside `std_function` on a `std::function` type's entry,}}
  - …and on a function-pointer typedef, `safe` [[(`[FFI-56]`).]]
  + …and on a function-pointer typedef, `safe` {{(`[FFI-56]`, `[FFI-17a]`), within `[FFI-2]`'s limit on raw pointers.}}
  + …typedef, `safe` (`[FFI-56]`). `@ffi(effects=[…])` states effects. {{An overlay entry and a hand declaration leave their calls unsafe by being written `unsafe fn` (`[GRM-35]`, `[FFI-10]`, verdicts 130 and 133); `unsafe` is no contract word, in an overlay or in `@ffi(…)` (`E0104`).}}

### FFI-55 (p16-ffi.md; 533 words changed)
  - …* `[FFI-55]` [[**An]]
  + …* `[FFI-55]` {{**Every}}
  - …* `[FFI-55]` **An overlay [[names the header it was written for.** An overlay that another package applies (`[FFI-30c]`) records the hash of]]
  + …* `[FFI-55]` **An overlay {{is checked against}}
  + …the header it was written for, {{by a record Ember keeps** (verdict 121). A package with an overlay of a C or C++ header has a file `ember.overlays` beside its `ember.toml`, written by Ember and never by hand. For each overlay it holds}}
  + …header it was written for, the {{overlay's file, its header and}}
  + …it was written for, the one {{`blake3:` fingerprint of each declaration the overlay describes, as the importer binds it (`[FFI-14]`), keyed by the declaration's name and, for a member of a C++ name declared several times (`[FFI-57]`), by the declaration as the header spells it, quoted as TOML asks (`"gfx::Renderer::draw(const gfx::Mesh&)" = "blake3:…"`), one line per member, which a `rename` in the overlay does not change: the blake3 of the declaration as `[FFI-14]` records it, its name, its canonical type with every typedef resolved, its parameter names (`[FFI-12]`), and the size, alignment and field offsets of each complete aggregate it names (`[FFI-5]`), so that a renamed parameter or a changed layout is a changed declaration, and a comment, a macro the declaration does not use, or a C++ default argument is not: An overlay is applied only by an import of the header its first line names (`[GRM-35]`), the two spellings resolving to one file under the import's include paths (`[FFI-6]`), and the record's `header` is that name; an import of another header with it is `E5010`: "`overlays/sdl.em` is written for `SDL.h` (line 1), and this import is of `SDL2/SDL_video.h`; import `SDL.h` here, or write an overlay for `SDL2/SDL_video.h`". ```text # ember.overlays: the headers this package's overlays were checked against. # Written by ember; `ember bind <header> --check <overlay> --merge` rewrites a line. [[overlay]] file = "overlays/sdl.em" header = "SDL.h" SDL_FreeSurface = "blake3:…" ``` Ember writes the record the first time the package's own build applies the overlay, and only `ember bind <header> --check <overlay> --merge`, run once a person has checked the overlay against the header, rewrites a line of it (`[CLI-7]`); a declaration the overlay describes that the record lacks, an entry just written, is added by the package's own build. Every build that applies an overlay,}}
  - …for, the one its own package's [[`.embind` holds for]]
  + …for, the one its own package's {{and any other's, compares each declaration the overlay describes with the header it imports. One}}
  - …own package's `.embind` holds for that [[header (`[FFI-14]`), on its first line: `unsafe overlay c "lib.h" with (header_hash = "blake3:9f2c…"):`. Applying another package's overlay to a header whose hash differs, or applying one that records none,]]
  + …own package's `.embind` holds for that {{differs}}
  - …is `E5010`, naming the overlay, the [[header and both hashes, with the two ways out: import]]
  + …is `E5010`, naming the overlay, the {{declaration as}}
  + …two ways out: import the header {{imported declares it, and the way out ("`SDL_FreeSurface` changed since `overlays/sdl.em` was checked: the `SDL.h` imported here declares `void SDL_FreeSurface(SDL_Surface*, int)`; check}}
  - …out: import the header the overlay [[was written for,]]
  + …out: import the header the overlay {{against this header, then run `ember bind SDL.h --check overlays/sdl.em --merge`"); in another package's build, an overlay}}
  - …the overlay was written for, or [[write]]
  + …the overlay was written for, or {{a declaration its package's record lacks is `E5010` too. A change to a declaration}}
  - …was written for, or write the [[package's own]]
  - …or write the package's own overlay [[for]]
  + …or write the package's own overlay {{does not describe changes nothing. Every program has a manifest, a single file with none being given a default one by its first compile (`[MAN-9]`), so every program keeps}}
  - …the package's own overlay for the [[header it imports.]]
  + …the package's own overlay for the {{record beside it.}}
  - …the header it imports. An overlay [[applied to a header]]
  + …the header it imports. An overlay {{in no package's folder is the importing program's own: its record is that package's, naming the overlay by its path from the package (`../vendor/sdl.em`). Nothing}}
  - …overlay applied to a header of [[its own package needs no hash: its package vouches for it (`[TIER-1]`).]]
  + …overlay applied to a header of {{this is written in the overlay or in the program.}}

### FFI-36a (p16-ffi.md; 76 words changed)
  - …of `T`'s declaration, `@ffi(destroys(f))` on `type [[T`;]]
  + …of `T`'s declaration, `@ffi(destroys(f))` on `type {{T`, or, for an imported C++ class with no such entry, the class's own destructor and `delete` (`[FFI-36]`);}}
  + …directly and no `adopt` is written. {{A foreign type has one destructor: every `returns_owned(destructor=f)` giving an `owns T` and `T`'s `@ffi(destroys(f))` name the same `f`, and two different ones are `E5002` ("`Widget` is ended by `widget_free` (line 3), and line 12 says `widget_destroy`; a foreign type has one destructor"), so an `owns T` foreign value stays one word, two for a `threads(creator)` type (`[FFI-33b]`).}}

### FFI-21 (p16-ffi.md; 586 words changed)
  + …or an `extern "C" fn` value. {{A function-pointer parameter of a `safe fn` or of an overlay entry carries a `callback(…)` word, or the declaration is `E5002` ("`set_timer`'s `cb` is a function pointer, and nothing says whether C keeps it or which thread runs it; write `param(cb, callback(retained, threads(any)))`, or `callback(borrowed)` if C runs it only during the call"); an `unsafe fn` may take one with no word. A function coerced to `extern "C" fn(…)` anywhere gets a C entry that attaches the calling thread (`[FFI-22]`) and then runs it, and its body is checked as an export under `threads=any` is (`[FN-6]`, `[FFI-33]`, `[THR-20]`: `E7012` for a `static mut` it reaches without its lock), since the compiler cannot see which thread C calls it from; `@ffi(no_callbacks)` on a function whose contract takes a function pointer is `E5069`, as for a `callback(…)` parameter. The callable type of a `callback(…)` parameter is `fn(P…) -> R`: the C function pointer's parameters mapped by `[FFI-8]` with the user-data one removed, a raw pointer among them staying raw (a lambda's body uses it in `unsafe:`), while a raw pointer as `R` is what the function returns to C, which `[FFI-2]`'s limit refuses on a `safe fn` or a plain overlay entry (`E5002`, naming the parameter's result); under `callback(retained)` or `callback(once)` C keeps the function, so the parameter is an `owns` parameter, `owns cb: fn(P…) -> R`, and its argument says `owns` at the call as every argument for an `owns` parameter does (`[FN-2a]`, verdicts 147 and 158), a function written in the call (`register_cb(owns fn() owns game: …)`) and a named one (`register_cb(owns add)`, after which `add` is handed over, `E3040` at a later use) alike, `E3028` without it; under `callback(borrowed)` the parameter is plain and the function is lent for the call with no word, as to any call that runs it then; `thread_shared fn(P…) -> R` under `threads(any)` (`[FFI-22]`), and `once fn(P…) -> R` under `callback(once)`; a function given to it is written as any function value is (`fn() owns game: …`).}}
  - …`shares mut` and `weakref` only under [[`threads(creator)`]]
  + …`shares mut` and `weakref` only under {{`threads(creator)`,}}
  - …and `weakref` only under `threads(creator)` or [[`threads(main)`,]]
  + …and `weakref` only under `threads(creator)` or {{under `threads(main)` where the registering call is in a function only the main thread runs, decided as `[THR-20]` decides it (not reachable from `thread.spawn`, from a job submitted outside a scope, from a callback C may run on another thread, or from an export under `threads=any` or `creator`), since elsewhere the owner reaches another thread (`[THR-2]`, `E7015`: "C runs this function on the main thread (`on_main`'s `cb` says `threads(main)`), and `worker` may run on another thread (`thread.spawn`, line 18), so `log`, a `shares mut` owner of that thread, would be used by two threads; share the log between threads: write `log = thread_shared Log()` in place of line 12, with `lines` `guarded by` a lock, or register the callback from `main`");}}
  - …say what it holds: write `return [[register_cb(fn()]]
  + …say what it holds: write `return {{register_cb(owns fn()}}
  - …that lets the game end)". `retained` [[returns]]
  + …that lets the game end)". `retained` {{and `once` return}}
  - …end)". `retained` returns a `Retained[ForeignCallback]` token [[(`[FFI-23]`)]]
  + …end)". `retained` returns a `Retained[ForeignCallback]` token {{(`[FFI-23]`), the call's result (where the C function has a result of its own, the Ember function returns both as `[FFI-11b]` orders, that result first, then the token, or under `-> status` the token as the `Result`'s value, `[FFI-16]`; the token keeps the result for `destroys(f)`, `[FFI-11]`),}}
  - …declared `destroys(f)` (`E5041` if none is [[declared), and]]
  + …declared `destroys(f)` (`E5041` if none is {{declared, for `once` too, since a token that ends before C runs the function must unregister it);}}
  - …ends the block after the first [[call.]]
  + …ends the block after the first {{call, and the token, which keeps the block until C has run the function or the token ends, whichever is first, records that the call ran: a token ending before the call unregisters the callback through `destroys(f)` and ends the block, and one ending after it calls nothing, since C gave the registration up by running it, and frees the slot's record.}}

### FFI-22 (p16-ffi.md; 2 words changed)
  - …thread_shared Game) -> Retained[ForeignCallback]:` and `return [[register_cb(fn()]]
  + …thread_shared Game) -> Retained[ForeignCallback]:` and `return {{register_cb(owns fn()}}

### FFI-52 (p16-ffi.md; 67 words changed)
  - …package counts them at every foreign [[call.]]
  + …package counts them at every foreign {{call; and a package's own foreign call across which it uses a view parameter is an open line of its function's summary, resolved by each caller from another package against the callbacks that caller's program keeps (`[BRW-21]`, `E3031`), so nothing in a library is refused for callbacks it cannot see.}}
  + …this.score += 1 unsafe extern "C": {{@safety("`cb` and `ud` are a pair `register_cb` was given") unsafe}}
  - …callback(retained, user_data=ud), threads(creator)), destroys(unregister)) safe fn [[register_cb(cb: extern "C" fn(*mut void),]]
  + …callback(retained, user_data=ud), threads(creator)), destroys(unregister)) safe fn {{register_cb(owns cb: fn(),}}
  - …shares mut Game) -> Retained[ForeignCallback]: return [[register_cb(fn()]]
  + …shares mut Game) -> Retained[ForeignCallback]: return {{register_cb(owns fn()}}

### FFI-53 (p16-ffi.md; 260 words changed)
  - …generation in the top 16, which [[are never all zero,]]
  + …generation in the top 16, which {{always has its bits 0, 7 and 15 set (bits 48, 55 and 63 of the number; 8,192 values before the slot retires): with bit 63 set and bits 56 to 62 clear the number is non-canonical on x86-64 under 48- and 57-bit addressing, and with bits 48 and 55 set it stays non-canonical on AArch64 under top-byte-ignore with 48- or 52-bit addresses,}}
  + …through it faults before writing anything. {{On a 32-bit target the slot is the low 16 bits and the generation the top 16, and the number may be a user-space address.}}
  - …a `callback(once)` slot carries a used-up [[mark.]]
  + …a `callback(once)` slot carries a used-up {{mark, taken with one atomic exchange before any of the callback's state is read, so of two threads calling it at once one runs it and the other stops the program with the second-call text, and the block is ended when the call that ran it returns.}}
  + …compare per registration, nothing per call. {{The table grows by chunks and never moves a slot, as an `Arena` grows (`[ARN-4]`), so a slot is free while memory is and the slot field is wide; when every slot the number can name is live or retired, which only a 32-bit target's 65,536 slots can reach, the registering call stops the program, in every profile, as a full `Pool`'s `insert` does (`[HND-3]`), the failure `Panic(Explicit)` lists (`[EFF-16]`), on every target, so that a program's text means one thing wherever it is built (a `@nopanic(explicit)` function that registers a kept callback is `E4040`, as one that allocates is, `[ALC-6]`): "`on_click(...)` on line 16: no callback slot is free; every one of the 65,536 slots a 32-bit target has is live or retired" (`[COST-6]`).}}

### FFI-23 (p16-ffi.md; 99 words changed)
  + …callback that ends its own token). {{A token is bound to the thread that registered the callback, whatever the callback's contract (`[THR-8]`: neither `Send` nor `ThreadShared`; handing it to another thread is `E7008`: "`token` keeps the callback registered on line 13 for C, and its end unregisters the callback on the thread that holds the token; keep `token` in `main`, or end it there before the thread starts: `mem.drop(owns token)` above line 14"), so its end, the `destroys(f)` call and the block's end, runs on that thread, and a one-thread owner the block holds under `threads(creator)` or `threads(main)` is counted on one thread only.}}
  - …to a parameter whose contract says [[`owned`]]
  + …to a parameter whose contract says {{`owns`}}

### FFI-54 (p16-ffi.md; 437 words changed)
  - …declared as keeping an object (`@ffi(param(widget, [[owned,]]
  + …declared as keeping an object (`@ffi(param(widget, {{owns,}}
  - …given one by an ordinary hand-over, [[`widget_take(first)`:]]
  + …given one by an ordinary hand-over, {{`widget_take(owns first)`:}}
  - …the name ends and Ember frees [[nothing, with no word.]]
  + …the name ends and Ember frees {{nothing.}}
  - …cannot say so (a plain pointer, [[`fn]]
  + …cannot say so (a plain pointer, {{`unsafe fn}}
  + …value's clean-up (`mem.forget` is not Ember). {{It is written on a value whose block the runtime laid out or C gave (an `owns T` of a foreign type, a value made in the built-in manager: a place with no `using` is a `using Global` place, `[TYP-46]`), which `ember_rt_object_release` can end; on a value Ember made `using` another manager it is `E2114` ("`w` was made in `frame`, an `Arena`, and C would end it through the runtime, which gives back only blocks of the built-in manager; make it there: `second = owns Widget(id=2)`"), and `from_raw` gives back a value of the built-in manager, as its type says.}}
  + …and indent line 18 under it". {{On an object of a trampoline class `into_raw()` gives the C++ base subobject's address, which a C++ raw-pointer parameter expects (`[FFI-39]`). `std.ffi.from_raw[C](p: ObjectPtr[C]) -> C`, and `from_raw[T](p: *mut T) -> owns T` for an `owns T` given so, is an `unsafe fn` (`@safety("`p` is an address `into_raw()` gave, taken back once")`) that makes the value owned again, by one name, so its end frees it as before; a class object comes back with one owner, so a `weakref` link made before `into_raw()` still finds it alive (`[UNS-4]`); taking one address back twice is caught as adopting twice is (`[FFI-36b]`: at run time in every profile, and `E5068` on one local). A C host ends an object given with `into_raw()` with `ember_rt_object_release(void*)` (`[FFI-27]`), on an attached thread, which does what its last owner's end would (`[CLS-6]`: the block is freed only when no `weakref` link remains). An object whose type is not `Send` is ended, and taken back with `from_raw`, on the thread that made it, and that is checked: `into_raw()` records the thread, one entry in a table of the runtime that is removed when C ends the object or Ember takes it back, and `ember_rt_object_release` and `from_raw` compare it with the calling thread, in every profile, and stop the program otherwise ("C ended a `Widget` made on another thread; `Widget` is not `Send`: its `log` is a `shares` owner"), as a `threads(creator)` trampoline does (`[FFI-17d]`, `[FFI-22]`). When that thread has ended, the objects of it that C still holds can only be ended, one at a time under the runtime's lock, and `from_raw` of one stops the program. An object of a `Send` type is ended or taken back on any attached thread (§X.4). An `ObjectPtr[C]` is a raw pointer for `[THR-8]`, `[THR-9]` and `[UNS-1]`: thread-bound, and read through only in unsafe code.}}
  + …unsafe extern "C": @ffi(destroys(widget_free)) type Widget {{@safety("`widget` is a widget C made, not freed yet") unsafe}}
  - …widget_new(id: i32) -> owns Widget @ffi(param(widget, [[owned,]]
  + …widget_new(id: i32) -> owns Widget @ffi(param(widget, {{owns,}}
  + …safe fn widget_take(owns widget: owns Widget) {{@safety("`p` is a widget whose ownership C takes") unsafe}}
  - …w.into_raw() fn main(): first = widget_new(1) [[widget_take(first)]]
  + …w.into_raw() fn main(): first = widget_new(1) {{widget_take(owns first)}}
  - …widget_take(first) second = widget_new(2) p = [[address_for_c(second)]]
  + …widget_take(first) second = widget_new(2) p = {{address_for_c(owns second)}}

### FFI-56 (p16-ffi.md; 91 words changed)
  + …argument C passes to a callback {{or to an export, itself or in a `@layout(c)` value (`[FFI-31b]`)}}
  - …`unsafe extern "C" fn(…)` (`Option[…]` when [[nullable, `[FN-6]`),]]
  + …`unsafe extern "C" fn(…)` (`Option[…]` when {{its slot's contract says `nullable`, `[FFI-11]`; a field of an imported struct, which no contract reaches (`[FFI-30a]`), is always the `Option`, since C may leave it null and safe code must be able to read what C wrote),}}
  - …a scalar or a `@layout(c)` value [[type]]
  + …a scalar or a `@layout(c)` value {{type, within `[FFI-2]`'s limit on raw pointers}}
  - …or a `@layout(c)` value type (`E5002` [[otherwise):]]
  + …or a `@layout(c)` value type (`E5002` {{otherwise, naming where the pointer is held):}}
  - …an asserted fact, listed by `ember [[tcb`.]]
  + …an asserted fact, listed by `ember {{tcb`; a `std::function` value is unsafe to call for the same reason, and `safe` beside `std_function` on its type's entry asserts it so under the same limit (`[FFI-17a]`).}}

### FFI-33b (p16-ffi.md; 52 words changed)
  - …foreign value of such a type [[records]]
  + …foreign value of such a type {{is two words, the address and the id of the thread that made it, and}}
  - …of such a type records its [[creating thread and,]]
  + …of such a type records its {{end is one compare of that id with the current thread's,}}
  - …creating thread and, in every profile, [[stops]]
  + …creating thread and, in every profile, {{stopping}}
  - …line that made it and both [[threads]]
  + …line that made it and both {{threads; safe code never hands such a value to another thread (`[THR-8]`, `E7008`), so the stop backs unsafe code and C, which may move it}}

### FFI-31b (p16-ffi.md; 472 words changed)
  - …header (`[BLD-FFI-5a]`). `@layout(c)` values, scalars, opaque [[handles,]]
  + …header (`[BLD-FFI-5a]`). `@layout(c)` values, scalars, opaque {{handles and}}
  + …`@layout(c)` values, scalars, opaque handles, `CStrView` {{may,}}
  + …values, scalars, opaque handles, `CStrView` and {{so may an}}
  - …handles, `CStrView` and `extern "C" fn` [[may.]]
  + …handles, `CStrView` and `extern "C" fn` {{that Ember hands C: a result, or a field of an `@export_table` (`[FFI-26]`; a function Ember made, `[FN-6]`). A function pointer C hands an export — a parameter of it, or a field, at any depth, of a `@layout(c)` value a parameter passes, by value, behind `ref mut` or as a span's element — is one nothing describes (`[FFI-56]`), so its type is `unsafe extern "C" fn(…)`, called inside `unsafe:` with a `# SAFETY:` note; a raw pointer in an export's result, or in a `@layout(c)` value it returns, is given to C out of an `unsafe:` block with a `# SAFETY:` note (`[UNS-1]`, `E3100` otherwise), and one in an export's parameter is read through only in unsafe code, as any raw pointer is; a safe `extern "C" fn` there is `E5015`: "`fill` is a function pointer the host supplies, and nothing says what the function does with its arguments (`[FFI-56]`); write `fill: unsafe extern "C" fn(*mut u8)` and call it inside `unsafe:` with a `# SAFETY:` note".}}
  + …a value that is not one". {{An export's parameters and result are written in the safe types of `[FFI-10]`'s mapping and lowered to the same carriers: a plain `x: T` of a `@layout(c)` type as `const T*` (a scalar by value), `ref mut x: T` as `T*`, `Span[T]` and `MutSpan[T]` as a pointer and a `size_t`, an `Option` of a pointer-shaped type as a null pointer for `None`, `CStrView` as `const char*`, and a `@repr` enum as its `@repr` type; such an enum is `@non_exhaustive`, or the export is `E5015` ("`Mode` has two values and C may pass any `int32_t`; write `@non_exhaustive` above `enum Mode`, and give `match m:` an `else:` arm"). `ember build --emit-header` writes above each prototype what the caller must make true, `[UNS-4]`'s words per parameter: a pointer valid for the call, a `T*` reached by no other argument, a `const char*` NUL-terminated, a count that fits its pointer; a C caller that breaks one is outside the contract, as a C library is. So a `Span[T]` or `MutSpan[T]` whose `T` is a scalar or a `@layout(c)` value type may appear in an exported signature, and so may `ref mut x: T` for such a `T`: that the count is right, the pointer is not null and the memory lives and is not read or changed by C for the call is C's side of the export's contract, an asserted fact `ember tcb` lists (`[TCB-1]`), and the header writes each with that contract as a comment. A `StrView` may not: C would hand bytes to it unvalidated (`E5063`: "`s` would be text C never validated; take `s: Span[u8]` and write `s.to_str()?` for the `StrView` (`[TXT-2]`)"); neither may a `ref T`, a `MutSpan` of any other `T`, a view type, or a result that is a view (`E5015`), whose lifetime nothing states.}}

### FFI-33 (p16-ffi.md; 62 words changed)
  - …the `Atomic`'s). The same holds under [[`creator`.]]
  + …the `Atomic`'s). The same holds under {{`creator`: `creator` and `main` name the same thread, and under `main` (`[FFI-33c]`) the body may also use `threads(creator)` values, while under `creator` it may not reach a `static mut` without its lock, because `creator` is also the word of a callback's and a handle's contract and says only that the caller is the maker, not that the module's state is its own.}}

### BLD-FFI-3 (p16-ffi.md; 414 words changed)
  - …Existing libraries are linked by name [[(`link]]
  + …Existing libraries are linked by name {{or path, each named in the manifest under `[link] unsafe_libs` (`unsafe_libs}}
  - …are linked by name (`link = [[["vulkan-1"]`) or path;]]
  + …are linked by name (`link = {{["vulkan-1"]`, `[MAN-8]`) and nowhere else, so that the manifest says the program uses libraries nothing checks, as the import of each says in the code (`unsafe import`, `[FFI-51]`; verdict 172), a library a header itself asks for included (`#pragma comment(lib, "uuid.lib")`; verdict 173);}}
  + …path; link order follows declaration order. {{A library only one system has is named for that system (verdict 174): `[link.windows]`, `[link.linux]` and `[link.macos]` each take `unsafe_libs`, linked after `[link]`'s own when the build is for that system (the system of `--target` or `[build] target`, the host's when neither is written) and not looked for when it is for another, so `unsafe_libs = ["libcpmt", "uuid"]` under `[link.windows]` leaves a Linux build as it was; a library named in both lists is linked once, and the import of its header is written `unsafe import` on every system, one line of source serving them all. The key `libs` is not Ember (`E9001`: "`libs = [\"vulkan-1\"]` links code that nothing checks; write `unsafe_libs = [\"vulkan-1\"]`"). Besides the libraries the manifest names, a build links only the runtime's own: the C runtime and the system libraries `ember_rt` needs, which `[RT-1]` lists for each system and `ember tcb` lists under the runtime. Every other library the link would add because something asked for it is named under `unsafe_libs` too, whether a header asked (verdict 173) or an object of a named library did (`/DEFAULTLIB` in a COFF `.drectve` section, `.deplibs` in ELF, `LC_LINKER_OPTION` in Mach-O); otherwise it is `E9001`, naming the library, what asked for it and the line to add. Shims and thunk units are compiled without the C compiler's own default-library names (`/Zl`), so a request left in one is the header's. What a shared library loads when it runs is that library's code, vouched for with it, and `ember tcb` lists it under it. A library named under `unsafe_libs` that nothing uses is `E9001` too (verdict 175): one of which no import and no `unsafe extern` block of the package declares a name, and which no header and no named library asks for ("`unsafe_libs` names `foo` (`ember.toml`, line 7), which nothing in this package imports, and its start-up code would still run when the program loads; remove the line, or import its header with `unsafe import`"); it is judged by what is declared, not by what is called, so a library used only for what it does when it loads stays possible, with the code saying so.}}
  + …link order follows declaration order. Each {{named library}}

### HR-2 (p20-annex-b.md; 47 words changed)
  + …naming the cause. Commit performs only {{stores:}}
  - …pointer stores into memory reserved in [[prepare:]]
  + …pointer stores into memory reserved in {{prepare, and, for an instance migrated in place (`[HR-15]`), the copy of its prepared bytes over its old block, of a size prepare computed and into memory that exists, made before the map is applied so that no rewritten pointer is followed into an old layout;}}

### HR-2a (p20-annex-b.md; 82 words changed)
  - …and runs no user code. Prepare [[allocates]]
  + …and runs no user code. Prepare {{builds}}
  - …no user code. Prepare allocates every [[new]]
  + …no user code. Prepare allocates every {{migrated}}
  + …instance in the transaction arena (`[HR-37]`), {{an in-place one too, copies into the arena the old values of the fields that will not be carried over by name (removed fields, the old value of each converted field, and every field of an instance rebuilt by `migrate_from`),}}
  - …instance, and computes the old-to-new address [[map.]]
  + …instance, and computes the old-to-new address {{map; it writes nothing into an old block.}}
  - …map and repoints the thunks. Reclaim [[frees each migrated instance's]]
  + …map and repoints the thunks. Reclaim {{ends those saved}}
  - …Reclaim frees each migrated instance's old [[version field by field without]]
  + …Reclaim frees each migrated instance's old {{values, running their `drop`s and never}}
  - …instance lives on in its new [[form,]]
  + …instance lives on in its new {{form (a field carried over by name has been handed over}}
  - …on in its new form, and [[ends removed fields, running their `drop`.]]
  + …on in its new form, and {{is not ended), then frees each old block whose instance moved.}}

### HR-3b (p20-annex-b.md; 91 words changed)
  + …depends on once the call returns: {{those a value it still uses is of, holds, or views (a view into a field, an element or a part depends on the type that holds the part and on every type on the path to it, through links and `owns` blocks alike, read from the places the view carries, `[BCK-1]`);}}
  - …instances migrate as at a poll [[(`[HR-14]`), and one]]
  + …instances migrate as at a poll {{(`[HR-14]`); an instance}}
  - …one a frame's view points into [[keeps]]
  + …one a frame's view points into {{is not changed in place while that frame lives: a layout change of}}
  - …frame's view points into keeps its [[address, which `[HR-15]` allows when]]
  + …frame's view points into keeps its {{type is a dependence of the frame, and is refused naming the function, the line of}}
  - …address, which `[HR-15]` allows when its [[new size fits;]]
  + …address, which `[HR-15]` allows when its {{call and the view, and}}
  - …fits; one that must move is [[refused,]]
  + …fits; one that must move is {{refused too,}}

### HR-44 (p20-annex-b.md; 189 words changed)
  - …function, as a package's interface file [[does.]]
  + …function, as a package's interface file {{does, and with them the proofs that read through a reloadable callee (a `@parallel` loop's or a vectorised loop's, `[PAR-2b]`, `[SIMD-5]`) and the lock each field is `guarded by` (`[HR-11]`), so that a reload that changes the callee so the proof no longer holds, or changes a field's lock while code the reload does not rebuild uses the field, is refused in plan (`E9030`), naming the loop and the callee, or the field, its new lock and the code that uses it.}}
  + …computed again on the next call. {{Whether a type is plain (`[RC-4]` point 4) is such a fact for every line that makes a `thread_shared` value of it; when a reload takes it away, prepare, which walks the live instances while every registered thread is at depth zero (`[HR-3]`, `[HR-12]`), hands over every live `thread_shared` value of the type as `[RC-4]` point 4 does (its home count moved into its away count, the handed-over mark set), and ends one it finds queued whose two counts add up to zero with the old image's `drop`, which is plain, so none waits in a queue whose check points may no longer run its `drop`.}}
  + …again on the next call. Nothing {{else}}

### HR-11 (p20-annex-b.md; 16 words changed)
  - …`sharedref mut`, `ref`, `ref mut`, `using [[A`),]]
  + …`sharedref mut`, `ref`, `ref mut`, `using {{A`) and the lock it is `guarded by` (none, or the lock field's name),}}
  - …one is a change of the [[field's type.]]
  + …one is a change of the {{field.}}

### HR-12 (p20-annex-b.md; 249 words changed)
  + …reach); non-reloadable builds keep `[OBJ-1]`'s layout. {{A `thread_shared` value's 32-byte header (`[OBJ-1]`) is 48 bytes in a reloadable build. Registering links the object into a list through those 16 bytes: four stores when it is made (two of them into its own header) and two when it ends. An object whose own type is not `Send` never leaves the thread that made it (`[THR-8]`), so it is linked into that thread's list with plain stores, which touch nothing another thread touches (`[RT-11]`); the runtime keeps a thread's list when the thread ends, for what unsafe code gave to C from it (`[FFI-54]`). An object whose own type is `Send` may be ended by another thread than the one that made it (a task's result, a value sent down a channel or handed to a spawned thread), and that thread would unlink it from a list its maker is changing, so its registration, like a `thread_shared` object's, is ordered by the runtime, one lock or one atomic exchange, on whichever thread makes or ends it. Which of the two an object takes is read from its own type, where it is made and where it ends, never from the place that holds it (`[BLD-16]`). Plan and prepare walk the list only while every other registered thread is at depth zero (`[HR-3]`). A value made shared in a block of its own (`[RC-1]`) is registered as a class instance is, with its type's schema, and migrated once, whatever the number of owners and links that reach it.}}

### HR-14 (p20-annex-b.md; 89 words changed)
  + …| old value ended in reclaim {{| | a kept view points into a field that is removed, converted or rebuilt | refused (`[HR-15c]`) | | a lambda's captures changed while a state of it lives | the field rows, applied to the captures (`[HR-48]`)}}
  + …| base class changed | refused {{| | the promise an `open` or `abstract` class's header makes (`implements Send`, `implements ThreadShared`, an interface that needs them, `[THR-8]`) added or removed | refused (`E9030`) unless the image rebuilds every package that imports the class, whose derived classes and hand-overs are then checked again when they compile (`[HR-44]`)}}

### HR-16 (p20-annex-b.md; 148 words changed)
  + …read-only view of the old instance. {{`Old<Name>`'s fields have the types of the previous schema: a field whose type is itself migrated in this reload is of that type's `Old` form (`log: shares OldLog`; a container field `parts: Array[Part]` is `Array[OldPart]`, read-only, its elements `OldPart` values), through links, `owns` blocks, containers and tuples alike, so that nothing read through `old` is read with a new layout; a field whose type is unchanged keeps its type. An `Old` type has its fields and no methods. A `shares`, `shares mut`, `thread_shared` or `weakref` link read from `old` may be stored into the new instance's field of the new type (`log=shares old.log`): it names the old instance until commit, and commit's map (`[HR-2a]`) repoints it to the migrated one; so `migrate_from` never reads the new instance of another type, and never stores a view of an `Old` value (`E3063`, since the old block is freed in reclaim).}}

### HR-35 (p20-annex-b.md; 202 words changed)
  - …count at its maximum, a stale [[handle]]
  + …count at its maximum, a stale {{handle, a `runtime` name's test (`[LT-48]`)}}
  + …message))` instead of stopping the program. {{The rule covers what `migrate_from` runs, not only what it writes: every function it calls, directly or through what it calls, and every `drop` it may run, is compiled for that use as an instance of itself under this rule (`[MNG-6]`: `[reload]` among its arguments), with the same `E2225` for an explicit panic and the same `Err` returns for a check; a call through a function value, an interface, `dyn` or `virtual` method, in `migrate_from` or in what it calls, reaches a body compiled to stop the program, and is `E2225` ("`sum_all` calls `parts.reduce(f)` on line 9 through a function value, which may stop the program during a reload; call the function by its name, or write `@allow_reload_terminate` on `migrate_from`"), which `[HR-43]` admits as it admits a `noexcept` call; a call to an `extern` function follows `[HR-39]`, whose refusal is `E2225` ("`finish` is asserted `noexcept` and may end the process; call it outside the migration, or write `@allow_reload_terminate`"). A field default or `Default.default()` that prepare runs for an added field (`[HR-14]`) is compiled the same way, and an explicit panic in it makes the row `field added, neither` (refused, naming the field and the panic's line).}}

### HR-41 (p20-annex-b.md; 63 words changed)
  + …changed | plan | refused (`[HR-45]`) {{| | a live function value's captures or a captured type changed | plan | refused (`[HR-48]`) | | a stored function value or kept callback the new image does not count | plan | refused (`[HR-44a]`) | | an object in use by an in-flight GPU frame must move | plan, after at most `frames_in_flight + 1` polls | refused, `Timeout` (`[HR-24]`)}}

### BLD-FFI-5a (p21-annex-c.md; 69 words changed)
  - …same name with the class's `pub` [[methods,]]
  + …same name with the class's `pub` {{methods a `thread_shared` name may call in Ember (`[THR-1]`: those whose summary changes the object only through its `Atomic`, lock, channel and `guarded by` fields), the others left out with a comment naming the rule, each of them an export under `threads=any` (`[FFI-33]`) whose wrapper attaches the calling thread (`[FFI-22]`), whose body is checked as such an export's is, and whose parameters and result follow `[FFI-31b]`;}}
  - …one more owner on the value's [[atomic]]
  + …one more owner on the value's {{away}}
  - …owner on the value's atomic count [[(`[RT-8]`)]]
  + …owner on the value's atomic count {{(`[RC-4]`, `[RT-8]`)}}

### FFI-17a (p21-annex-c.md; 274 words changed)
  - …`ref mut [T; N]`, `ref [T; [[N]`]]
  + …`ref mut [T; N]`, `ref [T; {{N]`; a result needs a lifetime word (`[FFI-2a]`)}}
  - …is the fallible conversion to `StrView` [[(`[TXT-2]`)]]
  + …is the fallible conversion to `StrView` {{(`[TXT-2]`); a result needs a lifetime word (`[FFI-2a]`)}}
  - …for a `T` that maps by [[value]]
  + …for a `T` that maps by {{value; the thunk converts at the call, in both directions (`has_value()` and the value: one branch and one copy of `T`), since the two layouts differ, as it does for a `std::variant`'s generated enum, a conversion `ember inspect` reports (`[FFI-17f]`)}}
  + …its overlay entry, a function value {{which owns its target, a foreign function: of type `unsafe fn(A…) -> R` (`[FN-6]`), since nothing tells the compiler what the target does with its arguments, as for a C function pointer (`[FFI-56]`; a call of it outside unsafe code is `E3100`: "`fill` holds a `std::function` from `fill.hpp` whose target nothing describes; put line 6 in an `unsafe:` block with a `# SAFETY:` note saying why the call is safe", and, only where `[FFI-2]`'s limit allows it (`[DIA-25]`), "or, if a call of it within its signature cannot break memory safety, write `@ffi(std_function, safe)` on `type Fill` in `overlays/fill.em`"), or}}
  - …value of type `fn(A…) -> R` [[(`[CLO-3]`), called]]
  + …value of type `fn(A…) -> R` {{(`[CLO-3]`) with `@ffi(std_function, safe)` on the entry, which asserts the target safe to call within `[FFI-2]`'s limit on raw pointers (`E5002` otherwise, naming where the pointer is held and giving the entry without `safe`), an asserted fact `ember tcb` lists (`[TCB-1]`, `[TCB-4]`); for `[EFF-2]` and `[CLO-17]` that function is one the program can put in every place the value may reach (recorded across a package's edge}}
  - …`fn(A…) -> R` (`[CLO-3]`), called as [[any other, which owns]]
  + …`fn(A…) -> R` (`[CLO-3]`), called as {{the function values a package stores, `[EFF-4]`), with every effect (`[EFF-3]`) and the callbacks, exports and overrides a foreign call may run (`[FFI-52]`); `@ffi(no_callbacks)` beside `std_function` on the type's entry says it runs none; the value is neither `Send` nor `ThreadShared` (`[FFI-17e]`), and}}
  - …as any other, which owns its [[target;]]
  + …as any other, which owns its {{end runs the `std::function`'s destructor through a thunk (`[FFI-17f]`);}}

### FFI-17e (p21-annex-c.md; 10 words changed)
  - …and `CppShared[T]` are declared in `std.ffi`; [[they]]
  + …and `CppShared[T]` are declared in `std.ffi`; {{they, and the function value a `std_function` import makes (`[FFI-17a]`),}}

### FFI-36 (p21-annex-c.md; 99 words changed)
  - …`owns T` ended through the contract's [[destructor.]]
  + …`owns T` ended through the contract's {{destructor: `@ffi(destroys(f))` on the class's overlay entry where one is written, else, for an imported C++ class, its own destructor and deallocation, `delete` in a thunk the importer generates (`[FFI-17]`), as a `std::unique_ptr<T>` with the default deleter ends it (`[FFI-17a]`); that the pointer is to an object made with `new` that nothing else ends is what the `adopt`'s `@safety` asks of its caller (`[UNS-7]`). A class with no public destructor is refused (`E5002`: "`Enemy` has no public destructor, so nothing Ember can call ends an adopted one; name the function that does in the overlay: `@ffi(destroys(destroy_enemy))` above `type Enemy`").}}

### FFI-39 (p21-annex-c.md; 292 words changed)
  + …`fn init(…)` calling `super.init(…)` exactly once. {{An `override fn` matching no virtual method of the base, by name and first line — each parameter's word and type exactly those the import gives the virtual method's (`[FFI-40]`), and its result; an upcast or a widening counts for nothing, since C++ hands the override its own argument (a `FocusEvent` for an `on(const Event&)`) — is `E5056`, whose message gives each virtual method of that name as the `override fn` line to write ("`on(e: gui.ui.KeyEvent)` overrides no virtual method of `ui.Handler`: it declares `on(e: gui.ui.Event)` and `on(timer_id: c_int)`; write `override fn on(e: gui.ui.Event):` or `override fn on(timer_id: c_int):`"); where the base declares the name several times (`[FFI-57]`), the override replaces the one member it matches and the trampoline passes on that one, the others staying the base's. An `override fn` whose result is a raw pointer, or holds one, returns it out of an `unsafe:` block with a `# SAFETY:` note, since C++ reads through it (`[UNS-1]`, `E3100` otherwise). An object of such a class is one block: the header (`[OBJ-1]`), then the C++ base subobject (the trampoline's `this`, at a fixed offset every upcast adds), then the Ember fields. The trampoline subclass declares a virtual destructor that first marks the object ended through `ember_rt` (`[WK-3]`), then runs the Ember `drop`, ends the Ember fields and runs the base destructor (`[FFI-17c]`); and an `operator delete` that returns the block to `ember_rt`, which frees it only when no `weakref` link remains (`[CLS-6]`), subtracting the base subobject's fixed offset to reach the header; so C++'s `delete` frees nothing itself and does what the end of the last Ember owner would. An upcast, a hand-over to C++ and `into_raw()` on such an object give the base subobject's address, the header's plus that offset (`[FFI-54]`).}}

### FFI-39c (p21-annex-c.md; 3 words changed)
  - …the pointer it is given (contracted [[`owned`]]
  + …the pointer it is given (contracted {{`owns`}}
  - …(`[FFI-17d]`), the argument is handed over [[(`panel.add_child(b)`:]]
  + …(`[FFI-17d]`), the argument is handed over {{(`panel.add_child(owns b)`:}}

### FFI-17d (p21-annex-c.md; 186 words changed)
  - …it has been handed to C++ [[(`[FFI-39c]`).]]
  + …it has been handed to C++ {{(`[FFI-39c]`); until then the object is Ember's and ends when its last Ember owner does, as an `owner="ember"` one. `@ffi(trampoline, owner=…, threads=creator|any)` says which threads C++ may call the class's overrides from and end its objects on; the default is `creator`: an object of the class is bound to the thread that made it (`[THR-8]`: not `Send`, `E7008` if Ember hands it to another thread), whose id the trampoline subobject holds, one word; every override thunk and the trampoline's destructor attach the calling thread first (`[FFI-22]`), compare it with that id, in every profile, and stop the program otherwise ("C++ called `on_click` of a `Button` made on another thread (line 16); its class says `threads=creator`"). `threads=any` requires the class to be `Send` and `ThreadShared` (for an `open` class, the promise its header writes, `[THR-8]`) (`E7015` at the class, naming the field: "`Button.log` is a `shares mut` owner, which stays on one thread, and C++ may call `Button`'s overrides from any thread (`threads=any`); make the log `thread_shared`, with `lines` `guarded by` a lock"), and each override is then checked as an export under `threads=any` is (`[FFI-33]`, `[THR-20]`).}}

### FFI-40 (p21-annex-c.md; 218 words changed)
  - …x: T` where the type is [[movable;]]
  + …x: T` where the type is {{movable: C++ lends the object for a move, so this `owns` is not `[FFI-11]`'s. At a call from Ember the argument is handed over (`cache.add(owns splash)`), the thunk passes its address, and when the call returns the moved-from object is ended by `T`'s destructor, as C++ ends a moved-from local; in an `override fn` (`[FFI-39]`) the trampoline makes the parameter's object by `T`'s move constructor from the reference C++ passed, before the override runs, so the override ends an object of its own (`[FFI-32]`) and C++'s object stays C++'s to end; a `T&&` parameter of a type with no public move constructor is skipped (`W5033`), as an `&&`-qualified member is;}}
  + …the type is movable; a `T&` {{or `const T&`}}
  + …type is movable; a `T&` result {{is a pointer result (`[FFI-2a]`): its lifetime is never derived, and the overlay's entry gives it a lifetime word, `from(this)`, `from(p)` or `from(static)` (`[FFI-11]`), under which it}}
  + …result is a `ref mut T` {{or `ref T`}}
  + …a `ref mut T` view into {{what}}
  - …`ref mut T` view into the [[object]]
  + …`ref mut T` view into the {{word names (`[LT-1a]`); without one the method stays unsafe to call, its result a `*mut T` or `*T`, listed by `ember bind --report` among the result-position pointers with no lifetime word (`[FFI-35a]`),}}
  - …view into the object and a [[`const T&` result]]
  + …view into the object and a {{call of it outside `unsafe:` is `E3100` ("`child.parent()` returns}}
  - …and a `const T&` result a [[`ref T`]]
  + …and a `const T&` result a {{C++ reference whose target no contract names; put line 7 in an `unsafe:` block with a `# SAFETY:` note, or give `parent` a lifetime word in the overlay: `fn parent() -> borrowed}}
  - …T&` result a `ref T` one [[(`[LT-1]`).]]
  + …T&` result a `ref T` one {{from(this)`").}}

### FFI-40a (p21-annex-c.md; 88 words changed)
  - …(ODR-160). An overlay's `class C:` section [[restates a method as an Ember method line (`fn lookups() -> c_int`);]]
  + …(ODR-160). An overlay's `class C:` section {{holds the entry of each member it states (`[GRM-35]`, verdict 131):}}
  - …(`fn lookups() -> c_int`); `@ffi(const)` above [[the line]]
  + …(`fn lookups() -> c_int`); `@ffi(const)` above {{a method's entry (`fn lookups()`)}}
  - …an asserted fact, which `ember tcb` [[lists, and a line]]
  + …an asserted fact, which `ember tcb` {{lists; an entry}}
  - …without it imports the method as [[changing]]
  + …without it imports the method as {{one that changes}}
  - …imports the method as changing its [[object. A line]]
  + …imports the method as changing its {{object, whatever the importer reads from the body (verdict 132): an entry says what the method does, and leaves nothing to a reading the reader of the overlay cannot see. An entry}}
  - …changing its object. A line whose [[name,]]
  + …changing its object. A line whose {{name or}}
  - …object. A line whose name, parameters [[or result]]
  - …or result differ from the import's [[method]]
  + …or result differ from the import's {{member, or that fits none or several members of one name (`[FFI-57]`),}}
  - …differ from the import's method is [[`E5010`.]]
  + …differ from the import's method is {{`E5010` (`[FFI-12]`); the non-`const` member of a pair is entered under its own name, `at_mut` (`[FFI-40b]`).}}

### FFI-41 (p21-annex-c.md; 85 words changed)
  + …(`cpp.engine.Grid.made += 1`), in `unsafe` unless {{the header declares}}
  - …+= 1`), in `unsafe` unless it [[is]]
  + …`unsafe` unless it is `const` and {{its type is `ThreadShared` (`[STA-1]`),}}
  - …it is `const` and holds no [[pointer,]]
  + …it is `const` and holds no {{pointer and has no `mutable` member, its own or a base's (`[FFI-40a]`: a `const` method may change such a member on any thread, so the value is not unchanged),}}
  - …through a thunk that gives its [[address,]]
  + …through a thunk that gives its {{address; a read outside `unsafe` of one that fails a condition is `E3100` ("`STATS` is a C++ `const` object whose type `Stats` has a `mutable` member (`hits`), which C++ may change while this reads it; put line 9 in an `unsafe:` block with a `# SAFETY:` note"),}}
