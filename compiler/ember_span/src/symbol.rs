//! Interned identifiers.
//!
//! A [`Symbol`] is a 32-bit index standing for one string. Interning gives
//! name resolution O(1) identity comparison and keeps tokens cheap to copy.
//!
//! `[LEX-12]` — identifiers are NFC-normalised and two identifiers are the
//! same iff their NFC forms are byte-equal. Normalisation happens in the
//! lexer, before interning; this module compares bytes.
//!
//! Interned strings are leaked deliberately. A compiler process interns a
//! bounded set of identifiers and exits; leaking buys a plain `&'static str`
//! with no unsafe code and no lifetime plumbing through every stage.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Mutex, OnceLock};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol(u32);

struct Interner {
    strings: Vec<&'static str>,
    lookup: HashMap<&'static str, Symbol>,
}

impl Interner {
    fn new() -> Interner {
        Interner { strings: Vec::new(), lookup: HashMap::new() }
    }

    fn intern(&mut self, text: &str) -> Symbol {
        if let Some(&sym) = self.lookup.get(text) {
            return sym;
        }
        let leaked: &'static str = Box::leak(text.to_string().into_boxed_str());
        let sym = Symbol(self.strings.len() as u32);
        self.strings.push(leaked);
        self.lookup.insert(leaked, sym);
        sym
    }

    fn resolve(&self, sym: Symbol) -> &'static str {
        self.strings[sym.0 as usize]
    }
}

fn interner() -> &'static Mutex<Interner> {
    static INTERNER: OnceLock<Mutex<Interner>> = OnceLock::new();
    INTERNER.get_or_init(|| Mutex::new(Interner::new()))
}

impl Symbol {
    pub fn intern(text: &str) -> Symbol {
        interner().lock().expect("symbol interner poisoned").intern(text)
    }

    pub fn as_str(self) -> &'static str {
        interner().lock().expect("symbol interner poisoned").resolve(self)
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }

    pub fn is(self, text: &str) -> bool {
        self.as_str() == text
    }

    /// `_`, the discard pattern (`[LEX-13]`). Never a variable.
    pub fn is_discard(self) -> bool {
        self.as_str() == "_"
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}`", self.as_str())
    }
}

impl From<&str> for Symbol {
    fn from(text: &str) -> Symbol {
        Symbol::intern(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_text_interns_to_the_same_symbol() {
        let a = Symbol::intern("position");
        let b = Symbol::intern("position");
        assert_eq!(a, b);
        assert_ne!(a, Symbol::intern("velocity"));
    }

    #[test]
    fn a_symbol_resolves_back_to_its_text() {
        let s = Symbol::intern("on_update");
        assert_eq!(s.as_str(), "on_update");
        assert_eq!(s.to_string(), "on_update");
        assert!(s.is("on_update"));
    }

    #[test]
    fn discard_is_recognised() {
        assert!(Symbol::intern("_").is_discard());
        assert!(!Symbol::intern("_x").is_discard());
    }
}
