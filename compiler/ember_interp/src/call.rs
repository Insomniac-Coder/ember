//! Calls: direct, through a class's virtual table (`[DSP-2]`), through an interface's table
//! (`[TYP-22]`, `[DSP-3]`), through a callable value (`[CLO-3]`, `[CLO-10]`), and the dynamic
//! `Box` a concrete value becomes. The tables are the evaluator's own, built from what MIR says,
//! as the C backend builds its: a class's slots from the bodies that declare them, an interface
//! table from the cast that makes the value.

use std::collections::HashMap;

use ember_mir::{BasicBlockId, FuncRef, InterfaceAdapterMethod, Operand, ParameterMode, Place};
use ember_span::{Span, Symbol};
use ember_types::{ClassId, ClassOpenness, FnParamMode, Ty, TyKind};

use crate::memory::{self, Region};
use crate::{fault, two_words, u64_of, Interp, Ret, R};

/// An interface table: the concrete type behind it and each slot's method.
pub struct Table {
    pub concrete: Ty,
    pub implementations: Vec<Option<InterfaceAdapterMethod>>,
}

/// A callable value's descriptor, one per closure body and state type (`descriptor_ids`): the
/// state's type, which its drop and clone follow.
pub struct Descriptor {
    pub env: Ty,
}

#[derive(Default)]
pub struct Dispatch {
    /// Each class's virtual table: the body in each slot, its bases' first.
    virtuals: HashMap<ClassId, Vec<Option<usize>>>,
    /// Each class's interface implementations, from the casts that make class-interface handles.
    itables: HashMap<(ClassId, Symbol), Vec<Option<InterfaceAdapterMethod>>>,
    pub tables: Vec<Table>,
    table_ids: HashMap<(Ty, Vec<Symbol>), usize>,
    pub descriptors: Vec<Descriptor>,
    descriptor_ids: HashMap<(usize, Ty), usize>,
}

/// What a frame does when it returns, before its result is written: a `mut self` interface
/// call's adapter ends the write access it began on the object (`[EXC-15]`).
pub(crate) enum After {
    EndObjectWrite(u64),
    FreeSlot(u64),
}

/// A callable value: its entry, its descriptor, three words of state.
pub fn fn_value(entry: u64, descriptor: u64, words: [u64; 3]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(40);
    for word in [entry, descriptor, words[0], words[1], words[2]] {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    bytes
}

impl Dispatch {
    pub(crate) fn new(interp: &Interp<'_, '_>) -> Dispatch {
        let mut dispatch = Dispatch::default();
        let types = interp.types;
        let mut declared: HashMap<(ClassId, usize), usize> = HashMap::new();
        for (index, body) in interp.bodies.iter().enumerate() {
            if let (Some(owner), Some(slot)) = (body.class_owner, body.class_virtual_slot) {
                declared.insert((owner, slot), index);
            }
            for block in &body.blocks {
                for stmt in &block.stmts {
                    let ember_mir::StmtKind::Assign { rvalue: ember_mir::Rvalue::Cast { kind: ember_mir::CastKind::ClassInterfaceUpcast { concrete, interface, implementations, .. }, .. }, .. } = &stmt.kind else {
                        continue;
                    };
                    if let TyKind::Class(class) = *types.kind(*concrete) {
                        dispatch.itables.entry((class, *interface)).or_insert_with(|| implementations.clone());
                    }
                }
            }
        }
        let classes: Vec<ClassId> = types.runtime_classes().map(|(id, _)| id).collect();
        for id in classes {
            dispatch.layout(interp, id, &declared);
        }
        dispatch
    }

    fn layout(&mut self, interp: &Interp<'_, '_>, id: ClassId, declared: &HashMap<(ClassId, usize), usize>) -> Vec<Option<usize>> {
        if let Some(layout) = self.virtuals.get(&id) {
            return layout.clone();
        }
        let mut layout = interp.types.class_def(id).base.map(|base| self.layout(interp, base, declared)).unwrap_or_default();
        for (&(owner, slot), &body) in declared {
            if owner != id {
                continue;
            }
            if layout.len() <= slot {
                layout.resize(slot + 1, None);
            }
            layout[slot] = Some(body);
        }
        self.virtuals.insert(id, layout.clone());
        layout
    }

    /// The interface table for `concrete` behind `interfaces`, registered on first use.
    pub fn table(&mut self, concrete: Ty, interfaces: &[Symbol], implementations: &[Option<InterfaceAdapterMethod>]) -> u64 {
        let key = (concrete, interfaces.to_vec());
        let index = match self.table_ids.get(&key) {
            Some(&index) => index,
            None => {
                self.tables.push(Table { concrete, implementations: implementations.to_vec() });
                self.table_ids.insert(key, self.tables.len() - 1);
                self.tables.len() - 1
            }
        };
        memory::address(Region::Table, index)
    }

    pub fn descriptor(&mut self, closure: usize, env: Ty) -> u64 {
        let index = *self.descriptor_ids.entry((closure, env)).or_insert_with(|| {
            self.descriptors.push(Descriptor { env });
            self.descriptors.len() - 1
        });
        memory::address(Region::Descriptor, index)
    }
}

impl<'a> Interp<'a, '_> {
    pub(crate) fn body_now(&self) -> &'a ember_mir::Body {
        self.frames.last().expect("a frame is running").body
    }

    /// The parameters' modes of what `func` calls, for `[RC-1]`'s retain of a copied handle given
    /// to an owned parameter.
    fn owned_argument(&self, func: &FuncRef, index: usize) -> bool {
        match func {
            FuncRef::Direct { symbol, .. } => self
                .by_symbol
                .get(symbol.as_str())
                .and_then(|&body| self.bodies[body].param_modes.get(index))
                .is_some_and(|mode| *mode == ParameterMode::Owned),
            FuncRef::Indirect { ty, .. } => match self.types.kind(*ty) {
                TyKind::Fn { params, .. } => params.get(index).is_some_and(|p| p.mode == FnParamMode::Owned),
                _ => false,
            },
            FuncRef::Interface { param_modes, .. } => {
                index.checked_sub(1).and_then(|p| param_modes.get(p)).is_some_and(|mode| *mode == ParameterMode::Owned)
            }
            FuncRef::Virtual { param_modes, .. } => param_modes.get(index).is_some_and(|mode| *mode == ParameterMode::Owned),
            _ => false,
        }
    }

    /// Each argument's bytes, a copied one given to an owned parameter retained first.
    fn arguments(&mut self, func: &FuncRef, args: &[Operand], params: &[Ty]) -> R<Vec<Vec<u8>>> {
        let body = self.body_now();
        let mut values = Vec::with_capacity(args.len());
        for (index, arg) in args.iter().enumerate() {
            if self.owned_argument(func, index)
                && let Operand::Copy(place) = arg
            {
                let ty = self.place_ty(body, place);
                let loc = self.place(place)?;
                self.retain_loc(loc, ty)?;
            }
            let want = params.get(index).copied().or_else(|| self.operand_ty(body, arg));
            values.push(self.operand(arg, want)?);
        }
        Ok(values)
    }

    fn param_types(&self, index: usize) -> Vec<Ty> {
        let body = &self.bodies[index];
        body.args().map(|(_, decl)| decl.ty).collect()
    }

    pub(crate) fn call(&mut self, func: &FuncRef, args: &[Operand], dest: &Place, next: BasicBlockId, span: Span) -> R<()> {
        match func {
            FuncRef::Direct { symbol, .. } => {
                let Some(&index) = self.by_symbol.get(symbol.as_str()) else {
                    return fault(format!("a call of `{symbol}`, which has no body"));
                };
                let params = self.param_types(index);
                let values = self.arguments(func, args, &params)?;
                self.enter(index, values, Ret::Place(dest.clone(), next))
            }
            FuncRef::Virtual { owner, slot, .. } => {
                let mut values = self.arguments(func, args, &[])?;
                let receiver = u64_of(&values[0]);
                // The slot's receiver type says whether the argument is the handle or its address.
                let by_reference = self.virtual_receiver_is_reference(*owner, *slot);
                let object = if by_reference { self.mem.read_u64(receiver)? } else { receiver };
                let class = self.object_class(object)?;
                let Some(index) = self.virtual_body(class, *slot) else {
                    return fault(format!("no body in slot {slot} of `{}`", self.types.class_display_name(class)));
                };
                self.fit_arguments(index, &mut values)?;
                self.enter(index, values, Ret::Place(dest.clone(), next))
            }
            FuncRef::Interface { slot, class_handle, interface, .. } => {
                let values = self.arguments(func, args, &[])?;
                self.interface_call(values, *slot, *class_handle, *interface, args, Ret::Place(dest.clone(), next))
            }
            FuncRef::Indirect { operand, ty, .. } => {
                let callee = self.operand(operand, Some(*ty))?;
                match self.types.kind(*ty) {
                    TyKind::Fn { abi: None, .. } => {
                        let fn_place = match operand {
                            Operand::Copy(place) | Operand::Move(place) => Some(self.place(place)?),
                            Operand::Const(_) => None,
                        };
                        let values = self.arguments(func, args, &[])?;
                        self.call_callable(&callee, fn_place.map(|loc| loc.addr), values, Ret::Place(dest.clone(), next))
                    }
                    _ => {
                        let code = u64_of(&callee);
                        let Some((Region::Code, index)) = memory::decode(code) else {
                            return fault(format!("a call through {code:#x}, which is no function"));
                        };
                        let params = self.param_types(index);
                        let values = self.arguments(func, args, &params)?;
                        self.enter(index, values, Ret::Place(dest.clone(), next))
                    }
                }
            }
            FuncRef::DynBoxNew { concrete, interfaces, implementations, .. } => {
                let value = self.operand(&args[0], Some(*concrete))?;
                let data = if matches!(self.types.kind(*concrete), TyKind::Class(_)) {
                    u64_of(&value)
                } else {
                    let layout = self.layout(*concrete);
                    let data = self.mem.alloc(layout.size, layout.align)?;
                    self.mem.write(data, &value)?;
                    data
                };
                let table = self.calls.table(*concrete, interfaces, implementations);
                let loc = self.place(dest)?;
                self.store(loc, &two_words(data, table))?;
                self.goto(next)
            }
            FuncRef::Builtin { which, arg_ty } => {
                let result = self.builtin(which, *arg_ty, args, dest, span)?;
                if let Some(bytes) = result {
                    let loc = self.place(dest)?;
                    self.store(loc, &bytes)?;
                }
                self.goto(next)
            }
        }
    }

    fn virtual_receiver_is_reference(&self, owner: ClassId, slot: usize) -> bool {
        // The slot's signature is the first declaration's in the chain (`virtual_slot_signature`).
        let declared = self.bodies.iter().find(|body| body.class_owner == Some(owner) && body.class_virtual_slot == Some(slot));
        declared.is_some_and(|body| body.locals.get(1).is_some_and(|receiver| matches!(self.types.kind(receiver.ty), TyKind::Ref { .. })))
    }

    pub(crate) fn virtual_body(&self, class: ClassId, slot: usize) -> Option<usize> {
        self.calls.virtuals.get(&class).and_then(|slots| slots.get(slot).copied().flatten()).filter(|&body| !self.bodies[body].is_abstract)
    }

    /// An override whose receiver is the handle where the slot's takes its address, or the
    /// other way round, gets the argument it declares.
    fn fit_arguments(&mut self, index: usize, values: &mut [Vec<u8>]) -> R<()> {
        let _ = (index, values);
        Ok(())
    }

    /// `[TYP-22]`, `[OBJ-2]` — a call through an interface's table: the receiver's adapter, which
    /// gives the method the receiver as it takes it (`emit_interface_adapters`).
    fn interface_call(&mut self, mut values: Vec<Vec<u8>>, slot: usize, class_handle: bool, interface: Symbol, args: &[Operand], ret: Ret) -> R<()> {
        let (data, implementation, concrete) = if class_handle {
            // A one-word class-interface handle, or a `ref mut` to one.
            let body = self.body_now();
            let receiver_ty = self.operand_ty(body, &args[0]);
            let mut object = u64_of(&values[0]);
            if receiver_ty.is_some_and(|ty| matches!(self.types.kind(ty), TyKind::Ref { inner, .. } if matches!(self.types.kind(*inner), TyKind::ClassInterface(_)))) {
                object = self.mem.read_u64(object)?;
            }
            let class = self.object_class(object)?;
            let Some((owner, methods)) = self.class_itable(class, interface) else {
                return fault(format!("`{}` has no table for `{interface}`", self.types.class_display_name(class)));
            };
            // The class whose cast made the table: its slots say which virtual a method is.
            let concrete = self.types.find(&TyKind::Class(owner)).unwrap_or(self.common.void);
            (object, methods.get(slot).cloned().flatten(), concrete)
        } else {
            let fat = &values[0];
            let (data, table) = (u64_of(&fat[..8]), u64_of(&fat[8..16]));
            let Some((Region::Table, index)) = memory::decode(table) else {
                return fault("an interface call through no table");
            };
            let table = &self.calls.tables[index];
            (data, table.implementations.get(slot).cloned().flatten(), table.concrete)
        };
        let Some(method) = implementation else {
            return fault(format!("an interface call of slot {slot}, which has no implementation"));
        };
        let Some(mut index) = self.by_symbol.get(method.symbol.as_str()).copied() else {
            return fault(format!("no body `{}`", method.symbol));
        };
        let class = match self.types.kind(concrete) {
            TyKind::Class(class) => Some(*class),
            _ => None,
        };
        // D-375 — an open class's override runs: through the object's own table.
        if let Some(class) = class
            && self.types.class_def(class).openness != ClassOpenness::Final
            && let Some(slot) = self.calls.virtuals.get(&class).and_then(|slots| slots.iter().position(|b| *b == Some(index)))
        {
            let object_class = self.object_class(data)?;
            if let Some(body) = self.virtual_body(object_class, slot) {
                index = body;
            }
        }
        let callee = &self.bodies[index];
        let ret_ty = callee.return_ty();
        let mut after = None;
        let receiver = match (method.receiver, class.is_some()) {
            (ParameterMode::Borrow, true) if self.types.is_view(ret_ty) => {
                let slot = self.mem.alloc(8, 8)?;
                self.mem.write_u64(slot, data)?;
                after = Some(After::FreeSlot(slot));
                slot.to_le_bytes().to_vec()
            }
            (ParameterMode::Borrow, true) => data.to_le_bytes().to_vec(),
            (ParameterMode::Borrow, false)
                if self.types.passed_by_address(concrete) || (self.types.is_view(ret_ty) && !self.types.is_view(concrete)) =>
            {
                data.to_le_bytes().to_vec()
            }
            (ParameterMode::Borrow, false) => self.mem.read(data, self.size(concrete))?.to_vec(),
            // `[EXC-15]` — the adapter takes the whole object for the call, at no place of the
            // program's (its `nowhere`).
            (ParameterMode::Mut, true) => {
                self.object_access(data, true, true, Span::DUMMY)?;
                let slot = self.mem.alloc(8, 8)?;
                self.mem.write_u64(slot, data)?;
                after = Some(After::EndObjectWrite(slot));
                slot.to_le_bytes().to_vec()
            }
            (ParameterMode::Mut, false) => data.to_le_bytes().to_vec(),
            (ParameterMode::Owned, _) => return fault("an owned receiver in an interface table"),
        };
        values[0] = receiver;
        self.enter(index, values, ret)?;
        if let Some(after) = after {
            self.frames.last_mut().expect("the frame just entered").after = Some(after);
        }
        Ok(())
    }

    /// The methods class `class` (or a base, which it names) implements interface `interface` with.
    fn class_itable(&self, class: ClassId, interface: Symbol) -> Option<(ClassId, Vec<Option<InterfaceAdapterMethod>>)> {
        let mut current = Some(class);
        while let Some(id) = current {
            if let Some(methods) = self.calls.itables.get(&(id, interface)) {
                return Some((id, methods.clone()));
            }
            current = self.types.class_def(id).base;
        }
        None
    }

    /// What `After` asks, as the frame returns.
    pub(crate) fn run_after(&mut self, after: After) -> R<()> {
        match after {
            After::EndObjectWrite(slot) => {
                let object = self.mem.read_u64(slot)?;
                self.mem.free(slot)?;
                self.object_access(object, true, false, Span::DUMMY)
            }
            After::FreeSlot(slot) => Ok(self.mem.free(slot)?),
        }
    }

    // -- callable values (`[CLO-3]`, `[CLO-10]`) --------------------------------------------

    /// `fn_value_make`: a closure's value, its state in the value's three words when it fits
    /// them and in an allocation (its pointer the first word) otherwise.
    pub(crate) fn make_callable(&mut self, closure: &str, env: Ty, env_bytes: &[u8]) -> R<Vec<u8>> {
        let Some(&index) = self.by_symbol.get(closure) else {
            return fault(format!("no closure body `{closure}`"));
        };
        let descriptor = self.calls.descriptor(index, env);
        let mut words = [0u64; 3];
        if env_bytes.len() <= 24 {
            let mut padded = env_bytes.to_vec();
            padded.resize(24, 0);
            for (i, word) in words.iter_mut().enumerate() {
                *word = u64_of(&padded[i * 8..i * 8 + 8]);
            }
        } else {
            let layout = self.layout(env);
            let boxed = self.mem.alloc(layout.size, layout.align)?;
            self.mem.write(boxed, env_bytes)?;
            words[0] = boxed;
        }
        Ok(fn_value(memory::address(Region::Code, index), descriptor, words))
    }

    /// A call through a callable value. `place` is where the value lives, whose state a closure
    /// reads and writes in place.
    fn call_callable(&mut self, value: &[u8], place: Option<u64>, mut values: Vec<Vec<u8>>, ret: Ret) -> R<()> {
        let entry = u64_of(&value[..8]);
        let descriptor = u64_of(&value[8..16]);
        let Some((Region::Code, index)) = memory::decode(entry) else {
            return fault(format!("a callable value whose entry {entry:#x} is no function"));
        };
        // A function's value: no state.
        if matches!(memory::decode(descriptor), Some((Region::Code, _))) {
            return self.enter(index, values, ret);
        }
        let Some((Region::Descriptor, d)) = memory::decode(descriptor) else {
            return fault("a callable value with no descriptor");
        };
        let env = self.calls.descriptors[d].env;
        let env_size = self.size(env);
        // The state: in the words, or behind the first.
        let mut temporary = None;
        let state = if env_size <= 24 {
            match place {
                Some(addr) => addr + 16,
                None => {
                    let slot = self.mem.alloc(24, 8)?;
                    self.mem.write(slot, &value[16..40])?;
                    temporary = Some(slot);
                    slot
                }
            }
        } else {
            u64_of(&value[16..24])
        };
        let body = &self.bodies[index];
        let env_param = body.locals.get(1).map(|local| local.ty);
        let env_argument = if env_param.is_some_and(|ty| matches!(self.types.kind(ty), TyKind::Ref { .. } | TyKind::Ptr { .. })) {
            state.to_le_bytes().to_vec()
        } else {
            self.mem.read(state, env_size)?.to_vec()
        };
        values.insert(0, env_argument);
        self.enter(index, values, ret)?;
        if let Some(slot) = temporary {
            self.frames.last_mut().expect("the frame just entered").after = Some(After::FreeSlot(slot));
        }
        Ok(())
    }

    /// `d->drop(&w)`: a closure value's state ended, and its allocation freed.
    pub(crate) fn drop_callable(&mut self, addr: u64) -> R<()> {
        let descriptor = self.mem.read_u64(addr + 8)?;
        if descriptor == 0 || matches!(memory::decode(descriptor), Some((Region::Code, _))) {
            return Ok(());
        }
        let Some((Region::Descriptor, d)) = memory::decode(descriptor) else {
            return fault("a callable value with no descriptor");
        };
        let env = self.calls.descriptors[d].env;
        let size = self.size(env);
        if size <= 24 {
            if self.types.needs_drop(env) {
                self.drop_at(addr + 16, env)?;
            }
        } else {
            let boxed = self.mem.read_u64(addr + 16)?;
            if self.types.needs_drop(env) {
                self.drop_at(boxed, env)?;
            }
            self.mem.free(boxed)?;
        }
        Ok(())
    }

    /// A `Box[dyn I]`'s end: its table's drop of the payload, then the payload's allocation (a
    /// class handle's box is the handle, released).
    pub(crate) fn drop_dyn_box(&mut self, data: u64, table: u64) -> R<()> {
        if data == 0 {
            return Ok(());
        }
        let Some((Region::Table, index)) = memory::decode(table) else {
            return fault("a dynamic box with no table");
        };
        let concrete = self.calls.tables[index].concrete;
        if matches!(self.types.kind(concrete), TyKind::Class(_)) {
            return self.release_object(data);
        }
        if self.types.needs_drop(concrete) {
            self.drop_at(data, concrete)?;
        }
        Ok(self.mem.free(data)?)
    }

    /// `[EXC-18]` — an access to a `dyn` value's object, through its table: a class concrete's
    /// whole-object access; nothing for any other.
    pub(crate) fn dyn_access(&mut self, place: &Place, mutable: bool, begin: bool, span: Span) -> R<()> {
        let body = self.body_now();
        let ty = self.place_ty(body, place);
        let loc = self.place(place)?;
        let fat_at = match self.types.kind(ty) {
            TyKind::Ref { .. } => self.pointer_at(loc)?,
            _ => loc.addr,
        };
        let (data, table) = (self.mem.read_u64(fat_at)?, self.mem.read_u64(fat_at + 8)?);
        let Some((Region::Table, index)) = memory::decode(table) else {
            return Ok(());
        };
        if matches!(self.types.kind(self.calls.tables[index].concrete), TyKind::Class(_)) {
            self.object_access(data, mutable, begin, span)?;
        }
        Ok(())
    }
}
