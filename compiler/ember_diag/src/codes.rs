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
    /// 0.9.9 `[CLI-19]`, `[PHIL-12]` — a construct this compiler does not
    /// implement (`E0900`) or the specification leaves unspecified (`E0901`).
    Support,
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
            Subsystem::Support => (900, 999),
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

/// `[BLD-1]` — a code in a kept record is written by its name and read back from this compiler's
/// registry ([`lookup`]).
impl serde::Serialize for Code {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> serde::Deserialize<'de> for Code {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Code, D::Error> {
        let name = String::deserialize(deserializer)?;
        lookup(&name).ok_or_else(|| serde::de::Error::custom(format!("no diagnostic code `{name}`")))
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
    E0005 = (Error, 5, Lex, "[LEX-14], [LEX-15]", "a keyword used as a name, any keyword (`type = 3`: \"`type` is a keyword\", help \"write `r#type` to use the word as a name\"), or a word reserved for a future version");
    E0006 = (Error, 6, Lex, "[VER-8]", "a `#!` language directive names a version other than the current one");
    E0007 = (Error, 7, Lex, "[LT-6]", "named lifetimes are not supported in this version");
    E0008 = (Error, 8, Lex, "[LEX-22]", "unterminated character literal");
    // ODR-228 — what `E0101` reported besides an unclosed delimiter: a literal II.5's grammar
    // does not write (an escape, a character's value, a raw string's prefix, a byte string's
    // character, an f-string's lone `}`).
    E0009 = (Error, 9, Lex, "[LEX-26]", "malformed literal");
    E0012 = (Error, 12, Lex, "[LEX-1a]", "a Unicode bidirectional format character (U+202A to U+202E, U+2066 to U+2069) in a source file, an overlay or the manifest, a comment or a literal included");

    // --- parser -------------------------------------------------------------
    E0100 = (Error, 100, Parse, "[GRM-2], [TYP-17], [GRM-40], [LT-48], [GRM-3a], [GRM-29], [GRM-34], [TYP-45a], [GRM-13]", "unexpected token, among them Python's `global`, a statement at module level outside the entry file (whose message gives the declaration to write: `static mut FRAME: int = 0`), a bound written without `needs` or joined with `+` (whose message gives the line with `needs`), and a clause after a function's brackets that Ember does not have (`changes FRAME`, `reads renderer`, `with` before the captures), whose message gives the first line to write, and a line that begins with `[` and is not directly above a function, struct or class (the line of a `match` arm excepted), `runtime` on a field, a result type, a type argument, a capture or a local declared without its type (it goes on the parameter or the typed local a call goes through), and a holding word between `weakref` and its target's type (`weakref shares T`, `weakref owns T`, `weakref ref T`), a statement outside the entry file or an import below an item or a statement, a one-element tuple (`(e,)`), a field in an `extend` block, and a word of `[TYP-45]` stacked on a word (`owns ref T`, `ref ref T`)");
    E0101 = (Error, 101, Parse, "[LEX-6]", "unclosed delimiter or literal");
    // The specification numbers these three E0010, E0011 and E0020, inside the
    // lexer's range, although all three need a parsed tree. Renumbered into
    // the parser range by the owner, 2026-09-07; see docs/spec-errata.md
    // ERR-001 for the mapping.
    E0102 = (Error, 102, Parse, "[GRM-23]", "membership test chained (`a in b in c`)");
    E0103 = (Error, 103, Parse, "[GRM-10]", "match arms mix statement and expression form");
    // Two rules name this code. `[ATT-1]`/`[ATT-2]`/`[ATT-5]`/`[FFI-34a]` use
    // it for an attribute or grade suffix that is not admitted where it
    // appears; `[GRM-23]` uses it for `a in b in c`. `[DIA-6a]` asks for one
    // code per rule and the document does not honour that here — the reading
    // that keeps both is that `E0104` means "this construct is not admitted in
    // this position", which covers both uses (errata ERR-026).
    E0104 = (Error, 104, Parse, "[ATT-1], [PHIL-12], [FFI-11], [LEX-10], [CG-C-3a]", "attribute or construct not admitted here, among them an attribute that is gone, with the one to write (`@sync`, `@static_safe`, `@no_runtime_checks`, `@fp(contract)`, `@borrows`, `@component`), and `unsafe` written as a contract word, in an overlay entry or in `@ffi(…)`, with the `unsafe fn` line to write, and a `#!` directive below an import or an item; `@inline` on a function that may call itself through direct calls");
    E0105 = (Error, 105, Parse, "[GRM-18]", "`;` is not a statement separator");
    E0106 = (Error, 106, Parse, "[GRM-17]", "a lambda body of more than one statement, or an indented block after a lambda's `:`");
    E0107 = (Error, 107, Parse, "[GRM-16]", "a jump expression may not be an operand");
    E0108 = (Error, 108, Parse, "[GRM-20]", "attribute is not permitted on this statement");
    E0109 = (Error, 109, Parse, "[OWN-9], [GRM-15], [OWN-13], [FN-2a]", "`owns` before a name whose place already hands the value over with no word (`e.mesh = m`; `b = a` for a value `=` hands over), after `return`, or before `fn` anywhere but an argument for an `owns` parameter, `owns` before a value made on the line for a place that is not an `owns T` (`b = owns (n * 2)`), and `weakref thread_shared x` before a value whose type already says `thread_shared` (`weakref x` is that link); `owns` before a function from a variable given to an `F`-bounded parameter; `owns` before a value made in the call with an owner's or a link's word (`owns shares mut game`), which says what is given (verdicts 147 and 160); a second `owns` before a value made in the call for a parameter whose type is an `owns T`, where one `owns` says both the block and the hand-over");
    E0110 = (Error, 110, Parse, "[GRM-33]", "function declared without a body outside an interface, extern block or abstract class");
    E0111 = (Error, 111, Parse, "[GRM-36]", "`ref` of an expression that is not a place");
    E0112 = (Error, 112, Parse, "[GRM-39]", "nested more deeply than the compiler accepts");
    E0113 = (Error, 113, Parse, "[GRM-42], [FN-1]", "`mut` alone on a name: a pattern name, a parameter, a lambda parameter, `this` (`static mut` is a declaration and is not this)");
    E0114 = (Error, 114, Parse, "[GRM-43], [LT-46]", "`sharedref` not followed by `mut`");
    E0115 = (Error, 115, Parse, "[GRM-44]", "`Some(...)` in an `if` or `while` header");
    E0116 = (Error, 116, Parse, "[GRM-42]", "in a list of names declared without types, a name with no word after a name with a word");
    E0117 = (Error, 117, Parse, "[FN-1]", "`ref` or `ref mut` written in a parameter's type instead of on its name");
    E0118 = (Error, 118, Parse, "[GRM-41]", "a first line whose parts are out of order (parameters, `owns this`, `-> T`, `from`)");
    E0119 = (Error, 119, Parse, "[GRM-47]", "`owns` as an operand of an operator (`owns i * 7`), which reads its operands in place and takes nothing over: a computed value handed over goes in brackets, `m.insert(owns (i * 7), owns i)`");
    E0120 = (Error, 120, Parse, "[GRM-42], [CLO-2]", "`ref mut` in a lambda's capture list");
    // 0.9.9 `[CLI-19]`: a construct the specification defines and this
    // compiler does not implement yet is rejected by name, never silently.
    E0900 = (Error, 900, Support, "[PHIL-12], [CLI-19]", "construct not implemented by this compiler");
    // `[PHIL-12]`: where the specification is silent, or two of its rules
    // disagree, the construct is rejected and the gap raised with the owner.
    E0901 = (Error, 901, Support, "[PHIL-12], [OBJ-1]", "construct this specification leaves unspecified, among them a `weakref` whose operand's type carries no weak count by a route `[OBJ-1]` does not list");

    // --- name resolution, modules, visibility -------------------------------
    E1010 = (Error, 1010, Resolve, "[MOD-3], [STR-2], [DET-4]", "a name or form the program may not use here: not in scope (`this` outside a method or in a `static fn`), gone from Ember (`self`, `Box`, `ForeignBox`, `RefCell`, `Weak`, `Shared`, the mode `owned`, `Sync` as a property, `cast x if D`, `cast x when D`, `upgrade`, `mem.forget`), or written with a word instead (`a.clone()` is `copy of a`; `.copied()` is `.copy_each()`; `det.log` is `det.ln`); `this` or a bare field name in a field default");
    E1020 = (Error, 1020, Resolve, "[GRM-4], [GRM-5]", "name is already declared in this block; a name twice in one target list");
    E1030 = (Error, 1030, Resolve, "[TYP-26]", "two items with the same name in one scope, such as a second `main` (a statement at module level is `E0100`)");
    E1031 = (Error, 1031, Resolve, "[MOD-8]", "a name bound by two glob imports is used; a name bound by a glob import and by the prelude");
    // ODR-201 — every item has one path (`[MNG-1]`).
    E1032 = (Error, 1032, Resolve, "[MOD-1]", "an item has the path of a module or the name of a built-in type");
    E1041 = (Error, 1041, Resolve, "[MOD-4]", "import cycle between packages");
    E1042 = (Error, 1042, Resolve, "[STD-6]", "a use of a name of a layer of `std` the package does not declare");
    E1050 = (Error, 1050, Resolve, "[MOD-7]", "field is read-only outside its module");
    E1051 = (Error, 1051, Resolve, "[MOD-7]", "`read` visibility is valid on fields only");
    E1052 = (Error, 1052, Resolve, "[MOD-2], [STR-1]", "item, field or constructor not visible here");
    E1060 = (Error, 1060, Resolve, "[MOD-3]", "no module of that name");
    E1061 = (Error, 1061, Resolve, "[MOD-3]", "no item of that name in the module");

    E1021 = (Error, 1021, Resolve, "[BLD-11]", "name is not linked in this build");

    // --- types, inference, interfaces, generics, patterns -------------------
    E2010 = (Error, 2010, Types, "[LEX-16]", "literal does not fit its type; a suffixed literal that does not fit its suffix's type (`128i8`)");
    E2011 = (Error, 2011, Types, "[TYP-31], [LEX-24]", "negative literal index");
    E2020 = (Error, 2020, Types, "[TYP-4], [CLO-3], [TYP-45], [TYP-6], [TYP-5], [UNS-12], [FN-6], [GRM-13]", "mismatched types, among them a lambda parameter whose word differs from the function type it is given to (a plain `fn(s)` where the type hands the value over, `fn(owns String)`); a `weakref` link used as its target without opening it; an object stored into a `weakref` place without `weakref`; a value given to a `using A` place that was not made in an instance of `A`; a `weakref T` link given to a `weakref thread_shared T` place, or the reverse (neither converts); an integer cast to an enum (`7 as Color`), `as` between types no pair of `[TYP-6]` lists, and a `ref mut Dog` given to a `ref mut Animal` place; an `unsafe fn` written as `drop`, as an `override` of a method that is not `unsafe`, or as the implementation of an interface method the interface declares without `unsafe`; a parameter or result of an `extern \"C\" fn` type outside what an export's signature admits; a list pattern against a `String` or `StrView`, or against a `[T; N]` whose `N` can never equal the pattern's count");
    E2021 = (Error, 2021, Types, "[TYP-25], [STR-1], [CLS-3]", "a call whose arguments do not fit the callee's parameters: a positional argument after a named one, more arguments than parameters, a parameter with no default given none, a name no parameter has, one parameter given twice; at a memberwise constructor or `super.init(…)`, the same of the fields (shape N6)");
    E2035 = (Error, 2035, Types, "[CTL-0]", "condition must be `bool`");
    E2036 = (Error, 2036, Types, "[GRM-19]", "this pattern always matches: an `if` or `while` opening whose left side always holds a value, or a kind test that always passes");
    E2037 = (Error, 2037, Types, "[TYP-44], [DSP-4]", "a kind test that can never pass: the type tested for does not implement the interface, or derive from the base class, of the value tested");
    E2030 = (Error, 2030, Types, "[TYP-34]", "`@view` on a type that is not a view");
    E2040 = (Error, 2040, Types, "[IFC-6], [TYP-17], [BLD-14], [ERR-4], [IFC-3], [ERR-3], [TYP-43], [TYP-36], [TYP-32]", "unsatisfied interface bound; for a `pub` generic whose bounds are not written, or a generic that writes some of its bounds and not all the body needs, one message on the first line of its body that needs a missing bound, giving the whole line of what it needs; `unwrap` or `expect` on a `Result` whose error type has no `Display`; an implementation that does not meet its interface's parents for every type it covers, at the implementation (a one-thread error converted into `AnyError` at `?` or `.into()`, whose `From[E]` needs `Send` and `ThreadShared`; a generic implementation without the needs its parents put on its parameters, given the bounded line); an implementation's method whose first line differs from the interface's; `ThreadShared` written by hand on a struct, an enum or a class that cannot be subclassed; a bound needing `Ord` on `owns dyn I`; a `some` list's hidden type lacking a marker written");
    E2041 = (Error, 2041, Types, "[TYP-19]", "overlapping `extend` implementations");
    E2042 = (Error, 2042, Types, "[STD-27]", "`Float` is implemented only by `f32` and `f64`");
    E2043 = (Error, 2043, Types, "[IFC-4]", "an associated type's default leads back to itself");
    E2050 = (Error, 2050, Types, "[IFC-5], [TYP-22]", "interface is not `dyn`-compatible");
    E2060 = (Error, 2060, Types, "[TYP-23], [TYP-22]", "cannot infer type; a `dyn I` that leaves an associated type of `I` unnamed");
    E2061 = (Error, 2061, Types, "[TYP-23]", "lambda parameter types cannot be inferred here");
    E2062 = (Error, 2062, Types, "[TYP-23]", "ambiguous type");
    E2070 = (Error, 2070, Types, "[TYP-24]", "ambiguous interface method");
    E2071 = (Error, 2071, Types, "[SOA-1]", "`SoA[T]` of a type that is not a struct");
    E2072 = (Error, 2072, Types, "[STD-13], [SOA-1], [FFI-42]", "no method or field of that name (with the Ember name for a Python one; shape N13 for an undeclared field; in a generic body that is not `pub`, a method no bound's interface declares); a flattened field's name as a column (`ps.pos`); `g[i]` on an imported C++ class whose only `operator[]` is not `const` (it has `index_mut` alone)");
    E2073 = (Error, 2073, Types, "[STD-26]", "`len` of a string (Python counts characters, `s.len()` bytes)");
    E2074 = (Error, 2074, Types, "[ECS-4a]", "a component type listed twice in one ECS view, or listed and excluded");
    E2230 = (Error, 2230, Types, "[CTL-10]", "a name assigned in every branch at different types");
    E2228 = (Error, 2228, Types, "[CLO-3]", "retired: folded into `E2020`: a lambda parameter whose word differs from the function type it is given to");
    E2080 = (Error, 2080, Types, "[STR-3], [ARN-11], [ERR-3], [SER-1], [DRV-1]", "`@derive(Copy)` on a type with a field that is not Copy, or on a type that declares `drop`; `@derive(Zeroable)` on an enum whose zero bytes are no variant or whose zero variant's payload is not `Zeroable`; a derive's requirement a field does not meet, `@from`'s included (`@derive(Copy)` on a type with a `String` field); a `@ser(skip)` field with no declared default whose type has no `Default`");
    E2090 = (Error, 2090, Types, "[ENM-2]", "non-exhaustive match");
    E2100 = (Error, 2100, Types, "[CLS-2]", "field read before it is set, `init` calling a method whose body reads a field not yet set included; in a `pub` `open` or `abstract` class, `init` calling a `virtual` method before every field is set");
    E2101 = (Error, 2101, Types, "[CLS-10]", "derived class with a field that has no default needs an `init`");
    E2102 = (Error, 2102, Types, "[CLS-11]", "derived `init` uses `this` or an inherited field before `super.init`, or leaves an own field unassigned");
    E2103 = (Error, 2103, Types, "[CLS-7]", "`this` given another object in a class method");
    E2104 = (Error, 2104, Types, "[CLS-9]", "a `let` field changed after `init`, for every type");
    E2106 = (Error, 2106, Types, "[GRM-44], [GRM-45], [DSP-4]", "an `if` or `while` opening, or a kind test, that names a name already in use");
    E2108 = (Error, 2108, Types, "[CLS-2]", "`owns this` on `init`");
    E2110 = (Error, 2110, Types, "[CLS-4]", "override of a method that is not virtual");
    E2111 = (Error, 2111, Types, "[CLS-4]", "a method that replaces an inherited virtual one without `override`");
    E2112 = (Error, 2112, Types, "[CLS-4]", "`virtual` in a class that cannot be subclassed");
    E2113 = (Error, 2113, Types, "[CLS-4]", "an `override` whose first line differs from the virtual method it replaces: a parameter's name, word or type, the words on `this`, or the result");
    E2114 = (Error, 2114, Types, "[TYP-45], [TYP-46], [ALC-1], [LT-48], [FFI-54], [FFI-36a]", "a type word that cannot apply to its type: `owns` on a class type (it always has its own block); `using` after a type with no heap memory of its own; `into_raw()` on a value Ember made `using` a manager other than the built-in one (C ends it through the runtime, which gives back only blocks of the built-in manager); `runtime` before a type with one kind (a struct, an enum, a class neither `open` nor `abstract`, `some I`, a list, a number); `owns` before a value Ember makes of a foreign type, whose `owns T` holds only an object foreign code made");
    E2115 = (Error, 2115, Types, "[GRM-41], [FN-4]", "`static fn` outside a type body");
    E2116 = (Error, 2116, Types, "[OWN-8], [TYP-36]", "`copy of` before what it cannot copy as itself: a `weakref` link, a `ref mut T` view, a `MutSpan[T]`, or a struct holding one, a type with no `Clone`, a value whose views point into its own `let` field");
    E2117 = (Error, 2117, Types, "[CLS-12], [EFF-27]", "`@cached` on a method whose result it cannot keep safely: one with other parameters; one whose body changes anything, or reads a `shares`, `shares mut`, `thread_shared`, `weakref`, `sharedref mut` or `Cell` field, or a field another thread may change by its own operations (an `Atomic`, a `Mutex`, an `RwLock`, a channel end, a `guarded by` field); on a `virtual` method or an `override`; a `sharedref mut` link or `shares mut` owner made into what a `@cached` method reads, at the line that makes it; a `@cached` body that reads a field of a class object of a class carrying a weak count, reached through a field or element of `this`; on one whose body reads a `static mut` or a `static` with its own operations");
    E2118 = (Error, 2118, Types, "[LT-45], [TYP-15]", "a `from` that is missing where a view needs one (a view field of a class), names what a view cannot point into for its life (a field that is not `let`, a fixed array or an inline struct field), or names two things (a parameter and a field of one name)");
    E2119 = (Error, 2119, Types, "[WK-16], [WK-11], [OWN-13]", "a `weakref` of a part (a field, an element stored inline), of a value held `owns` or of a new value, whose target does not live on its own, and of a value whose kind is known only while the program runs (`dyn I`, `owns dyn I`, `shares dyn I`, `some I`; the type `weakref dyn I` too), whose header the link cannot know");
    E2120 = (Error, 2120, Types, "[IFC-2]", "inherent extension of a type from another package");
    E2121 = (Error, 2121, Types, "[TYP-20]", "an implementation outside the packages that may write it");
    E2122 = (Error, 2122, Types, "[RC-1], [OWN-13]", "`shares`, `shares mut` or `thread_shared` made of a place that has one owner and is not a local of this function (a list element, a field, a plain parameter), or of `this` in a method (`shares this`, `shares mut this`)");
    E2123 = (Error, 2123, Types, "[OWN-10], [GRM-15], [TYP-46]", "a value made for a place that owns it alone (an `owns` field, an `Option[owns T]`, a list of `owns` values), at first or later, without `owns` on the line that makes it");
    E2130 = (Error, 2130, Types, "[TYP-1]", "a `const` of a type that owns heap memory; use a `static`");
    E2131 = (Error, 2131, Types, "[CT-1]", "array length must be a constant");
    E2031 = (Error, 2031, Types, "[LT-1a]", "`from` names a parameter the result cannot point into (a number, or an `owns` parameter that is not a view)");
    E2172 = (Error, 2172, Types, "[GRM-8b]", "cannot index with a type");
    E2173 = (Error, 2173, Types, "[TYP-45], [IFC-5], [OBJ-2]", "a type position holding what is not a type: a bare interface name used as a type (`Array[Shape]`), `dyn I` with no word outside a parameter, or an argument that is neither a type nor a constant (an interface implemented by values only: ODR-182)");
    E2140 = (Error, 2140, Types, "[BRW-19], [EXP-5]", "a change to something that is not a place, which would be lost: an assignment to a value, or a method whose only effect is changing a temporary that ends with the line");
    E2150 = (Error, 2150, Types, "[EXP-9]", "`is` on operands that are neither class objects, owners (`shares`, `shares mut`, `thread_shared`), links, references nor `Option`s, and not a kind test (`x is T`)");
    E2151 = (Error, 2151, Types, "[TYP-30]", "integer `**` with a constant negative exponent");
    E2160 = (Error, 2160, Types, "[CTL-7]", "control flow cannot leave a `defer` block (`return`, `break`, `continue`, `yield`)");
    E2170 = (Error, 2170, Types, "[LAY-2]", "reference to a field of a packed struct");
    E2180 = (Error, 2180, Types, "[ERR-2]", "`?` outside a function returning `Option` or `Result`");
    E2181 = (Error, 2181, Types, "[ERR-2]", "`?` on an `Option` in a `Result` function, or the reverse");
    // 0.9.9 `[FN-10]`, ODR-023: only `void` and `Result[void, E]` have an
    // implicit value at the end of a body.
    E2182 = (Error, 2182, Types, "[FN-10]", "a function that returns a value can reach the end of its body");
    // 0.9.10_Hardened_5 `[FN-8]`, ODR-101: the forms `main` may take.
    E2183 = (Error, 2183, Types, "[FN-8]", "`main` has none of the forms `[FN-8]` allows");
    E2200 = (Error, 2200, Types, "[TYP-14]", "type has infinite size: a type that contains itself, or holds a view of its own type, with no block of its own between (an `owns` field, a list, a `shares` block)");
    // 0.9.9 `[TYP-28]`: `/` is true division; two integer operands are
    // rejected with the `//` and float fix-its.
    E2240 = (Error, 2240, Types, "[TYP-28]", "`/` on two integers");
    E2250 = (Error, 2250, Types, "[LEX-19]", "format spec does not apply to the value's type");
    E2260 = (Error, 2260, Types, "[TYP-32]", "`some` type outside a return position");
    E2261 = (Error, 2261, Types, "[TYP-32]", "returns of a `some` function have different types");

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
    E2221 = (Error, 2221, Types, "[CORO-6]", "a view into a generator's own frame held across `yield`");
    E2222 = (Error, 2222, Types, "[CORO-10]", "coroutine where an ordinary function is required");
    E2223 = (Error, 2223, Types, "[MONO-7]", "`@never_specialize` on a generic that is not shareable");
    E2224 = (Error, 2224, Types, "[HR-16]", "`migrate_from` may not take a `mut` receiver");
    E2225 = (Error, 2225, Types, "[HR-35]", "`migrate_from`, what it calls, the `drop`s it may run and a field default prepare runs may not panic; a call through a function value, an interface, `dyn` or `virtual` method in them, and a `noexcept` foreign call");
    E2226 = (Error, 2226, Types, "[STD-8]", "`in` on a type that does not implement `Contains`");
    E2227 = (Error, 2227, Types, "[HR-39]", "a `noexcept` foreign call in `migrate_from` may terminate the process");
    E2229 = (Error, 2229, Types, "[CORO-12]", "class generator method holds an access to its object across `yield`");
    E2231 = (Error, 2231, Types, "[CORO-13]", "`yield` while a `@must_drop` value is live");

    // --- ownership, borrows, regions, exclusivity, drops --------------------
    E3010 = (Error, 3010, Ownership, "[EXP-6]", "cannot hand over a field of a type with `drop` on its own");
    E3011 = (Error, 3011, Ownership, "[EXP-6]", "cannot hand over an array or span element on its own");
    E3012 = (Error, 3012, Ownership, "[EXP-6]", "retired: folded into `E3013`: a field of a class object is taken with `mem.take` or `mem.replace`");
    E3013 = (Error, 3013, Ownership, "[OWN-11], [EXP-6], [CLO-6], [CLO-6a], [GRM-13]", "a hand-over out of a name or place that does not own the value: a plain parameter's value, a function value's own capture where it may run twice, a plain `once fn()` parameter, or a field of a class object, or a part of a value that has several owners (`shares mut`), which other names may reach; a `..rest` name under `match owns xs:`, which would view what the match hands over; the value inside an `owns T` place handed out on its own (`m: Mesh = b` for `b: owns Mesh`); a `once fn` called out of a class field or through a link; a hand-over inside `debug_assert`'s argument, which a `release` build never evaluates");
    E3020 = (Error, 3020, Ownership, "[BRW-18], [CTL-2], [MOD-9a], [EFF-2a]", "a collection (a list, a map, a set or a string) changed while it is walked, whatever the change (an element written, a value under a key replaced, anything added or removed), by a `for` loop or by a generator that walks it and will still be used; through its own name, or through a call that may change it (a call through a function value or a library call by the functions it may run, a `spawn` by what its thread changes, a scoped task started in the walk, a foreign call by what the callbacks C keeps change, the end of an object by what its `drop` changes); a removal inside a counting loop; an override, implementation or stored function that changes what a recorded library line walks (`[MOD-9a]`); a call through a place another package can fill, counted as changing what a `pub` route reaches, while a loop walks it; an implementation, override or stored function that breaks a fact a library's recorded call relied on (`[BLD-14]` item 16)");
    // `[DIA-7a]` keys every E3xxx code to a diagnostic shape and forbids
    // emitting one that is absent from its table — but shapes B1, B3, B4, B5,
    // B8, B9 and B10 are given no code there, which leaves the most basic
    // borrow error in the language unreportable. Allocated here beside E3020,
    // which is the aliasing error the table does key. See errata ERR-021.
    E3021 = (Error, 3021, Ownership, "[BRW-1], [BRW-11], [LT-1], [LT-48], [ALC-5a]", "a read and a change of one place overlap: a change of what a read-only name reads while it is still used, a view a `dyn` call may give included where the kind is known only while running (help: `runtime` on the parameter or the typed local the call goes through, or a `runtime` name for the value); a `ref mut` name's place read through another name while it is still used; an argument that may reach into a changeable argument of the same call; a reset or end of a manager's instance while a `weakref` link to a value made in it lives");
    E3022 = (Error, 3022, Ownership, "[BRW-1], [BRW-11], [CLO-17], [MOD-9a], [MONO-10], [JOB-5]", "one place changed through two names: two changeable arguments of one call naming one place, other than one whole variable proved when compiling (a function value that changes it counts as one); a function value stored in a place that anything it changes through any name owns (a `static mut` included, `[CLO-17]`), or whose storing changes what a recorded library line holds or has lent (`[MOD-9a]`); a `ref mut` name's place changed through another name while it is still used, a reshape of a `ref mut` address included; a call through a function value, an interface, `dyn` or `virtual` method, or of an `extern` function, that names one whole variable twice; a second name for a job's arena while the first is still used");
    E3023 = (Error, 3023, Ownership, "[BRW-14], [BRW-15], [STA-1], [TYP-14], [ALC-1], [FN-2a], [PHIL-13]", "a change the name does not allow: a change through a name that only reads (a plain parameter; a loop, pattern, opened or part name; a plain name holding a `MutSpan`; a value made when compiling); `=` on a name that keeps its place for its scope (a parameter; a loop, pattern or opened name, one opened `exists as copy of` excepted; a `const ref mut` name given another ref); a read-only `ref` given to a `ref mut` name; a method that may change its object, or a stored function that may change its own state, called through a read-only name; an owner that may change made from one that only reads (a plain parameter of a `shares mut` type included); `=` on a `static` (`HOOK = v`); `=` of a whole value on a `ref mut` name of a `dyn I` type; `using x` through a plain parameter; a function that changes its own state given without `ref mut` to an `F`-bounded parameter; a call in the argument of `debug_assert` of a function that changes or ends anything that outlives the argument");
    E3024 = (Error, 3024, Ownership, "[TYP-15]", "a struct field would point into another field of the same struct (shape B5)");
    E3025 = (Error, 3025, Ownership, "[BRW-10]", "retired: what every method changes and reads is worked out from its body, so no method claims all of its object");
    E3026 = (Error, 3026, Ownership, "[CLO-4]", "a function value that may outlive a place it uses and does not own: a returned function that changes its maker's local without `owns`; a task or thread given a loop's name with no word, or with `ref` where it may outlive the element");
    E3027 = (Error, 3027, Ownership, "[FN-2a], [FN-5]", "`ref mut` before something this function may not change: an argument, a loop or pattern name over something it only reads, or a place reached through a read-only name; a default on a `ref mut` parameter; a `ref mut` argument inside `debug_assert`'s argument, which a `release` build never evaluates");
    E3028 = (Error, 3028, Ownership, "[FN-2a]", "a call's word does not match its parameter: no `ref mut` (or no `const` for a `const ref mut` parameter) before an argument for a `ref mut` parameter, a `MutSpan` one included, or `ref mut` before one for a parameter that is not `ref mut`; no `owns` before an argument for an `owns` parameter, or `owns` before one for a parameter that is not `owns` (shape B10)");
    E3030 = (Error, 3030, Ownership, "[CLO-2]", "retired: folded into `E3013`: a function value's own capture given away where it may run twice");
    E3031 = (Error, 3031, Ownership, "[BRW-21], [FFI-52]", "a call whose argument the callee's body reaches by another name in a way its parameter cannot allow: a part of what the callee may end or move before its last use of that parameter (a plain, `Span` or `ref mut` parameter, or a method's object); for a `ref mut` parameter, a part of what the callee reads or changes by another name while the parameter is used; at the line that stores a function into a function value, when that function does so with what the value's calls give it (shape B16); a call from another package whose argument a callback, export or override of that package changes across the callee's open line");
    E3032 = (Error, 3032, Ownership, "[WK-6], [WK-5], [WK-7], [MOD-9a]", "a loop of owners (`shares`, `shares mut` or `thread_shared` owners, a function value's state, channel ends) that the compiler cannot prove is opened again before its objects lose their last owner from outside (shape B17), a loop closed through a function another package adds included, at the call that gives its kind or at the line that adds it");
    E3040 = (Error, 3040, Ownership, "[OWN-3]", "a name used after it was handed over or ended (a number handed over with `owns` included); a part handed over on its own, or its owner used whole, before the part is given a new value");
    E3041 = (Error, 3041, Ownership, "[OWN-4]", "retired: a hand-over in a loop's later round is `E3040`");
    E3014 = (Error, 3014, Ownership, "[THR-5]", "scope binding may not be handed over");
    E3015 = (Error, 3015, Ownership, "[THR-6]", "value whose drop is required may not be leaked");
    E3016 = (Error, 3016, Ownership, "[OBJ-5], [CLS-7a]", "`this` escapes its own `drop`, or an owner of it is made there");
    E3042 = (Error, 3042, Ownership, "[EXP-6]", "retired: folded into `E3040`: a part handed over on its own, then its owner used whole");
    E3063 = (Error, 3063, Ownership, "[LT-45], [BRW-19], [ALC-1], [JOB-5], [ALC-5a], [ECS-8], [HR-16], [LT-4], [TYP-46a], [FFI-8]", "a view, or a value holding views, kept or returned where it would outlive what it points into: a view of a temporary, a view out of a lock's `with` block, a view of the callee's own locals put into the caller's view through `ref mut`, a kept view outliving its target; and a value made in a memory manager's instance, or one holding such a value, kept or returned where it would outlive that instance, a `thread_shared` value made in a local, a parameter, a field or a `static mut` instance handed to an unscoped thread, a job outside a scope or a callback foreign code keeps included; a `@ffi(handle)` value read out of an `owns T` kept or returned where it would outlive that owner, whose end destroys the object it names; a value made in a job's arena (`jobs.local_arena()`) that would leave the job's function; a `weakref` link, or a value holding one, kept where it would outlive the instance its target was made in; a listener given to `connect` that views a local the registry outlives; a view of an `Old` value stored into a migrated instance");
    E3064 = (Error, 3064, Ownership, "[LT-2]", "two independent regions in one view struct");
    E3065 = (Error, 3065, Ownership, "[LT-35]", "retired: a result points into every view argument it may come from, or into what `from` names; nothing is left to infer");
    E3095 = (Error, 3095, Ownership, "[DSJ-4]", "two single-object references given to `assert_disjoint`");
    E3096 = (Error, 3096, Ownership, "[ARN-6], [ARN-1]", "the parent arena, or a view into it made before the scope, used while a scope of it is live");
    E3050 = (Error, 3050, Ownership, "[BRW-7]", "a view of an uninitialised place");
    E3060 = (Error, 3060, Ownership, "[BRW-17], [LT-3], [MOD-9a]", "a view used after what it points into was ended, handed over, replaced or reshaped: a position after a removal or reorder; an address after a reshape; a view put in through `ref mut` after one of the call's arguments changed; a name opened from a link after a line or call that may end its object; a view of what a function value's or an `owns dyn I` value's state owns after that value is replaced or ended; a stored function, override or implementation that may end what a name opened from a link reads across a recorded library line");
    E3061 = (Error, 3061, Ownership, "[LT-4]", "retired: folded into `E3063`: a value made in a memory manager's instance kept where it would outlive it");
    E3062 = (Error, 3062, Ownership, "[LT-1a]", "a result that points into something its first line's `from` does not name (shape B6)");
    E3070 = (Error, 3070, Ownership, "[DRP-1]", "`drop` cannot be called explicitly");
    E3080 = (Error, 3080, Ownership, "[EXC-3]", "retired: no check of exclusive access runs while a program runs; overlapping uses are `E3021` and `E3022`");
    E3090 = (Error, 3090, Ownership, "[ARN-3], [ARN-2]", "arena allocation of a type that needs `drop`");
    E3100 = (Error, 3100, Ownership, "[UNS-1], [UNS-12], [ALC-2], [ARN-8], [FFI-51], [FFI-26], [FFI-54], [FFI-56], [UNS-5], [FFI-40], [FFI-8], [FFI-17a], [FFI-41], [TYP-43], [THR-8], [FFI-39], [FFI-31b]", "an unsafe operation outside unsafe code (in a `safe:` block included), `into_raw()` included, a raw pointer returned from a function foreign code may call or stored where foreign code reads it, and a raw-pointer or C function-pointer field of an imported aggregate written; `Send` declared by hand without `unsafe extend`, other than in the header of an `open` or `abstract` class, where it is a checked promise; a call of `transmute`, `copy` or `copy_nonoverlapping` outside unsafe code; a call of a C++ method whose reference result has no lifetime word outside unsafe code; a call of a `std::function` value whose type's entry does not say `safe` outside unsafe code; a read of a foreign `const` object whose type has a `mutable` member or is not `ThreadShared` outside unsafe code; an `import c` or `import cpp` of a header that does more than declare (code the loader or the C runtime runs, a definition the linker sees, a named section, assembly, an order to the linker) or declares a name a library of `[link] unsafe_libs` defines, without `unsafe`; `ThreadShared` declared for an `extern type` without `unsafe`");
    // `[UNS-10b]` forbids `UnsafeCell` in `@static_safe` code. The type checker
    // emits this for direct, nested, and body-local uses of the primitive.
    E3105 = (Error, 3105, Ownership, "[UNS-10]", "`UnsafeCell` in `@no_alias_checks` code");

    // --- effects and contracts ----------------------------------------------
    E4001 = (Error, 4001, Effects, "[EFF-5], [EFF-2a], [CLO-10], [EFF-6]", "`@noalloc` function reaches an allocation; a call through a place another package can fill whose declared contracts do not exclude `Alloc`; a function value whose state its body writes, made under `@noalloc`; a call through a `dyn` or `virtual` method of a `pub` interface or class that does not declare the contract");
    E4002 = (Error, 4002, Effects, "[EFF-5]", "`@nosync` function reaches a synchronising operation");
    E4003 = (Error, 4003, Effects, "[EFF-5]", "`@noblock` function reaches a blocking operation");
    E4030 = (Error, 4030, Effects, "[DSJ-6], [EFF-12], [LT-48]", "`assert_disjoint`, `check_disjoint` or a `runtime` name, the alias checks left to run time, inside `@no_alias_checks`");
    E4040 = (Error, 4040, Effects, "[EFF-17], [ALC-6], [FFI-53]", "`@nopanic(explicit)` function reaches a panic; a making line `using x` whose `allocate` may return null; a call registering a callback C keeps (the table may be full on a 32-bit target)");
    E4010 = (Error, 4010, Effects, "[EFF-2]", "implementation does not satisfy the interface's contract");
    E4020 = (Error, 4020, Effects, "[SIMD-2]", "`@simd(assert)` loop did not vectorise");

    // --- I/O, locking and determinism (0.6, 0.6.3) ----------------------------
    E4041 = (Error, 4041, Effects, "[EFF-20]", "`@noio` function reaches I/O");
    E4042 = (Error, 4042, Effects, "[EFF-21]", "`@nolock` function acquires a lock");
    E4070 = (Error, 4070, Effects, "[DET-1], [DET-2], [DET-3]", "`@deterministic` function reaches a nondeterministic operation; a read whose value depends on which thread acted first (a receive on a channel several threads send to, `try_lock` and `try_recv`, a lock's view or an `Atomic` read of a place another thread or an unjoined task may change); a platform floating-point function, or `**` on a float base, whose result IEEE 754 does not fix, where `std.math.det`'s is not used, and a SIMD `_approx` operation; a call through a `dyn` or `virtual` method of a `pub` interface or class that does not declare `@deterministic`");
    // Registered and never emitted, so the number is not reused: the case it
    // named is reported as `E4070` with the foreign frame named (XX §6).
    E4071 = (Error, 4071, Effects, "[DET-8]", "reserved: `@deterministic` reaching an undeclared `extern`");
    E4072 = (Error, 4072, Effects, "[DET-5]", "`@fastmath` or `@fp(allow_fma)` inside `@deterministic`");
    // Registered and never emitted: `[CORO-5]` makes it unreachable in v1.
    E4073 = (Error, 4073, Effects, "[CORO-5]", "reserved: a coroutine frame that allocates in `@noalloc`");

    // --- FFI -----------------------------------------------------------------
    E5001 = (Error, 5001, Ffi, "[FFI-5]", "imported type layout does not match");
    E5010 = (Error, 5010, Ffi, "[FFI-12], [FFI-55], [GRM-35]", "overlay does not match the C declaration; a declaration an overlay describes differs from the one `ember.overlays` records, or the package has no record of the overlay; an entry outside a class section that names only members of classes; an entry, `rename` or `hide` whose words fit none or several members of a C++ name declared several times; an overlay applied by an import of a header other than the one its first line names");
    E5020 = (Error, 5020, Ffi, "[FFI-15]", "`StrView` is not NUL-terminated");
    E5030 = (Error, 5030, Ffi, "[FFI-17a]", "`std::function` as a parameter");
    E5040 = (Error, 5040, Ffi, "[FFI-21]", "capturing closure passed where a C function pointer is expected");
    E5041 = (Error, 5041, Ffi, "[FFI-21]", "a `retained` or `once` callback needs a release function (`destroys(f)`) in the overlay");
    E5042 = (Error, 5042, Ffi, "[FFI-21a]", "a function C may call returns a pointer into a parameter C passes as a copy");
    E5002 = (Error, 5002, Ffi, "[FFI-2], [FFI-10], [FFI-56], [FFI-17a], [FFI-36a], [FFI-36], [FFI-21], [FFI-8]", "a foreign declaration whose contract cannot stand: a hand declaration that says neither `safe fn` nor `unsafe fn`, or both; `safe fn` on a function whose contract is incomplete; an assertion of the safe-to-call fact (`safe fn`, an overlay function entry not written `unsafe fn`, `@ffi(safe)` on a function-pointer typedef, `@ffi(std_function, safe)` on a `std::function` type) for a function whose calls the other rules make safe and would carry foreign code a raw pointer or C function pointer the caller supplies, as an argument, held at any depth by what an argument passes, or returned by an Ember function given to the call (`[FFI-2]`'s limit); two destructors for one foreign type (a `std::unique_ptr`'s deleter among them), a `returns_owned` result giving an `owns T` of a type that is not foreign, or `adopt` of a C++ class with no public destructor (a call of a function without a verified contract outside `unsafe` is `E3100`); a function-pointer parameter of a `safe fn` with no `callback(…)` word, or a `callback(…)` parameter written as the C pointer type or without `owns` under `retained` or `once`; a `user_data` witness too narrow for the tagged number; a pointer word on a handle parameter");
    E5011 = (Error, 5011, Ffi, "[FFI-30]", "one C identity imported with two different layouts");
    E5012 = (Error, 5012, Ffi, "[FFI-11]", "pointer contract has no count");
    E5013 = (Error, 5013, Ffi, "[FFI-8]", "a further argument of a variadic C function that C would promote to another type");
    E5014 = (Error, 5014, Ffi, "[FFI-29]", "two packages request the same implementation macro");
    E5015 = (Error, 5015, Ffi, "[FFI-31], [FFI-31b], [FFI-56]", "type may not cross the boundary; an exhaustive `@repr` enum in an export; a safe `extern \"C\" fn` C hands an export, as a parameter or in a `@layout(c)` value a parameter passes (write `unsafe extern \"C\" fn`)");
    E5016 = (Error, 5016, Ffi, "[FFI-8]", "`_Atomic` layout does not match `Atomic[T]`");
    E5017 = (Error, 5017, Ffi, "[FFI-8]", "struct with a flexible array member is unsized");
    E5018 = (Error, 5018, Ffi, "[FFI-8]", "`va_list` may not be constructed");
    E5031 = (Error, 5031, Ffi, "[FFI-32]", "C++ parameter cannot be mapped");
    E5032 = (Error, 5032, Ffi, "[FFI-32]", "opaque C++ type may not be constructed");
    W5002 = (Warning, 5002, Ffi, "[FFI-20a]", "declaration not imported (unsupported convention or construct)");
    E5090 = (Error, 5090, Ffi, "[UNS-6]", "inline assembly is not supported by the C backend");

    // --- grades, adoption, C++ boundary, text (0.6, 0.7.x) --------------------
    E5034 = (Error, 5034, Ffi, "[FFI-48]", "unsupported C++ construct");
    E5035 = (Error, 5035, Ffi, "[FFI-57]", "a use of a C++ name of several members (a call, `T.new(…)`, `super.init(…)`, a function value) that fits no member, is ambiguous in C++, is reached only through a conversion Ember does not make, or chooses one declared `= delete`");
    E5050 = (Error, 5050, Ffi, "[TCB-1]", "a foreign fact claims a grade whose evidence is absent or stale");
    E5051 = (Error, 5051, Ffi, "[FFI-35a]", "a foreign callee retains a pointer its contract does not declare `retained`");
    E5052 = (Error, 5052, Ffi, "[FFI-36b]", "`adopt` on a handle whose overlay declares `adopt = false`");
    E5053 = (Error, 5053, Ffi, "[FFI-37]", "an instrumented run contradicted a declared foreign effect");
    E5054 = (Error, 5054, Ffi, "[RNG-10], [FFI-31], [FFI-31b]", "range type, or `char`, in a foreign signature");
    E5055 = (Error, 5055, Ffi, "[FFI-17b]", "an Ember generic may not instantiate a C++ template");
    E5056 = (Error, 5056, Ffi, "[FFI-39]", "an `override fn` matching no virtual method of the C++ base by name and first line exactly, an upcast or a widening counting for nothing");
    E5057 = (Error, 5057, Ffi, "[FFI-39]", "the overlay names a method that is not virtual in the header");
    E5058 = (Error, 5058, Ffi, "[FFI-39b]", "C++ base has no default constructor and no declared `init`");
    E5059 = (Error, 5059, Ffi, "[FFI-17d]", "C++ trampoline base has no virtual destructor");
    E5060 = (Error, 5060, Ffi, "[FFI-39c]", "an object owned by Ember (`owner=\"ember\"`) given to a C++ parameter that keeps it");
    E5061 = (Error, 5061, Ffi, "[FFI-24]", "imported C++ function has no exception policy and none can be derived");
    E5062 = (Error, 5062, Ffi, "[FFI-43]", "contradictory C++ exception policies");
    E5063 = (Error, 5063, Ffi, "[TXT-2], [FFI-31b]", "foreign bytes reach `StrView` without validation; a `StrView` in an exported signature");
    E5064 = (Error, 5064, Ffi, "[TXT-3], [FFI-15]", "interior NUL: a `c\"...\"` literal containing a NUL, or a value converted to a `CStrView`");
    E5065 = (Error, 5065, Ffi, "[WK-14]", "`shares`/`weakref` and `CppShared`/`CppWeak` do not interconvert");
    E5066 = (Error, 5066, Ffi, "[FFI-43]", "an overlay marks a C++ function both `noexcept` and throwing");
    E5067 = (Error, 5067, Ffi, "[GRM-35], [TIER-1]", "an overlay that states facts is not declared `unsafe overlay`");
    E5068 = (Error, 5068, Ffi, "[FFI-36b], [FFI-54]", "one live foreign object adopted twice; one address taken back twice with `from_raw`");
    E5069 = (Error, 5069, Ffi, "[FFI-52], [FFI-11]", "`@ffi(no_callbacks)` on a foreign function whose own contract takes a callback (`callback(...)`); `@ffi(no_callbacks)` on a function whose contract takes a bare function pointer");

    // --- comptime -------------------------------------------------------------
    E6001 = (Error, 6001, Comptime, "[CT-3]", "comptime evaluation exceeded its limits, or a constant's value depends on itself");
    E6002 = (Error, 6002, Comptime, "[CT-4]", "comptime evaluation is not deterministic");
    E6004 = (Error, 6004, Comptime, "[CT-7]", "panic during compile-time evaluation");
    E6005 = (Error, 6005, Comptime, "[CT-6], [CT-2]", "`comptime(e)` refers to a run-time local; `comptime.read_file` or `comptime.env` given a run-time argument");
    E6010 = (Error, 6010, Comptime, "[CT-2], [CLI-21], [CT-5], [CT-1a]", "operation not available at compile time, among them a compile-time value a plain name could change (an `Atomic`, lock, `Once`, channel end, `guarded by` field or `@cached` class in a `comptime(e)`), and a `static mut`, or a `static` built on first access, reached by compile-time code; in `unsafe` code at compile time, a read or write through a pointer not at the bytes of a live value the evaluator holds; inline assembly; `comptime.read_file` of a file missing, unreadable or not UTF-8");

    // --- concurrency ----------------------------------------------------------
    E7001 = (Error, 7001, Concurrency, "[THR-1]", "`@sync` class with a field that is not `Sync`, or a non-`@sync` base or derived class");
    E7002 = (Error, 7002, Concurrency, "[STA-1], [THR-20], [THR-9], [CLS-12]", "`static` of a type that is not `ThreadShared`; a `guarded by` `static mut` of a type that is not `Send`; a `static` `Receiver`; a `static` of a class with a `@cached` method that declares nothing saying its objects are shared (its kept result is filled with plain stores)");
    E7003 = (Error, 7003, Concurrency, "[THR-1]", "write to a field of a `@sync` class after `init`, or a method on one that changes its object");
    E7004 = (Error, 7004, Concurrency, "[THR-17], [THR-10], [FFI-21]", "an unscoped thread or a kept callback uses a name with no capture word (a loop's name included)");
    E7005 = (Error, 7005, Concurrency, "[THR-11]", "value shared with a task is not `Sync`");
    E7006 = (Error, 7006, Concurrency, "[THR-14]", "memory order an atomic operation does not support");
    E7007 = (Error, 7007, Concurrency, "[THR-11], [THR-21], [TYP-46a]", "two tasks of one scope change one place other than by a reduction, through the lock that guards the place, through an `Atomic` or through a `Sender` (a task that reads the place before changing it, and a plain assignment, included); two tasks growing values made in one manager instance; two tasks, or a task and the parent, consuming one `Receiver`; a place two tasks reduce with different operators, or with `+=` and `-=` on an unsigned place");
    E7008 = (Error, 7008, Concurrency, "[THR-8], [THR-3], [FFI-23], [FFI-17d], [FFI-8]", "a thread-bound value (a guard, a `Scope`, a raw pointer, a `threads(creator)` handle, a `Retained[ForeignCallback]` token, an object of a trampoline class under `threads=creator`) sent to another thread; `ThreadShared` declared on a `@ffi(handle)` type or `extern type` whose contract says `threads(creator)` or `threads(main)`");
    E7011 = (Error, 7011, Concurrency, "[PAR-2a]", "parallel loop has a loop-carried dependency");
    E7012 = (Error, 7012, Concurrency, "[THR-20], [STA-1], [CLO-16]", "a `static mut` not `guarded by` a lock reached from another thread; a function given to a `thread_shared fn()` type whose chain reaches an unguarded `static mut`");
    E7013 = (Error, 7013, Concurrency, "[THR-18]", "a `guarded by` field used without holding its lock");
    E7014 = (Error, 7014, Concurrency, "[THR-1], [TYP-45], [THR-21]", "a `thread_shared` value changed directly, `ref mut` after `exists as` on a `weakref thread_shared T` link included; a reduction into a field of a `thread_shared` value");
    E7015 = (Error, 7015, Concurrency, "[THR-1], [THR-2], [THR-10], [CLO-16], [FFI-22], [OWN-13], [THR-8], [THR-9], [THR-15], [TYP-46a], [THR-11], [FFI-21], [FFI-8], [FFI-17d], [CLS-4], [CLS-12], [MOD-9a]", "a one-thread owner (`shares`, `shares mut`, a `weakref T` link) or a lent value reaches another thread: held by a function any thread may run (`thread_shared fn()`, `thread.spawn`'s included), in a plain `fn()` field of a class whose object is made `thread_shared` (at that line); a `thread_shared` value whose type is not `ThreadShared` (a `Cell`, an `RwLock[Cell[int]]`, a `Receiver`, a plain `fn()` reachable from it), at the line that makes it (verdict 165), and a function given to a `thread_shared fn()` type (not a `once fn`) whose state is not `ThreadShared`; a channel of a `T` that is not `Send`, at the line that makes it (verdict 164); a `weakref T` link made to a value reached through a value every thread may reach (a `thread_shared` value, a `static`, a `guarded by` `static mut`, a `comptime(e)` result), at the line or at a call by the callee's summary; a class whose methods pin its objects both to one thread (`weakref this`) and `thread_shared` (`thread_shared this`), or one-thread while it or a base promises `Send`; a `thread_shared` object of a class pinned to one thread or with a `weakref T` field, or of a class with a `@cached` method that declares nothing saying its objects are shared (its kept result is filled with plain stores); `thread_shared T using A`, at the making line or at the type, where the value would hold a link to an instance of a manager that not every thread may call; a scoped task or job that may make or end a `shares` owner, make, end or open a `weakref T` link, or end an object of a class family that carries a weak count; a `shares` owner or `weakref` link in a `threads(main)` callback registered off the main thread; a view of an `extern type` or opaque class reaching another thread; a one-thread owner in a trampoline class under `threads=any`; a field or a method that is not `Send` or `ThreadShared` in a class whose header, or a base's, promises it (`open class Script implements Send:`), at the class; a value held by the type of an `open` or `abstract` class another package may derive from, whose header promises nothing, reaching another thread or made `thread_shared`; a `weakref T` link that a function another package adds makes of what a call gives it, where a `thread_shared` value reaches that, at the call that gives its kind or at the line that adds it");
    E7016 = (Error, 7016, Concurrency, "[PAR-1]", "a `@parallel` loop over something it cannot split into chunks: neither a range nor a list walked by element");
    E7017 = (Error, 7017, Concurrency, "[THR-1], [OWN-13], [RC-1]", "`thread_shared` on a name whose value was not created `thread_shared`; `weakref thread_shared x` where `x` was not; an object with one owner (`Node()`) of a class whose methods say its objects are `thread_shared`; `shares` or `shares mut` on a `thread_shared` owner");
    E7018 = (Error, 7018, Concurrency, "[THR-19], [MOD-9a], [EXP-4]", "a wait for a lock this thread holds, while this thread holds that lock (its `with` block, a guard in a local until its last use, or a guard made as a temporary for its life, `[EXP-4]`): a call through a function value, or a `dyn` or virtual method, that may run a function taking that lock (inside a walk under that lock too), or a join of a thread (a scope's end included) whose function changes what the lock guards; a first access of a `static` whose initialiser takes the lock; a second `read()` of an `RwLock` this thread holds `read()`; a stored function, override or implementation that takes a lock held at a recorded library line");
    E7010 = (Error, 7010, Concurrency, "[PAR-2], [PAR-2b], [PAR-3]", "a parallel loop's chunks may write one place; an undeclared total a loop body combines; a call the analysis cannot see into that receives a captured list, span or column");
    E7020 = (Error, 7020, Concurrency, "[ECS-4]", "systems in one parallel run have conflicting access sets");

    // --- layout ---------------------------------------------------------------
    E8001 = (Error, 8001, Layout, "[GPU-10]", "GPU layout does not match the CPU layout");

    // --- build, manifest, toolchain -------------------------------------------
    // `[TYP-9c]` and `[MAN-3]` both name `E9010`. Both are followed.
    E9010 = (Error, 9010, Build, "[MAN-3]", "`[lints]` names a lint the compiler does not define");
    E9011 = (Error, 9011, Build, "[CG-C-11]", "the toolchain cannot disable floating-point contraction");
    E9020 = (Error, 9020, Build, "[BLD-FFI-1a]", "translation units of one target disagree on an inherited flag");
    E9021 = (Error, 9021, Build, "[BLD-FFI-1b]", "C++ standard library and CRT heap could not be determined");
    E9001 = (Error, 9001, Build, "[MAN-1], [PRF-3], [MAN-10], [BLD-FFI-3], [FFI-51]", "invalid manifest, including an unknown key; a dependency's manifest section or flag honoured only when the root names it in `[build] native`, and a dependency's C that reads a file outside its own root and the toolchain's headers, or holds assembly, without it; a library the link would add, because a header or a named library asks for it, that `[link] unsafe_libs` does not name; a library named there of which no import and no `unsafe extern` block declares a name, and which nothing asks for");
    E9002 = (Error, 9002, Build, "[BLD-FFI-1]", "no C compiler found, or it failed; the help names the host's remedy");
    E9003 = (Error, 9003, Build, "[CLI-1]", "invalid command line");

    E9013 = (Error, 9013, Build, "[TCB-1]", "invalid `[ffi]` manifest section");
    E9014 = (Error, 9014, Build, "[RT-12], [DIA-6a]", "the C toolchain cannot emit stack probes for a frame larger than a guard page");
    E9030 = (Error, 9030, Build, "[HR-18], [HR-44], [HR-45], [HR-46], [HR-47], [HR-15c], [HR-48], [HR-44a], [HR-13b], [HR-7], [HR-24], [HR-3b], [HR-11]", "hot reload refused; a kept view into a part that is removed, converted or rebuilt; a live lambda's captures changed; a stored function the new image does not count; a schema change of an unenumerable type; a removed function's thunk still named; a GPU object in flight that must move; the promise of an `open` or `abstract` class's header changed without every package that imports it rebuilt (Annex B §B.3)");
    E9031 = (Error, 9031, Build, "[MAN-7]", "invalid `reload` value");
    // Registered and never emitted: `[HR-6]`'s permanent thunks make a
    // reloadable function's address stable, so it cannot be stranded.
    E9032 = (Error, 9032, Build, "[HR-6]", "reserved: a reloadable function's address escaping to foreign code");
    E9033 = (Error, 9033, Build, "[MAN-7]", "retired: a `release` build ignores `reload` with a note (ODR-118)");
    E9034 = (Error, 9034, Build, "[MONO-3]", "invalid `max_instantiations` value");
    E9040 = (Error, 9040, Build, "[MNG-1]", "two generic instances hash to one symbol, or two packages define one instance symbol for two spellings");
    E9035 = (Error, 9035, Build, "[HR-12a]", "packages in one process disagree about the object-header layout");
    E9036 = (Error, 9036, Build, "[HR-29]", "a reloadable package may not link the runtime statically");
    E9037 = (Error, 9037, Build, "[ABI-1]", "reload ABI mismatch on image load");
    E9041 = (Error, 9041, Build, "[TYP-9c]", "the toolchain cannot honour `@fastmath` or `@fp(…)` for a function");
    E9042 = (Error, 9042, Build, "[BLD-15]", "a link ran past its time limit");

    // --- warnings --------------------------------------------------------------
    W0001 = (Warning, 1, Lex, "[LEX-11]", "retired: a dangling doc comment is discarded in silence");
    W1002 = (Warning, 1002, Resolve, "[GRM-12]", "binding shadows an enum variant of the same name");
    W1003 = (Warning, 1003, Resolve, "[MOD-3]", "a package module shadows the standard module of the same name");
    W2091 = (Warning, 2091, Types, "[CTL-5]", "unreachable match arm");
    W2015 = (Warning, 2015, Types, "[LEX-17a]", "float literal has more digits than its type keeps");
    W2016 = (Warning, 2016, Types, "[PAN-1]", "`debug_assert` argument has an effect of its own that changes nothing (it prints, or takes a lock and lets it go)");
    W2111 = (Warning, 2111, Types, "[CLS-4]", "`virtual` has no effect in a final class");
    W2190 = (Warning, 2190, Types, "[ERR-5]", "unused `Result`, `check_disjoint`'s included");
    W2192 = (Warning, 2192, Types, "[VER-3]", "use of a deprecated item");
    W5001 = (Warning, 5001, Ffi, "[FFI-6]", "function-like macro ignored");
    W5031 = (Warning, 5031, Ffi, "[FFI-20a]", "C++ declaration skipped: the header could not be parsed");

    W2220 = (Warning, 2220, Types, "[MONO-3]", "instantiation ceiling exceeded");
    W3012 = (Warning, 3012, Ownership, "[UNS-8]", "unsafe block with no SAFETY note");
    W3013 = (Warning, 3013, Ownership, "[UNS-8]", "a `# SAFETY:` note on an `unsafe fn` line, where `@safety` says it");
    W5033 = (Warning, 5033, Ffi, "[FFI-40], [FFI-42]", "`&&`-qualified member skipped; a `T&&` parameter of a type with no public move constructor skipped; a free operator whose first operand is not a class of the header skipped");
    W5034 = (Warning, 5034, Ffi, "[FFI-41]", "anonymous-namespace entity skipped");
    W5050 = (Warning, 5050, Ffi, "[TCB-1]", "unbacked or stale grade");
    W5054 = (Warning, 5054, Ffi, "[FFI-37]", "unexercised foreign fact");
    W9030 = (Warning, 9030, Build, "[HR-18]", "reload requires a restart");

    // --- lints ------------------------------------------------------------------
    L1001 = (Lint, 1001, Resolve, "[LNT-1]", "unused binding");
    L1002 = (Lint, 1002, Resolve, "[LNT-2]", "assignment declares a new binding");
    L3011 = (Lint, 3011, Ownership, "[CELL-7]", "`RefCell` guard held across a call");
    L3013 = (Lint, 3013, Ownership, "[EXC-7]", "long-term access held across a call");
    L3014 = (Lint, 3014, Ownership, "[LT-1b]", "retired: a result points into every view argument it may come from (as `E3065`)");
    L2001 = (Lint, 2001, Types, "[LNT-6]", "unnecessary `copy of`");
    L2002 = (Lint, 2002, Types, "[LNT-6]", "large `Copy` value passed by value");
    L3001 = (Lint, 3001, Ownership, "[WK-1]", "potential reference cycle");
    L3002 = (Lint, 3002, Ownership, "[LNT-6]", "a view kept in use longer than necessary");
    L3010 = (Lint, 3010, Ownership, "[UNS-3]", "`unsafe` block larger than necessary, or inside unsafe code already");
    L4001 = (Lint, 4001, Effects, "[LNT-6]", "allocation in a hot loop");
    L4002 = (Lint, 4002, Effects, "[LNT-6]", "dynamic dispatch on a final type");
    L4003 = (Lint, 4003, Effects, "[LNT-6]", "large `Array[int]`/`Array[float]` in a hot loop whose values fit 32 bits");
    L5001 = (Lint, 5001, Ffi, "[LNT-6]", "`unsafe extern` declaration with no contract");
    L5002 = (Lint, 5002, Ffi, "[LNT-6]", "conversion at the FFI boundary copies");
    L7001 = (Lint, 7001, Concurrency, "[LNT-6]", "lock held across a call that may block");

    L2003 = (Lint, 2003, Types, "[RNG-3a]", "fallible construction where a total one exists");
    L2004 = (Lint, 2004, Types, "[LNT-4]", "`gen fn` with no `yield`");
    L2005 = (Lint, 2005, Types, "[LNT-5]", "`@noreload` function calls a reloadable one in a loop");
    L3015 = (Lint, 3015, Ownership, "[UNS-7]", "an `unsafe fn` with no `@safety`: one declared, a lambda written in unsafe code, or a function `import c` or `import cpp` writes that its overlay entry leaves unsafe to call");
    L3016 = (Lint, 3016, Ownership, "[UNS-7]", "`@safety` text still reads `TODO`");
    L3017 = (Lint, 3017, Ownership, "[WK-4]", "a loop of owners found leaked at exit (made through `unsafe` or foreign code)");
    L3020 = (Lint, 3020, Ownership, "[UNS-12]", "a `safe:` block outside unsafe code, which changes nothing");
    L3018 = (Lint, 3018, Ownership, "[UNS-9]", "`unsafe` block with no reason category");
}

/// Look a code up by its rendered form, e.g. `"E3040"`. Used by `ember explain`.
pub fn lookup(text: &str) -> Option<Code> {
    REGISTRY.iter().copied().find(|c| c.to_string().eq_ignore_ascii_case(text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

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

    /// `[DIA-6a]` — the code registry is exhaustive in both directions and each code has one row:
    /// every code the development target names (outside a range such as `E5000`–`E5499` and its
    /// retired codes) is registered here; every code registered here is a row of its registry
    /// table (§XVII.9), exactly once, with the same rule among the row's, or of its retired codes'
    /// (kept so the number is never reused); and every rule a registered code cites is defined in
    /// the document.
    #[test]
    fn the_registry_and_the_specification_name_the_same_codes() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let pin = std::fs::read_to_string(root.join("docs/spec-source/development-target.json")).expect("the pin");
        let path = pin.split("\"path\": \"").nth(1).and_then(|rest| rest.split('"').next()).expect("its path");
        let spec = std::fs::read_to_string(root.join(path)).expect("the development target");
        let table = spec.split("## XVII.9 The diagnostic code registry").nth(1).expect("XVII.9");
        let (registry_table, retired_table) = table.split_once("**Retired codes.**").expect("the retired codes");
        let retired_table = retired_table.split("\n\n").find(|block| block.trim_start().starts_with('|')).unwrap_or("");
        let code_at = |text: &str| -> Option<String> {
            let bytes = text.as_bytes();
            (bytes.len() >= 5 && matches!(bytes[0], b'E' | b'W' | b'L') && bytes[1..5].iter().all(u8::is_ascii_digit))
                .then(|| text[..5].to_string())
        };
        // Each code the text names, as a whole word; ranges (`E5000`–`E5499`) are not codes.
        let mut named = HashSet::new();
        for (at, _) in spec.match_indices(|c: char| matches!(c, 'E' | 'W' | 'L')) {
            let Some(code) = code_at(&spec[at..]) else { continue };
            let before = spec[..at].chars().next_back();
            let after = spec[at + 5..].chars().next();
            let in_range = spec[at + 5..].trim_start_matches(['`', ' ']).starts_with(['–', '-', '—'])
                || spec[..at].trim_end_matches(['`', ' ']).ends_with(['–', '—']);
            if !before.is_some_and(|c| c.is_alphanumeric()) && !after.is_some_and(|c| c.is_alphanumeric()) && !in_range {
                named.insert(code);
            }
        }
        let retired: HashSet<String> = retired_table
            .lines()
            .filter_map(|line| line.strip_prefix("| `").and_then(code_at))
            .collect();
        let mut rows: HashMap<String, Vec<&str>> = HashMap::new();
        for line in registry_table.lines() {
            if let Some(code) = line.strip_prefix("| `").and_then(code_at) {
                rows.entry(code).or_default().push(line);
            }
        }
        let registered: HashSet<String> = REGISTRY.iter().map(|code| code.to_string()).collect();
        let mut unregistered: Vec<&String> = named.iter().filter(|code| !registered.contains(*code) && !retired.contains(*code)).collect();
        unregistered.sort();
        assert!(unregistered.is_empty(), "the specification names codes the registry does not hold: {unregistered:?}");
        for code in REGISTRY {
            let shown = code.to_string();
            // A retired code stays registered, so it is never reused: its row is the retired table's.
            if retired.contains(&shown) {
                assert!(!rows.contains_key(&shown), "{shown} is both retired and a row of the registry");
                continue;
            }
            let lines = rows.get(&shown).map_or(&[][..], Vec::as_slice);
            assert_eq!(lines.len(), 1, "{shown} is a row of the specification's registry {} times", lines.len());
            assert!(code.rule.split(", ").all(|rule| lines[0].contains(rule)),
                "{shown} cites {} here and the specification's row says {}", code.rule, lines[0]);
            for rule in code.rule.split(", ") {
                let defined = format!("* `{rule}`");
                assert!(spec.contains(&defined), "{shown} cites {rule}, which the specification does not define");
            }
        }
    }

    /// `[DIA-6a]` — a registered code says what its row of the specification's table says: the same
    /// words and every rule of the row, the first being the one that defines the code (`[RC-1]` and
    /// `[WK-16]`'s rows among them).
    #[test]
    fn a_registered_code_says_what_its_row_says() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let pin = std::fs::read_to_string(root.join("docs/spec-source/development-target.json")).expect("the pin");
        let path = pin.split("\"path\": \"").nth(1).and_then(|rest| rest.split('"').next()).expect("its path");
        let spec = std::fs::read_to_string(root.join(path)).expect("the development target");
        let table = spec.split("## XVII.9 The diagnostic code registry").nth(1).expect("XVII.9");
        let (registry_table, _) = table.split_once("**Retired codes.**").expect("the retired codes");
        let mut disagree = Vec::new();
        for line in registry_table.lines() {
            let Some(rest) = line.strip_prefix("| `") else { continue };
            let Some((shown, rest)) = rest.split_once("` | ") else { continue };
            let Some(code) = REGISTRY.iter().find(|code| code.to_string() == shown) else { continue };
            let Some((words, rules)) = rest.trim_end().trim_end_matches('|').trim_end().rsplit_once(" | ") else { continue };
            if code.title != words || code.rule != rules.replace('`', "") {
                disagree.push(shown.to_string());
            }
        }
        assert!(disagree.is_empty(), "rows that differ from the specification's table: {disagree:?}");
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
        assert_eq!(W2192.to_string(), "W2192");
    }
}
