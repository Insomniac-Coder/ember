# 01 reader-1: the words of spec lines 1-1491 (Parts I-IV)

Spec: `docs/spec-source/Ember_v0.9.10_Hardened_35.md`, lines 1-1491: the front matter, Part I
(overview, principles), Part II (lexical, with the keyword list II.4), Part III (grammar), Part IV
(types). Mains power: `Online` at the start and at the end.

Every word below was looked up in the slice with `target/redesign3/01-reader-1/words.py` (it lists
the lines where a word appears in code: inside a code block or a `backtick` span). Line numbers are
spec lines. Every claim about what today's compiler does carries the real output of
`target/redesign3/bin/ember.exe check`, pasted in section 4; the test files are in
`target/redesign3/01-reader-1/`.

Terms used here:
- **borrowed**: the function reads the caller's own value in place, not a copy (`[FN-1]`).
- **in-out**: the function may change the caller's own value, and the caller sees the change.
- **handed over** (moved): the caller gives the value away and may not use its name afterwards.
- **`Copy` type**: a type whose values are copied bit for bit when assigned (numbers, `bool`,
  class handles, `str`...). A non-`Copy` value (a `String`, an `Array`) is moved instead.
- **class handle**: the value a `class` name gives you: a counted pointer to one shared object.
  Copying a handle makes a second pointer to the same object (and adds 1 to its count).
- **view**: a value that points into something else and may not outlive it (`ref T`, `Span[T]`,
  `str`).

## 1. What fails, most important first

| # | word | test that fails | why a programmer would care | verdict |
|---|---|---|---|---|
| 1 | `owned` | means the same for every type | `owned fn` (lines 692, 714, 1129) moves a `String` capture but silently copies an `int` one: the JOB's own example. `cb = owned fn(): score += 1` is accepted with no warning (test t2) and, by `[CLO-15]`, changes a private copy; the same form with a `String` is refused as a move (t2b). | keep `owned` (hand-over); **remove the `owned fn` form** (agreed row 6 replaces it: `fn() with copied score:`) |
| 2 | `mut` | means the same for every type | On a value parameter `mut b: B` is in-out (`[FN-1]`); on a class-handle parameter it lets the callee re-point the caller's handle (`[FN-9]`); on a class method `mut self` means "holds a write access to every field for the whole call" (`[CLS-7]`, line 1740), while plain `self` in a class may already write fields (t5). Line 1480-1481 says the opposite of `[CLS-7]` ("each of these is a copy of the handle"). Also: the compiler accepts `State.Walking(mut path):`, which the pattern grammar (lines 821-829) does not have, and binds `path` read-only (t1). | keep `mut` for values and `ref mut`; the class-method use of `mut self` is the job of agreed row 3 (`changes <field>`); `mut` in a pattern is refused (agreed row 8) |
| 3 | `self` / plain parameters of a class type | the line says whether it may change | In a class, `fn add(self): self.n += 1` and `fn bump(c: Counter): c.n += 1` both compile (t5): no word says the object's field changes, while the same lines for a `struct` are refused. Agreed row 3 covers methods (`changes log`); a free function taking a class handle has no agreed word yet. | keep `self`; flag the free-function case to the Builder (area B or C) |
| 4 | `str` | plain English; Python spelling, Python meaning (`[PHIL-14]`, lines 202-208) | Python's `str` is an owned string; Ember's `str` is a borrowed view that may not be kept in a class field (`[TYP-15]`, lines 1125-1132). `name: str` in a class compiles when fed a literal (t8) and is refused when fed a `String` (t8b): the type word does not tell the reader which. | rename (e.g. `StrView`, or make `str` the owned string and give the view another name); the spelling is for 05/07 |
| 5 | `let` (field) | says whether it may change, the same for every type | `let x: int` can never change; `let items: Array[int]` can still be changed through (`self.items.push(v)`, `[CLS-9a]`, line 1749, outside my slice). The word reads "fixed" and is only half true. On a local, `let x = 5` is refused with a good message (t4). | rename (a word saying "set only in `init`"), or make `let` mean "never changes" for every type; reader-2 sees the rule itself |
| 6 | `Box` | the owner's ruling (JOB) | `owned` is the one-owner field that replaces `Box`. Uses in my slice: 163, 620, 857, 1105-1106, 1114-1116, 1127, 1354-1355, 1383, 1435, 1442, 1476. | merge into `owned T` (owner). Two things must carry over, see section 3 |
| 7 | `@nosync` vs `Sync` / `@sync` | one word, one meaning | `Sync` is a marker interface "safe to share between threads" (line 1257) and `@sync` makes a class `Sync` (line 1266); but `@nosync` (lines 140, 148) forbids the *effect* named `Sync` (`[EFF-18]`: taking a lock, an atomic operation; table line 3240). A reader takes `@nosync` as "not `@sync`". | rename `@nosync` and the effect `Sync` (e.g. `@nosynchronise` / effect `Synchronise`); keep `Sync`, `@sync` |
| 8 | `@static_safe` | says in plain words what it does | Reads "everything proved at compile time", but it forbids only run-time *aliasing* checks; bounds, overflow and stale-handle checks still run (`[EFF-12]`, line 3260; the table at lines 236-246 lists them). | rename to say what it forbids (e.g. `@no_alias_checks`); note `@no_runtime_checks` is already reserved and rejected |
| 9 | `ref` as a parameter type | two words for one thing | `fn f(x: ref int)` and `fn f(x: int)` mean the same (both accepted, t6). `fn f(x: ref mut int)` cannot write through `x` (t3), its help asks for `mut x: ref mut i64`, and that does not compile either (t3c): `mut x: T` is the only working in-out spelling. | keep `ref`/`ref mut` (agreed rows 4, 8); refuse or define `ref T` / `ref mut T` as a parameter's type, with a message pointing at `x: T` / `mut x: T` |
| 10 | `unsafe` | one word, one meaning | On a block, `extern`, `extend` and `overlay` it means "here I promise what the compiler cannot check"; on `unsafe fn`, `unsafe fn(...)` (type) and `unsafe interface` it means "whoever calls / implements this must promise", and the body of an `unsafe fn` is also an unchecked zone without a per-site mark (`[TIER-1]`, line 152). Rust has the same double meaning and changed it in 2024. | keep the word; split the two meanings (area E) |
| 11 | `RefCell` | says in plain words what it does | The name says nothing about the run-time check that panics on a second borrow (lines 241, 1257, 1429). | rename to a name that says "borrow checked while running" (spelling for 07) |
| 12 | `.clone()` vs the agreed `copied` | two words, one meaning | Agreed rows 6 and 9 make `copied` the word for a copy (`snapshot = copied items`). `.clone()` (lines 797-798, 1277-1278) makes the same copy. Today's error help still says `.clone()` (t1, t2b). | merge: `copied x` is the language word; `Clone` stays as the interface `copied` uses |
| 13 | `Handle` | one word, one meaning | "handle" is a class handle (IV.6, line 1162; `Option[Handle]` line 1107) and also `Handle[T]`, a generation-checked index into a pool (line 244, `[HND-1]`; the compiler's own help in t8b: "store a `u32` index or a `Handle[T]`"). | rename one of the two |
| 14 | `virtual` vs `open` | two words, one meaning | `open class` = may be subclassed (line 557); `virtual fn` = may be overridden (line 545). One idea, two words; `virtual` is C++ jargon. | merge into `open` (`open fn`), low priority, not memory-model |

Two observations for phase 2 that are not word verdicts:
- `b = a` has no word and does three things by type (IV.1 table, lines 849-858): copy (scalars,
  `Copy` values), move (collections, non-`Copy` values), share (class handles: a second pointer to
  the same object). Agreed row 9 settles the list case (`snapshot = items` hands over,
  `copied items` copies, `shared` named as the other way out).
- A lambda parameter written without a mode takes `owned` from the expected callable type
  (`[TYP-23]`, line 1468, ODR-025): the word is decided by the compiler, not written. Area D should
  say whether that stays.

## 2. Every word, one by one

Columns: **where and what** (meaning at each place, with lines); **same?** (same meaning
everywhere and for every type); **plain?** (says in plain English what it does); **on it?** (sits on
the thing it describes); **verdict**.

### 2.1 Reserved keywords (II.4, line 377-387: 47)

| word | where and what | same? | plain? | on it? | verdict |
|---|---|---|---|---|---|
| `and` | logical and (467, 695, 719) | yes | yes | yes | keep |
| `or` | logical or (467, 694, 718) | yes | yes | yes | keep |
| `not` | logical not (467, 696, 720); `not in` one operator (697, 722, 780-781); method name `not` after `fn`/`.` (396-399, ODR-040) | yes (the method is the operator `~`'s name, stated) | yes | yes | keep |
| `as` | import alias (503, 505, 509); conversion `x as T` (705, 730, 938-947: integer narrowing keeps the low bits); `as?` / `as!` checked downcast (471, 476, 810, 945, 1164) | two meanings (rename on import, convert in an expression); both read as English | yes; note `300 as u8` silently keeps the low bits, a known `as` meaning (`T.try_from` is the checked form, 946) | yes | keep |
| `break` | leave a loop (668, 691, 713, 768); `break label` (713) | yes | yes | yes | keep |
| `continue` | next round of the loop (691, 713, 768) | yes | yes | yes | keep |
| `class` | declares a type whose values are shared counted handles (261, 557, 587, 854, 1162) | yes | yes: the Python reader's own expectation (Python objects are shared references) | yes | keep. The "shared" is said once, on the declaration; see failure 3 for what is not said at each use |
| `comptime` | block or expression run at compile time (584, 657, 739, 767, 812) | yes | yes ("compile time") | yes | keep |
| `const` | named constant item (577); const generic parameter (549, 1197-1220); a pattern name that is a constant (835) | yes (a value fixed at compile time) | yes | yes | keep |
| `defer` | run a block at the exit of the enclosing block, on every path (655, 669-670) | yes | yes | yes | keep |
| `dyn` | an interface type whose concrete type is chosen while the program runs, used behind a pointer (608, 857, 1345, 1354-1355, 1435-1442) | yes | partly: short for "dynamic"; `any Shape` (Swift's pair to `some`) would read better, but `any(...)` is already a built-in function (line 802) | yes | keep. `Box[dyn I]` becomes `owned dyn I` (owner) |
| `elif` | else-if (645) | yes | yes (Python) | yes | keep |
| `else` | if's else (645), loop `else` runs when no `break` (647-648, 668), conditional expression (693, 717) | yes (Python's three uses) | yes | yes | keep |
| `enum` | declares a type with variants (565, 852, 1292) | yes | yes | yes | keep |
| `extern` | foreign (C) declaration (544-545, 580-583); `extern "C" fn` callable type (615, 856, 1157); `unsafe extern` (149, 153) | yes ("defined outside, C calling convention") | yes | yes | keep |
| `false` / `true` | the two `bool` values (829, 874, 1363) | yes | yes | yes | keep (both) |
| `fn` | function declaration (546); lambda `fn(x) => e`, `fn(x): stmt` (692, 714, 289, 333); callable type `fn(A) -> R` (615) | yes | yes | yes | keep |
| `for` | loop over an iterable (648); comprehension clause (749, 792); `for x in owned xs` consumes (795) | yes | yes | yes | keep |
| `if` | condition (645); `if pattern = e` (646, 676-678); match guard (652); conditional expression (693, 717); comprehension filter (749) | yes | yes | yes | keep |
| `implements` | a type implements an interface, on its header or on `extend` (556-559, 565, 576, 590, 1229, 1263) | yes | yes | yes | keep |
| `import` | import a module (503-505), `import c "h.h"` (505) | yes | yes | yes | keep |
| `in` | membership `x in c` (697, 722, 780-787); `for x in` (648, 749); range type `f32 in 0.0..=1.0` (579, 1024-1026, 1037) | three uses, each plain English "in" | yes | yes | keep |
| `interface` | declares an interface (571, 1228, 1277-1348) | yes | yes | yes | keep |
| `is` | identity `is`, `is None`, `is not` (466, 697, 722); method name `is[E]()` of `AnyError` (397-399) | yes (the method is stated) | yes | yes | keep |
| `let` | field assignable only in `init` (563; rule `[CLS-9]` line 1748) | **no**: fixes the field for a `Copy` type, only the binding for others (`[CLS-9a]`) | reads "never changes"; half true | yes | **rename** (failure 5) |
| `match` | match statement (651) and expression (750, 764); `match owned e:` binds by move (839) | yes | yes (Python 3.10) | yes | keep |
| `mut` | in-out parameter (553-554; 278, 1136, 1298, 1301); `mut self` (553, 1281, 1308, 1328, 1339, 1342, 1480); `static mut` (578, 582); `ref mut` reference (607, 706, 731, 813, 828, 855, 1112-1117, 1256-1257, 1339, 1431); raw `*mut T` (607, 1149-1156); callable-type parameter (617); lambda parameter that writes (716, 1469); "a `mut` site" (918) | **no** for class types (failure 2); yes for values | yes ("mutable") | yes | keep, with the class uses settled by agreed row 3 and patterns by row 8 (failure 2). Note: for a parameter, `mut b: B` passes the owner's test: "the caller's own value" is the stated default of every parameter (`[FN-2]`) and does not depend on the type, which is not true in a pattern, where a `Copy` field binds as a copy (838) |
| `override` | method replaces the base class's (545) | yes | yes | yes | keep |
| `owned` | hand-over parameter (266, 553-554), `owned self` (1318, 1414, 1481), callable-type parameter (617), lambda parameter (716; inferred, 1468), `owned e` in `for`/`match` (771, 794-796, 839), `IndexSet` / `From` parameters (1342, 1348); **`owned fn` lambda** (692, 714, 1129) | **no**: `owned fn` copies `Copy` captures (failure 1). Every other use means "the callee owns it"; for a `Copy` type (and a class handle) that is a copy the caller cannot tell from a hand-over, because the caller never sees the callee's changes either way | yes | yes | keep; remove the `owned fn` form; extends to the `Box` replacement (section 3) |
| `pass` | empty statement or body (560, 562, 633) | yes | yes (Python) | yes | keep |
| `pub` | visibility (538); `pub import c` (505) | yes | yes | yes | keep (see `package`, `read`) |
| `ref` | reference type `ref T` (607, 1112-1113, 1255-1256, 1336, 1345); borrow expression `ref e` (706, 731, 813-816); pattern `ref x` / `ref mut x` (828); return `-> ref Output` (1336) | yes: "points at that place, read-only" (`ref mut`: may change through it) | yes | yes | keep (agreed rows 4, 8). Parameter-type use duplicates the default mode (failure 9) |
| `return` | return from a function (667, 713, 768, 1029, 1451) | yes | yes | yes | keep |
| `self` | method receiver (553, 266-271, 1278-1348, 1480-1481); `self` / `mut self` / `owned self` modes | **no**: plain `self` is read-only for a value type and may write fields for a class (t5, failure 3) | yes | yes | keep; the class case is agreed row 3 (`changes <field>`) |
| `Self` | the implementing type (609, 738, 1278-1348, 1403, 1438) | yes | yes | yes | keep |
| `static` | static item (578, 582); the static region: lives for the whole program (421-422, 449, 1130-1142) | yes | yes | yes | keep |
| `struct` | declares a value type (255, 556, 852) | yes | yes | yes | keep |
| `super` | in my slice only in the keyword list (385); used as `super.init(...)` (line 1728) | yes | yes (Python) | yes | keep |
| `type` | type alias (579), range type with `in` (1024-1037), associated type (574, 1307-1317), opaque foreign type (583) | yes ("names a type") | yes | yes | keep |
| `unsafe` | `unsafe:` block (656), `unsafe fn` (149, 545), `unsafe extern` (149, 580), `unsafe interface` (571, 1269-1270), `unsafe extend` (576, 1263-1268), `unsafe fn(...)` callable type (615), `unsafe overlay` (154) | **no**: "I promise here" vs "you must promise" (failure 10) | yes | yes | keep the word; split the two meanings (area E) |
| `virtual` | method may be overridden; bodiless only in an `abstract class` (545, 589) | yes | no (C++ jargon) | yes | **merge** into `open` (failure 14) |
| `void` | the unit type, value `()` (609, 858, 876, 896-897) | yes | yes | yes | keep |
| `where` | generic bounds clause (551, 1261, 1438) | yes | yes | yes | keep |
| `while` | loop (647, 668) | yes | yes | yes | keep |
| `with` | `with` block holding a resource for the block (653-654); options of `import c ... with (...)` (510) | two uses, each English "with" | yes | yes | keep. Agreed row 6 adds a third (`fn() with copied score:`); it reads right and cannot be confused with the `with` statement (which binds `name = expr`), but 05 should list all three together |
| `yield` | give a value from a generator (691, 713, 769) | yes | yes (Python) | yes | keep |

### 2.2 Contextual and reserved-for-later words (`[LEX-15]`, lines 389-401)

| word | where and what | same? | plain? | on it? | verdict |
|---|---|---|---|---|---|
| `abstract` | `abstract class`: has bodiless `virtual` methods (390, 544, 557, 589) | yes | yes | yes | keep |
| `open` | `open class`: may be subclassed (391, 557) | yes | yes | yes | keep (and take over `virtual`) |
| `from` | `from m import x` (391, 504) | yes | yes | yes | keep |
| `extend` | add methods / implementations to a type (391, 576, 590-600, 1178-1190, 1237, 1263) | yes | yes | yes | keep |
| `gen` | `gen fn`: a generator (392, 545, 778) | yes | yes (short for generator; Python infers it from `yield`, Ember writes it, which is better for the owner's rule) | yes | keep |
| `once` | `once fn`: may be called at most once (392, 615, 692, 714, 758) | yes | yes | yes | keep |
| `safe` | `safe fn` in an extern block: callable without `unsafe` (392, 581) | yes | yes | yes | keep |
| `some` | `some I` in a return type: one hidden concrete type (392, 608, 623-626, 1443-1451) | yes | yes | yes | keep |
| `c`, `cpp` | `import c "h.h"`, `import cpp` (393, 505, 529) | yes | yes | yes | keep (both) |
| `language` | `#! language "v"` directive (108-110, 349, 393, 501, 519) | yes | yes | yes | keep |
| `threads` | `#! threads main|any|creator`: default thread contract of a file's exports (394, 501, 520-522) | yes | yes | yes | keep |
| `overlay`, `rename`, `hide` | overlay source over a C header (95, 154, 393-394); detail in `[GRM-35]` (reader-3) | yes | yes | yes | keep (all three) |
| `union` | reserved at the start of an item, so `Set` keeps its `union` method (395-396) | yes | yes | yes | keep |
| `async`, `await`, `macro` | reserved for later (303, 306, 401) | yes | yes | n/a | keep (all three) |

Words a programmer writes that are not on the must-cover list: `module` (`#! module overflow(saturate)`,
501, 523-528), `package` and `read` inside `pub(...)` (538: `pub(read)` = others may read it, not
change it), `init` (the constructor, 266), `SAFETY:` / `SAFETY(<category>):` comments that the compiler
reads (347, 359-363), `r#name` raw names (374). Each means one thing and reads right: keep. `pub(read)`
fits agreed row 7's `reads` (same English idea, no clash).

### 2.3 Attributes in my slice

| word | where and what | same? | plain? | on it? | verdict |
|---|---|---|---|---|---|
| `@derive` | ask the compiler to write `Copy`, `Ord`... (254, 1255, 1267, 1374) | yes | yes | yes | keep |
| `@noalloc` | function proven not to allocate (140, 148, 277) | yes | yes | yes | keep |
| `@nosync` | function may not take locks or atomics: forbids the effect `Sync` (140, 148; table line 3240) | **no** (clashes with `Sync`, `@sync`) | no | yes | **rename** (failure 7) |
| `@noblock` | function may not block (148) | yes | yes | yes | keep |
| `@nopanic` | `@nopanic(explicit)`: no panic the programmer writes (148; bare form refused, line 3263) | yes | yes with its argument | yes | keep |
| `@static_safe` | forbids run-time aliasing checks only (148, 234; `[EFF-12]`) | yes | **no** (says more than it does) | yes | **rename** (failure 8) |
| `@deterministic` | the function's results do not depend on anything that varies between runs (148) | yes | yes | yes | keep |
| `@realtime` | the contract set the declaring package's manifest names (198) | yes (always "this package's set"), but what it forbids is set elsewhere | yes | yes | keep; `ember inspect` must show the set it stands for |
| `@sync` | a class whose objects may be shared between threads (1266) | yes | yes | yes | keep |
| `@overflow` | `wrap` / `saturate` / `panic` for the operators in a function or module (956-963) | yes | yes | yes | keep |
| `@fastmath` | relaxes strict IEEE float rules in one function (1002, 1016, 1052) | yes | yes | yes | keep |
| `@fp` | `@fp(contract)`: allows fused multiply-add in one function (1003, 1014, 1016) | yes | yes with its argument | yes | keep |
| `@layout` | `@layout(c)`: C field order (1100) | yes | yes | yes | keep |
| `@repr` | the integer type of an enum (1101-1102) | yes | partly (short for "representation", Rust's word) | yes | keep |
| `@view` | optional note that a type holds a view; wrong use is `E2030` (1121-1124) | yes | yes | yes | keep |
| `@parallel`, `@simd`, `@unroll` | on a `for` statement only (679-683) | yes | yes | yes | keep (all three) |
| `@allow` | silence named warnings (680) | yes | yes | yes | keep |
| `@deprecated` | `@deprecated(since, note)` (113-115) | yes | yes | yes | keep |
| `@assume_noalloc` | inside `unsafe` only: treat a call as not allocating (740) | yes | yes ("assume" says it is a promise) | yes | keep |
| `@name` | not an attribute: the grammar's placeholder in `@name(args)` (524) | n/a | n/a | n/a | keep (nothing to judge) |

### 2.4 Library types and functions that act as language words

| word | where and what | same? | plain? | on it? | verdict |
|---|---|---|---|---|---|
| `Box` | one-owner heap value (163, 620, 857, 1105-1106, 1114-1117, 1127, 1354-1355, 1383, 1435, 1442, 1476) | yes | no ("box" says nothing about one owner) | yes | **merge** into `owned T` (owner's ruling; section 3) |
| `Shared` | counted shared heap value (163, 1127, 1476) | yes | yes | yes | keep; agreed row 9's lowercase `shared` must mean exactly this |
| `Weak` | handle that does not keep its object alive (301, 1163) | yes | yes | yes | keep |
| `Cell` | value changeable through a shared path, whole value at a time, no check (185, 241, 1257, 1383-1386, 1425) | yes | weak, but no hidden cost or panic | yes | keep |
| `RefCell` | borrow counted while running; a second conflicting borrow panics (185, 241, 1257, 1387, 1429) | yes | **no** | yes | **rename** (failure 11) |
| `UnsafeCell` | the unchecked base of the cells (1257) | yes | yes ("unsafe") | yes | keep |
| `Span` / `MutSpan` | read-only / changeable pointer-and-length view (278, 295, 855, 918-919, 1118-1122) | yes | yes | yes | keep (both) |
| `str` | a borrowed view of UTF-8 text (449, 855, 916, 1120, 1130-1140, 1366) | yes | **no** (Python's `str` is owned) | yes | **rename** (failure 4) |
| `String` | an owned, growable text (163, 262, 266, 293-294, 450, 853, 916, 1367, 1407-1408) | yes | yes | yes | keep |
| `cstr` | a C string view, non-null (422, 1105-1108) | yes | yes | yes | keep |
| `Option`, `None`, `Some` | optional value (620, 924-926, 1104-1108, 1163, 1369, 1376-1377, 1458-1466) | yes | yes | yes | keep (all three) |
| `Result`, `Ok`, `Err` | success-or-error value (175, 620, 896, 946, 1028, 1301, 1369, 1465) | yes | yes | yes | keep (all three) |
| `Mutex`, `RwLock`, `Atomic` | the types documented to synchronise (170) | yes | yes | yes | keep (detail is reader-3's) |
| `Send` | may be moved to another thread (246, 861, 1251-1266) | yes | partly (Rust's word) | yes | keep |
| `Sync` | may be shared between threads (170, 246, 861, 1195, 1251-1266) | **no** across the spec: also the name of the effect `@nosync` forbids (line 3240) | partly | yes | keep the marker; rename the effect (failure 7) |
| `Copy` | values are copied bit for bit on assignment (167, 254, 838, 852, 1096-1097, 1113, 1118, 1255, 1267) | yes | yes | yes | keep |
| `Clone`, `.clone()` | interface and method making an independent copy (797-798, 1076, 1277-1278, 1354, 1439-1442) | yes | yes | yes | `Clone` keep; `.clone()` **merge** into the agreed `copied` (failure 12) |
| `.copied()` | iterator adapter that copies each item it reads (1451) | yes | yes | yes | keep: same meaning as the agreed keyword `copied` |
| `Drop` / `drop` | code run when a value is destroyed (1280-1281) | yes | yes | yes | keep |
| `Never`, `Sized` | type of an expression that never finishes; marker of a type with a known size (768, 858, 877, 922; 1258, 1261, 1438) | yes | yes | yes | keep (both) |
| `mem` | `mem.replace(name, v)` (1135); `mem.take` etc. are outside my slice | yes | yes | yes | keep |
| `Handle` | a class handle (1107, 1162) and `Handle[T]`, a checked pool index (244) | **no** | yes | yes | **rename** one meaning (failure 13) |
| `null`, `ref_to_ptr` | null raw pointer of an exact type; a raw pointer from a reference (1150, 1155-1156) | yes | yes | yes | keep (both) |
| `checked`, `clamped`, `new_unchecked` | build a range value: checked, clamped, unchecked inside `unsafe` (1029, 1033, 1042-1046, 1082) | yes | yes | yes | keep |
| `BorrowList`, `ArenaArray` | containers that may hold views bounded by their own region (1144-1145) | yes | yes | yes | keep |
| `debug_assert` | the one check a profile turns off (200) | yes | yes | yes | keep |

## 3. What the `Box` to `owned` merge must carry (for 05 and 07)

The owner ruled that `owned` is the one-owner field that replaces `Box` (`mesh: owned Mesh`). In my
slice `Box` does four jobs, each of which the new word must keep or move:
1. It is a **heap allocation the source shows** (`[PHIL-2]`, line 163, lists `Box` among the
   allocating forms). `mesh: Mesh` is already one-owner and stored inline; `mesh: owned Mesh` differs
   only in being stored separately. The word `owned` alone does not say "stored separately", so the
   rule must say it, and `[PHIL-2]`'s list must name `owned T` in a type.
2. It **holds an unsized value**: `Box[dyn I]` (857, 1435): becomes `owned dyn I`; reads right.
3. It **makes a recursive enum possible**: `Box[Tree]` payload (1115-1116), read through where a
   `Tree` is wanted (`[TYP-14]`).
4. It is **stripped once in method lookup** (1476) and has a niche, so `Option[Box[T]]` is one
   pointer (1105-1107); `Clone`/`Eq`/`Display` follow what it holds (1354-1357, 1383).

## 4. Today's compiler, real output

All files in `target/redesign3/01-reader-1/`, checked with `target/redesign3/bin/ember.exe check <file>`.

**t1_mut_pattern.em** (failure 2): `State.Walking(mut path):` then `path.pop()`. The pattern grammar
(821-829) has no `mut x` pattern; the compiler parses it, binds `path` read-only, and its help talks
about `.clone()` (agreed row 8 wants "write `State.Walking(ref mut path):`").
```text
error[E3021]: cannot write through a shared reference
 --> t1_mut_pattern.em:9:13
  |
9 |             path.pop()
  |             ^^^^^^^^^^ this is `ref Array[i64]`, which only reads
  |
  = help: a reference is not pointed elsewhere: this writes through it, into what it refers to; to keep another value, keep a copy of it (`.clone()`), or its index
  = note: while a shared borrow is live the owner may read and copy but not write [BRW-1]

error: could not compile due to 1 error(s)
```

**t2_owned_fn.em** (failure 1): `score = 0`, `cb = owned fn(): score += 1`, `run(cb)`,
`println(score)`. Accepted, no warning; by `[CLO-15]` the `int` is copied into the lambda.
```text
exit=0
```
**t2b_owned_fn_string.em**: the same form capturing a `String`: moved, not copied.
```text
error[E3040]: `name` is borrowed after it has been moved out of
 --> t2b_owned_fn_string.em:9:13
  |
7 |     cb = owned fn(): name.push_str("b")
  |          ------------------------------ moved here
8 |     run(cb)
9 |     println(name)
  |             ^^^^ borrowed here after the move
  |
  = help: borrow before the move, or clone the value so the move takes the copy
  = note: the borrow would point at memory whose owner gave it away
  = note: assigning a list, a map or a set moves it; in Python both names would point at one list (Appendix E)

error: could not compile due to 1 error(s)
```
(First draft of t2 passed the lambda straight to `run(mut f: fn())`: `E3027 a 'mut' argument is not
a mutable place`; calling a closure that changes its captures needs a `mut` callable parameter.)

**t3_ref_mut_param.em** (failure 9): `fn bump_ref(x: ref mut int): x += 1`.
```text
error[E3023]: cannot mutate borrowed parameter `x`
 --> t3_ref_mut_param.em:6:5
  |
5 | fn bump_ref(x: ref mut int):
  |             -------------- `x` is borrowed here
6 |     x += 1
  |     ^ this write needs mutable access
  |
  = help: restructure to a single owner and declare this parameter `mut x: ref mut i64`
  = help: if the value is genuinely shared, use `RefCell[T]` for runtime-checked borrows
  = help: if the value has identity, make it a `class`
  = note: `RefCell` adds a one-word runtime borrow-state check in every profile (CELL-5, CELL-9); a class adds a heap allocation, a 24-byte header, reference counting and dynamic exclusivity, and is not `@static_safe` (OBJ-1, RC-1, EXC-1, EFF-13)

error[E2020]: expected `ref mut i64`, found `i64`
  --> t3_ref_mut_param.em:11:14
   |
 5 | fn bump_ref(x: ref mut int):
   |             -------------- parameter `x` is declared `ref mut i64` here
...
11 |     bump_ref(n)
   |              ^ this is `i64`
   |
   = help: borrow it explicitly: `ref mut` before the place (TYP-5 rule 7 borrows only as `ref`)

error: could not compile due to 2 error(s)
```
**t3c_mut_ref_mut_param.em**: the help applied literally (`fn bump_ref(mut x: ref mut int)`,
`bump_ref(ref mut n)`): still refused, so the help breaks `[PHIL-8a]` (line 181).
```text
error[E2020]: `+=` is not defined on `ref mut i64`
 --> t3c_mut_ref_mut_param.em:3:5
  |
3 |     x += 1
  |     ^^^^^^
  |
  = note: a type has `+=` by implementing `AddAssign`, or `Add` with its own type as `Output` [TYP-21]

error[E3027]: a `mut` argument is not a mutable place
 --> t3c_mut_ref_mut_param.em:7:14
  |
7 |     bump_ref(ref mut n)
  |              ^^^^^^^^^ this value has no mutable storage for the callee to borrow
  |
  = help: bind the value to a local first, or pass the owner place directly (`f(obj.field)` rather than `f(obj.get_field())`)
  = note: the callee's declared `mut` mode forms the mutable borrow; the call site does not write a mode [FN-2a]

error: could not compile due to 2 error(s)
```
**t3d_ref_mut_local.em**: a `ref mut` *local* writes through as `[TYP-14]` says (`r = ref mut n`,
`r = 5`, `r += 1`): `exit=0`. So `ref mut` works on a local and not on a parameter.

**t6_ref_param_type.em**: `fn show_mode(x: int)` and `fn show_ref(x: ref int)`, both called with
`n`: `exit=0` (two spellings of one meaning).

**t4_let_local.em**: `let x = 5` in a function (a good message: names the fix).
```text
error[E0100]: `let` is not needed: assigning a name declares it
 --> t4_let_local.em:3:5
  |
3 |     let x = 5
  |     ^^^
  |
  = help: remove `let`
  = note: `x = 5` declares `x` where no `x` is in scope, and assigns it where one is [GRM-4]

error: could not compile due to 1 error(s)
```

**t5_class_mut.em** (failures 2, 3): a class with `fn add(self): self.n += 1`,
`fn add_held(mut self): self.n += 1` and a free function `fn bump(c: Counter): c.n += 1`. All
accepted: `exit=0`. No word on `self` or `c` says the object changes.

**t7_match_copy.em / t7b_match_copy.em** (`[GRM-13]`, line 838): `match s: Slot.Full(count): s =
Slot.Empty; println(count)`: accepted (`exit=0`; t7 adds only `warning[L1001]: 'names' is never
read`). `count` is a copy and keeps the old value after `s` changes: the same as Python, so the
default does not surprise; recorded because it is the one place a pattern binds a copy with no word.

**t8_str_field.em / t8b_str_field.em** (failure 4): `class Player: name: str`. Given a literal:
`exit=0`. Given a `String` parameter:
```text
error[E3063]: `str` is a view, so it may not be passed to `init`, which stores it where only a `static` view may go
 --> t8b_str_field.em:8:19
  |
8 |     return Player(s)
  |                   ^ stored by this call
  |
  = help: store an owned copy — `String` for `str`, `Array[T]` for `Span[T]` — and note that costs one allocation per element; or store a `u32` index or a `Handle[T]` and name the container it indexes
  = note: the view borrows `s`, and this place has no bounding region: only a view with the `static` region, such as a string literal, may be stored in it (TYP-15, LT-3)

error: could not compile due to 1 error(s)
```

## 5. Error messages that do not say what to write instead

Reader-4 owns the error table; this is what my slice and my tests show, for 04 and 05 to use.

**From my tests (real compiler messages), 4:**

| code | test | what is wrong | proposed message |
|---|---|---|---|
| `E3021` | t1 | help offers `.clone()` and indexes, not the line that does what was meant | "`path` only reads the NPC's list; to change that list through `path`, write `State.Walking(ref mut path):`" (agreed row 8) |
| `E3023` | t3 | the help's line (`mut x: ref mut i64`) does not compile (t3c) | "`x: ref mut int` is not a changeable parameter; to change the caller's `int`, write `fn bump_ref(mut x: int):`" |
| `E2020` | t3c | "`+=` is not defined on `ref mut i64`": no line to write | as for `E3023` above |
| `E3040` | t2b | "borrow before the move, or clone the value" names no exact line | "`name` was handed to the function at line 7; to keep using `name`, capture a copy: `fn() with copied name:`" (agreed row 6 spelling) |

**Diagnostics my slice names without a fix, 52** (code, line): `E0001` 315, `E0003` 329, `E0004`
339, `E0005` 401 (for `async`/`await`/`macro`; the raw spelling `r#name` is not offered), `E0008` 403,
`E0009` 457, `E0101` 458, `E0102` 782 and 788 (should offer `a in b and b in c`), `E0103` 765,
`E0104` 527-528, `E0107` 770, `E0108` 682, `E0109` 773 (`owned e` outside `for`/`match`: should say
"assigning `x = e` already hands it over"), `E0112` 808, `E0900` 189 and `E0901` 100 (no alternative
may exist), `E1020` 662, `E1030` 1486, `E2010` 429, `E2020` at 597, 892, 932 (should name the `as`
cast or `T.try_from`) and 992, `E2030` 1124 (should say "remove `@view`"), `E2040` at 1268 (should
name `@derive(Copy)` / `@sync`) and 1352, `E2041` 1185, `E2050` 1439 (should offer
`where Self: Sized`), `E2060` 600 and 1459 (should name the annotation), `E2061` 1467, `E2121` 1239,
`E2151` 991, `E2160` 670, `E2172` 760, `E2173` 762, `E2210` 1040, `E2211` 1044 (should offer
`clamped`), `E2212` 1090, `E2213` 602, `E2214` at 1058 (the compound form at 1067 has its fix-it),
`E2215` 1084 (should offer `checked` / `clamped`), `E2250` 448, `E2260` 624, `E2261` 1446, `E3100`
1264 (should say "write `unsafe extend`"), `E5042` 27, `E6001` and `E6004` 1206, `E9011` 1013,
`E9041` 1016, `W1002` 837. The spec text may be shorter than the compiler's real message; 04 checks
the compiler.

## 6. Must-cover words not in my slice (for the Builder)

Attributes: `@ffi` `@ser` `@borrows` `@noreload` `@must_drop` `@export` `@reflect` `@nolock` `@packed`
`@align` `@noio` `@inline` `@attribute` `@safety` `@export_table` `@error` `@test` `@renamed_from`
`@reinit_on_reload` `@never_specialize` `@gpu` `@soa` `@should_panic` `@reloadable` `@non_exhaustive`
`@noinline` `@no_derive` `@must_use` `@hot` `@grade` `@gpu_layout` `@from` `@default` `@component`
`@cold` `@always_specialize` `@allow_reload_terminate` `@prelude` `@no_runtime_checks` `@dataclass`
`@bench` `@allocator` (all appear only after line 1491). Library: `Retained`, `Pool`, `.copy()`,
`mem.take`, `mem.swap`, `mem.drop`. Every must-cover word that does appear in lines 1-1491 is judged
above.
