//! `[TYP-36]`, `[TYP-39]`, `[STR-5]` — printing and formatting: each aggregate's `Debug` text
//! written by a function of its own (`ember.fmt.<type>`, the C backend's `fmt_N`), its parts by
//! the runtime's formatters; a written `Debug` through its writer (ADR-186).

use crate::func::{FnCx, Val};
use crate::glue::Pending;
use crate::types::{Repr, Sc};
use crate::{Module, Result, global_name, unsupported};
use ember_mir::FormatSpec;
use ember_types::{FloatTy, IntTy, Ty, TyKind, UintTy};

/// Whether `ty` prints through a generated function: a list, a view, a fixed array, a tuple, a
/// struct, an enum or a class handle.
pub(crate) fn is_display_aggregate(m: &Module<'_>, ty: Ty) -> bool {
    match m.ty.kind(ty) {
        TyKind::Vec { text, .. } => !text,
        TyKind::Span { .. } | TyKind::Array { .. } | TyKind::Tuple(_) | TyKind::Enum(_) | TyKind::Struct(_) => true,
        TyKind::Class(_) | TyKind::ClassInterface(_) => true,
        _ => false,
    }
}

/// The runtime formatter (and printer) suffix for a scalar or text of `ty`.
pub(crate) fn suffix(m: &Module<'_>, ty: Ty) -> &'static str {
    let ty = m.ty.through_range(ty);
    match m.ty.kind(ty) {
        TyKind::Bool => "bool",
        TyKind::Char => "char",
        TyKind::Int(IntTy::I128) => "i128",
        TyKind::Int(IntTy::I256) => "i256",
        TyKind::Uint(UintTy::U128) => "u128",
        TyKind::Uint(UintTy::U256) => "u256",
        TyKind::Int(_) => "i64",
        TyKind::Uint(_) => "u64",
        TyKind::Float(FloatTy::F64) => "f64",
        TyKind::Float(FloatTy::F16) => "f16",
        TyKind::Float(_) => "f32",
        _ => "str",
    }
}

/// The `Debug` function of `ty`, asking for it.
pub(crate) fn fmt_fn(m: &mut Module<'_>, ty: Ty) -> String {
    m.request(Pending::Fmt(ty));
    global_name(&format!("ember.fmt.{}", m.ty.table.mangled(ty)))
}

/// `void fmt(ember_vec* out, const T* v)`: `ty`'s `Debug` text appended to `out`.
pub(crate) fn emit_fmt_fn(m: &mut Module<'_>, ty: Ty) -> Result<()> {
    let symbol = format!("ember.fmt.{}", m.ty.table.mangled(ty));
    crate::glue::define_glue(m, &symbol, &["ptr", "ptr"], "", |cx| {
        let (out, v) = ("%p0".to_string(), "%p1".to_string());
        let table = cx.m.ty.table;
        let (_, align) = cx.m.ty.size_align(ty);
        if let Some(writer) = table.debug_writer(ty).map(str::to_string) {
            // ADR-186, `[STR-5]` — the program's own `Debug`: the value as its borrowed
            // parameter takes it, and the text being built.
            let value = if table.passed_by_address(ty) { Val::Imm(Sc::Ptr, v.clone()) } else { cx.value_at(&v, align, ty) };
            return cx.call_body(&writer, vec![value, Val::Imm(Sc::Ptr, out.clone())], None);
        }
        match table.kind(ty).clone() {
            TyKind::Vec { elem, .. } | TyKind::Span { elem, .. } => {
                let buffer = cx.load(Sc::Ptr, &v, 8);
                let len_at = cx.gep(&v, 8);
                let len = cx.load(Sc::I(64), &len_at, 8);
                cx.text(&out, "[");
                cx.list_items(&out, &buffer, &len, elem)?;
                cx.text(&out, "]");
            }
            TyKind::Array { elem, len } => {
                cx.text(&out, "[");
                cx.list_items(&out, &v, &len.to_string(), elem)?;
                cx.text(&out, "]");
            }
            TyKind::Class(_) | TyKind::ClassInterface(_) => {
                let handle = cx.load(Sc::Ptr, &v, 8);
                cx.call_rt("fmt_handle", vec![Val::Imm(Sc::Ptr, out.clone()), Val::Imm(Sc::Ptr, handle)], None);
            }
            TyKind::Tuple(items) => {
                let offsets = table.layout(ty).field_offsets;
                cx.text(&out, "(");
                for (index, (item, offset)) in items.iter().zip(offsets).enumerate() {
                    if index > 0 {
                        cx.text(&out, ", ");
                    }
                    let at = cx.gep(&v, offset);
                    cx.debug_into(&out, &at, align.min(crate::types::offset_align(offset)), *item)?;
                }
                // Python writes a one-element tuple `(1,)`.
                if items.len() == 1 {
                    cx.text(&out, ",");
                }
                cx.text(&out, ")");
            }
            TyKind::Struct(id)
                if matches!(
                    table.struct_def(id).origin.as_ref().map(|(name, _)| name.as_str()),
                    Some("std.collections.Map" | "std.collections.Set")
                ) =>
            {
                cx.map_body(&out, &v, ty)?;
            }
            // `[STR-5]`, `[TYP-36]` — `Point(x=1, y=2)`.
            TyKind::Struct(id) => {
                let def = table.struct_def(id).clone();
                let packed = table.layout_attrs(ty).packed;
                let offsets = table.layout(ty).field_offsets;
                cx.text(&out, &format!("{}(", debug_type_name(def.name.as_str(), def.origin.as_ref())));
                for (index, (field, offset)) in def.fields.iter().zip(offsets).enumerate() {
                    if index > 0 {
                        cx.text(&out, ", ");
                    }
                    cx.text(&out, &format!("{}=", field.name));
                    let at = cx.gep(&v, offset);
                    let field_align = if packed { 1 } else { align.min(crate::types::offset_align(offset)) };
                    cx.debug_into(&out, &at, field_align, field.ty)?;
                }
                cx.text(&out, ")");
            }
            TyKind::Enum(id) => {
                let def = table.enum_def(id).clone();
                // `Option` and `Result` show `Some(1)`, `Err('x')`; a program's enum its name.
                let compiler_known = def.name.as_str().starts_with("Option_") || def.name.as_str().starts_with("Result_");
                let prefix = if compiler_known { String::new() } else { format!("{}.", debug_type_name(def.name.as_str(), def.origin.as_ref())) };
                let packed = table.layout_attrs(ty).packed;
                let niche = table.option_niche(id).is_some();
                let offsets = table.variant_field_offsets(id);
                let at = crate::func::PlaceRef { ptr: v.clone(), ty, variant: None, align };
                let (tag, bits) = cx.discriminant_at(&at)?;
                let done = cx.label();
                let other = cx.label();
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
                cx.emit(format!("switch i{bits} {tag}, label %{other} [ {} ]", arms.join(" ")));
                for (index, (variant, label)) in def.variants.iter().zip(&labels).enumerate() {
                    cx.start(label);
                    cx.text(&out, &format!("{prefix}{}", variant.name));
                    if !variant.fields.is_empty() {
                        cx.text(&out, "(");
                        for (field_index, field) in variant.fields.iter().enumerate() {
                            if field_index > 0 {
                                cx.text(&out, ", ");
                            }
                            // A named payload field shows its name; a positional one does not.
                            if !field.name.as_str().starts_with('_') {
                                cx.text(&out, &format!("{}=", field.name));
                            }
                            let offset = if niche { 0 } else { offsets[index][field_index] };
                            let field_at = cx.gep(&v, offset);
                            let field_align = if packed { 1 } else { align.min(crate::types::offset_align(offset)) };
                            cx.debug_into(&out, &field_at, field_align, field.ty)?;
                        }
                        cx.text(&out, ")");
                    }
                    cx.emit(format!("br label %{done}"));
                }
                cx.start(&other);
                // `[FFI-8]` — a C enum's value no variant names: `Color(9)` (ODR-137).
                if def.open {
                    cx.text(&out, &format!("{}(", debug_type_name(def.name.as_str(), def.origin.as_ref())));
                    cx.debug_into(&out, &v, align, def.repr)?;
                    cx.text(&out, ")");
                }
                cx.emit(format!("br label %{done}"));
                cx.start(&done);
            }
            other => return unsupported(format!("a `Debug` function for {other:?}")),
        }
        Ok(())
    })
}

/// The name `Debug` shows for a struct or an enum: its own, without its module or its arguments.
fn debug_type_name(name: &str, origin: Option<&(ember_span::Symbol, Vec<Ty>)>) -> String {
    let name = origin.map(|(name, _)| name.as_str()).unwrap_or(name);
    name.rsplit('.').next().unwrap_or(name).to_string()
}

impl<'a> FnCx<'_, 'a> {
    /// The value of `ty` at `at`.
    pub(crate) fn value_at(&mut self, at: &str, align: u64, ty: Ty) -> Val {
        match self.m.ty.repr(ty) {
            Repr::Scalar(sc) => Val::Imm(sc, self.load(sc, at, align)),
            Repr::Zst => Val::Zst,
            Repr::Mem { .. } => Val::Mem(at.to_string(), align),
        }
    }

    /// `text` appended to the text at `out`.
    pub(crate) fn text(&mut self, out: &str, text: &str) {
        let bytes = self.m.bytes(text.as_bytes());
        self.call_rt(
            "vec_extend",
            vec![Val::Imm(Sc::Ptr, out.to_string()), Val::Imm(Sc::Ptr, bytes), Val::Imm(Sc::I(64), text.len().to_string())],
            None,
        );
    }

    /// The elements of a buffer, `", "` between them.
    fn list_items(&mut self, out: &str, base: &str, len: &str, elem: Ty) -> Result<()> {
        let (size, elem_align) = self.m.ty.size_align(elem);
        let out = out.to_string();
        let base = base.to_string();
        self.counted_loop(len, |cx, index| {
            let first = cx.value(format!("icmp eq i64 {index}, 0"));
            let (separate, item) = (cx.label(), cx.label());
            cx.emit(format!("br i1 {first}, label %{item}, label %{separate}"));
            cx.start(&separate);
            cx.text(&out, ", ");
            cx.emit(format!("br label %{item}"));
            cx.start(&item);
            let offset = cx.value(format!("mul i64 {index}, {size}"));
            let at = cx.value(format!("getelementptr i8, ptr {base}, i64 {offset}"));
            cx.debug_into(&out, &at, elem_align, elem)
        })
    }

    /// `[TYP-39]` — the `Debug` text of the value of `ty` at `v` appended to the text at `out`:
    /// text quoted, numbers and `bool` as they display, aggregates through their function.
    pub(crate) fn debug_into(&mut self, out: &str, v: &str, align: u64, ty: Ty) -> Result<()> {
        let table = self.m.ty.table;
        let (_, natural) = self.m.ty.size_align(ty);
        // `[LAY-2]` — a value in a `@packed` place is read from an aligned copy.
        if align < natural && matches!(self.m.ty.repr(ty), Repr::Mem { .. } | Repr::Scalar(_)) {
            let (size, _) = self.m.ty.size_align(ty);
            let copy = self.alloca(size, natural);
            self.copy(&copy, natural, v, align, size, false);
            return self.debug_into(out, &copy, natural, ty);
        }
        // `[TYP-39]` (ADR-191) — a `dyn` value is written by its writer.
        if table.debug_writer(ty).is_some() && !is_display_aggregate(self.m, ty) {
            let function = fmt_fn(self.m, ty);
            self.emit(format!("call void {function}(ptr {out}, ptr {v})"));
            return Ok(());
        }
        if let Some((payload, inner)) = self.printed_payload(v, ty) {
            // ODR-109 — a `Cell`'s value shown through a written `Debug` is a copy, retained.
            if table.debug_writer(inner).is_some()
                && let TyKind::Struct(id) = *self.m.ty.kind(ty)
                && table.compiler_box_inner(id).is_none()
            {
                let (size, inner_align) = self.m.ty.size_align(inner);
                let held = self.alloca(size, inner_align);
                self.copy(&held, inner_align, &payload, inner_align, size, false);
                self.retain_lines(&held, inner_align, inner)?;
                self.debug_into(out, &held, inner_align, inner)?;
                return self.drop_at(&held, inner_align, inner);
            }
            let (_, inner_align) = self.m.ty.size_align(inner);
            return self.debug_into(out, &payload, inner_align, inner);
        }
        // D-227 — a reference inside an aggregate prints what it points to.
        if let TyKind::Ref { inner, .. } = *self.m.ty.kind(ty)
            && !matches!(self.m.ty.kind(inner), TyKind::Dyn { .. })
        {
            let pointee = self.load(Sc::Ptr, v, align.min(8));
            let (_, inner_align) = self.m.ty.size_align(inner);
            return self.debug_into(out, &pointee, inner_align, inner);
        }
        if let Some(text) = self.text_at(v, ty) {
            self.call_rt("fmt_repr_str", vec![Val::Imm(Sc::Ptr, out.to_string()), text], None);
            return Ok(());
        }
        match self.m.ty.kind(ty).clone() {
            // `[TYP-36]` — `void` shows as `()`.
            TyKind::Void => {
                self.text(out, "()");
                Ok(())
            }
            TyKind::Char => {
                let c = self.load(Sc::I(32), v, align.min(4));
                self.call_rt("fmt_repr_char", vec![Val::Imm(Sc::Ptr, out.to_string()), Val::Imm(Sc::I(32), c)], None);
                Ok(())
            }
            _ if is_display_aggregate(self.m, ty) => {
                let function = fmt_fn(self.m, ty);
                self.emit(format!("call void {function}(ptr {out}, ptr {v})"));
                Ok(())
            }
            _ => {
                let value = self.value_at(v, align, ty);
                self.format_scalar(out, value, ty, None)
            }
        }
    }

    /// ODR-094 — the payload a compiler-known wrapper prints as: a `Box`'s value (behind its
    /// pointer), a `Cell`'s or a `some` wrapper's one field.
    fn printed_payload(&mut self, v: &str, ty: Ty) -> Option<(String, Ty)> {
        let table = self.m.ty.table;
        let inner = table.printed_as(ty)?;
        let TyKind::Struct(id) = *table.kind(ty) else { return None };
        if table.compiler_box_inner(id).is_some() {
            return Some((self.load(Sc::Ptr, v, 8), inner));
        }
        let offset = table.layout(ty).field_offsets.first().copied().unwrap_or(0);
        Some((self.gep(v, offset), inner))
    }

    /// A `str` or a `String` at `v` as a `str` value.
    fn text_at(&mut self, v: &str, ty: Ty) -> Option<Val> {
        match self.m.ty.kind(ty) {
            TyKind::Str | TyKind::Vec { text: true, .. } => {
                let ptr = self.load(Sc::Ptr, v, 8);
                let len_at = self.gep(v, 8);
                let len = self.load(Sc::I(64), &len_at, 8);
                let view = self.alloca(16, 8);
                self.emit(format!("store ptr {ptr}, ptr {view}, align 8"));
                let view_len = self.gep(&view, 8);
                self.emit(format!("store i64 {len}, ptr {view_len}, align 8"));
                Some(Val::Mem(view, 8))
            }
            _ => None,
        }
    }

    /// A scalar or text appended by the runtime's formatter (`fmt_<suffix>`, or with a spec
    /// `fmt_spec_<suffix>`).
    fn format_scalar(&mut self, out: &str, value: Val, ty: Ty, spec: Option<&FormatSpec>) -> Result<()> {
        let suffix = suffix(self.m, ty);
        let value = self.formatter_value(value, ty, suffix)?;
        let mut values = vec![Val::Imm(Sc::Ptr, out.to_string()), value];
        let name = match spec {
            Some(spec) => {
                values.push(self.spec_value(spec));
                format!("fmt_spec_{suffix}")
            }
            None => format!("fmt_{suffix}"),
        };
        self.call_rt(&name, values, None);
        Ok(())
    }

    /// A value as its formatter or printer takes it: an integer widened to 64 bits, a 256-bit
    /// count in memory.
    pub(crate) fn formatter_value(&mut self, value: Val, ty: Ty, suffix: &str) -> Result<Val> {
        let signed = self.m.ty.is_signed(ty);
        Ok(match (suffix, value) {
            ("i64" | "u64", Val::Imm(Sc::I(bits), v)) => Val::Imm(Sc::I(64), self.int_convert(&v, bits, 64, signed)),
            ("i256" | "u256", Val::Imm(sc, v)) => {
                let (slot, align) = self.spill(&Val::Imm(sc, v), 32, 16);
                Val::Mem(slot, align)
            }
            (_, value) => value,
        })
    }

    /// `ember_fmt_spec` as the spec's literal (`fmt_spec_literal`).
    fn spec_value(&mut self, spec: &FormatSpec) -> Val {
        let letter = |c: Option<char>| c.map_or(0, |c| c as u32);
        let slot = self.alloca(24, 4);
        let fields: [(u64, &str, u32); 9] = [
            (0, "i32", spec.fill as u32),
            (4, "i8", letter(spec.align)),
            (5, "i8", letter(spec.sign)),
            (6, "i8", spec.alternate as u32),
            (7, "i8", spec.zero as u32),
            (8, "i32", spec.width),
            (12, "i8", letter(spec.grouping)),
            (16, "i32", spec.precision.map_or(u32::MAX, |p| p)),
            (20, "i8", letter(spec.kind)),
        ];
        self.zero(&slot, 4, 24);
        for (offset, ty, value) in fields {
            let at = self.gep(&slot, offset);
            let value = if ty == "i8" { (value & 0xff) as i8 as i64 } else { value as i32 as i64 };
            let align = crate::types::offset_align(offset).min(4);
            self.emit(format!("store {ty} {value}, ptr {at}, align {align}"));
        }
        Val::Mem(slot, 4)
    }

    /// `[TYP-36]` — a unit-only enum's `Display`, its variant's name (an open one's number when no
    /// variant names it); `None` for any other type.
    fn unit_display(&mut self, v: &str, align: u64, ty: Ty) -> Result<Option<Val>> {
        let table = self.m.ty.table;
        if matches!(table.kind(ty), TyKind::Void) {
            return Ok(Some(self.str_bytes(b"()")));
        }
        let TyKind::Enum(id) = *table.kind(ty) else { return Ok(None) };
        let def = table.enum_def(id).clone();
        if !def.is_unit_only() {
            return Ok(None);
        }
        let at = crate::func::PlaceRef { ptr: v.to_string(), ty, variant: None, align };
        let (tag, bits) = self.discriminant_at(&at)?;
        let fallback = if def.open {
            let signed = matches!(table.kind(def.repr), TyKind::Int(_));
            let wide = self.int_convert(&tag, bits, 64, signed);
            let slot = self.alloca(16, 8);
            self.call_rt(if signed { "int_text" } else { "uint_text" }, vec![Val::Imm(Sc::I(64), wide)], Some((&slot, 8)));
            let ptr = self.load(Sc::Ptr, &slot, 8);
            let len_at = self.gep(&slot, 8);
            let len = self.load(Sc::I(64), &len_at, 8);
            (ptr, len)
        } else {
            (self.m.bytes(b""), "0".to_string())
        };
        let (mut ptr, mut len) = fallback;
        for variant in def.variants.iter().rev() {
            let name = variant.name.as_str().as_bytes().to_vec();
            let is = self.value(format!("icmp eq i{bits} {tag}, {}", crate::func::int_literal(variant.discriminant as u128, bits)));
            let text = self.m.bytes(&name);
            ptr = self.value(format!("select i1 {is}, ptr {text}, ptr {ptr}"));
            len = self.value(format!("select i1 {is}, i64 {}, i64 {len}", name.len()));
        }
        let slot = self.alloca(16, 8);
        self.emit(format!("store ptr {ptr}, ptr {slot}, align 8"));
        let len_at = self.gep(&slot, 8);
        self.emit(format!("store i64 {len}, ptr {len_at}, align 8"));
        Ok(Some(Val::Mem(slot, 8)))
    }

    fn str_bytes(&mut self, bytes: &[u8]) -> Val {
        let ptr = self.m.bytes(bytes);
        let slot = self.alloca(16, 8);
        self.emit(format!("store ptr {ptr}, ptr {slot}, align 8"));
        let len = self.gep(&slot, 8);
        self.emit(format!("store i64 {}, ptr {len}, align 8", bytes.len()));
        Val::Mem(slot, 8)
    }

    /// ODR-094 — the value as it prints: a `Box[T]` or a `Cell[T]` as the `T` it holds.
    fn unwrap_printed(&mut self, mut at: String, mut align: u64, mut ty: Ty) -> (String, u64, Ty) {
        while let Some((payload, inner)) = self.printed_payload(&at, ty) {
            at = payload;
            ty = inner;
            align = self.m.ty.size_align(inner).1;
        }
        (at, align, ty)
    }

    /// `[STD-9]` — `print` and its kin of `value` of `ty`.
    pub(crate) fn print_value(&mut self, printer: &str, value: Val, ty: Ty) -> Result<()> {
        let (size, align) = self.m.ty.size_align(ty);
        let (at, at_align) = self.spill(&value, size, align);
        let (at, at_align, ty) = self.unwrap_printed(at, at_align, ty);
        let table = self.m.ty.table;
        let through = self.m.ty.through_range(ty);
        if let Some(name) = self.unit_display(&at, at_align, through)? {
            self.call_rt(&format!("{printer}_str"), vec![name], None);
            return Ok(());
        }
        if let Some(text) = self.text_at(&at, ty) {
            self.call_rt(&format!("{printer}_str"), vec![text], None);
            return Ok(());
        }
        if is_display_aggregate(self.m, ty) {
            // Its text, built in a buffer and printed whole.
            let buffer = self.alloca(24, 8);
            self.zero(&buffer, 8, 24);
            let function = fmt_fn(self.m, ty);
            self.emit(format!("call void {function}(ptr {buffer}, ptr {at})"));
            let text = self.text_at(&buffer, table.find(&TyKind::Str).unwrap_or(ty)).expect("a buffer is text");
            let to_stderr = printer.starts_with('e');
            self.call_rt(if to_stderr { "eprint_str" } else { "print_str" }, vec![text], None);
            self.call_rt("vec_free", vec![Val::Imm(Sc::Ptr, buffer), Val::Imm(Sc::I(64), "1".to_string())], None);
            return Ok(());
        }
        let suffix = suffix(self.m, through);
        if suffix == "str" && !matches!(table.kind(through), TyKind::Str) {
            return unsupported(format!("printing `{}`", table.display(ty)));
        }
        let value = self.value_at(&at, at_align, through);
        let value = self.formatter_value(value, through, suffix)?;
        self.call_rt(&format!("{printer}_{suffix}"), vec![value], None);
        Ok(())
    }

    /// `[LEX-19]`, `[TYP-39]` — an f-string's piece appended to the text at `out` (`Format`), with
    /// its spec (`FormatWith`).
    pub(crate) fn format_piece(&mut self, out: &str, value: Val, ty: Ty, spec: Option<&FormatSpec>) -> Result<()> {
        let (size, align) = self.m.ty.size_align(ty);
        let (at, at_align) = self.spill(&value, size, align);
        let (at, at_align, ty) = self.unwrap_printed(at, at_align, ty);
        let more = spec.is_some_and(|spec| (FormatSpec { kind: None, ..*spec }) != FormatSpec::PLAIN);
        let plain_spec = spec.map(|spec| FormatSpec { kind: None, ..*spec });
        let debug = spec.is_some_and(|spec| spec.kind == Some('?'));
        let through = self.m.ty.through_range(ty);
        if is_display_aggregate(self.m, through) || matches!(self.m.ty.kind(through), TyKind::Void) {
            if (!debug || matches!(self.m.ty.kind(through), TyKind::Void))
                && let Some(name) = self.unit_display(&at, at_align, through)?
            {
                let mut values = vec![Val::Imm(Sc::Ptr, out.to_string()), name];
                if more {
                    values.push(self.spec_value(plain_spec.as_ref().expect("a spec")));
                    self.call_rt("fmt_spec_str", values, None);
                } else {
                    self.call_rt("fmt_str", values, None);
                }
                return Ok(());
            }
            let function = fmt_fn(self.m, through);
            if !more {
                self.emit(format!("call void {function}(ptr {out}, ptr {at})"));
                return Ok(());
            }
            // An aggregate takes a spec only after `!r`/`!s` made it text: the spec pads that.
            let text = self.alloca(24, 8);
            self.zero(&text, 8, 24);
            self.emit(format!("call void {function}(ptr {text}, ptr {at})"));
            let view = self.text_at(&text, self.m.ty.table.find(&TyKind::Str).unwrap_or(ty)).expect("a buffer is text");
            let spec = self.spec_value(plain_spec.as_ref().expect("a spec"));
            self.call_rt("fmt_spec_str", vec![Val::Imm(Sc::Ptr, out.to_string()), view, spec], None);
            self.call_rt("vec_free", vec![Val::Imm(Sc::Ptr, text), Val::Imm(Sc::I(64), "1".to_string())], None);
            return Ok(());
        }
        if let Some(text) = self.text_at(&at, through) {
            let mut values = vec![Val::Imm(Sc::Ptr, out.to_string()), text];
            match spec {
                Some(spec) => {
                    values.push(self.spec_value(spec));
                    self.call_rt("fmt_spec_str", values, None);
                }
                None => self.call_rt("fmt_str", values, None),
            }
            return Ok(());
        }
        let value = self.value_at(&at, at_align, through);
        self.format_scalar(out, value, through, spec)
    }

    /// `[TYP-39]` (ODR-034) — a `Map`'s `{k: v, …}` or a `Set`'s `{a, …}` (an empty one
    /// `set()`), in insertion order: the entries are the first field's `Array[Option[MapSlot]]`.
    fn map_body(&mut self, out: &str, v: &str, ty: Ty) -> Result<()> {
        let table = self.m.ty.table;
        let TyKind::Struct(id) = *table.kind(ty) else { unreachable!("a Map or Set is a struct") };
        let def = table.struct_def(id);
        let is_set = def.origin.as_ref().is_some_and(|(name, _)| name.as_str() == "std.collections.Set");
        let (map_ty, map_at) = if is_set {
            let offset = table.layout(ty).field_offsets[0];
            (def.fields[0].ty, self.gep(v, offset))
        } else {
            (ty, v.to_string())
        };
        let TyKind::Struct(map_id) = *table.kind(map_ty) else { unreachable!("a Set holds a Map") };
        let map_def = table.struct_def(map_id);
        let entries_offset = table.layout(map_ty).field_offsets[0];
        let TyKind::Vec { elem: option, .. } = *table.kind(map_def.fields[0].ty) else { unreachable!("a Map's entries are an Array") };
        let TyKind::Enum(option_id) = *table.kind(option) else { unreachable!("an entry is an Option") };
        let option_def = table.enum_def(option_id);
        let some_index = option_def.variants.iter().position(|variant| variant.name.as_str() == "Some").expect("an Option has Some");
        let some = &option_def.variants[some_index];
        let payload_ty = some.fields[0].ty;
        let payload_offset = if table.option_niche(option_id).is_some() { 0 } else { table.variant_field_offsets(option_id)[some_index][0] };
        let TyKind::Struct(slot_id) = *table.kind(payload_ty) else { unreachable!("an entry holds a MapSlot") };
        let slot = table.struct_def(slot_id);
        let slot_offsets = table.layout(payload_ty).field_offsets;
        let field = |name: &str| {
            let index = slot.fields.iter().position(|f| f.name.as_str() == name).expect("a MapSlot field");
            (slot_offsets[index], slot.fields[index].ty)
        };
        let ((key_offset, key_ty), (value_offset, value_ty)) = (field("key"), field("value"));
        let (option_size, option_align) = self.m.ty.size_align(option);
        let some_tag = some.discriminant;
        let done = self.label();
        if is_set {
            // A set with nothing live prints `set()`.
            let live_index = def.fields.iter().position(|f| f.name.as_str() == "live");
            let map_live = map_def.fields.iter().position(|f| f.name.as_str() == "live");
            let live_at = match (live_index, map_live) {
                (Some(index), _) => Some(self.gep(v, table.layout(ty).field_offsets[index])),
                (None, Some(index)) => Some(self.gep(&map_at, table.layout(map_ty).field_offsets[index])),
                _ => None,
            };
            if let Some(live_at) = live_at {
                let live = self.load(Sc::I(64), &live_at, 8);
                let empty = self.value(format!("icmp eq i64 {live}, 0"));
                let (word, walk) = (self.label(), self.label());
                self.emit(format!("br i1 {empty}, label %{word}, label %{walk}"));
                self.start(&word);
                self.text(out, "set()");
                self.emit(format!("br label %{done}"));
                self.start(&walk);
            }
        }
        self.text(out, "{");
        let entries = self.gep(&map_at, entries_offset);
        let buffer = self.load(Sc::Ptr, &entries, 8);
        let len_at = self.gep(&entries, 8);
        let len = self.load(Sc::I(64), &len_at, 8);
        let first = self.alloca(1, 1);
        self.emit(format!("store i8 1, ptr {first}, align 1"));
        let out_owned = out.to_string();
        self.counted_loop(&len, |cx, index| {
            let offset = cx.value(format!("mul i64 {index}, {option_size}"));
            let entry = cx.value(format!("getelementptr i8, ptr {buffer}, i64 {offset}"));
            let at = crate::func::PlaceRef { ptr: entry.clone(), ty: option, variant: None, align: option_align };
            let (tag, bits) = cx.discriminant_at(&at)?;
            let live = cx.value(format!("icmp eq i{bits} {tag}, {}", crate::func::int_literal(some_tag as u128, bits)));
            let (item, skip) = (cx.label(), cx.label());
            cx.emit(format!("br i1 {live}, label %{item}, label %{skip}"));
            cx.start(&item);
            let was_first = cx.load(Sc::I(8), &first, 1);
            let is_first = cx.value(format!("icmp ne i8 {was_first}, 0"));
            let (separate, write) = (cx.label(), cx.label());
            cx.emit(format!("br i1 {is_first}, label %{write}, label %{separate}"));
            cx.start(&separate);
            cx.text(&out_owned, ", ");
            cx.emit(format!("br label %{write}"));
            cx.start(&write);
            cx.emit(format!("store i8 0, ptr {first}, align 1"));
            let payload = cx.gep(&entry, payload_offset);
            let key_at = cx.gep(&payload, key_offset);
            let key_align = cx.m.ty.size_align(key_ty).1;
            cx.debug_into(&out_owned, &key_at, key_align, key_ty)?;
            if !is_set {
                cx.text(&out_owned, ": ");
                let value_at = cx.gep(&payload, value_offset);
                let value_align = cx.m.ty.size_align(value_ty).1;
                cx.debug_into(&out_owned, &value_at, value_align, value_ty)?;
            }
            cx.emit(format!("br label %{skip}"));
            cx.start(&skip);
            Ok(())
        })?;
        self.text(out, "}");
        self.emit(format!("br label %{done}"));
        self.start(&done);
        Ok(())
    }
}
