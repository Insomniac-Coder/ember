//! `[SOA-1]`..`[SOA-7]` — an `SoA[T]` as `emit_soa_helpers` writes its operations: one column
//! per field of `T` (a flattened field's split into its own), all in one block where each column
//! starts on a 64-byte boundary, grown by doubling from 8; then the length and the capacity, and
//! for an `ArenaSoA` the arena its one block came from.

use ember_hir::SoAOp;
use ember_mir::{Operand, Place};
use ember_span::Span;
use ember_types::{Ty, TyKind};

use crate::{fault, two_words, Interp, R};

/// Where an `SoA`'s parts are, and its columns'.
struct Shape {
    /// Each column's pointer's offset in the `SoA`, its element type and size, and where its field
    /// is in a whole element.
    columns: Vec<(u64, Ty, u64, u64)>,
    len: u64,
    cap: u64,
    arena: Option<u64>,
    element: Ty,
}

impl Interp<'_, '_> {
    fn soa_shape(&self, soa: Ty) -> R<Shape> {
        let TyKind::Struct(id) = *self.types.kind(soa) else { return fault("an `SoA` that is no struct") };
        let def = self.types.struct_def(id);
        let Some((_, args)) = &def.origin else { return fault("an `SoA` with no element") };
        let element = args[0];
        let TyKind::Struct(element_id) = *self.types.kind(element) else { return fault("an `SoA` of no struct") };
        let layout = self.layout(soa);
        let field = |name: &str| def.fields.iter().position(|f| f.name.as_str() == name).map(|i| layout.field_offsets[i]);
        let mut columns = Vec::new();
        for (k, column) in self.types.soa_columns(element_id).into_iter().enumerate() {
            // Where the field is in a whole element (`.pos.x`).
            let (mut structure, mut ty, mut offset) = (element, element, 0);
            for &index in &column.path {
                let TyKind::Struct(inner) = *self.types.kind(structure) else { break };
                offset += self.layout(structure).field_offsets[index];
                ty = self.types.struct_def(inner).fields[index].ty;
                structure = ty;
            }
            columns.push((layout.field_offsets[k], ty, self.size(ty), offset));
        }
        let (Some(len), Some(cap)) = (field("len"), field("cap")) else { return fault("an `SoA` with no length") };
        Ok(Shape { columns, len, cap, arena: field("arena"), element })
    }

    /// `_layout`: each column's offset in a block for `cap` elements, and the block's size.
    fn soa_layout(&self, shape: &Shape, cap: u64) -> (Vec<u64>, u64) {
        let mut end = 0u64;
        let mut at = Vec::with_capacity(shape.columns.len());
        for &(_, ty, size, _) in &shape.columns {
            let a = self.layout(ty).align.max(64);
            end = end.div_ceil(a) * a;
            at.push(end);
            end += cap * size;
        }
        (at, end)
    }

    /// `_grow`: room for `need` elements.
    fn soa_grow(&mut self, shape: &Shape, s: u64, need: u64) -> R<()> {
        let (len, old) = (self.mem.read_u64(s + shape.len)?, self.mem.read_u64(s + shape.cap)?);
        let mut cap = if old != 0 { old } else { 8 };
        while cap < need {
            cap *= 2;
        }
        if !shape.columns.is_empty() {
            let (at, size) = self.soa_layout(shape, cap);
            let block = self.mem.alloc(size.max(1), 64)?;
            for (k, &(pointer, _, elem, _)) in shape.columns.iter().enumerate() {
                if len != 0 {
                    let from = self.mem.read_u64(s + pointer)?;
                    self.mem.copy(from, block + at[k], len * elem)?;
                }
            }
            if old != 0 {
                let first = self.mem.read_u64(s + shape.columns[0].0)?;
                self.mem.free(first)?;
            }
            for (k, &(pointer, ..)) in shape.columns.iter().enumerate() {
                self.mem.write_u64(s + pointer, block + at[k])?;
            }
        }
        self.mem.write_u64(s + shape.cap, cap)?;
        Ok(())
    }

    fn soa_slot(&self, s: u64, column: (u64, Ty, u64, u64), index: u64) -> R<u64> {
        Ok(self.mem.read_u64(s + column.0)? + index * column.2)
    }

    /// Element `index`'s fields, each from its column, into a whole element.
    fn soa_read(&mut self, shape: &Shape, s: u64, index: u64) -> R<Vec<u8>> {
        let mut value = vec![0u8; self.size(shape.element) as usize];
        for &column in &shape.columns {
            let from = self.soa_slot(s, column, index)?;
            let bytes = self.mem.read(from, column.2)?;
            value[column.3 as usize..(column.3 + column.2) as usize].copy_from_slice(bytes);
        }
        Ok(value)
    }

    fn soa_write(&mut self, shape: &Shape, s: u64, index: u64, value: &[u8]) -> R<()> {
        for &column in &shape.columns {
            let to = self.soa_slot(s, column, index)?;
            self.mem.write(to, &value[column.3 as usize..(column.3 + column.2) as usize])?;
        }
        Ok(())
    }

    fn soa_move(&mut self, shape: &Shape, s: u64, from: u64, to: u64) -> R<()> {
        for &column in &shape.columns {
            let (a, b) = (self.soa_slot(s, column, from)?, self.soa_slot(s, column, to)?);
            self.mem.copy(a, b, column.2)?;
        }
        Ok(())
    }

    /// Element `index`'s fields dropped.
    fn soa_drop_slot(&mut self, shape: &Shape, s: u64, index: u64) -> R<()> {
        for &column in &shape.columns {
            if self.needs_drop(column.1) {
                let at = self.soa_slot(s, column, index)?;
                self.drop_at(at, column.1)?;
            }
        }
        Ok(())
    }

    fn soa_clear(&mut self, shape: &Shape, s: u64) -> R<()> {
        let len = self.mem.read_u64(s + shape.len)?;
        for i in 0..len {
            self.soa_drop_slot(shape, s, i)?;
        }
        self.mem.write_u64(s + shape.len, 0)?;
        Ok(())
    }

    /// `_drop`: the elements' fields, then the block.
    pub(crate) fn soa_drop(&mut self, addr: u64, soa: Ty) -> R<()> {
        let shape = self.soa_shape(soa)?;
        self.soa_clear(&shape, addr)?;
        if self.mem.read_u64(addr + shape.cap)? != 0 && !shape.columns.is_empty() {
            let first = self.mem.read_u64(addr + shape.columns[0].0)?;
            self.mem.free(first)?;
        }
        Ok(())
    }

    pub(crate) fn soa(&mut self, op: SoAOp, soa: Ty, args: &[Operand], dest: &Place, span: Span) -> R<Option<Vec<u8>>> {
        let shape = self.soa_shape(soa)?;
        let int = |interp: &Self, value: i64| interp.int_result(interp.common.i64, i128::from(value));
        Ok(Some(match op {
            SoAOp::WithCapacity | SoAOp::WithCapacityIn => {
                let s = self.place(dest)?.addr;
                self.mem.fill(s, self.size(soa), 0)?;
                if op == SoAOp::WithCapacity {
                    let n = self.signed(&args[0])?;
                    if n > 0 {
                        self.soa_grow(&shape, s, n as u64)?;
                    }
                    return Ok(None);
                }
                // `[SOA-4]` — one block from the arena, never grown or freed.
                let arena = self.pointer(&args[0])?;
                let n = self.signed(&args[1])?.max(0) as u64;
                if let Some(field) = shape.arena {
                    self.mem.write_u64(s + field, arena)?;
                }
                let (at, size) = self.soa_layout(&shape, n);
                let state = self.mem.read_u64(arena)?;
                let block = self.arena_alloc(state, size.max(1), 64)?;
                for (k, &(pointer, ..)) in shape.columns.iter().enumerate() {
                    self.mem.write_u64(s + pointer, block + at[k])?;
                }
                self.mem.write_u64(s + shape.cap, n)?;
                return Ok(None);
            }
            SoAOp::Push | SoAOp::TryPush => {
                let s = self.pointer(&args[0])?;
                let value = self.operand(&args[1], Some(shape.element))?;
                let (len, cap) = (self.mem.read_u64(s + shape.len)?, self.mem.read_u64(s + shape.cap)?);
                if len == cap {
                    if op == SoAOp::TryPush {
                        return Ok(Some(vec![1]));
                    }
                    self.soa_grow(&shape, s, len + 1)?;
                }
                self.soa_write(&shape, s, len, &value)?;
                self.mem.write_u64(s + shape.len, len + 1)?;
                if op == SoAOp::TryPush {
                    vec![0]
                } else {
                    return Ok(None);
                }
            }
            SoAOp::TakeLast => {
                let s = self.pointer(&args[0])?;
                let len = self.mem.read_u64(s + shape.len)? - 1;
                self.mem.write_u64(s + shape.len, len)?;
                self.soa_read(&shape, s, len)?
            }
            SoAOp::Len => {
                let s = self.pointer(&args[0])?;
                int(self, self.mem.read_u64(s + shape.len)? as i64)
            }
            SoAOp::Clear => {
                let s = self.pointer(&args[0])?;
                self.soa_clear(&shape, s)?;
                return Ok(None);
            }
            SoAOp::Reserve => {
                let s = self.pointer(&args[0])?;
                let n = self.signed(&args[1])?;
                let (len, cap) = (self.mem.read_u64(s + shape.len)?, self.mem.read_u64(s + shape.cap)?);
                if n > 0 && len + n as u64 > cap {
                    self.soa_grow(&shape, s, len + n as u64)?;
                }
                return Ok(None);
            }
            SoAOp::SwapRemove => {
                let s = self.pointer(&args[0])?;
                let i = self.signed(&args[1])?;
                let len = self.mem.read_u64(s + shape.len)?;
                if i < 0 || i as u64 >= len {
                    return self.panic(&crate::bounds_message(i as u64, len), span);
                }
                let value = self.soa_read(&shape, s, i as u64)?;
                let last = len - 1;
                self.mem.write_u64(s + shape.len, last)?;
                if i as u64 != last {
                    self.soa_move(&shape, s, last, i as u64)?;
                }
                value
            }
            SoAOp::Swap => {
                let s = self.pointer(&args[0])?;
                let (i, j) = (self.signed(&args[1])? as u64, self.signed(&args[2])? as u64);
                let (a, b) = (self.soa_read(&shape, s, i)?, self.soa_read(&shape, s, j)?);
                self.soa_write(&shape, s, i, &b)?;
                self.soa_write(&shape, s, j, &a)?;
                return Ok(None);
            }
            SoAOp::Truncate => {
                let s = self.pointer(&args[0])?;
                let n = self.signed(&args[1])?.max(0) as u64;
                let len = self.mem.read_u64(s + shape.len)?;
                if n < len {
                    for i in n..len {
                        self.soa_drop_slot(&shape, s, i)?;
                    }
                    self.mem.write_u64(s + shape.len, n)?;
                }
                return Ok(None);
            }
            SoAOp::MoveSlot => {
                let s = self.pointer(&args[0])?;
                let (from, to) = (self.signed(&args[1])? as u64, self.signed(&args[2])? as u64);
                self.soa_move(&shape, s, from, to)?;
                return Ok(None);
            }
            SoAOp::DropAt => {
                let s = self.pointer(&args[0])?;
                let i = self.signed(&args[1])? as u64;
                self.soa_drop_slot(&shape, s, i)?;
                return Ok(None);
            }
            SoAOp::SetLen => {
                let s = self.pointer(&args[0])?;
                let n = self.signed(&args[1])? as u64;
                self.mem.write_u64(s + shape.len, n)?;
                return Ok(None);
            }
            // `ps.f`: the column's pointer, and the length.
            SoAOp::Column { .. } => {
                let pointer = self.pointer(&args[0])?;
                let len = self.word(&args[1])?;
                two_words(self.mem.read_u64(pointer)?, len)
            }
            SoAOp::Element { .. } | SoAOp::Iter { .. } => return fault(format!("`{}` reached the evaluator", op.name())),
        }))
    }
}
