//! What the backend writes beside the program's own functions: reference counting
//! (`[RC-4]`, `[RT-10]`: the header's inline fast paths, reproduced exactly), drop glue
//! (`[OWN-2]`, `[DRP-2]`), and each class's type information, destructor adapter and field glue
//! (`[RT-3]`, `[CLS-6]`).

use std::collections::BTreeSet;
use std::fmt::Write as _;

use crate::func::{FnCx, Val};
use crate::types::Sc;
use crate::{Module, Profile, Result, global_name, unsupported};
use ember_types::{ClassId, Ty, TyKind};

/// Work the module's functions asked for, done after them (each may ask for more).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) enum Pending {
    Drop(Ty),
    Retain(Ty),
    ClassInfo(ClassId),
}

/// An internal function of the module the backend writes itself: `void @symbol(ptr %p0, …)`,
/// its body what `build` writes.
pub(crate) fn define_glue<'a>(
    m: &mut Module<'a>,
    symbol: &str,
    params: &[&str],
    attrs: &str,
    build: impl FnOnce(&mut FnCx<'_, 'a>) -> Result<()>,
) -> Result<()> {
    let mut cx = FnCx::glue(m);
    build(&mut cx)?;
    cx.emit("ret void");
    let params: Vec<String> = params.iter().enumerate().map(|(index, ty)| format!("{ty} %p{index}")).collect();
    let base = crate::func::fn_attrs(cx.m);
    let text = format!(
        "define internal void {}({}) {base}{attrs} {{\nentry:\n{}  br label %start\nstart:\n{}}}\n\n",
        global_name(symbol),
        params.join(", "),
        cx.allocas_text(),
        cx.out_text()
    );
    cx.m.fns.push_str(&text);
    Ok(())
}

/// The class's type information's symbol (`[MNG-4]`: `em_ti_6Animal`), asking for it.
pub(crate) fn type_info(m: &mut Module<'_>, id: ClassId) -> String {
    m.request(Pending::ClassInfo(id));
    ember_branding::type_info(&m.ty.table.mangled_class(id))
}

/// Whether a class (or a base of it) declares `drop`.
fn has_user_drop(m: &Module<'_>, id: ClassId) -> bool {
    let mut current = Some(id);
    while let Some(class) = current {
        let def = m.ty.table.class_def(class);
        if def.has_drop {
            return true;
        }
        current = def.base;
    }
    false
}

fn has_dropping_fields(m: &Module<'_>, id: ClassId) -> bool {
    let mut current = Some(id);
    while let Some(class) = current {
        let def = m.ty.table.class_def(class);
        if def.fields.iter().any(|field| m.ty.table.needs_drop(field.ty)) {
            return true;
        }
        current = def.base;
    }
    false
}

/// `[OBJ-1]`, `[EXC-19]` — a class object as the C backend declares it: the runtime's header,
/// then each class level's access words (a zero word after an odd count) and its fields, bases
/// first. Each field's offset from the header's end is the type table's
/// (`TypeTable::class_field_offsets`, which reflection reads too).
pub(crate) struct ObjectLayout {
    pub size: u64,
    pub align: u64,
    /// Each field's access word's offset from the object's start, where it has one.
    pub words: Vec<Option<u64>>,
    /// Each class level's words, bases first.
    pub levels: Vec<Vec<u64>>,
}

pub(crate) fn object_layout(m: &Module<'_>, id: ClassId) -> ObjectLayout {
    let table = m.ty.table;
    let count = table.class_field_count(id);
    let offsets = table.class_field_offsets(id);
    let owner = |index: usize| table.class_field_at_info(id, index).map(|(owner, _)| owner);
    let mut words = vec![None; count];
    let mut levels = Vec::new();
    let mut at = 24u64;
    let mut align = 8u64;
    let mut start = 0;
    while start < count {
        let mut end = start + 1;
        while end < count && owner(end) == owner(start) {
            end += 1;
        }
        let mut level = Vec::new();
        for (index, slot) in words.iter_mut().enumerate().take(end).skip(start) {
            if table.class_field_has_access_word(id, index) {
                let offset = at + 4 * level.len() as u64;
                *slot = Some(offset);
                level.push(offset);
            }
        }
        at += 4 * (level.len() as u64 + level.len() as u64 % 2);
        for index in start..end {
            if let Some(field) = table.class_field_at(id, index) {
                let (size, field_align) = m.ty.size_align(field.ty);
                align = align.max(field_align);
                at = 24 + offsets[index] + size;
            }
        }
        levels.push(level);
        start = end;
    }
    ObjectLayout { size: at.div_ceil(align) * align, align, words, levels }
}

/// Every pending piece of glue, to a fixpoint.
pub(crate) fn emit_pending(m: &mut Module<'_>) -> Result<()> {
    let mut done: BTreeSet<Pending> = BTreeSet::new();
    while let Some(work) = m.pending.pop() {
        if !done.insert(work) {
            continue;
        }
        match work {
            Pending::Drop(ty) => emit_drop_glue(m, ty)?,
            Pending::Retain(ty) => emit_retain_glue(m, ty)?,
            Pending::ClassInfo(id) => emit_class(m, id)?,
        }
    }
    Ok(())
}

fn emit_drop_glue(m: &mut Module<'_>, ty: Ty) -> Result<()> {
    let symbol = drop_glue_symbol(m, ty);
    define_glue(m, &symbol, &["ptr"], "", |cx| {
        let (_, align) = cx.m.ty.size_align(ty);
        cx.drop_lines("%p0", align, ty)
    })
}

fn emit_retain_glue(m: &mut Module<'_>, ty: Ty) -> Result<()> {
    let symbol = retain_glue_symbol(m, ty);
    define_glue(m, &symbol, &["ptr"], "", |cx| {
        let (_, align) = cx.m.ty.size_align(ty);
        cx.retain_lines("%p0", align, ty)
    })
}

/// The drop glue of `ty`'s symbol: one function per type, so a recursive type's glue calls
/// itself and every place drops through one copy.
pub(crate) fn drop_glue_symbol(m: &Module<'_>, ty: Ty) -> String {
    format!("ember.drop.{}", m.ty.table.mangled(ty))
}

fn retain_glue_symbol(m: &Module<'_>, ty: Ty) -> String {
    format!("ember.retain.{}", m.ty.table.mangled(ty))
}

/// A class's type information, its destructor adapter and its field glue.
fn emit_class(m: &mut Module<'_>, id: ClassId) -> Result<()> {
    let table = m.ty.table;
    let def = table.class_def(id).clone();
    let mangled = table.mangled_class(id);
    let info = ember_branding::type_info(&mangled);
    let layout = object_layout(m, id);
    if table.class_reflection(id).is_some() {
        return unsupported("a `@reflect` class's description ([RFL-2])");
    }
    let drop = if has_user_drop(m, id) {
        let symbol = format!("{}_{mangled}", ember_branding::generated("drop_adapter"));
        emit_drop_adapter(m, id, &symbol)?;
        global_name(&symbol)
    } else {
        "null".to_string()
    };
    let drop_fields = if has_dropping_fields(m, id) {
        let symbol = format!("{}_{mangled}", ember_branding::generated("drop_fields"));
        emit_drop_fields(m, id, &symbol)?;
        global_name(&symbol)
    } else {
        "null".to_string()
    };
    let base = match def.base {
        Some(base) => global_name(&type_info(m, base)),
        None => "null".to_string(),
    };
    // `[EXC-19]` — every access word of the class, its bases' included.
    let mut words = Vec::new();
    for (index, word) in layout.words.iter().enumerate() {
        if let (Some(offset), Some(field)) = (word, table.class_field_at(id, index)) {
            let name = m.bytes(field.name.as_str().as_bytes());
            words.push(format!("{{ ptr, i32 }} {{ ptr {name}, i32 {offset} }}"));
        }
    }
    let (fields, field_count) = if words.is_empty() {
        ("null".to_string(), 0)
    } else {
        let symbol = format!("{info}_access_words");
        m.global(format!(
            "{} = internal constant [{} x {{ ptr, i32 }}] [{}], align 8",
            global_name(&symbol),
            words.len(),
            words.join(", ")
        ));
        (global_name(&symbol), words.len())
    };
    let name = m.bytes(def.name.as_str().as_bytes());
    let flags = if def.is_sync { 1 } else { 0 };
    let vtable = crate::dispatch::class_vtable(m, id)?;
    let (itables, itable_count) = crate::dispatch::class_itables(m, id)?;
    m.global(format!(
        "{} = internal constant {{ i32, i32, i32, ptr, ptr, ptr, ptr, ptr, ptr, i32, ptr, i32, ptr }} \
         {{ i32 {}, i32 {}, i32 {flags}, ptr {name}, ptr {base}, ptr {drop}, ptr {drop_fields}, ptr {vtable}, \
         ptr {itables}, i32 {itable_count}, ptr {fields}, i32 {field_count}, ptr null }}, align 8",
        global_name(&info),
        layout.size,
        layout.align,
    ));
    Ok(())
}

/// `[CLS-6]` — the runtime's `drop` entry for a class: its own `drop`, then each base's, under a
/// write access to every field (`[EXC-15]`: the runtime is the caller, D-453).
fn emit_drop_adapter(m: &mut Module<'_>, id: ClassId, symbol: &str) -> Result<()> {
    let table = m.ty.table;
    let def = table.class_def(id).clone();
    // The location is the first `drop` in the chain.
    let mut location = None;
    let mut chain = Some(id);
    while let Some(class) = chain {
        let class_def = table.class_def(class);
        if class_def.has_drop {
            let symbol = table.class_method_symbol(class, "drop");
            location = m.bodies.get(symbol.as_str()).map(|body| body.span);
            break;
        }
        chain = class_def.base;
    }
    let location = location.unwrap_or(ember_span::Span::DUMMY);
    define_glue(m, symbol, &["ptr"], "", |cx| {
        let object_write = !def.is_sync;
        if object_write {
            let loc = cx.loc(location);
            cx.call_rt("object_begin_write", vec![Val::Imm(Sc::Ptr, "%p0".to_string()), loc], None);
        }
        let mut current = Some(id);
        while let Some(class) = current {
            let class_def = cx.m.ty.table.class_def(class);
            if class_def.has_drop {
                let method = cx.m.ty.table.class_method_symbol(class, "drop");
                // D-685 — `drop(mut self)` takes the handle's address; `drop(self)` the handle.
                let by_address = cx.m.bodies.get(method.as_str()).is_none_or(|body| {
                    body.locals.get(1).is_none_or(|receiver| matches!(cx.m.ty.kind(receiver.ty), TyKind::Ref { .. }))
                });
                let argument = if by_address {
                    let slot = cx.alloca(8, 8);
                    cx.emit(format!("store ptr %p0, ptr {slot}, align 8"));
                    slot
                } else {
                    "%p0".to_string()
                };
                cx.call_body(&method, vec![Val::Imm(Sc::Ptr, argument)], None)?;
            }
            current = class_def.base;
        }
        if object_write {
            let loc = cx.loc(location);
            cx.call_rt("object_end_write", vec![Val::Imm(Sc::Ptr, "%p0".to_string()), loc], None);
        }
        Ok(())
    })
}

/// `[CLS-6]` — the fields' half of a class's destruction: the most derived fields first, each
/// level's in reverse order.
fn emit_drop_fields(m: &mut Module<'_>, id: ClassId, symbol: &str) -> Result<()> {
    let table = m.ty.table;
    let offsets = table.class_field_offsets(id);
    let mut levels = Vec::new();
    let mut current = Some(id);
    while let Some(class) = current {
        levels.push(class);
        current = table.class_def(class).base;
    }
    define_glue(m, symbol, &["ptr"], "", |cx| {
        let count = table.class_field_count(id);
        for class in levels {
            let mut fields: Vec<(usize, Ty)> = (0..count)
                .filter(|&index| table.class_field_at_info(id, index).is_some_and(|(owner, _)| owner == class))
                .filter_map(|index| table.class_field_at(id, index).map(|field| (index, field.ty)))
                .collect();
            fields.reverse();
            for (index, field_ty) in fields {
                if !table.needs_drop(field_ty) {
                    continue;
                }
                let offset = 24 + offsets[index];
                let at = cx.gep("%p0", offset);
                let align = cx.m.ty.size_align(field_ty).1.min(crate::types::offset_align(offset));
                cx.drop_at(&at, align, field_ty)?;
            }
        }
        Ok(())
    })
}

impl<'a> FnCx<'_, 'a> {
    // -- reference counts ---------------------------------------------------------------------

    /// The count helper `operation` (`retain`, `release`) for a handle of `ty`: the plain or
    /// atomic inline form for a class, chosen by its declaration (`[RC-4]`); the type information's
    /// choice for any other handle.
    fn count_helper(&mut self, operation: &str, ty: Ty) -> String {
        let kind = match self.m.ty.kind(ty) {
            TyKind::Class(id) if self.m.ty.table.class_def(*id).is_sync => "sync",
            TyKind::Class(_) => "plain",
            _ => "any",
        };
        count_helper(self.m, operation, kind)
    }

    pub(crate) fn retain_handle(&mut self, handle: &str, ty: Ty) {
        let helper = self.count_helper("retain", ty);
        self.emit(format!("call void {helper}(ptr {handle})"));
    }

    pub(crate) fn release_handle(&mut self, handle: &str, ty: Ty) {
        let helper = self.count_helper("release", ty);
        self.emit(format!("call void {helper}(ptr {handle})"));
    }

    /// `[OWN-7]` — the strong references a copy of the value at `ptr` takes: each class handle
    /// it holds retained, through glue where an enum's switch nests (D-680).
    pub(crate) fn retain_lines(&mut self, ptr: &str, align: u64, ty: Ty) -> Result<()> {
        let table = self.m.ty.table;
        match table.kind(ty).clone() {
            TyKind::Class(_) | TyKind::ClassInterface(_) => {
                let handle = self.load(Sc::Ptr, ptr, align.min(8));
                self.retain_handle(&handle, ty);
            }
            TyKind::Struct(id) => {
                let def = table.struct_def(id);
                if crate::func::shared_inner(table, id).is_some() {
                    let handle = self.load(Sc::Ptr, ptr, align.min(8));
                    let helper = count_helper(self.m, "retain", "any");
                    self.emit(format!("call void {helper}(ptr {handle})"));
                    return Ok(());
                }
                if weak_inner(table, id).is_some() {
                    let handle = self.load(Sc::Ptr, ptr, align.min(8));
                    self.call_rt("weak_retain", vec![Val::Imm(Sc::Ptr, handle)], None);
                    return Ok(());
                }
                if def.name.as_str().starts_with("MaybeUninit_") || !def.drops_fields {
                    return Ok(());
                }
                let offsets = table.layout(ty).field_offsets;
                for (field, offset) in def.fields.clone().iter().zip(offsets) {
                    let at = self.gep(ptr, offset);
                    let field_align = align.min(crate::types::offset_align(offset));
                    self.retain_lines(&at, field_align, field.ty)?;
                }
            }
            TyKind::Tuple(items) => {
                let offsets = table.layout(ty).field_offsets;
                for (item, offset) in items.into_iter().zip(offsets) {
                    let at = self.gep(ptr, offset);
                    self.retain_lines(&at, align.min(crate::types::offset_align(offset)), item)?;
                }
            }
            TyKind::Array { elem, len } => {
                if !self.holds_counts(elem) {
                    return Ok(());
                }
                let (size, _) = self.m.ty.size_align(elem);
                for index in 0..len {
                    let at = self.gep(ptr, index * size);
                    self.retain_lines(&at, align.min(crate::types::offset_align(index * size)), elem)?;
                }
            }
            TyKind::Enum(id) => {
                if !self.holds_counts(ty) {
                    return Ok(());
                }
                self.per_variant(ptr, align, id, false, |cx, at, field_align, field_ty| {
                    if matches!(cx.m.ty.kind(field_ty), TyKind::Enum(_)) || cx.retain_nests(field_ty) {
                        cx.m.request(Pending::Retain(field_ty));
                        let symbol = global_name(&retain_glue_symbol(cx.m, field_ty));
                        cx.emit(format!("call void {symbol}(ptr {at})"));
                        Ok(())
                    } else {
                        cx.retain_lines(at, field_align, field_ty)
                    }
                })?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Whether a copy of `ty` retains anything.
    pub(crate) fn holds_counts(&self, ty: Ty) -> bool {
        let table = self.m.ty.table;
        match table.kind(ty) {
            TyKind::Class(_) | TyKind::ClassInterface(_) => true,
            TyKind::Struct(id) => {
                let def = table.struct_def(*id);
                crate::func::shared_inner(table, *id).is_some()
                    || weak_inner(table, *id).is_some()
                    || (!def.name.as_str().starts_with("MaybeUninit_")
                        && def.drops_fields
                        && def.fields.iter().any(|field| self.holds_counts(field.ty)))
            }
            TyKind::Tuple(items) => items.iter().any(|item| self.holds_counts(*item)),
            TyKind::Array { elem, .. } => self.holds_counts(*elem),
            TyKind::Enum(id) => table.enum_def(*id).variants.iter().any(|v| v.fields.iter().any(|f| self.holds_counts(f.ty))),
            _ => false,
        }
    }

    fn retain_nests(&self, ty: Ty) -> bool {
        match self.m.ty.kind(ty) {
            TyKind::Enum(_) => self.holds_counts(ty),
            TyKind::Struct(_) | TyKind::Tuple(_) | TyKind::Array { .. } => {
                // A struct's retain is lines, unless it nests an enum that retains.
                let table = self.m.ty.table;
                match table.kind(ty) {
                    TyKind::Struct(id) => table.struct_def(*id).fields.iter().any(|f| self.retain_nests(f.ty)),
                    TyKind::Tuple(items) => items.iter().any(|item| self.retain_nests(*item)),
                    TyKind::Array { elem, .. } => self.retain_nests(*elem),
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// For each variant of the enum `id` at `ptr` with a field `pick` admits, its fields
    /// (reverse order for a drop), under a switch on the tag.
    fn per_variant(
        &mut self,
        ptr: &str,
        align: u64,
        id: ember_types::EnumId,
        reverse: bool,
        mut each: impl FnMut(&mut Self, &str, u64, Ty) -> Result<()>,
    ) -> Result<()> {
        let table = self.m.ty.table;
        let def = table.enum_def(id).clone();
        let niche = table.option_niche(id);
        let offsets = table.variant_field_offsets(id);
        let ty = table.find(&TyKind::Enum(id)).expect("an enum is interned");
        let at = crate::func::PlaceRef { ptr: ptr.to_string(), ty, variant: None, align };
        let (tag, bits) = self.discriminant_at(&at)?;
        let done = self.label();
        let mut arms = Vec::new();
        let mut bodies = Vec::new();
        for (index, variant) in def.variants.iter().enumerate() {
            if !variant.fields.iter().any(|field| table.needs_drop(field.ty) || self.holds_counts(field.ty)) {
                continue;
            }
            let label = self.label();
            arms.push(format!("i{bits} {}, label %{label}", crate::func::int_literal(variant.discriminant as u128, bits)));
            bodies.push((label, index));
        }
        if arms.is_empty() {
            return Ok(());
        }
        self.emit(format!("switch i{bits} {tag}, label %{done} [ {} ]", arms.join(" ")));
        for (label, index) in bodies {
            self.start(&label);
            let variant = &def.variants[index];
            let mut fields: Vec<(usize, Ty)> = variant.fields.iter().map(|field| field.ty).enumerate().collect();
            if reverse {
                fields.reverse();
            }
            for (field_index, field_ty) in fields {
                let offset = if niche.is_some() { 0 } else { offsets[index][field_index] };
                let field_at = self.gep(ptr, offset);
                let field_align = align.min(crate::types::offset_align(offset)).min(self.m.ty.size_align(field_ty).1);
                each(self, &field_at, field_align, field_ty)?;
            }
            self.emit(format!("br label %{done}"));
        }
        self.start(&done);
        Ok(())
    }

    // -- drops -------------------------------------------------------------------------------

    /// The value at `ptr` dropped: a handle released in line, anything else through its glue.
    pub(crate) fn drop_at(&mut self, ptr: &str, align: u64, ty: Ty) -> Result<()> {
        let table = self.m.ty.table;
        if !table.needs_drop(ty) {
            return Ok(());
        }
        if matches!(table.kind(ty), TyKind::Class(_) | TyKind::ClassInterface(_)) {
            let handle = self.load(Sc::Ptr, ptr, align.min(8));
            self.release_handle(&handle, ty);
            return Ok(());
        }
        let (size, natural) = self.m.ty.size_align(ty);
        // `[LAY-2]` — a value in a `@packed` place is dropped from an aligned copy.
        let ptr = if align < natural {
            let copy = self.alloca(size, natural);
            self.copy(&copy, natural, ptr, align, size, false);
            copy
        } else {
            ptr.to_string()
        };
        self.m.request(Pending::Drop(ty));
        let symbol = global_name(&drop_glue_symbol(self.m, ty));
        self.emit(format!("call void {symbol}(ptr {ptr})"));
        Ok(())
    }

    /// What dropping the value at `ptr` runs (`[OWN-2]`: its own `drop` first, then its fields
    /// in reverse order; `[DRP-2]`: an enum's active variant only).
    pub(crate) fn drop_lines(&mut self, ptr: &str, align: u64, ty: Ty) -> Result<()> {
        let table = self.m.ty.table;
        match table.kind(ty).clone() {
            TyKind::Class(_) | TyKind::ClassInterface(_) => self.drop_at(ptr, align, ty)?,
            // `[CLO-10]` — an owned callable value ends its captured state.
            TyKind::Fn { abi: None, .. } => {
                let descriptor_at = self.gep(ptr, 8);
                let descriptor = self.load(Sc::Ptr, &descriptor_at, 8);
                let has = self.value(format!("icmp ne ptr {descriptor}, null"));
                let (check, call, done) = (self.label(), self.label(), self.label());
                self.emit(format!("br i1 {has}, label %{check}, label %{done}"));
                self.start(&check);
                let drop_at = self.gep(&descriptor, 16);
                let drop = self.load(Sc::Ptr, &drop_at, 8);
                let has_drop = self.value(format!("icmp ne ptr {drop}, null"));
                self.emit(format!("br i1 {has_drop}, label %{call}, label %{done}"));
                self.start(&call);
                let state = self.gep(ptr, 16);
                self.emit(format!("call void {drop}(ptr {state})"));
                self.emit(format!("br label %{done}"));
                self.start(&done);
            }
            TyKind::Vec { elem, .. } => {
                if table.needs_drop(elem) {
                    let (size, elem_align) = self.m.ty.size_align(elem);
                    let buffer = self.load(Sc::Ptr, ptr, 8);
                    let len_at = self.gep(ptr, 8);
                    let len = self.load(Sc::I(64), &len_at, 8);
                    self.counted_loop(&len, |cx, index| {
                        let offset = cx.value(format!("mul i64 {index}, {size}"));
                        let element = cx.value(format!("getelementptr i8, ptr {buffer}, i64 {offset}"));
                        cx.drop_at(&element, elem_align, elem)
                    })?;
                }
                self.vec_free(ptr, elem);
            }
            TyKind::Struct(id) => self.drop_struct(ptr, align, ty, id)?,
            TyKind::Tuple(items) => {
                let offsets = table.layout(ty).field_offsets;
                for (item, offset) in items.into_iter().zip(offsets).rev() {
                    let at = self.gep(ptr, offset);
                    self.drop_at(&at, align.min(crate::types::offset_align(offset)), item)?;
                }
            }
            TyKind::Array { elem, len } => {
                let (size, _) = self.m.ty.size_align(elem);
                for index in 0..len {
                    let at = self.gep(ptr, index * size);
                    self.drop_at(&at, align.min(crate::types::offset_align(index * size)), elem)?;
                }
            }
            TyKind::Enum(id) => {
                let def = table.enum_def(id);
                if def.has_drop {
                    let symbol = table.method_symbol(ty, None, "drop", None);
                    self.call_body(&symbol, vec![Val::Imm(Sc::Ptr, ptr.to_string())], None)?;
                }
                if !def.is_unit_only() {
                    self.per_variant(ptr, align, id, true, |cx, at, field_align, field_ty| cx.drop_at(at, field_align, field_ty))?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn drop_struct(&mut self, ptr: &str, align: u64, ty: Ty, id: ember_types::StructId) -> Result<()> {
        let table = self.m.ty.table;
        let def = table.struct_def(id).clone();
        if crate::func::shared_inner(table, id).is_some() {
            let handle = self.load(Sc::Ptr, ptr, align.min(8));
            let helper = count_helper(self.m, "release", "any");
            self.emit(format!("call void {helper}(ptr {handle})"));
            return Ok(());
        }
        if weak_inner(table, id).is_some() {
            let handle = self.load(Sc::Ptr, ptr, align.min(8));
            self.call_rt("weak_release", vec![Val::Imm(Sc::Ptr, handle)], None);
            return Ok(());
        }
        if let Some(inner) = table.compiler_box_inner(id) {
            if matches!(table.kind(inner), TyKind::Dyn { .. }) {
                // `[TYP-22]` — the table's `drop` ends the value, then its block is freed (a class
                // handle's table frees nothing: its size is 0).
                let data = self.load(Sc::Ptr, ptr, align.min(8));
                let vtable_at = self.gep(ptr, 8);
                let vtable = self.load(Sc::Ptr, &vtable_at, align.min(8));
                let drop = self.load(Sc::Ptr, &vtable, 8);
                self.call_c(&drop, None, &[crate::func::Param::Ptr], vec![Val::Imm(Sc::Ptr, data.clone())], None);
                let size_at = self.gep(&vtable, 8);
                let size = self.load(Sc::I(64), &size_at, 8);
                let align_at = self.gep(&vtable, 16);
                let block_align = self.load(Sc::I(64), &align_at, 8);
                let owns = self.value(format!("icmp ne i64 {size}, 0"));
                let (free, done) = (self.label(), self.label());
                self.emit(format!("br i1 {owns}, label %{free}, label %{done}"));
                self.start(&free);
                self.call_rt("free", vec![Val::Imm(Sc::Ptr, data), Val::Imm(Sc::I(64), size), Val::Imm(Sc::I(64), block_align)], None);
                self.emit(format!("br label %{done}"));
                self.start(&done);
                return Ok(());
            }
            // IX.1, `[DRP-6]` — the boxed value first, then exactly its allocation.
            let payload = self.load(Sc::Ptr, ptr, align.min(8));
            let (size, inner_align) = self.m.ty.size_align(inner);
            self.drop_at(&payload, inner_align, inner)?;
            // C's `sizeof` is at least 1.
            let size = size.max(1);
            self.call_rt(
                "free",
                vec![Val::Imm(Sc::Ptr, payload), Val::Imm(Sc::I(64), size.to_string()), Val::Imm(Sc::I(64), inner_align.to_string())],
                None,
            );
            return Ok(());
        }
        let origin = def.origin.as_ref().map(|(origin, _)| origin.as_str().to_string());
        match origin.as_deref() {
            Some("std.sync.MutexGuard" | "SoA" | "std.thread.Scope" | "std.jobs.JobScope" | "std.thread.JoinHandle"
                | "std.sync.RwLockReadGuard" | "std.sync.RwLockWriteGuard") => {
                return unsupported(format!("the drop of `{}`", table.display(ty)));
            }
            _ => {}
        }
        let name = def.name.as_str();
        if name == "Arena" || name == "ScopedArena" || name.starts_with("RefMut_") || name.starts_with("Ref_") {
            return unsupported(format!("the drop of `{}`", table.display(ty)));
        }
        if def.has_drop {
            let symbol = table.method_symbol(ty, None, "drop", None);
            self.call_body(&symbol, vec![Val::Imm(Sc::Ptr, ptr.to_string())], None)?;
        }
        if def.drops_fields {
            let packed = table.layout_attrs(ty).packed;
            let offsets = table.layout(ty).field_offsets;
            for (field, offset) in def.fields.iter().zip(offsets).rev() {
                let at = self.gep(ptr, offset);
                let field_align = if packed { 1 } else { align.min(crate::types::offset_align(offset)) };
                self.drop_at(&at, field_align, field.ty)?;
            }
        }
        Ok(())
    }

    /// `[HEAP-9]` — a list's buffer freed: `ember_vec_free` with the element's size, or its
    /// `_aligned` twin past 16 bytes of alignment.
    fn vec_free(&mut self, ptr: &str, elem: Ty) {
        let (size, align) = self.m.ty.size_align(elem);
        let size = size.max(1);
        if align > 16 {
            self.call_rt(
                "vec_free_aligned",
                vec![Val::Imm(Sc::Ptr, ptr.to_string()), Val::Imm(Sc::I(64), size.to_string()), Val::Imm(Sc::I(64), align.to_string())],
                None,
            );
        } else {
            self.call_rt("vec_free", vec![Val::Imm(Sc::Ptr, ptr.to_string()), Val::Imm(Sc::I(64), size.to_string())], None);
        }
    }

    /// A loop over `0..count` (an `i64`), `each` given the index.
    pub(crate) fn counted_loop(&mut self, count: &str, mut each: impl FnMut(&mut Self, &str) -> Result<()>) -> Result<()> {
        let (head, body, done) = (self.label(), self.label(), self.label());
        let entry = self.block_label();
        self.emit(format!("br label %{head}"));
        self.start(&head);
        let index = self.tmp();
        let next = self.tmp();
        let latch = self.label();
        self.emit(format!("{index} = phi i64 [ 0, %{entry} ], [ {next}, %{latch} ]"));
        let more = self.value(format!("icmp ult i64 {index}, {count}"));
        self.emit(format!("br i1 {more}, label %{body}, label %{done}"));
        self.start(&body);
        each(self, &index)?;
        self.emit(format!("br label %{latch}"));
        self.start(&latch);
        self.emit(format!("{next} = add i64 {index}, 1"));
        self.emit(format!("br label %{head}"));
        self.start(&done);
        Ok(())
    }

    // -- classes -------------------------------------------------------------------------------

    /// `[CLS-1]` — a new object of the class: its header from the runtime, its fields for the
    /// constructor to write.
    pub(crate) fn class_new(&mut self, id: ClassId) -> String {
        let info = global_name(&type_info(self.m, id));
        let slot = self.alloca(8, 8);
        self.call_rt("obj_new", vec![Val::Imm(Sc::Ptr, info)], Some((&slot, 8)));
        self.load(Sc::Ptr, &slot, 8)
    }
}

/// `[EXC-1]`, `[EXC-2]`, `[EXC-19]` — the header's inline field-access operations on a class
/// field's own word (bit 31 the writer, bits 0..30 the reader count), reproduced: `operation` is
/// `begin_read`, `begin_write`, `end_read`, `end_write`, `check_read` or `check_write`. Each
/// takes the word, what a conflict names and a pointer to the access's `ember_loc`. In the debug
/// profile a conflict names where the active access began (`EMBER_RT_DEBUG_ACCESS`).
pub(crate) fn field_helper(m: &mut Module<'_>, operation: &str) -> String {
    let symbol = format!("ember.field_{operation}");
    if m.helper_defined(&symbol) {
        return global_name(&symbol);
    }
    let debug = m.profile == Profile::Debug;
    let writing = operation.ends_with("write");
    let built = define_glue(m, &symbol, &["ptr", "ptr", "ptr"], " alwaysinline", |cx| {
        let (word, what, loc) = ("%p0", "%p1", "%p2");
        let held = cx.load(Sc::I(32), word, 4);
        let conflict = match operation {
            "begin_read" | "end_read" => {
                let writer = cx.value(format!("and i32 {held}, -2147483648"));
                let has_writer = cx.value(format!("icmp ne i32 {writer}, 0"));
                let readers = cx.value(format!("and i32 {held}, 2147483647"));
                let edge = if operation == "begin_read" { "2147483647" } else { "0" };
                let at_edge = cx.value(format!("icmp eq i32 {readers}, {edge}"));
                cx.value(format!("or i1 {has_writer}, {at_edge}"))
            }
            "check_read" => {
                let writer = cx.value(format!("and i32 {held}, -2147483648"));
                cx.value(format!("icmp ne i32 {writer}, 0"))
            }
            "end_write" => cx.value(format!("icmp ne i32 {held}, -2147483648")),
            _ => cx.value(format!("icmp ne i32 {held}, 0")),
        };
        let (fail, ok) = (cx.label(), cx.label());
        cx.emit(format!("br i1 {conflict}, label %{fail}, label %{ok}"));
        cx.start(&fail);
        let loc_value = Val::Mem(loc.to_string(), 8);
        if operation.starts_with("end") {
            cx.call_rt("panic_exclusivity", vec![Val::Imm(Sc::Ptr, what.to_string()), loc_value.clone()], None);
        } else if debug {
            cx.call_rt(
                "panic_field_exclusivity_debug",
                vec![
                    Val::Imm(Sc::Bool, writing.to_string()),
                    Val::Imm(Sc::I(32), held.clone()),
                    Val::Imm(Sc::Ptr, word.to_string()),
                    Val::Imm(Sc::Ptr, what.to_string()),
                    loc_value.clone(),
                ],
                None,
            );
        } else {
            cx.call_rt(
                "panic_field_exclusivity",
                vec![Val::Imm(Sc::Bool, writing.to_string()), Val::Imm(Sc::I(32), held.clone()), Val::Imm(Sc::Ptr, what.to_string()), loc_value.clone()],
                None,
            );
        }
        cx.start(&ok);
        let next = match operation {
            "begin_read" => Some(cx.value(format!("add i32 {held}, 1"))),
            "end_read" => Some(cx.value(format!("sub i32 {held}, 1"))),
            "begin_write" => Some("-2147483648".to_string()),
            "end_write" => Some("0".to_string()),
            _ => None,
        };
        if let Some(next) = next {
            cx.emit(format!("store i32 {next}, ptr {word}, align 4"));
            if debug {
                let track = if operation.starts_with("begin") { "debug_field_begin" } else { "debug_field_end" };
                cx.call_rt(track, vec![Val::Imm(Sc::Ptr, word.to_string()), Val::Imm(Sc::Bool, writing.to_string()), loc_value], None);
            }
        }
        Ok(())
    });
    built.expect("the field-access helpers are always built");
    global_name(&symbol)
}

/// `ember_vec_push` and `ember_vec_push_aligned`, the header's inline appends, reproduced: room
/// made when the list is full (`ember_vec_grown`, or for an element aligned past 16 its
/// `reserve_aligned`), the element's bytes copied after the last, the length one more. Its
/// parameters: the list, the element's size and alignment, and the value's address.
pub(crate) fn vec_push_helper(m: &mut Module<'_>, aligned: bool) -> String {
    let symbol = if aligned { "ember.vec_push_aligned" } else { "ember.vec_push" };
    if m.helper_defined(symbol) {
        return global_name(symbol);
    }
    let built = define_glue(m, symbol, &["ptr", "i64", "i64", "ptr"], " alwaysinline", |cx| {
        let (list, size, align, value) = ("%p0", "%p1", "%p2", "%p3");
        let len_at = cx.gep(list, 8);
        let len = cx.load(Sc::I(64), &len_at, 8);
        let cap_at = cx.gep(list, 16);
        let cap = cx.load(Sc::I(64), &cap_at, 8);
        let full = cx.value(format!("icmp eq i64 {len}, {cap}"));
        let (grow, put) = (cx.label(), cx.label());
        cx.emit(format!("br i1 {full}, label %{grow}, label %{put}"));
        cx.start(&grow);
        if aligned {
            let want = cx.value(format!("add i64 {len}, 1"));
            cx.call_rt(
                "vec_reserve_aligned",
                vec![Val::Imm(Sc::Ptr, list.to_string()), Val::Imm(Sc::I(64), size.to_string()), Val::Imm(Sc::I(64), align.to_string()), Val::Imm(Sc::I(64), want)],
                None,
            );
        } else {
            let grown = cx.alloca(24, 8);
            cx.call_rt("vec_grown", vec![Val::Mem(list.to_string(), 8), Val::Imm(Sc::I(64), size.to_string())], Some((&grown, 8)));
            cx.copy(list, 8, &grown, 8, 24, false);
        }
        cx.emit(format!("br label %{put}"));
        cx.start(&put);
        let buffer = cx.load(Sc::Ptr, list, 8);
        let offset = cx.value(format!("mul i64 {len}, {size}"));
        let slot = cx.value(format!("getelementptr i8, ptr {buffer}, i64 {offset}"));
        let intrinsic = "llvm.memcpy.p0.p0.i64";
        cx.m.declare(intrinsic, || format!("declare void @{intrinsic}(ptr, ptr, i64, i1)"));
        cx.emit(format!("call void @{intrinsic}(ptr align 1 {slot}, ptr align 1 {value}, i64 {size}, i1 false)"));
        let next = cx.value(format!("add i64 {len}, 1"));
        cx.emit(format!("store i64 {next}, ptr {len_at}, align 8"));
        Ok(())
    });
    built.expect("the push helpers are always built");
    global_name(symbol)
}

pub(crate) fn weak_inner(table: &ember_types::TypeTable, id: ember_types::StructId) -> Option<Ty> {
    let def = table.struct_def(id);
    match &def.origin {
        Some((name, args)) if name.is("Weak") && args.len() == 1 => Some(args[0]),
        _ => None,
    }
}

/// `[RC-4]`, `[RT-10]` — the header's inline count operations, reproduced: `kind` is `plain`
/// (a class that is not `@sync`), `sync` (one that is) or `any` (read from the type
/// information). The fast path is in line; only a count at its end calls the runtime.
pub(crate) fn count_helper(m: &mut Module<'_>, operation: &str, kind: &str) -> String {
    let symbol = format!("ember.{operation}_{kind}");
    if m.helper_defined(&symbol) {
        return global_name(&symbol);
    }
    let debug = m.profile == Profile::Debug;
    let mut cx = FnCx::glue(m);
    let object = "%p0";
    match (operation, kind) {
        ("retain", "plain") | ("release", "plain") => {
            // `EMBER_OBJ_CHECK_THREAD` in the debug profile (`[FFI-33a]`), then the count.
            let (live, fast, slow, done) = (cx.label(), cx.label(), cx.label(), cx.label());
            let null = cx.value(format!("icmp eq ptr {object}, null"));
            cx.emit(format!("br i1 {null}, label %{slow}, label %{live}"));
            cx.start(&live);
            if debug {
                cx.call_rt("obj_check_thread", vec![Val::Imm(Sc::Ptr, object.to_string())], None);
            }
            let strong = cx.load(Sc::I(32), object, 8);
            let room = if operation == "retain" {
                let less = cx.value(format!("sub i32 {strong}, 1"));
                cx.value(format!("icmp ult i32 {less}, -2"))
            } else {
                cx.value(format!("icmp ugt i32 {strong}, 1"))
            };
            cx.emit(format!("br i1 {room}, label %{fast}, label %{slow}"));
            cx.start(&fast);
            let step = if operation == "retain" { "add" } else { "sub" };
            let next = cx.value(format!("{step} i32 {strong}, 1"));
            cx.emit(format!("store i32 {next}, ptr {object}, align 8"));
            cx.emit(format!("br label %{done}"));
            cx.start(&slow);
            let entry = if operation == "retain" { "obj_retain" } else { "obj_release" };
            cx.call_rt(entry, vec![Val::Imm(Sc::Ptr, object.to_string())], None);
            cx.emit(format!("br label %{done}"));
            cx.start(&done);
        }
        ("retain", "sync") | ("release", "sync") => {
            let (live, ended, done) = (cx.label(), cx.label(), cx.label());
            let null = cx.value(format!("icmp eq ptr {object}, null"));
            cx.emit(format!("br i1 {null}, label %{done}, label %{live}"));
            cx.start(&live);
            if operation == "retain" {
                // `EMBER_COUNT_UP`: relaxed (`[RT-8]`).
                let previous = cx.value(format!("atomicrmw add ptr {object}, i32 1 monotonic, align 8"));
                let less = cx.value(format!("sub i32 {previous}, 1"));
                let failed = cx.value(format!("icmp uge i32 {less}, -2"));
                cx.emit(format!("br i1 {failed}, label %{ended}, label %{done}"));
                cx.start(&ended);
                cx.call_rt("obj_retain_failed", vec![Val::Imm(Sc::Ptr, object.to_string()), Val::Imm(Sc::I(32), previous)], None);
            } else {
                // `EMBER_COUNT_DOWN`: acquire-release.
                let previous = cx.value(format!("atomicrmw sub ptr {object}, i32 1 acq_rel, align 8"));
                let last = cx.value(format!("icmp ule i32 {previous}, 1"));
                cx.emit(format!("br i1 {last}, label %{ended}, label %{done}"));
                cx.start(&ended);
                cx.call_rt("obj_release_ended", vec![Val::Imm(Sc::Ptr, object.to_string()), Val::Imm(Sc::I(32), previous)], None);
                cx.emit(format!("br label %{done}"));
            }
            cx.start(&done);
        }
        (_, "any") => {
            let plain = count_helper(cx.m, operation, "plain");
            let sync = count_helper(cx.m, operation, "sync");
            let (live, atomic, ordinary, done) = (cx.label(), cx.label(), cx.label(), cx.label());
            let null = cx.value(format!("icmp eq ptr {object}, null"));
            cx.emit(format!("br i1 {null}, label %{ordinary}, label %{live}"));
            cx.start(&live);
            let info_at = cx.gep(object, 16);
            let info = cx.load(Sc::Ptr, &info_at, 8);
            let has_info = cx.value(format!("icmp ne ptr {info}, null"));
            let (flags_label, _) = (cx.label(), ());
            cx.emit(format!("br i1 {has_info}, label %{flags_label}, label %{ordinary}"));
            cx.start(&flags_label);
            let flags_at = cx.gep(&info, 8);
            let flags = cx.load(Sc::I(32), &flags_at, 8);
            let bit = cx.value(format!("and i32 {flags}, 1"));
            let is_sync = cx.value(format!("icmp ne i32 {bit}, 0"));
            cx.emit(format!("br i1 {is_sync}, label %{atomic}, label %{ordinary}"));
            cx.start(&atomic);
            cx.emit(format!("call void {sync}(ptr {object})"));
            cx.emit(format!("br label %{done}"));
            cx.start(&ordinary);
            cx.emit(format!("call void {plain}(ptr {object})"));
            cx.emit(format!("br label %{done}"));
            cx.start(&done);
        }
        _ => unreachable!("a count helper is `retain` or `release`"),
    }
    cx.emit("ret void");
    let attrs = crate::func::fn_attrs(cx.m);
    let text = format!(
        "define internal void {}(ptr %p0) {attrs} alwaysinline {{\nentry:\n{}  br label %start\nstart:\n{}}}\n\n",
        global_name(&symbol),
        cx.allocas_text(),
        cx.out_text()
    );
    let _ = writeln!(cx.m.fns, "{}", text.trim_end());
    cx.m.fns.push('\n');
    global_name(&symbol)
}
