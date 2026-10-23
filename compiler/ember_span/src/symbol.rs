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
//!
//! `[BLD-7]` — reading a symbol's text takes no lock: the texts sit in pages
//! of write-once slots, each slot set (under the interning lock) before its
//! symbol exists, so the analyses on the driver's threads, which read names
//! all the time, never wait on one another. Only interning a new text locks.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Mutex, OnceLock};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol(u32);

/// Symbols per page, and pages: room for 64 million texts, a page allocated when first used.
const PAGE: usize = 4096;
const PAGES: usize = 16384;

type Page = Box<[OnceLock<&'static str>]>;

/// The texts, by symbol: each slot written once, when its symbol is made.
static TEXTS: [OnceLock<Page>; PAGES] = [const { OnceLock::new() }; PAGES];

fn slot(sym: Symbol) -> Option<&'static OnceLock<&'static str>> {
    let index = sym.0 as usize;
    TEXTS.get(index / PAGE)?.get().map(|page| &page[index % PAGE])
}

/// What interning looks up and counts; only interning takes it.
struct Interner {
    lookup: HashMap<&'static str, Symbol>,
}

impl Interner {
    fn intern(&mut self, text: &str) -> Symbol {
        if let Some(&sym) = self.lookup.get(text) {
            return sym;
        }
        let index = self.lookup.len();
        assert!(index < PAGE * PAGES, "more than {} distinct identifiers", PAGE * PAGES);
        let leaked: &'static str = Box::leak(text.to_string().into_boxed_str());
        let page = TEXTS[index / PAGE].get_or_init(|| (0..PAGE).map(|_| OnceLock::new()).collect());
        page[index % PAGE].set(leaked).expect("a symbol's slot is written once");
        let sym = Symbol(index as u32);
        self.lookup.insert(leaked, sym);
        sym
    }
}

fn interner() -> &'static Mutex<Interner> {
    static INTERNER: OnceLock<Mutex<Interner>> = OnceLock::new();
    INTERNER.get_or_init(|| Mutex::new(Interner { lookup: HashMap::new() }))
}

impl Symbol {
    pub fn intern(text: &str) -> Symbol {
        interner().lock().unwrap_or_else(|poisoned| poisoned.into_inner()).intern(text)
    }

    pub fn as_str(self) -> &'static str {
        slot(self).and_then(OnceLock::get).copied().expect("a symbol is made by interning its text")
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

    /// `[BLD-7]` — threads interning and reading at once each get one symbol per text, and every
    /// symbol reads back as its text, past a page's end too.
    #[test]
    fn threads_intern_and_read_alike() {
        let made: Vec<Vec<(String, Symbol)>> = std::thread::scope(|scope| {
            let threads: Vec<_> = (0..8)
                .map(|thread| {
                    scope.spawn(move || {
                        (0..PAGE + 100)
                            .map(|n| {
                                let text = format!("shared_{n}_{}", n % (thread + 1));
                                let symbol = Symbol::intern(&text);
                                assert_eq!(symbol.as_str(), text);
                                (text, symbol)
                            })
                            .collect()
                    })
                })
                .collect();
            threads.into_iter().map(|thread| thread.join().expect("a thread")).collect()
        });
        for (text, symbol) in made.iter().flatten() {
            assert_eq!(Symbol::intern(text), *symbol);
            assert_eq!(symbol.as_str(), text);
        }
    }

    #[test]
    fn discard_is_recognised() {
        assert!(Symbol::intern("_").is_discard());
        assert!(!Symbol::intern("_x").is_discard());
    }
}
