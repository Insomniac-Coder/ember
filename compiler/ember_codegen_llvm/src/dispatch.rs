//! Dynamic dispatch: class method tables (`[DSP-2]`), interface tables and their adapters for
//! `dyn` values, boxes and class-interface handles (`[TYP-22]`, `[OBJ-2]`, `[DSP-3]`), and the
//! calls through them. Each table is the C backend's, slot for slot: a table entry is called
//! through a C function pointer, so every adapter has the C signature of its slot.

use std::collections::{BTreeMap, BTreeSet};

use crate::func::{FnCx, Param, PlaceRef, Val, define_c_fn};
use crate::types::Sc;
use crate::{Module, Result, global_name, unsupported};
use ember_hir::InterfaceSlot;
use ember_mir::{Body, CastKind, FuncRef, InterfaceAdapterMethod, Operand, ParameterMode, Rvalue, StmtKind, Terminator};
use ember_span::Symbol;
use ember_types::{ClassId, ClassOpenness, Ty, TyKind};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct VirtualMethod {
    pub owner: ClassId,
    pub symbol: String,
    pub is_abstract: bool,
}

/// One interface table's slot: the erased signature's parameters (after the receiver) and result.
type SlotSignature = Option<(Vec<Ty>, Ty)>;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Adapter {
    pub concrete: Ty,
    pub identity: String,
    pub layout: Vec<SlotSignature>,
    pub implementations: Vec<Option<(String, ParameterMode)>>,
}

/// What the whole program's MIR says about dispatch, gathered before any body is written.
#[derive(Clone, Default)]
pub(crate) struct Dispatch {
    /// Each class's slots, its bases' first.
    pub virtual_layouts: BTreeMap<ClassId, Vec<Option<VirtualMethod>>>,
    /// Each (class, slot)'s declaring method: the signature a call through the slot uses.
    pub slot_declarations: BTreeMap<(ClassId, usize), String>,
    /// Each adapter, by its interfaces' identity and its concrete type's mangled name.
    pub adapters: BTreeMap<(String, String), Adapter>,
    /// The interface tables a class's type information lists, by identity.
    pub class_tables: BTreeMap<ClassId, BTreeMap<String, Ty>>,
    /// The identities whose tables copy their value (`[TYP-22]`, ODR-237).
    pub clones: BTreeSet<String>,
    /// IX.1 (ODR-241) — the identities whose tables compare two values (an interface requires
    /// `Eq`): an `eq` entry follows `access` (and `clone`).
    pub eqs: BTreeSet<String>,
    /// The tables and markers already written.
    emitted: BTreeSet<String>,
}

/// `[MNG-4]` — the interfaces' identity, as the C backend spells it: `_7Observe_6Reader`.
pub(crate) fn identity(m: &Module<'_>, interfaces: &[Symbol]) -> String {
    let mut out = String::new();
    for interface in interfaces {
        out.push('_');
        out.push_str(&m.ty.table.mangled_interface(*interface));
    }
    out
}

fn concrete_name(m: &Module<'_>, concrete: Ty) -> String {
    let table = m.ty.table;
    match table.kind(concrete) {
        TyKind::Class(id) => table.mangled_class(*id),
        TyKind::Struct(id) => table.mangled_struct(*id),
        TyKind::Enum(id) => table.mangled_enum(*id),
        _ => table.mangled(concrete),
    }
}

fn adapter_of(
    concrete: Ty,
    identity: String,
    layout: &[Option<InterfaceSlot>],
    implementations: &[Option<InterfaceAdapterMethod>],
) -> Adapter {
    Adapter {
        concrete,
        identity,
        layout: layout.iter().map(|slot| slot.as_ref().map(|slot| (slot.params.clone(), slot.ret))).collect(),
        implementations: implementations
            .iter()
            .map(|method| method.as_ref().map(|method| (method.symbol.clone(), method.receiver)))
            .collect(),
    }
}

impl Dispatch {
    pub(crate) fn collect(m: &Module<'_>, bodies: &[Body]) -> Dispatch {
        let mut dispatch = Dispatch::default();
        let declared: BTreeMap<(ClassId, usize), VirtualMethod> = bodies
            .iter()
            .filter_map(|body| {
                let (owner, slot) = (body.class_owner?, body.class_virtual_slot?);
                Some(((owner, slot), VirtualMethod { owner, symbol: body.symbol.clone(), is_abstract: body.is_abstract }))
            })
            .collect();
        let classes: Vec<ClassId> = m.ty.table.runtime_classes().map(|(id, _)| id).collect();
        for id in classes {
            dispatch.layout(m, id, &declared);
        }
        let add = |dispatch: &mut Dispatch, adapter: Adapter| {
            let key = (adapter.identity.clone(), concrete_name(m, adapter.concrete));
            dispatch.adapters.entry(key).or_insert(adapter);
        };
        for body in bodies {
            for block in &body.blocks {
                for stmt in &block.stmts {
                    let StmtKind::Assign { rvalue: Rvalue::Cast { kind, .. }, .. } = &stmt.kind else { continue };
                    match kind {
                        CastKind::InterfaceUpcast { concrete, interfaces, layout, implementations } => {
                            let id = identity(m, interfaces);
                            dispatch.note_entries(m, interfaces, &id);
                            add(&mut dispatch, adapter_of(*concrete, id, layout, implementations));
                        }
                        CastKind::ClassInterfaceUpcast { concrete, interface, layout, implementations } => {
                            let id = identity(m, &[*interface]);
                            dispatch.note_entries(m, &[*interface], &id);
                            add(&mut dispatch, adapter_of(*concrete, id.clone(), layout, implementations));
                            if let TyKind::Class(class) = m.ty.kind(*concrete) {
                                dispatch.class_tables.entry(*class).or_default().insert(id, *concrete);
                            }
                        }
                        _ => {}
                    }
                }
                if let Terminator::Call { func: FuncRef::DynBoxNew { concrete, interfaces, layout, implementations, .. }, .. } = &block.terminator {
                    let id = identity(m, interfaces);
                    dispatch.note_entries(m, interfaces, &id);
                    add(&mut dispatch, adapter_of(*concrete, id, layout, implementations));
                }
                if let Terminator::Call { func: FuncRef::Interface { interfaces, .. }, .. } = &block.terminator {
                    dispatch.note_entries(m, interfaces, &identity(m, interfaces));
                }
            }
        }
        dispatch
    }

    /// `[TYP-22]` (ODR-237), IX.1 (ODR-241) — a table for `interfaces` copies its value when one
    /// of them requires `Clone`, and compares two when one requires `Eq`.
    fn note_entries(&mut self, m: &Module<'_>, interfaces: &[Symbol], id: &str) {
        if m.ty.table.dyn_clones(interfaces) {
            self.clones.insert(id.to_string());
        }
        if m.ty.table.dyn_eqs(interfaces) {
            self.eqs.insert(id.to_string());
        }
    }

    fn layout(&mut self, m: &Module<'_>, id: ClassId, declared: &BTreeMap<(ClassId, usize), VirtualMethod>) -> Vec<Option<VirtualMethod>> {
        if let Some(layout) = self.virtual_layouts.get(&id) {
            return layout.clone();
        }
        let base = m.ty.table.class_def(id).base;
        let mut layout = base.map(|base| self.layout(m, base, declared)).unwrap_or_default();
        if let Some(base) = base {
            let inherited: Vec<(usize, String)> = self
                .slot_declarations
                .iter()
                .filter(|((class, _), _)| *class == base)
                .map(|((_, slot), symbol)| (*slot, symbol.clone()))
                .collect();
            for (slot, symbol) in inherited {
                self.slot_declarations.insert((id, slot), symbol);
            }
        }
        for ((owner, slot), method) in declared {
            if *owner != id {
                continue;
            }
            if layout.len() <= *slot {
                layout.resize(*slot + 1, None);
            }
            layout[*slot] = Some(method.clone());
            // The first declaration in the chain is the slot's signature.
            self.slot_declarations.entry((id, *slot)).or_insert_with(|| method.symbol.clone());
        }
        self.virtual_layouts.insert(id, layout.clone());
        layout
    }

    /// `[DSP-1]`, `[DSP-5]` — the one body a virtual call on a `receiver` can run: the whole
    /// program is here, so the receiver's class and those below it are every class its object
    /// can be.
    fn only_implementation(&self, m: &Module<'_>, receiver: ClassId, slot: usize) -> Option<String> {
        // `[HR-9a]` — a reloadable image is never devirtualised: a reload may add an override.
        if m.reload.is_some() {
            return None;
        }
        let mut only: Option<&VirtualMethod> = None;
        for (class, slots) in &self.virtual_layouts {
            if !m.ty.table.class_is_subclass_of(*class, receiver) {
                continue;
            }
            let method = slots.get(slot)?.as_ref()?;
            if method.is_abstract {
                if m.ty.table.class_def(*class).openness == ClassOpenness::Abstract {
                    continue;
                }
                return None;
            }
            if only.is_some_and(|seen| seen.symbol != method.symbol) {
                return None;
            }
            only = Some(method);
        }
        only.map(|method| method.symbol.clone())
    }

    /// `[DSP-3]` — a class's interface entries: its own, then each base's it does not replace.
    pub(crate) fn class_itable_entries(&self, m: &Module<'_>, id: ClassId) -> Vec<(String, Ty)> {
        let mut entries: Vec<(String, Ty)> = Vec::new();
        let mut current = Some(id);
        while let Some(class) = current {
            for (interface, concrete) in self.class_tables.get(&class).into_iter().flatten() {
                if !entries.iter().any(|(seen, _)| seen == interface) {
                    entries.push((interface.clone(), *concrete));
                }
            }
            current = m.ty.table.class_def(class).base;
        }
        entries
    }
}

impl Module<'_> {
    /// `[DSP-5]` — a virtual call on a class that is not final, made directly to the one body it
    /// can run: `--emit-optimization-report`'s line for it, as the C backend writes it.
    pub(crate) fn note_direct_call(&mut self, receiver: ClassId, slot: usize, span: ember_span::Span) {
        let class = self.ty.table.class_def(receiver);
        if class.openness == ClassOpenness::Final {
            return;
        }
        let Some(method) = self.dispatch.only_implementation(self, receiver, slot).and_then(|symbol| self.bodies.get(symbol.as_str()).copied()) else {
            return;
        };
        let Some(owner) = method.class_owner else { return };
        let line = format!(
            "{}: calls `{}.{}` directly, the only implementation for class `{}` ([DSP-5])",
            self.map.location(span),
            self.ty.table.class_def(owner).name,
            method.name,
            class.name
        );
        self.direct_calls.insert((span.file, span.start, line));
    }
}

/// `[DSP-2]` — the class's method table, slot for slot (an abstract slot `null`), or `null` when
/// it has no method to call.
pub(crate) fn class_vtable(m: &mut Module<'_>, id: ClassId) -> Result<String> {
    let slots = m.dispatch.virtual_layouts.get(&id).cloned().unwrap_or_default();
    if !slots.iter().flatten().any(|method| !method.is_abstract) {
        return Ok("null".to_string());
    }
    let symbol = ember_branding::vtable(&m.ty.table.mangled_class(id));
    if m.dispatch.emitted.insert(symbol.clone()) {
        let mut entries = Vec::new();
        for method in &slots {
            match method {
                Some(method) if !method.is_abstract => {
                    crate::func::declare_body_symbol(m, &method.symbol)?;
                    entries.push(format!("ptr {}", global_name(&method.symbol)));
                }
                _ => entries.push("ptr null".to_string()),
            }
        }
        m.global(format!("{} = internal constant [{} x ptr] [{}], align 8", global_name(&symbol), entries.len(), entries.join(", ")));
    }
    Ok(global_name(&symbol))
}

/// `[DSP-3]` — the class's interface entries (`{interface id, table}`) and how many.
pub(crate) fn class_itables(m: &mut Module<'_>, id: ClassId) -> Result<(String, usize)> {
    let entries = m.dispatch.class_itable_entries(m, id);
    if entries.is_empty() {
        return Ok(("null".to_string(), 0));
    }
    let mut rows = Vec::new();
    for (identity, concrete) in &entries {
        let marker = interface_id(m, identity);
        let table = adapter_table(m, *concrete, identity)?;
        rows.push(format!("{{ ptr, ptr }} {{ ptr {marker}, ptr {table} }}"));
    }
    let symbol = format!("{}_{}", ember_branding::generated("itables"), m.ty.table.mangled_class(id));
    m.global(format!("{} = internal constant [{} x {{ ptr, ptr }}] [{}], align 8", global_name(&symbol), rows.len(), rows.join(", ")));
    Ok((global_name(&symbol), entries.len()))
}

/// `[DSP-3]` — an interface's identity in the runtime's tables: the address of a byte of its own;
/// in a reloadable image, a record the runtime makes one across images (`ember_rl_identity`).
pub(crate) fn interface_id(m: &mut Module<'_>, identity: &str) -> String {
    let symbol = format!("{}{identity}", ember_branding::generated("interface_id"));
    if m.dispatch.emitted.insert(symbol.clone()) {
        if m.reload.is_some() {
            let name = m.bytes(symbol.as_bytes());
            m.global(format!("{} = internal global %ember.rl_identity {{ ptr {name}, ptr null }}, align 8", global_name(&symbol)));
            m.reload_ids.insert(symbol.clone());
        } else {
            m.global(format!("{} = internal constant i8 0, align 1", global_name(&symbol)));
        }
    }
    global_name(&symbol)
}

/// The offset of a table's slot: after `drop`, `size`, `align`, `access` and, in a table that
/// copies its value, `clone`, and in one that compares two, `eq`.
fn slot_offset(m: &Module<'_>, identity: &str, slot: usize) -> u64 {
    let header = 4 + u64::from(m.dispatch.clones.contains(identity)) + u64::from(m.dispatch.eqs.contains(identity));
    8 * (header + slot as u64)
}

/// The table adapting `concrete` to the interfaces `identity` names (`em_vt_dyn_7Observe_for_6Reader`):
/// `{drop, size, align, access, [clone,] [eq,] slot…}`, each slot an adapter of the slot's C signature.
pub(crate) fn adapter_table(m: &mut Module<'_>, concrete: Ty, identity: &str) -> Result<String> {
    let name = concrete_name(m, concrete);
    let symbol = ember_branding::vtable(&format!("dyn{identity}_for_{name}"));
    if !m.dispatch.emitted.insert(symbol.clone()) {
        return Ok(global_name(&symbol));
    }
    let Some(adapter) = m.dispatch.adapters.get(&(identity.to_string(), name)).cloned() else {
        return unsupported("an interface table no cast made");
    };
    let table = m.ty.table;
    let class = match table.kind(concrete) {
        TyKind::Class(class) => Some(*class),
        _ => None,
    };
    let value_payload = table.is_primitive_scalar(concrete) || matches!(table.kind(concrete), TyKind::Struct(_) | TyKind::Enum(_));
    // The value's own end: a payload dropped in place (the box frees its block), a class
    // handle released.
    let (drop, size, align) = if value_payload {
        let drop = format!("{symbol}_drop");
        crate::glue::define_glue(m, &drop, &["ptr"], "", |cx| {
            let (_, align) = cx.m.ty.size_align(concrete);
            cx.drop_at("%p0", align, concrete)
        })?;
        let (size, align) = m.ty.size_align(concrete);
        (global_name(&drop), size.max(1), align)
    } else if class.is_some() {
        let drop = format!("{symbol}_drop");
        crate::glue::define_glue(m, &drop, &["ptr"], "", |cx| {
            let helper = crate::glue::count_helper(cx.m, "release", "any");
            cx.emit(format!("call void {helper}(ptr %p0)"));
            Ok(())
        })?;
        (global_name(&drop), 0, 0)
    } else {
        ("null".to_string(), 0, 0)
    };
    // `[EXC-18]` — a class concrete's whole-object access, for a view a `dyn` call returns.
    let access = match class {
        Some(_) => {
            let access = format!("{symbol}_access");
            define_c_fn(m, &access, "internal ", None, &[Param::Ptr, Param::Int, Param::Loc], "", |cx, values, _| {
                let Val::Imm(_, operation) = &values[1] else { unreachable!("an int") };
                let operation = operation.clone();
                let object = Val::Imm(Sc::Ptr, "%p0".to_string());
                let loc = values[2].clone();
                let labels: Vec<String> = (0..4).map(|_| cx.label()).collect();
                let done = cx.label();
                cx.emit(format!(
                    "switch i32 {operation}, label %{} [ i32 0, label %{} i32 1, label %{} i32 2, label %{} ]",
                    labels[3], labels[0], labels[1], labels[2]
                ));
                for (label, entry) in labels.iter().zip(["object_begin_read", "object_end_read", "object_begin_write", "object_end_write"]) {
                    cx.start(label);
                    cx.call_rt(entry, vec![object.clone(), loc.clone()], None);
                    cx.emit(format!("br label %{done}"));
                }
                cx.start(&done);
                Ok(())
            })?;
            global_name(&access)
        }
        None => "null".to_string(),
    };
    // `[TYP-22]` (ODR-237) — the copy entry of a table that copies its value: a clone of the value
    // in a new block (a class handle's copy is the handle, retained).
    let clone = if m.dispatch.clones.contains(identity) {
        let function = format!("{symbol}_clone");
        let mut cx = FnCx::glue(m);
        let copy = if class.is_some() {
            let helper = crate::glue::count_helper(cx.m, "retain", "any");
            cx.emit(format!("call void {helper}(ptr %p0)"));
            "%p0".to_string()
        } else {
            let (size, align) = cx.m.ty.size_align(concrete);
            let slot = cx.alloca(8, 8);
            cx.call_rt(
                "alloc",
                vec![Val::Imm(Sc::I(64), size.max(1).to_string()), Val::Imm(Sc::I(64), align.max(1).to_string())],
                Some((&slot, 8)),
            );
            let block = cx.load(Sc::Ptr, &slot, 8);
            cx.clone_part(&block, "%p0", align.max(1), concrete)?;
            block
        };
        let attrs = crate::func::fn_attrs(cx.m);
        let text = format!(
            "define internal ptr {}(ptr %p0) {attrs} {{\nentry:\n{}  br label %start\nstart:\n{}  ret ptr {copy}\n}}\n\n",
            global_name(&function),
            cx.allocas_text(),
            cx.out_text()
        );
        cx.m.fns.push_str(&text);
        Some(global_name(&function))
    } else {
        None
    };
    // IX.1 (ODR-241) — the `eq` entry of a table that compares two values: both of this type
    // (the caller has found the two tables one), compared as it writes `==` (std's `dyn_eq`
    // instance), each value borrowed.
    let eq = if m.dispatch.eqs.contains(identity) {
        match table.dyn_eq_symbol(concrete).map(str::to_string) {
            None => Some("null".to_string()),
            Some(compare) => {
                let Some(ret) = m.bodies.get(compare.as_str()).map(|body| body.return_ty()) else {
                    return unsupported("a comparing function with no body");
                };
                let function = format!("{symbol}_eq");
                define_c_fn(m, &function, "internal ", Some(ret), &[Param::Ptr, Param::Ptr], "", |cx, values, result| {
                    let mut arguments = Vec::with_capacity(2);
                    for value in values {
                        let Val::Imm(_, raw) = value.clone() else { unreachable!("a pointer") };
                        // A class value is its handle, the data word itself; a value passed by
                        // address is its address; another is read.
                        arguments.push(if class.is_some() || cx.m.ty.table.passed_by_address(concrete) {
                            Val::Imm(Sc::Ptr, raw)
                        } else {
                            let (_, align) = cx.m.ty.size_align(concrete);
                            match cx.m.ty.repr(concrete) {
                                crate::types::Repr::Scalar(sc) => Val::Imm(sc, cx.load(sc, &raw, align)),
                                crate::types::Repr::Zst => Val::Zst,
                                crate::types::Repr::Mem { .. } => Val::Mem(raw, align),
                            }
                        });
                    }
                    let dest = result.as_ref().map(|(ptr, align)| (ptr.as_str(), *align));
                    cx.call_body(&compare, arguments, dest)
                })?;
                Some(global_name(&function))
            }
        }
    } else {
        None
    };
    let mut slots = Vec::new();
    for (slot, (signature, implementation)) in adapter.layout.iter().zip(&adapter.implementations).enumerate() {
        let (Some((params, ret)), Some((implementation, receiver))) = (signature, implementation) else {
            slots.push("ptr null".to_string());
            continue;
        };
        let function = format!("{symbol}_slot{slot}");
        emit_slot_adapter(m, &function, concrete, class, params, *ret, implementation, *receiver)?;
        slots.push(format!("ptr {}", global_name(&function)));
    }
    let mut fields = vec![format!("ptr {drop}"), format!("i64 {size}"), format!("i64 {align}"), format!("ptr {access}")];
    fields.extend(clone.map(|clone| format!("ptr {clone}")));
    fields.extend(eq.map(|eq| format!("ptr {eq}")));
    fields.extend(slots);
    let types: Vec<&str> = fields.iter().map(|field| field.split(' ').next().unwrap_or("ptr")).collect();
    m.global(format!(
        "{} = internal constant {{ {} }} {{ {} }}, align 8",
        global_name(&symbol),
        types.join(", "),
        fields.join(", ")
    ));
    Ok(global_name(&symbol))
}

/// One slot's adapter: `ret slot(void* receiver, params…)`, calling the implementation with the
/// receiver its mode wants. A `mut self` call on a class object takes its write access here
/// (`[EXC-15]`): the caller holds only the erased value. A virtual implementation on a class
/// that can be subclassed is called through the object's own table, so an override runs (D-375).
#[allow(clippy::too_many_arguments)]
fn emit_slot_adapter(
    m: &mut Module<'_>,
    function: &str,
    concrete: Ty,
    class: Option<ClassId>,
    params: &[Ty],
    ret: Ty,
    implementation: &str,
    receiver: ParameterMode,
) -> Result<()> {
    let mut signature = vec![Param::Ptr];
    signature.extend(params.iter().map(|ty| Param::Ty(*ty)));
    let table = m.ty.table;
    let ret_is_view = table.is_view(ret);
    // D-375 — the slot of the implementation in a subclassable class's own table.
    let virtual_slot = class.filter(|class| table.class_def(*class).openness != ClassOpenness::Final).and_then(|class| {
        m.dispatch.virtual_layouts.get(&class).and_then(|slots| {
            slots.iter().position(|method| method.as_ref().is_some_and(|method| !method.is_abstract && method.symbol == implementation))
        })
    });
    let implementation = implementation.to_string();
    define_c_fn(m, function, "internal ", Some(ret), &signature, "", |cx, values, result| {
        // The receiver: the first parameter, after an `sret` result's.
        let Val::Imm(_, raw) = values[0].clone() else { unreachable!("the receiver is a pointer") };
        let receiver_value = match (receiver, class.is_some()) {
            // D-218 — a view-returning method takes its class receiver by address.
            (ParameterMode::Borrow, true) if ret_is_view => {
                let slot = cx.alloca(8, 8);
                cx.emit(format!("store ptr {raw}, ptr {slot}, align 8"));
                Val::Imm(Sc::Ptr, slot)
            }
            (ParameterMode::Borrow, true) => Val::Imm(Sc::Ptr, raw.clone()),
            (ParameterMode::Borrow, false)
                if cx.m.ty.table.passed_by_address(concrete) || (ret_is_view && !cx.m.ty.table.is_view(concrete)) =>
            {
                Val::Imm(Sc::Ptr, raw.clone())
            }
            (ParameterMode::Borrow, false) => {
                let (_, align) = cx.m.ty.size_align(concrete);
                match cx.m.ty.repr(concrete) {
                    crate::types::Repr::Scalar(sc) => Val::Imm(sc, cx.load(sc, &raw, align)),
                    crate::types::Repr::Zst => Val::Zst,
                    crate::types::Repr::Mem { .. } => Val::Mem(raw.clone(), align),
                }
            }
            (ParameterMode::Mut, true) => {
                let slot = cx.alloca(8, 8);
                cx.emit(format!("store ptr {raw}, ptr {slot}, align 8"));
                Val::Imm(Sc::Ptr, slot)
            }
            (ParameterMode::Mut, false) => Val::Imm(Sc::Ptr, raw.clone()),
            (ParameterMode::Owned, _) => return unsupported("an owned receiver in a `dyn` table"),
        };
        let object_write = receiver == ParameterMode::Mut && class.is_some();
        let nowhere = Val::Mem("@ember.nowhere".to_string(), 8);
        if object_write {
            cx.m.declare("@ember.nowhere", || "@ember.nowhere = internal constant { ptr, i32, i32 } zeroinitializer, align 8".to_string());
            cx.call_rt("object_begin_write", vec![Val::Imm(Sc::Ptr, raw.clone()), nowhere.clone()], None);
        }
        let mut arguments = vec![receiver_value];
        arguments.extend(values[1..].iter().cloned());
        let dest = result.as_ref().map(|(ptr, align)| (ptr.as_str(), *align));
        match virtual_slot {
            Some(slot) => {
                let entry = cx.vtable_entry(&raw, slot);
                let body = cx.m.bodies.get(implementation.as_str()).copied();
                let Some(body) = body else { return unsupported("a virtual implementation without a body") };
                let sig = crate::func::body_signature(cx.m, body);
                let sizes: Vec<(Val, u64, u64)> = arguments
                    .into_iter()
                    .zip(body.args())
                    .map(|(value, (_, decl))| {
                        let (size, align) = cx.m.ty.size_align(decl.ty);
                        (value, size, align)
                    })
                    .collect();
                let (size, _) = cx.m.ty.size_align(body.return_ty());
                cx.lowered_call(&entry, &sig, &sizes, dest.map(|(ptr, align)| (ptr, align, size)), "");
            }
            None => cx.call_body(&implementation, arguments, dest)?,
        }
        if object_write {
            cx.call_rt("object_end_write", vec![Val::Imm(Sc::Ptr, raw), nowhere], None);
        }
        Ok(())
    })
}

impl<'a> FnCx<'_, 'a> {
    /// The function in slot `slot` of the method table of the object `object` points at.
    pub(crate) fn vtable_entry(&mut self, object: &str, slot: usize) -> String {
        let info_at = self.gep(object, 16);
        let info = self.load(Sc::Ptr, &info_at, 8);
        let vtable_at = self.gep(&info, 48);
        let vtable = self.load(Sc::Ptr, &vtable_at, 8);
        let entry_at = self.gep(&vtable, 8 * slot as u64);
        self.load(Sc::Ptr, &entry_at, 8)
    }

    /// `[DSP-2]` — a call through the receiver's class table, or straight to the one body every
    /// class the object can be holds in the slot (`[DSP-1]`, `[DSP-5]`).
    pub(crate) fn virtual_call(&mut self, owner: ClassId, slot: usize, receiver: ClassId, args: &[Operand], dest: Option<(String, u64)>) -> Result<()> {
        let mut values = Vec::with_capacity(args.len());
        for argument in args {
            values.push(self.operand(argument)?);
        }
        let dest = dest.as_ref().map(|(ptr, align)| (ptr.as_str(), *align));
        self.virtual_call_values(owner, slot, receiver, values, dest)
    }

    /// `virtual_call` of the argument values themselves.
    pub(crate) fn virtual_call_values(&mut self, owner: ClassId, slot: usize, receiver: ClassId, values: Vec<Val>, dest: Option<(&str, u64)>) -> Result<()> {
        if let Some(symbol) = self.m.dispatch.only_implementation(self.m, receiver, slot) {
            return self.call_body(&symbol, values, dest);
        }
        let Some(declaring) = self.m.dispatch.slot_declarations.get(&(owner, slot)).cloned() else {
            return unsupported("a virtual call through a slot no class declares");
        };
        let Some(body) = self.m.bodies.get(declaring.as_str()).copied() else {
            return unsupported("a virtual slot without a declaring body");
        };
        // The object: the receiver, or what it points at when it is a reference.
        let receiver_ty = body.args().next().map(|(_, decl)| decl.ty);
        let Some(Val::Imm(_, receiver_value)) = values.first().cloned() else {
            return unsupported("a virtual call without a receiver");
        };
        let object = match receiver_ty.map(|ty| self.m.ty.kind(ty).clone()) {
            Some(TyKind::Ref { .. }) => self.load(Sc::Ptr, &receiver_value, 8),
            _ => receiver_value,
        };
        let entry = self.vtable_entry(&object, slot);
        let sig = crate::func::body_signature(self.m, body);
        let sized: Vec<(Val, u64, u64)> = values
            .into_iter()
            .zip(body.args())
            .map(|(value, (_, decl))| {
                let (size, align) = self.m.ty.size_align(decl.ty);
                (value, size, align)
            })
            .collect();
        let (size, _) = self.m.ty.size_align(body.return_ty());
        self.lowered_call(&entry, &sig, &sized, dest.map(|(ptr, align)| (ptr, align, size)), "");
        Ok(())
    }

    /// `[TYP-22]`, `[DSP-3]` — a call through an interface's table: a class-interface handle's,
    /// found through its type information (`ember_itable_lookup`, reproduced), or a `dyn`
    /// carrier's own.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn interface_call(
        &mut self,
        interfaces: &[Symbol],
        interface: Symbol,
        slot: usize,
        params: &[Ty],
        ret: Ty,
        class_handle: bool,
        args: &[Operand],
        dest: Option<(String, u64)>,
    ) -> Result<()> {
        let identity = identity(self.m, interfaces);
        let receiver_operand = args.first().expect("an interface call has a receiver");
        let mut values = Vec::with_capacity(args.len());
        let (data, table) = if class_handle {
            let mut handle = match self.operand(receiver_operand)? {
                Val::Imm(Sc::Ptr, handle) => handle,
                other => return unsupported(format!("a class-interface receiver held as {other:?}")),
            };
            // A `ref mut I` receiver: the handle is what it points at.
            if let Some(ty) = self.operand_type(receiver_operand)
                && let TyKind::Ref { inner, .. } = self.m.ty.kind(ty)
                && matches!(self.m.ty.kind(*inner), TyKind::ClassInterface(_))
            {
                handle = self.load(Sc::Ptr, &handle, 8);
            }
            let marker = interface_id(self.m, &self::identity(self.m, &[interface]));
            let lookup = itable_lookup(self.m);
            let info_at = self.gep(&handle, 16);
            let info = self.load(Sc::Ptr, &info_at, 8);
            let table = self.value(format!("call ptr {lookup}(ptr {info}, ptr {marker})"));
            (handle, table)
        } else {
            let carrier = self.operand(receiver_operand)?;
            let (carrier, align) = self.spill(&carrier, 16, 8);
            let data = self.load(Sc::Ptr, &carrier, align);
            let vtable_at = self.gep(&carrier, 8);
            let vtable = self.load(Sc::Ptr, &vtable_at, align.min(8));
            (data, vtable)
        };
        values.push(Val::Imm(Sc::Ptr, data));
        for argument in &args[1..] {
            values.push(self.operand(argument)?);
        }
        let entry_at = self.gep(&table, slot_offset(self.m, &identity, slot));
        let entry = self.load(Sc::Ptr, &entry_at, 8);
        let mut signature = vec![Param::Ptr];
        signature.extend(params.iter().map(|ty| Param::Ty(*ty)));
        let dest = dest.as_ref().map(|(ptr, align)| (ptr.as_str(), *align));
        self.call_c(&entry, Some(ret), &signature, values, dest);
        Ok(())
    }

    fn operand_type(&self, operand: &Operand) -> Option<Ty> {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => Some(self.place_ty(place)),
            _ => None,
        }
    }

    /// `[TYP-22]` — a `Box[dyn I]`: the value moved into a block of its own (a class handle is its
    /// own), with the concrete type's table.
    pub(crate) fn dyn_box_new(&mut self, concrete: Ty, interfaces: &[Symbol], args: &[Operand], dest: Option<(String, u64)>) -> Result<()> {
        let identity = identity(self.m, interfaces);
        let table = adapter_table(self.m, concrete, &identity)?;
        let value = self.operand(&args[0])?;
        let data = if matches!(self.m.ty.kind(concrete), TyKind::Class(_)) {
            let Val::Imm(_, handle) = value else { return unsupported("a class handle held in memory") };
            handle
        } else {
            let (size, align) = self.m.ty.size_align(concrete);
            let (source, _) = self.spill(&value, size, align);
            let slot = self.alloca(8, 8);
            self.call_rt(
                "box_new_copy",
                vec![Val::Imm(Sc::I(64), size.max(1).to_string()), Val::Imm(Sc::I(64), align.to_string()), Val::Imm(Sc::Ptr, source)],
                Some((&slot, 8)),
            );
            self.load(Sc::Ptr, &slot, 8)
        };
        if let Some((to, align)) = dest {
            self.emit(format!("store ptr {data}, ptr {to}, align {}", align.min(8)));
            let vtable_at = self.gep(&to, 8);
            self.emit(format!("store ptr {table}, ptr {vtable_at}, align {}", align.min(8)));
        }
        Ok(())
    }

    /// `[TYP-22]` — a borrow as the `{data, vtable}` carrier of the concrete type's table: a class
    /// handle's object, anything else the value the borrow points at.
    pub(crate) fn interface_upcast(&mut self, concrete: Ty, interfaces: &[Symbol], operand: &Operand) -> Result<Val> {
        let identity = identity(self.m, interfaces);
        let table = adapter_table(self.m, concrete, &identity)?;
        let Val::Imm(_, pointer) = self.operand(operand)? else {
            return unsupported("a `dyn` made from a value held in memory");
        };
        let data = if matches!(self.m.ty.kind(concrete), TyKind::Class(_)) { self.load(Sc::Ptr, &pointer, 8) } else { pointer };
        let carrier = self.alloca(16, 8);
        self.emit(format!("store ptr {data}, ptr {carrier}, align 8"));
        let vtable_at = self.gep(&carrier, 8);
        self.emit(format!("store ptr {table}, ptr {vtable_at}, align 8"));
        Ok(Val::Mem(carrier, 8))
    }

    /// `[OBJ-2]` — a class handle as an interface handle: the same pointer, its table found
    /// through the object's type information (`class_itables`).
    pub(crate) fn class_interface_upcast(&mut self, operand: &Operand) -> Result<Val> {
        self.operand(operand)
    }

    /// The `dyn` value behind a `Box[dyn I]` or a reference to one, and its table: an access
    /// begun or ended through the table's `access` entry, which only a class concrete has (F4).
    pub(crate) fn dyn_access(&mut self, at: &PlaceRef, code: u8, loc: Val) {
        let data = self.load(Sc::Ptr, &at.ptr, at.align.min(8));
        let vtable_at = self.gep(&at.ptr, 8);
        let vtable = self.load(Sc::Ptr, &vtable_at, at.align.min(8));
        let access_at = self.gep(&vtable, 24);
        let access = self.load(Sc::Ptr, &access_at, 8);
        let has = self.value(format!("icmp ne ptr {access}, null"));
        let (call, done) = (self.label(), self.label());
        self.emit(format!("br i1 {has}, label %{call}, label %{done}"));
        self.start(&call);
        self.call_c(&access, None, &[Param::Ptr, Param::Int, Param::Loc], vec![Val::Imm(Sc::Ptr, data), Val::Imm(Sc::I(32), code.to_string()), loc], None);
        self.emit(format!("br label %{done}"));
        self.start(&done);
    }
}

/// `[DSP-3]` — `ember_itable_lookup`, the header's inline table search, reproduced: the object's
/// own entries, which for a class this program wrote carry its bases' too, then the runtime's
/// out-of-line walk up the base chain.
fn itable_lookup(m: &mut Module<'_>) -> String {
    let symbol = "ember.itable_lookup";
    if m.helper_defined(symbol) {
        return global_name(symbol);
    }
    let mut cx = FnCx::glue(m);
    let (info, interface) = ("%p0", "%p1");
    let entries_at = cx.gep(info, 56);
    let entries = cx.load(Sc::Ptr, &entries_at, 8);
    let count_at = cx.gep(info, 64);
    let count = cx.load(Sc::I(32), &count_at, 8);
    let (head, body, found, next, slow) = (cx.label(), cx.label(), cx.label(), cx.label(), cx.label());
    let entry_block = cx.block_label();
    cx.emit(format!("br label %{head}"));
    cx.start(&head);
    let left = cx.tmp();
    let at = cx.tmp();
    let (left_next, at_next) = (cx.tmp(), cx.tmp());
    cx.emit(format!("{left} = phi i32 [ {count}, %{entry_block} ], [ {left_next}, %{next} ]"));
    cx.emit(format!("{at} = phi ptr [ {entries}, %{entry_block} ], [ {at_next}, %{next} ]"));
    let done = cx.value(format!("icmp eq i32 {left}, 0"));
    cx.emit(format!("br i1 {done}, label %{slow}, label %{body}"));
    cx.start(&body);
    let id = cx.load(Sc::Ptr, &at, 8);
    let same = cx.value(format!("icmp eq ptr {id}, {interface}"));
    // `ember_same_identity` — in a reloadable image two records are one identity when the runtime
    // made them one: their canonical records, each image having its own. Neither is null: an
    // entry and the marker asked for are both an interface's record.
    let same = if cx.m.reload.is_some() {
        let canonical_at = cx.gep(&id, 8);
        let canonical = cx.load(Sc::Ptr, &canonical_at, 8);
        let wanted_at = cx.gep(interface, 8);
        let wanted = cx.load(Sc::Ptr, &wanted_at, 8);
        let one = cx.value(format!("icmp eq ptr {canonical}, {wanted}"));
        let made = cx.value(format!("icmp ne ptr {canonical}, null"));
        let one = cx.value(format!("and i1 {one}, {made}"));
        cx.value(format!("or i1 {same}, {one}"))
    } else {
        same
    };
    cx.emit(format!("br i1 {same}, label %{found}, label %{next}"));
    cx.start(&found);
    let table_at = cx.gep(&at, 8);
    let table = cx.load(Sc::Ptr, &table_at, 8);
    cx.emit(format!("ret ptr {table}"));
    cx.start(&next);
    cx.emit(format!("{left_next} = sub i32 {left}, 1"));
    cx.emit(format!("{at_next} = getelementptr i8, ptr {at}, i64 16"));
    cx.emit(format!("br label %{head}"));
    cx.start(&slow);
    let base_at = cx.gep(info, 24);
    let base = cx.load(Sc::Ptr, &base_at, 8);
    let result = cx.alloca(8, 8);
    cx.call_rt("itable_lookup_slow", vec![Val::Imm(Sc::Ptr, base), Val::Imm(Sc::Ptr, interface.to_string())], Some((&result, 8)));
    let table = cx.load(Sc::Ptr, &result, 8);
    cx.emit(format!("ret ptr {table}"));
    let attrs = crate::func::fn_attrs(cx.m);
    let text = format!(
        "define internal ptr {}(ptr %p0, ptr %p1) {attrs} alwaysinline {{\nentry:\n{}  br label %start\nstart:\n{}}}\n\n",
        global_name(symbol),
        cx.allocas_text(),
        cx.out_text()
    );
    cx.m.fns.push_str(&text);
    global_name(symbol)
}
