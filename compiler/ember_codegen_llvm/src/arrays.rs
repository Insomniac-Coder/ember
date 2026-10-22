//! `[STD-15]` — the per-element-type helpers an `Array` method calls (the C backend's
//! `array_helper_N`), and the structural clones (`clone_parts_N`) a clone of a tuple, an enum or
//! a box runs. Each helper is a function of the module, written once per element type; a sort
//! is the runtime header's own (`EMBER_SORT_STABLE` / `EMBER_SORT_UNSTABLE`), instantiated in the
//! module's C shim.

use crate::func::{FnCx, Val};
use crate::types::Sc;
use crate::{Module, Result, global_name, unsupported};
use ember_types::{Ty, TyKind};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Helper {
    Pop,
    Remove,
    Drain,
    SwapRemove,
    Truncate,
    Extend,
    Clear,
    Clone,
}

impl Helper {
    fn name(self) -> &'static str {
        match self {
            Helper::Pop => "pop",
            Helper::Remove => "remove",
            Helper::Drain => "drain",
            Helper::SwapRemove => "swap_remove",
            Helper::Truncate => "truncate",
            Helper::Extend => "extend",
            Helper::Clear => "clear",
            Helper::Clone => "clone",
        }
    }
}

/// The helper `helper` for elements of `ty` (for `Pop`, the `Option[T]` it returns), writing it
/// the first time. Each takes the list's address first, then its operands, and the address of
/// its result last where it has one.
pub(crate) fn helper(m: &mut Module<'_>, helper: Helper, ty: Ty) -> Result<String> {
    let symbol = format!("ember.array.{}.{}", helper.name(), m.ty.table.mangled(ty));
    if m.helper_defined(&symbol) {
        return Ok(global_name(&symbol));
    }
    let params: &[&str] = match helper {
        Helper::Pop => &["ptr", "ptr"],
        Helper::Remove | Helper::SwapRemove => &["ptr", "i64", "ptr"],
        Helper::Drain => &["ptr", "i64", "i64", "ptr"],
        Helper::Truncate => &["ptr", "i64"],
        Helper::Extend => &["ptr", "ptr", "i64"],
        Helper::Clear => &["ptr"],
        Helper::Clone => &["ptr", "i64", "ptr"],
    };
    crate::glue::define_glue(m, &symbol, params, "", |cx| match helper {
        Helper::Pop => pop(cx, ty),
        Helper::Remove | Helper::SwapRemove => remove(cx, ty, helper == Helper::SwapRemove),
        Helper::Drain => drain(cx, ty),
        Helper::Truncate | Helper::Clear => truncate(cx, ty, helper == Helper::Clear),
        Helper::Extend => extend(cx, ty),
        Helper::Clone => clone(cx, ty),
    })?;
    Ok(global_name(&symbol))
}

fn size_word(size: u64) -> Val {
    Val::Imm(Sc::I(64), size.to_string())
}

/// `pop`: `None` from an empty list; else the last element moved out into `Some`.
fn pop(cx: &mut FnCx<'_, '_>, option: Ty) -> Result<()> {
    let (list, dest) = ("%p0", "%p1");
    let table = cx.m.ty.table;
    let TyKind::Enum(id) = *table.kind(option) else { return unsupported("`pop` that does not return an `Option`") };
    let def = table.enum_def(id).clone();
    let (none, some) = (0usize, 1usize);
    let elem = def.variants[some].fields[0].ty;
    let (size, elem_align) = cx.m.ty.size_align(elem);
    let (option_size, option_align) = cx.m.ty.size_align(option);
    let len_at = cx.gep(list, 8);
    let len = cx.load(Sc::I(64), &len_at, 8);
    let empty = cx.value(format!("icmp eq i64 {len}, 0"));
    let (nothing, take, done) = (cx.label(), cx.label(), cx.label());
    cx.emit(format!("br i1 {empty}, label %{nothing}, label %{take}"));
    cx.start(&nothing);
    let at = crate::func::PlaceRef { ptr: dest.to_string(), ty: option, variant: None, align: option_align };
    cx.zero(dest, option_align, option_size);
    cx.write_variant(&at, id, none)?;
    cx.emit(format!("br label %{done}"));
    cx.start(&take);
    let last = cx.value(format!("sub i64 {len}, 1"));
    cx.emit(format!("store i64 {last}, ptr {len_at}, align 8"));
    let buffer = cx.load(Sc::Ptr, list, 8);
    let offset = cx.value(format!("mul i64 {last}, {size}"));
    let element = cx.value(format!("getelementptr i8, ptr {buffer}, i64 {offset}"));
    cx.zero(dest, option_align, option_size);
    let payload_offset = if table.option_niche(id).is_some() { 0 } else { table.variant_field_offsets(id)[some][0] };
    let payload = cx.gep(dest, payload_offset);
    cx.copy(&payload, elem_align, &element, elem_align, size, false);
    if table.option_niche(id).is_none() {
        cx.write_variant(&at, id, some)?;
    }
    cx.emit(format!("br label %{done}"));
    cx.start(&done);
    Ok(())
}

/// `remove(i)`: the element moved out and those after it moved down; `swap_remove(i)`: the last
/// element moved into its place.
fn remove(cx: &mut FnCx<'_, '_>, elem: Ty, swap: bool) -> Result<()> {
    let (list, index, dest) = ("%p0", "%p1", "%p2");
    let (size, align) = cx.m.ty.size_align(elem);
    let len_at = cx.gep(list, 8);
    let len = cx.load(Sc::I(64), &len_at, 8);
    let buffer = cx.load(Sc::Ptr, list, 8);
    let offset = cx.value(format!("mul i64 {index}, {size}"));
    let at = cx.value(format!("getelementptr i8, ptr {buffer}, i64 {offset}"));
    cx.copy(dest, align, &at, align, size, false);
    let last = cx.value(format!("sub i64 {len}, 1"));
    cx.emit(format!("store i64 {last}, ptr {len_at}, align 8"));
    if size == 0 {
        return Ok(());
    }
    let intrinsic = "llvm.memmove.p0.p0.i64";
    cx.m.declare(intrinsic, || format!("declare void @{intrinsic}(ptr, ptr, i64, i1)"));
    if swap {
        let last_offset = cx.value(format!("mul i64 {last}, {size}"));
        let tail = cx.value(format!("getelementptr i8, ptr {buffer}, i64 {last_offset}"));
        cx.emit(format!("call void @{intrinsic}(ptr align {align} {at}, ptr align {align} {tail}, i64 {size}, i1 false)"));
    } else {
        let next = cx.gep(&at, size);
        let after = cx.value(format!("sub i64 {last}, {index}"));
        let bytes = cx.value(format!("mul i64 {after}, {size}"));
        cx.emit(format!("call void @{intrinsic}(ptr align {align} {at}, ptr align {align} {next}, i64 {bytes}, i1 false)"));
    }
    Ok(())
}

/// `drain(lo, hi)`: the elements moved out bitwise into a list of their own, those after moved
/// down. An empty range returns before any address is formed (an empty list's buffer is null).
fn drain(cx: &mut FnCx<'_, '_>, elem: Ty) -> Result<()> {
    let (list, lo, hi, dest) = ("%p0", "%p1", "%p2", "%p3");
    let (size, align) = cx.m.ty.size_align(elem);
    let none = cx.value(format!("icmp eq i64 {lo}, {hi}"));
    let (empty, take, done) = (cx.label(), cx.label(), cx.label());
    cx.emit(format!("br i1 {none}, label %{empty}, label %{take}"));
    cx.start(&empty);
    cx.zero(dest, 8, 24);
    cx.emit(format!("br label %{done}"));
    cx.start(&take);
    let buffer = cx.load(Sc::Ptr, list, 8);
    let offset = cx.value(format!("mul i64 {lo}, {size}"));
    let at = cx.value(format!("getelementptr i8, ptr {buffer}, i64 {offset}"));
    let count = cx.value(format!("sub i64 {hi}, {lo}"));
    if align > 16 {
        cx.call_rt("vec_from_elems_aligned", vec![size_word(size), size_word(align), Val::Imm(Sc::Ptr, at.clone()), Val::Imm(Sc::I(64), count.clone())], Some((dest, 8)));
    } else {
        cx.call_rt("vec_from_elems", vec![size_word(size), Val::Imm(Sc::Ptr, at.clone()), Val::Imm(Sc::I(64), count.clone())], Some((dest, 8)));
    }
    let len_at = cx.gep(list, 8);
    let len = cx.load(Sc::I(64), &len_at, 8);
    if size > 0 {
        let moved = cx.value(format!("mul i64 {count}, {size}"));
        let from = cx.value(format!("getelementptr i8, ptr {at}, i64 {moved}"));
        let after = cx.value(format!("sub i64 {len}, {hi}"));
        let bytes = cx.value(format!("mul i64 {after}, {size}"));
        let intrinsic = "llvm.memmove.p0.p0.i64";
        cx.m.declare(intrinsic, || format!("declare void @{intrinsic}(ptr, ptr, i64, i1)"));
        cx.emit(format!("call void @{intrinsic}(ptr align 1 {at}, ptr align 1 {from}, i64 {bytes}, i1 false)"));
    }
    let left = cx.value(format!("sub i64 {len}, {count}"));
    cx.emit(format!("store i64 {left}, ptr {len_at}, align 8"));
    cx.emit(format!("br label %{done}"));
    cx.start(&done);
    Ok(())
}

/// `truncate(n)` (and `clear`, `n` 0): the elements past `n` dropped.
fn truncate(cx: &mut FnCx<'_, '_>, elem: Ty, clear: bool) -> Result<()> {
    let list = "%p0";
    let n = if clear { "0".to_string() } else { "%p1".to_string() };
    let (size, align) = cx.m.ty.size_align(elem);
    let len_at = cx.gep(list, 8);
    let len = cx.load(Sc::I(64), &len_at, 8);
    let (work, done) = (cx.label(), cx.label());
    let shorter = cx.value(format!("icmp ult i64 {n}, {len}"));
    cx.emit(format!("br i1 {shorter}, label %{work}, label %{done}"));
    cx.start(&work);
    if cx.m.ty.table.needs_drop(elem) {
        let buffer = cx.load(Sc::Ptr, list, 8);
        let count = cx.value(format!("sub i64 {len}, {n}"));
        let n_owned = n.clone();
        cx.counted_loop(&count, |cx, index| {
            let at_index = cx.value(format!("add i64 {n_owned}, {index}"));
            let offset = cx.value(format!("mul i64 {at_index}, {size}"));
            let element = cx.value(format!("getelementptr i8, ptr {buffer}, i64 {offset}"));
            cx.drop_at(&element, align, elem)
        })?;
    }
    cx.emit(format!("store i64 {n}, ptr {len_at}, align 8"));
    cx.emit(format!("br label %{done}"));
    cx.start(&done);
    Ok(())
}

/// `extend(elems, count)`: room made (checked, `[HEAP-8]`), the elements copied after the last,
/// each copy's handles retained or each value cloned.
fn extend(cx: &mut FnCx<'_, '_>, elem: Ty) -> Result<()> {
    let (list, elems, count) = ("%p0", "%p1", "%p2");
    let (size, align) = cx.m.ty.size_align(elem);
    if align > 16 {
        cx.call_rt(
            "vec_reserve_more_aligned",
            vec![Val::Imm(Sc::Ptr, list.to_string()), size_word(size), size_word(align), Val::Imm(Sc::I(64), count.to_string())],
            None,
        );
    } else {
        cx.call_rt("vec_reserve_more_inline", vec![Val::Imm(Sc::Ptr, list.to_string()), size_word(size), Val::Imm(Sc::I(64), count.to_string())], None);
    }
    let len_at = cx.gep(list, 8);
    let len = cx.load(Sc::I(64), &len_at, 8);
    if size > 0 {
        let buffer = cx.load(Sc::Ptr, list, 8);
        let offset = cx.value(format!("mul i64 {len}, {size}"));
        let to = cx.value(format!("getelementptr i8, ptr {buffer}, i64 {offset}"));
        let bytes = cx.value(format!("mul i64 {count}, {size}"));
        let intrinsic = "llvm.memcpy.p0.p0.i64";
        cx.m.declare(intrinsic, || format!("declare void @{intrinsic}(ptr, ptr, i64, i1)"));
        cx.emit(format!("call void @{intrinsic}(ptr align 1 {to}, ptr align 1 {elems}, i64 {bytes}, i1 false)"));
        copy_parts(cx, &to, elems, count, elem)?;
    }
    let grown = cx.value(format!("add i64 {len}, {count}"));
    cx.emit(format!("store i64 {grown}, ptr {len_at}, align 8"));
    Ok(())
}

/// `clone`: a list of exactly the elements, each copy's handles retained or each value cloned.
fn clone(cx: &mut FnCx<'_, '_>, elem: Ty) -> Result<()> {
    let (elems, count, dest) = ("%p0", "%p1", "%p2");
    let (size, align) = cx.m.ty.size_align(elem);
    if align > 16 {
        cx.call_rt("vec_from_elems_aligned", vec![size_word(size), size_word(align), Val::Imm(Sc::Ptr, elems.to_string()), Val::Imm(Sc::I(64), count.to_string())], Some((dest, 8)));
    } else {
        cx.call_rt("vec_from_elems", vec![size_word(size), Val::Imm(Sc::Ptr, elems.to_string()), Val::Imm(Sc::I(64), count.to_string())], Some((dest, 8)));
    }
    if size > 0 {
        let buffer = cx.load(Sc::Ptr, dest, 8);
        copy_parts(cx, &buffer, elems, count, elem)?;
    }
    Ok(())
}

/// After a bitwise copy of `count` elements from `from` to `to`: a `Copy` element's handles
/// retained, any other element cloned in place from its source.
fn copy_parts(cx: &mut FnCx<'_, '_>, to: &str, from: &str, count: &str, elem: Ty) -> Result<()> {
    let table = cx.m.ty.table;
    let copy = table.is_copy(elem);
    if copy && !cx.holds_counts(elem) {
        return Ok(());
    }
    let (size, align) = cx.m.ty.size_align(elem);
    let (to, from) = (to.to_string(), from.to_string());
    cx.counted_loop(count, |cx, index| {
        let offset = cx.value(format!("mul i64 {index}, {size}"));
        let dest = cx.value(format!("getelementptr i8, ptr {to}, i64 {offset}"));
        if copy {
            cx.retain_lines(&dest, align, elem)
        } else {
            let source = cx.value(format!("getelementptr i8, ptr {from}, i64 {offset}"));
            cx.clone_part(&dest, &source, align, elem)
        }
    })
}

impl<'a> FnCx<'_, 'a> {
    /// One value cloned from `source` into `dest` (`clone_part_lines`): a `Copy` one copied and its
    /// handles retained, a list through its clone helper, a type with a `clone` method through
    /// it, a tuple, an enum or a box through its structural clone.
    pub(crate) fn clone_part(&mut self, dest: &str, source: &str, align: u64, ty: Ty) -> Result<()> {
        let table = self.m.ty.table;
        let (size, _) = self.m.ty.size_align(ty);
        if table.is_copy(ty) {
            self.copy(dest, align, source, align, size, false);
            return self.retain_lines(dest, align, ty);
        }
        if let TyKind::Vec { elem, .. } = *table.kind(ty) {
            let function = helper(self.m, Helper::Clone, elem)?;
            let buffer = self.load(Sc::Ptr, source, 8);
            let len_at = self.gep(source, 8);
            let len = self.load(Sc::I(64), &len_at, 8);
            self.emit(format!("call void {function}(ptr {buffer}, i64 {len}, ptr {dest})"));
            return Ok(());
        }
        if let Some((symbol, by_address)) = self.m.clone_fns.get(&ty).cloned() {
            let argument = if by_address { Val::Imm(Sc::Ptr, source.to_string()) } else { self.value_at(source, align, ty) };
            return self.call_body(&symbol, vec![argument], Some((dest, align)));
        }
        let structural = matches!(table.kind(ty), TyKind::Tuple(_) | TyKind::Enum(_))
            || matches!(table.kind(ty), TyKind::Struct(id) if table.compiler_box_inner(*id).is_some());
        if !structural {
            return unsupported(format!("a clone of `{}`", table.display(ty)));
        }
        let function = clone_parts_fn(self.m, ty)?;
        self.emit(format!("call void {function}(ptr {source}, ptr {dest})"));
        Ok(())
    }
}

/// `void clone_parts(const T* src, T* dst)` for a tuple, an enum or a box.
pub(crate) fn clone_parts_fn(m: &mut Module<'_>, ty: Ty) -> Result<String> {
    let symbol = format!("ember.clone_parts.{}", m.ty.table.mangled(ty));
    if m.helper_defined(&symbol) {
        return Ok(global_name(&symbol));
    }
    crate::glue::define_glue(m, &symbol, &["ptr", "ptr"], "", |cx| {
        let (src, dst) = ("%p0", "%p1");
        let table = cx.m.ty.table;
        let (size, align) = cx.m.ty.size_align(ty);
        match table.kind(ty).clone() {
            TyKind::Tuple(items) => {
                let offsets = table.layout(ty).field_offsets;
                for (item, offset) in items.into_iter().zip(offsets) {
                    let (from, to) = (cx.gep(src, offset), cx.gep(dst, offset));
                    let item_align = align.min(crate::types::offset_align(offset));
                    cx.clone_part(&to, &from, item_align, item)?;
                }
            }
            TyKind::Enum(id) => {
                let def = table.enum_def(id).clone();
                let niche = table.option_niche(id);
                let offsets = table.variant_field_offsets(id);
                let at_src = crate::func::PlaceRef { ptr: src.to_string(), ty, variant: None, align };
                let (tag, bits) = cx.discriminant_at(&at_src)?;
                let done = cx.label();
                let mut arms = Vec::new();
                let mut labels = Vec::new();
                let mut seen = std::collections::BTreeSet::new();
                for variant in &def.variants {
                    let label = cx.label();
                    if seen.insert(variant.discriminant) {
                        arms.push(format!("i{bits} {}, label %{label}", crate::func::int_literal(variant.discriminant as u128, bits)));
                    }
                    labels.push(label);
                }
                cx.emit(format!("switch i{bits} {tag}, label %{done} [ {} ]", arms.join(" ")));
                let at_dst = crate::func::PlaceRef { ptr: dst.to_string(), ty, variant: None, align };
                for (index, (variant, label)) in def.variants.iter().zip(&labels).enumerate() {
                    cx.start(label);
                    cx.zero(dst, align, size);
                    match niche {
                        None => cx.write_variant(&at_dst, id, index)?,
                        Some(_) if variant.fields.is_empty() => cx.write_variant(&at_dst, id, index)?,
                        Some(_) => {}
                    }
                    for (field_index, field) in variant.fields.iter().enumerate() {
                        let offset = if niche.is_some() { 0 } else { offsets[index][field_index] };
                        let (from, to) = (cx.gep(src, offset), cx.gep(dst, offset));
                        let field_align = align.min(crate::types::offset_align(offset)).min(cx.m.ty.size_align(field.ty).1);
                        cx.clone_part(&to, &from, field_align, field.ty)?;
                    }
                    cx.emit(format!("br label %{done}"));
                }
                cx.start(&done);
            }
            // ODR-237 — a new box holding a clone of the value.
            TyKind::Struct(id) => {
                let inner = table.compiler_box_inner(id).expect("a box");
                // A `Box[dyn I]`'s through the copy entry of its table (after `drop`, `size`,
                // `align` and `access`), which keeps the table.
                if matches!(table.kind(inner), TyKind::Dyn { .. }) {
                    let data = cx.load(Sc::Ptr, src, 8);
                    let vtable_at = cx.gep(src, 8);
                    let vtable = cx.load(Sc::Ptr, &vtable_at, 8);
                    let clone_at = cx.gep(&vtable, 32);
                    let clone = cx.load(Sc::Ptr, &clone_at, 8);
                    let copy = cx.value(format!("call ptr {clone}(ptr {data})"));
                    cx.emit(format!("store ptr {copy}, ptr {dst}, align 8"));
                    let dst_vtable = cx.gep(dst, 8);
                    cx.emit(format!("store ptr {vtable}, ptr {dst_vtable}, align 8"));
                    return Ok(());
                }
                let (inner_size, inner_align) = cx.m.ty.size_align(inner);
                let slot = cx.alloca(8, 8);
                cx.call_rt("alloc", vec![size_word(inner_size.max(1)), size_word(inner_align)], Some((&slot, 8)));
                let block = cx.load(Sc::Ptr, &slot, 8);
                cx.emit(format!("store ptr {block}, ptr {dst}, align 8"));
                let source = cx.load(Sc::Ptr, src, 8);
                cx.clone_part(&block, &source, inner_align, inner)?;
            }
            other => return unsupported(format!("a structural clone of {other:?}")),
        }
        Ok(())
    })?;
    Ok(global_name(&symbol))
}

/// `[STD-15]` — `sort` for elements of `ty`, the runtime header's sort instantiated in the C shim:
/// introsort where equal elements are the same value (numbers, `bool`, `char`), else the stable
/// merge sort. `None` where the element has no C spelling this needs.
pub(crate) fn sort_fn(m: &mut Module<'_>, ty: Ty) -> Result<String> {
    let ty = m.ty.through_range(ty);
    // Elements of no bytes are all equal: in order already.
    if m.ty.is_zst(ty) {
        let symbol = "ember.array.sort.void";
        if !m.helper_defined(symbol) {
            crate::glue::define_glue(m, symbol, &["ptr"], "", |_| Ok(()))?;
        }
        return Ok(global_name(symbol));
    }
    let rt = ember_branding::RUNTIME_PREFIX;
    let (c_type, less, identical): (String, String, bool) = match m.ty.kind(ty).clone() {
        TyKind::Float(ember_types::FloatTy::F64) => ("double".into(), format!("{rt}total_lt_f64(*(const double*)a, *(const double*)b)"), true),
        TyKind::Float(ember_types::FloatTy::F32) => ("float".into(), format!("{rt}total_lt_f32(*(const float*)a, *(const float*)b)"), true),
        TyKind::Float(ember_types::FloatTy::F16) => ("uint16_t".into(), format!("{rt}total_lt_f16(*(const uint16_t*)a, *(const uint16_t*)b)"), true),
        TyKind::Str => (format!("{rt}str"), format!("{rt}str_cmp(*(const {rt}str*)a, *(const {rt}str*)b) < 0"), false),
        TyKind::Vec { text: true, .. } => (
            format!("{rt}vec"),
            format!("{rt}str_cmp({rt}vec_as_str((const {rt}vec*)a), {rt}vec_as_str((const {rt}vec*)b)) < 0"),
            false,
        ),
        TyKind::Int(ember_types::IntTy::I128) => (format!("{rt}i128"), format!("{rt}i128_lt(*(const {rt}i128*)a, *(const {rt}i128*)b)"), true),
        TyKind::Uint(ember_types::UintTy::U128) => (format!("{rt}u128"), format!("{rt}u128_lt(*(const {rt}u128*)a, *(const {rt}u128*)b)"), true),
        kind => {
            let c = match (kind, m.ty.repr(ty)) {
                (TyKind::Bool, _) => "bool",
                (TyKind::Char, _) => "uint32_t",
                (TyKind::Int(_) | TyKind::Enum(_), crate::types::Repr::Scalar(Sc::I(8))) => "int8_t",
                (TyKind::Int(_) | TyKind::Enum(_), crate::types::Repr::Scalar(Sc::I(16))) => "int16_t",
                (TyKind::Int(_) | TyKind::Enum(_), crate::types::Repr::Scalar(Sc::I(32))) => "int32_t",
                (TyKind::Int(_) | TyKind::Enum(_), crate::types::Repr::Scalar(Sc::I(64))) => "int64_t",
                (TyKind::Uint(_), crate::types::Repr::Scalar(Sc::I(8))) => "uint8_t",
                (TyKind::Uint(_), crate::types::Repr::Scalar(Sc::I(16))) => "uint16_t",
                (TyKind::Uint(_), crate::types::Repr::Scalar(Sc::I(32))) => "uint32_t",
                (TyKind::Uint(_), crate::types::Repr::Scalar(Sc::I(64))) => "uint64_t",
                _ => return unsupported(format!("a sort of `{}`", m.ty.table.display(ty))),
            };
            // An enum over an unsigned representation sorts as its unsigned discriminant.
            let unsigned_enum = matches!(m.ty.kind(ty), TyKind::Enum(id) if !matches!(m.ty.kind(m.ty.table.enum_def(*id).repr), TyKind::Int(_)));
            let c = if unsigned_enum { c.replace("int", "uint") } else { c.to_string() };
            (c.clone(), format!("*(const {c}*)a < *(const {c}*)b"), !matches!(m.ty.kind(ty), TyKind::Enum(_)))
        }
    };
    let symbol = ember_branding::runtime(&format!("ll_sort_{}", m.ty.table.mangled(ty)));
    let prefix = ember_branding::SYMBOL_PREFIX.to_uppercase();
    let kind = if identical { "UNSTABLE" } else { "STABLE" };
    let source = format!(
        "#define {symbol}_less(a, b) ({less})\n{prefix}_SORT_{kind}({symbol}_impl, {c_type}, {symbol}_less)\nvoid {symbol}({rt}vec* v) {{ {symbol}_impl(v); }}\n"
    );
    m.shim_functions.entry(symbol.clone()).or_insert(source);
    let name = global_name(&symbol);
    m.declare(&symbol, || format!("declare void {name}(ptr) nounwind"));
    Ok(name)
}
