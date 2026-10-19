---

# Appendix G — Resolution of Findings F-001–F-214

Every finding of the 2026-09-23 research pass (`tasks/audit/FINDINGS.md`) and how this revision
resolves it. **SPEC**: the language text changed or gained a rule. **IMPL**: the rule stands or was
clarified and the implementation must meet it. **GATE**: a test or CI obligation now in the text.
**OUT**: outside a language specification, with the reason. **WDN**: withdrawn.

| Finding | Kind | Resolution | Rules |
|---|---|---|---|
| F-001 — `tasks/audit/TASKS.md` is cited by every reproducer and exists on no branch | OUT | Audit bookkeeping, not language text; `FINDINGS.md` Part D rebuilt the missing index. | — |
| F-002 — `()` is not accepted as a value of type `void` | IMPL | `()` is the value of `void`; `Ok(())` is the success value of `Result[void, E]`. | `[TYP-27]` |
| F-003 — `import a.b.c` does not bind `c` | IMPL | `import a.b.c` binds `c`; unknown modules are `E1060`. | `[MOD-3]` |
| F-004 — importing a module that does not exist is silently accepted | SPEC | An import of a missing module is `E1060`; nothing is accepted silently. | `[MOD-3]`, `[PHIL-12]` |
| F-005 — every E2020 type mismatch carries a note about numeric conversion | SPEC | The numeric-conversion note appears only when both operands are numeric. | `[TYP-4]` |
| F-006 — N1 suggestion quality | SPEC | Suggestion threshold scales with name length and kind. | `[DIA-24]` |
| F-007 — the showcase program passes a `str` literal where a `String` is required | SPEC | A string literal initialises a `String` where one is expected. | `[TXT-9]` |
| F-008 — one bad character reports once per byte | SPEC | A run of invalid characters is one diagnostic. | `[DIA-20]` |
| F-009 — "not implemented in this phase" reuses `E1010` | SPEC | Unimplemented constructs are `E0900`, never a name-resolution code. | `[PHIL-12]`, `[CLI-19]` |
| F-010 — cascades after a failed import | SPEC | Names bound to a failed import produce no further errors. | `[DIA-14]` |
| F-011 — cascade after a private constructor | SPEC | One error per failed construction; privacy has its own code. | `[DIA-14]`, `[MOD-2]` |
| F-012 — `println` of any `String`, including every f-string, emits C that does not compile | IMPL | `print`/`println` are specified; an accepted program that yields bad C is a compiler defect, caught by the C gate. | `[STD-9]`, `[CG-C-2]`, `[TST-27]` |
| F-013 — "accepted by Ember, rejected by the C compiler" is a whole defect class with no gate | GATE | Every accepted test program is compiled by each host C compiler with warnings as errors. | `[TST-27]`, `[CG-C-2]` |
| F-014 — a name assigned in every branch of an `if`/`else` is not visible after it | SPEC | A name assigned in every branch at one type is declared after the branch. | `[CTL-10]` |
| F-015 — `if x = 5:` (a `==` typo) gets a misleading fix | SPEC | `if x = 5:` gets the `==` fix-it. | `[DIA-21]` |
| F-016 — the 0.8.2c change log says there is no if-let; the grammar has one | SPEC | Pattern conditions are in the grammar; 0.9.9 carries no stale change log. | `[GRM-19]` |
| F-017 — the EBNF still contains what `[GRM-16]` and `[GRM-11]` delete | SPEC | Part III is a complete grammar with no deleted productions. | `[GRM-11]`, `[GRM-16]` |
| F-018 — two path separators, `.` and `::` | SPEC | One path separator, `.`; `::` is not a token. | `[GRM-24]`, `[LEX-21]` |
| F-019 — `as` binds tighter than unary minus | SPEC | Prefix `-`/`~` bind tighter than `as`. | `[TYP-6]` |
| F-020 — `@deprecated` has two signatures | SPEC | `@deprecated(since, note)` is the one form. | `[VER-3]` |
| F-021 — `@nopanic` is listed as v2 in the attribute table, while the 0.8.4_Hardened_1 change log calls `@nopanic(explicit)` one of Ember's current c | SPEC | `@nopanic(explicit)` is a current contract; bare `@nopanic` is `E0104`. | `[EFF-17]` |
| F-022 — a pasted editorial instruction inside `[BLD-11]` | SPEC | Rule rewritten; no editorial instructions remain in rule text (end check). | `[BLD-11]` |
| F-023 — `E1020` means two things in the compiler | SPEC | `E1020` means only a duplicate declaration; visibility errors are `E1052`. | `[GRM-4]`, `[MOD-2]` |
| F-024 — `[GRM-14]` adds a third kind of generic parameter (`access P`) for one library type, `std.ecs.Query` | SPEC | `Query` uses the ordinary marker type `Mut[T]`; no access-mode generic kind. | `[ECS-3]` |
| F-025 — `[LEX-22]` reserves `'a` for "v2 named lifetimes" | SPEC | Ember has no lifetime syntax, now or later. | `[LEX-22]` |
| F-026 — `[LEX-2]`: the formatter writes the platform's line ending by default | SPEC | The formatter writes LF everywhere. | `[FMT-1]` |
| F-027 — `[LEX-15]` still says "The reserved set therefore has 48 entries" | SPEC | The reserved-word count is stated once, correctly. | `[LEX-15]` |
| F-028 — the Part I.4 table still promises "~2 ns per access pair" | SPEC | The I.4 table states check mechanisms, not nanosecond promises. | `[PHIL-15]` |
| F-029 — `xs = Array[i32]()` is rejected: the explicit type argument is ignored | IMPL | Explicit constructor type arguments fix the type. | `[TYP-18]` |
| F-030 — `min`, `max` and `clamp` are used as free functions and declared nowhere | SPEC | `min`, `max`, `abs`, `clamp` are prelude functions. | `[MOD-5]` |
| F-031 — strings are the biggest Python-ergonomics gap | SPEC | Literal-to-`String` coercion, `String + str`, `+=`, `to_string`. | `[TXT-9]`, `[TXT-11]` |
| F-032 — `[TYP-15]` and `[TYP-15a]` disagree about containers of static views | SPEC | One rule for where views may be stored; containers of static `str` are legal. | `[TYP-15]` |
| F-033 — `<<` overflow is undefined by the text | SPEC | Shift amount checked; bits shifted out are not overflow; `>>` arithmetic on signed. | `[TYP-10]` |
| F-034 — `[TYP-13]` guarantees a niche for `*fn` | SPEC | The niche list names `extern fn`, which exists. | `[TYP-13]` |
| F-035 — `as?` / `as!` are used in IV.6 and absent from the grammar | SPEC | `as?` and `as!` are single tokens in the grammar. | `[GRM-30]`, `[LEX-21]` |
| F-036 — `@view` is required on a struct the compiler already knows is a view | SPEC | View-ness is inferred; `@view` is optional documentation. | `[TYP-34]` |
| F-037 — the default integer is `i32` | SPEC | `int` is `i64`, `float` is `f64`; literals default to them after context. | `[TYP-1]`, `[LEX-16]` |
| F-038 — floats have no `Ord`, so sorting a list of floats needs ceremony | SPEC | Floats implement `Ord` by totalOrder; operators stay IEEE; `sort()` works. | `[TYP-37]` |
| F-039 — `[RNG-10]` carries `[RNG-10a]`, `[RNG-10b]` and `[RNG-10c]` inline in one bullet | SPEC | Each range rule is its own bullet. | `[RNG-10]` |
| F-040 — the orphan rule `[TYP-20]` is not enforced, and as written it is stricter than Rust's | SPEC | Coherence is per package: an `extend` may sit in any module of the package declaring the type or interface. | `[TYP-20]` |
| F-041 — "interface methods need the interface imported" (`[TYP-24]` step 2) is not enforced | SPEC | Interface methods resolve without importing the interface when one visible implementation provides them. | `[TYP-24]` |
| F-042 — no table says which standard interfaces the built-in types implement | SPEC | A normative table of which built-in types implement which interfaces. | `[TYP-36]` |
| F-043 — which interface `<` uses is unclear, and `Ord` excludes floats | SPEC | Comparison operators are built in for scalars and `Ord.cmp` in generic code; no `PartialOrd`. | `[TYP-37]` |
| F-044 — `Formatter`, `FmtError`, `Ordering` and `Into` are used by the interface sketch and declared nowhere | SPEC | `Ordering` is in the prelude; `Formatter`, `FmtError` are declared in `std.fmt`. | `[MOD-5]`, `[STD-18]` |
| F-045 — three iteration interfaces | SPEC | Three interfaces: `Iterator`, `Iterable`, `IntoIterator`; `iter_mut` is a method convention. | `[CTL-1]`, `[TYP-36]` |
| F-046 — eight selectable language versions before 1.0 | SPEC | Before 1.0 there is one language; no version selectors. | `[VER-8]` |
| F-047 — the `[MOD-5]` prelude list is incomplete against the rest of the document | SPEC | The prelude table is the single authoritative list. | `[MOD-5]` |
| F-048 — `Map` and `Set` are not in the prelude | SPEC | `Map` and `Set` are prelude names. | `[MOD-5]` |
| F-049 — Python chained comparison is rejected | SPEC | Comparisons chain with Python meaning; the middle operand is evaluated once. | `[GRM-25]` |
| F-050 — `[GRM-23]` gives `a in b in c` code `E0104` | SPEC | `a in b in c` is `E0102`. | `[GRM-23]` |
| F-051 — `extend[T: Display] Array[T] implements Display` (V.6) is not in the grammar | SPEC | `extend [T: B] Array[T] implements I:` is in the grammar. | `[GRM-34]` |
| F-052 — impl granularity is inconsistent | SPEC | Inherent and interface extensions both follow package granularity. | `[IFC-1]`, `[TYP-20]` |
| F-053 — three rules stated twice, verbatim, inside themselves | SPEC | Rules are written once; the rewritten parts carry no duplicates. | `[GRM-20]`, `[GRM-8d]` |
| F-054 — every struct needs `@derive(...)` boilerplate | SPEC | `Eq`, `Debug`, `Clone` are implicit when fields allow; `@no_derive` opts out. | `[STR-5]` |
| F-055 — no runtime-initialised globals in v1 | SPEC | Statics may have run-time initialisers: lazy, once, thread-safe. | `[STA-1]`, `[STA-3]` |
| F-056 — `fn main(args: Span[str])` | SPEC | `main(args: Array[String])` decodes lossily; raw bytes via `args_os()`. | `[FN-8]` |
| F-057 — undeclared names in normative examples | SPEC | Examples either compile or are marked `ember,fragment`; math names are imported or methods. | `[TST-7]` |
| F-058 — a closure cannot be returned or boxed; the spec's own form is rejected | SPEC | Callable types outside parameters are owned callable values; returning a closure is `-> fn(A) -> R`. | `[CLO-3]`, `[CLO-10]` |
| F-059 — what a `fn(A) -> R` type means outside a parameter is unspecified | SPEC | Position-dependent meaning of `fn(A) -> R` is specified. | `[CLO-3]` |
| F-060 — calling a callable field needs a temporary | SPEC | `b.on_click()` calls a callable field when no method of that name exists. | `[CLO-11]` |
| F-061 — a boxed once-callable is not callable (`[CLO-6a]`) | SPEC | An owned `once fn`, boxed or not, is callable; the call consumes it. | `[CLO-6a]` |
| F-062 — a class `gen fn` with `mut self` contradicts `[CORO-6]` or `[CLS-7]`, and the text does not say which | SPEC | Class generator methods take `self`; no class long-term access spans a `yield`. | `[CORO-12]` |
| F-063 — `Coroutine[R]` names one type parameter; a coroutine has two | SPEC | `Generator[Y, R]` names both the yield and the return type. | `[CORO-1]` |
| F-064 — temporaries in an `if`-condition live through the `else` | SPEC | Temporaries of an `if` condition are dropped before `else`. | `[EXP-4]` |
| F-065 — `%` follows C (sign of the dividend) while the syntax follows Python | SPEC | `%` and `//` have Python floor semantics; `rem_trunc`/`div_trunc` give C's. | `[TYP-28]` |
| F-066 — integer `**` with a run-time negative exponent | SPEC | `**` on integers: negative exponent panics (constant: `E2151`); overflow panics. | `[TYP-30]` |
| F-067 — a method call borrows all of `self`, so the classic Rust partial-borrow error is back | SPEC | Private methods borrow only the fields they touch. | `[BRW-10]` |
| F-068 — the Python swap `a[i], a[j] = a[j], a[i]` is a parse error | SPEC | Unparenthesised tuples on the right of `=` and after `return`. | `[GRM-29]` |
| F-069 — premise disproved by probe; see F-168 | WDN | Withdrawn: premise disproved by probe (see F-168). | — |
| F-070 — `@borrows(arena)` is mandatory on every Arena wrapper, at every nesting level | SPEC | Arena elision: a view returned from a function whose only view source is one `Arena` borrows it. | `[LT-44]` |
| F-071 — the `@borrows` example contradicts the rule and its own semantics | SPEC | `@borrows` stands on its own line; the example no longer contradicts it. | `[LT-1a]` |
| F-072 — `[DRP-4]` is unenforceable as written | SPEC | A panic in `drop` aborts; stated as behaviour, not an unenforceable obligation. | `[DRP-4]` |
| F-073 — writing a class field through a handle from outside a method fails | SPEC | Class fields are written through `self` or any handle without `mut`; each access is checked. | `[CLS-7]`, `[FN-9]` |
| F-074 — no way to return an iterator without naming its concrete type | SPEC | Opaque `some I` returns and generators. | `[TYP-32]`, `[CORO-1]` |
| F-075 — `split_at_mut` is named by `[BRW-5]` and `[TST-25]` (item 8); the API is `split_at` | SPEC | One API name: `split_at`. | `[SPN-5]` |
| F-076 — VII.7's example has statements at file scope | SPEC | The example is inside `main`. | `[GRM-2]` |
| F-077 — spec examples call APIs that do not exist | SPEC | The APIs examples use are specified in Part XV. | `[STD-15]`, `[STD-19]` |
| F-078 — the text contradicts itself on whether a `Sync` class has dynamic exclusivity at all, which decides whether Safe Ember has a data race | SPEC | Only `@sync` classes are shareable, and their fields are immutable after `init`; no data race. | `[THR-1]`, `[THR-13]` |
| F-079 — assigning a non-`Copy` field through a handle is neither an "instantaneous" nor a "long-term" access | SPEC | Assigning a non-`Copy` field through a handle is a write access. | `[EXC-16]` |
| F-080 — `String(literal)` does not exist | SPEC | `String.from(s)`, `s.to_string()` and literal coercion are specified. | `[TXT-9]`, `[TXT-11]` |
| F-081 — writing a field through a borrowed class-handle parameter is rejected (update to F-073) | SPEC | Borrowed class-handle parameters may be written through. | `[CLS-7]`, `[FN-9]` |
| F-082 — reference cycles leak silently in release, and Python users build cycles | SPEC | Debug runs report leaked cycles at exit by default. | `[WK-15]` |
| F-083 — objects may die before their last syntactic use | SPEC | An object dies when its last owner ends as the source says, never earlier (ODR-063); `L3019` is retired. | `[RC-3]` |
| F-084 — the VIII.7 idiom uses three things the compiler lacks or rejects | SPEC | Callable fields, pattern conditions and owned callable values make the idiom writable. | `[CLO-3]`, `[CLO-11]` |
| F-085 — the development target's examples are not checked by any gate, and half of them do not parse | GATE | Every `ember` block of the current document is extracted and checked. | `[TST-7]` |
| F-086 — `[ARN-4]` makes `@noalloc`-cleanliness a property of a value, which the effect system cannot see | SPEC | `FixedArena` is the `@noalloc` arena type; `Arena.fixed` returns it. | `[ARN-4]` |
| F-087 — `Shared[T]` overlaps `class` almost completely | SPEC | Kept, with the selection table saying when each applies. | `[SEL-1]` |
| F-088 — almost none of the Part IX.1 heap library exists | IMPL | The heap library is specified; building it is implementation work. | `[HEAP-1]`, `[STD-15]`, `[STD-16]` |
| F-089 — `[UNS-8]` and `[LEX-11]` contradict each other about whether a comment can produce a warning | SPEC | Safety notes are the one stated exception to "comments never affect compilation". | `[LEX-11]`, `[LEX-23]`, `[UNS-8]` |
| F-090 — the `assert_disjoint` example uses `dst` after moving it | SPEC | `assert_disjoint` returns a `Result` that gives the views back on failure. | `[DSJ-1]` |
| F-091 — `[ALC-3]` declares `static ALLOC: dyn Allocator` | SPEC | The global allocator is named in the manifest; no unsized static. | `[ALC-3]` |
| F-092 — phantom type parameters | SPEC | Phantom type parameters are legal. | `[TYP-35]` |
| F-093 — `Map`/`Set` iterate in unspecified order; Python's `dict` iterates in insertion order | SPEC | `Map`/`Set` iterate in insertion order and are not `Nondet`. | `[STD-11]`, `[DET-2]` |
| F-094 — is `DefaultHasher` seeded per process? | SPEC | Fixed-seed `DefaultHasher`; `RandomState` for untrusted keys. | `[HASH-2]` |
| F-095 — `[EFF-2]`: a `fn(A) -> R` parameter "is assumed to carry all effects unless written `@noalloc fn(A) -> R`" | SPEC | Callable parameters contribute the passed function's effects, per instantiation. | `[EFF-2]` |
| F-096 — `RuntimeCheck` has four kinds; `[EFF-17]` says `@no_runtime_checks` "excludes all five kinds" | SPEC | Four `RuntimeCheck` kinds, named. | `[EFF-9]` |
| F-097 — `[EFF-16]`'s last two sentences have no clear antecedent | SPEC | What `@nopanic(explicit)` allows is stated plainly. | `[EFF-16]` |
| F-098 — `@realtime`'s default set includes `@nopanic(explicit)`, and `[EFF-16]` makes every division by a non-constant, every shift by a variable, a | SPEC | `RuntimeCheck` is allowed under `@nopanic(explicit)` and `@realtime`; `NonZero` removes division checks. | `[EFF-16]`, `[EFF-19]`, `[STD-4]` |
| F-099 — the attribute table omits `@nopanic(explicit)` (refines F-021) | SPEC | The attribute table lists `@nopanic(explicit)`. | `[ATT-1]` |
| F-100 — the first-hour path does not exist | GATE | Single-file programs need no manifest; the user guide and first-week corpus are release artefacts. | `[CLI-4]`, `[DOC-2]`, `[TST-8]` |
| F-101 — is the stale-handle check removed in `shipping`? | SPEC | Stale-handle checks run in every profile. | `[HND-1]`, `[COST-3]` |
| F-102 — "release wraps and emits nothing" is only true if the C is written so that wrapping is defined | SPEC | Overflow panics in every profile; the emitted C is UB-free. | `[TYP-8]`, `[CG-C-1]` |
| F-103 — `[TOOL-1]`–`[TOOL-4]` sit inside X.3 "Inspection" | SPEC | Toolchain rules live in Part XVII. | `[CLI-4]` |
| F-104 — `[THR-1]` admits a `Sync` class with a plain mutable scalar field, and scalar field writes through a handle are unchecked | SPEC | Plain mutable fields cannot exist in a `@sync` class. | `[THR-1]` |
| F-105 — the job-system example breaks two rules | SPEC | The job example uses a scope, borrows legally, and writes no call-site modes. | `[JOB-2]`, `[FN-2a]` |
| F-106 — what `return`, `break`, `continue` and `?` mean inside `with scope = thread.scope():` | SPEC | A scope is a `with` block; jumps keep their meaning and join first. | `[THR-5]` |
| F-107 — async is v2 | SPEC | `async` stays reserved; generators are designed not to block it. | `[LEX-15]` |
| F-108 — `[CT-1]` excludes allocation from comptime and then supports allocating types | SPEC | Compile-time code may allocate. | `[CT-1]` |
| F-109 — what happens when comptime-materialised heap data is mutated at run time | SPEC | Heap results are read-only static data, run-time initialised, or cloned per evaluation. | `[CT-5]` |
| F-110 — `comptime` is used as an expression with a block value, which the grammar does not have | SPEC | `comptime(e)` is the expression form. | `[CT-6]`, `[GRM-32]` |
| F-111 — more syntax in examples that the grammar lacks | SPEC | Examples use grammar that exists: proxies are `SoARef`, `@from` is on the variant, no call-site `mut`. | `[SOA-6]`, `[ERR-3]`, `[FN-2a]` |
| F-112 — `@derive(SoA)` generates a different `SoA[T]` per `T`, which is specialisation | SPEC | `SoA[T]` is a compiler-known type constructor. | `[SOA-1]` |
| F-113 — a default error type would shorten most signatures | SPEC | `Result[T, E = AnyError]`. | `[ERR-9]`, `[ERR-8]` |
| F-114 — `[DRV-1]`: every derive is specified as a hand-written `extend` in `std/derive/*.em`, and the generator "MUST produce the same MIR" | SPEC | The spec text itself defines what each derive generates. | `[DRV-1]` |
| F-115 — the build order is inverted against the goal | SPEC | Conformance profiles put the core language and C before C++; C++ is an optional annex. | `[CONF-2]`, `[CONF-4]` |
| F-116 — `min`/`max`/`clamp`/`abs`/`sqrt`/`sin` are declared in `std.math`, but the spec's examples call them unimported | SPEC | Prelude has `min`/`max`/`abs`/`clamp`; `sqrt` is a method; other math is imported. | `[MOD-5]`, `[STD-20]` |
| F-117 — `[STD-8]`'s mandated help text is Rust syntax | SPEC | The mandated help is Ember syntax. | `[STD-8]` |
| F-118 — `[TXT-2]` still names `CppString.as_str()` | SPEC | Conversions from foreign text are fallible `to_str()`. | `[TXT-2]` |
| F-119 — `print`/`println` have no signature | SPEC | `print`/`println` signature, `sep=`, `end=`, several arguments. | `[STD-9]`, `[TYP-26]` |
| F-120 — `unsafe(reason = "…")` (`[UNS-9]`) is not in the grammar | SPEC | No `unsafe(reason=…)`; the category goes in the `# SAFETY(…):` note. | `[LEX-23]`, `[UNS-8]` |
| F-121 — sections filed under the wrong Part | SPEC | Each rule sits in its Part. | `[STD-4]` |
| F-122 — the `[MOD-5]` prelude list omits names Part XV puts in the prelude | SPEC | The prelude table includes them. | `[MOD-5]` |
| F-123 — Ember cannot call C at all today, and the C-import directive is silently ignored | SPEC | Manual declarations are module items; a function declared `safe fn` with a complete contract is safe to call (an asserted fact). | `[FFI-10]` |
| F-124 — two more unapplied editorial instructions inside normative rules | SPEC | Rewritten without editorial instructions; grades move to Annex C. | `[TCB-1]` |
| F-125 — `std::string` → `CppString` "with `.as_str()`" | SPEC | `CppString.to_str()` everywhere. | `[FFI-17a]` |
| F-126 — the overlay language has no grammar | SPEC | The overlay language has a grammar. | `[GRM-35]` |
| F-127 — scope: the C++ importer is a project the size of the rest of the compiler | SPEC | C++ is an optional annex after C. | `[CONF-4]`, `[FFI-3]` |
| F-128 — the "five count contracts" are never listed | SPEC | The count axis is listed: `one`, `count(n)`, `nul_terminated`, `fixed(N)`, `inout_count(p)`. | `[FFI-11]`, `[FFI-11a]` |
| F-129 — the `[FFI-39]` example uses constructor syntax the grammar does not have | SPEC | Constructors are `fn init(self, …)` everywhere, including C++ subclasses. | `[FFI-39]`, `[CLS-2]` |
| F-130 — two package settings turn memory-safety checks off for Safe code, and `[PHIL-10]` lists no such exception | SPEC | No setting removes a safety check; the two keys are gone. | `[PRF-1]`, `[EXC-14]`, `[GPU-1]` |
| F-131 — XVII.5 writes `mut` at a call site | SPEC | No call-site modes in examples. | `[FN-2a]` |
| F-132 — the mandated refusal report suggests a signature the rules reject | SPEC | Refusal reports do not suggest signatures; `migrate_from` returns `Result`. | `[HR-16]`, `[HR-18]` |
| F-133 — `migrate_from` may not contain a single integer `+` | SPEC | Run-time check failures in `migrate_from` become `ReloadError.Migration`. | `[HR-35]` |
| F-134 — is a base class's `init` inherited? | SPEC | A derived class with no `init` inherits its base's. | `[CLS-10]` |
| F-135 — wrong or loose citations | SPEC | Citations rewritten; one attribute per line. | `[HR-3]`, `[ATT-4]` |
| F-136 — the mangling scheme is not injective, and a collision is an internal compiler error | SPEC | Mangling is length-prefixed and injective; a hash collision is `E9040`. | `[MNG-1]` |
| F-137 — `for x in span:` is rejected | IMPL | `for x in span:` iterates a `Span`. | `[CTL-1]` |
| F-138 — `[CTL-3b]`'s guaranteed lowering is not implemented | IMPL | The guaranteed counted-loop lowering stands. | `[CTL-3]` |
| F-139 — float contraction is never disabled | SPEC | Floating-point flags and pragmas are required of the backend. | `[CG-C-11]`, `[TYP-9]` |
| F-140 — there is no performance suite; "fast like C" is unmeasured | GATE | A performance suite against equivalent C gates releases. | `[TST-28]` |
| F-141 — several promised backend/runtime pieces do not exist | SPEC | Inline header, allocator and backtrace requirements restated; mimalloc no longer required. | `[CG-C-3]`, `[RT-1]`, `[CG-C-10]` |
| F-142 — `compiler/ember_typeck/src/lib.rs` is 20,935 lines in one file | OUT | Source-file size in the compiler is not language text; noted for the compiler. | — |
| F-143 — XIX says effect analysis both runs after monomorphisation (XIX.1: *"Effect analysis runs  | SPEC | Effects are computed once, per instantiation for callable parameters. | `[EFF-2]`, `[EFF-15]` |
| F-144 — the instrument that measures the goal does not exist | GATE | First-week corpus and published acceptance rate. | `[TST-8]`, `[TST-10]` |
| F-145 — `E9010` has two meanings | SPEC | `E9010` is an unknown manifest key; an unhonourable float attribute is `E9041`. | `[MAN-1]`, `[TYP-9c]` |
| F-146 — most of the CLI in XX.1 is missing | SPEC | The CLI is listed; anything unimplemented is `E0900` and shown by `--matrix`. | `[CLI-19]` |
| F-147 — the example manifest makes shipping builds UB on an exclusivity violation by default | SPEC | The manifest example has no safety keys; none exist. | `[MAN-8]` |
| F-148 — `xs[-1]` compiles and panics at run time with index 18446744073709551615 | SPEC | Indices are `int`; a negative literal index is `E2011`; a negative index panics with a clear message. | `[TYP-31]`, `[LEX-24]` |
| F-149 — `x is None` gives two wrong errors instead of the fix | SPEC | `x is None` is legal. | `[EXP-9]` |
| F-150 — `let x = 5` gives a raw parse error | SPEC | `let x = 5` gets the `x = 5` fix-it. | `[DIA-21]` |
| F-151 — `a < b < c` fix-it is incomplete | SPEC | Chained comparisons are legal, so no fix-it is needed. | `[GRM-25]` |
| F-152 — `[DIA-7a]`'s table is flattened into one line | SPEC | Diagnostic tables are real tables. | `[DIA-7]` |
| F-153 — the N1 suggestion threshold admits `io` -> `Eq` (the rule behind F-006) | SPEC | Suggestion distance: 1 up to 4 characters, 2 above. | `[DIA-24]` |
| F-154 — stale conditional and dead codes in the diagnostics part | SPEC | Shape table rewritten; N8 no longer asks for a call-site mode. | `[DIA-7]`, `[FN-2a]` |
| F-155 — integer `/` is the silent Python trap | SPEC | Integer `/` is rejected with fix-its; `//` is floor division. | `[TYP-28]` |
| F-156 — no user guide and no "coming from Python" chapter | GATE | User guide with a Python chapter ships each release; Appendix E is its basis. | `[DOC-2]` |
| F-157 — C++ exception policy: "no default" vs a default | SPEC | One C++ exception policy, with a default. | `[FFI-24]` |
| F-158 — more amendment text appended instead of applied | SPEC | Rewritten without appended amendments. | `[TCB-1]` |
| F-159 — Phase 1 is recorded as complete, but its exit criterion ("conformance for Parts II–VI except closures/generics") is not met | OUT | Phase status is project tracking; `ember --version --matrix` makes implementation status visible. | `[CLI-19]` |
| F-160 — indexing does not produce a place that can be returned by reference, so milestone M2 cannot be written as specified | IMPL | Indexing yields a place; a `ref` result auto-borrows it. | `[TYP-5]`, `[GRM-36]` |
| F-161 — every contract attribute is accepted and ignored, and so is any made-up attribute | SPEC | Unbuilt contracts and unknown attributes are rejected, never ignored. | `[PHIL-12]`, `[EFF-23]` |
| F-162 — milestone M3 fails: `mem` is not in the prelude | SPEC | `mem` is in the prelude. | `[MOD-5]` |
| F-163 — XXI.5 lists `book/` as "(user guide, v1.1)" | SPEC | The user guide is a release artefact. | `[DOC-2]` |
| F-164 — the Stage 0 example uses syntax and semantics the rest of the document rejects | OUT | The engine-specific Stage 0 example is outside the language document. | — |
| F-165 — stale lists in XXIII | SPEC | New glossary and reserved-word statement. | `[LEX-15]` |
| F-166 — three owner decisions bear directly on the goal and should be re-put with the evidence from this pass | SPEC | Decided in this revision: 64-bit number defaults, branch-declared names, no `::`. | `[TYP-1]`, `[CTL-10]`, `[GRM-24]` |
| F-167 — the quick reference does not type-check, and one of its errors is in the reference itself | GATE | Annex A is generated from a checked fixture. | `[TST-6]` |
| F-168 — `with_views*` and `@latebound`: a chain of features whose net effect is to reject safe programs | SPEC | `with_views*` and `@latebound` are removed; callable types get fresh regions per call. | `[LT-7]` |
| F-169 — diagnostics that originate in a callee's contract are reported at the callee, not the caller | SPEC | Diagnostics from a callee's contract point at the call site. | `[DIA-22]` |
| F-170 — the headline multi-region example does not compile under the document's own rules | SPEC | The multi-region example takes `mut b`. | `[FN-2a]` |
| F-171 — the machine-readable implementation matrix the spec requires does not exist | GATE | The implementation matrix is a compiler output. | `[CLI-19]` |
| F-172 — Part XXIV's normative rules have no category | OUT | Part XXIV (compiler internals) is not in 0.9.9; its user-visible obligations are Part XVIII. | `[IMP-11]` |
| F-173 — one 870 KB file is three documents | OUT | 0.9.9 is the language document only; compiler design and history are elsewhere. | — |
| F-174 — `[GEN-COH-1]` makes impl ownership a *package* matter | SPEC | One coherence rule, at package granularity. | `[TYP-20]` |
| F-175 — what happens when a `Pool` runs out of generations | SPEC | 64-bit handles by default; exhausted slots retire; full pool is `CapacityError`. | `[HND-2]`, `[HND-3]` |
| F-176 — `[HR-IMPL-2]` describes reclaiming an old image | SPEC | Old images are never unloaded; statics live outside images. | `[HR-4]`, `[HR-4a]` |
| F-177 — Appendix B is a "mandatory CI" checklist that fails against the file it is in | OUT | No self-checking checklist in the document; the end checks are tools. | `[TST-4]` |
| F-178 — Appendix H is 730 lines of audit records that "bind nobody" | OUT | Appendix H is a change list, not audit records. | — |
| F-179 — 1.0 is tied to a full RageV production migration | OUT | 1.0 is defined by conformance and gates, not by a host migration. | `[CONF-1]` |
| F-180 — the conditional expression `a if c else b` is not implemented | IMPL | Conditional expressions are in the grammar. | `[GRM-11]` |
| F-181 — built-in scalars do not implement `Ord`, so no generic comparison can be written | SPEC | Scalars implement `Ord`. | `[TYP-36]` |
| F-182 — `Option`/`Array`/`str` basics are missing | IMPL | The combinator and container APIs are specified. | `[ERR-4]`, `[STD-15]`, `[TXT-10]` |
| F-183 — a single-statement block lambda inside brackets is rejected | IMPL | A one-statement lambda body in brackets is legal. | `[GRM-17]` |
| F-184 — range types reach the C compiler broken | IMPL | Range types print and compile; the C gate catches broken C. | `[CG-C-2]`, `[TST-27]` |
| F-185 — a parent/child class pair with a constructor overflows the compiler's stack | IMPL | An internal compiler error is always a defect. | `[CG-C-2]` |
| F-186 — the runtime exclusivity panic names a C symbol, not the Ember object | SPEC | Panics name Ember entities. | `[DIA-23]`, `[EXC-6]` |
| F-187 — `L3011` fires on calls that cannot re-enter the cell | SPEC | `L3011` fires only for calls that can reach the cell. | `[CELL-7]` |
| F-188 — a view escaping its source gets two errors, the first wrong | SPEC | Storage end with a live loan is one diagnostic, shape B7. | `[BCK-4]`, `[DIA-7]` |
| F-189 — `Array.iter()` does not exist | IMPL | `Array.iter()` is specified. | `[STD-15]` |
| F-190 — growable-buffer arithmetic is unchecked | SPEC | Capacity arithmetic is checked. | `[HEAP-8]` |
| F-191 — `Array[T]` storage is aligned to 16 bytes whatever `T` needs | SPEC | Heap storage honours element alignment. | `[HEAP-9]` |
| F-192 — `@align(N)` is silently ignored | SPEC | Layout attributes are honoured or rejected. | `[LAY-2]`, `[PHIL-12]` |
| F-193 — every reference-count operation is an out-of-line call, and a `Sync` retain is a CAS loop | SPEC | Count fast paths are inline; a `@sync` retain is one `fetch_add`. | `[RT-10]` |
| F-194 — every allocation goes through `_aligned_malloc` (MSVC) / `aligned_alloc`, and `ember_realloc` always allocates-copies-frees | SPEC | System allocator for ordinary alignment; per-thread statistics. | `[RT-1]`, `[RT-11]` |
| F-195 — reference-count overflow reports the wrong reason | SPEC | Count overflow panics with its own message. | `[RT-7]` |
| F-196 — redeclaring a name in the same block is accepted | IMPL | Redeclaring in one block is `E1020`. | `[GRM-4]` |
| F-197 — raw strings, byte strings and f-string format specs are not implemented | IMPL | Raw strings and format specs are specified. | `[LEX-19]`, `[LEX-25]` |
| F-198 — the ten most common Python habits get no help at all | SPEC | A table of Python habits with fix-its. | `[DIA-21]`, `[STD-13]` |
| F-199 — the ledgers say "none open" while ~80 defects reproduce | OUT | Defect ledgers are project process; Appendix G lists every open finding with its resolution. | — |
| F-200 — specification churn far outpaces the compiler | OUT | Process: 0.9.9 consolidates to one revision and a smaller document. | — |
| F-201 — `docs/HANDOFF.md` is 10,698 lines | OUT | Repository handoff size is not language text. | — |
| F-202 — `examples/` has only `hello.em` | GATE | Documentation samples are compiled and run. | `[DOC-2]` |
| F-203 — a generic struct's methods cannot name their own type | SPEC | `Self` and the type's own name work inside its declaration. | `[STR-7]` |
| F-204 — the textbook recursive enum cannot be traversed | SPEC | `Box[T]` reads through like a reference. | `[TYP-14]` |
| F-205 — calling a parameter whose type is a generic bounded by `Callable` fails | SPEC | Callable bounds are written `F: fn(A) -> R`; `Callable[…]` is not source. | `[CLO-14]` |
| F-206 — 32 error pages for 210 registered codes | GATE | Every code has an error page; a missing page fails CI. | `[DIA-6]` |
| F-207 — the six "green" gates are green because their baselines absorb almost everything | GATE | Baselines are reported and a heavily baselined gate is not called green. | `[TST-29]` |
| F-208 — range slicing `xs[a..b]` is not implemented | IMPL | Slicing is specified; its failure must not cascade. | `[DIA-14]` |
| F-209 — comprehensions | SPEC | List, set and map comprehensions. | `[GRM-27]` |
| F-210 — interfaces are nominal only | SPEC | Interfaces stay nominal, with the reason stated. | `[TYP-40]` |
| F-211 — `enumerate` over an `Array` cannot be written | IMPL | `enumerate` over an array is specified. | `[STD-19]` |
| F-212 — two errata are marked open that the grammar already fixed | OUT | The errata ledger is outside the document. | — |
| F-213 — `i32.MIN % -1` | SPEC | `MIN % -1 == 0`; `MIN // -1` panics as overflow, naming the right operator. | `[TYP-28]` |
| F-214 — the test suite is green (263 tests) but has a flaky UI pair, and `cargo test` hides everything after the first failure | GATE | The runner reports all failures and quarantines flaky tests by name. | `[TST-30]` |
