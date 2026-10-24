// Precedent for A1: Rust already exposes a property inferred from a function body across crates.
// `make` does not say whether its result is Send; the body decides (it captures an Rc).
use std::rc::Rc;
pub fn make() -> impl Fn() -> usize {
    let shared = Rc::new(5usize);
    move || *shared
}
