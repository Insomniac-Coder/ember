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
    E0104 = (Error, 104, Parse, "[ATT-1]", "unknown attribute");

    // --- name resolution, modules, visibility -------------------------------
    E1010 = (Error, 1010, Resolve, "[MOD-3]", "cannot find name in this scope");
    E1020 = (Error, 1020, Resolve, "[GRM-4]", "name is already declared in this block");
    E1030 = (Error, 1030, Resolve, "[TYP-26]", "two items with the same name in one scope");
    E1040 = (Error, 1040, Resolve, "[MOD-3]", "wildcard import from a module that is not a prelude");
    E1041 = (Error, 1041, Resolve, "[MOD-4]", "import cycle between packages");
    E1050 = (Error, 1050, Resolve, "[MOD-7]", "field is read-only outside its module");
    E1051 = (Error, 1051, Resolve, "[MOD-7]", "`read` visibility is valid on fields only");

    // --- types, inference, interfaces, generics, patterns -------------------
    E2010 = (Error, 2010, Types, "[LEX-16]", "literal does not fit its type");
    E2020 = (Error, 2020, Types, "[TYP-4]", "mismatched types");
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
    E2140 = (Error, 2140, Types, "[EXP-5]", "cannot assign to a value expression");
    E2150 = (Error, 2150, Types, "[VI.3]", "`is` requires a type with identity");
    E2151 = (Error, 2151, Types, "[VI.3]", "integer `**` with a negative exponent");
    E2160 = (Error, 2160, Types, "[CTL-7]", "control flow cannot leave a `defer` block");
    E2170 = (Error, 2170, Types, "[IX.5]", "cannot take a reference to a field of a packed struct");
    E2180 = (Error, 2180, Types, "[ERR-2]", "`?` outside a function returning `Option` or `Result`");
    E2200 = (Error, 2200, Types, "[XVIII.4.4]", "type has infinite size");

    // --- ownership, borrows, regions, exclusivity, drops --------------------
    E3010 = (Error, 3010, Ownership, "[EXP-6]", "cannot move out of a field of a type with `drop`");
    E3011 = (Error, 3011, Ownership, "[EXP-6]", "cannot move out of an array or span element");
    E3012 = (Error, 3012, Ownership, "[EXP-6]", "cannot move out of a class field");
    E3013 = (Error, 3013, Ownership, "[EXP-6]", "cannot move out of a reference");
    E3020 = (Error, 3020, Ownership, "[CTL-2]", "iterable is mutated while the loop borrows it");
    E3030 = (Error, 3030, Ownership, "[CLO-2]", "closure would move a captured value out");
    E3040 = (Error, 3040, Ownership, "[OWN-3]", "use of moved value");
    E3041 = (Error, 3041, Ownership, "[OWN-4]", "value moved in a previous loop iteration");
    E3050 = (Error, 3050, Ownership, "[BRW-7]", "use of an uninitialised or moved place");
    E3060 = (Error, 3060, Ownership, "[LT-3]", "borrowed value does not live long enough");
    E3061 = (Error, 3061, Ownership, "[LT-4]", "arena allocation cannot outlive its arena");
    E3062 = (Error, 3062, Ownership, "[XVIII.4.7]", "returned reference does not derive from a parameter");
    E3070 = (Error, 3070, Ownership, "[DRP-1]", "`drop` cannot be called explicitly");
    E3080 = (Error, 3080, Ownership, "[EXC-3]", "overlapping access through the same handle");
    E3090 = (Error, 3090, Ownership, "[ARN-3]", "arena allocation of a type that needs `drop`");

    // --- effects and contracts ----------------------------------------------
    E4001 = (Error, 4001, Effects, "[EFF-5]", "`@noalloc` function reaches an allocation");
    E4010 = (Error, 4010, Effects, "[EFF-2]", "implementation does not satisfy the interface's contract");
    E4020 = (Error, 4020, Effects, "[SIMD-2]", "`@simd(assert)` loop did not vectorise");

    // --- FFI -----------------------------------------------------------------
    E5001 = (Error, 5001, Ffi, "[FFI-5]", "imported type layout does not match");
    E5010 = (Error, 5010, Ffi, "[FFI-12]", "overlay does not match the C declaration");
    E5020 = (Error, 5020, Ffi, "[FFI-15]", "`str` is not NUL-terminated");
    E5030 = (Error, 5030, Ffi, "[FFI-17]", "`std::function` cannot be imported");
    E5040 = (Error, 5040, Ffi, "[FFI-21]", "capturing closure passed where a C function pointer is expected");
    E5041 = (Error, 5041, Ffi, "[FFI-21]", "retained callback needs a release function in the overlay");
    E5090 = (Error, 5090, Ffi, "[UNS-6]", "inline assembly is not supported by the C backend");

    // --- comptime -------------------------------------------------------------
    E6001 = (Error, 6001, Comptime, "[CT-3]", "comptime evaluation exceeded its limits");
    E6002 = (Error, 6002, Comptime, "[CT-4]", "comptime evaluation is not deterministic");
    E6010 = (Error, 6010, Comptime, "[XVIII.5]", "operation is not supported at compile time");

    // --- concurrency ----------------------------------------------------------
    E7001 = (Error, 7001, Concurrency, "[THR-1]", "`@sync` class has a field that is not Sync");
    E7010 = (Error, 7010, Concurrency, "[PAR-2]", "parallel loop writes to a shared place");
    E7020 = (Error, 7020, Concurrency, "[ECS-4]", "systems in one parallel run have conflicting access sets");

    // --- layout ---------------------------------------------------------------
    E8001 = (Error, 8001, Layout, "[IX.5]", "GPU layout does not match the CPU layout");

    // --- build, manifest, toolchain -------------------------------------------
    E9001 = (Error, 9001, Build, "[MAN-1]", "invalid manifest");
    E9002 = (Error, 9002, Build, "[BLD-4]", "the C toolchain could not be found or failed");
    E9003 = (Error, 9003, Build, "[CLI-1]", "invalid command line");

    // --- warnings --------------------------------------------------------------
    W0001 = (Warning, 1, Lex, "[LEX-11]", "dangling doc comment");
    W1002 = (Warning, 1002, Resolve, "[GRM-12]", "binding shadows an enum variant of the same name");
    W2091 = (Warning, 2091, Types, "[CTL-5]", "unreachable match arm");
    W2111 = (Warning, 2111, Types, "[CLS-4]", "`virtual` has no effect in a final class");
    W2190 = (Warning, 2190, Types, "[ERR-5]", "unused `Result`");
    W5001 = (Warning, 5001, Ffi, "[FFI-6]", "function-like macro ignored");
    W5031 = (Warning, 5031, Ffi, "[FFI-19]", "declaration skipped: the header could not be parsed");

    // --- lints ------------------------------------------------------------------
    L2001 = (Lint, 2001, Types, "[XIX.8]", "unnecessary clone");
    L2002 = (Lint, 2002, Types, "[XIX.8]", "large Copy value passed by value");
    L3001 = (Lint, 3001, Ownership, "[WK-1]", "potential reference cycle");
    L3002 = (Lint, 3002, Ownership, "[XIX.8]", "borrow held longer than necessary");
    L3010 = (Lint, 3010, Ownership, "[UNS-3]", "unsafe block larger than necessary");
    L4001 = (Lint, 4001, Effects, "[XIX.8]", "allocation in a hot loop");
    L4002 = (Lint, 4002, Effects, "[XIX.8]", "dynamic dispatch on a final type");
    L5001 = (Lint, 5001, Ffi, "[XIX.8]", "unsafe extern without a contract");
    L5002 = (Lint, 5002, Ffi, "[XIX.8]", "conversion at the FFI boundary copies");
    L7001 = (Lint, 7001, Concurrency, "[XIX.8]", "lock held across a call that may block");
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
