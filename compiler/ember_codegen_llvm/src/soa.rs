//! `[SOA-1]`..`[SOA-7]` — an `SoA[T]`'s operations, as the C backend's `emit_soa_helpers` writes
//! them: one helper function per operation and `SoA` type. One block holds every column, each at
//! the larger of 64 bytes and its element's alignment (`[SOA-7]`); growth takes a new block,
//! copies the columns over and frees the old one.

use crate::func::{FnCx, Val};
use crate::types::Sc;
use crate::{Module, Result, global_name, unsupported};
use ember_mir::SoAOp;
use ember_types::{Ty, TyKind};

/// A column: its element type, size and alignment, where the `SoA` holds its pointer, and where
/// a whole element holds its field (`.pos.x`).
struct Column {
    ty: Ty,
    size: u64,
    align: u64,
    pointer: u64,
    within: u64,
}

struct Shape {
    columns: Vec<Column>,
    len: u64,
    cap: u64,
    /// `[SOA-4]` — an `ArenaSoA`'s borrow of its arena.
    arena: Option<u64>,
    soa_size: u64,
    elem_size: u64,
    elem_align: u64,
}

fn shape(m: &Module<'_>, soa: Ty) -> Result<Shape> {
    let table = m.ty.table;
    let TyKind::Struct(id) = *table.kind(soa) else { return unsupported("an `SoA` that is not a struct") };
    let def = table.struct_def(id);
    match &def.origin {
        Some((origin, args)) if matches!(origin.as_str(), "SoA" | "ArenaSoA") && args.len() == 1 => {}
        _ => return unsupported("an `SoA` of no element"),
    }
    let elem = def.origin.as_ref().expect("an SoA has its element").1[0];
    let TyKind::Struct(elem_id) = *table.kind(elem) else { return unsupported("an `SoA` of a non-struct") };
    let offsets = table.layout(soa).field_offsets;
    let at = |name: &str| def.fields.iter().position(|field| field.name.as_str() == name).map(|index| offsets[index]);
    let mut columns = Vec::new();
    for column in table.soa_columns(elem_id) {
        let mut within = 0;
        let mut structure = elem;
        for &index in &column.path {
            within += table.layout(structure).field_offsets[index];
            let TyKind::Struct(inner) = *table.kind(structure) else { unreachable!("a column's path goes through structs") };
            structure = table.struct_def(inner).fields[index].ty;
        }
        let (size, align) = m.ty.size_align(column.ty);
        let pointer = at(&format!("col_{}", column.name)).expect("an SoA has a pointer per column");
        columns.push(Column { ty: column.ty, size, align, pointer, within });
    }
    let (elem_size, elem_align) = m.ty.size_align(elem);
    Ok(Shape {
        columns,
        len: at("len").expect("an SoA has a length"),
        cap: at("cap").expect("an SoA has a capacity"),
        arena: at("arena"),
        soa_size: m.ty.size_align(soa).0,
        elem_size,
        elem_align,
    })
}

/// `[HR-13]` — `size_t f(size_t cap, size_t* at)`: an `SoA`'s one block laid out for `cap`
/// elements, each column's start in `at`, its size returned; a reload rebuilding the block calls it
/// (the C backend's `<SoA>_layout`).
pub(crate) fn layout_fn(m: &mut Module<'_>, soa: Ty) -> Result<String> {
    let symbol = format!("ember.soa.layout.{}", m.ty.table.mangled(soa));
    if m.helper_defined(&symbol) {
        return Ok(symbol);
    }
    let shape = shape(m, soa)?;
    let mut cx = FnCx::glue(m);
    let (offsets, size) = cx.soa_layout(&shape, "%p0");
    for (index, offset) in offsets.iter().enumerate() {
        let at = cx.gep("%p1", 8 * index as u64);
        cx.emit(format!("store i64 {offset}, ptr {at}, align 8"));
    }
    let attrs = crate::func::fn_attrs(cx.m);
    let text = format!(
        "define internal i64 {}(i64 %p0, ptr %p1) {attrs} {{\nentry:\n{}  br label %start\nstart:\n{}  ret i64 {size}\n}}\n\n",
        global_name(&symbol),
        cx.allocas_text(),
        cx.out_text()
    );
    cx.m.fns.push_str(&text);
    Ok(symbol)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Helper {
    Grow,
    WithCapacity,
    Push,
    TakeLast,
    Clear,
    Swap,
    MoveSlot,
    DropAt,
    Truncate,
    Reserve,
    SwapRemove,
    Drop,
}

impl Helper {
    fn name(self) -> &'static str {
        match self {
            Helper::Grow => "grow",
            Helper::WithCapacity => "with_capacity",
            Helper::Push => "push",
            Helper::TakeLast => "take_last",
            Helper::Clear => "clear",
            Helper::Swap => "swap",
            Helper::MoveSlot => "move_slot",
            Helper::DropAt => "drop_at",
            Helper::Truncate => "truncate",
            Helper::Reserve => "reserve",
            Helper::SwapRemove => "swap_remove",
            Helper::Drop => "drop",
        }
    }

    /// The parameters: the `SoA`'s address first (but `with_capacity`'s), the result's last.
    fn params(self) -> &'static [&'static str] {
        match self {
            Helper::Grow | Helper::DropAt | Helper::Truncate | Helper::Reserve => &["ptr", "i64"],
            Helper::WithCapacity => &["i64", "ptr"],
            Helper::Push | Helper::TakeLast => &["ptr", "ptr"],
            Helper::Clear | Helper::Drop => &["ptr"],
            Helper::Swap | Helper::MoveSlot => &["ptr", "i64", "i64"],
            Helper::SwapRemove => &["ptr", "i64", "ptr", "ptr"],
        }
    }
}

/// The helper `helper` for `soa`, written the first time.
fn helper(m: &mut Module<'_>, helper: Helper, soa: Ty) -> Result<String> {
    let symbol = format!("ember.soa.{}.{}", helper.name(), m.ty.table.mangled(soa));
    if m.helper_defined(&symbol) {
        return Ok(global_name(&symbol));
    }
    let shape = shape(m, soa)?;
    crate::glue::define_glue(m, &symbol, helper.params(), "", |cx| {
        let s = "%p0";
        match helper {
            Helper::Grow => grow(cx, &shape),
            Helper::WithCapacity => {
                cx.zero("%p1", 8, shape.soa_size);
                let (grow, done) = (cx.label(), cx.label());
                let some = cx.value("icmp sgt i64 %p0, 0");
                cx.emit(format!("br i1 {some}, label %{grow}, label %{done}"));
                cx.start(&grow);
                let function = self::helper(cx.m, Helper::Grow, soa)?;
                cx.emit(format!("call void {function}(ptr %p1, i64 %p0)"));
                cx.emit(format!("br label %{done}"));
                cx.start(&done);
                Ok(())
            }
            Helper::Push => {
                let len = cx.word_at(s, shape.len);
                let cap = cx.word_at(s, shape.cap);
                let full = cx.value(format!("icmp eq i64 {len}, {cap}"));
                let (grow, store) = (cx.label(), cx.label());
                cx.emit(format!("br i1 {full}, label %{grow}, label %{store}"));
                cx.start(&grow);
                let need = cx.value(format!("add i64 {len}, 1"));
                let function = self::helper(cx.m, Helper::Grow, soa)?;
                cx.emit(format!("call void {function}(ptr {s}, i64 {need})"));
                cx.emit(format!("br label %{store}"));
                cx.start(&store);
                for column in &shape.columns {
                    let slot = cx.slot(s, column, &len);
                    let field = cx.gep("%p1", column.within);
                    cx.copy(&slot, column.align, &field, column.align, column.size, false);
                }
                let longer = cx.value(format!("add i64 {len}, 1"));
                cx.set_len(s, &shape, &longer);
                Ok(())
            }
            Helper::TakeLast => {
                let len = cx.word_at(s, shape.len);
                let last = cx.value(format!("sub i64 {len}, 1"));
                cx.set_len(s, &shape, &last);
                cx.zero("%p1", shape.elem_align, shape.elem_size);
                for column in &shape.columns {
                    let slot = cx.slot(s, column, &last);
                    let field = cx.gep("%p1", column.within);
                    cx.copy(&field, column.align, &slot, column.align, column.size, false);
                }
                Ok(())
            }
            Helper::Clear => {
                let len = cx.word_at(s, shape.len);
                cx.drop_range(s, &shape, "0", &len)?;
                cx.set_len(s, &shape, "0");
                Ok(())
            }
            Helper::Swap => {
                for column in &shape.columns {
                    let (a, b) = (cx.slot(s, column, "%p1"), cx.slot(s, column, "%p2"));
                    let temp = cx.alloca(column.size, column.align);
                    cx.copy(&temp, column.align, &a, column.align, column.size, false);
                    cx.copy(&a, column.align, &b, column.align, column.size, true);
                    cx.copy(&b, column.align, &temp, column.align, column.size, false);
                }
                Ok(())
            }
            Helper::MoveSlot => {
                for column in &shape.columns {
                    let (from, to) = (cx.slot(s, column, "%p1"), cx.slot(s, column, "%p2"));
                    cx.copy(&to, column.align, &from, column.align, column.size, true);
                }
                Ok(())
            }
            Helper::DropAt => {
                for column in &shape.columns {
                    let slot = cx.slot(s, column, "%p1");
                    cx.drop_at(&slot, column.align, column.ty)?;
                }
                Ok(())
            }
            Helper::Truncate => {
                let floor = cx.value("icmp slt i64 %p1, 0");
                let n = cx.value(format!("select i1 {floor}, i64 0, i64 %p1"));
                let len = cx.word_at(s, shape.len);
                let shorter = cx.value(format!("icmp ult i64 {n}, {len}"));
                let (cut, done) = (cx.label(), cx.label());
                cx.emit(format!("br i1 {shorter}, label %{cut}, label %{done}"));
                cx.start(&cut);
                cx.drop_range(s, &shape, &n, &len)?;
                cx.set_len(s, &shape, &n);
                cx.emit(format!("br label %{done}"));
                cx.start(&done);
                Ok(())
            }
            Helper::Reserve => {
                let len = cx.word_at(s, shape.len);
                let cap = cx.word_at(s, shape.cap);
                let some = cx.value("icmp sgt i64 %p1, 0");
                let need = cx.value(format!("add i64 {len}, %p1"));
                let over = cx.value(format!("icmp ugt i64 {need}, {cap}"));
                let grows = cx.value(format!("and i1 {some}, {over}"));
                let (grow, done) = (cx.label(), cx.label());
                cx.emit(format!("br i1 {grows}, label %{grow}, label %{done}"));
                cx.start(&grow);
                let function = self::helper(cx.m, Helper::Grow, soa)?;
                cx.emit(format!("call void {function}(ptr {s}, i64 {need})"));
                cx.emit(format!("br label %{done}"));
                cx.start(&done);
                Ok(())
            }
            Helper::SwapRemove => {
                // A negative index is a huge one, as C's `(size_t)i` makes it.
                let len = cx.word_at(s, shape.len);
                let outside = cx.value(format!("icmp uge i64 %p1, {len}"));
                let (panic, take) = (cx.label(), cx.label());
                cx.emit(format!("br i1 {outside}, label %{panic}, label %{take}"));
                cx.start(&panic);
                cx.call_rt(
                    "panic_bounds",
                    vec![Val::Imm(Sc::I(64), "%p1".into()), Val::Imm(Sc::I(64), len.clone()), Val::Mem("%p2".into(), 8)],
                    None,
                );
                cx.start(&take);
                cx.zero("%p3", shape.elem_align, shape.elem_size);
                for column in &shape.columns {
                    let slot = cx.slot(s, column, "%p1");
                    let field = cx.gep("%p3", column.within);
                    cx.copy(&field, column.align, &slot, column.align, column.size, false);
                }
                let last = cx.value(format!("sub i64 {len}, 1"));
                cx.set_len(s, &shape, &last);
                let moved = cx.value(format!("icmp ne i64 %p1, {last}"));
                let (fill, done) = (cx.label(), cx.label());
                cx.emit(format!("br i1 {moved}, label %{fill}, label %{done}"));
                cx.start(&fill);
                for column in &shape.columns {
                    let (hole, tail) = (cx.slot(s, column, "%p1"), cx.slot(s, column, &last));
                    cx.copy(&hole, column.align, &tail, column.align, column.size, false);
                }
                cx.emit(format!("br label %{done}"));
                cx.start(&done);
                Ok(())
            }
            Helper::Drop => {
                let function = self::helper(cx.m, Helper::Clear, soa)?;
                cx.emit(format!("call void {function}(ptr {s})"));
                let Some(first) = shape.columns.first() else { return Ok(()) };
                let cap = cx.word_at(s, shape.cap);
                let owns = cx.value(format!("icmp ne i64 {cap}, 0"));
                let (free, done) = (cx.label(), cx.label());
                cx.emit(format!("br i1 {owns}, label %{free}, label %{done}"));
                cx.start(&free);
                let (_, size) = cx.soa_layout(&shape, &cap);
                let block = cx.field_ptr(s, first.pointer);
                cx.free_block(&block, &size);
                cx.emit(format!("br label %{done}"));
                cx.start(&done);
                Ok(())
            }
        }
    })?;
    Ok(global_name(&symbol))
}

/// `grow(s, need)`: the capacity doubled from 8 until it holds `need`, one new block, each
/// column copied over, the old block freed.
fn grow(cx: &mut FnCx<'_, '_>, shape: &Shape) -> Result<()> {
    let s = "%p0";
    let old = cx.word_at(s, shape.cap);
    let empty = cx.value(format!("icmp eq i64 {old}, 0"));
    let first = cx.value(format!("select i1 {empty}, i64 8, i64 {old}"));
    let entry = cx.block_label();
    let (head, double, done) = (cx.label(), cx.label(), cx.label());
    cx.emit(format!("br label %{head}"));
    cx.start(&head);
    let (cap, doubled) = (cx.tmp(), cx.tmp());
    cx.emit(format!("{cap} = phi i64 [ {first}, %{entry} ], [ {doubled}, %{double} ]"));
    let short = cx.value(format!("icmp ult i64 {cap}, %p1"));
    cx.emit(format!("br i1 {short}, label %{double}, label %{done}"));
    cx.start(&double);
    cx.emit(format!("{doubled} = mul i64 {cap}, 2"));
    cx.emit(format!("br label %{head}"));
    cx.start(&done);
    if let Some(first) = shape.columns.first() {
        let (at, size) = cx.soa_layout(shape, &cap);
        let nonzero = cx.value(format!("icmp eq i64 {size}, 0"));
        let size = cx.value(format!("select i1 {nonzero}, i64 1, i64 {size}"));
        let block = cx.alloca(8, 8);
        cx.call_rt("alloc", vec![Val::Imm(Sc::I(64), size), Val::Imm(Sc::I(64), "64".into())], Some((&block, 8)));
        let block = cx.load(Sc::Ptr, &block, 8);
        let len = cx.word_at(s, shape.len);
        let intrinsic = "llvm.memcpy.p0.p0.i64";
        cx.m.declare(intrinsic, || format!("declare void @{intrinsic}(ptr, ptr, i64, i1)"));
        let mut new_columns = Vec::new();
        for (column, at) in shape.columns.iter().zip(&at) {
            let to = cx.value(format!("getelementptr i8, ptr {block}, i64 {at}"));
            let from = cx.field_ptr(s, column.pointer);
            let bytes = cx.value(format!("mul i64 {len}, {}", column.size));
            cx.emit(format!("call void @{intrinsic}(ptr align {} {to}, ptr align {} {from}, i64 {bytes}, i1 false)", column.align, column.align));
            new_columns.push(to);
        }
        let had = cx.value(format!("icmp ne i64 {old}, 0"));
        let (free, set) = (cx.label(), cx.label());
        cx.emit(format!("br i1 {had}, label %{free}, label %{set}"));
        cx.start(&free);
        let (_, old_size) = cx.soa_layout(shape, &old);
        let old_block = cx.field_ptr(s, first.pointer);
        cx.free_block(&old_block, &old_size);
        cx.emit(format!("br label %{set}"));
        cx.start(&set);
        for (column, to) in shape.columns.iter().zip(&new_columns) {
            let at = cx.gep(s, column.pointer);
            cx.emit(format!("store ptr {to}, ptr {at}, align 8"));
        }
    }
    let cap_at = cx.gep(s, shape.cap);
    cx.emit(format!("store i64 {cap}, ptr {cap_at}, align 8"));
    Ok(())
}

impl<'a> FnCx<'_, 'a> {
    /// The `i64` at `offset` in `s`.
    fn word_at(&mut self, s: &str, offset: u64) -> String {
        let at = self.gep(s, offset);
        self.load(Sc::I(64), &at, 8)
    }

    /// The pointer at `offset` in `s`.
    fn field_ptr(&mut self, s: &str, offset: u64) -> String {
        let at = self.gep(s, offset);
        self.load(Sc::Ptr, &at, 8)
    }

    /// Element `index`'s slot in `column`.
    fn slot(&mut self, s: &str, column: &Column, index: &str) -> String {
        let base = self.field_ptr(s, column.pointer);
        self.element_at(&base, index, column.size)
    }

    /// The length set to `len`.
    fn set_len(&mut self, s: &str, shape: &Shape, len: &str) {
        let at = self.gep(s, shape.len);
        self.emit(format!("store i64 {len}, ptr {at}, align 8"));
    }

    /// Each column's elements from `from` to `to` dropped, where any needs it.
    fn drop_range(&mut self, s: &str, shape: &Shape, from: &str, to: &str) -> Result<()> {
        let table = self.m.ty.table;
        if !shape.columns.iter().any(|column| table.needs_drop(column.ty)) {
            return Ok(());
        }
        let count = self.value(format!("sub i64 {to}, {from}"));
        self.counted_loop(&count, |cx, index| {
            let index = cx.value(format!("add i64 {index}, {from}"));
            for column in &shape.columns {
                if cx.m.ty.table.needs_drop(column.ty) {
                    let slot = cx.slot(s, column, &index);
                    cx.drop_at(&slot, column.align, column.ty)?;
                }
            }
            Ok(())
        })
    }

    /// The block's layout for `cap` elements: each column's offset, and the block's size.
    fn soa_layout(&mut self, shape: &Shape, cap: &str) -> (Vec<String>, String) {
        let mut end = "0".to_string();
        let mut at = Vec::with_capacity(shape.columns.len());
        for column in &shape.columns {
            let align = column.align.max(64);
            let raised = self.value(format!("add i64 {end}, {}", align - 1));
            let start = self.value(format!("and i64 {raised}, {}", (align as i64).wrapping_neg()));
            let bytes = self.value(format!("mul i64 {cap}, {}", column.size));
            end = self.value(format!("add i64 {start}, {bytes}"));
            at.push(start);
        }
        (at, end)
    }

    /// The block at `block` of `size` bytes freed (C's `size ? size : 1`).
    fn free_block(&mut self, block: &str, size: &str) {
        let empty = self.value(format!("icmp eq i64 {size}, 0"));
        let size = self.value(format!("select i1 {empty}, i64 1, i64 {size}"));
        self.call_rt(
            "free",
            vec![Val::Imm(Sc::Ptr, block.to_string()), Val::Imm(Sc::I(64), size), Val::Imm(Sc::I(64), "64".into())],
            None,
        );
    }

    /// `[SOA-1]` — an `SoA` operation.
    pub(crate) fn soa_op(&mut self, op: &SoAOp, soa: Ty, args: &[ember_mir::Operand], dest: &ember_mir::Place, span: ember_span::Span) -> Result<()> {
        let call = |cx: &mut Self, which: Helper, rest: &str| -> Result<()> {
            let function = helper(cx.m, which, soa)?;
            cx.emit(format!("call void {function}({rest})"));
            Ok(())
        };
        match op {
            SoAOp::WithCapacity => {
                let n = self.word(&args[0])?;
                let at = self.result_slot(dest, shape(self.m, soa)?.soa_size, 8)?;
                call(self, Helper::WithCapacity, &format!("i64 {n}, ptr {at}"))
            }
            // `[borrow of the column's pointer, the length]`: the view of the column.
            SoAOp::Column { .. } => {
                let pointer = self.pointer(&args[0])?;
                let base = self.load(Sc::Ptr, &pointer, 8);
                let len = self.word(&args[1])?;
                let Some((at, align)) = self.dest_of(dest)? else { return Ok(()) };
                self.emit(format!("store ptr {base}, ptr {at}, align {}", align.min(8)));
                let len_at = self.gep(&at, 8);
                self.emit(format!("store i64 {len}, ptr {len_at}, align {}", align.min(8)));
                Ok(())
            }
            SoAOp::Len => {
                let s = self.pointer(&args[0])?;
                let len = self.word_at(&s, shape(self.m, soa)?.len);
                self.store_word(dest, &len)
            }
            SoAOp::SetLen => {
                let s = self.pointer(&args[0])?;
                let n = self.word(&args[1])?;
                let at = self.gep(&s, shape(self.m, soa)?.len);
                self.emit(format!("store i64 {n}, ptr {at}, align 8"));
                Ok(())
            }
            SoAOp::Push => {
                let s = self.pointer(&args[0])?;
                let shape = shape(self.m, soa)?;
                let value = self.operand(&args[1])?;
                let (value, _) = self.spill(&value, shape.elem_size, shape.elem_align);
                call(self, Helper::Push, &format!("ptr {s}, ptr {value}"))
            }
            SoAOp::TakeLast | SoAOp::SwapRemove => {
                let s = self.pointer(&args[0])?;
                let shape = shape(self.m, soa)?;
                let at = self.result_slot(dest, shape.elem_size, shape.elem_align)?;
                if matches!(op, SoAOp::TakeLast) {
                    return call(self, Helper::TakeLast, &format!("ptr {s}, ptr {at}"));
                }
                let index = self.word(&args[1])?;
                let loc = self.loc(span);
                let (loc, _) = self.spill(&loc, 16, 8);
                call(self, Helper::SwapRemove, &format!("ptr {s}, i64 {index}, ptr {loc}, ptr {at}"))
            }
            SoAOp::Clear => {
                let s = self.pointer(&args[0])?;
                call(self, Helper::Clear, &format!("ptr {s}"))
            }
            SoAOp::Reserve | SoAOp::Truncate | SoAOp::DropAt => {
                let s = self.pointer(&args[0])?;
                let n = self.word(&args[1])?;
                let which = match op {
                    SoAOp::Reserve => Helper::Reserve,
                    SoAOp::Truncate => Helper::Truncate,
                    _ => Helper::DropAt,
                };
                call(self, which, &format!("ptr {s}, i64 {n}"))
            }
            SoAOp::Swap | SoAOp::MoveSlot => {
                let s = self.pointer(&args[0])?;
                let (i, j) = (self.word(&args[1])?, self.word(&args[2])?);
                let which = if matches!(op, SoAOp::Swap) { Helper::Swap } else { Helper::MoveSlot };
                call(self, which, &format!("ptr {s}, i64 {i}, i64 {j}"))
            }
            // `[SOA-4]` — an `ArenaSoA`: its one block taken from the arena, never grown or freed.
            SoAOp::WithCapacityIn => {
                let shape = shape(self.m, soa)?;
                let arena = self.pointer(&args[0])?;
                let arena_ty = self.element_of(self.operand_ty(&args[0]).unwrap_or(soa));
                let state = self.word_of(&arena, arena_ty, "state")?;
                let n = self.word(&args[1])?;
                let positive = self.value(format!("icmp sgt i64 {n}, 0"));
                let cap = self.value(format!("select i1 {positive}, i64 {n}, i64 0"));
                let s = self.result_slot(dest, shape.soa_size, 8)?;
                self.zero(&s, 8, shape.soa_size);
                if let Some(at) = shape.arena {
                    let field = self.gep(&s, at);
                    self.emit(format!("store ptr {arena}, ptr {field}, align 8"));
                }
                let (offsets, size) = self.soa_layout(&shape, &cap);
                let empty = self.value(format!("icmp eq i64 {size}, 0"));
                let size = self.value(format!("select i1 {empty}, i64 1, i64 {size}"));
                let block = self.alloca(8, 8);
                let words = vec![
                    Val::Imm(Sc::Ptr, state),
                    Val::Imm(Sc::I(64), "1".to_string()),
                    Val::Imm(Sc::I(64), size),
                    Val::Imm(Sc::I(64), "64".to_string()),
                ];
                self.call_rt("arena_alloc_uninit", words, Some((&block, 8)));
                let block = self.load(Sc::Ptr, &block, 8);
                for (column, offset) in shape.columns.iter().zip(&offsets) {
                    let start = self.value(format!("getelementptr i8, ptr {block}, i64 {offset}"));
                    let field = self.gep(&s, column.pointer);
                    self.emit(format!("store ptr {start}, ptr {field}, align 8"));
                }
                let cap_at = self.gep(&s, shape.cap);
                self.emit(format!("store i64 {cap}, ptr {cap_at}, align 8"));
                Ok(())
            }
            // `true` when it was full and nothing changed.
            SoAOp::TryPush => {
                let shape = shape(self.m, soa)?;
                let s = self.pointer(&args[0])?;
                let value = self.operand(&args[1])?;
                let (value, _) = self.spill(&value, shape.elem_size, shape.elem_align);
                let len = self.word_at(&s, shape.len);
                let cap = self.word_at(&s, shape.cap);
                let full = self.value(format!("icmp eq i64 {len}, {cap}"));
                let (store, done) = (self.label(), self.label());
                self.emit(format!("br i1 {full}, label %{done}, label %{store}"));
                self.start(&store);
                for column in &shape.columns {
                    let slot = self.slot(&s, column, &len);
                    let field = self.gep(&value, column.within);
                    self.copy(&slot, column.align, &field, column.align, column.size, false);
                }
                let longer = self.value(format!("add i64 {len}, 1"));
                self.set_len(&s, &shape, &longer);
                self.emit(format!("br label %{done}"));
                self.start(&done);
                self.store_result(dest, &full, Sc::Bool, false)
            }
            SoAOp::Element { .. } | SoAOp::Iter { .. } => unsupported("an `SoA` element or iterator outside a field access or `for`"),
        }
    }

    /// The `SoA` at `ptr` dropped: its elements' fields, then its block.
    pub(crate) fn soa_drop(&mut self, ptr: &str, soa: Ty) -> Result<()> {
        let function = helper(self.m, Helper::Drop, soa)?;
        self.emit(format!("call void {function}(ptr {ptr})"));
        Ok(())
    }

    /// Where a helper writes its result: the destination, or a scratch slot when it is unused.
    fn result_slot(&mut self, dest: &ember_mir::Place, size: u64, align: u64) -> Result<String> {
        Ok(match self.dest_of(dest)? {
            Some((ptr, _)) => ptr,
            None => self.alloca(size, align),
        })
    }

    /// An `i64` result stored into `dest`, narrowed or widened to its type.
    fn store_word(&mut self, dest: &ember_mir::Place, value: &str) -> Result<()> {
        let at = self.place(dest)?;
        match self.m.ty.repr(at.ty) {
            crate::types::Repr::Scalar(Sc::I(bits)) => {
                let value = self.int_convert(value, 64, bits, true);
                self.store_scalar(Sc::I(bits), &value, &at.ptr, at.align);
                Ok(())
            }
            crate::types::Repr::Zst => Ok(()),
            _ => unsupported("an `SoA` length stored into a non-integer"),
        }
    }
}
