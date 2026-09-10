//! The error-code registry (Part XIX §6).
//!
//! Every code named in the specification appears here with its title and the
//! normative rule it enforces, whether or not the compiler emits it yet. The
//! registry is what `ember explain EXXXX` reads and what CI checks against
//! `docs/errors/` (`[DIA-6]`).
//!
//! Ranges:
//!
//! | Range | Subsystem |
//! |---|---|
//! | E0000–E0099 | lexer / indentation / directives |
//! | E0100–E0499 | parser |
//! | E1000–E1499 | name resolution, modules, visibility |
//! | E2000–E2499 | types, inference, interfaces, generics, patterns |
//! | E3000–E3499 | ownership, moves, borrows, regions, exclusivity, drops |
//! | E4000–E4499 | effects and contracts |
//! | E5000–E5499 | FFI |
//! | E6000–E6499 | comptime |
//! | E7000–E7499 | concurrency |
//! | E8000–E8499 | layout attributes, GPU layout |
//! | E9000–E9499 | build system, manifest, toolchain |

use std::fmt;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum CodeKind {
    Error,
    Warning,
    Lint,
}

impl CodeKind {
    fn prefix(self) -> char {
        match self {
            CodeKind::Error => 'E',
            CodeKind::Warning => 'W',
            CodeKind::Lint => 'L',
        }
    }
}

/// Which subsystem a code belongs to. Declared explicitly rather than derived
/// from the number, so that a code placed in the wrong range is caught by a
/// test rather than by a reader.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Subsystem {
    Lex,
    Parse,
    Resolve,
    Types,
    Ownership,
    Effects,
    Ffi,
    Comptime,
    Concurrency,
    Layout,
    Build,
}

impl Subsystem {
    /// The numeric range this subsystem owns, inclusive.
    pub fn range(self) -> (u16, u16) {
        match self {
            Subsystem::Lex => (0, 99),
            Subsystem::Parse => (100, 499),
            Subsystem::Resolve => (1000, 1499),
            Subsystem::Types => (2000, 2499),
            Subsystem::Ownership => (3000, 3499),
            Subsystem::Effects => (4000, 4499),
            Subsystem::Ffi => (5000, 5499),
            Subsystem::Comptime => (6000, 6499),
            Subsystem::Concurrency => (7000, 7499),
            Subsystem::Layout => (8000, 8499),
            Subsystem::Build => (9000, 9499),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Code {
    pub kind: CodeKind,
    pub number: u16,
    pub subsystem: Subsystem,
    /// One line, lowercase, no trailing period (`[DIA-2]`).
    pub title: &'static str,
    /// The normative rule this code enforces, e.g. `"[OWN-3]"`.
    pub rule: &'static str,
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{:04}", self.kind.prefix(), self.number)
    }
}

impl fmt::Debug for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

macro_rules! codes {
    ($( $name:ident = ($kind:ident, $num:expr, $sub:ident, $rule:literal, $title:literal); )*) => {
        $(
            pub const $name: Code = Code {
                kind: CodeKind::$kind,
                number: $num,
                subsystem: Subsystem::$sub,
                title: $title,
                rule: $rule,
            };
        )*

        /// Every declared code, in declaration order.
        pub const REGISTRY: &[Code] = &[ $($name),* ];
    };
}

codes! {
    // --- lexer, indentation, directives ------------------------------------
    E0001 = (Error, 1, Lex, "[LEX-1]", "invalid UTF-8 in source");
    E0002 = (Error, 2, Lex, "[LEX-4]", "tab used for indentation");
    E0003 = (Error, 3, Lex, "[LEX-5]", "inconsistent dedent");
    E0004 = (Error, 4, Lex, "[LEX-9]", "expected an indented block");
    E0005 = (Error, 5, Lex, "[LEX-14]", "reserved keyword used as an identifier");
    E0006 = (Error, 6, Lex, "[MOD-6]", "unsupported language version");
    E0007 = (Error, 7, Lex, "[LT-6]", "named lifetimes are not supported in this version");

    // --- parser -------------------------------------------------------------
    E0100 = (Error, 100, Parse, "[AST-2]", "unexpected token");
    E0101 = (Error, 101, Parse, "[AST-2]", "unclosed delimiter");
    // The specification numbers these three E0010, E0011 and E0020, inside the
    // lexer's range, although all three need a parsed tree. Renumbered into
    // the parser range by the owner, 2026-09-07; see docs/spec-errata.md
    // ERR-001 for the mapping.
    E0102 = (Error, 102, Parse, "[III.5]", "chained comparison");
    E0103 = (Error, 103, Parse, "[GRM-10]", "match arms mix statement and expression form");
    // Two rules name this code. `[ATT-1]`/`[ATT-2]`/`[ATT-5]`/`[FFI-34a]` use
    // it for an attribute or grade suffix that is not admitted where it
    // appears; `[GRM-23]` uses it for `a in b in c`. `[DIA-6a]` asks for one
    // code per rule and the document does not honour that here — the reading
    // that keeps both is that `E0104` means "this construct is not admitted in
    // this position", which covers both uses (errata ERR-026).
    E0104 = (Error, 104, Parse, "[ATT-1], [GRM-23]", "construct not admitted in this position");
    E0105 = (Error, 105, Parse, "[GRM-18]", "`;` is not a statement separator");
    E0106 = (Error, 106, Parse, "[GRM-17]", "a multi-statement closure cannot be written inside brackets");
    E0107 = (Error, 107, Parse, "[GRM-16]", "a jump expression may not be an operand");
    E0108 = (Error, 108, Parse, "[ATT-3]", "attribute is not permitted on this statement");
    E0109 = (Error, 109, Parse, "[GRM-15]", "`owned` is not permitted in expression position");

    // --- name resolution, modules, visibility -------------------------------
    E1010 = (Error, 1010, Resolve, "[MOD-3]", "cannot find name in this scope");
    E1020 = (Error, 1020, Resolve, "[GRM-4]", "name is already declared in this block");
    E1030 = (Error, 1030, Resolve, "[TYP-26]", "two items with the same name in one scope");
    E1040 = (Error, 1040, Resolve, "[MOD-3]", "wildcard import from a module that is not a prelude");
    E1041 = (Error, 1041, Resolve, "[MOD-4]", "import cycle between packages");
    E1050 = (Error, 1050, Resolve, "[MOD-7]", "field is read-only outside its module");
    E1051 = (Error, 1051, Resolve, "[MOD-7]", "`read` visibility is valid on fields only");

    E1021 = (Error, 1021, Resolve, "[BLD-11]", "name is not linked in this build");

    // --- types, inference, interfaces, generics, patterns -------------------
    E2010 = (Error, 2010, Types, "[LEX-16]", "literal does not fit its type");
    E2020 = (Error, 2020, Types, "[TYP-4]", "mismatched types");
    E2035 = (Error, 2035, Types, "[CTL-0]", "condition must be `bool`");
    E2036 = (Error, 2036, Types, "[GRM-19]", "this pattern always matches");
    E2030 = (Error, 2030, Types, "[TYP-14]", "a struct carrying a borrow must be declared `@view`");
    E2040 = (Error, 2040, Types, "[TYP-17]", "unsatisfied interface bound");
    E2041 = (Error, 2041, Types, "[TYP-19]", "overlapping `extend` implementations");
    E2050 = (Error, 2050, Types, "[TYP-22]", "interface is not `dyn`-compatible");
    E2060 = (Error, 2060, Types, "[TYP-23]", "cannot infer type");
    E2061 = (Error, 2061, Types, "[TYP-23]", "lambda parameter types cannot be inferred here");
    E2062 = (Error, 2062, Types, "[TYP-23]", "ambiguous type");
    E2070 = (Error, 2070, Types, "[TYP-24]", "ambiguous interface method");
    E2080 = (Error, 2080, Types, "[STR-3]", "`@derive(Copy)` on a type with a field that is not Copy");
    E2090 = (Error, 2090, Types, "[ENM-2]", "non-exhaustive match");
    E2100 = (Error, 2100, Types, "[CLS-2]", "field read before it is initialised");
    E2110 = (Error, 2110, Types, "[CLS-4]", "override of a method that is not virtual");
    E2120 = (Error, 2120, Types, "[IFC-2]", "inherent extension of a type from another package");
    E2130 = (Error, 2130, Types, "[STA-1]", "static initialiser is not comptime-evaluable");
    E2131 = (Error, 2131, Types, "[IV.3]", "array length must be a constant");
    E2031 = (Error, 2031, Types, "[LT-1a]", "`@borrows` names a parameter that is not view-typed");
    E2172 = (Error, 2172, Types, "[GRM-8b]", "cannot index with a type");
    E2173 = (Error, 2173, Types, "[GRM-8b]", "not a type or const-generic argument");
    E2140 = (Error, 2140, Types, "[EXP-5]", "cannot assign to a value expression");
    E2150 = (Error, 2150, Types, "[VI.3]", "`is` requires a type with identity");
    E2151 = (Error, 2151, Types, "[VI.3]", "integer `**` with a negative exponent");
    E2160 = (Error, 2160, Types, "[CTL-7]", "control flow cannot leave a `defer` block");
    E2170 = (Error, 2170, Types, "[IX.5]", "cannot take a reference to a field of a packed struct");
    E2180 = (Error, 2180, Types, "[ERR-2]", "`?` outside a function returning `Option` or `Result`");
    E2200 = (Error, 2200, Types, "[XVIII.4.4]", "type has infinite size");

    // --- range and domain types (`[RNG-*]`, 0.6) ------------------------------
    E2210 = (Error, 2210, Types, "[RNG-2]", "a value of one range type where another was expected");
    E2211 = (Error, 2211, Types, "[RNG-3]", "constant outside the target range type");
    E2212 = (Error, 2212, Types, "[RNG-1]", "range endpoints are not constants of the representation, or are inverted");
    // Two sites name `E2213`: IV.2a's diagnostic list ("an `in` clause on a
    // non-numeric representation") and `[GRM-8d]` (an `in` clause in a
    // position or on an alias shape that admits none). One title covers both,
    // which is what `[DIA-6a]`'s one-code-one-rule check needs — errata ERR-038.
    E2213 = (Error, 2213, Types, "[RNG-1]", "invalid `in` clause on a type alias");
    E2214 = (Error, 2214, Types, "[RNG-5]", "arithmetic between two distinct range types");
    E2215 = (Error, 2215, Types, "[RNG-10]", "range-typed value constructed outside the permitted set");

    // --- coroutines, monomorphisation, migration, membership (0.6.3, 0.7.x) ---
    E2220 = (Error, 2220, Types, "[CORO-2]", "`yield` outside a `gen fn`");
    E2221 = (Error, 2221, Types, "[CORO-6]", "borrow held across a `yield`");
    E2222 = (Error, 2222, Types, "[CORO-10]", "coroutine where an ordinary function is required");
    E2223 = (Error, 2223, Types, "[MONO-7]", "`@never_specialize` on a generic that is not shareable");
    E2224 = (Error, 2224, Types, "[HR-16]", "`migrate_from` may not take a `mut` receiver");
    E2225 = (Error, 2225, Types, "[HR-35]", "`migrate_from` may not panic");
    E2226 = (Error, 2226, Types, "[STD-8]", "`in` on a type that does not implement `Contains`");
    E2227 = (Error, 2227, Types, "[HR-39]", "a `noexcept` foreign call in `migrate_from` may terminate the process");

    // --- ownership, borrows, regions, exclusivity, drops --------------------
    E3010 = (Error, 3010, Ownership, "[EXP-6]", "cannot move out of a field of a type with `drop`");
    E3011 = (Error, 3011, Ownership, "[EXP-6]", "cannot move out of an array or span element");
    E3012 = (Error, 3012, Ownership, "[EXP-6]", "cannot move out of a class field");
    E3013 = (Error, 3013, Ownership, "[EXP-6]", "cannot move out of a reference");
    E3020 = (Error, 3020, Ownership, "[CTL-2]", "iterable is mutated while the loop borrows it");
    // `[DIA-7a]` keys every E3xxx code to a diagnostic shape and forbids
    // emitting one that is absent from its table — but shapes B1, B3, B4, B5,
    // B8, B9 and B10 are given no code there, which leaves the most basic
    // borrow error in the language unreportable. Allocated here beside E3020,
    // which is the aliasing error the table does key. See errata ERR-021.
    E3021 = (Error, 3021, Ownership, "[BRW-1]", "a shared and a mutable borrow overlap");
    E3022 = (Error, 3022, Ownership, "[BRW-1]", "two mutable borrows of the same place");
    E3023 = (Error, 3023, Ownership, "[BRW-1]", "aliased mutation of a value type");
    E3024 = (Error, 3024, Ownership, "[LT-2]", "a struct field would borrow another field of the same struct");
    E3025 = (Error, 3025, Ownership, "[BRW-4]", "a method takes all of `self`, defeating disjoint-field access");
    E3026 = (Error, 3026, Ownership, "[CLO-4]", "a closure outlives what it captures");
    E3027 = (Error, 3027, Ownership, "[FN-2a]", "a `mut` argument is not a mutable place");
    E3030 = (Error, 3030, Ownership, "[CLO-2]", "closure would move a captured value out");
    E3040 = (Error, 3040, Ownership, "[OWN-3]", "use of moved value");
    E3041 = (Error, 3041, Ownership, "[OWN-4]", "value moved in a previous loop iteration");
    E3014 = (Error, 3014, Ownership, "[THR-5]", "scope binding may not be moved");
    E3015 = (Error, 3015, Ownership, "[THR-6]", "value whose drop is required may not be leaked");
    E3016 = (Error, 3016, Ownership, "[CLS-7a]", "`self` escapes its own drop");
    E3042 = (Error, 3042, Ownership, "[EXP-6]", "partial move then use of the whole value");
    E3063 = (Error, 3063, Ownership, "[TYP-15]", "stored view may not outlive its source");
    E3064 = (Error, 3064, Ownership, "[LT-2]", "two independent regions in one view struct");
    E3095 = (Error, 3095, Ownership, "[DSJ-4]", "disjointness is not establishable for these operands");
    E3096 = (Error, 3096, Ownership, "[ARN-6]", "arena is scoped here");
    E3050 = (Error, 3050, Ownership, "[BRW-7]", "use of an uninitialised or moved place");
    E3060 = (Error, 3060, Ownership, "[LT-3]", "borrowed value does not live long enough");
    E3061 = (Error, 3061, Ownership, "[LT-4]", "arena allocation cannot outlive its arena");
    E3062 = (Error, 3062, Ownership, "[XVIII.4.7]", "returned reference does not derive from a parameter");
    E3070 = (Error, 3070, Ownership, "[DRP-1]", "`drop` cannot be called explicitly");
    E3080 = (Error, 3080, Ownership, "[EXC-3]", "overlapping access through the same handle");
    E3090 = (Error, 3090, Ownership, "[ARN-3]", "arena allocation of a type that needs `drop`");
    E3100 = (Error, 3100, Ownership, "[UNS-1]", "this operation requires an `unsafe` block");
    // Registered ahead of its emitter: `[UNS-10b]` forbids `UnsafeCell` in
    // `@static_safe` code, and `UnsafeCell` itself is 0.8.5 spec and not yet
    // built. `[DIA-6a]` requires a code named in the specification to be in the
    // registry, so it is registered here rather than left dangling.
    E3105 = (Error, 3105, Ownership, "[UNS-10b]", "`UnsafeCell` is not permitted in `@static_safe` code");

    // --- effects and contracts ----------------------------------------------
    E4001 = (Error, 4001, Effects, "[EFF-5]", "`@noalloc` function reaches an allocation");
    E4030 = (Error, 4030, Effects, "[EFF-13]", "`@static_safe` function performs a dynamically checked access");
    E4040 = (Error, 4040, Effects, "[EFF-17]", "`@nopanic(explicit)` function reaches a panic");
    E4010 = (Error, 4010, Effects, "[EFF-2]", "implementation does not satisfy the interface's contract");
    E4020 = (Error, 4020, Effects, "[SIMD-2]", "`@simd(assert)` loop did not vectorise");

    // --- I/O, locking and determinism (0.6, 0.6.3) ----------------------------
    E4041 = (Error, 4041, Effects, "[EFF-20]", "`@noio` function reaches I/O");
    E4042 = (Error, 4042, Effects, "[EFF-21]", "`@nolock` function acquires a lock");
    E4070 = (Error, 4070, Effects, "[DET-1]", "`@deterministic` function reaches a nondeterministic operation");
    // Registered and never emitted, so the number is not reused: the case it
    // named is reported as `E4070` with the foreign frame named (XX §6).
    E4071 = (Error, 4071, Effects, "[DET-8]", "reserved: `@deterministic` reaching an undeclared `extern`");
    E4072 = (Error, 4072, Effects, "[DET-5]", "`@fastmath` or `@fp(contract)` inside `@deterministic`");
    // Registered and never emitted: `[CORO-5]` makes it unreachable in v1.
    E4073 = (Error, 4073, Effects, "[CORO-5]", "reserved: a coroutine frame that allocates in `@noalloc`");

    // --- FFI -----------------------------------------------------------------
    E5001 = (Error, 5001, Ffi, "[FFI-5]", "imported type layout does not match");
    E5010 = (Error, 5010, Ffi, "[FFI-12]", "overlay does not match the C declaration");
    E5020 = (Error, 5020, Ffi, "[FFI-15]", "`str` is not NUL-terminated");
    E5030 = (Error, 5030, Ffi, "[FFI-17]", "`std::function` cannot be imported");
    E5040 = (Error, 5040, Ffi, "[FFI-21]", "capturing closure passed where a C function pointer is expected");
    E5041 = (Error, 5041, Ffi, "[FFI-21]", "retained callback needs a release function in the overlay");
    E5002 = (Error, 5002, Ffi, "[DIA-18]", "foreign call requires `unsafe`: its contract is incomplete");
    E5011 = (Error, 5011, Ffi, "[FFI-30]", "one C identity imported with two different layouts");
    E5012 = (Error, 5012, Ffi, "[FFI-11]", "pointer contract has no count");
    E5014 = (Error, 5014, Ffi, "[FFI-29]", "two packages request the same implementation macro");
    E5015 = (Error, 5015, Ffi, "[FFI-31]", "type may not cross the boundary");
    E5016 = (Error, 5016, Ffi, "[FFI-8]", "`_Atomic` layout does not match `Atomic[T]`");
    E5017 = (Error, 5017, Ffi, "[FFI-8]", "struct with a flexible array member is unsized");
    E5018 = (Error, 5018, Ffi, "[FFI-8]", "`va_list` may not be constructed");
    E5031 = (Error, 5031, Ffi, "[FFI-32b]", "C++ parameter cannot be mapped");
    E5032 = (Error, 5032, Ffi, "[FFI-32]", "opaque C++ mirror may not be constructed");
    W5002 = (Warning, 5002, Ffi, "[FFI-9]", "unsupported calling convention; declaration skipped");
    E5090 = (Error, 5090, Ffi, "[UNS-6]", "inline assembly is not supported by the C backend");

    // --- grades, adoption, C++ boundary, text (0.6, 0.7.x) --------------------
    E5034 = (Error, 5034, Ffi, "[FFI-48]", "unsupported C++ construct");
    E5050 = (Error, 5050, Ffi, "[FFI-34]", "a foreign fact claims a grade whose evidence is absent or stale");
    E5051 = (Error, 5051, Ffi, "[FFI-35]", "a foreign callee retains a pointer the contract does not declare `retained`");
    E5052 = (Error, 5052, Ffi, "[FFI-36b]", "`adopt` on a handle whose overlay declares `adopt = false`");
    E5053 = (Error, 5053, Ffi, "[FFI-37]", "an instrumented run contradicted a declared foreign effect");
    E5054 = (Error, 5054, Ffi, "[RNG-10b]", "range type in a foreign signature");
    E5055 = (Error, 5055, Ffi, "[FFI-17b]", "an Ember generic may not instantiate a C++ template");
    E5056 = (Error, 5056, Ffi, "[FFI-39]", "override of a virtual not named in `virtuals`");
    E5057 = (Error, 5057, Ffi, "[FFI-39]", "`virtuals` names a method that is not virtual in the header");
    E5058 = (Error, 5058, Ffi, "[FFI-39b]", "foreign base has no default constructor and no declared `init`");
    E5059 = (Error, 5059, Ffi, "[FFI-17d]", "`@ffi(trampoline)` base has no virtual destructor and no `owner`");
    E5060 = (Error, 5060, Ffi, "[FFI-39c]", "upcast in a context that cannot hold the `Retained` token");
    E5061 = (Error, 5061, Ffi, "[FFI-43]", "imported C++ function has no exception policy");
    E5062 = (Error, 5062, Ffi, "[FFI-43a]", "contradictory C++ exception policies");
    E5063 = (Error, 5063, Ffi, "[TXT-2]", "foreign bytes reach `str` without validation");
    E5064 = (Error, 5064, Ffi, "[TXT-3]", "interior null in a value converted to `CStr`");
    E5065 = (Error, 5065, Ffi, "[SEL-2]", "`Shared`/`Weak` and `CppShared`/`CppWeak` do not interconvert");

    // --- comptime -------------------------------------------------------------
    E6001 = (Error, 6001, Comptime, "[CT-3]", "comptime evaluation exceeded its limits");
    E6002 = (Error, 6002, Comptime, "[CT-4]", "comptime evaluation is not deterministic");
    E6010 = (Error, 6010, Comptime, "[XVIII.5]", "operation is not supported at compile time");

    // --- concurrency ----------------------------------------------------------
    E7001 = (Error, 7001, Concurrency, "[THR-1]", "`@sync` class has a field that is not Sync");
    E7011 = (Error, 7011, Concurrency, "[PAR-2a]", "parallel loop has a loop-carried dependency");
    E7010 = (Error, 7010, Concurrency, "[PAR-2]", "parallel loop writes to a shared place");
    E7020 = (Error, 7020, Concurrency, "[ECS-4]", "systems in one parallel run have conflicting access sets");

    // --- layout ---------------------------------------------------------------
    E8001 = (Error, 8001, Layout, "[GPU-10]", "GPU layout does not match the CPU layout");

    // --- build, manifest, toolchain -------------------------------------------
    // `[TYP-9c]` and `[MAN-3]` both name `E9010`. Both are followed.
    E9010 = (Error, 9010, Build, "[TYP-9c], [MAN-3]", "the toolchain or the manifest names something it cannot honour");
    E9011 = (Error, 9011, Build, "[TYP-9a]", "toolchain cannot disable FP contraction");
    E9020 = (Error, 9020, Build, "[BLD-FFI-1a]", "translation units of one target disagree on an inherited flag");
    E9021 = (Error, 9021, Build, "[BLD-FFI-1b]", "C++ standard library and CRT heap could not be determined");
    E9001 = (Error, 9001, Build, "[MAN-1]", "invalid manifest");
    E9002 = (Error, 9002, Build, "[BLD-4]", "the C toolchain could not be found or failed");
    E9003 = (Error, 9003, Build, "[CLI-1]", "invalid command line");

    // Registered, never emitted: XX §6 names `E9012` under "manifest sections"
    // and no rule assigns it. `[MAN-3]` was given it here for a while, because
    // `[DIA-6a]` wants one code per rule and `E9010` was already `[TYP-9c]`'s
    // — but `[MAN-3]` says `E9010` in as many words, and the document decides
    // (errata ERR-039). It is kept in the registry because XX §6 names it and
    // `[DIA-6a]` requires every named code to be registered.
    E9012 = (Error, 9012, Build, "[MAN-3]", "reserved for a manifest section");
    E9013 = (Error, 9013, Build, "[MAN-5]", "invalid `[ffi]` manifest section");
    E9030 = (Error, 9030, Build, "[HR-18]", "hot reload refused");
    E9031 = (Error, 9031, Build, "[MAN-7]", "invalid `reload` value");
    // Registered and never emitted: `[HR-6]`'s permanent thunks make a
    // reloadable function's address stable, so it cannot be stranded.
    E9032 = (Error, 9032, Build, "[HR-6]", "reserved: a reloadable function's address escaping to foreign code");
    E9033 = (Error, 9033, Build, "[PRF-2]", "`reload` is forbidden in `shipping`");
    E9034 = (Error, 9034, Build, "[MONO-3]", "invalid `max_instantiations` value");
    E9035 = (Error, 9035, Build, "[HR-12a]", "packages in one process disagree about the object-header layout");
    E9036 = (Error, 9036, Build, "[HR-29]", "a reloadable package may not link the runtime statically");
    E9037 = (Error, 9037, Build, "[ABI-2]", "reload ABI mismatch on image load");

    // --- warnings --------------------------------------------------------------
    W0001 = (Warning, 1, Lex, "[LEX-11]", "dangling doc comment");
    W1002 = (Warning, 1002, Resolve, "[GRM-12]", "binding shadows an enum variant of the same name");
    W2091 = (Warning, 2091, Types, "[CTL-5]", "unreachable match arm");
    W2015 = (Warning, 2015, Types, "[LEX-17a]", "float literal loses precision at f32");
    W2111 = (Warning, 2111, Types, "[CLS-4]", "`virtual` has no effect in a final class");
    W2190 = (Warning, 2190, Types, "[ERR-5]", "unused `Result`");
    W5001 = (Warning, 5001, Ffi, "[FFI-6]", "function-like macro ignored");
    W5031 = (Warning, 5031, Ffi, "[FFI-19]", "declaration skipped: the header could not be parsed");

    W2220 = (Warning, 2220, Types, "[MONO-3]", "instantiation ceiling exceeded");
    W3012 = (Warning, 3012, Ownership, "[UNS-8]", "unsafe block with no SAFETY note");
    W5033 = (Warning, 5033, Ffi, "[FFI-40]", "`&&`-qualified member skipped");
    W5034 = (Warning, 5034, Ffi, "[FFI-41]", "anonymous-namespace entity skipped");
    W5050 = (Warning, 5050, Ffi, "[FFI-34]", "unbacked or stale grade");
    W5054 = (Warning, 5054, Ffi, "[FFI-37b]", "unexercised foreign fact");
    W9030 = (Warning, 9030, Build, "[HR-18]", "reload requires a restart");

    // --- lints ------------------------------------------------------------------
    L1001 = (Lint, 1001, Resolve, "[LNT-1]", "unused binding");
    L1002 = (Lint, 1002, Resolve, "[LNT-2]", "assignment declares a new binding");
    L3011 = (Lint, 3011, Ownership, "[CELL-7]", "`RefCell` guard held across a call");
    L3013 = (Lint, 3013, Ownership, "[EXC-7]", "long-term access held across a call");
    L3014 = (Lint, 3014, Ownership, "[LT-1b]", "return region is the intersection of N parameters");
    L2001 = (Lint, 2001, Types, "[XX.8]", "unnecessary clone");
    L2002 = (Lint, 2002, Types, "[XX.8]", "large Copy value passed by value");
    L3001 = (Lint, 3001, Ownership, "[WK-1]", "potential reference cycle");
    L3002 = (Lint, 3002, Ownership, "[XX.8]", "borrow held longer than necessary");
    L3010 = (Lint, 3010, Ownership, "[UNS-3]", "unsafe block larger than necessary");
    L4001 = (Lint, 4001, Effects, "[XX.8]", "allocation in a hot loop");
    L4002 = (Lint, 4002, Effects, "[XX.8]", "dynamic dispatch on a final type");
    L5001 = (Lint, 5001, Ffi, "[XX.8]", "unsafe extern without a contract");
    L5002 = (Lint, 5002, Ffi, "[XX.8]", "conversion at the FFI boundary copies");
    L7001 = (Lint, 7001, Concurrency, "[XX.8]", "lock held across a call that may block");

    L2003 = (Lint, 2003, Types, "[RNG-5a]", "fallible construction where a total one exists");
    L2004 = (Lint, 2004, Types, "[LNT-4]", "`gen fn` with no `yield`");
    L2005 = (Lint, 2005, Types, "[LNT-5]", "`@noreload` function calls a reloadable one inside a loop");
    L3015 = (Lint, 3015, Ownership, "[UNS-7]", "undocumented unsafe obligation");
    L3016 = (Lint, 3016, Ownership, "[UNS-7]", "`@safety` text still reads `TODO`");
    L3017 = (Lint, 3017, Ownership, "[WK-4]", "reference cycle detected");
    L3018 = (Lint, 3018, Ownership, "[UNS-9]", "`unsafe` block with no reason category");
}

/// Look a code up by its rendered form, e.g. `"E3040"`. Used by `ember explain`.
pub fn lookup(text: &str) -> Option<Code> {
    REGISTRY.iter().copied().find(|c| c.to_string().eq_ignore_ascii_case(text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn numbers_are_unique_within_a_kind() {
        let mut seen = HashSet::new();
        for code in REGISTRY {
            assert!(seen.insert((code.kind, code.number)), "duplicate code {code}");
        }
    }

    #[test]
    fn numbers_fall_in_their_subsystem_range() {
        for code in REGISTRY {
            let (lo, hi) = code.subsystem.range();
            assert!(
                code.number >= lo && code.number <= hi,
                "{code} ({:?}) is outside {lo}..={hi}",
                code.subsystem
            );
        }
    }

    #[test]
    fn titles_follow_the_message_style() {
        for code in REGISTRY {
            // `[DIA-2]`: lowercase first letter. An acronym (`GPU`, `ABI`) and
            // a backtick-quoted identifier are the only things that may start
            // a message with a capital.
            let first_word = code.title.split_whitespace().next().unwrap();
            let is_acronym = first_word.chars().all(|c| c.is_uppercase() || !c.is_alphabetic());
            let is_quoted = first_word.starts_with('`');
            assert!(
                !code.title.chars().next().unwrap().is_uppercase() || is_acronym || is_quoted,
                "{code} title starts with a capital: {:?}",
                code.title
            );
            assert!(
                !code.title.ends_with('.'),
                "{code} title ends with a period: {:?}",
                code.title
            );
            let words: Vec<&str> = code
                .title
                .split(|c: char| !c.is_alphanumeric())
                .collect();
            assert!(
                !words.contains(&"you") && !words.contains(&"your"),
                "{code} addresses the reader directly: {:?}",
                code.title
            );
        }
    }

    #[test]
    fn every_code_names_a_rule() {
        for code in REGISTRY {
            assert!(!code.rule.is_empty(), "{code} has no rule reference");
        }
    }

    #[test]
    fn lookup_round_trips() {
        assert_eq!(lookup("E3040"), Some(E3040));
        assert_eq!(lookup("e3040"), Some(E3040));
        assert_eq!(lookup("L3001"), Some(L3001));
        assert_eq!(lookup("E9999"), None);
    }

    #[test]
    fn display_is_four_digits() {
        assert_eq!(E0001.to_string(), "E0001");
        assert_eq!(E3040.to_string(), "E3040");
        assert_eq!(W0001.to_string(), "W0001");
    }
}
