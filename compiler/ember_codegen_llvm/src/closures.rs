//! Callable values (`[FN-6]`, `[CLO-3]`, `[CLO-10]`): a value of a native function type is the C
//! backend's `ember_fn_value`, `{native, descriptor, w[3]}` (40 bytes), its call one indirect call
//! through `native` with the value's state first; a named function's state is empty, a closure's
//! is its environment, in the three words when it fits them and else in a block whose address
//! is the first word. Each descriptor is `{native, c, drop, once}`, as the C backend's.

use std::collections::{BTreeMap, BTreeSet};

use crate::func::{FnCx, Param, Val, define_c_fn};
use crate::types::Sc;
use crate::{Module, Result, global_name, unsupported};
use ember_mir::{AggregateKind, Body, Builtin, CastKind, Const, FuncRef, Operand, ParameterMode, Rvalue, StmtKind, Terminator};
use ember_types::{FnParam, FnParamMode, Ty, TyKind};

#[derive(Default)]
pub(crate) struct Callables {
    /// Each named function used as a native value, with its function type.
    pub natives: BTreeMap<String, Ty>,
    /// Each closure made into an owned value, with its environment's type.
    pub closures: BTreeMap<String, Ty>,
    /// The function types the program gives to C (`CastKind::FnToC`).
    pub given_to_c: BTreeSet<Ty>,
    /// The descriptors written.
    written: BTreeSet<String>,
}

impl Callables {
    pub(crate) fn collect(m: &Module<'_>, bodies: &[Body]) -> Callables {
        let mut callables = Callables::default();
        let native: BTreeSet<&str> = bodies
            .iter()
            .filter(|body| body.abi.is_none() && !body.is_extern_declaration)
            .map(|body| body.symbol.as_str())
            .collect();
        let local_ty = |body: &Body, place: &ember_mir::Place| -> Ty {
            crate::func::place_type(m, body, place)
        };
        for body in bodies {
            for stmt in body.blocks.iter().flat_map(|block| &block.stmts) {
                match &stmt.kind {
                    StmtKind::Assign { place, rvalue: Rvalue::Use(Operand::Const(Const::Fn(symbol))) } if native.contains(symbol.as_str()) => {
                        let ty = local_ty(body, place);
                        if matches!(m.ty.kind(ty), TyKind::Fn { abi: None, .. }) {
                            callables.natives.insert(symbol.clone(), ty);
                        }
                    }
                    StmtKind::Assign { rvalue: Rvalue::Cast { kind: CastKind::FnToC, operand: Operand::Copy(from) | Operand::Move(from), .. }, .. } => {
                        callables.given_to_c.insert(local_ty(body, from));
                    }
                    StmtKind::Assign { rvalue: Rvalue::Aggregate { kind: AggregateKind::Callable, operands }, .. } => {
                        if let [Operand::Const(Const::Fn(closure)), env] = &operands[..]
                            && let Some(env_ty) = operand_ty(m, body, env)
                        {
                            callables.closures.entry(closure.clone()).or_insert(env_ty);
                        }
                    }
                    _ => {}
                }
            }
            for block in &body.blocks {
                if let Terminator::Call { func: FuncRef::Builtin { which: Builtin::ScopeSpawn { closure: Some(_) }, .. }, args, .. } = &block.terminator
                    && let [_, Operand::Const(Const::Fn(closure)), env, ..] = &args[..]
                    && let Some(env_ty) = operand_ty(m, body, env)
                {
                    callables.closures.entry(closure.clone()).or_insert(env_ty);
                }
            }
        }
        callables
    }
}

fn operand_ty(m: &Module<'_>, body: &Body, operand: &Operand) -> Option<Ty> {
    match operand {
        Operand::Copy(place) | Operand::Move(place) => Some(crate::func::place_type(m, body, place)),
        Operand::Const(Const::Int { ty, .. } | Const::Float { ty, .. } | Const::SourceLine { ty, .. }) => Some(*ty),
        _ => None,
    }
}

/// `[FN-6a]` — how a callable's parameter crosses: `ref` to a value passed by address and `mut`
/// (but a `MutSpan`) by pointer, anything else as itself. A C entry (`c_abi`) takes a borrowed
/// aggregate by value (`[FFI-9]`).
pub(crate) fn param_of(m: &Module<'_>, param: FnParam, c_abi: bool) -> Param {
    match param.mode {
        FnParamMode::Borrow if m.ty.table.passed_by_address(param.ty) && !c_abi => Param::Ptr,
        FnParamMode::Borrow | FnParamMode::Owned => Param::Ty(param.ty),
        FnParamMode::Mut if matches!(m.ty.kind(param.ty), TyKind::Span { mutable: true, .. }) => Param::Ty(param.ty),
        FnParamMode::Mut => Param::Ptr,
    }
}

/// The symbol of the descriptor of the named function `symbol` as a value.
pub(crate) fn native_descriptor_symbol(symbol: &str) -> String {
    format!("ember.native_fn_value.{symbol}")
}

/// A descriptor's fields after `native`, `c` and `drop`: in a reloadable image, what the captured
/// state holds (`[HR-15]`), then `once`.
fn descriptor_type(m: &Module<'_>) -> &'static str {
    if m.reload.is_some() { "{ ptr, ptr, ptr, ptr, ptr }" } else { "{ ptr, ptr, ptr, ptr }" }
}

/// Where a descriptor's `once` entry is.
fn once_offset(m: &Module<'_>) -> u64 {
    if m.reload.is_some() { 32 } else { 24 }
}

/// The descriptor of the named function `symbol` as a value of `signature`: its `native` thunk
/// ignores the (empty) state; its `c` entry exists where the program gives the type to C.
pub(crate) fn native_descriptor(m: &mut Module<'_>, symbol: &str, signature: Ty) -> Result<String> {
    let descriptor = native_descriptor_symbol(symbol);
    if !m.callables.written.insert(descriptor.clone()) {
        return Ok(global_name(&descriptor));
    }
    let TyKind::Fn { params, ret, .. } = m.ty.kind(signature).clone() else {
        return unsupported("a native function value of a non-function type");
    };
    let thunk = format!("{descriptor}.native");
    let mut thunk_params = vec![Param::Ptr];
    thunk_params.extend(params.iter().map(|param| param_of(m, *param, false)));
    let symbol_owned = symbol.to_string();
    define_c_fn(m, &thunk, "internal ", Some(ret), &thunk_params, "", |cx, values, result| {
        let dest = result.as_ref().map(|(ptr, align)| (ptr.as_str(), *align));
        cx.call_body(&symbol_owned, values[1..].to_vec(), dest)
    })?;
    // `[FFI-21]` — C's entry: the thread attached, a borrowed aggregate passed by value.
    let given = m.callables.given_to_c.contains(&signature);
    let ffi_safe = given
        && params.iter().all(|param| m.ty.table.is_ffi_safe(param.ty) && !m.ty.table.needs_drop(param.ty))
        && m.ty.table.is_ffi_safe(ret)
        && !m.ty.table.needs_drop(ret);
    let c_entry = if ffi_safe {
        let entry = format!("{descriptor}.c");
        let c_params: Vec<Param> = params.iter().map(|param| param_of(m, *param, true)).collect();
        let by_address: Vec<Option<(u64, u64)>> = params
            .iter()
            .map(|param| {
                (param.mode == FnParamMode::Borrow && m.ty.table.passed_by_address(param.ty)).then(|| m.ty.size_align(param.ty))
            })
            .collect();
        let symbol_owned = symbol.to_string();
        define_c_fn(m, &entry, "internal ", Some(ret), &c_params, "", |cx, values, result| {
            cx.call_rt("rt_thread_attach", Vec::new(), None);
            // `[HR-42]` — C calling back into Ember code: this thread's depth counter.
            let reload = cx.m.reload.is_some();
            if reload {
                cx.call_rt("reload_enter", Vec::new(), None);
            }
            let mut arguments = Vec::with_capacity(values.len());
            for (value, by_address) in values.iter().zip(&by_address) {
                match by_address {
                    // The callee takes its address.
                    Some((size, align)) => {
                        let (ptr, _) = cx.spill(value, *size, *align);
                        arguments.push(Val::Imm(Sc::Ptr, ptr));
                    }
                    None => arguments.push(value.clone()),
                }
            }
            let dest = result.as_ref().map(|(ptr, align)| (ptr.as_str(), *align));
            cx.call_body(&symbol_owned, arguments, dest)?;
            if reload {
                cx.call_rt("reload_leave", Vec::new(), None);
            }
            Ok(())
        })?;
        global_name(&entry)
    } else {
        "null".to_string()
    };
    let thunk = global_name(&thunk);
    let captures = if m.reload.is_some() { "ptr null, " } else { "" };
    m.global(format!(
        "{} = internal constant {} {{ ptr {thunk}, ptr {c_entry}, ptr null, {captures}ptr {thunk} }}, align 8",
        global_name(&descriptor),
        descriptor_type(m)
    ));
    Ok(global_name(&descriptor))
}

/// The descriptor of the closure `closure` made into an owned value: its `native` entry (the
/// environment found in the words or behind the first), its `drop` and its `once` call
/// (`[CLO-6a]`).
fn closure_descriptor(m: &mut Module<'_>, closure: &str) -> Result<String> {
    let descriptor = format!("ember.closure_value.{closure}");
    if !m.callables.written.insert(descriptor.clone()) {
        return Ok(global_name(&descriptor));
    }
    let Some(env) = m.callables.closures.get(closure).copied() else {
        return unsupported("a closure value no statement makes");
    };
    let Some(body) = m.bodies.get(closure).copied() else {
        return unsupported("a closure without a body");
    };
    let (env_size, env_align) = m.ty.size_align(env);
    let boxed = env_size > 24;
    let mut args = body.args();
    let (_, env_param) = args.next().expect("a closure takes its environment first");
    let by_pointer = matches!(m.ty.kind(env_param.ty), TyKind::Ref { .. } | TyKind::Ptr { .. });
    let rest: Vec<Ty> = args.map(|(_, decl)| decl.ty).collect();
    let ret = body.return_ty();
    let mut params = vec![Param::Ptr];
    params.extend(rest.iter().map(|ty| Param::Ty(*ty)));
    let closure_owned = closure.to_string();
    // The environment's own slot: the block, or a copy of the words (an environment aligned past
    // a pointer is never used in place, D-620), copied back after the call.
    // `state` is the state's pointer: the first parameter, after an `sret` result's.
    let find = move |cx: &mut FnCx<'_, '_>, state: &str| -> (String, bool) {
        if boxed {
            (cx.load(Sc::Ptr, state, 8), false)
        } else if env_align > 8 {
            let local = cx.alloca(env_size, env_align);
            cx.copy(&local, env_align, state, 8, env_size, false);
            (local, true)
        } else {
            (state.to_string(), false)
        }
    };
    let state_of = |values: &[Val]| match &values[0] {
        Val::Imm(_, state) => state.clone(),
        _ => unreachable!("the state is a pointer"),
    };
    let native = format!("{descriptor}.native");
    define_c_fn(m, &native, "internal ", Some(ret), &params, "", |cx, values, result| {
        let state = state_of(values);
        let (env_at, copied) = find(cx, &state);
        let env_value = if by_pointer { Val::Imm(Sc::Ptr, env_at.clone()) } else { env_val(cx, &env_at, env) };
        let mut arguments = vec![env_value];
        arguments.extend(values[1..].iter().cloned());
        let dest = result.as_ref().map(|(ptr, align)| (ptr.as_str(), *align));
        cx.call_body(&closure_owned, arguments, dest)?;
        if copied {
            cx.copy(&state, 8, &env_at, env_align, env_size, false);
        }
        Ok(())
    })?;
    let drop = format!("{descriptor}.drop");
    crate::glue::define_glue(m, &drop, &["ptr"], "", |cx| {
        let (env_at, _) = find(cx, "%p0");
        cx.drop_at(&env_at, env_align.min(if boxed { env_align } else { 8 }), env)?;
        if boxed {
            cx.call_rt("free", vec![Val::Imm(Sc::Ptr, env_at), Val::Imm(Sc::I(64), env_size.to_string()), Val::Imm(Sc::I(64), env_align.to_string())], None);
        }
        Ok(())
    })?;
    let once = format!("{descriptor}.once");
    let takes_whole = body.param_modes.first() == Some(&ParameterMode::Owned);
    let closure_owned = closure.to_string();
    let (native_name, drop_name) = (global_name(&native), global_name(&drop));
    let native_sig = crate::func::c_fn_signature(m, Some(ret), &params);
    define_c_fn(m, &once, "internal ", Some(ret), &params, "", |cx, values, result| {
        let dest = result.as_ref().map(|(ptr, align)| (ptr.as_str(), *align));
        let state = state_of(values);
        if takes_whole {
            let (env_at, _) = find(cx, &state);
            let env_value = env_val(cx, &env_at, env);
            let mut arguments = vec![env_value];
            arguments.extend(values[1..].iter().cloned());
            cx.call_body(&closure_owned, arguments, dest)?;
            if boxed {
                cx.call_rt("free", vec![Val::Imm(Sc::Ptr, env_at), Val::Imm(Sc::I(64), env_size.to_string()), Val::Imm(Sc::I(64), env_align.to_string())], None);
            }
        } else {
            let sized: Vec<(Val, u64, u64)> = values
                .iter()
                .cloned()
                .zip(&params)
                .map(|(value, param)| match param {
                    Param::Ty(ty) => {
                        let (size, align) = cx.m.ty.size_align(*ty);
                        (value, size, align)
                    }
                    _ => (value, 8, 8),
                })
                .collect();
            let size = cx.m.ty.size_align(ret).0;
            cx.lowered_call(&native_name, &native_sig, &sized, dest.map(|(ptr, align)| (ptr, align, size)), "");
            cx.emit(format!("call void {drop_name}(ptr {state})"));
        }
        Ok(())
    })?;
    // `[HR-15]` — in a reloadable image, what the captured state holds, for a reload to follow.
    let captures = if m.reload.is_some() { format!("ptr {}, ", crate::reload::captures(m, env)) } else { String::new() };
    m.global(format!(
        "{} = internal constant {} {{ ptr {native_name}, ptr null, ptr {drop_name}, {captures}ptr {} }}, align 8",
        global_name(&descriptor),
        descriptor_type(m),
        global_name(&once)
    ));
    Ok(global_name(&descriptor))
}

/// The environment at `at` as the value a closure taking it whole receives.
fn env_val(cx: &mut FnCx<'_, '_>, at: &str, env: Ty) -> Val {
    let (_, align) = cx.m.ty.size_align(env);
    match cx.m.ty.repr(env) {
        crate::types::Repr::Scalar(sc) => Val::Imm(sc, cx.load(sc, at, align.min(8))),
        crate::types::Repr::Zst => Val::Zst,
        crate::types::Repr::Mem { .. } => Val::Mem(at.to_string(), align.min(8)),
    }
}

impl<'a> FnCx<'_, 'a> {
    /// `[FN-6]` — a named function as a native value: `{native, &descriptor, 0, 0, 0}`.
    pub(crate) fn native_fn_value(&mut self, symbol: &str, signature: Ty) -> Result<Val> {
        let descriptor = native_descriptor(self.m, symbol, signature)?;
        let slot = self.alloca(40, 8);
        self.zero(&slot, 8, 40);
        let native = self.load(Sc::Ptr, &descriptor, 8);
        self.emit(format!("store ptr {native}, ptr {slot}, align 8"));
        let descriptor_at = self.gep(&slot, 8);
        self.emit(format!("store ptr {descriptor}, ptr {descriptor_at}, align 8"));
        Ok(Val::Mem(slot, 8))
    }

    /// `[CLO-10]` — a closure made into an owned value: its environment moved into the words,
    /// or into a block of its own when it does not fit them (`fn_value_make`).
    pub(crate) fn make_callable(&mut self, operands: &[Operand], to: &str, to_align: u64) -> Result<()> {
        let [Operand::Const(Const::Fn(closure)), env] = operands else {
            return unsupported("an owned callable value not made of a closure and its environment");
        };
        let descriptor = closure_descriptor(self.m, closure)?;
        let env_ty = self.m.callables.closures.get(closure.as_str()).copied().expect("collected");
        let (size, align) = self.m.ty.size_align(env_ty);
        let value = self.operand(env)?;
        let (source, source_align) = self.spill(&value, size, align);
        self.zero(to, to_align, 40);
        let native = self.load(Sc::Ptr, &descriptor, 8);
        self.emit(format!("store ptr {native}, ptr {to}, align {}", to_align.min(8)));
        let descriptor_at = self.gep(to, 8);
        self.emit(format!("store ptr {descriptor}, ptr {descriptor_at}, align {}", to_align.min(8)));
        let words = self.gep(to, 16);
        if size <= 24 {
            self.copy(&words, to_align.min(8), &source, source_align, size, false);
        } else {
            let block = self.alloca(8, 8);
            self.call_rt(
                "box_new_copy",
                vec![Val::Imm(Sc::I(64), size.to_string()), Val::Imm(Sc::I(64), align.to_string()), Val::Imm(Sc::Ptr, source)],
                Some((&block, 8)),
            );
            let block = self.load(Sc::Ptr, &block, 8);
            self.emit(format!("store ptr {block}, ptr {words}, align 8"));
        }
        Ok(())
    }

    /// `[CLO-3]`, `[FN-6b]` — a call through a value of function type: a native value's `native`
    /// (or `once`) entry with its state first; a C function pointer as C calls one, a borrowed
    /// aggregate passed by value.
    pub(crate) fn indirect_call(&mut self, callee: &Operand, ty: Ty, args: &[Operand], dest: Option<(String, u64)>) -> Result<()> {
        let TyKind::Fn { abi, params, ret, once, .. } = self.m.ty.kind(ty).clone() else {
            return unsupported("a call through a value that is not a function");
        };
        let mut values = Vec::with_capacity(args.len() + 1);
        let dest = dest.as_ref().map(|(ptr, align)| (ptr.as_str(), *align));
        if abi.is_some() {
            let (function, _) = self.scalar(callee)?;
            let mut c_params = Vec::with_capacity(params.len());
            for (argument, param) in args.iter().zip(&params) {
                let value = self.operand(argument)?;
                // A borrowed aggregate passed by address in Ember is passed by value to C.
                if param.mode == FnParamMode::Borrow && self.m.ty.table.passed_by_address(param.ty) {
                    let Val::Imm(_, pointer) = value else { return unsupported("a borrowed argument held in memory") };
                    let (_, align) = self.m.ty.size_align(param.ty);
                    values.push(Val::Mem(pointer, align));
                } else {
                    values.push(value);
                }
                c_params.push(Param::Ty(param.ty));
            }
            self.call_c(&function, Some(ret), &c_params, values, dest);
            return Ok(());
        }
        // D-880 — a call that consumes its callee (moves it in: an owned callable value given to a
        // `once fn` parameter, `[CLO-6]`) ends it through the `once` entry, whatever its type says,
        // as nothing else ends it.
        let once = once || matches!(callee, Operand::Move(_));
        let callee_value = self.operand(callee)?;
        let (mut value_at, _) = self.spill(&callee_value, 40, 8);
        // `[CLO-6a]` — a `once fn` value in a `Box` is taken out of the box, which is freed, first.
        if once && matches!(self.operand_ty(callee).map(|ty| self.m.ty.kind(ty).clone()), Some(TyKind::Struct(_))) {
            let boxed = self.load(Sc::Ptr, &value_at, 8);
            let taken = self.alloca(40, 8);
            self.copy(&taken, 8, &boxed, 8, 40, false);
            self.call_rt(
                "free",
                vec![Val::Imm(Sc::Ptr, boxed), Val::Imm(Sc::I(64), "40".to_string()), Val::Imm(Sc::I(64), "8".to_string())],
                None,
            );
            value_at = taken;
        }
        let entry = if once {
            let descriptor_at = self.gep(&value_at, 8);
            let descriptor = self.load(Sc::Ptr, &descriptor_at, 8);
            let once_at = self.gep(&descriptor, once_offset(self.m));
            self.load(Sc::Ptr, &once_at, 8)
        } else {
            self.load(Sc::Ptr, &value_at, 8)
        };
        let state = self.gep(&value_at, 16);
        values.push(Val::Imm(Sc::Ptr, state));
        let mut signature = vec![Param::Ptr];
        for (argument, param) in args.iter().zip(&params) {
            values.push(self.operand(argument)?);
            signature.push(param_of(self.m, *param, false));
        }
        self.call_c(&entry, Some(ret), &signature, values, dest);
        Ok(())
    }

    /// ODR-102 — a native value's C entry, which one holding captured state lacks: giving that to
    /// C panics.
    pub(crate) fn fn_to_c(&mut self, operand: &Operand, span: ember_span::Span) -> Result<Val> {
        let value = self.operand(operand)?;
        let (value_at, _) = self.spill(&value, 40, 8);
        let descriptor_at = self.gep(&value_at, 8);
        let descriptor = self.load(Sc::Ptr, &descriptor_at, 8);
        let c_at = self.gep(&descriptor, 8);
        let entry = self.load(Sc::Ptr, &c_at, 8);
        let missing = self.value(format!("icmp eq ptr {entry}, null"));
        let (fail, ok) = (self.label(), self.label());
        self.emit(format!("br i1 {missing}, label %{fail}, label %{ok}"));
        self.start(&fail);
        let message = b"a callable value holding captured state has no C entry";
        let text = self.m.bytes(message);
        let loc = self.loc(span);
        self.call_rt("panic", vec![Val::Imm(Sc::Ptr, text), Val::Imm(Sc::I(64), message.len().to_string()), loc], None);
        self.start(&ok);
        Ok(Val::Imm(Sc::Ptr, entry))
    }
}
