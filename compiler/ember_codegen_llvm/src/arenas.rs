//! `[ARN-1]`..`[ARN-11]`, `[JOB-5]` — arenas: an `Arena` owns the runtime's arena state, which
//! its allocations copy into and its end frees; a `ScopedArena` marks the state and rewinds to
//! the mark at its end; a `FixedArena` bumps through a caller's buffer. The arena-backed `Array`
//! and `Map` take their storage from one once. As the C backend's builtins, through the same
//! runtime entry points.

use crate::func::{FnCx, Val};
use crate::types::Sc;
use crate::{Result, unsupported};
use ember_mir::{Builtin, Operand, Place};
use ember_types::{Ty, TyKind};

/// The offset of `ty`'s field `name`.
pub(crate) fn field_offset(m: &crate::Module<'_>, ty: Ty, name: &str) -> Option<u64> {
    let table = m.ty.table;
    let TyKind::Struct(id) = *table.kind(ty) else { return None };
    let index = table.struct_def(id).fields.iter().position(|field| field.name.as_str() == name)?;
    Some(table.layout(ty).field_offsets[index])
}

impl<'a> FnCx<'_, 'a> {
    /// The word at `name` of the struct of type `ty` at `at`.
    pub(crate) fn word_of(&mut self, at: &str, ty: Ty, name: &str) -> Result<String> {
        let Some(offset) = field_offset(self.m, ty, name) else { return unsupported(format!("a `{}` without `{name}`", self.m.ty.table.display(ty))) };
        let field = self.gep(at, offset);
        Ok(self.load(Sc::Ptr, &field, 8))
    }

    /// An arena operand's state: `arena->state`, the operand a reference to the arena.
    fn arena_state(&mut self, operand: &Operand, arena: Ty) -> Result<String> {
        let at = self.pointer(operand)?;
        self.word_of(&at, arena, "state")
    }

    /// A value operand's address, a zero-sized one's a byte of its own (`copy_source`).
    fn value_address(&mut self, operand: &Operand, elem: Ty) -> Result<String> {
        let (size, align) = self.m.ty.size_align(elem);
        let value = self.operand(operand)?;
        Ok(self.spill(&value, size.max(1), align).0)
    }

    /// A view `{pointer, count}` stored at `dest`.
    fn store_view_words(&mut self, dest: &Place, pointer: &str, count: &str) -> Result<()> {
        let Some((at, align)) = self.dest_of(dest)? else { return Ok(()) };
        self.emit(format!("store ptr {pointer}, ptr {at}, align {}", align.min(8)));
        let len_at = self.gep(&at, 8);
        self.emit(format!("store i64 {count}, ptr {len_at}, align {}", align.min(8)));
        Ok(())
    }

    /// The struct at `dest`, zeroed, its fields `values` in order from the first (words).
    fn store_words(&mut self, dest: &Place, values: &[(&str, &str)]) -> Result<()> {
        let ty = self.place_ty(dest);
        let Some((at, align)) = self.dest_of(dest)? else { return Ok(()) };
        let (size, _) = self.m.ty.size_align(ty);
        self.zero(&at, align, size);
        let offsets = self.m.ty.table.layout(ty).field_offsets;
        for ((ir_ty, value), offset) in values.iter().zip(offsets) {
            let field = self.gep(&at, offset);
            self.emit(format!("store {ir_ty} {value}, ptr {field}, align {}", align.min(8)));
        }
        Ok(())
    }

    /// `[ARN-*]` — an arena builtin, or `None` for another.
    pub(crate) fn arena_builtin(&mut self, which: &Builtin, arg_ty: Ty, args: &[Operand], dest: &Place, span: ember_span::Span) -> Option<Result<()>> {
        Some(match which {
            Builtin::ArenaWithCapacity => (|| {
                let n = self.word(&args[0])?;
                let state = self.alloca(8, 8);
                self.call_rt("arena_new", vec![Val::Imm(Sc::I(64), n)], Some((&state, 8)));
                let state = self.load(Sc::Ptr, &state, 8);
                self.store_words(dest, &[("ptr", &state)])
            })(),
            Builtin::ArenaArrayWithCapacity { elem, .. } | Builtin::ArenaMapWithCapacity { map: elem, .. } => (|| {
                let arena = self.pointer(&args[0])?;
                let arena_ty = self.element_of(self.operand_ty(&args[0]).unwrap_or(arg_ty));
                let state = self.word_of(&arena, arena_ty, "state")?;
                let n = self.word(&args[1])?;
                // The map's storage is its slots: what its pointer field points at.
                let (slot, zeroed) = match which {
                    Builtin::ArenaMapWithCapacity { .. } => {
                        let TyKind::Struct(map_id) = *self.m.ty.kind(*elem) else { return unsupported("an `ArenaMap` that is not a struct") };
                        let pointer = self.m.ty.table.struct_def(map_id).fields[1].ty;
                        let TyKind::Ptr { inner, .. } = *self.m.ty.kind(pointer) else { return unsupported("an `ArenaMap` without its slots") };
                        (inner, true)
                    }
                    _ => (*elem, false),
                };
                let (size, align) = self.m.ty.size_align(slot);
                let storage = self.alloca(8, 8);
                let entry = if zeroed { "arena_alloc_zeroed" } else { "arena_alloc_uninit" };
                let words = vec![Val::Imm(Sc::Ptr, state), Val::Imm(Sc::I(64), n.clone()), Val::Imm(Sc::I(64), size.to_string()), Val::Imm(Sc::I(64), align.to_string())];
                self.call_rt(entry, words, Some((&storage, 8)));
                let storage = self.load(Sc::Ptr, &storage, 8);
                self.store_words(dest, &[("ptr", &arena), ("ptr", &storage), ("i64", "0"), ("i64", &n)])
            })(),
            Builtin::ArenaAlloc { elem } | Builtin::ScopedArenaAlloc { elem } => (|| {
                let holder = self.element_of(self.operand_ty(&args[0]).unwrap_or(arg_ty));
                let state = self.arena_state(&args[0], holder)?;
                let (size, align) = self.m.ty.size_align(*elem);
                let value = self.value_address(&args[1], *elem)?;
                let words = vec![Val::Imm(Sc::Ptr, state), Val::Imm(Sc::I(64), size.to_string()), Val::Imm(Sc::I(64), align.to_string()), Val::Imm(Sc::Ptr, value)];
                self.call_rt_into("arena_alloc_copy", words, dest)
            })(),
            Builtin::ArenaAllocUninit { elem } | Builtin::ArenaAllocArrayZeroed { elem } | Builtin::ArenaAllocArrayDefault { elem, .. } => (|| {
                let holder = self.element_of(self.operand_ty(&args[0]).unwrap_or(arg_ty));
                let state = self.arena_state(&args[0], holder)?;
                let n = self.word(&args[1])?;
                let (size, align) = self.m.ty.size_align(*elem);
                let entry = if matches!(which, Builtin::ArenaAllocArrayZeroed { .. }) { "arena_alloc_zeroed" } else { "arena_alloc_uninit" };
                let storage = self.alloca(8, 8);
                let words = vec![Val::Imm(Sc::Ptr, state), Val::Imm(Sc::I(64), n.clone()), Val::Imm(Sc::I(64), size.to_string()), Val::Imm(Sc::I(64), align.to_string())];
                self.call_rt(entry, words, Some((&storage, 8)));
                let storage = self.load(Sc::Ptr, &storage, 8);
                self.store_view_words(dest, &storage, &n)
            })(),
            Builtin::ArenaReset => (|| {
                let holder = self.element_of(self.operand_ty(&args[0]).unwrap_or(arg_ty));
                let state = self.arena_state(&args[0], holder)?;
                self.call_rt("arena_reset", vec![Val::Imm(Sc::Ptr, state)], None);
                Ok(())
            })(),
            Builtin::FixedArenaAlloc { elem } => (|| {
                let fixed_ty = self.element_of(self.operand_ty(&args[0]).unwrap_or(arg_ty));
                let fixed = self.pointer(&args[0])?;
                let Some(buffer_at) = field_offset(self.m, fixed_ty, "buffer") else { return unsupported("a `FixedArena` without its buffer") };
                let Some(used_at) = field_offset(self.m, fixed_ty, "used") else { return unsupported("a `FixedArena` without its count") };
                let buffer = self.gep(&fixed, buffer_at);
                let pointer = self.load(Sc::Ptr, &buffer, 8);
                let capacity_at = self.gep(&buffer, 8);
                let capacity = self.load(Sc::I(64), &capacity_at, 8);
                let used = self.gep(&fixed, used_at);
                let (size, align) = self.m.ty.size_align(*elem);
                let value = self.value_address(&args[1], *elem)?;
                let words = vec![
                    Val::Imm(Sc::Ptr, pointer),
                    Val::Imm(Sc::I(64), capacity),
                    Val::Imm(Sc::Ptr, used),
                    Val::Imm(Sc::I(64), size.to_string()),
                    Val::Imm(Sc::I(64), align.to_string()),
                    Val::Imm(Sc::Ptr, value),
                ];
                self.call_rt_into("fixed_arena_alloc_copy", words, dest)
            })(),
            Builtin::FixedArenaReset => (|| {
                let fixed_ty = self.element_of(self.operand_ty(&args[0]).unwrap_or(arg_ty));
                let fixed = self.pointer(&args[0])?;
                let Some(used_at) = field_offset(self.m, fixed_ty, "used") else { return unsupported("a `FixedArena` without its count") };
                let used = self.gep(&fixed, used_at);
                self.emit(format!("store i64 0, ptr {used}, align 8"));
                Ok(())
            })(),
            // `{&parent->token, parent->state, arena_mark(parent->state), 0}`.
            Builtin::ArenaScope { .. } => (|| {
                let parent_ty = self.element_of(self.operand_ty(&args[0]).unwrap_or(arg_ty));
                let parent = self.pointer(&args[0])?;
                let Some(token_at) = field_offset(self.m, parent_ty, "token") else { return unsupported("an arena without its token") };
                let token = self.gep(&parent, token_at);
                let state = self.word_of(&parent, parent_ty, "state")?;
                let mark = self.alloca(8, 8);
                self.call_rt("arena_mark", vec![Val::Imm(Sc::Ptr, state.clone())], Some((&mark, 8)));
                let mark = self.load(Sc::Ptr, &mark, 8);
                self.store_words(dest, &[("ptr", &token), ("ptr", &state), ("ptr", &mark)])
            })(),
            // `[JOB-5]` — a scope of the running job's arena; the runtime panics outside a job.
            Builtin::JobLocalArena { .. } => (|| {
                let anchor = self.pointer(&args[0])?;
                let loc = self.loc(span);
                let state = self.alloca(8, 8);
                self.call_rt("job_local_arena", vec![loc], Some((&state, 8)));
                let state = self.load(Sc::Ptr, &state, 8);
                let mark = self.alloca(8, 8);
                self.call_rt("arena_mark", vec![Val::Imm(Sc::Ptr, state.clone())], Some((&mark, 8)));
                let mark = self.load(Sc::Ptr, &mark, 8);
                self.store_words(dest, &[("ptr", &anchor), ("ptr", &state), ("ptr", &mark)])
            })(),
            _ => return None,
        })
    }

    /// `[ARN-1]` — an `Arena`'s end frees its state; `[ARN-6]` — a `ScopedArena`'s rewinds the
    /// shared state to its mark.
    pub(crate) fn arena_drop(&mut self, at: &str, ty: Ty, scoped: bool) -> Result<()> {
        let state = self.word_of(at, ty, "state")?;
        if scoped {
            let mark = self.word_of(at, ty, "mark")?;
            self.call_rt("arena_rewind", vec![Val::Imm(Sc::Ptr, state), Val::Imm(Sc::Ptr, mark)], None);
        } else {
            self.call_rt("arena_free", vec![Val::Imm(Sc::Ptr, state)], None);
        }
        Ok(())
    }
}
