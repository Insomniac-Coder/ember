//! `[ARN-1]`..`[ARN-12]` — arenas as the runtime keeps them (`ember_arena_*`): a list of chunks
//! that never move, the first of the asked-for capacity, each later one a megabyte or the
//! allocation's size; a bump allocation in the current chunk or a later one; a reset or a rewind
//! to a mark makes the chunks' room free again. An `Arena`'s `state` is the address of a block of
//! the evaluator's own, which names its chunks here.

use ember_hir::Builtin;
use ember_mir::Operand;
use ember_types::{Ty, TyKind};

use crate::{fault, two_words, Interp, R};

const GROWTH_CHUNK: u64 = 1024 * 1024;
/// `_Alignof(arena_max_align)`.
const MAX_ALIGN: u64 = 16;

struct Chunk {
    data: u64,
    capacity: u64,
    used: u64,
    align: u64,
}

#[derive(Default)]
pub struct ArenaState {
    chunks: Vec<Chunk>,
    current: Option<usize>,
}

impl Chunk {
    /// `arena_chunk_take`.
    fn take(&mut self, size: u64, align: u64) -> Option<u64> {
        if self.align < align {
            return None;
        }
        let offset = (self.used + align - 1) & !(align - 1);
        if offset > self.capacity || size > self.capacity - offset {
            return None;
        }
        self.used = offset + size;
        Some(self.data + offset)
    }
}

impl Interp<'_, '_> {
    fn arena_panic<T>(&mut self, message: &str) -> R<T> {
        self.panic_in("<arena>", message)
    }

    fn arena_new(&mut self, capacity: u64) -> R<u64> {
        let state = self.mem.alloc(16, 16)?;
        let mut arena = ArenaState::default();
        if capacity != 0 {
            let data = self.mem.alloc(capacity, MAX_ALIGN)?;
            arena.chunks.push(Chunk { data, capacity, used: 0, align: MAX_ALIGN });
            arena.current = Some(0);
        }
        self.system.arenas.insert(state, arena);
        Ok(state)
    }

    /// `arena_alloc_raw`: `size` bytes aligned to `align`, not cleared.
    pub(crate) fn arena_alloc(&mut self, state: u64, size: u64, align: u64) -> R<u64> {
        if !align.is_power_of_two() || !self.system.arenas.contains_key(&state) {
            return self.arena_panic("invalid arena allocation");
        }
        let size = size.max(1);
        let arena = self.system.arenas.get_mut(&state).expect("an arena");
        let start = arena.current.unwrap_or(0);
        for index in start..arena.chunks.len() {
            if let Some(slot) = arena.chunks[index].take(size, align) {
                arena.current = Some(index);
                return Ok(slot);
            }
        }
        let capacity = size.max(GROWTH_CHUNK);
        let chunk_align = align.max(MAX_ALIGN);
        let data = self.mem.alloc(capacity, chunk_align)?;
        let arena = self.system.arenas.get_mut(&state).expect("an arena");
        arena.chunks.push(Chunk { data, capacity, used: 0, align: chunk_align });
        arena.current = Some(arena.chunks.len() - 1);
        match arena.chunks.last_mut().expect("the new chunk").take(size, align) {
            Some(slot) => Ok(slot),
            None => self.arena_panic("arena allocation overflow"),
        }
    }

    fn arena_alloc_count(&mut self, state: u64, count: u64, elem: u64, align: u64, zeroed: bool) -> R<u64> {
        let Some(bytes) = count.checked_mul(elem) else { return self.arena_panic("arena allocation overflow") };
        let slot = self.arena_alloc(state, bytes, align)?;
        if zeroed && bytes != 0 {
            self.mem.fill(slot, bytes, 0)?;
        }
        Ok(slot)
    }

    fn arena_mark(&self, state: u64) -> u64 {
        match self.system.arenas.get(&state) {
            Some(ArenaState { chunks, current: Some(at) }) => chunks[*at].data + chunks[*at].used,
            _ => 0,
        }
    }

    fn arena_reset(&mut self, state: u64) {
        if let Some(arena) = self.system.arenas.get_mut(&state) {
            arena.chunks.iter_mut().for_each(|chunk| chunk.used = 0);
            arena.current = (!arena.chunks.is_empty()).then_some(0);
        }
    }

    /// `ember_arena_rewind`: a scope's end gives back what was taken after its mark.
    pub(crate) fn arena_rewind(&mut self, state: u64, mark: u64) -> R<()> {
        if mark == 0 {
            self.arena_reset(state);
            return Ok(());
        }
        let Some(arena) = self.system.arenas.get_mut(&state) else { return Ok(()) };
        let Some(at) = arena.chunks.iter().position(|c| c.data <= mark && mark - c.data <= c.capacity) else {
            return self.arena_panic("invalid arena mark");
        };
        arena.chunks[at].used = mark - arena.chunks[at].data;
        arena.chunks[at + 1..].iter_mut().for_each(|chunk| chunk.used = 0);
        arena.current = Some(at);
        Ok(())
    }

    /// `ember_arena_free`: an `Arena`'s end frees its chunks.
    pub(crate) fn arena_free(&mut self, state: u64) -> R<()> {
        if let Some(arena) = self.system.arenas.remove(&state) {
            for chunk in arena.chunks {
                self.mem.free(chunk.data)?;
            }
            self.mem.free(state)?;
        }
        Ok(())
    }

    /// The offset of `name` in the struct `ty`.
    pub(crate) fn field_offset(&self, ty: Ty, name: &str) -> R<u64> {
        let TyKind::Struct(id) = *self.types.kind(ty) else { return fault(format!("a field `{name}` of no struct")) };
        let Some(index) = self.types.struct_def(id).fields.iter().position(|f| f.name.as_str() == name) else {
            return fault(format!("`{}` has no field `{name}`", self.types.display(ty)));
        };
        Ok(self.layout(ty).field_offsets[index])
    }

    /// What a reference type refers to.
    fn referent(&self, ty: Ty) -> Ty {
        match self.types.kind(ty) {
            TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. } => *inner,
            _ => ty,
        }
    }

    /// The state of the arena (or scope) a borrow of type `of` points at.
    fn arena_state(&mut self, operand: &Operand, of: Ty) -> R<u64> {
        let at = self.pointer(operand)?;
        let field = self.field_offset(self.referent(of), "state")?;
        Ok(self.mem.read_u64(at + field)?)
    }

    pub(crate) fn arena_builtin(&mut self, which: &Builtin, arg_ty: Ty, args: &[Operand], dest_ty: Ty) -> R<Option<Vec<u8>>> {
        let body = self.body();
        let first_ty = self.operand_ty(body, &args[0]).unwrap_or(arg_ty);
        let copy = |interp: &mut Self, state: u64, elem: Ty, value: &Operand| -> R<Vec<u8>> {
            let bytes = interp.operand(value, Some(elem))?;
            let align = if interp.is_void(elem) { 1 } else { interp.layout(elem).align };
            let slot = interp.arena_alloc(state, interp.size(elem), align)?;
            interp.mem.write(slot, &bytes)?;
            Ok(slot.to_le_bytes().to_vec())
        };
        Ok(Some(match which {
            Builtin::ArenaWithCapacity => {
                let capacity = self.word(&args[0])?;
                let state = self.arena_new(capacity)?;
                let mut value = vec![0u8; self.size(dest_ty) as usize];
                let at = self.field_offset(dest_ty, "state")? as usize;
                value[at..at + 8].copy_from_slice(&state.to_le_bytes());
                value
            }
            Builtin::ArenaAlloc { elem } | Builtin::ScopedArenaAlloc { elem } => {
                let state = self.arena_state(&args[0], first_ty)?;
                copy(self, state, *elem, &args[1])?
            }
            Builtin::ArenaAllocUninit { elem } | Builtin::ArenaAllocArrayZeroed { elem } | Builtin::ArenaAllocArrayDefault { elem, .. } => {
                let state = self.arena_state(&args[0], first_ty)?;
                let count = self.word(&args[1])?;
                let (size, align) = (self.size(*elem), self.layout(*elem).align);
                let zeroed = matches!(which, Builtin::ArenaAllocArrayZeroed { .. });
                let slot = self.arena_alloc_count(state, count, size, align, zeroed)?;
                two_words(slot, count)
            }
            Builtin::ArenaReset => {
                let state = self.arena_state(&args[0], first_ty)?;
                self.arena_reset(state);
                return Ok(None);
            }
            // `ember_fixed_arena_alloc_copy`: from the caller's buffer, never grown.
            Builtin::FixedArenaAlloc { elem } => {
                let fixed = self.referent(first_ty);
                let at = self.pointer(&args[0])?;
                let (buffer, used_at) = (at + self.field_offset(fixed, "buffer")?, at + self.field_offset(fixed, "used")?);
                let (base, capacity) = (self.mem.read_u64(buffer)?, self.mem.read_u64(buffer + 8)?);
                let used = self.mem.read_u64(used_at)?;
                let size = self.size(*elem).max(1);
                let align = if self.is_void(*elem) { 1 } else { self.layout(*elem).align };
                if capacity == 0 {
                    return self.panic_in("<fixed arena>", "fixed arena exhausted");
                }
                let aligned = (base + used + align - 1) & !(align - 1);
                let offset = aligned - base;
                if offset > capacity || size > capacity - offset {
                    return self.panic_in("<fixed arena>", "fixed arena exhausted");
                }
                self.mem.write_u64(used_at, offset + size)?;
                let bytes = self.operand(&args[1], Some(*elem))?;
                self.mem.write(aligned, &bytes)?;
                aligned.to_le_bytes().to_vec()
            }
            Builtin::FixedArenaReset => {
                let fixed = self.referent(first_ty);
                let at = self.pointer(&args[0])?;
                self.mem.write_u64(at + self.field_offset(fixed, "used")?, 0)?;
                return Ok(None);
            }
            // `[ARN-6]` — the parent's token borrowed, its state, and the mark to rewind to.
            Builtin::ArenaScope { scoped } => {
                let parent = self.referent(first_ty);
                let at = self.pointer(&args[0])?;
                let state = self.mem.read_u64(at + self.field_offset(parent, "state")?)?;
                let token = at + self.field_offset(parent, "token")?;
                let mark = self.arena_mark(state);
                let mut value = vec![0u8; self.size(*scoped) as usize];
                for (name, word) in [("parent_token", token), ("state", state), ("mark", mark)] {
                    let offset = self.field_offset(*scoped, name)? as usize;
                    value[offset..offset + 8].copy_from_slice(&word.to_le_bytes());
                }
                value
            }
            // `[ARN-5]` — the view: its arena, its fixed storage, no length, the capacity.
            Builtin::ArenaArrayWithCapacity { elem, array } => {
                let arena = self.pointer(&args[0])?;
                let state = self.arena_state(&args[0], first_ty)?;
                let n = self.word(&args[1])?;
                let (size, align) = (self.size(*elem), self.layout(*elem).align);
                let slot = self.arena_alloc_count(state, n, size, align, false)?;
                self.struct_of_words(*array, &[arena, slot, 0, n])?
            }
            Builtin::ArenaMapWithCapacity { map, .. } => {
                let arena = self.pointer(&args[0])?;
                let state = self.arena_state(&args[0], first_ty)?;
                let n = self.word(&args[1])?;
                let TyKind::Struct(id) = *self.types.kind(*map) else { return fault("an `ArenaMap` that is no struct") };
                let TyKind::Ptr { inner: slot_ty, .. } = *self.types.kind(self.types.struct_def(id).fields[1].ty) else {
                    return fault("an `ArenaMap` with no slots");
                };
                let (size, align) = (self.size(slot_ty), self.layout(slot_ty).align);
                let slot = self.arena_alloc_count(state, n, size, align, true)?;
                self.struct_of_words(*map, &[arena, slot, 0, n])?
            }
            _ => return fault(format!("the arena builtin `{}`", which.name())),
        }))
    }

    /// A struct whose fields, in order, hold `words`.
    fn struct_of_words(&self, ty: Ty, words: &[u64]) -> R<Vec<u8>> {
        let layout = self.layout(ty);
        let mut value = vec![0u8; layout.size as usize];
        let TyKind::Struct(id) = *self.types.kind(ty) else { return fault("no struct") };
        for (index, word) in words.iter().enumerate() {
            let field = self.types.struct_def(id).fields[index].ty;
            let size = self.size(field) as usize;
            let offset = layout.field_offsets[index] as usize;
            value[offset..offset + size].copy_from_slice(&word.to_le_bytes()[..size]);
        }
        Ok(value)
    }
}
