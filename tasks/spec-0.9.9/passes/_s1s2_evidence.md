| Finding | Sev | Rule | Rule text (start) |
|---|---|---|---|
| F-002 | S2 | `[TYP-27]` | `()` is the value of type `void`; `Ok(())` is the success value of `Result[void, E]`. A function with no `->` returns `void`.… |
| F-003 | S2 | `[MOD-3]` | `import a.b.c` binds the name `c` to module `a.b.c`, and `import a.b.c as d` binds `d`. `from a.b import x, y as z` binds items. A module path that na… |
| F-004 | S2 | `[MOD-3]` | `import a.b.c` binds the name `c` to module `a.b.c`, and `import a.b.c as d` binds `d`. `from a.b import x, y as z` binds items. A module path that na… |
| F-004 | S2 | `[PHIL-12]` | **No silent acceptance.** Every construct a program writes — an attribute, an import, a directive, a contract, a manifest key, a statement form — eith… |
| F-012 | S1 | `[STD-9]` | `print(a, b, …, sep=" ", end="")` and `println(a, b, …, sep=" ", end="\n")` write their arguments' `Display` text to standard output separated by `sep… |
| F-012 | S1 | `[CG-C-2]` | **Accepted programs compile.** A program Ember accepts never produces C that the C compiler rejects or warns about; such a case is a compiler defect (… |
| F-012 | S1 | `[TST-27]` | **The C gate.** Every accepted program in the test suite is compiled through the C compiler with warnings as errors on each supported host compiler. A… |
| F-013 | S2 | `[TST-27]` | **The C gate.** Every accepted program in the test suite is compiled through the C compiler with warnings as errors on each supported host compiler. A… |
| F-013 | S2 | `[CG-C-2]` | **Accepted programs compile.** A program Ember accepts never produces C that the C compiler rejects or warns about; such a case is a compiler defect (… |
| F-014 | S2 | `[CTL-10]` | **Names assigned in every branch.** In an `if`/`elif`/`else` that has an `else`, or an exhaustive `match` statement, a name that is not in scope befor… |
| F-029 | S2 | `[TYP-18]` | Generic arguments are inferred from the arguments of a call. Explicit arguments may be written: `f[int](x)`, `Array[f32]()`. A constructor's explicit … |
| F-031 | S2 | `[TXT-9]` | **A string literal initialises a `String`.** Wherever a `String` is expected — an initialiser, an argument, a field, a collection element, a return va… |
| F-031 | S2 | `[TXT-11]` | **`String` operations.** Everything `str` has (by read-through), plus `String()`, `with_capacity(n)`, `push(c: char)`, `push_str(s)`, `insert`, `remov… |
| F-046 | S2 | `[VER-8]` | Before 1.0 there is exactly one language: the current one. A source file does not select a language version, and `#! language` directives are accepted… |
| F-048 | S2 | `[MOD-5]` | **The prelude.** Every module implicitly imports these names from `std`, and this list is the only definition of the prelude:… |
| F-055 | S2 | `[STA-1]` | `static NAME: T = e` is one value per program with a stable address. A `static` is immutable; mutation goes through a `Sync` interior type (`Atomic`, … |
| F-055 | S2 | `[STA-3]` | A static whose initialiser can be evaluated at compile time is placed in the image with no initialisation code. Any other initialiser runs **once**, o… |
| F-058 | S2 | `[CLO-3]` | **What `fn(A) -> R` means depends on where it is written.** * As a **parameter type** it is a bound, not a representation: the parameter is an implici… |
| F-058 | S2 | `[CLO-10]` | An owned callable value holds up to **three pointer-sized words** of captured state inline and makes no allocation; larger captured state is placed in… |
| F-059 | S2 | `[CLO-3]` | **What `fn(A) -> R` means depends on where it is written.** * As a **parameter type** it is a bound, not a representation: the parameter is an implici… |
| F-062 | S2 | `[CORO-12]` | A `gen fn` **method of a class** takes `self` (the frame retains the handle). Each access to the object inside it is checked on its own; a long-term a… |
| F-067 | S2 | `[BRW-10]` | **Methods borrow the fields they use.** A call to a method of a struct or enum that is not visible outside its package and not `virtual` borrows only … |
| F-073 | S2 | `[CLS-7]` | Inside a class method, `self` is a handle. **Any method may read and write the object's fields through `self`**; each access is checked on its own by … |
| F-073 | S2 | `[FN-9]` | A mode on a **class-handle** parameter governs the handle, not the object. A borrowed handle parameter may still be used to read and write the object'… |
| F-078 | S1 | `[THR-1]` | **A class is `Sync` only when it is declared `@sync class`.** Every other class is neither `Sync` nor `Send`, whatever its fields, and uses plain (non… |
| F-078 | S1 | `[THR-13]` | **The data-race guarantee.** In Safe Ember a memory location is reachable from two threads only through (a) a `@sync` object, whose fields are immutab… |
| F-079 | S1 | `[EXC-16]` | Assigning a value to a class field whose type is not `Copy` is a write access (table above): it conflicts with, for example, a `for` loop over the sam… |
| F-080 | S2 | `[TXT-9]` | **A string literal initialises a `String`.** Wherever a `String` is expected — an initialiser, an argument, a field, a collection element, a return va… |
| F-080 | S2 | `[TXT-11]` | **`String` operations.** Everything `str` has (by read-through), plus `String()`, `with_capacity(n)`, `push(c: char)`, `push_str(s)`, `insert`, `remov… |
| F-081 | S2 | `[CLS-7]` | Inside a class method, `self` is a handle. **Any method may read and write the object's fields through `self`**; each access is checked on its own by … |
| F-081 | S2 | `[FN-9]` | A mode on a **class-handle** parameter governs the handle, not the object. A borrowed handle parameter may still be used to read and write the object'… |
| F-085 | S2 | `[TST-7]` | Every ` ```ember ` block in this document is extracted and must pass `ember check` (`--syntax-only` for blocks that name items they do not declare, ma… |
| F-088 | S2 | `[HEAP-1]` | Every heap type allocates through `ember_alloc`/`ember_realloc`/`ember_free` and carries the `Alloc` effect where it allocates.… |
| F-088 | S2 | `[STD-15]` | **`Array[T]`** is a growable contiguous list (Python's `list`). `len`, `is_empty`, `capacity`, `reserve`, `push`, `pop -> Option[T]`, `insert(i, v)`, … |
| F-088 | S2 | `[STD-16]` | **`Map` operations.** `m[k]` reads the value and panics when the key is missing (`key not found: <Debug of k>; use .get(k) for an Option`); `m[k] = v`… |
| F-093 | S2 | `[STD-11]` | **`Map[K, V, H = DefaultHasher]`** is a hash map that **iterates in insertion order**, like Python's `dict`. Re-assigning an existing key keeps its po… |
| F-093 | S2 | `[DET-2]` | `Nondet` is introduced by exactly: floating-point contraction, reassociation or any `@fastmath` relaxation; a transcendental function from the platfor… |
| F-100 | S2 | `[CLI-4]` | `ember run file.em` and `ember build file.em` accept a single file with no manifest, as a package named after the file with default settings. A first … |
| F-100 | S2 | `[DOC-2]` | The user guide (`docs/book/`), with a "coming from Python" chapter built on Appendix E, is published with each release, and every sample in it is run … |
| F-100 | S2 | `[TST-8]` | `tests/firstweek/` holds at least 24 first-draft programs a newcomer plausibly writes in week one (a text adventure, a CSV summariser, a scene graph w… |
| F-102 | S2 | `[TYP-8]` | **Integer overflow panics in every profile.** An arithmetic operation (`+ - * // % **`, unary `-`, and the compound assignments) whose mathematical re… |
| F-102 | S2 | `[CG-C-1]` | **The emitted C has no undefined behaviour.** Checked signed arithmetic uses the compiler's overflow builtins (`__builtin_add_overflow` and friends; c… |
| F-104 | S1 | `[THR-1]` | **A class is `Sync` only when it is declared `@sync class`.** Every other class is neither `Sync` nor `Send`, whatever its fields, and uses plain (non… |
| F-109 | S2 | `[CT-5]` | **Where results live.** A compile-time result is placed in the image as constant data. A value that owns heap memory (`Array`, `String`, `Map`, `Box`,… |
| F-115 | S1 | `[CONF-2]` | **Ember Core**: Parts II–VII, X, XIII and XV's core and alloc layers.… |
| F-115 | S1 | `[CONF-4]` | **Ember Native**: adds Part XVI. Annex C (C++) is a separate, optional claim.… |
| F-123 | S2 | `[FFI-10]` | An `unsafe extern "C":` block declares foreign functions, statics and opaque types; `unsafe` records that the programmer asserts the signatures. The d… |
| F-130 | S2 | `[PRF-1]` | **A profile never changes what a program means.** Every profile accepts the same programs, performs the same safety checks (bounds, overflow, exclusiv… |
| F-130 | S2 | `[EXC-14]` | The 0.9.8 `exclusivity = "unchecked"` setting is removed. Code that cannot afford a check per access uses value types, `mut self` methods (`[EXC-15]`)… |
| F-130 | S2 | `[GPU-1]` | GPU objects are named by generational handles (`[HND-1]`), which are `Copy` and `Send`. A stale handle is detected by its generation **in every profil… |
| F-136 | S2 | `[MNG-1]` | **Mangling is injective.** A symbol is `em_` followed by each path component (package, modules, item) written as its length in decimal and then its te… |
| F-137 | S2 | `[CTL-1]` | `for pattern in e:` iterates: * a place `e` whose type is `Iterable`: borrows `e` for the loop and calls `e.iter()`; elements are `ref T` (read throug… |
| F-139 | S2 | `[CG-C-11]` | **Floating-point flags.** Every translation unit begins with `#pragma STDC FP_CONTRACT OFF` and is compiled with `-ffp-contract=off -fno-fast-math` (C… |
| F-139 | S2 | `[TYP-9]` | Floating point is strict IEEE 754: no reassociation, no contraction of `a * b + c` into a fused operation, no assumption that values are finite, unles… |
| F-140 | S1 | `[TST-28]` | **The performance gate.** `tests/perf/` holds benchmark programs, each with an equivalent C program, a checker of identical output, and a threshold (f… |
| F-144 | S1 | `[TST-8]` | `tests/firstweek/` holds at least 24 first-draft programs a newcomer plausibly writes in week one (a text adventure, a CSV summariser, a scene graph w… |
| F-144 | S1 | `[TST-10]` | The acceptance rate is published with each release; a release that lowers it says why.… |
| F-148 | S2 | `[TYP-31]` | **Sizes and indices are `int`.** Every standard container's `len()` returns `int`. Indexing accepts an index of any integer type; an index that is neg… |
| F-148 | S2 | `[LEX-24]` | A unary minus applied directly to an untyped integer literal forms a negative constant of the literal's eventual type. If that type is unsigned the pr… |
| F-155 | S2 | `[TYP-28]` | **Division.** `/` is **true division** and applies to floats. `/` with two integer operands is `E2240`, whose fix-its are `//` (floor division) and co… |
| F-156 | S2 | `[DOC-2]` | The user guide (`docs/book/`), with a "coming from Python" chapter built on Appendix E, is published with each release, and every sample in it is run … |
| F-159 | S2 | `[CLI-19]` | **The implementation matrix.** `ember --version --matrix` prints, for every Part, rule family, command and flag of this document, whether this compile… |
| F-160 | S2 | `[TYP-5]` | **Coercions — the complete list.** At a coercion site (assignment or initialisation, argument, return, field initialiser, collection element, default … |
| F-160 | S2 | `[GRM-36]` | `ref e` and `ref mut e` borrow the place `e` (`[BRW-1]`). The operand MUST be a place — a local, a field, an index or a dereferenced reference — and `… |
| F-161 | S1 | `[PHIL-12]` | **No silent acceptance.** Every construct a program writes — an attribute, an import, a directive, a contract, a manifest key, a statement form — eith… |
| F-161 | S1 | `[EFF-23]` | A contract attribute whose checking an implementation has not built is rejected with `E0900` (`[PHIL-12]`); it is never accepted unchecked.… |
| F-168 | S1 | `[LT-7]` | **Callable types.** Each call through a value or parameter of callable type `fn(P1, …, Pn) -> R` gets fresh regions for its reference and view paramet… |
| F-171 | S2 | `[CLI-19]` | **The implementation matrix.** `ember --version --matrix` prints, for every Part, rule family, command and flag of this document, whether this compile… |
| F-173 | S2 | `[—]` | … |
| F-180 | S2 | `[GRM-11]` | There are no block expressions. A value computed by several statements is written with a `match` expression, a conditional expression, a local functio… |
| F-181 | S2 | `[TYP-36]` | **Which types implement which interfaces.** The table is normative; a `—` means the implementation does not exist and a bound requiring it is `E2040`.… |
| F-182 | S2 | `[ERR-4]` | `Option` and `Result` provide `is_some`/`is_none`, `is_ok`/`is_err`, `unwrap`, `expect`, `unwrap_or`, `unwrap_or_else`, `unwrap_or_default`, `map`, `m… |
| F-182 | S2 | `[STD-15]` | **`Array[T]`** is a growable contiguous list (Python's `list`). `len`, `is_empty`, `capacity`, `reserve`, `push`, `pop -> Option[T]`, `insert(i, v)`, … |
| F-182 | S2 | `[TXT-10]` | **`str` operations.** `len()` is the length in bytes and `char_count()` in characters; `is_empty`, `chars`, `char_indices`, `bytes`, `as_bytes`, `line… |
| F-184 | S1 | `[CG-C-2]` | **Accepted programs compile.** A program Ember accepts never produces C that the C compiler rejects or warns about; such a case is a compiler defect (… |
| F-184 | S1 | `[TST-27]` | **The C gate.** Every accepted program in the test suite is compiled through the C compiler with warnings as errors on each supported host compiler. A… |
| F-185 | S1 | `[CG-C-2]` | **Accepted programs compile.** A program Ember accepts never produces C that the C compiler rejects or warns about; such a case is a compiler defect (… |
| F-190 | S2 | `[HEAP-8]` | Capacity arithmetic is checked: a requested capacity whose byte size overflows `usize`, or exceeds the allocator's limit, panics with `capacity overfl… |
| F-191 | S2 | `[HEAP-9]` | Heap storage for elements of type `T` is aligned to at least `align_of[T]()`, whatever that alignment is.… |
| F-192 | S2 | `[LAY-2]` | Layout attributes change layout exactly as stated, in every profile and on every backend, and are never ignored (`[ATT-6]`): * `@layout(c)` — the targ… |
| F-192 | S2 | `[PHIL-12]` | **No silent acceptance.** Every construct a program writes — an attribute, an import, a directive, a contract, a manifest key, a statement form — eith… |
| F-193 | S2 | `[RT-10]` | **Counting is inline.** The fast path of retain (one increment and an overflow test) and release (one decrement and a test for zero) is defined in `em… |
| F-198 | S2 | `[DIA-21]` | **Python habits.** Each of these is recognised and answered with the Ember form as a machine-applicable fix-it:… |
| F-198 | S2 | `[STD-13]` | **One naming convention.** Names are `snake_case` words joined by underscores (`starts_with`, `to_upper`, `push`, `is_empty`). Where Python's name for… |
| F-199 | S2 | `[—]` | … |
| F-200 | S2 | `[—]` | … |
| F-203 | S2 | `[STR-7]` | Inside the body of a `struct`, `enum`, `class` or `extend` block, `Self` names the type being declared with its own generic parameters, and the type's… |
| F-204 | S2 | `[TYP-14]` | A reference, and a `Box[T]`, is read through wherever a `T` is wanted — a field access, a method call, an operand, an argument to a borrowed parameter… |
| F-206 | S2 | `[DIA-6]` | Every code has a page, `docs/errors/EXXXX.md`, with a program that triggers it, the rendered diagnostic, why the rule exists and the fix; `ember expla… |
| F-207 | S2 | `[TST-29]` | **Honest baselines.** A gate with a baseline of known failures reports the baseline size beside its result; the baseline only shrinks, and a gate whos… |
| F-208 | S2 | `[DIA-14]` | Only the first error of a cascade is reported: nothing is reported about an expression whose type is already an error, a name bound to a failed import… |
