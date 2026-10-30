//! `[FFI-10]`, `[FFI-11]`, `[FFI-11b]`, `[FFI-16]`, `[FFI-21]`, `[FFI-23]`, `[FFI-36a]` — a
//! foreign function's counted wrapper: the Ember function a contracted declaration
//! (`@ffi(param(…), result(…))`) makes, which maps each Ember argument to the C parameters its
//! contract names (a view to its element pointer and length, an `out` value to a local's address,
//! a callable to a trampoline and its user data), calls the C function by the platform's C ABI,
//! and makes C's result and outputs the Ember result. It is the C backend's
//! `emit_ffi_counted_wrapper`, written as IR; each check panics with the C backend's message.

use crate::abi::{self, Scalar, Shape};
use crate::func::{FnCx, Param, PlaceRef, Val, define_c_fn};
use crate::types::Sc;
use crate::{Module, Result, global_name, link_symbol, unsupported};
use ember_hir::{ExportThreadPolicy, FfiAbiParam, FfiCallback, FfiCallbackMode, FfiCounted, FfiUnregisterArg, Mode};
use ember_mir::{Body, LocalId, Operand, ParameterMode, Place};
use ember_types::{IntTy, Ty, TyKind, UintTy};

const PTR: Shape = Shape::Scalar(Scalar::Ptr);

/// A C struct's field offsets, size and alignment for fields of these sizes and alignments.
fn c_struct(fields: &[(u64, u64)]) -> (Vec<u64>, u64, u64) {
    let mut offsets = Vec::with_capacity(fields.len());
    let (mut end, mut align) = (0u64, 1u64);
    for &(size, field_align) in fields {
        let field_align = field_align.max(1);
        end = end.div_ceil(field_align) * field_align;
        offsets.push(end);
        end += size;
        align = align.max(field_align);
    }
    (offsets, end.div_ceil(align).max(1) * align, align)
}

/// `[FFI-21]` — where callback `index` of a wrapper keeps what its trampoline needs: the callable
/// itself (`borrowed`, by address), or a record holding it (by address for `borrowed`, moved in
/// for `once` and `retained`), the creator thread under `threads(creator)`, and for `retained`
/// each scalar argument `destroys(f)` is given, and C's result when `destroys(f)` is given it
/// (ODR-246). `None` where the callable's address is all of it.
struct Record {
    size: u64,
    align: u64,
    creator: Option<u64>,
    /// The saved arguments: the wrapper's parameter, its type and its offset.
    saved: Vec<(usize, Ty, u64)>,
    /// `[FFI-11]` (ODR-246) — where C's result is kept, written once the registering call returns.
    result: Option<u64>,
}

/// `[FFI-11]` (ODR-246) — the type of the registering call's result, when `destroys(f)` is given it.
fn kept_result(callback: &FfiCallback) -> Option<Ty> {
    callback.unregister.iter().flat_map(|unregister| &unregister.args).find_map(|arg| match arg {
        FfiUnregisterArg::Result { ty } => Some(*ty),
        _ => None,
    })
}

fn record(m: &Module<'_>, callback: &FfiCallback) -> Option<Record> {
    let creator = callback.threads == ExportThreadPolicy::Creator;
    if callback.mode == FfiCallbackMode::Borrowed && !creator {
        return None;
    }
    let mut fields = vec![if callback.mode == FfiCallbackMode::Borrowed { (8, 8) } else { m.ty.size_align(callback.callable) }];
    if creator {
        fields.push((4, 4));
    }
    let saved: Vec<(usize, Ty)> = callback
        .unregister
        .iter()
        .flat_map(|unregister| &unregister.args)
        .filter_map(|arg| match arg {
            FfiUnregisterArg::Saved { public_index, ty } => Some((*public_index, *ty)),
            _ => None,
        })
        .collect();
    fields.extend(saved.iter().map(|(_, ty)| m.ty.size_align(*ty)));
    let result = kept_result(callback);
    fields.extend(result.map(|ty| m.ty.size_align(ty)));
    let (offsets, size, align) = c_struct(&fields);
    let creator_at = creator.then(|| offsets[1]);
    let first_saved = 1 + usize::from(creator);
    let result = result.map(|_| offsets[first_saved + saved.len()]);
    let saved = saved.iter().enumerate().map(|(at, (index, ty))| (*index, *ty, offsets[first_saved + at])).collect();
    Some(Record { size, align, creator: creator_at, saved, result })
}

/// `[FFI-21]` — what the wrapper passes C for one callback: the user data, for a `retained` one
/// the token the call returns (its slot and type), and for a `nullable` one whether the caller
/// passed `Some` (ODR-253).
struct Passed {
    data: String,
    token: Option<(String, Ty)>,
    some: Option<String>,
}

fn entry_symbol(body: &Body, index: usize) -> String {
    format!("{}_cb{index}", body.symbol)
}

/// `[FFI-36a]` — the `void (T*)` a `ForeignBox` holds for `destructor`: the function itself, or
/// for one with a result a thunk that calls it and drops the result.
pub(crate) fn foreign_destructor(m: &mut Module<'_>, destructor: &str, ret: Ty) -> Result<String> {
    let returns = !m.ty.is_zst(ret);
    declare_c(m, destructor, returns.then_some(ret), &[Param::Ptr]);
    if !returns {
        return Ok(global_name(destructor));
    }
    let thunk = format!("{}_ffi_destroy_{destructor}", ember_branding::SYMBOL_PREFIX);
    if !m.helper_defined(&thunk) {
        let callee = global_name(destructor);
        define_c_fn(m, &thunk, "internal ", None, &[Param::Ptr], "", |cx, values, _| {
            let (size, align) = cx.m.ty.size_align(ret);
            let scratch = cx.alloca(size, align);
            cx.call_c(&callee, Some(ret), &[Param::Ptr], values.to_vec(), Some((&scratch, align)));
            Ok(())
        })?;
    }
    Ok(global_name(&thunk))
}

/// A foreign C function's declaration, by the linker's symbol.
pub(crate) fn declare_c(m: &mut Module<'_>, symbol: &str, ret: Option<Ty>, params: &[Param]) {
    let sig = crate::func::c_fn_signature(m, ret, params);
    declare_sig(m, symbol, &sig);
}

fn declare_sig(m: &mut Module<'_>, symbol: &str, sig: &abi::Signature) {
    declare_named(m, symbol, &global_name(symbol), sig);
}

/// The declaration of the foreign function `symbol`, under its IR name `name`.
fn declare_named(m: &mut Module<'_>, symbol: &str, name: &str, sig: &abi::Signature) {
    m.declare(&link_symbol(symbol), || {
        format!("declare {}{} {name}({})", sig.conv.ir(), crate::func::ret_with_attr(sig), crate::func::ir_params(sig, false).join(", "))
    });
}

/// The wrapper `body` and the trampolines its callbacks use.
pub(crate) fn emit_counted_wrapper<'a>(m: &mut Module<'a>, body: &'a Body) -> Result<()> {
    let counted = body.ffi_counted.as_ref().expect("a counted wrapper's contract");
    for (index, callback) in counted.callbacks.iter().enumerate() {
        trampoline(m, body, index, callback)?;
    }
    crate::func::define_body(m, body, |cx| {
        cx.start("bb0");
        cx.counted_wrapper(body, counted)?;
        cx.ret()
    })
}

/// `[FFI-21]` — the trampoline C calls for callback `index`, and for a `retained` one the release
/// its token's end calls. The trampoline attaches the thread (`[FFI-22]`), checks in `debug` that
/// C calls it on the thread its contract names (`[FFI-33b]`), and calls the callable at the user
/// data: a lambda's body directly with its environment, or a callable value through its entry.
/// Under `once` it then ends the callable and frees what held it. The release unregisters the
/// callback through `destroys(f)`, then ends the callable and frees the record.
fn trampoline<'a>(m: &mut Module<'a>, body: &'a Body, index: usize, callback: &'a FfiCallback) -> Result<()> {
    let entry = entry_symbol(body, index);
    let TyKind::Fn { params, ret, .. } = m.ty.kind(callback.c_fn).clone() else {
        return unsupported("a callback that is not a C function pointer");
    };
    let record = record(m, callback);
    let c_params: Vec<Param> = params.iter().map(|param| crate::closures::param_of(m, *param, true)).collect();
    let debug = m.profile == crate::Profile::Debug;
    let closure = callback.closure.as_deref().and_then(|symbol| m.bodies.get(symbol).copied());
    let (_, callable_align) = m.ty.size_align(callback.callable);
    define_c_fn(m, &entry, "internal ", Some(ret), &c_params, "", |cx, values, result| {
        cx.call_rt("rt_thread_attach", Vec::new(), None);
        let data = match &values[callback.slot] {
            Val::Imm(Sc::Ptr, data) => data.clone(),
            Val::Imm(Sc::I(bits), data) => cx.value(format!("inttoptr i{bits} {data} to ptr")),
            _ => return unsupported("a callback's user data that is not a pointer or an integer"),
        };
        // Where the callable is, and the thread the contract names.
        let env = match (&record, callback.mode) {
            (Some(_), FfiCallbackMode::Borrowed) => cx.load(Sc::Ptr, &data, 8),
            _ => data.clone(),
        };
        if debug {
            let thread = match (&record, callback.threads) {
                (Some(Record { creator: Some(at), .. }), _) => {
                    let at = cx.gep(&data, *at);
                    Some(cx.load(Sc::I(32), &at, 4))
                }
                (_, ExportThreadPolicy::Main) => Some(u32::MAX.to_string()),
                _ => None,
            };
            if let Some(thread) = thread {
                let what = cx.m.bytes(b"a callback");
                cx.call_rt("ffi_check_thread", vec![Val::Imm(Sc::I(32), thread), Val::Imm(Sc::Ptr, what)], None);
            }
        }
        let others: Vec<(usize, ember_types::FnParam)> = params.iter().copied().enumerate().filter(|(at, _)| *at != callback.slot).collect();
        let dest = result.as_ref().map(|(ptr, align)| (ptr.as_str(), *align));
        // `[HR-42]` — C calling back into Ember code: this thread's depth counter.
        let reload = cx.m.reload.is_some();
        if reload {
            cx.call_rt("reload_enter", Vec::new(), None);
        }
        let consumed = match closure {
            Some(closure) => {
                // ODR-278 — a function's own type holds no state: its function takes none.
                let item = matches!(cx.m.ty.kind(callback.callable), TyKind::FnItem { .. });
                let mut closure_args = closure.args();
                let mut arguments = Vec::new();
                if !item {
                    let (_, env_param) = closure_args.next().expect("a closure takes its environment first");
                    let by_address = matches!(cx.m.ty.kind(env_param.ty), TyKind::Ref { .. } | TyKind::Ptr { .. });
                    arguments.push(if by_address { Val::Imm(Sc::Ptr, env.clone()) } else { cx.value_in(&env, callback.callable) });
                }
                let locals: Vec<Ty> = closure_args.map(|(_, decl)| decl.ty).collect();
                for ((at, param), local) in others.iter().zip(&locals) {
                    let value = values[*at].clone();
                    if param.mode == ember_types::FnParamMode::Borrow && matches!(cx.m.ty.kind(*local), TyKind::Ref { .. } | TyKind::Ptr { .. }) {
                        let (size, align) = cx.m.ty.size_align(param.ty);
                        let (address, _) = cx.spill(&value, size, align);
                        arguments.push(Val::Imm(Sc::Ptr, address));
                    } else {
                        arguments.push(value);
                    }
                }
                cx.call_body(&closure.symbol, arguments, dest)?;
                // A lambda that takes its environment over ends it itself.
                !item && closure.param_modes.first() == Some(&ParameterMode::Owned)
            }
            None => {
                let native = cx.load(Sc::Ptr, &env, 8);
                let state = cx.gep(&env, 16);
                let mut signature = vec![Param::Ptr];
                let mut arguments = vec![Val::Imm(Sc::Ptr, state)];
                for (at, param) in &others {
                    let value = values[*at].clone();
                    if param.mode == ember_types::FnParamMode::Borrow && cx.m.ty.table.passed_by_address(param.ty) {
                        let (size, align) = cx.m.ty.size_align(param.ty);
                        let (address, _) = cx.spill(&value, size, align);
                        arguments.push(Val::Imm(Sc::Ptr, address));
                    } else {
                        arguments.push(value);
                    }
                    signature.push(crate::closures::param_of(cx.m, *param, false));
                }
                cx.call_c(&native, Some(ret), &signature, arguments, dest);
                false
            }
        };
        if reload {
            cx.call_rt("reload_leave", Vec::new(), None);
        }
        if callback.mode == FfiCallbackMode::Once {
            // Its one call is over: the callable ends (unless the call took it), and so does what
            // held it.
            if !consumed {
                cx.drop_at(&env, callable_align.min(16), callback.callable)?;
            }
            if let Some(record) = &record {
                cx.call_rt(
                    "free",
                    vec![Val::Imm(Sc::Ptr, data), Val::Imm(Sc::I(64), record.size.to_string()), Val::Imm(Sc::I(64), record.align.to_string())],
                    None,
                );
            }
        }
        Ok(())
    })?;
    let Some(unregister) = &callback.unregister else { return Ok(()) };
    let Some(record) = record else { return unsupported("a retained callback kept in no record") };
    let release = format!("{entry}_release");
    let entry_name = global_name(&entry);
    let params: Vec<Param> = unregister.params.iter().map(|ty| Param::Ty(*ty)).collect();
    let returns = !m.ty.is_zst(unregister.ret);
    declare_c(m, &unregister.symbol, returns.then_some(unregister.ret), &params);
    let callee = global_name(&unregister.symbol);
    define_c_fn(m, &release, "internal ", None, &[Param::Ptr], "", |cx, values, _| {
        let Val::Imm(_, data) = values[0].clone() else { unreachable!("a pointer") };
        if debug && let Some(at) = record.creator {
            let at = cx.gep(&data, at);
            let thread = cx.load(Sc::I(32), &at, 4);
            let what = cx.m.bytes(b"a callback");
            cx.call_rt("ffi_check_thread", vec![Val::Imm(Sc::I(32), thread), Val::Imm(Sc::Ptr, what)], None);
        }
        let mut arguments = Vec::with_capacity(unregister.args.len());
        for (arg, ty) in unregister.args.iter().zip(&unregister.params) {
            arguments.push(match arg {
                FfiUnregisterArg::Entry => Val::Imm(Sc::Ptr, entry_name.clone()),
                FfiUnregisterArg::Data => cx.pointer_as(&data, *ty),
                FfiUnregisterArg::Saved { public_index, ty } => {
                    let at = record.saved.iter().find(|(index, ..)| index == public_index).map_or(0, |(_, _, at)| *at);
                    let at = cx.gep(&data, at);
                    cx.value_in(&at, *ty)
                }
                FfiUnregisterArg::Result { ty } => {
                    let at = cx.gep(&data, record.result.expect("a kept result has its field"));
                    cx.value_in(&at, *ty)
                }
            });
        }
        let scratch = returns.then(|| {
            let (size, align) = cx.m.ty.size_align(unregister.ret);
            (cx.alloca(size, align), align)
        });
        cx.call_c(&callee, returns.then_some(unregister.ret), &params, arguments, scratch.as_ref().map(|(ptr, align)| (ptr.as_str(), *align)));
        cx.drop_at(&data, record.align.min(16), callback.callable)?;
        cx.call_rt(
            "free",
            vec![Val::Imm(Sc::Ptr, data), Val::Imm(Sc::I(64), record.size.to_string()), Val::Imm(Sc::I(64), record.align.to_string())],
            None,
        );
        Ok(())
    })?;
    Ok(())
}

/// `[FFI-11]` — the largest count a C count type holds, where it is below `size_t`'s.
fn count_limit(ty: &TyKind) -> Option<u64> {
    Some(match ty {
        TyKind::Int(IntTy::I8) => i8::MAX as u64,
        TyKind::Int(IntTy::I16) => i16::MAX as u64,
        TyKind::Int(IntTy::I32) => i32::MAX as u64,
        TyKind::Int(IntTy::I64 | IntTy::Isize) => i64::MAX as u64,
        TyKind::Uint(UintTy::U8) => u8::MAX as u64,
        TyKind::Uint(UintTy::U16) => u16::MAX as u64,
        TyKind::Uint(UintTy::U32) => u32::MAX as u64,
        _ => return None,
    })
}

impl<'a> FnCx<'_, 'a> {
    /// The value of type `ty` at `at`.
    pub(crate) fn value_in(&mut self, at: &str, ty: Ty) -> Val {
        let (_, align) = self.m.ty.size_align(ty);
        match self.m.ty.repr(ty) {
            crate::types::Repr::Zst => Val::Zst,
            crate::types::Repr::Scalar(sc) => Val::Imm(sc, self.load(sc, at, align)),
            crate::types::Repr::Mem { .. } => Val::Mem(at.to_string(), align),
        }
    }

    /// A pointer as a value of `ty`: itself, or its address as an integer.
    fn pointer_as(&mut self, pointer: &str, ty: Ty) -> Val {
        match self.m.ty.repr(ty) {
            crate::types::Repr::Scalar(Sc::I(bits)) => Val::Imm(Sc::I(bits), self.value(format!("ptrtoint ptr {pointer} to i{bits}"))),
            _ => Val::Imm(Sc::Ptr, pointer.to_string()),
        }
    }

    /// Wrapper parameter `index`'s place.
    fn param_place(&mut self, index: usize) -> Result<PlaceRef> {
        self.place(&Place::local(LocalId(index as u32 + 1)))
    }

    fn param_value(&mut self, index: usize) -> Result<Val> {
        self.operand(&Operand::Copy(Place::local(LocalId(index as u32 + 1))))
    }

    /// A view parameter's words: its pointer and its length.
    fn view_param(&mut self, index: usize) -> Result<(String, String)> {
        let at = self.param_place(index)?;
        let pointer = self.load(Sc::Ptr, &at.ptr, at.align.min(8));
        let len_at = self.gep(&at.ptr, 8);
        let len = self.load(Sc::I(64), &len_at, at.align.min(8));
        Ok((pointer, len))
    }

    /// `[FFI-11]` — a nullable view's `None`: a null pointer with the length `SIZE_MAX`.
    fn view_none(&mut self, pointer: &str, len: &str) -> String {
        let null = self.value(format!("icmp eq ptr {pointer}, null"));
        let max = self.value(format!("icmp eq i64 {len}, -1"));
        self.value(format!("and i1 {null}, {max}"))
    }

    /// A view parameter's length, `0` for a nullable one's `None`.
    fn view_length(&mut self, index: usize, nullable: bool) -> Result<String> {
        let (pointer, len) = self.view_param(index)?;
        if !nullable {
            return Ok(len);
        }
        let none = self.view_none(&pointer, &len);
        Ok(self.value(format!("select i1 {none}, i64 0, i64 {len}")))
    }

    /// A panic with `message` at the wrapper's declaration when `fails`.
    fn panic_if(&mut self, fails: &str, message: &[u8], span: ember_span::Span) {
        let (fail, ok) = (self.label(), self.label());
        self.emit(format!("br i1 {fails}, label %{fail}, label %{ok}"));
        self.start(&fail);
        let text = self.m.bytes(message);
        let loc = self.loc(span);
        self.call_rt("panic", vec![Val::Imm(Sc::Ptr, text), Val::Imm(Sc::I(64), message.len().to_string()), loc], None);
        self.start(&ok);
    }

    /// `[FFI-21]` — before the call: where callback `index`'s trampoline finds its callable (the
    /// user data), for a `retained` one the token the call returns, and for a `nullable` one
    /// whether the caller passed `Some`: `None` gives C null user data and registers nothing.
    fn callback_data(&mut self, body: &Body, index: usize, callback: &FfiCallback) -> Result<Passed> {
        let table = self.m.ty.table;
        let local = body.args().nth(callback.public_index).expect("the callback's parameter").1.ty;
        let at = self.param_place(callback.public_index)?;
        let by_address = matches!(self.m.ty.kind(local), TyKind::Ref { .. } | TyKind::Ptr { .. });
        let mut address = if by_address { self.load(Sc::Ptr, &at.ptr, at.align.min(8)) } else { at.ptr.clone() };
        // `[FFI-21]` (ODR-253) — a `nullable` one: whether the caller passed `Some`, and its callable.
        let some = match callback.nullable {
            None => None,
            Some(option) => {
                let TyKind::Enum(id) = *table.kind(option) else { return unsupported("a nullable callback that is not an `Option`") };
                let def = table.enum_def(id);
                let some_at = def.variants.iter().position(|variant| variant.fields.len() == 1).expect("`Option` has `Some`");
                let discriminant = def.variants[some_at].discriminant;
                let (_, align) = self.m.ty.size_align(option);
                let (tag, bits) = self.discriminant_at(&PlaceRef { ptr: address.clone(), ty: option, variant: None, align })?;
                let some = self.value(format!("icmp eq i{bits} {tag}, {}", crate::func::int_literal(discriminant as u128, bits)));
                address = self.gep(&address, table.variant_field_offsets(id)[some_at][0]);
                Some(some)
            }
        };
        let given = |cx: &mut Self, data: String| match &some {
            Some(some) => cx.value(format!("select i1 {some}, ptr {data}, ptr null")),
            None => data,
        };
        let data = match record(self.m, callback) {
            None => given(self, address),
            Some(record) if callback.mode == FfiCallbackMode::Borrowed => {
                let slot = self.alloca(record.size, record.align);
                self.emit(format!("store ptr {address}, ptr {slot}, align 8"));
                let creator_at = self.gep(&slot, record.creator.expect("a borrowed record names its creator"));
                self.call_rt("ffi_thread", Vec::new(), Some((&creator_at, 4)));
                given(self, slot)
            }
            Some(record) => {
                // `None` moves nothing and allocates nothing: the user data stays null.
                let block = self.alloca(8, 8);
                let done = some.as_ref().map(|some| {
                    let (moved, done) = (self.label(), self.label());
                    self.emit(format!("store ptr null, ptr {block}, align 8"));
                    self.emit(format!("br i1 {some}, label %{moved}, label %{done}"));
                    self.start(&moved);
                    done
                });
                // The callable moves into an allocation of its own: C keeps it past the call.
                let slot = self.alloca(record.size, record.align);
                self.zero(&slot, record.align, record.size);
                let (size, align) = self.m.ty.size_align(callback.callable);
                self.copy(&slot, record.align.min(align), &address, align, size, false);
                if let Some(creator) = record.creator {
                    let creator_at = self.gep(&slot, creator);
                    self.call_rt("ffi_thread", Vec::new(), Some((&creator_at, 4)));
                }
                for (public_index, ty, offset) in &record.saved {
                    let value = self.param_value(*public_index)?;
                    let field = self.gep(&slot, *offset);
                    let (size, align) = self.m.ty.size_align(*ty);
                    let (from, from_align) = self.spill(&value, size, align);
                    self.copy(&field, align, &from, from_align, size, false);
                }
                self.call_rt(
                    "box_new_copy",
                    vec![Val::Imm(Sc::I(64), record.size.to_string()), Val::Imm(Sc::I(64), record.align.to_string()), Val::Imm(Sc::Ptr, slot)],
                    Some((&block, 8)),
                );
                if let Some(done) = done {
                    self.emit(format!("br label %{done}"));
                    self.start(&done);
                }
                self.load(Sc::Ptr, &block, 8)
            }
        };
        let Some(unregister) = &callback.unregister else { return Ok(Passed { data, token: None, some }) };
        // The token: `{ {data, release}, 0 }` (`[HR-20]` — no `on_relocate` record).
        let TyKind::Struct(token) = *table.kind(unregister.token) else { return unsupported("a `Retained` token that is not a struct") };
        let held_ty = table.struct_def(token).fields[0].ty;
        let held_at = table.layout(unregister.token).field_offsets[0];
        let words = table.layout(held_ty).field_offsets;
        let (size, align) = self.m.ty.size_align(unregister.token);
        let slot = self.alloca(size, align);
        self.zero(&slot, align, size);
        let data_at = self.gep(&slot, held_at + words[0]);
        self.emit(format!("store ptr {data}, ptr {data_at}, align 8"));
        let release_at = self.gep(&slot, held_at + words[1]);
        let release = global_name(&format!("{}_release", entry_symbol(body, index)));
        self.emit(format!("store ptr {release}, ptr {release_at}, align 8"));
        let Some(some) = some else { return Ok(Passed { data, token: Some((slot, unregister.token)), some: None }) };
        // `None` registered nothing, so the call returns `None` for its token.
        let TyKind::Enum(id) = *table.kind(unregister.output) else { return unsupported("a nullable callback's token that is not an `Option`") };
        let def = table.enum_def(id);
        let some_at = def.variants.iter().position(|variant| variant.fields.len() == 1).expect("`Option` has `Some`");
        let none_at = def.variants.iter().position(|variant| variant.fields.is_empty()).expect("`Option` has `None`");
        let (output_size, output_align) = self.m.ty.size_align(unregister.output);
        let output = self.alloca(output_size, output_align);
        self.zero(&output, output_align, output_size);
        let place = PlaceRef { ptr: output.clone(), ty: unregister.output, variant: None, align: output_align };
        let (made, none, done) = (self.label(), self.label(), self.label());
        self.emit(format!("br i1 {some}, label %{made}, label %{none}"));
        self.start(&made);
        let payload = self.gep(&output, table.variant_field_offsets(id)[some_at][0]);
        self.copy(&payload, align, &slot, align, size, false);
        self.write_variant(&place, id, some_at)?;
        self.emit(format!("br label %{done}"));
        self.start(&none);
        self.write_variant(&place, id, none_at)?;
        self.emit(format!("br label %{done}"));
        self.start(&done);
        Ok(Passed { data, token: Some((output, unregister.output)), some: Some(some) })
    }

    /// The wrapper's code: its arguments mapped, the call, its result.
    fn counted_wrapper(&mut self, body: &Body, counted: &FfiCounted) -> Result<()> {
        let table = self.m.ty.table;
        let span = body.span;
        let mut callbacks = Vec::with_capacity(counted.callbacks.len());
        for (index, callback) in counted.callbacks.iter().enumerate() {
            callbacks.push(self.callback_data(body, index, callback)?);
        }
        let mut shapes = Vec::with_capacity(counted.abi_params.len());
        let mut values = Vec::with_capacity(counted.abi_params.len());
        // The outputs, in C's order: each one's slot and type.
        let mut outs: Vec<(String, Ty)> = Vec::new();
        for param in &counted.abi_params {
            match param {
                // `[FFI-21]` — the trampoline; a `retained` callback's token is an output in its place.
                FfiAbiParam::Callback { index } => {
                    shapes.push(PTR);
                    let entry = global_name(&entry_symbol(body, *index));
                    // `[FFI-21]` (ODR-253) — `None` gives C a null pointer.
                    let entry = match &callbacks[*index].some {
                        Some(some) => self.value(format!("select i1 {some}, ptr {entry}, ptr null")),
                        None => entry,
                    };
                    values.push(Val::Imm(Sc::Ptr, entry));
                    if let Some(token) = &callbacks[*index].token {
                        outs.push(token.clone());
                    }
                }
                FfiAbiParam::CallbackData { index } => {
                    let data_ty = counted.callbacks[*index].data;
                    let data = callbacks[*index].data.clone();
                    shapes.push(self.m.ty.shape(data_ty));
                    values.push(self.pointer_as(&data, data_ty));
                }
                // `[FFI-11]` — C writes the value here; the call returns it.
                FfiAbiParam::Out { ty } => {
                    let (size, align) = self.m.ty.size_align(*ty);
                    let slot = self.alloca(size, align);
                    self.zero(&slot, align, size);
                    shapes.push(PTR);
                    values.push(Val::Imm(Sc::Ptr, slot.clone()));
                    outs.push((slot, *ty));
                }
                FfiAbiParam::Value { public_index, ty, mode } => {
                    let value = self.param_value(*public_index)?;
                    match mode {
                        Mode::Mut => {
                            shapes.push(PTR);
                            values.push(value);
                        }
                        // A borrowed value Ember passes by address is C's by value.
                        Mode::Borrow if table.passed_by_address(*ty) => {
                            let Val::Imm(_, pointer) = value else { return unsupported("a borrowed foreign argument held in memory") };
                            let (_, align) = self.m.ty.size_align(*ty);
                            shapes.push(self.m.ty.shape(*ty));
                            values.push(Val::Mem(pointer, align));
                        }
                        _ => {
                            shapes.push(self.m.ty.shape(*ty));
                            values.push(value);
                        }
                    }
                }
                FfiAbiParam::SpanPointer { public_index, elem, nullable, .. } => {
                    let (pointer, len) = self.view_param(*public_index)?;
                    let pointer = if *nullable {
                        // An empty view still gives C a pointer to an element; `None` gives null.
                        let (size, align) = self.m.ty.size_align(*elem);
                        let symbol = format!("ember.ffi_empty.{}", self.m.ty.table.mangled(*elem));
                        let empty = global_name(&symbol);
                        if !self.m.helper_defined(&symbol) {
                            self.m.global(format!("{empty} = internal global [{} x i8] zeroinitializer, align {}", size.max(1), align.max(1)));
                        }
                        let none = self.view_none(&pointer, &len);
                        let null = self.value(format!("icmp eq ptr {pointer}, null"));
                        let some = self.value(format!("select i1 {null}, ptr {empty}, ptr {pointer}"));
                        self.value(format!("select i1 {none}, ptr null, ptr {some}"))
                    } else {
                        pointer
                    };
                    shapes.push(PTR);
                    values.push(Val::Imm(Sc::Ptr, pointer));
                }
                // The fixed array's address (null for a nullable one's `None`).
                FfiAbiParam::ArrayPointer { public_index, .. } => {
                    let at = self.param_place(*public_index)?;
                    let pointer = self.load(Sc::Ptr, &at.ptr, at.align.min(8));
                    shapes.push(PTR);
                    values.push(Val::Imm(Sc::Ptr, pointer));
                }
                FfiAbiParam::Count { sources, ty, inout } => {
                    let first = self.view_length(sources[0].public_index, sources[0].nullable)?;
                    for other in sources.iter().skip(1) {
                        let other = self.view_length(other.public_index, other.nullable)?;
                        let differ = self.value(format!("icmp ne i64 {first}, {other}"));
                        self.panic_if(&differ, b"FFI span lengths differ", span);
                    }
                    if let Some(limit) = count_limit(table.kind(*ty)) {
                        let over = self.value(format!("icmp ugt i64 {first}, {limit}"));
                        self.panic_if(&over, b"FFI count does not fit", span);
                    }
                    let crate::types::Repr::Scalar(Sc::I(bits)) = self.m.ty.repr(*ty) else { return unsupported("a count that is not an integer") };
                    let count = self.int_convert(&first, 64, bits, false);
                    if *inout {
                        // `[FFI-11b]` — C reads the capacity and writes the count back here.
                        let slot = self.alloca(u64::from(bits / 8), u64::from(bits / 8));
                        self.emit(format!("store i{bits} {count}, ptr {slot}, align {}", bits / 8));
                        shapes.push(PTR);
                        values.push(Val::Imm(Sc::Ptr, slot.clone()));
                        outs.push((slot, *ty));
                    } else {
                        shapes.push(self.m.ty.shape(*ty));
                        values.push(Val::Imm(Sc::I(bits), count));
                    }
                }
                // `[FFI-23]` — C keeps the address of the object the token holds.
                FfiAbiParam::Retained { public_index } => {
                    let ty = body.args().nth(*public_index).expect("the retained parameter").1.ty;
                    let at = self.param_place(*public_index)?;
                    let (token_at, token) = match *table.kind(ty) {
                        TyKind::Ref { inner, .. } => (self.load(Sc::Ptr, &at.ptr, at.align.min(8)), inner),
                        _ => (at.ptr.clone(), ty),
                    };
                    let value_at = self.gep(&token_at, table.layout(token).field_offsets[0]);
                    let address = self.load(Sc::Ptr, &value_at, 8);
                    shapes.push(PTR);
                    values.push(Val::Imm(Sc::Ptr, address));
                }
                // `[FFI-11]` — C takes the object: its box leaves the record and frees nothing.
                FfiAbiParam::Owned { public_index, nullable } => {
                    let ty = body.args().nth(*public_index).expect("the owned parameter").1.ty;
                    let at = self.param_place(*public_index)?;
                    let pointer = if *nullable {
                        let TyKind::Enum(id) = *table.kind(ty) else { return unsupported("a nullable owned value that is not an `Option`") };
                        let def = table.enum_def(id).clone();
                        let boxed = def.variants[1].fields[0].ty;
                        let payload = self.gep(&at.ptr, table.variant_field_offsets(id)[1][0]);
                        let (discriminant, bits) = self.discriminant_at(&at)?;
                        let some = self.value(format!("icmp eq i{bits} {discriminant}, {}", def.variants[1].discriminant));
                        let (give, done) = (self.label(), self.label());
                        let before = self.block_label();
                        self.emit(format!("br i1 {some}, label %{give}, label %{done}"));
                        self.start(&give);
                        let pointer = self.release_box(&payload, boxed);
                        let after = self.block_label();
                        self.emit(format!("br label %{done}"));
                        self.start(&done);
                        self.value(format!("phi ptr [ null, %{before} ], [ {pointer}, %{after} ]"))
                    } else {
                        self.release_box(&at.ptr, ty)
                    };
                    shapes.push(PTR);
                    values.push(Val::Imm(Sc::Ptr, pointer));
                }
            }
        }
        // C's own result: a pointer for a view, a fixed array or a new object, else its type.
        let ret = body.return_ty();
        let c_result = counted.outputs.as_ref().map_or(ret, |outputs| outputs.c_result);
        let pointer_result = counted.result_owned.is_some() || counted.result_span.is_some() || counted.result_array.is_some();
        let result_shape = if pointer_result {
            Some(PTR)
        } else {
            (!self.m.ty.is_zst(c_result)).then(|| self.m.ty.shape(c_result))
        };
        // `[FFI-9]` — the foreign function's own convention.
        let (sig, callee) = if counted.abi.as_deref() == Some("vectorcall") {
            if self.m.ty.target != abi::Target::Win64 {
                return unsupported("`extern \"vectorcall\"` off Windows x64 ([FFI-9]: the convention is x86's)");
            }
            let name = format!("\u{1}{}", abi::vectorcall_symbol(&link_symbol(&counted.foreign_symbol), &shapes));
            (abi::classify_vectorcall(result_shape.as_ref(), &shapes), global_name(&name))
        } else {
            (abi::classify(self.m.ty.target, result_shape.as_ref(), &shapes), global_name(&counted.foreign_symbol))
        };
        declare_named(self.m, &counted.foreign_symbol, &callee, &sig);
        if let Some(ffi) = self.m.ffi.as_mut() {
            ffi.declared.insert(counted.foreign_symbol.clone(), callee.clone());
        }
        let values: Vec<(Val, u64, u64)> = values.into_iter().zip(&shapes).map(|(value, shape)| (value, shape.size(), shape.align())).collect();
        let (result_size, result_align) = match &result_shape {
            Some(shape) => (shape.size(), shape.align()),
            None => (0, 1),
        };
        let result = result_shape.as_ref().map(|_| self.alloca(result_size, result_align));
        self.lowered_call(&callee, &sig, &values, result.as_ref().map(|slot| (slot.as_str(), result_align, result_size)), "");
        let out_at = self.ret_slot();
        // `[FFI-16]` — a status: `Ok` of the outputs when C returned the success code, else
        // `Err(N(code))` (an output is not read then).
        if let Some(outputs) = &counted.outputs
            && let Some(ok) = outputs.status_ok
        {
            let status = result.expect("a status has a value");
            let crate::types::Repr::Scalar(Sc::I(bits)) = self.m.ty.repr(outputs.c_result) else { return unsupported("a status that is not an integer") };
            let code = self.load(Sc::I(bits), &status, result_align);
            let Some(out_at) = out_at else { return unsupported("a status call with no result") };
            let TyKind::Enum(id) = *table.kind(ret) else { return unsupported("a status call that returns no `Result`") };
            let def = table.enum_def(id).clone();
            let variant = |name: &str| def.variants.iter().position(|variant| variant.name.as_str() == name).expect("`Ok` and `Err`");
            let (ok_index, err_index) = (variant("Ok"), variant("Err"));
            let (size, align) = self.m.ty.size_align(ret);
            self.zero(&out_at, align, size);
            let offsets = table.variant_field_offsets(id);
            let succeeded = self.value(format!("icmp eq i{bits} {code}, {}", crate::func::int_literal(ok as u128, bits)));
            let (good, bad, done) = (self.label(), self.label(), self.label());
            self.emit(format!("br i1 {succeeded}, label %{good}, label %{bad}"));
            self.start(&good);
            let ok_ty = def.variants[ok_index].fields[0].ty;
            if !self.m.ty.is_zst(ok_ty) {
                let payload = self.gep(&out_at, offsets[ok_index][0]);
                self.tuple_into(&payload, ok_ty, &[], &outs);
            }
            let place = PlaceRef { ptr: out_at.clone(), ty: ret, variant: None, align };
            self.write_variant(&place, id, ok_index)?;
            self.emit(format!("br label %{done}"));
            self.start(&bad);
            let payload = self.gep(&out_at, offsets[err_index][0]);
            self.emit(format!("store i{bits} {code}, ptr {payload}, align {}", (bits / 8).min(8)));
            self.write_variant(&place, id, err_index)?;
            self.emit(format!("br label %{done}"));
            self.start(&done);
            return Ok(());
        }
        // C's result as the Ember value its contract makes of it.
        let value: Option<(String, Ty)> = if let Some(owned) = &counted.result_owned {
            let object = self.load(Sc::Ptr, result.as_ref().expect("a pointer result"), 8);
            Some((self.adopt(object, owned, c_result, span)?, c_result))
        } else if let Some(view) = &counted.result_span {
            let pointer = self.load(Sc::Ptr, result.as_ref().expect("a pointer result"), 8);
            let len = self.view_length(view.public_index, view.source_nullable)?;
            let len = if view.nullable {
                let null = self.value(format!("icmp eq ptr {pointer}, null"));
                self.value(format!("select i1 {null}, i64 -1, i64 {len}"))
            } else {
                len
            };
            let slot = self.alloca(16, 8);
            self.emit(format!("store ptr {pointer}, ptr {slot}, align 8"));
            let len_at = self.gep(&slot, 8);
            self.emit(format!("store i64 {len}, ptr {len_at}, align 8"));
            Some((slot, c_result))
        } else {
            result.map(|slot| (slot, c_result))
        };
        // `[FFI-11]` (ODR-246) — what `destroys(f)` is given of C's result is kept with the
        // callable (where the call registered one: a `nullable` one's `None` registered nothing).
        for (callback, passed) in counted.callbacks.iter().zip(&callbacks) {
            let (Some(at), Some((value, ty))) = (record(self.m, callback).and_then(|record| record.result), &value) else { continue };
            let done = passed.some.as_ref().map(|_| {
                let (kept, done) = (self.label(), self.label());
                let registered = self.value(format!("icmp ne ptr {}, null", passed.data));
                self.emit(format!("br i1 {registered}, label %{kept}, label %{done}"));
                self.start(&kept);
                done
            });
            let field = self.gep(&passed.data, at);
            let (size, align) = self.m.ty.size_align(*ty);
            self.copy(&field, align, value, align, size, false);
            if let Some(done) = done {
                self.emit(format!("br label %{done}"));
                self.start(&done);
            }
        }
        if let Some(out_at) = out_at {
            let first: Vec<(String, Ty)> = value.into_iter().collect();
            self.tuple_into(&out_at, ret, &first, &outs);
        }
        Ok(())
    }

    /// `[FFI-11]` — what a call returns of C's result and its outputs, written at `to` (of type
    /// `ty`): the one value alone, or a tuple of them in order (ODR-125, ODR-137).
    fn tuple_into(&mut self, to: &str, ty: Ty, first: &[(String, Ty)], outs: &[(String, Ty)]) {
        let parts: Vec<&(String, Ty)> = first.iter().chain(outs).collect();
        let offsets = if parts.len() > 1 { self.m.ty.table.layout(ty).field_offsets } else { vec![0] };
        for (part, offset) in parts.iter().zip(offsets) {
            let (size, align) = self.m.ty.size_align(part.1);
            let at = self.gep(to, offset);
            self.copy(&at, align, &part.0, align, size, false);
        }
    }

    /// `[FFI-11]` — a `ForeignBox` at `at` given to C: out of the runtime's record; its pointer.
    fn release_box(&mut self, at: &str, boxed: Ty) -> String {
        let offsets = self.m.ty.table.layout(boxed).field_offsets;
        let pointer = self.load(Sc::Ptr, at, 8);
        let creator_at = self.gep(at, offsets[2]);
        let creator = self.load(Sc::I(32), &creator_at, 4);
        self.call_rt("ffi_release", vec![Val::Imm(Sc::Ptr, pointer.clone()), Val::Imm(Sc::I(32), creator)], None);
        pointer
    }

    /// `[FFI-36a]` — a new foreign object's `ForeignBox` (recorded, with its destructor and under
    /// `threads(creator)` its creator), or for a nullable one `None` for null: its slot.
    fn adopt(&mut self, object: String, owned: &ember_hir::FfiOwnedResult, ty: Ty, span: ember_span::Span) -> Result<String> {
        let table = self.m.ty.table;
        let boxed = if owned.nullable {
            let TyKind::Enum(id) = *table.kind(ty) else { return unsupported("a nullable owned result that is not an `Option`") };
            table.enum_def(id).variants[1].fields[0].ty
        } else {
            ty
        };
        let destructor = foreign_destructor(self.m, owned.destructor.as_str(), owned.destructor_ret)?;
        let (size, align) = self.m.ty.size_align(ty);
        let slot = self.alloca(size, align);
        self.zero(&slot, align, size);
        let make = |cx: &mut Self, at: &str| {
            let offsets = cx.m.ty.table.layout(boxed).field_offsets;
            let loc = cx.loc(span);
            cx.call_rt("ffi_adopt", vec![Val::Imm(Sc::Ptr, object.clone()), loc], Some((at, 8)));
            let destructor_at = cx.gep(at, offsets[1]);
            cx.emit(format!("store ptr {destructor}, ptr {destructor_at}, align 8"));
            if owned.creator {
                let creator_at = cx.gep(at, offsets[2]);
                cx.call_rt("ffi_thread", Vec::new(), Some((&creator_at, 4)));
            }
        };
        if !owned.nullable {
            make(self, &slot);
            return Ok(slot);
        }
        let TyKind::Enum(id) = *table.kind(ty) else { unreachable!("checked above") };
        let place = PlaceRef { ptr: slot.clone(), ty, variant: None, align };
        let null = self.value(format!("icmp eq ptr {object}, null"));
        let (none, some, done) = (self.label(), self.label(), self.label());
        self.emit(format!("br i1 {null}, label %{none}, label %{some}"));
        self.start(&none);
        self.write_variant(&place, id, 0)?;
        self.emit(format!("br label %{done}"));
        self.start(&some);
        let payload = self.gep(&slot, table.variant_field_offsets(id)[1][0]);
        make(self, &payload);
        self.write_variant(&place, id, 1)?;
        self.emit(format!("br label %{done}"));
        self.start(&done);
        Ok(slot)
    }
}

/// `[BLD-FFI-5a]` (ODR-162) — what a library gives C and C++ of each `@sync` class an export hands
/// them, as its header declares it (`ember_codegen_c::ClassExport`): a handle's count and its end,
/// and each `pub` method, which attaches the calling thread and calls the method's body (through the
/// object's method table when it is virtual) with what C passed: a handle as the object's address
/// (with a count of its own where the method keeps it), a view from its address and length, a value
/// the method borrows by its address.
pub(crate) fn emit_class_exports(m: &mut Module<'_>, classes: &[ember_codegen_c::ClassExport]) -> Result<()> {
    for class in classes {
        define_c_fn(m, &class.retain, "", None, &[Param::Ptr], "", |cx, values, _| {
            cx.call_rt("obj_retain", vec![values[0].clone()], None);
            Ok(())
        })?;
        define_c_fn(m, &class.release, "", None, &[Param::Ptr], "", |cx, values, _| {
            cx.call_rt("rt_thread_attach", Vec::new(), None);
            cx.call_rt("obj_release", vec![values[0].clone()], None);
            Ok(())
        })?;
        for method in &class.methods {
            class_method(m, method)?;
        }
    }
    Ok(())
}

fn class_method(m: &mut Module<'_>, method: &ember_codegen_c::MethodExport) -> Result<()> {
    use ember_codegen_c::Crossed;
    let Some(body) = m.bodies.get(method.body.as_str()).copied() else {
        return unsupported(format!("the class method `{}` without its body", method.body));
    };
    let declared: Vec<Ty> = body.args().skip(1).map(|(_, decl)| decl.ty).collect();
    let mut params = vec![Param::Ptr];
    for (crossed, ty) in method.crossings.iter().zip(&declared) {
        match crossed {
            Crossed::Handle { .. } => params.push(Param::Ptr),
            Crossed::View => params.extend([Param::Ptr, Param::Size]),
            Crossed::Borrowed => match m.ty.kind(*ty) {
                TyKind::Ref { inner, .. } => params.push(Param::Ty(*inner)),
                _ => return unsupported("a borrowed class-method parameter that is not a reference"),
            },
            Crossed::Value => params.push(Param::Ty(*ty)),
        }
    }
    let virtual_slot = body.class_owner.zip(body.class_virtual_slot);
    let symbol = body.symbol.clone();
    define_c_fn(m, &method.symbol, "", Some(body.return_ty()), &params, "", |cx, values, result| {
        cx.call_rt("rt_thread_attach", Vec::new(), None);
        let by_address = |cx: &mut FnCx<'_, '_>, value: &Val, size: u64, align: u64| {
            let (ptr, _) = cx.spill(value, size, align);
            Val::Imm(Sc::Ptr, ptr)
        };
        let mut arguments = vec![if method.receiver_by_address { by_address(cx, &values[0], 8, 8) } else { values[0].clone() }];
        let mut next = 1;
        for (crossed, ty) in method.crossings.iter().zip(&declared) {
            let value = values[next].clone();
            next += 1;
            arguments.push(match crossed {
                Crossed::Handle { by_address: address, kept } => {
                    if *kept {
                        cx.call_rt("obj_retain", vec![value.clone()], None);
                    }
                    if *address { by_address(cx, &value, 8, 8) } else { value }
                }
                Crossed::View => {
                    let length = values[next].clone();
                    next += 1;
                    let view = cx.alloca(16, 8);
                    cx.store(&value, &view, 8, 8, false);
                    let at = cx.gep(&view, 8);
                    cx.store(&length, &at, 8, 8, false);
                    Val::Mem(view, 8)
                }
                Crossed::Borrowed => {
                    let TyKind::Ref { inner, .. } = *cx.m.ty.kind(*ty) else { unreachable!("checked above") };
                    let (size, align) = cx.m.ty.size_align(inner);
                    by_address(cx, &value, size, align)
                }
                Crossed::Value => value,
            });
        }
        let dest = result.as_ref().map(|(ptr, align)| (ptr.as_str(), *align));
        match virtual_slot {
            Some((owner, slot)) => cx.virtual_call_values(owner, slot, owner, arguments, dest),
            None => cx.call_body(&symbol, arguments, dest),
        }
    })
}

/// `[FFI-37]` — an instrumented test's foreign functions: each callee symbol's index, each
/// function's name and the symbol whose address names its library, and the call sites so far, as
/// the C backend numbers them; the IR's name for a C function a counted wrapper declares.
pub(crate) struct FfiInstrument {
    index: std::collections::BTreeMap<String, u32>,
    pub functions: Vec<(String, String)>,
    pub sites: Vec<(u32, String)>,
    pub declared: std::collections::BTreeMap<String, String>,
}

impl FfiInstrument {
    pub(crate) fn new(bodies: &[Body]) -> FfiInstrument {
        let mut index = std::collections::BTreeMap::new();
        let mut functions = Vec::new();
        for body in bodies.iter().filter(|body| body.contracts.foreign.is_some()) {
            let address = match &body.ffi_counted {
                Some(counted) => counted.foreign_symbol.clone(),
                None => body.symbol.clone(),
            };
            index.insert(body.symbol.clone(), functions.len() as u32);
            functions.push((body.name.clone(), address));
        }
        FfiInstrument { index, functions, sites: Vec::new(), declared: std::collections::BTreeMap::new() }
    }

    /// Whether a call to `symbol` is counted.
    pub(crate) fn counts(&self, symbol: &str) -> bool {
        self.index.contains_key(symbol)
    }

    /// The next call site, of `symbol` at `place`: its function's number and its own.
    pub(crate) fn site(&mut self, symbol: &str, place: String) -> Option<(u32, usize)> {
        let function = *self.index.get(symbol)?;
        self.sites.push((function, place));
        Some((function, self.sites.len() - 1))
    }
}
