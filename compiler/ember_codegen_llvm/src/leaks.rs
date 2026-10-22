//! `[WK-8]`, `[WK-15]`, `[WK-4]` — the debug leak check's ownership edges: for each class and each
//! `Shared[T]`, a function that reports every reference one live object holds (the runtime's
//! `debug_edge_enumerator`), registered with the type's information when the program starts, so the
//! runtime can name a leaked cycle by its fields. Only built with the leak check: a program without
//! it carries none of this.

use crate::func::FnCx;
use crate::types::Sc;
use crate::{Module, Result, global_name};
use ember_types::{ClassId, StructId, Ty, TyKind};

const STRONG: u32 = 1;
const WEAK: u32 = 2;
const UNKNOWN: u32 = 3;

/// The enumerator of `id`'s objects: each field, labelled `Class.field`; a strong field holding
/// another class is predicted from its declaration, with its `Weak` replacement named.
fn class_edges(m: &mut Module<'_>, id: ClassId) -> Result<String> {
    let symbol = format!("ember.edges.{}", m.ty.table.mangled_class(id));
    let table = m.ty.table;
    let offsets = table.class_field_offsets(id);
    crate::glue::define_glue(m, &symbol, &["ptr", "ptr", "ptr"], "", |cx| {
        for index in 0..table.class_field_count(id) {
            let Some((owner, field)) = table.class_field_at_info(id, index) else { continue };
            let label = format!("{}.{}", table.class_def(owner).name, field.name);
            let replacement = match (table.class_def(owner).origin.is_none(), table.kind(field.ty)) {
                (true, TyKind::Class(target)) => Some(format!("Weak[{}]", table.class_def(*target).name)),
                _ => None,
            };
            let at = cx.gep("%p0", 24 + offsets[index]);
            cx.edges_of(&at, field.ty, &label, replacement.as_deref(), true)?;
        }
        Ok(())
    })?;
    Ok(global_name(&symbol))
}

/// The enumerator of a `Shared[T]`'s objects: what its value holds.
fn shared_edges(m: &mut Module<'_>, id: StructId, inner: Ty) -> Result<String> {
    let symbol = format!("ember.edges.{}", m.ty.table.mangled_struct(id));
    let label = format!("Shared[{}].value", m.ty.table.display(inner));
    let (_, align) = m.ty.size_align(inner);
    crate::glue::define_glue(m, &symbol, &["ptr", "ptr", "ptr"], "", |cx| {
        let payload = cx.gep("%p0", crate::glue::payload_offset(align.max(1)));
        cx.edges_of(&payload, inner, &label, None, false)
    })?;
    Ok(global_name(&symbol))
}

impl<'a> FnCx<'_, 'a> {
    /// One edge to the visitor (`%p1`, with its context `%p2`).
    fn visit(&mut self, target: &str, field: &str, kind: u32, replacement: Option<&str>, predicted: bool) {
        let field = self.m.bytes(field.as_bytes());
        let replacement = match replacement {
            Some(text) => self.m.bytes(text.as_bytes()),
            None => "null".to_string(),
        };
        self.emit(format!("call void %p1(ptr %p2, ptr {target}, ptr {field}, i32 {kind}, ptr {replacement}, i1 zeroext {predicted})"));
    }

    /// The edges the value of `ty` at `at` holds.
    fn edges_of(&mut self, at: &str, ty: Ty, field: &str, replacement: Option<&str>, predicted: bool) -> Result<()> {
        let table = self.m.ty.table;
        match table.kind(ty).clone() {
            TyKind::Class(_) => {
                let target = self.load(Sc::Ptr, at, 8);
                self.visit(&target, field, STRONG, replacement, predicted);
            }
            // An erased class-interface field is a strong handle whose class is not predicted.
            TyKind::ClassInterface(_) => {
                let target = self.load(Sc::Ptr, at, 8);
                self.visit(&target, field, STRONG, None, false);
            }
            TyKind::Vec { elem, .. } => {
                let (size, _) = self.m.ty.size_align(elem);
                let buffer = self.load(Sc::Ptr, at, 8);
                let len_at = self.gep(at, 8);
                let len = self.load(Sc::I(64), &len_at, 8);
                let present = self.value(format!("icmp ne ptr {buffer}, null"));
                let (walk, done) = (self.label(), self.label());
                self.emit(format!("br i1 {present}, label %{walk}, label %{done}"));
                self.start(&walk);
                let field = field.to_string();
                self.counted_loop(&len, |cx, index| {
                    let offset = cx.value(format!("mul i64 {index}, {size}"));
                    let element = cx.value(format!("getelementptr i8, ptr {buffer}, i64 {offset}"));
                    cx.edges_of(&element, elem, &field, None, predicted)
                })?;
                self.emit(format!("br label %{done}"));
                self.start(&done);
            }
            TyKind::Struct(id) => {
                if crate::func::shared_inner(table, id).is_some() {
                    let target = self.load(Sc::Ptr, at, 8);
                    self.visit(&target, field, STRONG, None, predicted);
                } else if crate::glue::weak_inner(table, id).is_some() {
                    let target = self.load(Sc::Ptr, at, 8);
                    self.visit(&target, field, WEAK, None, true);
                } else if let Some(inner) = table.compiler_box_inner(id) {
                    // A `Box[dyn I]`'s value is the dynamic type's: the edge is unknown.
                    let pointer = self.load(Sc::Ptr, at, 8);
                    let present = self.value(format!("icmp ne ptr {pointer}, null"));
                    let (walk, done) = (self.label(), self.label());
                    self.emit(format!("br i1 {present}, label %{walk}, label %{done}"));
                    self.start(&walk);
                    if matches!(table.kind(inner), TyKind::Dyn { .. }) {
                        self.visit("null", field, UNKNOWN, None, false);
                    } else {
                        self.edges_of(&pointer, inner, field, None, predicted)?;
                    }
                    self.emit(format!("br label %{done}"));
                    self.start(&done);
                } else {
                    let fields: Vec<Ty> = table.struct_def(id).fields.iter().map(|field| field.ty).collect();
                    let offsets = table.layout(ty).field_offsets;
                    for (nested, offset) in fields.iter().zip(offsets) {
                        let place = self.gep(at, offset);
                        self.edges_of(&place, *nested, field, None, predicted)?;
                    }
                }
            }
            TyKind::Enum(id) => {
                let def = table.enum_def(id).clone();
                let niche = table.option_niche(id).is_some();
                let offsets = if niche { vec![vec![0; 1]; def.variants.len()] } else { table.variant_field_offsets(id) };
                let holds: Vec<usize> =
                    (0..def.variants.len()).filter(|&variant| def.variants[variant].fields.iter().any(|f| self.holds_edges(f.ty))).collect();
                if holds.is_empty() {
                    return Ok(());
                }
                let place = crate::func::PlaceRef { ptr: at.to_string(), ty, variant: None, align: self.m.ty.size_align(ty).1 };
                let (discriminant, bits) = self.discriminant_at(&place)?;
                let done = self.label();
                for variant in holds {
                    let this = self.value(format!("icmp eq i{bits} {discriminant}, {}", crate::func::int_literal(def.variants[variant].discriminant as u128, bits)));
                    let (walk, next) = (self.label(), self.label());
                    self.emit(format!("br i1 {this}, label %{walk}, label %{next}"));
                    self.start(&walk);
                    for (index, nested) in def.variants[variant].fields.iter().enumerate() {
                        let place = self.gep(at, offsets[variant].get(index).copied().unwrap_or(0));
                        self.edges_of(&place, nested.ty, field, None, predicted)?;
                    }
                    self.emit(format!("br label %{done}"));
                    self.start(&next);
                }
                self.emit(format!("br label %{done}"));
                self.start(&done);
            }
            TyKind::Tuple(items) => {
                let offsets = table.layout(ty).field_offsets;
                for (item, offset) in items.iter().zip(offsets) {
                    let place = self.gep(at, offset);
                    self.edges_of(&place, *item, field, None, predicted)?;
                }
            }
            TyKind::Array { elem, len } => {
                let (size, _) = self.m.ty.size_align(elem);
                if !self.holds_edges(elem) {
                    return Ok(());
                }
                let field = field.to_string();
                let at = at.to_string();
                self.counted_loop(&len.to_string(), |cx, index| {
                    let offset = cx.value(format!("mul i64 {index}, {size}"));
                    let element = cx.value(format!("getelementptr i8, ptr {at}, i64 {offset}"));
                    cx.edges_of(&element, elem, &field, None, predicted)
                })?;
            }
            TyKind::Dyn { .. } | TyKind::Param { .. } | TyKind::Assoc { .. } => self.visit("null", field, UNKNOWN, None, false),
            _ => {}
        }
        Ok(())
    }

    /// Whether a value of `ty` can hold an edge.
    fn holds_edges(&self, ty: Ty) -> bool {
        let table = self.m.ty.table;
        match table.kind(ty) {
            TyKind::Class(_) | TyKind::ClassInterface(_) | TyKind::Dyn { .. } | TyKind::Param { .. } | TyKind::Assoc { .. } => true,
            TyKind::Vec { elem, .. } | TyKind::Array { elem, .. } => self.holds_edges(*elem),
            TyKind::Tuple(items) => items.iter().any(|item| self.holds_edges(*item)),
            TyKind::Struct(id) => {
                crate::func::shared_inner(table, *id).is_some()
                    || crate::glue::weak_inner(table, *id).is_some()
                    || table.compiler_box_inner(*id).is_some()
                    || table.struct_def(*id).fields.iter().any(|field| self.holds_edges(field.ty))
            }
            TyKind::Enum(id) => table.enum_def(*id).variants.iter().any(|variant| variant.fields.iter().any(|field| self.holds_edges(field.ty))),
            _ => false,
        }
    }

    /// `[WK-8]` — at start, each class's and each `Shared[T]`'s enumerator registered with its type
    /// information.
    pub(crate) fn register_edges(&mut self) -> Result<()> {
        let table = self.m.ty.table;
        let classes: Vec<ClassId> = table.runtime_classes().map(|(id, _)| id).collect();
        for id in classes {
            let info = global_name(&crate::glue::type_info(self.m, id));
            let edges = class_edges(self.m, id)?;
            self.call_rt("debug_register_type_edges", vec![crate::func::Val::Imm(Sc::Ptr, info), crate::func::Val::Imm(Sc::Ptr, edges)], None);
        }
        let shareds: Vec<(StructId, Ty)> = table.structs().filter_map(|(id, _)| crate::func::shared_inner(table, id).map(|inner| (id, inner))).collect();
        for (id, inner) in shareds {
            if table.is_generic(inner) {
                continue;
            }
            let info = crate::glue::shared_type_info(self.m, id)?;
            let edges = shared_edges(self.m, id, inner)?;
            self.call_rt("debug_register_type_edges", vec![crate::func::Val::Imm(Sc::Ptr, info), crate::func::Val::Imm(Sc::Ptr, edges)], None);
        }
        Ok(())
    }
}
