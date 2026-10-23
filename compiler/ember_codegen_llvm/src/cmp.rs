//! D-187 — `==` between aggregates and text: text by its bytes, numbers as their type compares
//! them (IEEE for floats), aggregates part by part through a function of their own
//! (`ember.eq.<type>`, the C backend's `eq_N`), which stops at the first part that differs.

use crate::func::{FnCx, Val};
use crate::glue::Pending;
use crate::types::{Repr, Sc, offset_align};
use crate::{Module, Result, global_name, unsupported};
use ember_types::{Ty, TyKind};

/// Whether `ty` compares through a generated function.
fn has_eq_fn(m: &Module<'_>, ty: Ty) -> bool {
    match m.ty.kind(ty) {
        TyKind::Struct(_) | TyKind::Tuple(_) | TyKind::Array { .. } => true,
        TyKind::Vec { text, .. } => !text,
        TyKind::Enum(id) => !m.ty.table.enum_is_bare(*id),
        _ => false,
    }
}

pub(crate) fn eq_fn_symbol(m: &Module<'_>, ty: Ty) -> String {
    format!("ember.eq.{}", m.ty.table.mangled(ty))
}

/// `bool eq(const T* a, const T* b)`.
pub(crate) fn emit_eq_fn(m: &mut Module<'_>, ty: Ty) -> Result<()> {
    let symbol = eq_fn_symbol(m, ty);
    let mut cx = FnCx::glue(m);
    let (a, b) = ("%p0".to_string(), "%p1".to_string());
    let table = cx.m.ty.table;
    let (_, align) = cx.m.ty.size_align(ty);
    let differ = cx.label();
    // Each part: compared, and on a difference straight to `false`.
    let part = |cx: &mut FnCx<'_, '_>, offset: u64, part_align: u64, part_ty: Ty| -> Result<()> {
        let at_a = cx.gep(&a, offset);
        let at_b = cx.gep(&b, offset);
        let same = cx.eq_at(&at_a, &at_b, part_align, part_ty)?;
        let next = cx.label();
        cx.emit(format!("br i1 {same}, label %{next}, label %{differ}"));
        cx.start(&next);
        Ok(())
    };
    match table.kind(ty).clone() {
        // IX.1 (ODR-241) — two boxes compare what they hold; two `Box[dyn I]` of one kind (one
        // table) through the table's `eq` (after `drop`, `size`, `align`, `access` and, in a table
        // that copies its value, `clone`), of two kinds never.
        TyKind::Struct(id) if table.compiler_box_inner(id).is_some() => {
            let inner = table.compiler_box_inner(id).expect("a box");
            let (data_a, data_b) = (cx.load(Sc::Ptr, &a, 8), cx.load(Sc::Ptr, &b, 8));
            let same = match table.kind(inner).clone() {
                TyKind::Dyn { interfaces } => {
                    let (at_a, at_b) = (cx.gep(&a, 8), cx.gep(&b, 8));
                    let (vtable_a, vtable_b) = (cx.load(Sc::Ptr, &at_a, 8), cx.load(Sc::Ptr, &at_b, 8));
                    let one_kind = cx.value(format!("icmp eq ptr {vtable_a}, {vtable_b}"));
                    let compare = cx.label();
                    cx.emit(format!("br i1 {one_kind}, label %{compare}, label %{differ}"));
                    cx.start(&compare);
                    let eq_at = cx.gep(&vtable_a, 32 + 8 * u64::from(table.dyn_clones(&interfaces)));
                    let eq = cx.load(Sc::Ptr, &eq_at, 8);
                    cx.value(format!("call i1 {eq}(ptr {data_a}, ptr {data_b})"))
                }
                _ => {
                    let (_, inner_align) = cx.m.ty.size_align(inner);
                    cx.eq_at(&data_a, &data_b, inner_align, inner)?
                }
            };
            cx.emit(format!("br i1 {same}, label %same, label %{differ}"));
        }
        TyKind::Struct(id) => {
            let packed = table.layout_attrs(ty).packed;
            let offsets = table.layout(ty).field_offsets;
            for (field, offset) in table.struct_def(id).fields.iter().zip(offsets) {
                let field_align = if packed { 1 } else { align.min(offset_align(offset)) };
                part(&mut cx, offset, field_align, field.ty)?;
            }
        }
        TyKind::Tuple(items) => {
            let offsets = table.layout(ty).field_offsets;
            for (item, offset) in items.iter().zip(offsets) {
                part(&mut cx, offset, align.min(offset_align(offset)), *item)?;
            }
        }
        TyKind::Array { elem, len } => {
            let (size, elem_align) = cx.m.ty.size_align(elem);
            for index in 0..len {
                part(&mut cx, index * size, elem_align.min(align), elem)?;
            }
        }
        TyKind::Vec { elem, .. } => {
            let len_a_at = cx.gep(&a, 8);
            let len_a = cx.load(Sc::I(64), &len_a_at, 8);
            let len_b_at = cx.gep(&b, 8);
            let len_b = cx.load(Sc::I(64), &len_b_at, 8);
            let same_len = cx.value(format!("icmp eq i64 {len_a}, {len_b}"));
            let walk = cx.label();
            cx.emit(format!("br i1 {same_len}, label %{walk}, label %{differ}"));
            cx.start(&walk);
            let (buffer_a, buffer_b) = (cx.load(Sc::Ptr, &a, 8), cx.load(Sc::Ptr, &b, 8));
            let (size, elem_align) = cx.m.ty.size_align(elem);
            let found = cx.alloca(1, 1);
            cx.emit(format!("store i8 1, ptr {found}, align 1"));
            cx.counted_loop(&len_a, |cx, index| {
                let at_a = cx.element_at(&buffer_a, &index, size);
                let at_b = cx.element_at(&buffer_b, &index, size);
                let same = cx.eq_at(&at_a, &at_b, elem_align, elem)?;
                let (mark, next) = (cx.label(), cx.label());
                cx.emit(format!("br i1 {same}, label %{next}, label %{mark}"));
                cx.start(&mark);
                cx.emit(format!("store i8 0, ptr {found}, align 1"));
                cx.emit(format!("br label %{next}"));
                cx.start(&next);
                Ok(())
            })?;
            let all = cx.load(Sc::I(8), &found, 1);
            let equal = cx.value(format!("icmp ne i8 {all}, 0"));
            cx.emit(format!("br i1 {equal}, label %{}, label %{differ}", "same"));
        }
        TyKind::Enum(id) => {
            let def = table.enum_def(id).clone();
            let packed = table.layout_attrs(ty).packed;
            let niche = table.option_niche(id).is_some();
            let offsets = table.variant_field_offsets(id);
            let at_a = crate::func::PlaceRef { ptr: a.clone(), ty, variant: None, align };
            let at_b = crate::func::PlaceRef { ptr: b.clone(), ty, variant: None, align };
            let (tag_a, bits) = cx.discriminant_at(&at_a)?;
            let (tag_b, _) = cx.discriminant_at(&at_b)?;
            let same_tag = cx.value(format!("icmp eq i{bits} {tag_a}, {tag_b}"));
            let arms_label = cx.label();
            cx.emit(format!("br i1 {same_tag}, label %{arms_label}, label %{differ}"));
            cx.start(&arms_label);
            let mut arms = Vec::new();
            let mut bodies = Vec::new();
            let mut seen = std::collections::BTreeSet::new();
            for (index, variant) in def.variants.iter().enumerate() {
                if variant.fields.is_empty() || !seen.insert(variant.discriminant) {
                    continue;
                }
                let label = cx.label();
                arms.push(format!("i{bits} {}, label %{label}", crate::func::int_literal(variant.discriminant as u128, bits)));
                bodies.push((label, index));
            }
            cx.emit(format!("switch i{bits} {tag_a}, label %same [ {} ]", arms.join(" ")));
            for (label, index) in bodies {
                cx.start(&label);
                for (field_index, field) in def.variants[index].fields.iter().enumerate() {
                    let offset = if niche { 0 } else { offsets[index][field_index] };
                    let field_align = if packed { 1 } else { align.min(offset_align(offset)).min(cx.m.ty.size_align(field.ty).1) };
                    part(&mut cx, offset, field_align, field.ty)?;
                }
                cx.emit("br label %same");
            }
        }
        other => return unsupported(format!("an equality function for {other:?}")),
    }
    if !cx.terminated() {
        cx.emit("br label %same");
    }
    cx.start("same");
    cx.emit("ret i1 true");
    cx.start(&differ);
    cx.emit("ret i1 false");
    let attrs = crate::func::fn_attrs(cx.m);
    let text = format!(
        "define internal i1 {}(ptr %p0, ptr %p1) {attrs} {{\nentry:\n{}  br label %start\nstart:\n{}}}\n\n",
        global_name(&symbol),
        cx.allocas_text(),
        cx.out_text()
    );
    cx.m.fns.push_str(&text);
    Ok(())
}

impl<'a> FnCx<'_, 'a> {
    /// Whether the values of `ty` at `a` and `b` are equal (`eq_expr`).
    pub(crate) fn eq_at(&mut self, a: &str, b: &str, align: u64, ty: Ty) -> Result<String> {
        let table = self.m.ty.table;
        // `void` has one value (`[TYP-36]`).
        if self.m.ty.is_zst(ty) {
            return Ok("true".to_string());
        }
        if matches!(table.kind(ty), TyKind::Str | TyKind::Vec { text: true, .. }) {
            let (va, vb) = (self.str_view(a), self.str_view(b));
            let slot = self.alloca(1, 1);
            self.call_rt("str_eq", vec![va, vb], Some((&slot, 1)));
            return Ok(self.load(Sc::Bool, &slot, 1));
        }
        if let TyKind::Fn { abi: None, .. } = table.kind(ty) {
            // Two values are equal when they call the same entry with the same state.
            let mut all = "true".to_string();
            for offset in [8u64, 16, 24, 32] {
                let (at_a, at_b) = (self.gep(a, offset), self.gep(b, offset));
                let (wa, wb) = (self.load(Sc::Ptr, &at_a, 8), self.load(Sc::Ptr, &at_b, 8));
                let same = self.value(format!("icmp eq ptr {wa}, {wb}"));
                all = self.value(format!("and i1 {all}, {same}"));
            }
            return Ok(all);
        }
        if has_eq_fn(self.m, ty) {
            self.m.request(Pending::Eq(ty));
            let symbol = global_name(&eq_fn_symbol(self.m, ty));
            let (_, natural) = self.m.ty.size_align(ty);
            let (a, b) = if align < natural {
                let (size, _) = self.m.ty.size_align(ty);
                let (ca, cb) = (self.alloca(size, natural), self.alloca(size, natural));
                self.copy(&ca, natural, a, align, size, false);
                self.copy(&cb, natural, b, align, size, false);
                (ca, cb)
            } else {
                (a.to_string(), b.to_string())
            };
            return Ok(self.value(format!("call i1 {symbol}(ptr {a}, ptr {b})")));
        }
        let Repr::Scalar(sc) = self.m.ty.repr(ty) else {
            return unsupported(format!("`==` of `{}`", table.display(ty)));
        };
        let (va, vb) = (self.load(sc, a, align), self.load(sc, b, align));
        if self.m.ty.is_f16(ty) {
            // D-316 — IEEE equality, through the widened values.
            let (wa, wb) = (self.alloca(8, 8), self.alloca(8, 8));
            self.call_rt("f16_to_f64", vec![Val::Imm(Sc::I(16), va)], Some((&wa, 8)));
            self.call_rt("f16_to_f64", vec![Val::Imm(Sc::I(16), vb)], Some((&wb, 8)));
            let (da, db) = (self.load(Sc::F64, &wa, 8), self.load(Sc::F64, &wb, 8));
            return Ok(self.value(format!("fcmp oeq double {da}, {db}")));
        }
        Ok(if sc.is_float() {
            self.value(format!("fcmp oeq {} {va}, {vb}", sc.ir()))
        } else {
            self.value(format!("icmp eq {} {va}, {vb}", sc.ir()))
        })
    }

    /// A `str` or a `String` at `at` as a `str` value.
    fn str_view(&mut self, at: &str) -> Val {
        let ptr = self.load(Sc::Ptr, at, 8);
        let len_at = self.gep(at, 8);
        let len = self.load(Sc::I(64), &len_at, 8);
        let view = self.alloca(16, 8);
        self.emit(format!("store ptr {ptr}, ptr {view}, align 8"));
        let view_len = self.gep(&view, 8);
        self.emit(format!("store i64 {len}, ptr {view_len}, align 8"));
        Val::Mem(view, 8)
    }

    /// D-187 — `==`, `!=` and, for text, the orders, between two values of `ty`.
    pub(crate) fn value_compare(&mut self, op: ember_mir::BinOp, a: Val, b: Val, ty: Ty) -> Result<String> {
        use ember_mir::BinOp;
        let (size, align) = self.m.ty.size_align(ty);
        let (pa, aa) = self.spill(&a, size, align);
        let (pb, ab) = self.spill(&b, size, align);
        let text = matches!(self.m.ty.kind(ty), TyKind::Str | TyKind::Vec { text: true, .. });
        if text && !matches!(op, BinOp::Eq | BinOp::Ne) {
            let (va, vb) = (self.str_view(&pa), self.str_view(&pb));
            let slot = self.alloca(4, 4);
            self.call_rt("str_cmp", vec![va, vb], Some((&slot, 4)));
            let order = self.load(Sc::I(32), &slot, 4);
            let condition = match op {
                BinOp::Lt => "slt",
                BinOp::Le => "sle",
                BinOp::Gt => "sgt",
                _ => "sge",
            };
            return Ok(self.value(format!("icmp {condition} i32 {order}, 0")));
        }
        let equal = self.eq_at(&pa, &pb, aa.min(ab), ty)?;
        Ok(if matches!(op, BinOp::Ne) { self.value(format!("xor i1 {equal}, true")) } else { equal })
    }
}
