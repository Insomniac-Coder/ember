//! `[HR-3b]` (A1, ODR-133) — frame records of a reloadable image with instances. Each function keeps
//! one while it runs (pushed on entry, popped on return): which function it is and which of its calls
//! it is in. What each call's continuation depends on is the planner's (`image::Planner::frame_info`);
//! this writes the record's C.

use super::*;

impl Emitter<'_> {
    /// Whether this image keeps frame records: a reloadable build whose instances migrate.
    pub(crate) fn frames_on(&self) -> bool {
        self.unit.is_none() && self.reload.as_ref().is_some_and(|reload| reload.instances)
    }

    /// The site number of the call `terminator` ends, when it is one of `body`'s calls.
    pub(crate) fn frame_site(&self, terminator: &Terminator, body: &Body) -> Option<u32> {
        if !matches!(terminator, Terminator::Call { .. }) {
            return None;
        }
        let block = body.blocks.iter().position(|block| std::ptr::eq(&block.terminator, terminator))?;
        self.rl_sites.get(&block).copied()
    }
}
