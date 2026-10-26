# 04 reader-4: the words and the error messages of spec lines 5015-6930

Slice: `docs/spec-source/Ember_v0.9.10_Hardened_35.md` lines 5015-6930 (Part XVII toolchain and
diagnostics, Part XVIII requirements on implementations, Annex A syntax summary, Annex B hot reload,
Annex C C++, Annex D GPU, Appendix E coming from Python, Appendix F glossary). Read in full.

Power: `PowerLineStatus` printed `Online` at the start.

My files, all under `target/redesign3/04-reader-4/`:
- `slice.md`: the slice, copied (line 1 = spec line 5015).
- `words.py`: lists every line where each must-cover word is used as code (inside backticks or an
  `ember` block, comments removed) and every `@name` in the slice. All line numbers below are spec
  line numbers and come from its output.
- `codes.py` / `codes.tsv`: every code of the registry (spec 5624-5875), with the shape that covers it
  and every spec line outside the registry that gives it a help.
- `t01`...`t21` `.em`: the snippets behind every claim about today's compiler, each checked with
  `target/redesign3/bin/ember.exe check <file>`; the outputs are pasted where they are used.

Terms used: a **word** is anything a programmer writes to say something (a keyword, an `@` attribute,
a type or function that acts as a language word). **Owner's test**: for every name in a line, point at
the word saying what it is (the owner's own thing, a copy, or handed over) and the word saying whether
it may change. **Error-message rule**: a refusal names what was refused in the program's own names,
says why in one line, and gives the exact line to write instead. A **help** is the `= help:` line of a
diagnostic; a **shape** is the spec's category of error that fixes which help is printed (XVII.6.1,
XVII.6.2, `[DIA-21]`).

Not judged here: whether the memory model is good (phase 2).

---

## Part 1: the words

### 1.1 What is in the slice

`words.py` found these must-cover words used as code in the slice (counts are distinct lines):

- Reserved keywords used (41 of 47): `and` (1) `as` (9) `class` (8) `comptime` (3) `const` (12)
  `defer` (3) `dyn` (5) `else` (3) `enum` (1) `extern` (7) `false` (3) `fn` (50) `for` (18) `if` (11)
  `implements` (1) `import` (6) `in` (28) `interface` (2) `is` (8) `let` (5) `match` (5) `mut` (27)
  `not` (4) `or` (2) `override` (2) `owned` (11) `pass` (3) `pub` (1) `ref` (8) `return` (12) `self`
  (38) `Self` (1) `static` (6) `struct` (4) `super` (4) `true` (3) `type` (4) `unsafe` (21) `virtual`
  (2) `with` (9) `yield` (9).
- Not in the slice: `break` `continue` `elif` `void` `where` `while`.
- Contextual words used: `open` (4) `from` (3) `extend` (3) `gen` (6) `once` (1) `safe` (1) `some` (3)
  `c` (1 as the keyword, line 6135; the other hits are `--backend c` and a variable `c`) `cpp` (8)
  `overlay` (5); `abstract` only in prose (5644, 6524); `language` only as a manifest key (5119, 5170)
  and in E0006's text (5631). Not in the slice: `threads` `rename` `hide` `union` `async` `await`
  `macro`.
- Attributes in the slice (40): see 1.4. `@realtime` appears only as the manifest section
  `[realtime]` (5151, 5174). `@dataclass` (6766) is Python's, quoted in Appendix E: not an Ember word.
- Library words used: `Box` (4) `Shared` (5) `Weak` (8) `RefCell` (5) `Cell` (3) `Span` (7) `MutSpan`
  (4) `str` (14) `String` (9) `.clone()` (1 as code, more in prose) `mem.take` (3) `mem.replace` (1)
  `mem.drop` (1) `swap` (1) `Option` (18) `None` (8) `Retained` (3) `Pool` (1) `Handle` (4) `Send` (4)
  `Sync` (6). Not in the slice: `.copy()` `mem.swap` `Mutex` `RwLock`.
- Words I added (language-like words in the slice that are not on the list): `Copy`, `Clone`,
  `.copied()`, `drop`, `ForeignBox`, `adopt`, `retained` (a contract word), `OldSelf`,
  `migrate_from`, `read` (in `pub(read)`), `Atomic`, `Result`, `debug_assert`, `assert_disjoint`,
  `Generator`, `CppShared` (and `CppWeak`, `CppVector`, `CppString`), `TODO(...)`, `cstr`.

### 1.2 The findings that matter, first

Ten words fail one of the owner's tests in this slice. In short:

| word | test that fails | why a programmer cares | verdict |
|---|---|---|---|
| `owned fn` (the prefix use of `owned`) | a keyword does the same for every type | `owned fn` moves a `String` but copies an `int` (`[CLO-2]`, spec 2064); the owner named it as unacceptable (JOB line 50) | remove the prefix; `owned` itself stays |
| `Box` | plain English; owner ruling | `Box[dyn Shape]` says nothing about one owner; the owner: `owned` replaces `Box` | rename to an `owned` field |
| `ForeignBox` | plain English, and it is named after `Box` | once `Box` goes, the name points at nothing a reader knows | rename with `Box` |
| `@static_safe` | plain English | it forbids only aliasing checks and allows bounds, overflow and stale-handle checks (`[EFF-12]`, spec 3260); a reader takes it for "every safety check proved before running" | rename |
| `@nosync` | same meaning everywhere | `@nosync` is not "not `@sync`": `@sync` = a class shareable across threads; `Sync` = may be read from several threads; the effect `Sync` = an atomic, lock, channel or count operation (spec 3184). Three meanings for one word | rename the effect and the contract |
| `@fp(contract)` | same meaning everywhere | "contract" in Ember means a checked promise (`@noalloc`, glossary 6890) or an FFI fact list; here it means "allow fused multiply-add" | rename the argument |
| `RefCell` | plain English | the name hides that every access is checked while the program runs and may panic (`[PRF-1]` lists it among panicking checks, 5259) | rename |
| `str` | plain English, and it means the opposite of Python's | Python's `str` is an owned string; Ember's is a borrowed view. The slice has a whole catalogued Python-habit error for it (P26, 5461) | rename `str`; keep `String` (agreed example 5 uses it) |
| `let` (on a class field) | plain English | `let entity: int` needs the comment "assigned only in init" (6152); a JavaScript reader reads `let` as a changeable variable, the opposite; `let x = 5` as a local is itself a refused habit (P9, 5444) | rename |
| `MutSpan` | a keyword does exactly what it says | `buf: MutSpan[float]` cannot be written through unless the parameter is also `mut` (t18, t21 below); two words say "may change" and the type's word alone does nothing | merge with `mut` |

Four more are kept but must be passed on (details in 1.3-1.5): `.clone()` (helps must name `copied`,
agreed example 9), `with` (three uses), `Handle` (two meanings in prose and messages), `class`
(assignment of a class value aliases with no word saying so: phase 2, area C).

Evidence from today's compiler for the `MutSpan` finding. The spec's own required acceptance `[BCK-6]`
(spec 6045-6052, "these are accepted") is refused today:

```text
$ ember.exe check t20_bck6.em          (the [BCK-6] example verbatim, plus a main)
error[E3023]: cannot mutate borrowed parameter `xs`
 --> t20_bck6.em:4:28
  |
1 | fn first_even(xs: MutSpan[int]) -> ref mut int:
  |               ---------------- `xs` is borrowed here
...
4 |             return ref mut xs[i]
  |                            ^^^^^ this write needs mutable access
  |
  = help: restructure to a single owner and declare this parameter `mut xs: MutSpan[i64]`
  = help: if the value is genuinely shared, use `RefCell[T]` for runtime-checked borrows
  = help: if the value has identity, make it a `class`
  = note: `RefCell` adds a one-word runtime borrow-state check in every profile (CELL-5, CELL-9); a class adds a heap allocation, a 24-byte header, reference counting and dynamic exclusivity, and is not `@static_safe` (OBJ-1, RC-1, EXC-1, EFF-13)
(the same error again at 5:5 `xs[0] = 0` and 6:20 `return ref mut xs[0]`)
error: could not compile due to 3 error(s)
exit=1
```

With `mut` the same function is accepted (`t19_mutspan_mut.em`, `fn fill(mut buf: MutSpan[float], ...)`,
exit 0, no output). Either the spec example or the compiler is wrong; for the word, the point is that
`MutSpan` alone does not say what the code may do.

### 1.3 Reserved and contextual keywords

Columns: **lines** = spec lines where it is used as code; **same?** = same meaning everywhere and for
every type; **plain?** = says in plain English what it does; **on it?** = sits on the thing it
describes.

| word | lines and meaning at each | same? | plain? | on it? | verdict and reason |
|---|---|---|---|---|---|
| `owned` | (a) parameter mode "moved in", the caller hands the value over: 6190 `owned xs: Array[int]`, 6671 `owned frame: Frame`, 6900 glossary "mode", 5497 B7 help "taking it `owned`", 5665 E2031; (b) the closure prefix `owned fn`, "owns its captures": 5410 `[DIA-10]`, 5499 B9 help, 6221 `task = owned fn() => println("done")`; (c) `owned e` in a `for`/`match`, "take the container over": 5483 O2 help; (d) E0109 "`owned` is not permitted in expression position" 5643; (e) a C++ `T&&` parameter imports as `owned T`, 6592 | no: (b) moves some captures and copies others (`[CLO-2]`, spec 2064: "moved in (copied if `Copy`, retained if a handle)") | (a), (c), (e) yes; (b) no | (a), (c), (e) yes; (b) no: it sits on `fn`, not on the captured names | **keep** for (a), (c), (e), which mean "ownership handed over" as the owner ruled. **Remove** use (b): the owner's named failure (JOB line 50); agreed example 6 puts the word on the captured name (`fn() with copied score:`), so a capture handed over is `with owned x`. E0109's text is also wrong today: `task = owned fn() => ...` is an `owned` in expression position and is accepted (t01, exit 0, no output) |
| `ref` | 6219 `r = ref xs[0]` a read-only borrow; 6370 `old: ref OldSelf`; 5457 `ref dyn Shape`; 5497 `ref T`; 6504 `ref [T; N]`; 5645 E0111; 6592 a `T&` result "is a `ref`"; 5483 O2 help | yes | yes ("reference") | yes | **keep** |
| `mut` | (a) parameter mode "in-out, the caller sees the change": 6186 `mut buf`, 6856 `mut rng`, 6900, 5494 B4, 5500 B10, 5739 E3027; (b) receiver `mut self`: 6147, 6670-6674, 5718, 5812, 6038, 6590; (c) inside `ref mut` (6046-6051, 6188, 6504) and `*mut T` (6562); (d) a C++ `T&` imports as `mut T` (6591) | yes in the slice: "may change the caller's own value" | yes | yes | **keep**. Two notes: `mut buf: MutSpan[float]` says "may change" twice (see `MutSpan`); and outside the slice, agreed example 8 refuses `State.Walking(mut path):` because there `mut` would read as Rust's "changeable copy" |
| `self` / `Self` | `self` the receiver, read-only by default: 6145-6162, 6174-6178; `Self` the type being declared: 6370 | yes | yes | yes | **keep** |
| `class` | 6151 `open class Script`, 6159, 6165, 6765 (Python's classes), 6851, glossary 6889 "handles are counted, identity is observable" | yes | partly: it does not say "shared, counted handle" | yes | **keep** the word. Pass to phase 2 (area C): `p2 = player` aliases a class object with no word in the line saying so, which the owner's test refuses for a name with no word ("never aliases"); `snapshot = items` for an `Array` hands over (agreed example 9). Same `=`, two meanings decided by the declaration elsewhere |
| `struct` | 6142 value type; 5966 `struct str`; 6520 `@layout(c) struct`; 6766 replaces `@dataclass` | yes | yes | yes | **keep** |
| `let` | 6152 `let entity: int  # assigned only in init`; 5686 E2104 "assignment to a `let` field outside its `init`"; 5314 "`let` fields"; 6353 "`let` changed"; 5444 P9 `let x = 5` refused as a habit | one meaning (a field fixed after `init`), only legal on fields | **no** | yes | **rename** (e.g. `fixed entity: int`): the line needs a comment to say what the word means, a JavaScript reader reads `let` as "changeable", and the same spelling as a local is a refused habit. A programmer cares because the word must say "never changes after init" |
| `const` | 6137 `const MAX: int = 1024` compile-time constant; 5691 E2130. (6503, 6590-6617 are C++'s `const`, quoted) | yes | yes | yes | **keep** |
| `static` | 6138-6139 a program-wide value; 5453 P18 help; 5691; 5811 E7002; glossary 6922. (5925 `static inline` is C's) | yes | yes | yes | **keep** |
| `pub` / `read` | 6153 `pub(read) health` "readable everywhere, written here"; 5657 E1051 | yes | yes | yes | **keep**. Note for 05: agreed example 7 adds `reads renderer` on a method (what it reads); `pub(read)` is who may read. Different meaning, nearly the same word |
| `open` | 6151 `open class` subclassable; 6765 | yes (as a keyword; `File.open` is a method) | yes | yes | **keep** |
| `virtual` / `override` | 6156 `virtual fn on_update`, 6161 `override fn`; 5687 E2110, 5688 E2111, 5863 W2111 | yes | yes | yes | **keep**. W2111 accepts `virtual` in a final class where it does nothing: under the error-message rule a form that does not do what its word says is refused (Part 2) |
| `fn` | everywhere: functions, methods, lambdas `fn(x) => e` (6202, 6220), callable types (5068, 6424, 6512) | yes | yes ("function") | yes | **keep** |
| `gen` | 6197 `gen fn countdown(n) -> Generator[int]`; 5560 L2004; 5711 E2220; 6762 | yes | short for "generator"; the result type says `Generator` again | yes | **keep** |
| `yield` | 6199; 5711-5720; 6896 | yes | yes (Python's word) | yes | **keep** |
| `once` | 5486 O5 help "declare the parameter `once fn`" | yes | yes | yes, on the type | **keep** |
| `some` | 6201 `-> some Iterator[Item = int]`; 5723 E2260, 5724 E2261 | yes | yes ("some iterator") | yes | **keep** |
| `dyn` | 5457 `ref dyn Shape`, `Box[dyn Shape]`; 5068 `Box[dyn fn]`; 5524 N10; 5672 E2050; 5967 | yes | an abbreviation of "dynamic"; the owner wrote it himself (`no_share(dyn Pool)`) | yes | **keep** |
| `interface` / `implements` / `extend` | 6173 `interface Drawable`; 6176 `extend Shape implements Drawable:`; 5523 N9, 5669 E2041 | yes | yes | yes | **keep** |
| `enum` / `type` | 6168 tagged union; 6666 `type TextureHandle = Handle[gpu.Texture]` alias; 6511 a `type` overlay entry | yes | yes | yes | **keep** |
| `super` | 6571 `super.init(…)`, 5684, 6765 | yes | yes | yes | **keep** |
| `import` / `from` / `as` | 6132-6135, 6849; `as` also casts: 5438 `x as int`, 5450, 5518, 6755, 6779 | `as` has two meanings (rename on import, numeric cast) | yes, both read as English | yes | **keep** both; note the two uses of `as` |
| `c` / `cpp` / `overlay` | 6135 `import c "zlib.h"`; 6458 `import cpp "Header.hpp"`; 6461 `overlay cpp "Header.hpp":`; 5803 `unsafe overlay`; 5106, 5330 | yes | yes; "overlay" is defined in the glossary (6905) | yes | **keep** |
| `with` | (a) a binding ended at the block's end: 6222 `with scope = thread.scope():`, 6691 `with cmd = frame.command_list():`, 5446/6763 `with f = fs.File.open(p)?:`, 5489 O8, 5497 B7; (b) import options: 6135 `with (link=["z"])`, 6458 `with (project=...)`; (c) outside the slice, the agreed capture list `fn() with copied score:` (example 6) | three uses | each reads as English | yes | **keep**; 05 and 07 should note that `with` now has three uses |
| `defer` | 6224 `defer: println("leaving demo")`; 5447 P12; 5696 E2160 | yes | yes | yes | **keep** |
| `comptime` | 6225 `comptime: assert(...)`; 5808 `comptime(e)`; 5144 manifest `[comptime]` | yes | yes (compile time) | yes | **keep** |
| `unsafe` | (a) a block where the programmer vouches for safety: 6226 `unsafe: ptr_write_example()`, 6708 `unsafe: cmd.native()`, 5508 U1, 5755 E3100; (b) a declaration the caller must vouch for: 5105, 5566/5855 `unsafe extern`, 6563 `ffi.adopt` "(`unsafe`)"; (c) `unsafe overlay` (5803 E5067), "this file asserts facts the compiler cannot check"; (d) glossary 6916: an `unsafe fn` body is all outside Safe Ember | two directions: (a) and (c) "I vouch", (b) "you must vouch" | yes ("not checked") | yes | **keep**. Pass to area E: (d) means the unsafe operations inside an `unsafe fn` are not marked, which Rust changed in 2024 for that reason |
| `safe` | 5769 E5002 "`safe fn` on a function whose contract is [incomplete]": in an extern block, the programmer asserts the call is safe | yes | yes; that it is an assertion is said by `unsafe overlay` and `ember tcb` | yes | **keep** |
| `extern` | 5566, 5855 `unsafe extern`; 6010; 6302, 6424 `extern "C" fn`; 6456, 6486 (C++'s `extern "C"`) | yes | yes | yes | **keep** |
| `match` / `if` / `else` / `for` / `in` / `is` / `not` / `and` / `or` / `pass` / `return` / `true` / `false` | Annex A 6181-6229 and Appendix E 6742-6798; `in` also the range-type clause (5708 E2213) and membership (5716); `is` tests `None` and identity (5694, 6214); `else` after `for` (6747) has Python's meaning | yes (each as in Python) | yes | yes | **keep** all. `for ... else` keeps Python's meaning, so a Python reader is not surprised |
| `abstract` | prose only: 5644 E0110 "abstract class"; 6524 (C++'s) | yes | yes | yes | **keep** |
| `language` | 5119/5170 manifest key `language = "0.9.9"`; 5631 E0006 "`#!` language directive" | yes | yes | yes | **keep** |

### 1.4 Attributes (`@name`)

| word | lines and meaning | same? | plain? | on it? | verdict and reason |
|---|---|---|---|---|---|
| `@sync` | a class whose handles may cross threads, fields fixed after `init`, atomic counts: 6164, 5810, 5812, 6077, 6083, 6475, 6923, 5322, 5368 | yes | partly ("synchronised") | yes | **keep**; a `@sync` class is `Sync`, so it agrees with the interface |
| `@nosync` | 5758 E4002 "`@nosync` function reaches a synchronising operation"; the effect it forbids is `Sync` = "an atomic read-modify-write stronger than relaxed, a lock, a channel operation, thread spawn or join, retaining or releasing a `Sync` handle" (spec 3184) | **no**: `Sync` the effect ≠ `Sync` the interface (6917, "may be read from several threads at once") ≠ `@sync` the class marker | no | yes | **rename** the effect and the contract (e.g. effect `Atomic`/`ThreadSync`, contract `@no_thread_sync`). A programmer cares: `@nosync` reads as "not a `@sync` class", and E7005 "value shared with a task is not `Sync`" and E4002 use the same word for different things |
| `@static_safe` | 5506 S1, 5756 E3105, 5762 E4030, 6315: forbids only `RuntimeCheck(Aliasing)` (spec 3243, 3260) | yes | **no** | yes | **rename** (e.g. `@no_alias_checks`): it does not mean "safe, proved statically"; bounds, overflow and stale-handle checks remain. `static` also already means a global |
| `@fp` | `@fp(contract)` 5767 E4072, 5835 E9041, 5920: allows fused multiply-add | the argument `contract` clashes with Ember's "contract" (6890) | no | yes | **rename** the argument (e.g. `@fp(fuse)`); keep `@fp` |
| `@fastmath` | 5767, 5835, 5920 | yes | yes | yes | **keep** |
| `@noalloc` `@noblock` `@noio` `@nolock` | 5757, 5759, 5764, 5765, 5366, 6551, 6890: forbid allocation, blocking, I/O, locks | yes | yes | yes | **keep** |
| `@nopanic` | 5763 `@nopanic(explicit)`: forbids explicit panics; checks may still panic | yes | yes, because the argument is required (`[EFF-17]`) | yes | **keep** |
| `@deterministic` | 5766, 5767 | yes | yes | yes | **keep** |
| `@realtime` | only as the manifest section `[realtime] contracts = [...]` (5151, 5174) | yes | yes | n/a | **keep** |
| `@simd` | 5281, 5761 `@simd(assert)`, 6315 | yes | yes | yes | **keep** |
| `@parallel` | 6227 `@parallel(chunk=256)` on a loop; 6315 | yes | yes | yes | **keep** |
| `@inline` `@noinline` `@cold` `@hot` | 5925, 5931-5934, 6310: `@inline` is binding, the others hints; `@inline` does nothing on a reloadable function in a hot-reload build (5932) | yes | yes | yes | **keep**. Where `@inline` has no effect the build should print a note saying so (it changes no result, so not an error) |
| `@export` | 5972 `@export("name")` sets the symbol; 6320 | yes | yes | yes | **keep** |
| `@always_specialize` `@never_specialize` | 6011, 5714 E2223 | yes | yes | yes | **keep** |
| `@reflect` | 6068 field descriptors | yes | yes | yes | **keep** |
| `@derive` | 5680 E2080, 6141 `@derive(Copy)`, 6520 | yes | yes | yes | **keep** |
| `@layout` / `@gpu_layout` / `@gpu` | 6520 `@layout(c)`; 6724 `@gpu_layout(std140 | std430)`; 6729 `@gpu` reserved | yes | yes | yes | **keep** |
| `@borrows` | 5496 B6 help `@borrows(p)`; 5665 E2031; 6921 | yes | yes ("the result borrows p") | yes | **keep** |
| `@view` | 5664 E2030 "`@view` on a type that is not a view"; 5067 `[CLI-20]`: `ember fmt --migrate` removes `@view` | yes | yes | yes | **keep**. The slice contradicts itself: `[TYP-34]` keeps `@view` as checked documentation, `[CLI-20]` strips it. Under the owner's rule that meaning stays written, the migration should not remove it |
| `@must_drop` | 5720 E2231 | yes | uses "drop" | yes | **keep** |
| `@overflow` | 6810 `@overflow(wrap)` | yes | yes | yes | **keep** |
| `@test` / `@should_panic` | 5027, 5305 | yes | yes | yes | **keep** |
| `@safety` | 5850 L3016 `@safety("…")` on an `unsafe fn`: what callers must ensure. Blocks use the comment `# SAFETY:` instead (5508, 6226, W3012) | two notes with one name: a precondition and a justification | yes | yes | **keep** (Rust has the same split; Part 2 asks that W3012's help gives the comment line) |
| `@noreload` / `@reloadable` / `@reinit_on_reload` / `@renamed_from` / `@allow_reload_terminate` | 5561, 5842, 6311-6317, 6411, 6420; 6361, 6393; 6347, 6357; 6381 | yes | yes | yes | **keep** |
| `@grade` | 6625 `@grade(instrumented)` | yes | yes | yes | **keep** |
| `@ffi` | 6511 `@ffi(variant=[...])`, 6512 `@ffi(std_function)`, 6546 `@ffi(noexcept)`, 6581 `@ffi(trampoline, owner="ember")` | one attribute, several facts; each argument plain | yes | yes | **keep**; `owner=` agrees with `owned` (who destroys the object) |
| `@dataclass` | 6766 | Python's word, quoted in Appendix E; Ember writes `struct` | n/a | n/a | **keep** (nothing to judge: not an Ember word) |

### 1.5 Library words that act as language words

| word | lines and meaning | same? | plain? | on it? | verdict and reason |
|---|---|---|---|---|---|
| `Box` | 5068 `Box[dyn fn]`; 5457 `Box[dyn Shape]` (P22 help); 5704 E2200 "a recursive value type without `Box`"; 6366 "`Box` contents" | yes (one owner, on the heap) | **no** | yes | **rename**: owner ruling, `owned` is the one-owner field (`mesh: owned Mesh`, JOB line 52); P22's help becomes `owned dyn Shape` |
| `ForeignBox` | 6508 `std::unique_ptr<T>` imports as `ForeignBox[T]`; 6523; 6563 `ffi.adopt[T](p)` | yes (one owner, freed by the C++ deleter) | no | yes | **rename** with `Box` (e.g. an `owned` foreign field) so the one-owner word is the same in Ember and at the C++ boundary |
| `adopt` | 5788 E5052, 5804 E5068, 6497, 6563: take ownership of a raw foreign pointer | yes | yes | yes | **keep** |
| `Shared` / `Weak` | 5321, 5422, 5424, 5801, 6083, 6365, 6510, 6765, 6809 | yes | yes | yes | **keep**. Note for 05: agreed example 9's message names `shared` (lower case) as a way out |
| `CppShared` `CppWeak` `CppVector` `CppString` | 6506-6518, 5801 | yes | yes | yes | **keep** |
| `RefCell` | 5259, 5422, 5424, 5846 L3011, 6898 | yes | **no** | yes | **rename**: the name does not say "checked while running, may panic"; Python readers get nothing from it. Today's E3023 help offers it as a fix (t20) |
| `Cell` | 5422, 5494 B4 "`Cell` for a counter", 6898 | yes | weak | yes | **keep**: it never panics and the change is visible at the use (`.set`); note it for area B |
| `Span` | 5320, 5454, 5497, 6201, 6503, 6505, 6926 | yes | yes | yes | **keep** |
| `MutSpan` | 6186 `fn fill(mut buf: MutSpan[float], v)`; 6046 `[BCK-6]` `xs: MutSpan[int]` written through; 5320; 6503; 6926 | no: with `mut` it can be written, without `mut` it cannot (t18, t20, t21) | the "Mut" promises what it does not give alone | yes | **merge** with `mut`: one word should say "may change" (`mut buf: Span[float]`, or `MutSpan` alone allowing writes). Today's refusal of the spec's own `[BCK-6]` example is the evidence |
| `str` | 5438, 5461 P26 "a `str` kept where a `String` is wanted", 5462, 5497, 5778, 5799, 5966, 6139, 6193, 6207, 6505, 6506, 6776, 6926 | yes (a borrowed text view) | **no**, and Python's `str` is the owned string | yes | **rename** `str` (e.g. `TextView`): a Python programmer reads it as the owned string, which P26 exists to catch |
| `String` | 5461, 5987, 6174, 6177, 6204, 6389, 6506, 6852 | yes (owned text) | yes | yes | **keep**: agreed example 5's message uses `String(...)`. But today `String("x")` is refused (see Part 4) |
| `.clone()` | 5422 `[DIA-9]`; 5482 O1 help `x.clone()`; 5485 O3 "clone per iteration"; 5522 N8; 6807 "offers `.clone()`" | yes | yes | yes | **merge** into `copied` in every help: agreed example 9 makes `copied items` the spelling of a copy and the error names `copied` or `shared`. Whether the `Clone` method stays is phase 2's |
| `Copy` / `Clone` | 5680 E2080, 6141 `@derive(Copy)`, 6142 "Eq, Debug, Clone implicit", 6520, 5839 L2002 | yes | yes | yes, on the type | **keep**. Pass to area C: a `Copy` value is copied by `q = p` with no word in the line |
| `.copied()` | 6202 `xs.iter().copied()` | yes, agrees with `copied` | yes | yes | **keep** |
| `mem.take` / `mem.replace` / `swap` | 5483 O2, 5484 O3, 5490 O9 | yes | yes | yes | **keep** |
| `mem.drop` / `drop` | 5488 O7 `mem.drop(x)` ends a value now; 5750 E3070 "`drop` cannot be called explicitly"; 5731 E3016; `fn drop` is the clean-up method | two uses (clean-up method, "end now") | "drop" is Rust's word, plain enough | yes | **keep**; E3070's help already says `mem.drop(x)` |
| `Option` / `None` / `Result` | 5436, 5445, 5700, 5701, 6214, 6670, 6682, 6193, 6370 | yes | yes | yes | **keep** |
| `Retained` / `retained` | 5796 E5060, 6422 `Retained.pin(handle, on_relocate: ...)`, 6575 `tok = Retained.pin(self)`; 5787 E5051 the contract word `retained` | yes (foreign code keeps a pointer) | yes | yes | **keep** |
| `Handle` / `Pool` | 5495 B5 "store an index or `Handle`"; 6666-6667 `Handle[gpu.Texture]`; 6897 glossary: "handle" = a class reference **or** a generational index | the prose word has two meanings; the type one | yes | yes | **keep** the type. In messages, "handle" also means a class object (E2150 5694, E3080 5751, `[PRF-1]` "stale handles"); 05/15 should give the class meaning another word |
| `Send` / `Sync` | 5813 E7004, 5810-5814, 6515, 6608, 6679, 6917 | `Send` yes; `Sync` also names an effect (see `@nosync`) | `Send` yes; `Sync` partly | yes | **keep** both as interfaces; the effect is renamed (see `@nosync`) |
| `Atomic` | 6138 `static HITS: Atomic[int]`, 6166, 5775 | yes | yes | yes | **keep** |
| `OldSelf` / `migrate_from` | 6370 `fn migrate_from(old: ref OldSelf) -> Result[Self, ReloadError]`; 5715 E2225 | yes | yes | yes | **keep** |
| `debug_assert` / `assert_disjoint` | 5260, 5271, 5861 W2016; 5501 B11 | yes | yes | yes | **keep** |
| `Generator` | 6197, 6762 | yes | yes | yes | **keep** |
| `TODO(...)` | 5101 an unknown foreign fact in a starter overlay; 5850 L3016 | yes | yes | yes | **keep** |
| `cstr` | 5800 E5064, 5966 | yes | yes ("C string") | yes | **keep** |

---

## Part 2: the error messages (04's extra duty)

### 2.1 The spec's own rule is weaker than the owner's

`[DIA-2]` (5386-5389) asks that "`help` says what to do". The owner's rule asks for **the exact line
to write**, in the program's own names. Proposed text for `[DIA-2]`: "A diagnostic names what was
refused in the program's own names, says why in one line, and gives in `help` the exact line to write
instead, in the program's own names and types (`int`, not `i64`, where the program wrote `int`). A
form that is accepted but does not do what its words say elsewhere is refused with that help."

`[DIA-10]` (5409-5410) names `owned fn` as a model concrete help. With `owned fn` removed (1.3), B9's
help becomes the capture word (`fn() with owned x:`, spelling from phase 2).

### 2.2 Today's messages for the agreed examples (real output)

Example 3 (a method changes one field while a loop walks another), `t06_ex3_method_self.em`:

```text
error[E3020]: cannot mutate `*self.items` while it is borrowed by this loop
 --> t06_ex3_method_self.em:8:13
  |
7 |         for x in self.items:
  |                  ---------- `*self.items` borrowed here for the whole loop
8 |             self.note(f"{x}")
  |             ^^^^^^^^^^^^^^^^^ mutable borrow here
  |
  = help: collect the indices or values first, then mutate; use `retain` or `drain` when the operation fits
  = help: or iterate over a fixed index range and mutate only after taking the current value
  = note: a `for` loop borrows the iterable until the loop ends (CTL-2)
```

It is classified B2 ("mutation while iterating"), not B8, and `*self.items` is not the program's
spelling. Proposed (B8, E3025): "`self.note(...)` may change all of `self`, and this loop is reading
`self.items`; `note` only changes `self.log`. Write its first line as `fn note(self, msg: String)
changes log:`" (the agreed line, word for word).

Example 4 (`return ref g[0]` inside `with g = scores.lock():`), `t07_ex4_lock.em`: accepted today,
exit 0, no output. A new refusal is needed (a new code in E3000-E3499, shape B12) with the agreed
message: "this points into the locked list, which can change once the lock ends; return a copy, or
do the work inside the `with` block". To meet the exact-line rule it may add the line, in the
program's names: `return copied g[0]` (spelling of `copied` from phase 2).

Example 5 (a returned view of a temporary), `t08_ex5_view.em`:

```text
error[E3060]: `_5` does not live long enough
 --> t08_ex5_view.em:3:12
  |
2 |     word = line.to_lower().trim()
  |            ---- `_5` is borrowed here
  |            --------------- `_5` is a local, so its storage ends with the frame
3 |     return word
  |            ^^^^ the borrow is still live when the function returns
  |
  = help: return an owned value, take the destination as a `mut` parameter, or borrow something the caller owns
  = note: a returned reference must derive from a parameter (LT-1)
```

`_5` is a compiler-internal name in the primary message, which `[DIA-2]` itself forbids. Proposed:
"`word` is a view of the lowered copy of `line`, made by `line.to_lower()`, which ends when `clean`
returns; return `String(word)` to hand back your own copy, and change the result type to `String`"
(agreed text: "return `String(...)` to hand back your own copy").

Example 7 (a helper called before a field is set), `t09_ex7_init.em`:

```text
error[E2100]: `self` is used before all class fields are initialized
 --> t09_ex7_init.em:6:23
  |
6 |         self.assets = self.load_assets()
  |                       ^^^^^^^^^^^^^^^^^^
  |
```

No help at all. Proposed: "`self.load_assets()` is called before `self.assets` is set, and the
compiler cannot see which fields `load_assets` reads; it reads `self.renderer`, which is set. Write its
first line as `fn load_assets(self) reads renderer -> Array[String]:`" (agreed spelling).

Example 8 (`State.Walking(path):` then `path.pop()`), `t03_walk_plain.em`; and with `mut path`,
`t04_walk_mut.em`, the same message:

```text
error[E3021]: cannot write through a shared reference
  --> t03_walk_plain.em:10:17
   |
10 |                 path.pop()
   |                 ^^^^^^^^^^ this is `ref Array[i64]`, which only reads
   |
   = help: a reference is not pointed elsewhere: this writes through it, into what it refers to; to keep another value, keep a copy of it (`.clone()`), or its index
   = note: while a shared borrow is live the owner may read and copy but not write [BRW-1]
```

`ref mut path` is accepted (`t05_walk_refmut.em`, exit 0). Proposed (agreed): "`path` reads the
NPC's list in `State.Walking`, so `path.pop()` cannot change it; to change that list through `path`,
write `State.Walking(ref mut path):`". And `State.Walking(mut path):` itself is refused with the same
help (today it is accepted as a pattern; the refusal comes only at `path.pop()`).

Example 9 (use after hand-over), `t02_e3040.em` (the `[TST-0]` example at 5292-5298):

```text
error[E3040]: `a` is borrowed after it has been moved out of
 --> t02_e3040.em:4:5
  |
3 |     b = a
  |         - moved here
4 |     a.push(4)
  |     ^^^^^^^^^ borrowed here after the move
  |
  = help: borrow before the move, or clone the value so the move takes the copy
  = note: the borrow would point at memory whose owner gave it away
  = note: assigning a list, a map or a set moves it; in Python both names would point at one list (Appendix E)
```

Proposed (agreed: the error names both ways out): "`a` was handed over to `b` on line 3, so `a` has
no list left to push to; to keep a copy, write `b = copied a`; to let both names use one list, write
`b = shared a`" (spellings from phase 2).

### 2.3 Messages in the slice that do not say what to write instead

Every row of the registry (spec 5626-5875) was checked against the shape tables in the slice and every
spec line outside the registry that gives that code a help (`codes.tsv`). The codes below do not
give **the exact line to write**: either no help exists anywhere in the spec, or the shape's help is
a category or a list of options (quoted in the "today" column). A proposal is written to the rule:
what (in the program's names, shown as `<name>`), why, and the line to write. 200 rows; with the 5
run-time messages of 2.4, 205 messages in the slice do not say what to write instead.

Lexing, parsing, directives:

| code | line | today | proposed |
|---|---|---|---|
| E0001 | 5626 | invalid UTF-8 in source | bytes `<hex>` at `<line>:<col>` are not UTF-8; save the file as UTF-8 |
| E0005 | 5630 | reserved keyword used as an identifier | `<word>` is a keyword and cannot name `<item>`; write `r#<word>`, or choose another name such as `<word>_` |
| E0008 | 5632 | unterminated character literal | `'<text>` opens a character and never closes; text is written `"<text>"`, one character `'<c>'` |
| E0009 | 5633 | malformed literal | `<literal>` is not a number: `<reason>`; write `<nearest valid literal>` |
| E0100 | 5634 | unexpected token (helps only for P-habits and `::`) | `<token>` cannot follow `<previous>` here; this place expects `<expected>`, e.g. `<example line>` |
| E0101 | 5635 | unclosed delimiter or literal | the `<delim>` opened at `<line:col>` is never closed; add `<closer>` before `<next item>` |
| E0102 | 5636 | membership test chained | `a in b in c` tests `(a in b) in c`; write `a in b and b in c` |
| E0103 | 5637 | match arms mix statement and expression form | arm `<pattern>` uses `=>` while arm `<pattern>` uses `:`; write every arm `<pattern>:` with a block, or every arm `<pattern> => <expr>` |
| E0104 | 5638 | attribute or construct not admitted here (help only for bare `@nopanic`) | `@<attr>` cannot stand on `<thing>`; it belongs on `<allowed things>`: move it to `<line>` |
| E0107 | 5641 | a jump expression may not be an operand | `<return/break> ...` is used as a value inside `<expr>`; put it on its own line: `if <cond>: return <v>` |
| E0108 | 5642 | attribute is not permitted on this statement | `@<attr>` cannot stand on a `<statement kind>`; only `@parallel`, `@unroll`, `@simd` and `@allow` may stand on a statement: move it to the function line |
| E0109 | 5643 | `owned` is not permitted in expression position | today's help (t11) "to move a value, pass it to an `owned` parameter" is wrong for assignment; proposed: "`owned <x>` is not an expression; `<y> = <x>` already hands `<x>` over: write `<y> = <x>`" |
| E0112 | 5646 | nested more deeply than the compiler accepts | this expression is nested `<n>` deep, past the limit `<m>`; move the inner part into a local: `<tmp> = <inner>` |
| E0900 | 5647 | construct not implemented by this compiler | `<construct>` is not implemented by this compiler (`ember --version --matrix`); write `<nearest implemented form>` or use a compiler that implements it |
| E0901 | 5648 | construct this specification leaves unspecified | `<construct>` has no defined meaning in Ember `<version>`; write `<defined alternative>` |

Names and modules:

| code | line | today | proposed |
|---|---|---|---|
| E1020 | 5650 | name is already declared in this block | `<name>` is declared again in this block (first at `<line>`); to change it write `<name> = <v>` without a type, or use a new name |
| E1021 | 5651 | name is not linked in this build | `<name>` lives in layer `<layer>`, which this package does not list; add `"<layer>"` to `layers` in `[package]` |
| E1030 | 5652 | two items with the same name in one scope | `<name>` is declared twice in `<module>` (lines `<a>`, `<b>`); rename one |
| E1031 | 5653 | a name bound by two glob imports is used | `<name>` comes from both `<m1>.*` and `<m2>.*`; write `from <m1> import <name>` |
| E1041 | 5655 | import cycle between packages | packages `<a>` → `<b>` → `<a>` import each other; move `<shared items>` into a third package both import |
| E1050 | 5656 | field is read-only outside its module | `<obj>.<field>` is `pub(read)` and may be changed only in `<module>`; call `<setter>` or change it there |
| E1051 | 5657 | `read` visibility is valid on fields only | `pub(read)` cannot stand on `<item kind>` `<name>`; write `pub` |
| E1052 | 5658 | item, field or constructor not visible here | `<name>` is private to `<module>`; mark it `pub` there, or call `<public alternative>` |
| E1061 | 5660 | no item of that name in the module | `<module>` has no `<name>`; did you mean `<candidates>`? write `from <module> import <candidate>` |

Types, inference, interfaces, patterns:

| code | line | today | proposed |
|---|---|---|---|
| E2020 | 5663 | mismatched types (general case; helps exist only for literals, sets, scalars N4 and P-habits) | `<expr>` is `<A>`, but `<place>` needs `<B>`; write `<conversion>` (e.g. `"x".to_string()`). Today `String("x")` gives `` `String()` takes no arguments `` with no help (t13 first run) |
| E2030 | 5664 | `@view` on a type that is not a view | `<Type>` holds no reference, span or `str`, so `@view` is wrong; remove `@view` |
| E2031 | 5665 | `@borrows` names a parameter the result cannot borrow from | the result cannot borrow from `<p>`, which `<reason>`; write `@borrows(<valid param>)`, or return an owned value |
| E2041 | 5669 | overlapping `extend` implementations | `<Type>` implements `<I>` twice (`<line a>`, `<line b>`); delete one |
| E2042 | 5670 | `Float` is implemented only by `f32` and `f64` | `<Type>` cannot implement `Float`; implement `<other interface>` |
| E2043 | 5671 | an associated type's default leads back to itself | `<I>.<A>` defaults to `<chain>`, which leads back to itself; give `<A>` a concrete default |
| E2050 | 5672 | interface is not `dyn`-compatible (N10 names the clause, not a fix) | `dyn <I>` is impossible because `<method>` `<clause>`; write `[T: <I>]` instead of `dyn <I>`, or move `<method>` to another interface |
| E2060 | 5673 | cannot infer type | the type of `<name>` cannot be worked out; write `<name>: <Type> = <value>` |
| E2061 | 5674 | lambda parameter types cannot be inferred here | write the parameter type: `fn(<x>: <Type>) => ...` |
| E2062 | 5675 | ambiguous type | `<expr>` could be `<A>` or `<B>`; write `<expr> as <A>` or annotate `<name>: <A>` |
| E2071 | 5677 | `SoA[T]` of a type that is not a struct | `SoA[<T>]` needs a struct, `<T>` is a `<kind>`; use `Array[<T>]` |
| E2080 | 5680 | `@derive(Copy)` on a type with a field that is not Copy | `<Type>` cannot be `Copy` because `<field>: <FT>` is not; remove `Copy` from `@derive`, or make `<field>` a `Copy` type |
| E2090 | 5681 | non-exhaustive match | `match <x>` does not handle `<missing patterns>`; add `<missing>: ...` or `_: ...` |
| E2100 | 5682 | field read before it is initialised | see 2.2 example 7 |
| E2101 | 5683 | derived class with a field that has no default needs an `init` | `<Class>.<field>` has no default and `<Class>` has no `init`; write `<field>: <T> = <default>` or `fn init(self, ...)` setting it |
| E2102 | 5684 | derived `init` uses `self` ... before `super.init` | `<line>` uses `self` before `super.init(...)` runs; move `super.init(...)` above it |
| E2103 | 5685 | assignment to `self` in a class method | `self = ...` would point this method's `self` elsewhere; change fields instead: `self.<field> = ...` |
| E2104 | 5686 | assignment to a `let` field outside its `init` | `<Class>.<field>` is fixed after `init`; set it in `init`, or remove `let` from its declaration |
| E2110 | 5687 | override of a method that is not virtual | `<Base>.<m>` is not `virtual`; mark it `virtual fn <m>` in `<Base>`, or rename this method |
| E2111 | 5688 | replaces an inherited virtual one without `override` | `<m>` replaces `<Base>.<m>`; write `override fn <m>(...)` |
| E2120 | 5689 | inherent extension of a type from another package | `<Type>` belongs to `<package>`; write an interface and `extend <Type> implements <I>:` |
| E2121 | 5690 | an implementation outside the packages that may write it | `<I>` for `<Type>` may be written only in `<pkg a>` or `<pkg b>`; move this `extend` there, or wrap `<Type>` in a struct of your own |
| E2131 | 5692 | array length must be a constant | `[<T>; <n>]` needs a constant length; write `const <N>: int = ...`, or use `Array[<T>]` |
| E2140 | 5693 | cannot assign to a value expression | `<expr>` is a temporary result, not a place; bind it first: `<tmp> = <expr>` then `<tmp>.<f> = ...` |
| E2151 | 5695 | integer `**` with a constant negative exponent | `<a> ** <-n>` is a fraction; write `<a> as float ** <-n>` |
| E2160 | 5696 | control flow cannot leave a `defer` block | `<return/break>` inside `defer:` would skip the rest of the clean-up; set a flag and act on it after the block |
| E2170 | 5697 | reference to a field of a packed struct | `<s>.<f>` may be unaligned in packed `<S>`; copy it out: `<v> = <s>.<f>` |
| E2172 | 5698 | cannot index with a type | `<x>[<T>]` indexes with a type; to instantiate a generic write `<x>[<T>](...)` |
| E2173 | 5699 | neither a type nor a constant (P22 covers one case) | `<name>` is a `<kind>`, not a type; write `ref dyn <I>`, `owned dyn <I>` or `[T: <I>]` |
| E2180 | 5700 | `?` outside a function returning `Option` or `Result` | `<f>` returns `<T>`, so `?` has nowhere to send the error; change its result to `Result[<T>]`, or write `match <expr>:` |
| E2182 | 5702 | a function that returns a value can reach the end of its body | `<f>` must return `<T>`, but the path through `<line>` ends without `return`; add `return <value>` |
| E2183 | 5703 | `main` has none of the forms `[FN-8]` allows | write `fn main():`, `fn main() -> Result[void, E]:` or either with `args: Array[String]` |
| E2200 | 5704 | type has infinite size | `<S>` contains `<S>` through `<field>`; write `<field>: Option[owned <S>]` (today the compiler crashes instead of reporting it: Part 4) |
| E2210 | 5705 | a value of one range type where another was expected | `<x>` is `<R1>`, `<place>` needs `<R2>`; write `<R2>.checked(<x>)?` |
| E2211 | 5706 | constant outside the target range type | `<n>` is outside `<R>` (`<a>..<b>`); write a value in range, or `<R>.clamped(<n>)` |
| E2212 | 5707 | range endpoints are not constants ..., or inverted | `<a>..<b>` must be constants with `<a>` ≤ `<b>`; write `type <R> = <T> in <b>..<a>` |
| E2213 | 5708 | invalid `in` clause on a type alias | `in` on `<alias>` needs a whole-number type and a constant range; write `type <R> = int in <a>..<b>` |
| E2215 | 5710 | range-typed value constructed outside the permitted set | `<R>(<v>)` is not proven in range; write `<R>.checked(<v>)?` or `<R>.clamped(<v>)` |
| E2220 | 5711 | `yield` outside a `gen fn` | `<f>` uses `yield`; write `gen fn <f>(...) -> Generator[<T>]:` |
| E2222 | 5713 | coroutine where an ordinary function is required | `gen fn <f>` cannot be `extern`, `@export`ed or passed to C; wrap it: a plain `fn` that collects `<f>(...)` |
| E2223 | 5714 | `@never_specialize` on a generic that is not shareable | `<f>` uses `<T>` as `<use>`, so one shared body cannot serve every type; remove `@never_specialize` |
| E2229 | 5718 | class generator method takes `mut self` or holds an access across `yield` | `gen fn <m>` keeps `self` while suspended; write `gen fn <m>(self)` and copy `<field>` out before `yield` |
| E2230 | 5719 | a name assigned in every branch at different types | `<x>` is `<A>` in one branch and `<B>` in another; write `<x>: <A> = ...` before the `if`, or convert in one branch |
| E2231 | 5720 | `yield` while a `@must_drop` value is live | `<v>` must be dropped and could be left suspended; end it before `yield`: `mem.drop(<v>)` |
| E2250 | 5722 | format spec does not apply to the value's type | `{<x>:<spec>}` does not fit `<T>`; write `{<x>}` or `{<x>:<valid spec>}` |
| E2260 | 5723 | `some` type outside a return position | `some <I>` may only be a result type; write `[T: <I>]` for a parameter |
| E2261 | 5724 | returns of a `some` function have different types | `<f>` returns `<A>` at `<l1>` and `<B>` at `<l2>`; write `-> owned dyn <I>` or return one type |

Ownership and borrows (shape exists but no exact line; agreed examples where they apply):

| code | line | today | proposed |
|---|---|---|---|
| E3014 | 5729 | scope binding may not be moved (O8 "bind it with `with`") | `<guard>` must end at the end of its block; write `with <guard> = <expr>:` and do not pass it on |
| E3015 | 5730 | value whose drop is required may not be leaked | `<v>` must be dropped; end it on every path: `mem.drop(<v>)` before `<line>` |
| E3016 | 5731 | `self` escapes its own drop (O9) | `drop` stores `self` into `<place>`; take the data out instead: `<d> = mem.take(self.<field>)` |
| E3021 | 5733 | shared and mutable borrow overlap (B3 "end the shared borrow first, or copy the value out") | `<r>` reads `<place>` until line `<l>`, and line `<m>` changes it; move line `<m>` after `<l>`, or copy: `<v> = copied <place>` (example 8 has its own message, 2.2) |
| E3023 | 5735 | two writers of one value (B4) | today's help on a `MutSpan` parameter (t18) says "bind the new value to a new local (`new_buf = …`)", which is wrong (nothing was rebound), and t20/t21 offer `RefCell` and `class`. Proposed: "`<p>` is borrowed for reading; to change what it holds write `mut <p>: <T>` in the signature" in the program's own type names |
| E3025 | 5737 | a method takes all of `self` (B8) | see 2.2 example 3: write `fn <m>(self, ...) changes <field>:` |
| E3026 | 5738 | a closure outlives what it captures (B9 `owned fn`) | `<closure>` keeps using `<x>`, which ends at `<line>`; hand it over: `fn() with owned <x>:`, or keep `<x>` alive with `thread.scope()` (spellings from phase 2) |
| E3030 | 5740 | closure would move a captured value out (O5 `once fn`) | today t13 is classified O2 (`E3013`) with ten options and not the right one; proposed: "`fn() => consume(<s>)` gives `<s>` away, so it can run only once, but `<param>` may call it many times; declare `<param>: once fn()`" |
| E3040 | 5741 | use of moved value (O1 `x.clone()`) | see 2.2 example 9: names `copied` and `shared` |
| E3041 | 5742 | moved in a previous loop iteration (O3) | `<x>` is handed over on the first pass of the loop at `<line>`; write `<x>` inside the loop, or `copied <x>` |
| E3050 | 5744 | use of an uninitialised place (O6 "name the path") | `<x>` has no value on the path through `<line>`; give it one before: `<x> = <default>` |
| E3060 | 5745 | borrowed value does not live long enough (B7) | t14: "return an owned value, take the destination as a `mut` parameter, or borrow something the caller owns". Proposed: "`<view>` points into `<x>`, which ends at `<line>`; return your own copy: `return String(<view>)`" (agreed example 5), or the same with the matching owned type |
| E3062 | 5747 | returned view does not derive from a parameter (B6) | `<f>` returns a view of `<local>`, which ends when `<f>` returns; return your own copy (`String(...)`, `copied ...`), or write `@borrows(<p>)` if it comes from `<p>` |
| E3080 | 5751 | overlapping access through the same handle (X1 "end the first access first; copy the value out") | `<obj>.<f>` is being changed by `<access 1>` while `<access 2>` uses it; copy it out first: `<v> = copied <obj>.<f>` |
| E3095 | 5753 | disjointness is not establishable (B11 `assert_disjoint`) | add `assert_disjoint(<a>, <b>)` before line `<l>` |
| E3100 | 5755 | requires an `unsafe` block (U1) | `<op>` is not checked by the compiler; use `<safe API>`, or write `unsafe:` with `# SAFETY: <why it is sound>` above it |
| E3105 | 5756 | `UnsafeCell` in `@static_safe` code | `<f>` promises no aliasing checks but uses `UnsafeCell`; remove the attribute or the cell |

Effects, FFI, compile time, threads, layout:

| code | line | today | proposed |
|---|---|---|---|
| E4002 | 5758 | `@nosync` reaches a synchronising operation | `<f>` promises no thread synchronisation but reaches `<op>` via `<chain>`; remove `<op>` from the path, or drop the attribute |
| E4003 | 5759 | `@noblock` reaches a blocking operation | as above with `<blocking op>`; use `<non-blocking variant>` (e.g. `try_lock`) |
| E4010 | 5760 | implementation does not satisfy the interface's contract | `<Type>.<m>` reaches `<effect>` but `<I>.<m>` forbids it; remove `<op>` from `<Type>.<m>` |
| E4020 | 5761 | `@simd(assert)` loop did not vectorise | the loop at `<line>` did not vectorise because `<reason>`; write `<fix>` or `@simd` without `assert` |
| E4030 | 5762 | `@static_safe` performs a dynamically checked access (S1 explains, no line) | `<access>` is checked while running because `<reason>`; write `<restructure>` (e.g. copy the field into a local) or remove the attribute |
| E4040 | 5763 | `@nopanic(explicit)` reaches a panic | `<f>` reaches `<unwrap/assert>` at `<line>`; write the `Result` form: `match <x>:` or `<x>?` |
| E4041 | 5764 | `@noio` reaches I/O | `<f>` reaches `<io op>`; return the text and print it outside `<f>` |
| E4042 | 5765 | `@nolock` acquires a lock | `<f>` locks `<m>` via `<chain>`; pass the locked value in as a parameter |
| E4070 | 5766 | `@deterministic` reaches a nondeterministic operation | `<f>` reaches `<op>`; pass `<value>` in as a parameter |
| E4072 | 5767 | `@fastmath`/`@fp(contract)` inside `@deterministic` | remove `@fastmath` from `<g>`, or `@deterministic` from `<f>` |
| E5001 | 5768 | imported type layout does not match | `<T>`'s Ember layout `<a>` differs from C's `<b>` at `<field>`; regenerate the overlay (`ember bind --init`) |
| E5010 | 5770 | overlay does not match the C declaration | overlay line `<l>` says `<ours>`, the header says `<theirs>`; write `<header-matching line>` |
| E5012 | 5772 | pointer contract has no count | `<param>` points at several items but says not how many; add `count(<n>)` to its `@ffi(param(...))` |
| E5014 | 5773 | two packages request the same implementation macro | `<macro>` is compiled by both `<a>` and `<b>`; keep it in one package's `implementation = [...]` |
| E5015 | 5774 | type may not cross the boundary | `<T>` owns Ember memory and cannot go to C; pass `<span/handle/scalar>` instead |
| E5016 | 5775 | `_Atomic` layout does not match `Atomic[T]` | `<field>` is `_Atomic <ct>`, which does not match `Atomic[<T>]`; declare it `<matching type>` |
| E5017 | 5776 | flexible array member is unsized | `<S>` ends in `<f>[]`; handle it through a pointer: `*mut <S>` |
| E5018 | 5777 | `va_list` may not be constructed | `<f>` takes a `va_list`; call the non-`v` variant `<g>` or wrap it in C |
| E5020 | 5778 | `str` is not NUL-terminated | `<arg>` is passed where C expects a NUL-terminated string; write `<arg>.to_cstr()?` |
| E5030 | 5779 | `std::function` as a parameter | `<param>` is a `std::function`; take `fn(...)` through a C++ wrapper, or mark the overlay entry `@ffi(std_function)` |
| E5031 | 5780 | C++ parameter cannot be mapped | `<param>: <C++ type>` has no Ember type; write an overlay line for `<f>` giving it one, or wrap it in C++ |
| E5032 | 5781 | opaque C++ type may not be constructed | `<T>` is opaque; call its constructor `<T>.new(...)` |
| E5040 | 5783 | capturing closure passed where a C function pointer is expected | `<closure>` captures `<x>`; pass a plain `fn` and give `<x>` through the `user_data` parameter |
| E5041 | 5784 | retained callback needs a release function | `<f>` keeps `<cb>`; add `release = <fn>` to its overlay entry |
| E5042 | 5785 | returns a pointer into a parameter C passes as a copy | `<f>` returns a pointer into its copy of `<p>`, which ends at return; return a value |
| E5050 | 5786 | grade whose evidence is absent or stale | `@grade(<g>)` on `<fact>` has no current evidence; run `ember test --instrument-ffi`, or write `@grade(asserted)` |
| E5051 | 5787 | callee retains a pointer not declared `retained` | `<f>` keeps `<p>`; add `retained` to `<p>` in the overlay |
| E5052 | 5788 | `adopt` where the overlay declares `adopt = false` | `<T>` may not be adopted; use the handle as borrowed |
| E5053 | 5789 | instrumented run contradicted a declared effect | `<f>` was seen to `<effect>` at `<site>`; remove `<effect>` from its `@ffi(effects=...)` claim |
| E5054 | 5790 | range type in a foreign signature | pass `<x> as <T>` and convert with `<R>.checked` on the Ember side |
| E5055 | 5791 | Ember generic may not instantiate a C++ template | list the instantiation in `instantiate=["<T<A>>"]` on the import |
| E5056 | 5792 | override of a C++ virtual the overlay does not name | add `<m>` to the overlay's `class <C>:` section |
| E5057 | 5793 | overlay names a method that is not virtual | `<C>::<m>` is not virtual in the header; remove it from the overlay's overrides |
| E5058 | 5794 | C++ base has no default constructor and no declared `init` | write `fn init(self, ...)` calling `super.init(<args>)` |
| E5059 | 5795 | trampoline base has no virtual destructor | give `<C>` a virtual destructor in C++, or set `owner="ember"` |
| E5060 | 5796 | upcast in a context that cannot hold the `Retained` token | write `tok = Retained.pin(self)` and pass `tok` |
| E5061 | 5797 | no exception policy and none can be derived | add `@ffi(noexcept)` or a throwing policy to `<f>`'s overlay entry |
| E5062 | 5798 | contradictory C++ exception policies | `<f>` is marked both ways in `<overlay a>` and `<overlay b>`; keep one |
| E5063 | 5799 | foreign bytes reach `str` without validation | write `<bytes>.to_str()?` |
| E5064 | 5800 | interior NUL in a value converted to `cstr` | `<s>` contains a NUL at `<i>`; remove it, or pass `<s>.as_bytes()` with its length |
| E5065 | 5801 | `Shared`/`Weak` and `CppShared`/`CppWeak` do not interconvert | keep `<x>` as `CppShared[<T>]`, or copy its value into an Ember `Shared` |
| E5066 | 5802 | marked both `noexcept` and throwing | remove one of the two marks on `<f>` |
| E5067 | 5803 | overlay that states facts is not declared `unsafe overlay` | write `unsafe overlay c "<header>":` |
| E5068 | 5804 | one live foreign object adopted twice | `<p>` is already owned by `<x>`; borrow `<x>` instead of adopting again |
| E5090 | 5805 | inline assembly not supported by the C backend | build with `--backend llvm`, or move the assembly into a C file |
| E6001 | 5806 | comptime exceeded its limits, or a constant depends on itself | raise `max_steps` in `[comptime]`, or break the cycle `<a>` → `<b>` → `<a>` |
| E6004 | 5807 | panic during compile-time evaluation | `<expr>` panicked at compile time: `<message>`; fix the value or move it to run time |
| E6005 | 5808 | `comptime(e)` refers to a run-time local | `<x>` is known only when running; use a `const` or move the computation out of `comptime` |
| E6010 | 5809 | operation not available at compile time | `<op>` cannot run at compile time (or with `--interp`); move it to run time, or build normally |
| E7001 | 5810 | `@sync` class with a field that is not `Sync` | `<Class>.<field>: <T>` cannot be read from two threads; make it `Atomic[<T>]` or a `Mutex[<T>]` |
| E7002 | 5811 | `static` of a type that is not `Sync` | write `static <N>: Mutex[<T>] = ...` or `Atomic[<T>]` |
| E7003 | 5812 | write to a `@sync` class field after `init` | `<obj>.<f>` is fixed after `init` in a `@sync` class; make it `Atomic[...]` or `Mutex[...]` |
| E7004 | 5813 | value crossing a thread boundary is not `Send` | `<x>: <T>` cannot move to another thread because `<field/reason>`; send `<Send alternative>` |
| E7005 | 5814 | value shared with a task is not `Sync` | `<x>` is read from two threads but `<T>` is not safe for that; wrap it: `Mutex(<x>)` |
| E7006 | 5815 | memory order an atomic operation does not support | `<op>` cannot use `<order>`; write `<valid order>` |
| E7010 | 5816 | parallel loop writes to a shared place | each pass of `@parallel` writes `<place>`; write to `<out>[i]` and combine after the loop |
| E7011 | 5817 | parallel loop has a loop-carried dependency | pass `i` uses `<x>` from pass `i-1`; remove `@parallel` or compute `<x>` from `i` |
| E7020 | 5818 | systems in one parallel run have conflicting access sets | `<s1>` writes `<C>` while `<s2>` reads it; run them in order or split `<C>` |
| E8001 | 5819 | GPU layout does not match the CPU layout | `<field>` is at offset `<a>` on the CPU and `<b>` in `std140`; add padding `<pad>` or reorder |

Build, manifest, toolchain (a line in `ember.toml` or a command is the "line to write"):

| code | line | today | proposed |
|---|---|---|---|
| E9003 | 5822 | invalid command line | `<arg>` is not a flag of `ember <command>`; did you mean `<nearest>`? |
| E9010 | 5823 | `[lints]` names a lint the compiler does not define | `<key>` is not a lint; did you mean `<nearest>`? |
| E9011 | 5824 | the toolchain cannot disable floating-point contraction | `<compiler>` cannot turn off fused multiply-add; set `c_compiler = "bundled"` |
| E9013 | 5825 | invalid `[ffi]` manifest section | `<key>` is not an `[ffi]` key; the section takes `evidence` |
| E9020 | 5826 | translation units disagree on an inherited flag | `<flag>` is `<a>` in `<tu1>` and `<b>` in `<tu2>`; set it the same in `[cpp.<project>]` |
| E9021 | 5827 | C++ standard library and CRT heap could not be determined | set `compiler` and `flags` in `[cpp.<project>]`, or point `cmake = { build_dir, target }` at the build |
| E9030 | 5828 | hot reload refused | reload refused: `<type/item>` `<change>`; `<what to do: add a default / migrate_from / restart>` (the `[HR-14]` table gives each) |
| E9031 | 5829 | invalid `reload` value | write `reload = "opt-in"`, `"all"`, `"bodies"` or `"none"` |
| E9034 | 5830 | invalid `max_instantiations` value | write a positive whole number: `max_instantiations = 200` |
| E9035 | 5831 | packages disagree about the object-header layout | build `<pkg>` with reload `<on/off>` like the rest of the process |
| E9036 | 5832 | a reloadable package may not link the runtime statically | write `runtime = "shared"` in `[build]` |
| E9037 | 5833 | reload ABI mismatch on image load | rebuild the host and the package with the same compiler (`<v1>` vs `<v2>`) |
| E9040 | 5834 | two generic instances hash to one symbol | names both; report it as a compiler defect (a help with the command `ember --version` output to attach) |
| E9041 | 5835 | the toolchain cannot honour `@fastmath` or `@fp(…)` | `<compiler>` cannot compile `<f>` separately with relaxed flags; set `c_compiler = "bundled"` or remove the attribute |

Lints and warnings (not refusals, but messages; each lacks the line to write):

| code | line | today | proposed |
|---|---|---|---|
| L1002 | 5837 | assignment declares a new binding | `<new>` is a new name next to `<old>`; did you mean `<old> = ...`? (fix-it exists in `[LNT-2]`; print it) |
| L2001 | 5838 | unnecessary clone | `<x>` is not used after line `<l>`; write `<x>` instead of the copy |
| L2002 | 5839 | large `Copy` value passed by value | `<p>: <T>` copies `<n>` bytes per call; write `<p>: ref <T>` |
| L2003 | 5840 | fallible construction where a total one exists | write `<R>.<total form>(<v>)` |
| L2004 | 5841 | `gen fn` with no `yield` | remove `gen`, or add `yield` |
| L2005 | 5842 | `@noreload` calls a reloadable one in a loop | mark `<g>` `@noreload` too, or call it outside the loop |
| L3001 | 5843 | potential reference cycle | `<A>.<f>` → `<B>.<g>` → `<A>` may form a cycle; write `<g>: Weak[<A>]` |
| L3002 | 5844 | borrow held longer than necessary | `<r>` is last used at `<l>` but kept to `<m>`; move its last use up |
| L3010 | 5845 | `unsafe` block larger than necessary | only `<line>` needs `unsafe`; move the other lines out of the block |
| L3011 | 5846 | `RefCell` guard held across a call | copy out first: `<v> = <cell>.borrow().<f>` then call `<g>` |
| L3013 | 5847 | long-term access held across a call | copy `<obj>.<f>` into a local before calling `<g>` |
| L3014 | 5848 | return region is the intersection of several parameters | write `@borrows(<p>)` naming the one it comes from |
| L3015 | 5849 | undocumented unsafe obligation | write `@safety("<what the caller must ensure>")` on `<f>` |
| L3016 | 5850 | `@safety` text still reads `TODO` | replace `TODO` with what the caller must ensure |
| L3017 | 5851 | reference cycle detected | `<A>` → ... → `<A>` was never freed; write `<field>: Weak[<T>]` |
| L4001 | 5852 | allocation in a hot loop | `<expr>` allocates every pass; create `<buf>` before the loop and `<buf>.clear()` each pass |
| L4002 | 5853 | dynamic dispatch on a final type | `<x>` is always `<T>`; write `<x>: <T>` instead of `dyn <I>` |
| L4003 | 5854 | large `Array[int]` in a hot loop whose values fit 32 bits | write `Array[i32]` |
| L5001 | 5855 | `unsafe extern` declaration with no contract | add `@ffi(param(...), result(...))` to `<f>` |
| L5002 | 5856 | conversion at the FFI boundary copies | pass `<x>.as_span()` |
| L7001 | 5857 | lock held across a call that may block | end the `with` block before calling `<g>` |
| W1002 | 5858 | binding shadows an enum variant | `<name>` in this pattern binds a new name, it does not match `<E>.<name>`; write `<E>.<name>` |
| W1003 | 5859 | package module shadows the standard module | `<m>` hides `std.<m>`; rename `<m>.em` |
| W2015 | 5860 | float literal has more digits than its type keeps | `<lit>` is stored as `<rounded>`; write `<rounded>` |
| W2016 | 5861 | `debug_assert` argument has side effects | move `<call>` before the assert: `<v> = <call>` then `debug_assert(<v> ...)` |
| W2091 | 5862 | unreachable match arm | arm `<p>` is covered by `<earlier>`; remove it or move it up |
| W2111 | 5863 | `virtual` has no effect in a final class | should be refused (the word does not do what it says): "`<C>` cannot be subclassed, so `virtual fn <m>` changes nothing; write `open class <C>` or remove `virtual`". Today, t10: `warning[W2111]: ... virtual has no effect in a final class`, no help, exit 0 |
| W2190 | 5864 | unused `Result` | the error of `<call>` is dropped; write `<call>?` or `match <call>:` |
| W2192 | 5865 | use of a deprecated item | `<item>` is deprecated: `<reason>`; write `<replacement>` |
| W2220 | 5866 | instantiation ceiling exceeded | `<g>` has `<n>` instances; raise `max_instantiations` or mark it `@never_specialize` |
| W3012 | 5867 | unsafe block with no SAFETY note | add `# SAFETY: <why this is sound>` above `unsafe:` |
| W5001 | 5868 | function-like macro ignored | wrap `<MACRO>` in a C function, or write it in the overlay |
| W5002 | 5869 | declaration not imported | `<decl>` uses `<unsupported>`; wrap it in C |
| W5031 | 5870 | C++ declaration skipped: header could not be parsed | fix `<include path / flag>` in `[cpp.<project>]` |
| W5033 | 5871 | `&&`-qualified member skipped | add a C++ wrapper for `<m>` |
| W5034 | 5872 | anonymous-namespace entity skipped | give `<e>` a named namespace in C++, or wrap it |
| W5050 | 5873 | unbacked or stale grade | run `ember test --instrument-ffi` |
| W5054 | 5874 | unexercised foreign fact | add a test that calls `<f>` |
| W9030 | 5875 | reload requires a restart | restart the program: `<item>` `<change>` cannot be reloaded |

Codes already giving a line to write somewhere in the spec (not listed above; `codes.tsv` gives the
spec line): E0002-E0004 (N12), E0006, E0105, E0106, E0110, E0111, E1010, E1032, E1060, E2010, E2011,
E2035, E2036, E2040, E2070, E2072, E2073, E2130, E2150, E2181, E2214, E2221, E2225, E2226, E2228,
E2240, E3010-E3013 (O2, but see E3030), E3020 (B2), E3022 (B1), E3024 (B5), E3027 (B10), E3042 (O4),
E3061, E3063, E3065, E3090, E3096 (A1), E4001, E5002, E5011, E9001, E9002, L1001 (today's help
"remove it, or rename it `_n`", t17), E3070 (O7: `mem.drop(x)`), E5034 (the `[FFI-48]` table's
"Instead" column, which the message should print).

### 2.4 Other error examples in the slice (run-time messages and notes)

| line | message | proposed |
|---|---|---|
| 5297 | `#$ error[E3040]: a is borrowed after it has been moved out of` | as 2.2 example 9; the registry text "use of moved value" and this text should be one wording |
| 5180 | note `hot reload is off in release builds` | add "build with `--profile development` to reload" |
| 5406 | `borrow of v created at 12:9, live through 19, because s is used at 19:14` | an explanation, not a refusal: no line needed |
| 6076 | panic `reference count overflow on <Class>` | a run-time panic; add the cause: "more than 2^63 handles to one `<Class>`; a handle is being copied in a loop that never ends" |
| 6092 | panic `stack overflow in <function>` | add "`<function>` called itself `<n>` deep; make the recursion a loop" where the compiler can name it |
| 6680 | panic `stale TextureHandle (generation 7, current 9)` | add "the texture was destroyed at `<file:line>`; check `device.is_live(h)` first" |
| 6704 | panic `buffer is in flight (frame 42); use a per-frame ring or wait` | has a fix; name it exactly: "use `Ring[<T>, N]` (`ring.current(frame)`)" |
| 5527 | `field 'x' is not declared in class C` (N13) | has a fix-it; keep |

---

## Part 3: hand-over notes for 05 (builder)

- Agreed examples 3, 5, 7 and 9 each need a shape help changed (B8, B6/B7, a new help for E2100, O1);
  example 4 needs a new code; example 8 needs N7/E3021's help and a refusal of `mut` in a pattern.
- Words with more than one meaning in this slice: `with` (3 uses), `as` (2), `unsafe` (2 directions),
  `Sync` (3: interface, effect, through `@sync`/`@nosync`), "handle" (2, in messages), "contract" (3:
  effect contract, FFI facts, `@fp(contract)`), `owned` (the `owned fn` use is the odd one out).
- `pub(read)` vs the agreed `reads` clause: same root, different meaning.

## Part 4: found while checking (outside the word duty)

1. **The compiler crashes on a recursive struct** instead of reporting E2200 (`t15_e2200.em`:
   `struct Node: value: int, next: Option[Node]`):

   ```text
   thread 'main' (25320) has overflowed its stack
   exit=127
   ```
   With `Option[Box[Node]]` it is accepted (`t17_e2200_box.em`, exit 0, only `L1001`).
2. **The spec's own required acceptance `[BCK-6]` (6045-6052) is refused today** (t20, pasted in 1.2).
3. **Agreed example 5's message names `String(...)`, which today is refused**:
   `t13_o5.em` first run, `s = String("a")`:
   ```text
   error[E2020]: `String()` takes no arguments
    --> t13_o5.em:9:9
     |
   9 |     s = String("a")
     |         ^^^^^^^^^^^
   ```
   `String.from("a")` is accepted (`t16_string_ctor.em`, exit 0). The builder should know the agreed
   spelling needs `String(x)` to become a constructor.
4. **Wrong shape for a closure that gives a capture away** (t13 second run): expected E3030 / O5
   "declare the parameter `once fn`"; today:
   ```text
   error[E3013]: cannot move out of a reference
     --> t13_o5.em:10:32
      |
   10 |     call_twice(fn() => consume(s))
      |                                ^ moved out here
      |
      = help: borrow it, `ref` (`ref mut` to change it where it is); move it out where its owner is held mutably, with `mem.take`, `mem.replace` or `swap`, or `pop`, `swap_remove`, `remove` or `drain` from a container; or copy it, `.clone()`, leaving the owner's value as it is
      = note: moving out of a `ref`/`ref mut` is forbidden [EXP-6]; the referent still owns the value and drops it
   ```
5. **Messages use the compiler's type names, not the program's**: `MutSpan[i64]` where the program
   wrote `MutSpan[int]` (t18, t20, t21), `ref Array[i64]` (t03), and an internal name `_5` (t08).
6. **`owned` in expression position**: E0109 says it is never allowed, Annex A line 6221 writes
   `task = owned fn() => ...` and today accepts it (t01). E0109's own help (t11) offers "pass it to an
   `owned` parameter" for `b = owned a`, where `b = a` already hands over.
