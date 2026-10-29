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
    E0005 = (Error, 5, Lex, "[LEX-14]", "reserved keyword used as an identifier");
    E0006 = (Error, 6, Lex, "[VER-8]", "a `#!` language directive names a version other than the current one");
    E0007 = (Error, 7, Lex, "[LT-6]", "named lifetimes are not supported in this version");
    E0008 = (Error, 8, Lex, "[LEX-22]", "unterminated character literal");
    // ODR-228 — what `E0101` reported besides an unclosed delimiter: a literal II.5's grammar
    // does not write (an escape, a character's value, a raw string's prefix, a byte string's
    // character, an f-string's lone `}`).
    E0009 = (Error, 9, Lex, "[LEX-26]", "malformed literal");

    // --- parser -------------------------------------------------------------
    E0100 = (Error, 100, Parse, "[GRM-2], [TYP-17]", "unexpected token, among them Python's `global`, a statement at module level outside the entry file (whose message gives the declaration to write: `static mut FRAME: int = 0`), a bound written without `needs` or joined with `+` (whose message gives the line with `needs`), and a clause after a function's brackets that Ember does not have (`changes FRAME`, `reads renderer`, `with` before the captures), whose message gives the first line to write");
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
    E0104 = (Error, 104, Parse, "[ATT-1], [PHIL-12]", "attribute or construct not admitted here, among them an attribute that is gone, with the one to write (`@sync`, `@static_safe`, `@no_runtime_checks`, `@fp(contract)`, `@borrows`, `@component`)");
    E0105 = (Error, 105, Parse, "[GRM-18]", "`;` is not a statement separator");
    E0106 = (Error, 106, Parse, "[GRM-17]", "a multi-statement closure cannot be written inside brackets");
    E0107 = (Error, 107, Parse, "[GRM-16]", "a jump expression may not be an operand");
    E0108 = (Error, 108, Parse, "[GRM-20]", "attribute is not permitted on this statement");
    E0109 = (Error, 109, Parse, "[OWN-9], [GRM-15]", "`owns` before a name whose place already hands the value over with no word (`e.mesh = m`; `b = a` for a value that is not a number; an argument; `return a`), or before `fn`");
    E0110 = (Error, 110, Parse, "[GRM-33]", "function declared without a body outside an interface, extern block or abstract class");
    E0111 = (Error, 111, Parse, "[GRM-36]", "`ref` of an expression that is not a place");
    E0112 = (Error, 112, Parse, "[GRM-39]", "nested more deeply than the compiler accepts");
    E0113 = (Error, 113, Parse, "[GRM-42], [FN-1]", "`mut` alone on a name: a pattern name, a parameter, a lambda parameter, `this` (`static mut` is a declaration and is not this)");
    E0114 = (Error, 114, Parse, "[GRM-43], [LT-46]", "`sharedref` not followed by `mut`");
    E0115 = (Error, 115, Parse, "[GRM-44]", "`Some(...)` in an `if` or `while` header");
    E0116 = (Error, 116, Parse, "[GRM-42]", "in a list of names declared without types, a name with no word after a name with a word");
    E0117 = (Error, 117, Parse, "[FN-1]", "`ref` or `ref mut` written in a parameter's type instead of on its name");
    E0118 = (Error, 118, Parse, "[GRM-41]", "a first line whose parts are out of order (parameters, `owns this`, `-> T`, `from`)");
    E0120 = (Error, 120, Parse, "[GRM-42], [CLO-2]", "`ref mut` in a lambda's capture list");
    // 0.9.9 `[CLI-19]`: a construct the specification defines and this
    // compiler does not implement yet is rejected by name, never silently.
    E0900 = (Error, 900, Support, "[PHIL-12], [CLI-19]", "construct not implemented by this compiler");
    // `[PHIL-12]`: where the specification is silent, or two of its rules
    // disagree, the construct is rejected and the gap raised with the owner.
    E0901 = (Error, 901, Support, "[PHIL-12]", "construct this specification leaves unspecified");

    // --- name resolution, modules, visibility -------------------------------
    E1010 = (Error, 1010, Resolve, "[MOD-3]", "a name or form the program may not use here: not in scope (`this` outside a method or in a `static fn`), gone from Ember (`self`, `Box`, `ForeignBox`, `RefCell`, `Weak`, `Shared`, the mode `owned`, `Sync` as a property, `cast x if D`, `cast x when D`, `upgrade`, `mem.forget`), or written with a word instead (`a.clone()` is `copy of a`; `.copied()` is `.copy_each()`)");
    E1020 = (Error, 1020, Resolve, "[GRM-4]", "name is already declared in this block");
    E1030 = (Error, 1030, Resolve, "[TYP-26]", "two items with the same name in one scope, such as a second `main` (a statement at module level is `E0100`)");
    E1031 = (Error, 1031, Resolve, "[MOD-8]", "a name bound by two glob imports is used");
    // ODR-201 — every item has one path (`[MNG-1]`).
    E1032 = (Error, 1032, Resolve, "[MOD-1]", "an item has the path of a module or the name of a built-in type");
    E1041 = (Error, 1041, Resolve, "[MOD-4]", "import cycle between packages");
    E1050 = (Error, 1050, Resolve, "[MOD-7]", "field is read-only outside its module");
    E1051 = (Error, 1051, Resolve, "[MOD-7]", "`read` visibility is valid on fields only");
    E1052 = (Error, 1052, Resolve, "[MOD-2]", "item, field or constructor not visible here");
    E1060 = (Error, 1060, Resolve, "[MOD-3]", "no module of that name");
    E1061 = (Error, 1061, Resolve, "[MOD-3]", "no item of that name in the module");

    E1021 = (Error, 1021, Resolve, "[BLD-11]", "name is not linked in this build");

    // --- types, inference, interfaces, generics, patterns -------------------
    E2010 = (Error, 2010, Types, "[LEX-16]", "literal does not fit its type");
    E2011 = (Error, 2011, Types, "[TYP-31], [LEX-24]", "negative literal index");
    E2020 = (Error, 2020, Types, "[TYP-4], [CLO-3]", "mismatched types, among them a lambda parameter whose word differs from the function type it is given to (a plain `fn(s)` where the type hands the value over, `fn(owns String)`); a `weakref` link used as its target without opening it; an object stored into a `weakref` place without `weakref`; a value given to a `using A` place that was not made in an instance of `A`");
    E2035 = (Error, 2035, Types, "[CTL-0]", "condition must be `bool`");
    E2036 = (Error, 2036, Types, "[GRM-19]", "this pattern always matches: an `if` or `while` opening whose left side always holds a value, or a kind test that always passes");
    E2037 = (Error, 2037, Types, "[TYP-44], [DSP-4]", "a kind test that can never pass: the type tested for does not implement the interface, or derive from the base class, of the value tested");
    E2030 = (Error, 2030, Types, "[TYP-34]", "`@view` on a type that is not a view");
    E2040 = (Error, 2040, Types, "[IFC-6], [TYP-17], [BLD-14]", "unsatisfied interface bound; for a `pub` generic whose bounds are not written, or a generic that writes some of its bounds and not all the body needs, one message on the first line of its body that needs a missing bound, giving the whole line of what it needs");
    E2041 = (Error, 2041, Types, "[TYP-19]", "overlapping `extend` implementations");
    E2042 = (Error, 2042, Types, "[STD-27]", "`Float` is implemented only by `f32` and `f64`");
    E2043 = (Error, 2043, Types, "[IFC-4]", "an associated type's default leads back to itself");
    E2050 = (Error, 2050, Types, "[IFC-5], [TYP-22]", "interface is not `dyn`-compatible");
    E2060 = (Error, 2060, Types, "[TYP-23]", "cannot infer type");
    E2061 = (Error, 2061, Types, "[TYP-23]", "lambda parameter types cannot be inferred here");
    E2062 = (Error, 2062, Types, "[TYP-23]", "ambiguous type");
    E2070 = (Error, 2070, Types, "[TYP-24]", "ambiguous interface method");
    E2071 = (Error, 2071, Types, "[SOA-1]", "`SoA[T]` of a type that is not a struct");
    E2072 = (Error, 2072, Types, "[STD-13]", "no method or field of that name (with the Ember name for a Python one; shape N13 for an undeclared field; in a generic body that is not `pub`, a method no bound's interface declares)");
    E2073 = (Error, 2073, Types, "[STD-26]", "`len` of a string (Python counts characters, `s.len()` bytes)");
    E2230 = (Error, 2230, Types, "[CTL-10]", "a name assigned in every branch at different types");
    E2228 = (Error, 2228, Types, "[CLO-3]", "callable parameter-mode mismatch (shape B15)");
    E2080 = (Error, 2080, Types, "[STR-3]", "`@derive(Copy)` on a type with a field that is not Copy");
    E2090 = (Error, 2090, Types, "[ENM-2]", "non-exhaustive match");
    E2100 = (Error, 2100, Types, "[CLS-2]", "field read before it is set, `init` calling a method whose body reads a field not yet set included");
    E2101 = (Error, 2101, Types, "[CLS-10]", "derived class with a field that has no default needs an `init`");
    E2102 = (Error, 2102, Types, "[CLS-11]", "derived `init` uses `this` or an inherited field before `super.init`, or leaves an own field unassigned");
    E2103 = (Error, 2103, Types, "[CLS-7]", "`this` given another object in a class method");
    E2104 = (Error, 2104, Types, "[CLS-9]", "a `let` field changed after `init`, for every type");
    E2106 = (Error, 2106, Types, "[GRM-44], [GRM-45], [DSP-4]", "an `if` or `while` opening, or a kind test, that names a name already in use");
    E2108 = (Error, 2108, Types, "[CLS-2]", "`owns this` on `init`");
    E2110 = (Error, 2110, Types, "[CLS-4]", "override of a method that is not virtual");
    E2111 = (Error, 2111, Types, "[CLS-4]", "a method that replaces an inherited virtual one without `override`");
    E2112 = (Error, 2112, Types, "[CLS-4]", "`virtual` in a class that cannot be subclassed");
    E2114 = (Error, 2114, Types, "[TYP-45], [TYP-46], [ALC-1]", "a type word that cannot apply to its type: `owns` on a class type (it always has its own block); `using` after a type with no heap memory of its own");
    E2115 = (Error, 2115, Types, "[GRM-41], [FN-4]", "`static fn` outside a type body");
    E2116 = (Error, 2116, Types, "[OWN-8]", "`copy of` before what it cannot copy as itself: a `weakref` link, a value of a view type (a struct holding a `ref mut` field included), a type with no `Clone`, a value whose views point into its own `let` field");
    E2117 = (Error, 2117, Types, "[CLS-12], [EFF-27]", "`@cached` on a method whose result it cannot keep safely: one with other parameters; one whose body changes anything, or reads a `shares`, `shares mut`, `thread_shared`, `weakref`, `sharedref mut` or `Cell` field, or a field other threads change under a lock; a `sharedref mut` link or `shares mut` owner made into what a `@cached` method reads, at the line that makes it");
    E2118 = (Error, 2118, Types, "[LT-45], [TYP-15]", "a `from` that is missing where a view needs one (a view field of a class), names what a view cannot point into for its life (a field that is not `let`, a fixed array or an inline struct field), or names two things (a parameter and a field of one name)");
    E2119 = (Error, 2119, Types, "[WK-16]", "a `weakref` of a part (a field, an element stored inline), whose target does not live on its own");
    E2120 = (Error, 2120, Types, "[IFC-2]", "inherent extension of a type from another package");
    E2121 = (Error, 2121, Types, "[TYP-20]", "an implementation outside the packages that may write it");
    E2122 = (Error, 2122, Types, "[RC-1], [OWN-13]", "`shares`, `shares mut` or `thread_shared` made of a place that has one owner and is not a local of this function (a list element, a field, a plain parameter)");
    E2123 = (Error, 2123, Types, "[OWN-10], [GRM-15], [TYP-46]", "a value made for a place that owns it alone (an `owns` field, an `Option[owns T]`, a list of `owns` values), at first or later, without `owns` on the line that makes it");
    E2130 = (Error, 2130, Types, "[TYP-1]", "a `const` of a type that owns heap memory; use a `static`");
    E2131 = (Error, 2131, Types, "[CT-1]", "array length must be a constant");
    E2031 = (Error, 2031, Types, "[LT-1a]", "`from` names a parameter the result cannot point into (a number, or an `owns` parameter that is not a view)");
    E2172 = (Error, 2172, Types, "[GRM-8b]", "cannot index with a type");
    E2173 = (Error, 2173, Types, "[TYP-45], [IFC-5], [OBJ-2]", "a type position holding what is not a type: a bare interface name used as a type (`Array[Shape]`), or an argument that is neither a type nor a constant (an interface implemented by values only: ODR-182)");
    E2140 = (Error, 2140, Types, "[BRW-19], [EXP-5]", "a change to something that is not a place, which would be lost: an assignment to a value, or a method whose only effect is changing a temporary that ends with the line");
    E2150 = (Error, 2150, Types, "[EXP-9]", "`is` on operands that are neither class objects, links, references nor `Option`s, and not a kind test (`x is T`)");
    E2151 = (Error, 2151, Types, "[TYP-30]", "integer `**` with a constant negative exponent");
    E2160 = (Error, 2160, Types, "[CTL-7]", "control flow cannot leave a `defer` block");
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
    E2225 = (Error, 2225, Types, "[HR-35]", "`migrate_from` may not panic");
    E2226 = (Error, 2226, Types, "[STD-8]", "`in` on a type that does not implement `Contains`");
    E2227 = (Error, 2227, Types, "[HR-39]", "a `noexcept` foreign call in `migrate_from` may terminate the process");
    E2229 = (Error, 2229, Types, "[CORO-12]", "class generator method takes `mut self` or holds an access across `yield`");
    E2231 = (Error, 2231, Types, "[CORO-13]", "`yield` while a `@must_drop` value is live");

    // --- ownership, borrows, regions, exclusivity, drops --------------------
    E3010 = (Error, 3010, Ownership, "[EXP-6]", "cannot hand over a field of a type with `drop` on its own");
    E3011 = (Error, 3011, Ownership, "[EXP-6]", "cannot hand over an array or span element on its own");
    E3012 = (Error, 3012, Ownership, "[EXP-6]", "cannot move out of a class field");
    E3013 = (Error, 3013, Ownership, "[OWN-11], [EXP-6], [CLO-6]", "a hand-over out of a name or place that does not own the value: a plain parameter's value, a function value's own capture where it may run twice, a plain `once fn()` parameter, or a field of a class object, which other names may reach");
    E3020 = (Error, 3020, Ownership, "[BRW-18], [CTL-2]", "a collection (a list, a map, a set or a string) changed while it is walked, whatever the change (an element written, a value under a key replaced, anything added or removed), by a `for` loop or by a generator that walks it and will still be used; through its own name, or through a call that may change it (a call through a function value or a library call by the functions it may run, a `spawn` by what its thread changes, a scoped task started in the walk, a foreign call by what the callbacks C keeps change, the end of an object by what its `drop` changes); a removal inside a counting loop");
    // `[DIA-7a]` keys every E3xxx code to a diagnostic shape and forbids
    // emitting one that is absent from its table — but shapes B1, B3, B4, B5,
    // B8, B9 and B10 are given no code there, which leaves the most basic
    // borrow error in the language unreportable. Allocated here beside E3020,
    // which is the aliasing error the table does key. See errata ERR-021.
    E3021 = (Error, 3021, Ownership, "[BRW-1], [BRW-11]", "a read and a change of one place overlap: a change of what a read-only name reads while it is still used; a `ref mut` name's place read through another name while it is still used; an argument that may reach into a changeable argument of the same call");
    E3022 = (Error, 3022, Ownership, "[BRW-1], [BRW-11]", "one place changed through two names: two changeable arguments of one call naming one place, other than one whole variable proved when compiling (a function value that changes it counts as one); a function value stored in a place that a variable it changes owns; a `ref mut` name's place changed through another name while it is still used, a reshape of a `ref mut` address included");
    E3023 = (Error, 3023, Ownership, "[BRW-14], [BRW-15], [STA-1]", "a change the name does not allow: a change through a name that only reads (a plain parameter; a loop, pattern, opened or part name; a plain name holding a `MutSpan`; a value made when compiling); `=` on a name that keeps its place for its scope (a parameter; a loop, pattern or opened name, one opened `exists as copy of` excepted; a `const ref mut` name given another ref); a read-only `ref` given to a `ref mut` name; a method that may change its object, or a stored function that may change its own state, called through a read-only name; an owner that may change made from one that only reads (a plain parameter of a `shares mut` type included); `=` on a `static` (`HOOK = v`)");
    E3024 = (Error, 3024, Ownership, "[TYP-15]", "a struct field would point into another field of the same struct (shape B5)");
    E3025 = (Error, 3025, Ownership, "[BRW-10]", "a method takes all of `self` (shape B8)");
    E3026 = (Error, 3026, Ownership, "[CLO-4]", "a function value that may outlive a place it uses and does not own: a returned function that changes its maker's local without `owns`; a task or thread given a loop's name with no word, or with `ref` where it may outlive the element");
    E3027 = (Error, 3027, Ownership, "[FN-2a]", "`ref mut` before something this function may not change: an argument, a loop or pattern name over something it only reads, or a place reached through a read-only name");
    E3028 = (Error, 3028, Ownership, "[FN-2a]", "a call's word does not match its parameter: no `ref mut` (or no `const` for a `const ref mut` parameter) before an argument for a `ref mut` parameter, a `MutSpan` one included, or `ref mut` before one for a parameter that is not `ref mut` (shape B10)");
    E3030 = (Error, 3030, Ownership, "[CLO-2]", "closure would move a captured value out");
    E3031 = (Error, 3031, Ownership, "[BRW-21], [FFI-52]", "a call whose argument the callee's body reaches by another name in a way its parameter cannot allow: a part of what the callee may end or move before its last use of that parameter (a plain, `Span` or `ref mut` parameter, or a method's object); for a `ref mut` parameter, a part of what the callee reads or changes by another name while the parameter is used; at the line that stores a function into a function value, when that function does so with what the value's calls give it (shape B16)");
    E3032 = (Error, 3032, Ownership, "[WK-6]", "a loop of owners (`shares`, `shares mut` or `thread_shared` owners, a function value's state, channel ends) that the compiler cannot prove is opened again before its objects lose their last owner from outside (shape B17)");
    E3040 = (Error, 3040, Ownership, "[OWN-3]", "a name used after it was handed over or ended (a number handed over with `owns` included); a part handed over on its own, or its owner used whole, before the part is given a new value");
    E3041 = (Error, 3041, Ownership, "[OWN-4]", "value moved in a previous loop iteration");
    E3014 = (Error, 3014, Ownership, "[THR-5]", "scope binding may not be handed over");
    E3015 = (Error, 3015, Ownership, "[THR-6]", "value whose drop is required may not be leaked");
    E3016 = (Error, 3016, Ownership, "[OBJ-5], [CLS-7a]", "`this` escapes its own `drop`, or an owner of it is made there");
    E3042 = (Error, 3042, Ownership, "[EXP-6]", "partial move then use of the whole value");
    E3063 = (Error, 3063, Ownership, "[LT-45], [BRW-19], [ALC-1]", "a view, or a value holding views, kept or returned where it would outlive what it points into: a view of a temporary, a view out of a lock's `with` block, a view of the callee's own locals put into the caller's view through `ref mut`, a kept view outliving its target; and a value made in a memory manager's instance, or one holding such a value, kept or returned where it would outlive that instance");
    E3064 = (Error, 3064, Ownership, "[LT-2]", "two independent regions in one view struct");
    E3065 = (Error, 3065, Ownership, "[LT-35]", "multi-region result provenance cannot be inferred");
    E3095 = (Error, 3095, Ownership, "[DSJ-4]", "two single-object references given to `assert_disjoint`");
    E3096 = (Error, 3096, Ownership, "[ARN-6]", "arena is scoped here");
    E3050 = (Error, 3050, Ownership, "[BRW-7]", "a view of an uninitialised place");
    E3060 = (Error, 3060, Ownership, "[BRW-17], [LT-3]", "a view used after what it points into was ended, handed over, replaced or reshaped: a position after a removal or reorder; an address after a reshape; a view put in through `ref mut` after one of the call's arguments changed; a name opened from a link after a line or call that may end its object; a view of what a function value's or an `owns dyn I` value's state owns after that value is replaced or ended");
    E3061 = (Error, 3061, Ownership, "[LT-4]", "arena allocation cannot outlive its arena");
    E3062 = (Error, 3062, Ownership, "[LT-1a]", "a result that points into something its first line's `from` does not name (shape B6)");
    E3070 = (Error, 3070, Ownership, "[DRP-1]", "`drop` cannot be called explicitly");
    E3080 = (Error, 3080, Ownership, "[EXC-3]", "overlapping access through the same handle");
    E3090 = (Error, 3090, Ownership, "[ARN-3]", "arena allocation of a type that needs `drop`");
    E3100 = (Error, 3100, Ownership, "[UNS-1], [UNS-12], [ALC-2], [ARN-8], [FFI-51], [FFI-26], [FFI-54], [FFI-56]", "an unsafe operation outside unsafe code (in a `safe:` block included), `into_raw()` included; `Send` declared by hand without `unsafe extend`");
    // `[UNS-10b]` forbids `UnsafeCell` in `@static_safe` code. The type checker
    // emits this for direct, nested, and body-local uses of the primitive.
    E3105 = (Error, 3105, Ownership, "[UNS-10]", "`UnsafeCell` in `@no_alias_checks` code");

    // --- effects and contracts ----------------------------------------------
    E4001 = (Error, 4001, Effects, "[EFF-5]", "`@noalloc` function reaches an allocation");
    E4002 = (Error, 4002, Effects, "[EFF-5]", "`@nosync` function reaches a synchronising operation");
    E4003 = (Error, 4003, Effects, "[EFF-5]", "`@noblock` function reaches a blocking operation");
    E4030 = (Error, 4030, Effects, "[DSJ-6], [EFF-12]", "`assert_disjoint`, the one alias check left to run time, inside `@no_alias_checks`");
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
    E4072 = (Error, 4072, Effects, "[DET-5]", "`@fastmath` or `@fp(allow_fma)` inside `@deterministic`");
    // Registered and never emitted: `[CORO-5]` makes it unreachable in v1.
    E4073 = (Error, 4073, Effects, "[CORO-5]", "reserved: a coroutine frame that allocates in `@noalloc`");

    // --- FFI -----------------------------------------------------------------
    E5001 = (Error, 5001, Ffi, "[FFI-5]", "imported type layout does not match");
    E5010 = (Error, 5010, Ffi, "[FFI-12], [FFI-55]", "overlay does not match the C declaration; a declaration an overlay describes differs from the one `ember.overlays` records, or the package has no record of the overlay");
    E5020 = (Error, 5020, Ffi, "[FFI-15]", "`StrView` is not NUL-terminated");
    E5030 = (Error, 5030, Ffi, "[FFI-17a]", "`std::function` as a parameter");
    E5040 = (Error, 5040, Ffi, "[FFI-21]", "capturing closure passed where a C function pointer is expected");
    E5041 = (Error, 5041, Ffi, "[FFI-21]", "retained callback needs a release function in the overlay");
    E5042 = (Error, 5042, Ffi, "[FFI-21a]", "a function C may call returns a pointer into a parameter C passes as a copy");
    E5002 = (Error, 5002, Ffi, "[FFI-2], [FFI-10], [FFI-36a]", "a foreign declaration whose contract cannot stand: `safe fn` on a function whose contract is incomplete, or two destructors for one foreign type (a call of a function without a verified contract outside `unsafe` is `E3100`)");
    E5011 = (Error, 5011, Ffi, "[FFI-30]", "one C identity imported with two different layouts");
    E5012 = (Error, 5012, Ffi, "[FFI-11]", "pointer contract has no count");
    E5013 = (Error, 5013, Ffi, "[FFI-8]", "a further argument of a variadic C function that C would promote to another type");
    E5014 = (Error, 5014, Ffi, "[FFI-29]", "two packages request the same implementation macro");
    E5015 = (Error, 5015, Ffi, "[FFI-31], [FFI-31b]", "type may not cross the boundary");
    E5016 = (Error, 5016, Ffi, "[FFI-8]", "`_Atomic` layout does not match `Atomic[T]`");
    E5017 = (Error, 5017, Ffi, "[FFI-8]", "struct with a flexible array member is unsized");
    E5018 = (Error, 5018, Ffi, "[FFI-8]", "`va_list` may not be constructed");
    E5031 = (Error, 5031, Ffi, "[FFI-17]", "C++ parameter cannot be mapped");
    E5032 = (Error, 5032, Ffi, "[FFI-32]", "opaque C++ type may not be constructed");
    W5002 = (Warning, 5002, Ffi, "[FFI-20a]", "declaration not imported (unsupported convention or construct)");
    E5090 = (Error, 5090, Ffi, "[UNS-6]", "inline assembly is not supported by the C backend");

    // --- grades, adoption, C++ boundary, text (0.6, 0.7.x) --------------------
    E5034 = (Error, 5034, Ffi, "[FFI-17]", "unsupported C++ construct");
    E5050 = (Error, 5050, Ffi, "[TCB-1]", "a foreign fact claims a grade whose evidence is absent or stale");
    E5051 = (Error, 5051, Ffi, "[FFI-35a]", "a foreign callee retains a pointer its contract does not declare `retained`");
    E5052 = (Error, 5052, Ffi, "[FFI-36b]", "`adopt` on a handle whose overlay declares `adopt = false`");
    E5053 = (Error, 5053, Ffi, "[FFI-37]", "an instrumented run contradicted a declared foreign effect");
    E5054 = (Error, 5054, Ffi, "[RNG-10], [FFI-31], [FFI-31b]", "range type, or `char`, in a foreign signature");
    E5055 = (Error, 5055, Ffi, "[FFI-17b]", "an Ember generic may not instantiate a C++ template");
    E5056 = (Error, 5056, Ffi, "[FFI-39]", "override of a C++ virtual the overlay does not name");
    E5057 = (Error, 5057, Ffi, "[FFI-39]", "the overlay names a method that is not virtual in the header");
    E5058 = (Error, 5058, Ffi, "[FFI-39]", "C++ base has no default constructor and no declared `init`");
    E5059 = (Error, 5059, Ffi, "[FFI-39]", "C++ trampoline base has no virtual destructor");
    E5060 = (Error, 5060, Ffi, "[FFI-39c]", "an object owned by Ember (`owner=\"ember\"`) given to a C++ parameter that keeps it");
    E5061 = (Error, 5061, Ffi, "[FFI-24]", "imported C++ function has no exception policy and none can be derived");
    E5062 = (Error, 5062, Ffi, "[FFI-43]", "contradictory C++ exception policies");
    E5063 = (Error, 5063, Ffi, "[TXT-2]", "foreign bytes reach `StrView` without validation");
    E5064 = (Error, 5064, Ffi, "[TXT-3], [FFI-15]", "interior NUL: a `c\"...\"` literal containing a NUL, or a value converted to a `CStrView`");
    E5065 = (Error, 5065, Ffi, "[WK-14]", "`shares`/`weakref` and `CppShared`/`CppWeak` do not interconvert");
    E5066 = (Error, 5066, Ffi, "[FFI-43]", "an overlay marks a C++ function both `noexcept` and throwing");
    E5067 = (Error, 5067, Ffi, "[GRM-35], [TIER-1]", "an overlay that states facts is not declared `unsafe overlay`");
    E5068 = (Error, 5068, Ffi, "[FFI-36b]", "one live foreign object adopted twice");
    E5069 = (Error, 5069, Ffi, "[FFI-52], [FFI-11]", "`@ffi(no_callbacks)` on a foreign function whose own contract takes a callback (`callback(...)`)");

    // --- comptime -------------------------------------------------------------
    E6001 = (Error, 6001, Comptime, "[CT-3]", "comptime evaluation exceeded its limits, or a constant's value depends on itself");
    E6002 = (Error, 6002, Comptime, "[CT-4]", "comptime evaluation is not deterministic");
    E6004 = (Error, 6004, Comptime, "[CT-7]", "panic during compile-time evaluation");
    E6005 = (Error, 6005, Comptime, "[CT-6]", "`comptime(e)` refers to a run-time local");
    E6010 = (Error, 6010, Comptime, "[CT-2]", "operation not available at compile time");

    // --- concurrency ----------------------------------------------------------
    E7001 = (Error, 7001, Concurrency, "[THR-1]", "`@sync` class with a field that is not `Sync`, or a non-`@sync` base or derived class");
    E7002 = (Error, 7002, Concurrency, "[STA-1]", "`static` of a type that is not `ThreadShared`");
    E7003 = (Error, 7003, Concurrency, "[THR-1]", "write to a field of a `@sync` class after `init`, or a `mut self` method on one");
    E7004 = (Error, 7004, Concurrency, "[THR-17], [THR-10], [FFI-21]", "an unscoped thread or a kept callback uses a name with no capture word (a loop's name included)");
    E7005 = (Error, 7005, Concurrency, "[THR-11]", "value shared with a task is not `Sync`");
    E7006 = (Error, 7006, Concurrency, "[THR-14]", "memory order an atomic operation does not support");
    E7007 = (Error, 7007, Concurrency, "[THR-11], [THR-21]", "two tasks of one scope change one place other than by a reduction, under a lock or through an `Atomic` (a task that reads the place before changing it, and a plain assignment, included)");
    E7008 = (Error, 7008, Concurrency, "[THR-8], [THR-3]", "a thread-bound value (a guard, a `Scope`, a raw pointer, a `threads(creator)` handle) sent to another thread");
    E7011 = (Error, 7011, Concurrency, "[PAR-2a]", "parallel loop has a loop-carried dependency");
    E7012 = (Error, 7012, Concurrency, "[THR-20], [STA-1]", "a `static mut` not `guarded by` a lock reached from another thread");
    E7013 = (Error, 7013, Concurrency, "[THR-18]", "a `guarded by` field used without holding its lock");
    E7014 = (Error, 7014, Concurrency, "[THR-1]", "a `thread_shared` value changed directly");
    E7015 = (Error, 7015, Concurrency, "[THR-1], [THR-2], [THR-10], [CLO-16], [FFI-22]", "a one-thread owner (`shares`, `shares mut`, a `weakref` link of one) or a lent value reaches another thread: held by a function any thread may run (`thread_shared fn()`, `thread.spawn`'s included), in a plain `fn()` field of a class whose object is made `thread_shared` (at that line)");
    E7016 = (Error, 7016, Concurrency, "[PAR-1]", "a `@parallel` loop over something it cannot split into chunks: neither a range nor a list walked by element");
    E7017 = (Error, 7017, Concurrency, "[THR-1]", "`thread_shared` on a name whose value was not created `thread_shared`");
    E7018 = (Error, 7018, Concurrency, "[THR-19]", "a wait for a lock this thread holds, inside that lock's `with` block: a call through a function value, or a `dyn` or virtual method, that may run a function taking that lock (inside a walk under that lock too), or a join of a thread (a scope's end included) whose function changes what the lock guards");
    E7010 = (Error, 7010, Concurrency, "[PAR-2]", "a parallel loop's chunks may write one place");
    E7020 = (Error, 7020, Concurrency, "[ECS-4]", "systems in one parallel run have conflicting access sets");

    // --- layout ---------------------------------------------------------------
    E8001 = (Error, 8001, Layout, "[GPU-10]", "GPU layout does not match the CPU layout");

    // --- build, manifest, toolchain -------------------------------------------
    // `[TYP-9c]` and `[MAN-3]` both name `E9010`. Both are followed.
    E9010 = (Error, 9010, Build, "[MAN-3]", "`[lints]` names a lint the compiler does not define");
    E9011 = (Error, 9011, Build, "[CG-C-11]", "the toolchain cannot disable floating-point contraction");
    E9020 = (Error, 9020, Build, "[BLD-FFI-1a]", "translation units of one target disagree on an inherited flag");
    E9021 = (Error, 9021, Build, "[BLD-FFI-1b]", "C++ standard library and CRT heap could not be determined");
    E9001 = (Error, 9001, Build, "[MAN-1], [PRF-3]", "invalid manifest, including an unknown key");
    E9002 = (Error, 9002, Build, "[BLD-FFI-1]", "no C compiler found, or it failed; the help names the host's remedy");
    E9003 = (Error, 9003, Build, "[CLI-1]", "invalid command line");

    E9013 = (Error, 9013, Build, "[TCB-1]", "invalid `[ffi]` manifest section");
    E9014 = (Error, 9014, Build, "[RT-12], [DIA-6a]", "the C toolchain cannot emit stack probes for a frame larger than a guard page");
    E9030 = (Error, 9030, Build, "[HR-18], [HR-44], [HR-45], [HR-46], [HR-47]", "hot reload refused");
    E9031 = (Error, 9031, Build, "[MAN-7]", "invalid `reload` value");
    // Registered and never emitted: `[HR-6]`'s permanent thunks make a
    // reloadable function's address stable, so it cannot be stranded.
    E9032 = (Error, 9032, Build, "[HR-6]", "reserved: a reloadable function's address escaping to foreign code");
    E9033 = (Error, 9033, Build, "[MAN-7]", "retired: a `release` build ignores `reload` with a note (ODR-118)");
    E9034 = (Error, 9034, Build, "[MONO-3]", "invalid `max_instantiations` value");
    E9040 = (Error, 9040, Build, "[MNG-1]", "two generic instances hash to one symbol");
    E9035 = (Error, 9035, Build, "[HR-12a]", "packages in one process disagree about the object-header layout");
    E9036 = (Error, 9036, Build, "[HR-29]", "a reloadable package may not link the runtime statically");
    E9037 = (Error, 9037, Build, "[ABI-1]", "reload ABI mismatch on image load");
    E9041 = (Error, 9041, Build, "[TYP-9c]", "the toolchain cannot honour `@fastmath` or `@fp(…)` for a function");
    E9042 = (Error, 9042, Build, "[BLD-15]", "a link ran past its time limit");

    // --- warnings --------------------------------------------------------------
    W0001 = (Warning, 1, Lex, "[LEX-11]", "dangling doc comment");
    W1002 = (Warning, 1002, Resolve, "[GRM-12]", "binding shadows an enum variant of the same name");
    W1003 = (Warning, 1003, Resolve, "[MOD-3]", "a package module shadows the standard module of the same name");
    W2091 = (Warning, 2091, Types, "[CTL-5]", "unreachable match arm");
    W2015 = (Warning, 2015, Types, "[LEX-17a]", "float literal has more digits than its type keeps");
    W2016 = (Warning, 2016, Types, "[PAN-1]", "`debug_assert` argument has side effects");
    W2111 = (Warning, 2111, Types, "[CLS-4]", "`virtual` has no effect in a final class");
    W2190 = (Warning, 2190, Types, "[ERR-5]", "unused `Result`, `check_disjoint`'s included");
    W2192 = (Warning, 2192, Types, "[VER-3]", "use of a deprecated item");
    W5001 = (Warning, 5001, Ffi, "[FFI-6]", "function-like macro ignored");
    W5031 = (Warning, 5031, Ffi, "[FFI-20a]", "C++ declaration skipped: the header could not be parsed");

    W2220 = (Warning, 2220, Types, "[MONO-3]", "instantiation ceiling exceeded");
    W3012 = (Warning, 3012, Ownership, "[UNS-8]", "unsafe block with no SAFETY note");
    W3013 = (Warning, 3013, Ownership, "[UNS-8]", "a `# SAFETY:` note on an `unsafe fn` line, where `@safety` says it");
    W5033 = (Warning, 5033, Ffi, "[FFI-40]", "`&&`-qualified member skipped");
    W5034 = (Warning, 5034, Ffi, "[FFI-41]", "anonymous-namespace entity skipped");
    W5050 = (Warning, 5050, Ffi, "[TCB-1]", "unbacked or stale grade");
    W5054 = (Warning, 5054, Ffi, "[FFI-37]", "unexercised foreign fact");
    W9030 = (Warning, 9030, Build, "[HR-18]", "reload requires a restart");

    // --- lints ------------------------------------------------------------------
    L1001 = (Lint, 1001, Resolve, "[LNT-1]", "unused binding");
    L1002 = (Lint, 1002, Resolve, "[LNT-2]", "assignment declares a new binding");
    L3011 = (Lint, 3011, Ownership, "[CELL-7]", "`RefCell` guard held across a call");
    L3013 = (Lint, 3013, Ownership, "[EXC-7]", "long-term access held across a call");
    L3014 = (Lint, 3014, Ownership, "[LT-1b]", "return region is the intersection of several parameters");
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
    L3015 = (Lint, 3015, Ownership, "[UNS-7]", "an `unsafe fn` with no `@safety`");
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
