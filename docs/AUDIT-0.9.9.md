# Auditing the existing implementation against 0.9.9

**Owner, 2026-09-23:** "check the existing implementation too and make sure it complies to the newly
set standards". This is the checklist. Each row is probed with a minimal program (`docs/HANDOFF.md`
§0.0 E), and the finding is sorted the usual way: **compliant**, **defect** (rule clear, compiler
wrong: `DEFECTS.md`), **gap** (not built: this plan), or **ODR** (the text is ambiguous).

Status words: `compliant`, `defect D-nnn`, `gap`, `fixed`, `not yet probed`.

## A. Constructs 0.9.9 removed (Appendix H §H.2)

| Construct | 0.9.9 | Compiler, 2026-09-23 | Status |
|---|---|---|---|
| `::` paths | `E0100` with help `use '.' for paths` (`[LEX-21]`, `[GRM-24]`) | `E1010 this expression is not supported yet` | fixed: `E0100 `::` is not a path separator` with the help, in expressions, patterns and wherever else a `::` stops the parse; `.` reaches nested modules and their types and variants |
| `#! language "0.9.8"` | `E0006` unless the current version (`[VER-8]`) | accepted | fixed: only `0.9.9` is accepted; `E0006` says to delete the line; the std modules' directives are gone |
| `@latebound` | removed; its fresh-per-call regions are the default for every callable parameter (`[FN-6]`, `[LT-7]`) | accepted; ordinary callable parameters do not get fresh regions | fixed: `@latebound` is `E0104`. Probed first: an ordinary callable type already has `[LT-7]`'s meaning (its example compiles and prints `42`, `1`), so only the modifier had to go. The inert `latebound` flag in the types, MIR and interface records is still there, set by nothing |
| `@thread_local` | removed | accepted | fixed: `E0104`, noted as removed in 0.9.9 |
| `std.borrow`, `with_views*` | removed (`[LT-44]` elision replaces them) | module present and importable | fixed: `std/src/borrow.em` removed |
| `Callable[…]` in source | not source syntax (`[CLO-14]`) | `E1010 cannot find type` | compliant (code) |
| `PartialEq`/`PartialOrd` | removed (`[TYP-37]`) | `E1010 cannot find interface` | compliant |
| `unsafe(reason = …)` | removed (`[UNS-8]`) | `E0100` | compliant |
| `@derive(SoA)`, `columns_mut` | removed (`SoA[T]` is compiler-known) | not probed | not yet probed |
| manifest keys `exclusivity`, `overflow`, `bounds_checks`, `gpu.validate` | `E9001` (`[PRF-3]`) | not probed | not yet probed |

Tests that use a removed construct (31 files: `grep -rln "@latebound\|with_views\|Callable\[\|::\|PartialEq\|#! language" tests`)
are migrated or retired with the construct, each named in the progress log.

## B. Rules whose meaning changed in 0.9.9 (160 rules marked *(changed in 0.9.9)*)

| Rule | Part | Rule text (Appendix I) | Status | Notes |
|---|---|---|---|---|
| `[VER-1]` | ? |  | not yet probed | |
| `[TIER-1]` | I | Safe code MUST NOT invoke an operation with an unverifiable … | not yet probed | |
| `[PHIL-2]` | I | No heap allocation happens that the source does not show. The … | not yet probed | |
| `[PHIL-10]` | I | A program containing no `unsafe` block, no `unsafe fn` and no false … | not yet probed | |
| `[LEX-2]` | II | Line endings are LF or CRLF, both normalised to LF before … | ok | probed 2026-09-25: a CRLF file compiles and runs |
| `[LEX-10]` | II | There are no block comments. A line beginning `#!` before the first … | ok | probed 2026-09-25: a `#!` line after the first item is a comment |
| `[LEX-11]` | II | A `##` comment attaches to the next declaration, ignoring blank … | ok | probed 2026-09-25: a `##` comment before nothing, and one ending a code line, are silent |
| `[LEX-15]` | II | The table above is the complete reserved set. Contextual keywords are … | compliant | `async = 1` is `E0005` naming the reservation; `extend` is contextual (ODR-030, `LEX-15/accept_extend_is_contextual.em`) |
| `[LEX-22]` | II | Ember has no lifetime syntax and never will. A `'` begins a character … | fixed | probed 2026-09-25: `'ab` was `E0007` "named lifetimes are not supported in this version"; it is `E0008` (new, with a page), with a note naming `[LT-6]`'s restructurings; the retired `E0007` is no longer emitted |
| `[LEX-16]` | II | An integer literal without a suffix is an untyped integer: it takes … | fixed | `int` default |
| `[LEX-17]` | II | A float literal without a suffix is an untyped float: it takes the … | fixed | `float` default |
| `[LEX-17a]` | II | A float literal that receives `f32` or `f16` and has more significant … | fixed | `W2015` for literals given `f32`/`f16` with too many digits |
| `[LEX-19]` | II | An f-string `{…}` contains a full expression. `{{` and `}}` are … | not yet probed | |
| `[LEX-20]` | II | A string literal has type `str` with the static region. At a site … | not yet probed | |
| `[LEX-21]` | II | Tokenisation is maximal munch: `//=` before `//` before `/`, `=` … | **defect** | `//`, `//=` done; `::` is `E1010 not supported yet` instead of `E0100` with `use '.' for paths` |
| `[GRM-2]` | III | A file contains imports and items and, in the entry file only (the … | fixed | scripts |
| `[GRM-4]` | III | `x = e` where no `x` is in scope declares `x` with the type of `e`; … | fixed | probed 2026-09-24: redeclaration in one block compiled (D-240, fixed: `E1020`); `[CTL-10]` hoisting is built |
| `[GRM-19]` | III | The pattern of a `condition` MUST be refutable. An irrefutable one is … | fixed | probed 2026-09-24: the help was only "write `x = e` on the preceding line"; a bare name now gets `did you mean `x == 5`?` first (a fix-it), the declaration as a note |
| `[GRM-16]` | III | `return`, `break` and `continue` are expressions of type `Never`; … | fixed | probed 2026-09-24: a jump as a conditional-expression branch was refused; it is now that branch's arm (`v = 1 if b else return 7`); `1 + return 2` stays `E0107` |
| `[ENM-1]` | III | Inside a pattern whose scrutinee type is known, a variant may be … | compliant | bare variant names in patterns of a known enum |
| `[TYP-4]` | IV | No value converts implicitly between scalar types in an operator. … | fixed | probed 2026-09-24: the note appeared for any operands and the help said `x as T`; now only for two numbers, naming the narrower operand's cast (`a as i64`) |
| `[TYP-5]` | IV | Coercions — the complete list. At a coercion site (assignment or … | partial | rule 11 built: a `T` becomes `Some(value)` at a return, argument, initialiser or default, after rules 1–4, one level only; rule 7 built (D-216): a place auto-borrows for a shared `ref T` at every site, generic inference included; the other rules were built before 0.9.9 and are not re-probed |
| `[TYP-6]` | IV | `x as T` converts explicitly: * integer → integer: keeps the low bits … | **defect** | `1e20 as i32` and `NaN as i32` give `-2147483648` (C undefined behaviour); must saturate, NaN to 0 |
| `[TYP-8]` | IV | Integer overflow panics in every profile. An arithmetic operation (`+ … | **defect** | `release` wraps `i32::MAX + 1` to `-2147483648`; must panic in every profile |
| `[TYP-10]` | IV | Shifts. `a << n` and `a >> n` accept any integer type for `n`. If `n` … | **defect** | `x << y` with `y: i64`, `x: i32` is `E2020`; any integer type is allowed. Masked under `@overflow(wrap)` too (§C) |
| `[TYP-9c]` | IV | A toolchain that cannot honour `@fastmath` or `@fp(…)` for one … | not yet probed | |
| `[RNG-4]` | IV | The compiler tracks a known range for numeric expressions — literals, … | not yet probed | |
| `[RNG-5a1]` | IV | The operators on range types come from compiler-generated … | not yet probed | |
| `[TYP-13]` | IV | `Option[T]` has the size of `T` when `T` has a niche: a class handle, … | not yet probed | |
| `[TYP-14]` | IV | A reference, and a `Box[T]`, is read through wherever a `T` is wanted … | fixed | an operand reads through a `Box` as through a reference (`TYP-14/accept_a_box_operand_is_read_through.em`) |
| `[TYP-20]` | IV | Coherence is per package. An implementation of interface `I` for type … | not yet probed | |
| `[TYP-24]` | IV | An interface method is found through any implementation visible in … | fixed | probed 2026-09-24: `I.m(recv, …)` was `E1010`; built, and D-241 (two interfaces' methods shared one implementation) fixed on the way |
| `[HASH-2]` | IV | `std.collections.DefaultHasher` is a fixed-seed hasher: the same keys … | not yet probed | |
| `[HASH-3]` | IV | `Map` and `Set` MUST NOT weaken equality to compensate for an … | not yet probed | |
| `[TYP-23]` | IV | Inference is local to a function body and bidirectional. Function … | fixed | ODR-022; expected types reach generic calls, constructors and `Arena.alloc`; `x = None` and `xs = []` are left open and fixed by a later assignment, `push`/`insert`, or a site expecting a type (the body is checked again with the type known); one still open is `E2060` at its first use. `Map()` waits for `Map`. ODR-025: an unannotated lambda parameter takes `owned` (never `mut`) from the expected callable type; inference reads `Option`/`Result` instances (D-229); a ternary's `None`/`[]` branch takes the other branch's type (D-230) |
| `[TYP-26]` | IV | There is no overloading: two functions of one name in one scope are … | compliant | two `f` in one module is `E1030` |
| `[TYP-39]` | IV | Collections, tuples and `Option`/`Result` implement `Display` the way … | fixed | `Array`, views, fixed arrays, tuples (`(7,)`), `Option`, `Result`, nested, each element by its `Debug` (text quoted as Python's `repr` quotes it), in `print` and f-strings; `Map`/`Set` wait for `Map`. A class handle, which has `Debug` but no `Display` (`[TYP-36]`), prints its `Debug`, `<Token at 0x…>`, naming the object's own class (`[STD-9]`'s fallback). After `!r`/`!s`, a spec pads the text as in Python |
| `[TYP-15]` | IV | Where views may be stored. A view value may be stored only in a … | **defect** | D-198: the type `Array[str]` is rejected (`E3063`) at its formation, so the rule's own `names = ["ann", "bob"]` fails; the rule checks the stored values' regions, as `Box` already does. Open: needs heap-element views to carry the `static` region in the region model (DEFECTS D-198). D-199 fixed the other direction: an inferred `Array` (`[xs[2..]]`) stored non-static views unchecked; a list literal, `push`, `insert` and `a[i] = v` now require `static` views, and a view read out of an `Array` of that view type counts as one of its (static) elements |
| `[MOD-2]` | V | Items are private to their module unless marked. `pub(package)` makes … | fixed | probed 2026-09-24: a private function was callable through its module; now `E1052`, which every privacy error uses. A qualified type (`m.T`) was not supported in type position; built 2026-09-25 (`resolve_qualified_type`), with its visibility (`MOD-3/reject_a_type_its_module_does_not_show.em`) |
| `[MOD-3]` | V | `import a.b.c` binds the name `c` to module `a.b.c`, and `import … | fixed | probed 2026-09-25: `import math` was "cannot find module"; a standard module is now found without `std.` (the loader and the binder), a package module of the name first |
| `[MOD-5]` | V | The prelude. Every module implicitly imports these names from `std`, … | partly | `mem`, `len`, `range`, `min`/`max`/`abs`/`clamp`, `sum`/`any`/`all`, `sorted`, `input` and the range types are built; `enumerate`/`zip`/`reversed` only as `for` heads, and `Map`/`Set` not at all |
| `[FN-1]` | V | Parameter modes. `a: A` — borrowed (the default). The callee reads the caller's … | fixed | ODR-024: a borrowed parameter is the caller's place, passed by address (`T*`) unless `[BRW-8]` lets it be copied; `Cell`/`RefCell` writes reach the caller (D-209, D-210). `FN-1/accept_cell_and_refcell_through_borrowed_parameters.em` |
| `[FN-3]` | V | A function returns by move. Returning a reference or view requires its … | compliant | through `[LT-1]`/`[LT-1a]` as amended |
| `[FN-5]` | V | Default argument expressions are evaluated at each call, after the … | partial | defaults of plain functions and of methods, at direct calls: checked once where declared (in the declaring module, no caller locals), evaluated at each call after the written arguments, and skippable by named arguments. Not built (`E0900`): a default that reads an earlier parameter, and a default on a generic function or a generic type's method |
| `[FN-6]` | V | Callable types. A function is a value. A callable type is written … | not yet probed | |
| `[FN-8]` | V | `main` is `fn main()`, `fn main() -> Result[void, E]` for any `E: … | fixed | scripts; `Result` main unchanged |
| `[STR-2]` | V | A field default is any expression; it is evaluated at each … | fixed | probed 2026-09-24: defaults were never evaluated (D-238, fixed); a default in a struct never constructed is not checked |
| `[STR-5]` | V | Implicit derives. A struct or enum implements `Eq`, `Debug` and … | partial (D-187 fixed) | `==`/`!=` field-wise works; a component with a hand-written `eq` fails closed (`E2040`); implicit `Clone` is built, generic instances included and emitted only where called (`[COST-1]`), except for a type with its own `drop` (ODR-026), and `@no_derive(Eq)`/`(Debug)`/`(Clone)` opt out (any other argument is `E0104`; printing or comparing an opted-out type says so); implicit `Debug` is built in `[TYP-36]`'s forms (`Point(x=1)`, `Shape.Circle(1)`, `Mode.Fast`; a unit-only enum displays as its variant name), except for the compiler-known wrappers (`Box`, `Cell`, …) |
| `[CLS-2]` | V | `fn init(self, …)` is the constructor. Before any `init` body runs, … | **gap** | field defaults are not evaluated before a base `init` runs (`[CLS-11]` two-phase) |
| `[CLS-4]` | V | A class is final unless declared `open` or `abstract`. Methods are … | fixed | probed 2026-09-24: a method without `override` over an inherited one was not checked (ODR-028: `E2111`/`E2110`), and an `override` could not be overridden (D-239) |
| `[CLS-7]` | V | Inside a class method, `self` is a handle. Any method may read and … | **gap** | a plain `self` method writing `self.n += 1` is `E3023`; 0.9.9 lets any method write fields, each access checked |
| `[CLS-8]` | V | A class is `Sync` only as `[THR-1]` allows; every field of a `Sync` … | not yet probed | |
| `[IFC-1]` | V | `extend T:` without `implements` adds inherent methods to `T`; it is … | ok | probed 2026-09-25: `extend T:` adds inherent methods |
| `[STA-1]` | V | `static NAME: T = e` is one value per program with a stable address. … | partly | probed 2026-09-25: a `static` of a literal works; `static NAMES: Array[String] = []` is `E2130` (initialiser must be a literal), which `[STA-3]`'s lazy statics would lift |
| `[ATT-1]` | V | An attribute that is neither in the table below nor a visible … | not yet probed | |
| `[EXP-2]` | VI | An assignment evaluates its right side first, into a temporary if it … | ok | probed 2026-09-24: `a[i], a[j] = a[j], a[i]` swaps, `x, y = y, x` swaps, `a[i] += x` evaluates once |
| `[EXP-4]` | VI | A temporary created while evaluating an expression statement is … | partial | a `for` iterable's temporary under a slice or a view (`for x in make()[1..]:`) lives to the loop's end; one that a call's view result borrows (`for x in tail(make()):`) is still `E3020` (older than 0.9.9) |
| `[CTL-1]` | VI | `for pattern in e:` iterates: * a place `e` whose type is `Iterable`: … | partly | `Array`, `[T; N]`, `Span`, ranges and `str`/`String` (by `char`) iterate; `Map`/`Set` and generators do not exist yet |
| `[CTL-3b]` | VI | Iteration over ranges, `Span`, `MutSpan`, `Array`, `[T; N]`, `SoA` … | **gap** | `(0..10).step_by(3)` is `E1010 not supported yet` |
| `[CLO-2]` | VI | Captures are inferred per variable: read only ⇒ shared borrow; … | ok | probed 2026-09-25: a block lambda that writes a capture updates it (`count += 1` twice gives 2) |
| `[CLO-3]` | VI | What `fn(A) -> R` means depends on where it is written. * As a … | **gap** | The parameter form (implicit generic, monomorphised) is built, constructors included (D-235). The owned callable value of any other position holds only a function or a capture-free lambda: a capturing or `owned fn` lambda, or a callable parameter, cannot be stored in an `fn(...)` field, local or collection (`expected fn(...), found Callable0`). |
| `[CLO-4]` | VI | A non-`owned` lambda cannot outlive what it borrows: storing it, … | not yet probed | |
| `[CLO-6]` | VI | A lambda that moves one of its captures out of itself (into an … | not yet probed | |
| `[CLO-6a]` | VI | An owned `once fn` value, including one inside a `Box` or a … | not yet probed | |
| `[CLO-7]` | VI | Standard-library APIs that store or send a callback (`thread.spawn`, … | partial | ODR-025 removed `Option.map` from its list (Hardened_6); no callback-storing API is built yet |
| `[CORO-1]` | VI | A `gen fn` declares a generator. Calling it runs none of its body; it … | gap | probed 2026-09-25: `gen fn` parses (`[GRM-21]`) but is not checked or lowered: `Generator[Y]` is "cannot find type"; no frame, no `yield`, no `next`. Shares its core with generator expressions (`[GRM-38]`) and lazy adapters (`[STD-19]`) |
| `[CORO-3]` | VI | `Generator[Y, R]` in a signature names the function's own frame type … | gap | probed 2026-09-25: see `[CORO-1]` |
| `[CORO-6]` | VI | A reference or view to a local of the generator's own frame may not … | gap | probed 2026-09-25: see `[CORO-1]` |
| `[OWN-6]` | VII | `mem.take(mut place: T) -> T` (leaves `Default`), `mem.replace(mut … | not yet probed | |
| `[LT-1]` | VII | Signature elision. A source parameter (ODR-024) is: a parameter whose … | fixed | sources fixed by the declared signature, type parameters counted as `Copy` (D-213); a `mut` `Copy` parameter is not one (D-212); rule 1 for any borrowed receiver except a class handle (D-218, open, needs `[EXC-18]`); `E3060`/`E3062` split as ruled; a reference to a view parameter's own slot is `E3060` (D-217). `LT-1/` (thirteen cases) |
| `[LT-1a]` | VII | `@borrows(p, …)`, on its own line before the function, replaces the … | fixed | names a source or `mut` parameter; `E2031` for a borrowed `Copy` or an `owned` non-view parameter, including an `owned` arena. `LT-1a/` |
| `[LT-1b]` | VII | The opt-in lint `L3014` reports rule 3 applying to more than one … | fixed | `L3014` counted view-typed parameters only; it now counts source parameters, for free functions and for methods without a borrowed receiver. `LT-1b/warns_on_rule_3_over_source_parameters.em` (red when views alone are counted); reported once per declaration, not again for each type taking an interface default (`LT-1b/warns_once_for_an_interface_default_method.em`) |
| `[LT-44]` | VII | A borrowed or `mut` arena parameter is a source parameter because it is … | fixed | `LT-44/accept_one_arena_parameter_needs_no_borrows.em` |
| `[BRW-8]` | VII | A borrowed parameter is passed by address: the callee reads the caller's … | fixed | views pass as themselves; a `Copy` value holding a `Cell` by address (D-211); the receiver of a view-returning method by address; direct, generic, `dyn`, callable-value, default, operator and derived-`clone` calls. `BRW-8/accept_a_copy_struct_holding_a_cell_is_passed_by_address.em` |
| `[BRW-3]` | VII | Two-phase borrows. For a method call whose receiver is a place, or an … | fixed | D-223: arguments are read before activation (`f(v, v[0])` with `x: int` compiles), and a borrow live at activation is `E3021` (`x: ref int`, or a borrowed `String` passed by address). `BRW-3/` |
| `[BRW-4]` | VII | Disjoint fields. `ref mut a.x` and `ref mut a.y` may be live together … | not yet probed | |
| `[LT-4]` | VII | Arena allocations borrow the arena (`[ARN-1]`). | not yet probed | |
| `[LT-6]` | VII | Named lifetimes are not part of Ember and will not be added. Where a … | not yet probed | |
| `[LT-7]` | VII | Callable types. Each call through a value or parameter of callable … | fixed | a by-address parameter of a callable type is passed as a pointer; a call through a callable value borrows the callable type's source parameters, and a function whose `@borrows` reaches past them is not a value (D-219). `LT-7/` |
| `[DRP-4]` | VII | A panic inside `drop` aborts the process (`[PAN-1]`). A `drop` SHOULD … | not yet probed | |
| `[SPN-5]` | VII | `split_at(i)` on a `Span` returns two `Span`s; on a `MutSpan` it … | not yet probed | |
| `[OBJ-1]` | VIII | The header is 24 bytes on 64-bit targets and is part of the runtime … | not yet probed | |
| `[RC-3]` | VIII | Further elisions are allowed only when semantics are preserved … | not yet probed | |
| `[RC-4]` | VIII | A non-`Sync` class's counts, and a `Shared`'s, use plain loads and … | not yet probed | |
| `[EXC-1]` | VIII | Beginning a write access to a field while any access to the same … | **defect** | D-202: a view of a class field (`v = b.items[1..]`, or `v: Span[int] = b.items`) begins no read access, so `c.items.clear()` through another handle frees what `v` points into (ASan: heap-use-after-free) |
| `[EXC-2]` | VIII | Beginning a read access to a field while a write access to it is … | not yet probed | |
| `[EXC-6]` | VIII | A panic from `[EXC-1]`/`[EXC-2]` names both the offending access and … | not yet probed | |
| `[EXC-7]` | VIII | The opt-in lint `L3013` reports a long-term access held across a … | not yet probed | |
| `[EXC-14]` | VIII | The 0.9.8 `exclusivity = "unchecked"` setting is removed. Code that … | not yet probed | |
| `[WK-11]` | VIII | `Weak(h)` creates a weak handle to a class object or a `Shared` or … | not yet probed | |
| `[WK-12]` | VIII | `w.upgrade() -> Option[O]` returns a retained strong handle while the … | compliant (single thread) | `Weak.upgrade` of a live object; `@sync` compare-exchange not probed |
| `[HEAP-4]` | IX | `s.get() -> ref T` borrows the payload: it begins a checked read … | not yet probed | |
| `[HEAP-5]` | IX | `s.get_mut() -> ref mut T` begins a checked write access (`[EXC-1]`) … | compliant (partial) | overlapping `get_mut` with loans that end at last use does not panic, as `[EXC-18]` says; a live overlap not yet probed |
| `[HEAP-7]` | IX | `Weak[O]` exists for `O` a class handle, a `Shared[T]` or a … | not yet probed | |
| `[ARN-4]` | IX | A growing `Arena` carries `Alloc` on its growth path. `FixedArena` … | not yet probed | |
| `[ALC-3]` | IX | A binary package may replace the global allocator with `[build] … | not yet probed | |
| `[UNS-8]` | IX | Every `unsafe:` block and `unsafe fn` carries a safety note … | not yet probed | |
| `[HND-2]` | IX | `Handle[T]` is a `u64`: 32 bits of index and 32 of generation. … | not yet probed | |
| `[DSJ-1]` | IX | `assert_disjoint(a, b) -> Result[(A, B), (A, B)]` compares the two … | not yet probed | |
| `[EFF-2]` | X | A call through a callable parameter (a generic bound, `[CLO-3]`) … | not yet probed | |
| `[EFF-9]` | X | `RuntimeCheck(k)` enters a function's effect set when a check of kind … | not yet probed | |
| `[EFF-15]` | X | Effects and contract verdicts are computed once, after the … | not yet probed | |
| `[EFF-16]` | X | `@nopanic(explicit)` forbids only the panics the programmer writes … | not yet probed | |
| `[DET-2]` | X | `Nondet` is introduced by exactly: floating-point contraction, … | not yet probed | |
| `[COST-3]` | X | The costs. | not yet probed | |
| `[THR-1]` | XI | A class is `Sync` only when it is declared `@sync class`. Every other … | not yet probed | |
| `[THR-2]` | XI | Handles of a `@sync` class are `Send` and `Sync`. A handle of any … | not yet probed | |
| `[THR-3]` | XI | `Mutex[T]` is a value type. `m.lock()` blocks and returns a … | not yet probed | |
| `[THR-6]` | XI | A type whose `drop` a safety guarantee depends on is `@must_drop`. A … | not yet probed | |
| `[THR-5]` | XI | `thread.scope()` returns a `Scope`, which MUST be bound by a `with` … | not yet probed | |
| `[PAR-3]` | XI | `@parallel(reduce=[total: +, best: max])` declares variables combined … | not yet probed | |
| `[PAR-4]` | XI | A `@parallel` loop has the effects `Sync` and `Block` (it waits for … | not yet probed | |
| `[JOB-2]` | XI | `jobs.scope()` follows `[THR-5]` and `[THR-11]`: jobs submitted to a … | not yet probed | |
| `[JOB-3]` | XI | `s.submit_after(deps, f)` starts `f` only after the jobs in `deps` … | not yet probed | |
| `[SOA-1]` | XII | `SoA[T]` is a compiler-known type constructor, like `Cell`: for any … | not yet probed | |
| `[SIMD-3]` | XII | Alias facts given to the backend (`restrict`) MUST be derived, never … | not yet probed | |
| `[SIMD-5]` | XII | Vectorisable form, which `@simd(assert)` checks, is computed by Ember … | not yet probed | |
| `[ECS-1]` | XII | `Entity` is a 32-bit generational handle: a 20-bit index and a 12-bit … | not yet probed | |
| `[ECS-3]` | XII | `Query[(A, Mut[B], Option[C], Not[D])]` is a view over a world's … | not yet probed | |
| `[ECS-4]` | XII | A query's read and write sets are compile-time constants computed … | not yet probed | |
| `[ERR-1]` | XIII | `Option[T]` is absence: `Some(v)` or `None`. `Result[T, E = … | not yet probed | |
| `[ERR-2]` | XIII | `e?` on a `Result[T, E]` in a function returning `Result[U, F]` … | not yet probed | |
| `[ERR-3]` | XIII | `Error` is the interface of error types: `Display + Debug` with … | not yet probed | |
| `[ERR-8]` | XIII | `AnyError` is the prelude's "any error" type: an owned, boxed `dyn … | not yet probed | |
| `[CT-1]` | XIV | These are evaluated during compilation: `comptime(e)`; `comptime:` … | not yet probed | |
| `[CT-2]` | XIV | An operation that cannot run at compile time — an `extern` call, a … | not yet probed | |
| `[CT-4]` | XIV | Compile-time evaluation is deterministic by construction: the … | not yet probed | |
| `[CT-5]` | XIV | Where results live. A compile-time result is placed in the image as … | not yet probed | |
| `[RFL-3]` | XIV | User attributes are declared. A struct marked `@attribute` may be … | not yet probed | |
| `[DRV-1]` | XIV | `@derive(…)` requests generated implementations. The built-in … | not yet probed | |
| `[TXT-4]` | XV | Slicing a string, `s[a..b]`, is by byte offset and panics in every … | not yet probed | |
| `[STD-5]` | XV | `sum` and `product` combine elements left to right in the element … | not yet probed | |
| `[FFI-1]` | XVI | Every foreign declaration has a contract: for each pointer-typed … | not yet probed | |
| `[FFI-2]` | XVI | A call to a foreign function whose contract contains an `unknown` … | not yet probed | |
| `[FFI-10]` | XVI | An `unsafe extern "C":` block declares foreign functions, statics and … | not yet probed | |
| `[FFI-8]` | XVI | Type mapping. | not yet probed | |
| `[FFI-11]` | XVI | Contract vocabulary. A pointer contract has five axes; mutability … | not yet probed | |
| `[FFI-21]` | XVI | A C function-pointer parameter accepts a capture-free Ember function … | not yet probed | |
| `[FFI-22]` | XVI | Every exported function and trampoline first attaches the calling … | not yet probed | |
| `[FFI-25]` | XVI | A panic in an exported function, or anywhere below it, aborts the … | not yet probed | |
| `[BLD-FFI-2]` | XVI | The toolchain ships a CMake module: `ember_add_library(name KIND … | not yet probed | |
| `[CLI-4]` | XVII | `ember run file.em` and `ember build file.em` accept a single file … | not yet probed | |
| `[MAN-1]` | XVII | An invalid manifest, including one with an unknown key, is `E9001`, … | not yet probed | |
| `[MAN-3]` | XVII | Every key in `[lints]` names a lint the compiler defines (`E9010` … | not yet probed | |
| `[BLD-10]` | XVII | The compile-time budgets below are release gates. | not yet probed | |
| `[PRF-1]` | XVII | A profile never changes what a program means. Every profile accepts … | not yet probed | |
| `[TST-11]` | XVII | Besides the per-rule cases, the suite covers these scenarios: … | not yet probed | |
| `[TST-6]` | XVII | Appendix A's code is generated from a fixture that is compiled in CI. … | not yet probed | |
| `[TST-7]` | XVII | Every ` ```ember ` block in this document is extracted and must pass … | not yet probed | |
| `[FMT-1]` | XVII | `ember fmt` writes LF line endings, four-space indentation and at … | not yet probed | |
| `[CONF-1]` | XVII | A compiler declares the profile it implements, and claims it only … | not yet probed | |
| `[CG-C-1]` | XVIII | The emitted C has no undefined behaviour. Checked signed arithmetic … | not yet probed | |
| `[CG-C-2]` | XVIII | Accepted programs compile. A program Ember accepts never produces C … | not yet probed | |
| `[CG-C-4]` | XVIII | Aliasing facts. For each loop, the base pointer of every view whose … | not yet probed | |
| `[MNG-1]` | XVIII | Mangling is injective. A symbol is `em_` followed by each path … | **gap** | the compiler still writes `em_` plus the path with `.` as `_`, which is not injective (`a_b.c` and `a.b_c`), and a user function named `fmt_0` or `eq_0` collides with the generated `em_fmt_0`/`em_eq_0` helpers; MNG-1's length-prefixed components fix both, since a user symbol then always has a digit after `em_` |
| `[RT-1]` | XVIII | The runtime `ember_rt` is C11 depending on libc and the OS only. … | not yet probed | |
| `[RT-7]` | XVIII | A reference count never wraps: a retain that would overflow panics … | not yet probed | |
| `[HR-14]` | ? |  | not yet probed | |
| `[HR-35]` | ? |  | not yet probed | |
| `[HR-17a]` | ? |  | not yet probed | |
| `[HR-20]` | ? |  | not yet probed | |
| `[FFI-44]` | ? |  | not yet probed | |
| `[FFI-17a]` | ? |  | not yet probed | |
| `[FFI-24]` | ? |  | not yet probed | |
| `[FFI-39d]` | ? |  | not yet probed | |
| `[TCB-1]` | ? |  | not yet probed | |
| `[CXX-1]` | ? |  | not yet probed | |
| `[GPU-1]` | ? |  | not yet probed | |
| `[GPU-4]` | ? |  | not yet probed | |

## C. Gaps found while implementing (not a changed rule)

| Where | 0.9.9 | Compiler | Status |
|---|---|---|---|
| `for x in span` | `[CTL-1]`: `Span` is iterable | `E2040 Span[i64] cannot be iterated` | fixed: a shared `Span` and a fixed array iterate as an `Array` does (a `MutSpan` still does not); `for x in [1, 2]` works (D-193) |
| `@overflow(wrap)` and shifts | `[TYP-10]`: an out-of-range shift panics in every profile | masked under `@overflow(wrap)` | defect |
| `Result[T]` | `[ERR-1]`: `Result[T, E = AnyError]` | `E2020 Result takes 2 type arguments` | gap (`AnyError` not built) |
| `Option.unwrap`, `unwrap_or`; `String.char_count`; `Array.sort` | Part XV (`[STD-15]` …) | `E1010 has no method` | partial: `Array` has `is_empty`, `contains`, `index_of`, `get`, `first`, `last`, `sort` (stable; any `T: Ord`, D-256), `sorted` (`Clone` elements), `reverse`, `pop`, `remove`, `insert`, `clear`, and (2026-09-25) `capacity`, `reserve`, `truncate`, `swap_remove`, `swap`, `extend` (from a span, cloning), `get_mut`, `split_at`, `iter`, `iter_mut`, `chunks`, `chunks_mut` (through a view), `windows` (shared overlapping views) and `drain(r) -> Array[T]` (any integer range) (ODR-031), and `retain`, `dedup`, `binary_search`, `join`, `sort_by`, `sort_by_key` written in Ember in `std.core` (`[STD-15]`; every method `[STD-15]` lists is now built; `sort`/`sorted` of other `T: Ord` go to `std.core`, D-256); `x in c` for collections, text and ranges (`[STD-8]`); `[ERR-4]`'s `is_some`/`is_none`, `is_ok`/`is_err`, `unwrap`, `expect`, `unwrap_or`, `unwrap_or_default`, `ok`, `err`, `ok_or` are built for both wrappers, each as a `match` (a failed `unwrap`/`expect` panics with the error's text, after `expect`'s message); the methods that take a function (`map`, `map_err`, `and_then`, `or_else`, `unwrap_or_else`, `ok_or_else`, `filter`) are `std.core` generics the type checker routes to (ODR-025; `fn(…)` for `once fn`, DEVIATIONS D6); `clone` for `Array` and `String`, and derived `Clone` over them (D-228); `take`/`replace`/`as_ref`/`as_mut`/`iter`/`context` and most of Part XV are not |
| a range as a value (`r = 0..=2`) | `[TYP-*]`: `Range`/`RangeInclusive` are prelude types | `E1010 this expression is not supported yet` outside a `for` | fixed (ODR-027): the four prelude range structs, `for` over a range value and over `a..`, `x in r`, `len(r)`, `range(n)`/`range(a, b)` as values |
| a float's text | `[STD-20]`: Python's `repr` (`1.0`, `1e+300`, `0.30000000000000004`) | the shortest round trip in `%g` form: `1` for `1.0`, the C library's exponent and NaN spellings | fixed: the shortest digits, fixed notation in [1e-4, 1e16) with `.0` on integral values, exponent notation otherwise, `nan`/`inf`/`-inf`; 13 corpus expectations moved from `9` to `9.0` |
| `f"{x:>8}"`, `{x=}`, `{x!r}` | `[LEX-19]`: Python's format spec, debugging form and `repr` | `E1010 a format spec is not supported yet` | fixed for numbers, text, `bool`, `char`, and (`[TYP-39]`) collections, tuples, `Option` and `Result`, which take no spec but `!r`/`=`; structs and enums print by their implicit `Debug` (`[STR-5]`), a unit-only enum by its variant name |
| `[x * x for x in xs if x > 0]`, `sum(x for x in xs)` | `[GRM-27]`, `[GRM-38]` | comprehensions did not parse | fixed for `Array` comprehensions over collections and ranges, and generator expressions as the argument of `sum`, `any`, `all`; `{…}` comprehensions wait for `Map`/`Set`, and a generator anywhere else is `E0900` |
| a type in a diagnostic | the type as written (`Option[int]`) | fixed (D-232): `Option[i64]`, `Range[u8]`, `Cell[i32]`; an alias shows its target (`int` is `i64`) |
| `a ** b` | `[TYP-30]`: exact integer powers, float `pow` | fixed: exact integer powers (square and multiply through the checked `*`), `E2151` for a literal negative exponent and a panic for a computed one, float `pow`; `a **= b`. An untyped base is an `int` unless the exponent is a float, so `y: u8 = 2 ** 3` is `E2020` (write `2u8 ** 3`) |
| `x if c else y` | Part III ternary (level 3) | `E1010 this expression is not supported yet` | fixed: a two-arm `match` on the condition |
| `xs[-1]` | `[TYP-31]`, `[LEX-24]`: a negative literal index is `E2011` | compiles; the literal wraps to `usize` and panics with index 18446744073709551615 | fixed: `E2011` with both fix-its; a computed negative index panics as `index -1` |
| `xs[n]` with `n: i32`; `n: int = xs.len()` | `[TYP-31]`: any integer index; `len()` is `int` | `E2020 expected usize` both ways | fixed: `len()`/`capacity()` are `int`; indices, sizes and counts take any integer |
| `-x as u8` | Part III precedence table (0.9.9): prefix `-` (16) binds tighter than `as` (15), so it is `(-x) as u8` | parsed as `-(x as u8)`; `-3.5 as u8` printed `253` | fixed (binding powers follow the 0.9.9 table) |
| `panic(…)`, `todo()`, `unreachable()` | `[MOD-5]` prelude functions | `E1010 cannot find panic` | fixed: type `Never`, a MIR assertion that cannot hold, then a block nothing reaches; `assert`, `assert_eq`, `assert_ne`, `debug_assert` (compiled out outside `debug`, `[PRF-3]`; `W2016` not built), `eprint`/`eprintln`. `[STD-26]`: `len` (`E2073` for a string), `min`, `max` (IEEE totalOrder for floats, `[TYP-37]`), `abs`, `clamp` (panics when `lo > hi`, `[ERR-13]`), `sum`, `any`, `all` over collections, and `range` (all three forms), `enumerate`, `zip`, `reversed` in a `for` header, as counted loops. Still missing: `input`, `format`, `sorted`; these functions over ranges, iterators and generator expressions (`E0900`); `enumerate`/`zip`/`reversed` and a stepped `range` as values (`E0900`); `range(n)` and `range(a, b)` are values since ODR-027 |
| `input(prompt)` | `[STD-10]`: print the prompt, flush, return one line without its ending; end of input panics naming `std.io.stdin().read_line()` | `E1010 cannot find input` | fixed: `ember_input` in the runtime, the panic at the call; run tests feed standard input with `#$ stdin:` (`STD-10/`) |
| `==` on `str`, `String`, structs, tuples, arrays, payload enums | `[STR-5]` implicit `Eq`, field-wise | accepted, then invalid C | fixed (D-187); lexicographic `<` on tuples/arrays/`Array` and `None < Some` still `E2040` |
| float `as` integer | `[TYP-6]` saturates, NaN to 0 | C's undefined conversion | fixed (D-188) |
| `ListView()` for a derived class with no `init` | `[CLS-10]`: it gets its base's constructor | fixed: the nearest base `init` runs on the new object after its field defaults; a derived field with no default is `E2101` at the declaration |
| `mem` in the prelude | `[MOD-5]`: `mem` (the module `std.mem`) is a prelude name | `E1010 cannot find mem` | fixed: `std.mem` is always loaded and `mem` is bound in every module; an import replaces it |
| falling off a non-`void` function | `[FN-10]`: only `void`/`Result[void, E]` have an implicit value | accepted; the C returned an uninitialised slot | fixed (D-186, `E2182`) |
| unknown attributes | `[ATT-1]`: `E0104`; `[ATT-6]`: a listed attribute whose effect is not built is `E0900`, never accepted and ignored | `@thread_local` (and any unknown name) on a `static` is accepted silently | fixed: every attribute is checked against the table; unknown, reserved and misplaced ones are `E0104`, unbuilt ones `E0900` (built: `@derive(Copy, Clone, Eq)`, `@repr` on enums, `@layout(c)`, `@view`, `@static_safe`, `@overflow`, `@borrows`) |
| `xs[a..b]` on a `Span` | `[TXT-4]`, `[SPN-2]`: slicing | `E1010 this expression is not supported yet`, then a cascading `E2020` | fixed: `a[i..j]`, `a[..j]`, `a[i..]`, `a[..]`, `a[i..=j]` of an `Array`, a fixed array, a `Span` (a shared `Span`) or text (a `str`, by byte offset, panicking off a character boundary); bounds checked once when the slice is taken; the slice borrows its source. Not built: slicing a `MutSpan`, and `s.get(a..b)` (`[TXT-4]`'s total form) |
| a run of invalid characters | `[DIA-20]`: one diagnostic | fixed: one `E0100` per run, and the parser adds none for the token the lexer rejected (`[DIA-14]`) |
| a view of a local returned | `[DIA-14]`: only the first error of a cascade | `E3021 cannot be written while it is borrowed` and then `E3060` for one `return view(tmp)` | fixed (D-190) |
| a panicking accessor's message | `[ERR-13]`: names its non-panicking twin (`index 7 out of range for length 3; use .get(i) for an Option`) | `index 7 is out of bounds for a length of 3`, shared by indexing, `split_at`, chunk sizes and `insert` | gap: each site needs its own twin |
| the conformance harness | `[TST-1]`: unexpected and missing diagnostics both fail | only missing ones fail, so a cascade or a second error for one mistake passes unnoticed | fixed (D-205): the harness reads `--json`, each annotation claims one diagnostic (a trailing one on its own line), and an unclaimed diagnostic fails; the corpus was migrated, which found D-206–D-208 |
| an `owned fn` returning a view of its own capture | `[LT-42]`, `[CLO-*]` do not say whether calling an `owned fn` keeps its captures alive for the result | `E3060 env.values does not live long enough` | not yet probed against the rules; the old `@latebound` test for it was retired |
| `extend[T]` of a generic struct, enum or built-in type | `[GRM-34]`: `extend [T: B] Array[T] implements I:` declares a generic extension | `E1010 cannot find type T` on anything but a generic class; an `Array`'s calls never reached an extension | fixed (D-242): every generic type, `Array`/`Span`/`Option`/`Result` included; the compiler's own `Array` methods first (`[TYP-24]`) |
| an inherent and an interface method of one name | `[TYP-24]`: they coexist; `I.m(x)` names the interface's | the interface's was dropped and its body compiled under the inherent symbol (a C redefinition) | fixed (D-244) |
| inherent methods on another package's type | `[IFC-2]`: `E2120` | accepted (`extend i64: fn twice(self)`) | fixed (D-243) |
| a bound's method in a generic body | `[TYP-17]`: only what the bound provides | an instantiation re-checks the body on the concrete type, where an inherent method of the name comes first | fixed for generic functions (D-245) and for generic types' and extensions' methods (D-248, D-250) |
| bounds met by the compiler | `[TYP-17]`, `[TYP-37]`, `[STR-5]`: `i64: Ord`, `String: Clone`, a struct's implicit `Eq` | `E2040 i64 does not implement std.core.Ord` for `fn f[T: Ord]` | fixed (D-246): also `x.clone()` of a `Copy` value, `cmp`, `Ordering` in the prelude, `min`/`max` of text |
| a public method of a generic extension | `[BLD-2]`: the module's interface is its declarations | each instance's specialization was recorded; two programs sharing a cache disagreed (an internal error) | fixed (D-247) |
| `m.T` and `m.Point(…)` through a module | `[MOD-3]`: `import a.b.m` binds `m`; its items are named through it | `E1010` for a type in type position, and for a struct or class constructed through the module | fixed: `resolve_qualified_type` (with arguments, `m.Pair[int]`) and constructors in `synth_qualified_call` |
| bounds on `Display`, `Debug`, `Copy` | `[TYP-36]`: a bound the table does not meet is `E2040` | waved through for any type (the interfaces are undeclared) | fixed (D-249) |
