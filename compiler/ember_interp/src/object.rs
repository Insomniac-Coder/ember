//! Class objects and counted values, as the runtime keeps them (`ember_rt.h`): a 24-byte header
//! (`strong`, `weak`, `access`, `flags`, then the type information), each class level's `u32`
//! access words before its fields (`[EXC-19]`); and drop glue, retain glue and the access
//! checks, as the C backend writes them (`drop_lines`, `retain_lines_for_value`, `access_call`).

use ember_mir::Place;
use ember_span::Span;
use ember_types::{ClassId, Niche, StructId, Ty, TyKind, TypeTable};

use crate::memory::{self, Region};
use crate::{fault, Interp, Loc, Stop, R};

/// The object header's size (`ember_obj_header`, 64-bit target, no reload).
pub const HEADER: u64 = 24;

/// Where the runtime's object operations say a panic of theirs is (`object_panic_text`).
const OBJECT_RUNTIME: &str = "<object-runtime>";
const STRONG: u64 = 0;
const WEAK: u64 = 4;
const ACCESS: u64 = 8;
const FLAGS: u64 = 12;
const TYPE_INFO: u64 = 16;
const DEINITIALISING: u32 = 1;
const WRITER: u32 = 1 << 31;
const READERS: u32 = WRITER - 1;
/// Type-information indices from here up are `Shared` values' (by struct id), below it classes'.
const SHARED_INFO: usize = 1 << 20;

/// `EMBER_OBJ_PAYLOAD_OFFSET`: a `Shared` payload's offset, after the header.
pub fn payload_offset(align: u64) -> u64 {
    let align = align.max(1);
    (HEADER + align - 1) & !(align - 1)
}

pub fn shared_inner(types: &TypeTable, id: StructId) -> Option<Ty> {
    match &types.struct_def(id).origin {
        Some((name, args)) if (name.is("Shared") || name.is("SyncShared")) && args.len() == 1 => Some(args[0]),
        _ => None,
    }
}

pub fn weak_inner(types: &TypeTable, id: StructId) -> Option<Ty> {
    match &types.struct_def(id).origin {
        Some((name, args)) if name.is("Weak") && args.len() == 1 => Some(args[0]),
        _ => None,
    }
}

/// What a type-information address names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Info {
    Class(ClassId),
    Shared(StructId),
}

/// A class's object layout: its size and alignment, each field's offset from the object's start,
/// and each field's access word's, if it has one.
#[derive(Clone, Debug)]
pub struct ClassLayout {
    pub size: u64,
    pub align: u64,
    pub fields: Vec<u64>,
    pub words: Vec<Option<u64>>,
}

#[derive(Default)]
pub struct Objects {
    layouts: std::collections::HashMap<ClassId, ClassLayout>,
    /// `[EXC-6]` — in `debug`, each live field access: its word, whether it writes, and where it
    /// began (`ember_debug_field_begin`).
    accesses: Vec<(u64, bool, Span)>,
    /// `[WK-15]` — with the leak check, each live object and the order it was made in.
    pub live: crate::memory::WordMap<u64, u64>,
    pub sequence: u64,
}

impl Interp<'_, '_> {
    pub(crate) fn class_layout(&mut self, id: ClassId) -> ClassLayout {
        if let Some(layout) = self.objects.layouts.get(&id) {
            return layout.clone();
        }
        let types = self.types;
        let count = types.class_field_count(id);
        let owner = |index: usize| types.class_field_at_info(id, index).map(|(owner, _)| owner);
        let mut fields = Vec::with_capacity(count);
        let mut words = vec![None; count];
        let mut at = HEADER;
        let mut align = 8u64;
        let mut start = 0;
        while start < count {
            let mut end = start + 1;
            while end < count && owner(end) == owner(start) {
                end += 1;
            }
            let worded: Vec<usize> = (start..end).filter(|&index| types.class_field_has_access_word(id, index)).collect();
            for (k, &index) in worded.iter().enumerate() {
                words[index] = Some(at + 4 * k as u64);
            }
            let n = worded.len() as u64;
            at += 4 * (n + n % 2);
            for index in start..end {
                if let Some(field) = types.class_field_at(id, index) {
                    let layout = self.layout(field.ty);
                    let a = layout.align.max(1);
                    at = at.div_ceil(a) * a;
                    align = align.max(a);
                    fields.push(at);
                    at += layout.size;
                }
            }
            start = end;
        }
        let layout = ClassLayout { size: at.div_ceil(align) * align, align, fields, words };
        self.objects.layouts.insert(id, layout.clone());
        layout
    }

    fn info_address(info: Info) -> u64 {
        match info {
            Info::Class(id) => memory::address(Region::TypeInfo, id.0 as usize),
            Info::Shared(id) => memory::address(Region::TypeInfo, SHARED_INFO + id.0 as usize),
        }
    }

    pub(crate) fn object_info(&self, object: u64) -> R<Info> {
        let ti = self.mem.read_u64(object + TYPE_INFO)?;
        match memory::decode(ti) {
            Some((Region::TypeInfo, index)) if index >= SHARED_INFO => Ok(Info::Shared(StructId((index - SHARED_INFO) as u32))),
            Some((Region::TypeInfo, index)) => Ok(Info::Class(ClassId(index as u32))),
            _ => fault(format!("an object at {object:#x} with no type information")),
        }
    }

    pub(crate) fn object_class(&self, object: u64) -> R<ClassId> {
        match self.object_info(object)? {
            Info::Class(id) => Ok(id),
            Info::Shared(_) => fault("a class operation on a shared value"),
        }
    }

    /// `ember_obj_new`: a zeroed object, one strong and one weak reference.
    pub(crate) fn new_object(&mut self, id: ClassId) -> R<u64> {
        let layout = self.class_layout(id);
        let object = self.mem.alloc(layout.size, layout.align)?;
        self.mem.write_u32(object + STRONG, 1)?;
        self.mem.write_u32(object + WEAK, 1)?;
        self.mem.write_u64(object + TYPE_INFO, Self::info_address(Info::Class(id)))?;
        self.track_object(object);
        Ok(object)
    }

    /// `ember_obj_new_copy`: a `Shared` value's object holding a copy of `value`.
    pub(crate) fn new_shared(&mut self, id: StructId, inner: Ty, value: &[u8]) -> R<u64> {
        let layout = self.layout(inner);
        let offset = payload_offset(layout.align);
        let object = self.mem.alloc(offset + layout.size, layout.align.max(8))?;
        self.mem.write_u32(object + STRONG, 1)?;
        self.mem.write_u32(object + WEAK, 1)?;
        self.mem.write_u64(object + TYPE_INFO, Self::info_address(Info::Shared(id)))?;
        self.mem.write(object + offset, value)?;
        self.track_object(object);
        Ok(object)
    }

    pub(crate) fn retain_object(&mut self, object: u64) -> R<()> {
        if object == 0 {
            return Ok(());
        }
        let strong = self.mem.read_u32(object + STRONG)?;
        // `[RT-7]` — the compiled runtime's limit and text, its limit's digits grouped.
        if strong >= count_max() {
            let name = self.object_name(object)?;
            return self.panic_in(OBJECT_RUNTIME, &format!("too many owners of a `{name}` ({})", grouped(count_max())));
        }
        if strong == 0 || self.mem.read_u32(object + FLAGS)? & DEINITIALISING != 0 && strong == 0 {
            return self.panic_in(OBJECT_RUNTIME, "retain of deinitialising or invalid object");
        }
        self.mem.write_u32(object + STRONG, strong + 1)?;
        Ok(())
    }

    pub(crate) fn release_object(&mut self, object: u64) -> R<()> {
        if object == 0 {
            return Ok(());
        }
        let strong = self.mem.read_u32(object + STRONG)?;
        if strong == 0 {
            return self.panic_in(OBJECT_RUNTIME, "release of object with no strong references");
        }
        self.mem.write_u32(object + STRONG, strong - 1)?;
        if strong == 1 {
            self.deinit(object)?;
        }
        Ok(())
    }

    fn deinit(&mut self, object: u64) -> R<()> {
        let flags = self.mem.read_u32(object + FLAGS)?;
        if flags & DEINITIALISING != 0 {
            return self.panic_in(OBJECT_RUNTIME, "recursive object deinitialisation");
        }
        self.mem.write_u32(object + FLAGS, flags | DEINITIALISING)?;
        match self.object_info(object)? {
            Info::Class(id) => {
                self.class_user_drop(object, id)?;
                self.check_resurrection(object)?;
                self.class_drop_fields(object, id)?;
                self.check_resurrection(object)?;
            }
            Info::Shared(id) => {
                let inner = shared_inner(self.types, id).expect("a shared value's type");
                let offset = payload_offset(self.layout(inner).align);
                self.drop_at(object + offset, inner)?;
            }
        }
        self.weak_release(object)
    }

    fn check_resurrection(&mut self, object: u64) -> R<()> {
        if self.mem.read_u32(object + STRONG)? != 0 {
            let name = self.object_name(object)?;
            return self.panic_in(OBJECT_RUNTIME, &format!("E-panic: object resurrected during drop: {name}"));
        }
        Ok(())
    }

    /// The class's name as the type information gives it.
    pub(crate) fn object_name(&self, object: u64) -> R<String> {
        Ok(match self.object_info(object)? {
            Info::Class(id) => self.types.class_def(id).name.to_string(),
            Info::Shared(id) => self.types.display(self.types.find(&TyKind::Struct(id)).unwrap_or(self.common.void)),
        })
    }

    /// `[CLS-6]` — the drop adapter: the class's own `drop`, then each base's, under a write
    /// access to every field (`[EXC-15]`).
    fn class_user_drop(&mut self, object: u64, id: ClassId) -> R<()> {
        let mut chain = Vec::new();
        let mut current = Some(id);
        while let Some(class) = current {
            let def = self.types.class_def(class);
            if def.has_drop {
                chain.push(class);
            }
            current = def.base;
        }
        if chain.is_empty() {
            return Ok(());
        }
        let sync = self.types.class_def(id).is_sync;
        let span = self.by_symbol.get(self.types.class_method_symbol(chain[0], "drop").as_str()).map_or(Span::DUMMY, |&i| self.bodies[i].span);
        if !sync {
            self.object_access(object, true, true, span)?;
        }
        for class in chain {
            let symbol = self.types.class_method_symbol(class, "drop");
            let Some(&index) = self.by_symbol.get(symbol.as_str()) else { continue };
            let by_address = self.bodies[index].locals.get(1).is_none_or(|receiver| matches!(self.types.kind(receiver.ty), TyKind::Ref { .. }));
            // The handle's address: a slot holding it.
            let argument = if by_address {
                let slot = self.mem.alloc(8, 8)?;
                self.mem.write_u64(slot, object)?;
                let result = self.invoke(index, vec![slot.to_le_bytes().to_vec()]);
                self.mem.free(slot)?;
                result?;
                continue;
            } else {
                object.to_le_bytes().to_vec()
            };
            self.invoke(index, vec![argument])?;
        }
        if !sync {
            self.object_access(object, true, false, span)?;
        }
        Ok(())
    }

    /// The field half of `[CLS-6]`: the most derived class's fields first, each level's in
    /// reverse order.
    fn class_drop_fields(&mut self, object: u64, id: ClassId) -> R<()> {
        let layout = self.class_layout(id);
        let mut current = Some(id);
        while let Some(class) = current {
            let def = self.types.class_def(class);
            let first = self.types.class_field_count(class) - def.fields.len();
            for (i, field) in def.fields.iter().enumerate().rev() {
                if self.types.needs_drop(field.ty) {
                    self.drop_at(object + layout.fields[first + i], field.ty)?;
                }
            }
            current = def.base;
        }
        Ok(())
    }

    pub(crate) fn weak_retain(&mut self, object: u64) -> R<()> {
        if object == 0 {
            return Ok(());
        }
        let weak = self.mem.read_u32(object + WEAK)?;
        if weak == 0 {
            return self.panic_in(OBJECT_RUNTIME, "retain of invalid weak reference");
        }
        // `[RT-7]` — as the compiled runtime: the link past the weak count's maximum stops the program.
        if weak >= count_max() {
            let name = self.object_name(object)?;
            return self.panic_in(OBJECT_RUNTIME, &format!("too many `weakref` links to a `{name}` ({})", grouped(count_max())));
        }
        self.mem.write_u32(object + WEAK, weak + 1)?;
        Ok(())
    }

    pub(crate) fn weak_release(&mut self, object: u64) -> R<()> {
        if object == 0 {
            return Ok(());
        }
        let weak = self.mem.read_u32(object + WEAK)?;
        if weak == 0 {
            return self.panic_in(OBJECT_RUNTIME, "release of object with no weak references");
        }
        self.mem.write_u32(object + WEAK, weak - 1)?;
        if weak == 1 && self.mem.read_u32(object + STRONG)? == 0 {
            self.untrack_object(object);
            self.mem.free(object)?;
        }
        Ok(())
    }

    /// `ember_weak_upgrade`: the object retained, or null when it is gone.
    pub(crate) fn weak_upgrade(&mut self, object: u64) -> R<u64> {
        if object == 0 {
            return Ok(0);
        }
        let strong = self.mem.read_u32(object + STRONG)?;
        let flags = self.mem.read_u32(object + FLAGS)?;
        if strong == 0 || strong == u32::MAX || flags & DEINITIALISING != 0 {
            return Ok(0);
        }
        self.mem.write_u32(object + STRONG, strong + 1)?;
        Ok(object)
    }

    /// `ember_downcast`: the object, if its class is `target` or derives from it.
    pub(crate) fn downcast(&self, object: u64, target: ClassId) -> R<u64> {
        if object == 0 {
            return Ok(0);
        }
        let mut current = Some(self.object_class(object)?);
        while let Some(class) = current {
            if class == target {
                return Ok(object);
            }
            current = self.types.class_def(class).base;
        }
        Ok(0)
    }

    // -- retain and drop glue ---------------------------------------------------------

    /// `retain_lines_for_value`: each class handle a value of `ty` at `addr` holds, retained.
    pub(crate) fn retain_at(&mut self, addr: u64, ty: Ty) -> R<()> {
        // What holds no counted handle needs no drop: most copies.
        if !self.needs_drop(ty) {
            return Ok(());
        }
        match self.types.kind(ty).clone() {
            TyKind::Class(_) | TyKind::ClassInterface(_) => {
                let object = self.mem.read_u64(addr)?;
                self.retain_object(object)
            }
            TyKind::Struct(id) => {
                if shared_inner(self.types, id).is_some() {
                    let object = self.mem.read_u64(addr)?;
                    return self.retain_object(object);
                }
                if weak_inner(self.types, id).is_some() {
                    let object = self.mem.read_u64(addr)?;
                    return self.weak_retain(object);
                }
                let def = self.types.struct_def(id);
                if def.name.as_str().starts_with("MaybeUninit_") || !def.drops_fields {
                    return Ok(());
                }
                let fields: Vec<Ty> = def.fields.iter().map(|f| f.ty).collect();
                let offsets = self.layout(ty).field_offsets.clone();
                for (field, offset) in fields.into_iter().zip(offsets) {
                    self.retain_at(addr + offset, field)?;
                }
                Ok(())
            }
            TyKind::Tuple(items) => {
                let offsets = self.layout(ty).field_offsets.clone();
                for (item, offset) in items.into_iter().zip(offsets) {
                    self.retain_at(addr + offset, item)?;
                }
                Ok(())
            }
            TyKind::Array { elem, len } => {
                let step = self.size(elem);
                for index in 0..len {
                    self.retain_at(addr + index * step, elem)?;
                }
                Ok(())
            }
            TyKind::Enum(id) => {
                let def = self.types.enum_def(id);
                if def.is_unit_only() {
                    return Ok(());
                }
                let loc = Loc::at(addr, ty);
                let tag = self.discriminant(loc)?;
                let Some(variant) = def.variants.iter().position(|v| v.discriminant == tag) else { return Ok(()) };
                let fields: Vec<Ty> = def.variants[variant].fields.iter().map(|f| f.ty).collect();
                let niche = self.types.option_niche(id).is_some();
                let offsets = if niche { Vec::new() } else { self.types.variant_field_offsets(id).swap_remove(variant) };
                for (index, field) in fields.into_iter().enumerate() {
                    let at = if niche { addr } else { addr + offsets[index] };
                    self.retain_at(at, field)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// `drop_lines`: what has to happen for the value of `ty` at `addr` to release everything
    /// it owns (`[DRP-2]`).
    pub(crate) fn drop_at(&mut self, addr: u64, ty: Ty) -> R<()> {
        match self.types.kind(ty).clone() {
            TyKind::Fn { abi: None, .. } => self.drop_callable(addr),
            TyKind::Class(_) | TyKind::ClassInterface(_) => {
                let object = self.mem.read_u64(addr)?;
                self.release_object(object)
            }
            TyKind::Vec { elem, .. } => {
                let (ptr, len) = (self.mem.read_u64(addr)?, self.mem.read_u64(addr + 8)?);
                if self.types.needs_drop(elem) {
                    let step = self.size(elem);
                    for index in 0..len {
                        self.drop_at(ptr + index * step, elem)?;
                    }
                }
                self.vec_free(addr)
            }
            TyKind::Struct(id) => self.drop_struct(addr, ty, id),
            TyKind::Tuple(items) => {
                let offsets = self.layout(ty).field_offsets.clone();
                for (index, item) in items.iter().enumerate().rev() {
                    if self.types.needs_drop(*item) {
                        self.drop_at(addr + offsets[index], *item)?;
                    }
                }
                Ok(())
            }
            TyKind::Array { elem, len } => {
                if !self.types.needs_drop(elem) {
                    return Ok(());
                }
                let step = self.size(elem);
                for index in 0..len {
                    self.drop_at(addr + index * step, elem)?;
                }
                Ok(())
            }
            TyKind::Enum(id) => {
                let def = self.types.enum_def(id);
                if def.has_drop {
                    self.call_drop_method(addr, ty)?;
                }
                if def.is_unit_only() {
                    return Ok(());
                }
                let tag = self.discriminant(Loc::at(addr, ty))?;
                let Some(variant) = def.variants.iter().position(|v| v.discriminant == tag) else { return Ok(()) };
                let fields: Vec<Ty> = def.variants[variant].fields.iter().map(|f| f.ty).collect();
                let niche = self.types.option_niche(id).is_some();
                let offsets = if niche { Vec::new() } else { self.types.variant_field_offsets(id).swap_remove(variant) };
                for (index, field) in fields.into_iter().enumerate().rev() {
                    if self.types.needs_drop(field) {
                        let at = if niche { addr } else { addr + offsets[index] };
                        self.drop_at(at, field)?;
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn drop_struct(&mut self, addr: u64, ty: Ty, id: StructId) -> R<()> {
        let types = self.types;
        let def = types.struct_def(id);
        if shared_inner(types, id).is_some() {
            let object = self.mem.read_u64(addr)?;
            return self.release_object(object);
        }
        if weak_inner(types, id).is_some() {
            let object = self.mem.read_u64(addr)?;
            return self.weak_release(object);
        }
        if let Some(inner) = types.compiler_box_inner(id) {
            if matches!(types.kind(inner), TyKind::Dyn { .. }) {
                let (data, table) = (self.mem.read_u64(addr)?, self.mem.read_u64(addr + 8)?);
                return self.drop_dyn_box(data, table);
            }
            if def.fields.len() == 1
                && let TyKind::Ptr { inner, .. } = types.kind(def.fields[0].ty)
            {
                let payload = self.mem.read_u64(addr)?;
                if types.needs_drop(*inner) {
                    self.drop_at(payload, *inner)?;
                }
                return self.mem.free(payload).map_err(Stop::from);
            }
        }
        let origin = def.origin.as_ref().map(|(name, _)| name.as_str().to_string());
        match origin.as_deref() {
            Some("std.sync.MutexGuard") => return self.guard_drop(addr, ty, "std.sync.Mutex", true),
            Some("std.sync.RwLockReadGuard") => return self.guard_drop(addr, ty, "std.sync.RwLock", false),
            Some("std.sync.RwLockWriteGuard") => return self.guard_drop(addr, ty, "std.sync.RwLock", true),
            Some("SoA") => return self.soa_drop(addr, ty),
            Some("std.thread.Scope" | "std.jobs.JobScope" | "std.thread.JoinHandle") => {
                return self.system_drop(addr, ty, origin.as_deref().unwrap_or(""));
            }
            _ => {}
        }
        let name = def.name.as_str();
        // `[ARN-1]` — an arena's end frees its chunks; `[ARN-6]` a scope's rewinds its arena.
        if name == "Arena" {
            let state = self.mem.read_u64(addr + self.field_offset(ty, "state")?)?;
            return self.arena_free(state);
        }
        if name == "ScopedArena" {
            let state = self.mem.read_u64(addr + self.field_offset(ty, "state")?)?;
            let mark = self.mem.read_u64(addr + self.field_offset(ty, "mark")?)?;
            return self.arena_rewind(state, mark);
        }
        if (name.starts_with("RefMut_") || name.starts_with("Ref_")) && def.fields.len() == 1 {
            // `[CELL-7]` — the guard points at the cell's value, at offset 0 of the cell, whose
            // borrow counter is its field `borrow`.
            if let Some(cell) = self.refcell_for_guard(ty) {
                let value = self.mem.read_u64(addr)?;
                let layout = self.layout(cell);
                let TyKind::Struct(cell_id) = *types.kind(cell) else { unreachable!() };
                let borrow_index = types.struct_def(cell_id).fields.iter().position(|f| f.name.is("borrow")).unwrap_or(1);
                let field = types.cell_held(types.struct_def(cell_id).fields[borrow_index].ty);
                let at = value + layout.field_offsets[borrow_index];
                let kind = self.int_kind(field).expect("a counter");
                let current = kind.signed_value(self.mem.read(at, kind.bytes() as u64)?);
                let next = if name.starts_with("RefMut_") { 0 } else { current - 1 };
                self.mem.write(at, &kind.encode(next as u128))?;
                return Ok(());
            }
        }
        if def.has_drop {
            self.call_drop_method(addr, ty)?;
        }
        if def.drops_fields {
            let fields: Vec<Ty> = def.fields.iter().map(|f| f.ty).collect();
            let offsets = self.layout(ty).field_offsets.clone();
            for (index, field) in fields.iter().enumerate().rev() {
                if types.needs_drop(*field) {
                    self.drop_at(addr + offsets[index], *field)?;
                }
            }
        }
        Ok(())
    }

    /// A struct's or enum's own `drop(mut self)`, on the value at `addr`.
    fn call_drop_method(&mut self, addr: u64, ty: Ty) -> R<()> {
        let symbol = self.types.method_symbol(ty, None, "drop", None);
        let Some(&index) = self.by_symbol.get(symbol.as_str()) else {
            return fault(format!("`{}` has no `drop` body `{symbol}`", self.types.display(ty)));
        };
        let by_address = self.bodies[index].locals.get(1).is_none_or(|receiver| matches!(self.types.kind(receiver.ty), TyKind::Ref { .. }));
        let argument = if by_address { addr.to_le_bytes().to_vec() } else { self.mem.read(addr, self.size(ty))?.to_vec() };
        self.invoke(index, vec![argument])?;
        Ok(())
    }

    /// The `RefCell` whose `Ref`/`RefMut` guard `guard` is (`refcell_for_guard`).
    fn refcell_for_guard(&self, guard: Ty) -> Option<Ty> {
        let TyKind::Struct(guard_id) = *self.types.kind(guard) else { return None };
        let field = self.types.struct_def(guard_id).fields.first()?;
        let TyKind::Ref { inner, .. } = *self.types.kind(field.ty) else { return None };
        self.types.all().find_map(|(ty, kind)| match kind {
            TyKind::Struct(id) => {
                let def = self.types.struct_def(*id);
                (def.name.as_str().starts_with("RefCell_") && def.fields.len() == 4 && self.types.cell_held(def.fields[0].ty) == inner).then_some(ty)
            }
            _ => None,
        })
    }

    // -- niches -----------------------------------------------------------------

    /// Whether the niche `Option` at `loc` is `None`.
    pub(crate) fn niche_holds(&self, niche: Niche, loc: Loc) -> R<bool> {
        let payload = niche.payload;
        let at = loc.addr;
        Ok(match self.types.kind(payload) {
            TyKind::Bool => self.mem.read(at, 1)?[0] == 2,
            TyKind::Char => self.mem.read_u32(at)? == 0x11_0000,
            TyKind::Span { .. } | TyKind::Str => self.mem.read_u64(at)? == 0 && self.mem.read_u64(at + 8)? == u64::MAX,
            TyKind::Ref { inner, .. } if matches!(self.types.kind(*inner), TyKind::Dyn { .. }) => self.mem.read_u64(at)? == 0,
            TyKind::Class(_) | TyKind::ClassInterface(_) | TyKind::Ref { .. } | TyKind::CStr | TyKind::Fn { abi: Some(_), .. } => {
                self.mem.read_u64(at)? == 0
            }
            TyKind::Enum(id) => {
                let sentinel = self.types.enum_unused_discriminant(*id).expect("an enum niche has an unused discriminant");
                let repr = self.int_kind(self.types.enum_def(*id).repr).expect("an integer repr");
                repr.signed_value(self.mem.read(at, repr.bytes() as u64)?) == sentinel
            }
            TyKind::Range(id) => {
                let repr = self.types.range_def(*id).repr;
                if self.float_bits(repr).is_some() {
                    let bytes = self.mem.read(at, self.size(repr))?;
                    self.float_value(repr, bytes).is_nan()
                } else {
                    let bits = self.types.range_unused_integer(*id).expect("an integer range niche has an unused value");
                    let kind = self.int_kind(repr).expect("an integer repr");
                    kind.unsigned(self.mem.read(at, kind.bytes() as u64)?) == bits & u128::MAX
                }
            }
            TyKind::Struct(id) => {
                if let Some(inner) = self.types.compiler_box_inner(*id) {
                    let _ = inner;
                    self.mem.read_u64(at)? == 0
                } else {
                    let field = self.types.struct_def(*id).fields[0].ty;
                    self.mem.read(at, self.size(field))?.iter().all(|b| *b == 0)
                }
            }
            _ => return fault(format!("a niche in `{}`", self.types.display(payload))),
        })
    }

    /// A niche `Option`'s `None`.
    pub(crate) fn niche_none(&self, niche: Niche) -> R<Vec<u8>> {
        let payload = niche.payload;
        let size = self.size(payload) as usize;
        let mut bytes = vec![0u8; size];
        match self.types.kind(payload) {
            TyKind::Bool => bytes[0] = 2,
            TyKind::Char => bytes.copy_from_slice(&0x11_0000u32.to_le_bytes()),
            TyKind::Span { .. } | TyKind::Str => bytes[8..16].copy_from_slice(&u64::MAX.to_le_bytes()),
            TyKind::Enum(id) => {
                let sentinel = self.types.enum_unused_discriminant(*id).expect("an enum niche has an unused discriminant");
                let repr = self.int_kind(self.types.enum_def(*id).repr).expect("an integer repr");
                bytes = repr.encode(sentinel as u128);
            }
            TyKind::Range(id) => {
                let repr = self.types.range_def(*id).repr;
                if let Some(bits) = self.float_bits(repr) {
                    bytes = match bits {
                        16 => 0x7E00u16.to_le_bytes().to_vec(),
                        32 => f32::NAN.to_bits().to_le_bytes().to_vec(),
                        _ => f64::NAN.to_bits().to_le_bytes().to_vec(),
                    };
                } else {
                    let value = self.types.range_unused_integer(*id).expect("an integer range niche has an unused value");
                    bytes = self.int_kind(repr).expect("an integer repr").encode(value);
                }
            }
            _ => {}
        }
        Ok(bytes)
    }

    // -- access checks (`[EXC-1]`, `[EXC-15]`, `[EXC-19]`, `[HEAP-5]`) ------------------------

    /// `access_call`: a begin or end of a read or write access to `place`.
    pub(crate) fn access(&mut self, place: &Place, mutable: bool, begin: bool, span: Span) -> R<()> {
        let body = self.frames_body();
        let ty = self.place_ty(body, place);
        let is_dyn = |this: &Self, ty: Ty| match this.types.kind(ty) {
            TyKind::Dyn { .. } => true,
            TyKind::Ref { inner, .. } => matches!(this.types.kind(*inner), TyKind::Dyn { .. }),
            TyKind::Struct(id) => this.types.compiler_box_inner(*id).is_some_and(|inner| matches!(this.types.kind(inner), TyKind::Dyn { .. })),
            _ => false,
        };
        // `[EXC-18]` — a `dyn` receiver's object, through its table's `access` entry. D-913: one a
        // class field holds is named with a `Deref` after the field, whose own place is its access
        // word (`[EXC-19]`), which a store into the field takes.
        if let Some((ember_mir::Projection::Deref, prefix)) = place.projection.split_last() {
            let carrier = Place { local: place.local, projection: prefix.to_vec() };
            if is_dyn(self, self.place_ty(body, &carrier)) {
                return self.dyn_access(&carrier, mutable, begin, span);
            }
        }
        let field_with_word = match place.projection.split_last() {
            Some((ember_mir::Projection::Field(index), prefix)) => {
                let handle = Place { local: place.local, projection: prefix.to_vec() };
                matches!(*self.types.kind(self.place_ty(body, &handle)), TyKind::Class(id) if self.types.class_field_has_access_word(id, *index))
            }
            _ => false,
        };
        if is_dyn(self, ty) && !field_with_word {
            return self.dyn_access(place, mutable, begin, span);
        }
        match self.types.kind(ty) {
            TyKind::Class(_) | TyKind::ClassInterface(_) => {
                let loc = self.place(place)?;
                let object = self.pointer_at(loc)?;
                return self.object_access(object, mutable, begin, span);
            }
            _ => {}
        }
        // A class field with its own word.
        if let Some((ember_mir::Projection::Field(index), prefix)) = place.projection.split_last() {
            let handle = Place { local: place.local, projection: prefix.to_vec() };
            if let TyKind::Class(id) = *self.types.kind(self.place_ty(body, &handle))
                && self.types.class_field_has_access_word(id, *index)
            {
                let loc = self.place(&handle)?;
                let object = self.pointer_at(loc)?;
                let layout = self.class_layout(id);
                let word = object + layout.words[*index].expect("a field with a word");
                let field = self.types.class_field_at(id, *index).map(|f| f.name.to_string()).unwrap_or_default();
                let what = format!("{}.{field}", self.types.class_display_name(id));
                return self.word_access(word, mutable, begin, &what, span);
            }
        }
        // A `Shared` payload: its object's header word.
        let loc = self.place(place)?;
        let object = self.pointer_at(loc)?;
        let what = self.types.display(ty);
        self.header_access(object, mutable, begin, &what, span)
    }

    fn frames_body(&self) -> &ember_mir::Body {
        self.body_now()
    }

    /// `ember_access_*`: the header's own word, a `Shared` value's.
    pub(crate) fn header_access(&mut self, object: u64, mutable: bool, begin: bool, what: &str, span: Span) -> R<()> {
        let word = object + ACCESS;
        let value = self.mem.read_u32(word)?;
        let next = match (begin, mutable) {
            (true, false) if value & WRITER != 0 || value & READERS == READERS => None,
            (true, false) => Some(value + 1),
            (true, true) if value != 0 => None,
            (true, true) => Some(WRITER),
            (false, false) if value & WRITER != 0 || value & READERS == 0 => None,
            (false, false) => Some(value - 1),
            (false, true) if value != WRITER => None,
            (false, true) => Some(0),
        };
        match next {
            Some(next) => Ok(self.mem.write_u32(word, next)?),
            None => self.panic(&format!("exclusivity violation: overlapping access to {what}"), span),
        }
    }

    /// `EMBER_FIELD_CONFLICT`: a field access meeting one active on its word; in `debug`, where
    /// the active one began (`ember_panic_field_exclusivity_debug`).
    fn field_conflict<T>(&mut self, writing: bool, word: u64, value: u32, what: &str, span: Span) -> R<T> {
        let active = if value & WRITER != 0 { "write" } else { "read" };
        let mut message = format!(
            "exclusivity violation: {} access to {what} while a {active} access to {what} is active",
            if writing { "write" } else { "read" }
        );
        if self.debug
            && let Some(&(_, prior_writing, at)) = self.objects.accesses.iter().rev().find(|(w, _, _)| *w == word)
        {
            let (file, line, col) = self.location(at).unwrap_or(("<unknown>".to_string(), 0, 0));
            message.push_str(&format!(
                "; active {} of {what} began at {file}:{line}:{col}",
                if prior_writing { "write" } else { "read" }
            ));
        }
        self.panic(&message, span)
    }

    fn track_begin(&mut self, word: u64, writing: bool, span: Span) {
        if self.debug {
            self.objects.accesses.push((word, writing, span));
        }
    }

    fn track_end(&mut self, word: u64, writing: bool) {
        if let Some(at) = self.objects.accesses.iter().rposition(|(w, wr, _)| *w == word && *wr == writing) {
            self.objects.accesses.remove(at);
        }
    }

    /// `ember_field_*`: one field's word.
    fn word_access(&mut self, word: u64, mutable: bool, begin: bool, what: &str, span: Span) -> R<()> {
        let value = self.mem.read_u32(word)?;
        match (begin, mutable) {
            (true, false) => {
                if value & WRITER != 0 || value & READERS == READERS {
                    return self.field_conflict(false, word, value, what, span);
                }
                self.mem.write_u32(word, value + 1)?;
                self.track_begin(word, false, span);
            }
            (true, true) => {
                if value != 0 {
                    return self.field_conflict(true, word, value, what, span);
                }
                self.mem.write_u32(word, WRITER)?;
                self.track_begin(word, true, span);
            }
            (false, false) => {
                if value & WRITER != 0 || value & READERS == 0 {
                    return self.panic(&format!("exclusivity violation: overlapping access to {what}"), span);
                }
                self.mem.write_u32(word, value - 1)?;
                self.track_end(word, false);
            }
            (false, true) => {
                if value != WRITER {
                    return self.panic(&format!("exclusivity violation: overlapping access to {what}"), span);
                }
                self.mem.write_u32(word, 0)?;
                self.track_end(word, true);
            }
        }
        Ok(())
    }

    /// `ember_object_*`: every field word of the object's class, its bases' included.
    pub(crate) fn object_access(&mut self, object: u64, mutable: bool, begin: bool, span: Span) -> R<()> {
        let id = self.object_class(object)?;
        if self.types.class_def(id).is_sync {
            return Ok(());
        }
        let layout = self.class_layout(id);
        let words: Vec<(usize, u64)> = layout.words.iter().enumerate().filter_map(|(i, w)| w.map(|w| (i, object + w))).collect();
        // `field_name`: the type information's name and the field's.
        let name = |interp: &Self, index: usize| {
            let field = interp.types.class_field_at(id, index).map(|f| f.name.to_string()).unwrap_or_default();
            format!("{}.{field}", interp.types.class_def(id).name)
        };
        // Every word is checked before any is changed, as the runtime does.
        for &(index, word) in &words {
            let value = self.mem.read_u32(word)?;
            let bad = match (begin, mutable) {
                (true, true) => value != 0,
                (true, false) => value & WRITER != 0 || value & READERS == READERS,
                (false, true) => value != WRITER,
                (false, false) => value & WRITER != 0 || value & READERS == 0,
            };
            if bad {
                let what = name(self, index);
                if begin {
                    return self.field_conflict(mutable, word, value, &what, span);
                }
                return self.panic(&format!("exclusivity violation: overlapping access to {what}"), span);
            }
            if !begin {
                let next = if mutable { 0 } else { value - 1 };
                self.mem.write_u32(word, next)?;
                self.track_end(word, mutable);
            }
        }
        if begin {
            for &(_, word) in &words {
                let value = self.mem.read_u32(word)?;
                self.mem.write_u32(word, if mutable { WRITER } else { value + 1 })?;
                self.track_begin(word, mutable, span);
            }
        }
        Ok(())
    }
}

/// `[RT-7]`, `[TST-31]` — a count's maximum: 4,294,967,295, or 255 for a test run with 8-bit counts
/// (`#$ count_bits: 8`), as the compiled runtime's `EMBER_COUNT_MAX`.
fn count_max() -> u32 {
    static MAX: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    *MAX.get_or_init(|| if std::env::var(ember_branding::count_bits_var()).is_ok_and(|bits| bits == "8") { 255 } else { u32::MAX })
}

/// `4294967295` as `4,294,967,295`.
fn grouped(value: u32) -> String {
    let digits = value.to_string();
    let mut out = String::new();
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(digit);
    }
    out
}
