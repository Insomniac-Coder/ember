//! `[CORO-4]`, `[CORO-7]` — a generator's suspension points, as lowering
//! leaves them for the coroutine transform (`ember_analysis::coroutines`).
//!
//! The checker writes a `gen fn`'s body as a function that calls
//! `std.core.__coroutine_start()` first and `std.core.__coroutine_yield[Y](e)`
//! at each `yield e` (ADR-193). Each call answers whether the generator was
//! resumed, and lowering branches on that answer: resumed, the body goes on;
//! dropped while suspended there, it ends the way a `return` does, dropping
//! everything its open scopes and statements own, last made first, and running
//! no `defer` (`[CORO-7]`: the locals live at the suspension point, in reverse
//! declaration order). Every analysis then sees the drop of a suspended frame
//! as ordinary code, which drop elaboration makes exact (drop flags, moved
//! fields); the transform keeps the resumed edge for `next` and the dropped
//! one for the frame's `drop`.

use crate::{Operand, Place, Terminator};

use super::Builder;

/// The C symbols of the suspension points: `__coroutine_start`, and each
/// instance of `__coroutine_yield` (its symbol and an instance hash, `[MNG-1]`).
/// Made once: every call the lowering and the analyses meet is asked whether it is one.
pub fn start_symbol() -> &'static str {
    static SYMBOL: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| ember_branding::item_symbol("std.core.__coroutine_start"));
    &SYMBOL
}

fn yield_symbol() -> &'static str {
    static SYMBOL: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| ember_branding::item_symbol("std.core.__coroutine_yield"));
    &SYMBOL
}

/// `std.core.__coroutine_resumed`, a `yield` used as a `void` expression.
pub fn resumed_symbol() -> &'static str {
    static SYMBOL: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| ember_branding::item_symbol("std.core.__coroutine_resumed"));
    &SYMBOL
}

pub fn is_start(symbol: &str) -> bool {
    symbol == start_symbol()
}

/// An instance of `std.core.__coroutine_yield`: its symbol, then `G` and the hash.
pub fn is_yield(symbol: &str) -> bool {
    symbol.strip_prefix(yield_symbol()).is_some_and(|rest| rest.starts_with(ember_branding::INSTANCE_MARK))
}

pub fn is_suspension(symbol: &str) -> bool {
    is_start(symbol) || is_yield(symbol)
}

impl Builder<'_> {
    /// The call that just ended the current block answered into `resumed`:
    /// branch on it, the dropped path ending as a `return` does without its
    /// `defer`s.
    pub(super) fn suspend(&mut self, resumed: Place) {
        let dropped = self.new_block();
        let resume = self.new_block();
        self.terminate(Terminator::SwitchInt { discr: Operand::Copy(resumed), targets: vec![(0, dropped)], otherwise: resume });
        self.current = dropped;
        self.emit_all_drops();
        self.terminate(Terminator::Return);
        self.current = resume;
    }
}
