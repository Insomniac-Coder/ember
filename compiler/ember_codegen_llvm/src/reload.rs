//! Annex B under the LLVM backend: a reloadable image as IR. What the image says of itself is the
//! planner's (`ember_codegen_c::image`, the decisions the C backend's image makes too); `LlvmWriter`
//! writes them as IR constants and functions. Beside them, the image's own code: each reloadable
//! function's body under a name of its own and a thunk under its symbol (`[HR-5]`, `[HR-6]`), the
//! tables the runtime fills (`[HR-4a]`, `[HR-6]`), and each function's frame record (`[HR-3b]`).

use std::collections::BTreeMap;

use crate::abi::{Return, Signature};
use crate::func::{FnCx, Param, Val, define_c_fn, float_literal, int_literal};
use crate::types::{Sc, offset_align};
use crate::{Module, Result, global_name};
use ember_codegen_c::image::{self, DropCase, FieldShape, FnRow, FrameInfo, Holder, InitCase, Manifest, Member, Num, Planner, Row, StaticRow, TypeRow, VariantRow, Writer};
use ember_mir::Body;
use ember_types::{Bound, ClassId, EnumId, FloatTy, IntTy, RangeId, StructId, Ty, TyKind, UintTy};

/// The runtime's reload records as IR struct types, field for field as `ember_rt.h` declares them
/// (a unit test compares each): every field a pointer, a `uint32_t` or a `uint64_t`, which C and
/// LLVM's x86-64 data layout place alike.
pub(crate) const RECORDS: &[(&str, &[&str])] = &[
    ("rl_field", &["ptr", "ptr", "ptr", "i64", "i32", "i32", "i32", "i32", "i32", "i32", "ptr", "ptr", "ptr", "ptr", "i32", "i32"]),
    ("rl_variant", &["ptr", "ptr", "i64", "i64", "i32", "ptr"]),
    ("rl_type", &["ptr", "i32", "ptr", "i32", "i32", "i32", "i64", "i64", "ptr", "ptr", "i32", "ptr", "i32", "ptr", "ptr", "ptr", "ptr", "ptr", "i64", "ptr"]),
    ("rl_fn", &["ptr", "ptr", "ptr", "ptr", "i64", "i64", "i32"]),
    ("rl_static", &["ptr", "i32", "i32", "i64", "i64", "i32", "ptr", "ptr", "ptr"]),
    ("rl_identity", &["ptr", "ptr"]),
    ("rl_header", &["ptr", "ptr"]),
    ("rl_site", &["i32", "i32", "ptr"]),
    ("rl_frame_info", &["ptr", "i32", "ptr"]),
    ("rl_frame", &["ptr", "ptr", "i32"]),
    (
        "rl_manifest",
        &["i64", "i32", "i32", "i32", "i32", "ptr", "ptr", "ptr", "i32", "ptr", "ptr", "ptr", "ptr", "i32", "ptr", "ptr", "ptr", "i32", "ptr", "i32", "ptr", "ptr", "i32", "ptr"],
    ),
];

/// `EMBER_RELOAD_MAGIC` and `EMBER_RELOAD_PROTOCOL` (a unit test reads both from the header).
pub(crate) const MAGIC: u64 = 0x4441_4F4C_4552_4D45;
pub(crate) const PROTOCOL: u32 = 1;

/// The IR name of a record's type.
fn record(name: &str) -> String {
    format!("%ember.{name}")
}

/// A record's size and alignment, its fields at their natural alignment.
pub(crate) fn record_layout(fields: &[&str]) -> (u64, u64) {
    let mut at = 0u64;
    let mut align = 1u64;
    for field in fields {
        let size = if *field == "i32" { 4 } else { 8 };
        at = at.div_ceil(size) * size;
        at += size;
        align = align.max(size);
    }
    (at.div_ceil(align) * align, align)
}

fn fields_of(name: &str) -> &'static [&'static str] {
    RECORDS.iter().find(|(known, _)| *known == name).map(|(_, fields)| *fields).expect("a reload record")
}

/// The records' type declarations, at the top of an image's module.
pub(crate) fn type_declarations() -> String {
    let mut out = String::new();
    for (name, fields) in RECORDS {
        out.push_str(&format!("{} = type {{ {} }}\n", record(name), fields.join(", ")));
    }
    out
}

/// The tables the runtime fills (`[HR-8]`): each slot's current target, its permanent address and
/// its value's descriptor; each static's storage and its state.
pub(crate) fn tables(m: &mut Module<'_>) {
    let plan = m.reload_plan.as_ref().expect("a reloadable image has its plan");
    let (count, statics) = (plan.slots.len().max(1), plan.static_entries().max(1));
    for (name, length) in [("targets", count), ("addresses", count), ("descriptors", count), ("static_data", statics), ("static_state", statics)] {
        m.global(format!("{} = internal global [{length} x ptr] zeroinitializer, align 8", global_name(&image::table(name))));
    }
}

/// The address of entry `index` of the image's table `name`.
fn slot_address(m: &Module<'_>, name: &str, index: usize) -> String {
    let plan = m.reload_plan.as_ref().expect("a reloadable image has its plan");
    let length = if name.starts_with("static") { plan.static_entries() } else { plan.slots.len() }.max(1);
    format!("getelementptr ([{length} x ptr], ptr {}, i64 0, i64 {index})", global_name(&image::table(name)))
}

/// The planner deciding with `m` writing what it decides.
pub(crate) fn with_plan<'a, R>(m: &mut Module<'a>, work: impl FnOnce(&mut Planner<'a>, &mut LlvmWriter<'_, 'a>) -> R) -> R {
    let mut plan = m.reload_plan.take().expect("a reloadable image has its plan");
    let result = work(&mut plan, &mut LlvmWriter(m));
    m.reload_plan = Some(plan);
    result
}

/// `[HR-6]` — the thunk of a reloadable function: its symbol, which loads the current target from
/// the image's table and jumps to it (`musttail`: one load and one indirect jump, `[HR-9]`).
pub(crate) fn emit_thunk(m: &mut Module<'_>, body: &Body, sig: &Signature) {
    let index = *m.reload_plan.as_ref().and_then(|plan| plan.slots.get(&body.symbol)).expect("a reloadable function has its slot");
    let params = crate::func::ir_params(sig, true);
    let ret = sig.ret.ir();
    let ext = crate::func::ret_attr(sig);
    let target = slot_address(m, "targets", index);
    // A `byval` argument (System V's aggregate in memory) is forwarded by a plain tail call, which
    // LLVM makes a jump where the argument already sits where the target wants it: x86-64's
    // `musttail` lowering of a forwarded `byval` corrupts it (the image crashed calling a method
    // taking a 32-byte `ArenaArray`, Linux only).
    let byval = sig.params.iter().any(|pass| matches!(pass, crate::abi::Pass::Indirect { byval: true, .. }));
    let call = format!("{} call{ext} {ret} %t({})", if byval { "tail" } else { "musttail" }, params.join(", "));
    let tail = if matches!(sig.ret, Return::Void | Return::Sret { .. }) { format!("  {call}\n  ret void\n") } else { format!("  %r = {call}\n  ret {ret} %r\n") };
    // An export's symbol is the one a host's table names (`[HR-21]`).
    let linkage = if body.abi.is_some() { visibility(m) } else { "internal " };
    let attrs = crate::func::fn_attrs(m);
    m.fns.push_str(&format!(
        "; [HR-6] the thunk of `{}`\ndefine {linkage}{} {}({}) {attrs} {{\nentry:\n  %t = load ptr, ptr {target}, align 8\n{tail}}}\n\n",
        body.name.replace('\n', " "),
        crate::func::ret_with_attr(sig),
        global_name(&body.symbol),
        params.join(", ")
    ));
}

/// What a symbol of the image that the runtime finds by its section, not by its name, is: hidden
/// in an ELF image (as the C backend's `-fvisibility=hidden`), as is on Windows, where an image
/// exports nothing.
fn visibility(m: &Module<'_>) -> &'static str {
    match m.ty.target {
        crate::abi::Target::SysV | crate::abi::Target::Aapcs64 => "hidden ",
        crate::abi::Target::Win64 => "",
    }
}

/// `[HR-6]` — a value of a reloadable function: the descriptor the runtime made stand for it in
/// every image, so the value is one whichever image made it, and calling it reaches the permanent
/// thunk. `None` for any other function.
pub(crate) fn fn_value(cx: &mut FnCx<'_, '_>, symbol: &str) -> Option<Val> {
    let index = *cx.m.reload_plan.as_ref()?.slots.get(symbol)?;
    if !cx.m.reload_plan.as_ref()?.is_reloadable(symbol) {
        return None;
    }
    let at = slot_address(cx.m, "descriptors", index);
    let descriptor = cx.value(format!("load ptr, ptr {at}, align 8"));
    let native = cx.load(Sc::Ptr, &descriptor, 8);
    let slot = cx.alloca(40, 8);
    cx.zero(&slot, 8, 40);
    cx.emit(format!("store ptr {native}, ptr {slot}, align 8"));
    let descriptor_at = cx.gep(&slot, 8);
    cx.emit(format!("store ptr {descriptor}, ptr {descriptor_at}, align 8"));
    Some(Val::Mem(slot, 8))
}

/// `[HR-4a]` — static `name`'s storage, the runtime's: its address, loaded from the image's table.
pub(crate) fn static_storage(cx: &mut FnCx<'_, '_>, name: &str) -> String {
    let index = cx.m.reload_plan.as_ref().expect("a reloadable image has its plan").static_index(name);
    let at = slot_address(cx.m, "static_data", index);
    cx.value(format!("load ptr, ptr {at}, align 8"))
}

/// `[HR-4a]` — static `name`'s first-access state, the runtime's.
pub(crate) fn static_state(cx: &mut FnCx<'_, '_>, name: &str) -> String {
    let index = cx.m.reload_plan.as_ref().expect("a reloadable image has its plan").static_index(name);
    let at = slot_address(cx.m, "static_state", index);
    cx.value(format!("load ptr, ptr {at}, align 8"))
}

/// `[HR-3b]` — the frame record of the body being written: the thread's innermost-record slot
/// (`ember_reload_frames()`), the record itself, and each call's site by block.
pub(crate) struct FrameRecord {
    pub head: String,
    pub record: String,
    pub sites: BTreeMap<usize, u32>,
}

/// The frame description of `body`, written beside it.
pub(crate) fn frame_info(m: &mut Module<'_>, body: &Body) -> FrameInfo {
    let info = m.reload_plan.as_ref().expect("a reloadable image has its plan").frame_info(body, m.map);
    LlvmWriter(m).frame_info(&info);
    info
}

/// `[HR-9]`'s rule for a call's signature hash under this backend: the Ember types of its result
/// and parameters, and for a function with a C ABI the schema of what crosses by value (`[HR-21]`).
fn signature_hash(m: &Module<'_>, body: &Body) -> u64 {
    let table = m.ty.table;
    let mut parts = vec![table.display(body.return_ty())];
    parts.extend(body.args().map(|(_, decl)| table.display(decl.ty)));
    if body.abi.is_some() {
        let crossing = |ty: Ty| {
            let ty = match table.kind(ty) {
                TyKind::Ref { inner, .. } => *inner,
                _ => ty,
            };
            match table.kind(ty) {
                TyKind::Class(id) => image::fnv(&["handle", &table.class_def(*id).name.to_string()]).to_string(),
                _ => image::type_schema(table, ty, &mut Vec::new()).to_string(),
            }
        };
        parts.push(crossing(body.return_ty()));
        parts.extend((1..=body.arg_count).filter_map(|index| body.locals.get(index)).map(|local| crossing(local.ty)));
    }
    image::fnv(&parts.iter().map(String::as_str).collect::<Vec<_>>())
}

/// A class's method tables' layout, hashed: each slot's method by its name and its Ember signature,
/// then the interfaces the class's type information lists.
fn methods_hash(m: &Module<'_>, id: ClassId) -> u64 {
    let table = m.ty.table;
    let mut parts = Vec::new();
    for slot in m.dispatch.virtual_layouts.get(&id).into_iter().flatten() {
        match slot.as_ref().and_then(|method| m.bodies.get(method.symbol.as_str())) {
            Some(body) => {
                parts.push(body.name.clone());
                parts.extend(body.args().map(|(_, decl)| table.display(decl.ty)));
                parts.push(table.display(body.return_ty()));
            }
            None => parts.push("-".to_string()),
        }
    }
    for (interface, _) in m.dispatch.class_itable_entries(m, id) {
        parts.push(interface);
    }
    image::fnv(&parts.iter().map(String::as_str).collect::<Vec<_>>())
}

/// `[HR-8]` — the manifest and every table it points at. Every class's type information and
/// identity, and every reloadable function's value descriptor, are written first, so the rows that
/// name them and the identities they bring (an interface's) are known.
pub(crate) fn emit_manifest(m: &mut Module<'_>, bodies: &[Body], has_entry: bool) -> Result<()> {
    let classes: Vec<ClassId> = m.ty.table.runtime_classes().map(|(id, _)| id).collect();
    for id in classes {
        crate::glue::type_info(m, id);
    }
    let natives: Vec<(String, Ty)> = m.callables.natives.iter().map(|(symbol, ty)| (symbol.clone(), *ty)).collect();
    for (symbol, ty) in natives {
        if m.reload_plan.as_ref().is_some_and(|plan| plan.is_reloadable(&symbol)) {
            crate::closures::native_descriptor(m, &symbol, ty)?;
        }
    }
    crate::glue::emit_pending(m)?;
    let mut failed = None;
    with_plan(m, |plan, writer| {
        plan.manifest(bodies, has_entry, writer);
        failed = writer.0.reload_error.take();
    });
    if let Some(error) = failed {
        return Err(error);
    }
    crate::glue::emit_pending(m)
}

/// The LLVM backend's `image::Writer`.
pub(crate) struct LlvmWriter<'m, 'a>(pub &'m mut Module<'a>);

impl<'a> LlvmWriter<'_, 'a> {
    /// A function the writer could not write: kept for the caller (`Writer`'s methods return
    /// nothing).
    fn fail(&mut self, error: crate::Unsupported) {
        if self.0.reload_error.is_none() {
            self.0.reload_error = Some(error);
        }
    }

    fn string(&mut self, text: &str) -> String {
        self.0.bytes(text.as_bytes())
    }

    fn optional_string(&mut self, text: Option<&str>) -> String {
        match text {
            Some(text) => self.string(text),
            None => "null".to_string(),
        }
    }

    fn reference(symbol: &Option<String>) -> String {
        symbol.as_ref().map_or_else(|| "null".to_string(), |symbol| global_name(symbol))
    }

    /// A member's type and offset in a value of its holder.
    fn member(&self, holder: Holder, member: &Member) -> (Option<Ty>, u64) {
        let table = self.0.ty.table;
        match (holder, member) {
            (Holder::Object(id), Member::Field(index)) => {
                (table.class_field_at(id, *index).map(|field| field.ty), self.0.ty.object_field_offsets(id)[*index])
            }
            (Holder::Ty(ty), Member::Field(index)) => match table.kind(ty) {
                TyKind::Struct(id) => (Some(table.struct_def(*id).fields[*index].ty), table.layout(ty).field_offsets[*index]),
                _ => unreachable!("a field of a struct"),
            },
            (Holder::Ty(ty), Member::Tuple(index)) => match table.kind(ty) {
                TyKind::Tuple(items) => (Some(items[*index]), table.layout(ty).field_offsets[*index]),
                _ => unreachable!("an item of a tuple"),
            },
            (Holder::Ty(ty), Member::Payload(variant, index)) => match table.kind(ty) {
                TyKind::Enum(id) => (Some(table.enum_def(*id).variants[*variant].fields[*index].ty), table.variant_field_offsets(*id)[*variant][*index]),
                _ => unreachable!("a payload of an enum"),
            },
            (Holder::Ty(ty), Member::Tag) => match table.kind(ty) {
                TyKind::Enum(id) => (Some(table.enum_def(*id).repr), 0),
                _ => unreachable!("the tag of an enum"),
            },
            (Holder::Ty(ty), Member::Named(name)) => match table.kind(ty) {
                TyKind::Struct(id) => {
                    let def = table.struct_def(*id);
                    let index = def.fields.iter().position(|field| field.name.as_str() == name).expect("a named field of the compiler's struct");
                    (Some(def.fields[index].ty), table.layout(ty).field_offsets[index])
                }
                _ => unreachable!("a named field of a struct"),
            },
            (Holder::Object(_), _) => unreachable!("a class object holds fields"),
        }
    }

    fn size(&self, ty: Ty) -> u64 {
        match self.0.ty.table.kind(ty) {
            // A `void` member is one byte, as the C backend declares it.
            TyKind::Void | TyKind::Never | TyKind::Error => 1,
            _ => self.0.ty.size_align(ty).0,
        }
    }

    fn num(&self, num: &Num) -> u64 {
        match num {
            Num::Zero => 0,
            Num::Size(Holder::Ty(ty)) => self.size(*ty),
            Num::Size(Holder::Object(id)) => crate::glue::object_layout(self.0, *id).size,
            Num::Align(Holder::Ty(ty)) => self.0.ty.size_align(*ty).1,
            Num::Align(Holder::Object(id)) => crate::glue::object_layout(self.0, *id).align,
            // `EMBER_VEC_ALIGN_OF`: the element's alignment, at least `EMBER_VEC_ALIGN`'s 16.
            Num::VecAlign(ty) => self.0.ty.size_align(*ty).1.max(16),
            Num::MemberSize(holder, member) => self.member(*holder, member).0.map_or(1, |ty| self.size(ty)),
            Num::Offset(holder, member) => self.member(*holder, member).1,
            Num::Within(element, path) => {
                let table = self.0.ty.table;
                let (mut at, mut structure) = (0, *element);
                for &index in path {
                    at += table.layout(structure).field_offsets[index];
                    let TyKind::Struct(id) = *table.kind(structure) else { unreachable!("a column's path goes through structs") };
                    structure = table.struct_def(id).fields[index].ty;
                }
                at
            }
            Num::SizeT => 8,
        }
    }

    fn row(&mut self, row: &Row) -> String {
        let shape: &FieldShape = &row.shape;
        let name = self.string(&row.name);
        let renamed = self.optional_string(row.renamed.as_deref());
        let type_name = self.string(&shape.type_name);
        let element_type = if shape.element_type.is_empty() { "null".to_string() } else { self.string(&shape.element_type) };
        let range = shape.range.map_or_else(|| "null".to_string(), |id| global_name(&image::range_test_symbol(id)));
        format!(
            "{} {{ ptr {name}, ptr {renamed}, ptr {type_name}, i64 {}, i32 {}, i32 {}, i32 {}, i32 {}, i32 {}, i32 {}, ptr {element_type}, ptr {}, ptr {}, ptr {range}, i32 {}, i32 {} }}",
            record("rl_field"),
            row.schema.unwrap_or(0) as i64,
            self.num(&row.offset),
            self.num(&row.size),
            image::flags_value(&shape.flags),
            image::constant(shape.kind),
            image::constant(shape.element_kind),
            self.num(&shape.element_size),
            Self::reference(&shape.layout),
            Self::reference(&shape.inner),
            self.num(&shape.align),
            self.num(&shape.within)
        )
    }

    fn type_row_ir(&mut self, row: &TypeRow) -> String {
        let name = self.string(&row.name);
        let base = self.optional_string(row.base.as_deref());
        let old_type = self.optional_string(row.old_type.as_deref());
        let list = |list: &Option<(usize, Option<String>)>| match list {
            Some((count, table)) => format!("i32 {count}, ptr {}", Self::reference(table)),
            None => "i32 0, ptr null".to_string(),
        };
        format!(
            "{} {{ ptr {name}, i32 {}, ptr {base}, i32 {}, i32 {}, i32 {}, i64 {}, i64 {}, ptr {}, ptr {}, {}, {}, ptr {}, ptr {}, ptr {old_type}, ptr {}, i64 {}, ptr {} }}",
            record("rl_type"),
            image::constant(row.kind),
            image::flags_value(&row.layout),
            self.num(&row.size),
            self.num(&row.align),
            row.schema as i64,
            row.methods.unwrap_or(0) as i64,
            Self::reference(&row.type_info),
            Self::reference(&row.identity),
            list(&row.fields),
            list(&row.variants),
            Self::reference(&row.init),
            Self::reference(&row.migrate),
            Self::reference(&row.drop_field),
            row.attributes.unwrap_or(0) as i64,
            Self::reference(&row.columns)
        )
    }

    fn array(&mut self, symbol: &str, kind: &str, rows: Vec<String>) {
        self.0.global(format!(
            "{} = internal constant [{} x {}] [{}], align 8",
            global_name(symbol),
            rows.len(),
            record(kind),
            rows.join(", ")
        ));
    }

    /// A function `void symbol(void* object, uint32_t index)` doing `case` for each case's index.
    fn switch_fn<C>(&mut self, symbol: &str, cases: &[C], index_of: impl Fn(&C) -> u32, mut each: impl FnMut(&mut FnCx<'_, 'a>, &C) -> Result<()>) {
        let done = define_c_fn(self.0, symbol, "internal ", None, &[Param::Ptr, Param::Int], "", |cx, values, _| {
            let Val::Imm(_, index) = values[1].clone() else { unreachable!("an index") };
            let labels: Vec<String> = cases.iter().map(|_| cx.label()).collect();
            let end = cx.label();
            let arms: Vec<String> = cases.iter().zip(&labels).map(|(case, label)| format!("i32 {}, label %{label}", index_of(case))).collect();
            cx.emit(format!("switch i32 {index}, label %{end} [ {} ]", arms.join(" ")));
            for (case, label) in cases.iter().zip(&labels) {
                cx.start(label);
                each(cx, case)?;
                cx.emit(format!("br label %{end}"));
            }
            cx.start(&end);
            Ok(())
        });
        if let Err(error) = done {
            self.fail(error);
        }
    }

    fn place(&self, holder: Holder, member: &Member, ty: Ty) -> (u64, u64) {
        let (_, offset) = self.member(holder, member);
        let (_, align) = self.0.ty.size_align(ty);
        (offset, align.max(1).min(offset_align(offset)))
    }
}

impl Writer for LlvmWriter<'_, '_> {
    /// `[HR-14]` — `int symbol(const void* value)`: whether a value of the range's representation is
    /// in its range (`T.checked`'s test, `[RNG-3]`); NaN in no range (`[RNG-6]`).
    fn range_test(&mut self, symbol: &str, id: RangeId) {
        let table = self.0.ty.table;
        let def = table.range_def(id).clone();
        let mut cx = FnCx::glue(self.0);
        // The upper end's comparison: `le` for `..=`, `lt` for `..`.
        let below = if def.inclusive { "le" } else { "lt" };
        let truth = match (table.kind(def.repr).clone(), def.lo, def.hi) {
            (TyKind::Int(int), Bound::Int(lo), Bound::Int(hi)) => {
                let bits = match int {
                    IntTy::I8 => 8,
                    IntTy::I16 => 16,
                    IntTy::I32 => 32,
                    IntTy::I128 => 128,
                    _ => 64,
                };
                let value = cx.value(format!("load i{bits}, ptr %p0, align 1"));
                let (value, width) = if bits < 64 { (cx.value(format!("sext i{bits} {value} to i64")), 64) } else { (value, bits) };
                let low = cx.value(format!("icmp sge i{width} {value}, {}", int_literal(lo as u128, width)));
                let high = cx.value(format!("icmp s{below} i{width} {value}, {}", int_literal(hi as u128, width)));
                cx.value(format!("and i1 {low}, {high}"))
            }
            (TyKind::Uint(uint), Bound::Int(lo), Bound::Int(hi)) => {
                let bits = match uint {
                    UintTy::U8 => 8,
                    UintTy::U16 => 16,
                    UintTy::U32 => 32,
                    UintTy::U128 => 128,
                    _ => 64,
                };
                let value = cx.value(format!("load i{bits}, ptr %p0, align 1"));
                let (value, width) = if bits < 64 { (cx.value(format!("zext i{bits} {value} to i64")), 64) } else { (value, bits) };
                let high = cx.value(format!("icmp u{below} i{width} {value}, {}", int_literal(hi as u128, width)));
                // A lower end of zero is no test: an unsigned value is never below it.
                if lo <= 0 {
                    high
                } else {
                    let low = cx.value(format!("icmp uge i{width} {value}, {}", int_literal(lo as u128, width)));
                    cx.value(format!("and i1 {low}, {high}"))
                }
            }
            (TyKind::Float(float @ (FloatTy::F32 | FloatTy::F64)), Bound::Float(lo), Bound::Float(hi)) => {
                let value = if float == FloatTy::F32 {
                    let narrow = cx.value("load float, ptr %p0, align 1");
                    cx.value(format!("fpext float {narrow} to double"))
                } else {
                    cx.value("load double, ptr %p0, align 1")
                };
                let low = cx.value(format!("fcmp oge double {value}, {}", float_literal(lo, Sc::F64)));
                let high = cx.value(format!("fcmp o{below} double {value}, {}", float_literal(hi, Sc::F64)));
                cx.value(format!("and i1 {low}, {high}"))
            }
            _ => unreachable!("only a testable range has a test (`image::range_testable`)"),
        };
        let result = cx.value(format!("zext i1 {truth} to i32"));
        let attrs = crate::func::fn_attrs(cx.m);
        let text = format!(
            "define internal i32 {}(ptr %p0) {attrs} {{\nentry:\n{}  br label %start\nstart:\n{}  ret i32 {result}\n}}\n\n",
            global_name(symbol),
            cx.allocas_text(),
            cx.out_text()
        );
        cx.m.fns.push_str(&text);
    }

    fn field(&mut self, symbol: &str, row: &Row) {
        let row = self.row(row);
        self.0.global(format!("{} = internal constant {row}, align 8", global_name(symbol)));
    }

    fn fields(&mut self, symbol: &str, rows: &[Row]) {
        let rows: Vec<String> = rows.iter().map(|row| self.row(row)).collect();
        self.array(symbol, "rl_field", rows);
    }

    fn tag(&mut self, symbol: &str, row: &Row) {
        let row = self.row(row);
        self.array(symbol, "rl_field", vec![row]);
    }

    fn type_row(&mut self, symbol: &str, row: &TypeRow) {
        let row = self.type_row_ir(row);
        self.0.global(format!("{} = internal constant {row}, align 8", global_name(symbol)));
    }

    fn variants(&mut self, symbol: &str, rows: &[VariantRow]) {
        let rows: Vec<String> = rows
            .iter()
            .map(|row| {
                let name = self.string(&row.name);
                let renamed = self.optional_string(row.renamed.as_deref());
                format!(
                    "{} {{ ptr {name}, ptr {renamed}, i64 {}, i64 {}, i32 {}, ptr {} }}",
                    record("rl_variant"),
                    row.discriminant as i64,
                    row.payload.unwrap_or(0) as i64,
                    row.fields.0,
                    Self::reference(&row.fields.1)
                )
            })
            .collect();
        self.array(symbol, "rl_variant", rows);
    }

    fn init_fn(&mut self, symbol: &str, cases: &[InitCase]) {
        let places: Vec<(u64, u64)> = cases.iter().map(|case| self.place(case.holder, &case.member, case.ty)).collect();
        let indexed: Vec<(usize, &InitCase)> = cases.iter().enumerate().collect();
        self.switch_fn(symbol, &indexed, |(_, case)| case.index, |cx, (at, case)| {
            let (offset, align) = places[*at];
            let to = cx.gep("%p0", offset);
            let has_bytes = !cx.m.ty.is_zst(case.ty);
            cx.call_body(&case.function, Vec::new(), has_bytes.then_some((to.as_str(), align)))
        });
    }

    fn drop_fn(&mut self, symbol: &str, cases: &[DropCase]) {
        let places: Vec<(u64, u64)> = cases.iter().map(|case| self.place(case.holder, &case.member, case.ty)).collect();
        let indexed: Vec<(usize, &DropCase)> = cases.iter().enumerate().collect();
        self.switch_fn(symbol, &indexed, |(_, case)| case.index, |cx, (at, case)| {
            let (offset, align) = places[*at];
            let to = cx.gep("%p0", offset);
            cx.drop_at(&to, align, case.ty)
        });
    }

    /// `[HR-16]` — `int symbol(void* out, const void* old, char* report, size_t report_size)`: the
    /// driver's wrapper builds the new value from `Old<Name>`, or writes its `ReloadError` into the
    /// report; 0 is success.
    fn migrate_fn(&mut self, symbol: &str, wrapper: &str, ret: Ty, option: EnumId, some: usize) {
        let table = self.0.ty.table;
        let def = table.enum_def(option).clone();
        let payload_ty = def.variants[some].fields[0].ty;
        let payload_at = if table.option_niche(option).is_some() { 0 } else { table.variant_field_offsets(option)[some][0] };
        let (payload_size, payload_align) = self.0.ty.size_align(payload_ty);
        let (size, align) = self.0.ty.size_align(ret);
        let mut cx = FnCx::glue(self.0);
        let text = cx.alloca(24, 8);
        cx.zero(&text, 8, 24);
        let result = cx.alloca(size, align);
        if let Err(error) = cx.call_body(wrapper, vec![Val::Imm(Sc::Ptr, "%p1".to_string()), Val::Imm(Sc::Ptr, text.clone())], Some((result.as_str(), align))) {
            drop(cx);
            self.fail(error);
            return;
        }
        let place = crate::func::PlaceRef { ptr: result.clone(), ty: ret, variant: None, align };
        let (tag, bits) = match cx.discriminant_at(&place) {
            Ok(found) => found,
            Err(error) => {
                drop(cx);
                self.fail(error);
                return;
            }
        };
        let done = cx.value(format!("icmp eq i{bits} {tag}, {}", int_literal(def.variants[some].discriminant as u128, bits)));
        let (ok, fail, write, end) = (cx.label(), cx.label(), cx.label(), cx.label());
        cx.emit(format!("br i1 {done}, label %{ok}, label %{fail}"));
        cx.start(&ok);
        let from = cx.gep(&result, payload_at);
        cx.copy("%p0", payload_align.max(1), &from, align.min(offset_align(payload_at)).max(1), payload_size, false);
        cx.emit(format!("br label %{end}"));
        cx.start(&fail);
        let room = cx.value("icmp ugt i64 %p3, 0");
        cx.emit(format!("br i1 {room}, label %{write}, label %{end}"));
        cx.start(&write);
        let len_at = cx.gep(&text, 8);
        let len = cx.load(Sc::I(64), &len_at, 8);
        let limit = cx.value("sub i64 %p3, 1");
        let shorter = cx.value(format!("icmp ult i64 {len}, {limit}"));
        let count = cx.value(format!("select i1 {shorter}, i64 {len}, i64 {limit}"));
        let bytes = cx.load(Sc::Ptr, &text, 8);
        cx.m.declare("llvm.memcpy.p0.p0.i64", || "declare void @llvm.memcpy.p0.p0.i64(ptr, ptr, i64, i1)".to_string());
        let (copy, after) = (cx.label(), cx.label());
        let any = cx.value(format!("icmp ugt i64 {count}, 0"));
        cx.emit(format!("br i1 {any}, label %{copy}, label %{after}"));
        cx.start(&copy);
        cx.emit(format!("call void @llvm.memcpy.p0.p0.i64(ptr align 1 %p2, ptr align 1 {bytes}, i64 {count}, i1 false)"));
        cx.emit(format!("br label %{after}"));
        cx.start(&after);
        let terminator = cx.value(format!("getelementptr i8, ptr %p2, i64 {count}"));
        cx.emit(format!("store i8 0, ptr {terminator}, align 1"));
        cx.emit(format!("br label %{end}"));
        cx.start(&end);
        cx.call_rt("vec_free", vec![Val::Imm(Sc::Ptr, text), Val::Imm(Sc::I(64), "1".to_string())], None);
        let status = cx.value(format!("select i1 {done}, i32 0, i32 1"));
        let attrs = crate::func::fn_attrs(cx.m);
        let body = format!(
            "define internal i32 {}(ptr %p0, ptr %p1, ptr %p2, i64 %p3) {attrs} {{\nentry:\n{}  br label %start\nstart:\n{}  ret i32 {status}\n}}\n\n",
            global_name(symbol),
            cx.allocas_text(),
            cx.out_text()
        );
        cx.m.fns.push_str(&body);
    }

    fn frame_info(&mut self, info: &FrameInfo) {
        let symbol = info.symbol.clone();
        let mut sites = Vec::new();
        for (index, (line, types)) in info.sites.iter().enumerate() {
            let list = if types.is_empty() {
                "null".to_string()
            } else {
                let names: Vec<String> = types.iter().map(|name| format!("ptr {}", self.string(name))).collect();
                let list = format!("{symbol}_t{index}");
                self.0.global(format!("{} = internal constant [{} x ptr] [{}], align 8", global_name(&list), names.len(), names.join(", ")));
                global_name(&list)
            };
            sites.push(format!("{} {{ i32 {line}, i32 {}, ptr {list} }}", record("rl_site"), types.len()));
        }
        let sites_at = if sites.is_empty() {
            "null".to_string()
        } else {
            let at = format!("{symbol}_sites");
            self.array(&at, "rl_site", sites.clone());
            global_name(&at)
        };
        let name = self.string(&info.function);
        self.0.global(format!(
            "{} = internal constant {} {{ ptr {name}, i32 {}, ptr {sites_at} }}, align 8",
            global_name(&symbol),
            record("rl_frame_info"),
            sites.len()
        ));
    }

    fn manifest(&mut self, manifest: &Manifest) {
        let mut fns = Vec::new();
        for row in &manifest.fns {
            fns.push(match row {
                FnRow::Reloadable { symbol, descriptor, signature, body_hash, flags } => {
                    let name = self.string(symbol);
                    format!(
                        "{} {{ ptr {name}, ptr {}, ptr {}, ptr {}, i64 {}, i64 {}, i32 {} }}",
                        record("rl_fn"),
                        global_name(&image::body_symbol(symbol)),
                        global_name(symbol),
                        Self::reference(descriptor),
                        *signature as i64,
                        *body_hash as i64,
                        image::flags_value(flags)
                    )
                }
                FnRow::NoReload { symbol, body_hash } => {
                    let name = self.string(symbol);
                    format!(
                        "{} {{ ptr {name}, ptr null, ptr null, ptr null, i64 0, i64 {}, i32 {} }}",
                        record("rl_fn"),
                        *body_hash as i64,
                        image::constant("EMBER_RL_FN_NORELOAD")
                    )
                }
            });
        }
        let mut statics = Vec::new();
        for row in &manifest.statics {
            statics.push(match row {
                StaticRow::Stored { name, ty, schema, initialiser, flags, constant, shape } => {
                    let (size, align) = self.0.ty.size_align(*ty);
                    let initial = if *constant {
                        match crate::statics::constant_static(self.0, name, *ty) {
                            Ok(at) => at,
                            Err(error) => {
                                self.fail(error);
                                "null".to_string()
                            }
                        }
                    } else {
                        "null".to_string()
                    };
                    let shown = self.string(name);
                    format!(
                        "{} {{ ptr {shown}, i32 {size}, i32 {align}, i64 {}, i64 {}, i32 {}, ptr {initial}, ptr null, ptr {} }}",
                        record("rl_static"),
                        *schema as i64,
                        *initialiser as i64,
                        image::flags_value(flags),
                        global_name(shape)
                    )
                }
                StaticRow::Folded { name, declared, initialiser, reinit } => {
                    let shown = self.string(name);
                    format!(
                        "{} {{ ptr {shown}, i32 0, i32 1, i64 {}, i64 {}, i32 {}, ptr null, ptr null, ptr null }}",
                        record("rl_static"),
                        *declared as i64,
                        *initialiser as i64,
                        if *reinit { image::constant("EMBER_RL_STATIC_REINIT") } else { 0 }
                    )
                }
            });
        }
        let types: Vec<String> = manifest.types.iter().map(|row| self.type_row_ir(row)).collect();
        let identities: Vec<String> = manifest.identities.iter().map(|identity| format!("ptr {}", global_name(identity))).collect();
        let mut headers = Vec::new();
        for (name, hash) in &manifest.headers {
            let (name, hash) = (self.string(name), self.string(hash));
            headers.push(format!("{} {{ ptr {name}, ptr {hash} }}", record("rl_header")));
        }
        let counts = (fns.len(), statics.len(), types.len(), identities.len(), headers.len());
        let list = |this: &mut Self, name: &str, kind: &str, rows: Vec<String>| -> String {
            let symbol = image::table(name);
            if rows.is_empty() {
                this.0.global(format!("{} = internal constant [1 x {}] zeroinitializer, align 8", global_name(&symbol), record(kind)));
            } else {
                this.array(&symbol, kind, rows);
            }
            global_name(&symbol)
        };
        let fns_at = list(self, "fns", "rl_fn", fns);
        let statics_at = list(self, "statics", "rl_static", statics);
        let types_at = list(self, "types", "rl_type", types);
        let identities_symbol = image::table("identities");
        let identities_at = global_name(&identities_symbol);
        if identities.is_empty() {
            self.0.global(format!("{identities_at} = internal constant [1 x ptr] zeroinitializer, align 8"));
        } else {
            self.0.global(format!("{identities_at} = internal constant [{} x ptr] [{}], align 8", identities.len(), identities.join(", ")));
        }
        let headers_at = list(self, "headers", "rl_header", headers);
        let layout = ember_branding::runtime(if manifest.instances { "obj_header_v40" } else { "obj_header_v24" });
        let witness = ember_branding::runtime("reload_protocol_v1");
        for symbol in [&layout, &witness] {
            let name = global_name(symbol);
            self.0.declare(symbol, || format!("declare void {name}()"));
        }
        let entry = if manifest.has_entry { global_name(&image::entry_symbol()) } else { "null".to_string() };
        let package = self.string(&manifest.package);
        let (size, _) = record_layout(fields_of("rl_manifest"));
        let table = |name: &str| global_name(&image::table(name));
        let manifest_symbol = image::manifest_symbol();
        let section = match self.0.ty.target {
            crate::abi::Target::Win64 => ".emrl".to_string(),
            crate::abi::Target::SysV | crate::abi::Target::Aapcs64 => ember_branding::runtime("reload"),
        };
        let (fns_n, statics_n, types_n, identities_n, headers_n) = counts;
        self.0.global(format!(
            "; [HR-8] the reload manifest\n{} = {}constant {} {{ i64 {}, i32 {PROTOCOL}, i32 {size}, i32 {}, i32 {}, ptr {package}, ptr {}, ptr {}, \
             i32 {fns_n}, ptr {fns_at}, ptr {}, ptr {}, ptr {}, i32 {statics_n}, ptr {statics_at}, ptr {}, ptr {}, i32 {types_n}, ptr {types_at}, \
             i32 {identities_n}, ptr {identities_at}, ptr {entry}, i32 {headers_n}, ptr {headers_at} }}, section \"{section}\", align 8",
            global_name(&manifest_symbol),
            visibility(self.0),
            record("rl_manifest"),
            MAGIC as i64,
            self.0.ty.header,
            manifest.mode,
            global_name(&layout),
            global_name(&witness),
            table("targets"),
            table("addresses"),
            table("descriptors"),
            table("static_data"),
            table("static_state"),
        ));
        // The image's link keeps it, however it trims (`[HR-8]`).
        self.0.global(format!("@llvm.used = appending global [1 x ptr] [ptr {}], section \"llvm.metadata\"", global_name(&manifest_symbol)));
    }

    fn soa_columns(&mut self, soa: StructId) -> String {
        let ty = self.0.ty.table.find(&TyKind::Struct(soa)).expect("an `SoA`'s struct is interned");
        match crate::soa::layout_fn(self.0, ty) {
            Ok(symbol) => symbol,
            Err(error) => {
                self.fail(error);
                String::new()
            }
        }
    }

    fn signature_hash(&self, body: &Body) -> u64 {
        signature_hash(self.0, body)
    }

    fn methods_hash(&self, class: ClassId) -> u64 {
        methods_hash(self.0, class)
    }

    fn descriptor(&self, symbol: &str) -> Option<String> {
        self.0.callables.natives.contains_key(symbol).then(|| crate::closures::native_descriptor_symbol(symbol))
    }

    fn interface_identities(&self) -> Vec<String> {
        self.0.reload_ids.iter().cloned().collect()
    }

    fn closure_envs(&self) -> Vec<Ty> {
        self.0.callables.closures.values().copied().collect()
    }

    fn dyn_concretes(&self) -> Vec<Ty> {
        self.0.dispatch.adapters.values().map(|adapter| adapter.concrete).collect()
    }
}

/// `[HR-15]` — what a closure's descriptor says its captured state holds: the table's address, or
/// `null` when it holds no object.
pub(crate) fn captures(m: &mut Module<'_>, env: Ty) -> String {
    with_plan(m, |plan, writer| plan.captures(env, writer)).map_or_else(|| "null".to_string(), |symbol| global_name(&symbol))
}

/// Whether a function the image has a body for is reached through its thunk.
pub(crate) fn is_reloadable(m: &Module<'_>, symbol: &str) -> bool {
    m.reload.as_ref().is_some_and(|reload| reload.reloadable.contains(symbol))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The header's typedef of `ember_<name>`: its members' C types, in order.
    fn header_fields(header: &str, name: &str) -> Vec<String> {
        let start = format!("typedef struct ember_{name} {{");
        let at = header.find(&start).unwrap_or_else(|| panic!("the header declares `ember_{name}`"));
        let body = &header[at + start.len()..];
        let end = body.find(&format!("}} ember_{name};")).expect("the typedef ends");
        let mut out = Vec::new();
        for line in body[..end].lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("/*") || line.starts_with('*') || line.starts_with("#") {
                continue;
            }
            let declaration = line.split("/*").next().unwrap_or(line).trim();
            if !declaration.ends_with(';') {
                continue;
            }
            let ty = if declaration.contains("(*") || declaration.contains('*') {
                "ptr"
            } else if declaration.starts_with("uint32_t") || declaration.starts_with("int ") {
                "i32"
            } else if declaration.starts_with("uint64_t") || declaration.starts_with("size_t") {
                "i64"
            } else {
                panic!("a member of `ember_{name}` this test does not know: {declaration}")
            };
            out.push(ty.to_string());
        }
        out
    }

    /// `[HR-8]` — each record the IR writes is the header's, member for member, and the manifest's
    /// magic and protocol are the header's.
    #[test]
    fn hr_8_reload_records_are_the_headers() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../runtime")
            .join(format!("{}_rt", ember_branding::SYMBOL_PREFIX))
            .join("include")
            .join(ember_branding::runtime_header());
        let header = std::fs::read_to_string(&root).expect("the runtime header is readable");
        for (name, fields) in RECORDS {
            assert_eq!(header_fields(&header, name), fields.iter().map(|field| field.to_string()).collect::<Vec<_>>(), "ember_{name}");
        }
        assert!(header.contains(&format!("#define EMBER_RELOAD_PROTOCOL {PROTOCOL}\n")), "the protocol");
        assert!(header.contains(&format!("#define EMBER_RELOAD_MAGIC UINT64_C(0x{MAGIC:016X})")), "the magic");
        // Every constant the planner's rows name, with the header's value.
        for line in header.lines() {
            let Some(rest) = line.strip_prefix("#define EMBER_RL_") else { continue };
            let mut words = rest.split_whitespace();
            let (Some(name), Some(value)) = (words.next(), words.next()) else { continue };
            let Some(value) = value.strip_prefix("UINT32_C(").and_then(|value| value.strip_suffix(')')) else { continue };
            assert_eq!(image::constant(&format!("EMBER_RL_{name}")), value.parse::<u32>().expect("a number"), "EMBER_RL_{name}");
        }
    }

    #[test]
    fn record_layouts_are_cs() {
        assert_eq!(record_layout(fields_of("rl_field")), (96, 8));
        assert_eq!(record_layout(fields_of("rl_frame")), (24, 8));
        assert_eq!(record_layout(fields_of("rl_manifest")).0 % 8, 0);
    }
}
