//! One MIR body as one IR function.
//!
//! Every local lives in a stack slot of its layout (an `alloca` in the entry block, which
//! `mem2reg` and SROA promote at `-O1` and above), a place is a pointer reached by byte offsets,
//! and a scalar operand is loaded into a register. Arguments and results cross by the platform's C
//! ABI (`crate::abi`): a value passed behind a pointer is the callee's own copy, so the parameter
//! is that copy's slot, and an `sret` result is written in place.

use std::collections::HashMap;
use std::fmt::Write as _;

use crate::abi::{self, Part, PartTy, Pass, Return, Shape, Signature};
use crate::runtime::{self, config};
use crate::types::{Repr, Sc, offset_align};
use crate::{Module, Options, Profile, Result, global_name, unsupported};
use ember_mir::{
    AggregateKind, AssertKind, BinOp, Body, Builtin, CastKind, Const, FuncRef, LocalId, Operand, Place, Projection,
    Rvalue, StmtKind, Terminator, UnOp,
};
use ember_types::{Ty, TyKind};

/// A value: nothing, a register, or bytes in memory (their address and its known alignment).
#[derive(Clone, Debug)]
pub(crate) enum Val {
    Zst,
    Imm(Sc, String),
    Mem(String, u64),
}

/// A place's address, its type, the variant a `Downcast` chose, and the alignment the address
/// is known to have.
#[derive(Clone)]
pub(crate) struct PlaceRef {
    pub ptr: String,
    pub ty: Ty,
    pub variant: Option<usize>,
    pub align: u64,
}

pub(crate) struct FnCx<'m, 'a> {
    pub(crate) m: &'m mut Module<'a>,
    /// The MIR body this function is (`None` for the glue the backend writes itself).
    body: Option<&'a Body>,
    /// The entry block's slots.
    allocas: String,
    /// What the entry block does after them: the parameters stored into their slots.
    prologue: String,
    out: String,
    next: u32,
    /// Each local's slot (`None`: it has no bytes).
    addr: Vec<Option<String>>,
    sig: Signature,
    /// The label of the block being written.
    block: String,
    /// The span of the statement or terminator being written, for the panics it may make.
    span: ember_span::Span,
    /// `[CG-LL-3]` — the function's `DISubprogram` and line, when it is made from a MIR body.
    scope: Option<(usize, u32)>,
    /// Each line and column's `DILocation` in it.
    locations: HashMap<(u32, u32), usize>,
    /// `[GRM-39]` — the literals written straight into the next literal's field (`fold`).
    folded: HashMap<LocalId, Rvalue>,
    /// `[HR-3b]` — in a reloadable image whose instances migrate, the body's frame record.
    frame_record: Option<crate::reload::FrameRecord>,
}

/// A literal's operand: its value, or a folded literal's temporary (`[GRM-39]`).
enum Field {
    Val(Val),
    Folded(LocalId),
}

/// The ABI signature of a function returning `ret` and taking `params`.
fn signature(m: &Module<'_>, ret: Ty, params: impl Iterator<Item = Ty>) -> Signature {
    let ret = (!m.ty.is_zst(ret)).then(|| m.ty.shape(ret));
    let params: Vec<Shape> = params.map(|ty| m.ty.shape(ty)).collect();
    abi::classify(m.ty.target, ret.as_ref(), &params)
}

pub(crate) fn body_signature(m: &Module<'_>, body: &Body) -> Signature {
    let values = ffi_value_params(m, body);
    let passed: Vec<Ty> = body.args().enumerate().map(|(index, (_, decl))| passed_ty(m, &values, index, decl.ty)).collect();
    let mut sig = if is_vectorcall(body) {
        let ret = (!m.ty.is_zst(body.return_ty())).then(|| m.ty.shape(body.return_ty()));
        let params: Vec<Shape> = passed.iter().map(|ty| m.ty.shape(*ty)).collect();
        abi::classify_vectorcall(ret.as_ref(), &params)
    } else {
        signature(m, body.return_ty(), passed.iter().copied())
    };
    // `[CG-LL-3]` — the facts the types and the borrow checker give, derived, never assumed
    // (`[SIMD-3]`): a reference is never null, its referent stays allocated for the call, and a
    // shared one to what holds no cell is only read (`abi::Ext::Ref`); a list kernel's views (`restrict_views`, the C
    // backend's `restrict` parameters) are each their elements' pointer, apart from every other,
    // and their length. Such a kernel is internal: its parameters are this backend's own.
    // An export's caller is foreign code, which the type system does not bind: no fact there; nor
    // in a function C may call through its value's C entry (`[FN-6b]`).
    let given_to_c = m.callables.natives.get(&body.symbol).is_some_and(|ty| m.callables.given_to_c.contains(ty))
        || m.callback_closures.contains(&body.symbol);
    let internal = body.abi.is_none() && !body.is_extern_declaration && !given_to_c;
    for (pass, ty) in sig.params.iter_mut().zip(&passed) {
        match (pass, m.ty.kind(*ty)) {
            (Pass::Direct(parts), &TyKind::Ref { inner, mutable })
                if internal && parts.len() == 1 && parts[0].ty == PartTy::Ptr && !m.ty.is_zst(inner) =>
            {
                let (size, _) = m.ty.size_align(inner);
                parts[0].ext = abi::Ext::Ref { size, readonly: !mutable && !m.ty.table.holds_a_cell(inner) };
            }
            (pass, TyKind::Span { .. }) if body.restrict_views && body.abi.is_none() => {
                *pass = Pass::Direct(vec![
                    Part { offset: 0, ty: PartTy::Ptr, ext: abi::Ext::NoAlias },
                    Part { offset: 8, ty: PartTy::I(64), ext: abi::Ext::None },
                ]);
            }
            // D-569 — a view as its two words, not behind a pointer to a copy (Windows x64).
            (pass, TyKind::Span { .. } | TyKind::Str) if m.split_views.contains(&body.symbol) => {
                *pass = Pass::Direct(vec![
                    Part { offset: 0, ty: PartTy::Ptr, ext: abi::Ext::None },
                    Part { offset: 8, ty: PartTy::I(64), ext: abi::Ext::None },
                ]);
            }
            _ => {}
        }
    }
    sig
}

/// `[FFI-9]` — a function of an `extern "vectorcall"` block, or one defined with the convention.
fn is_vectorcall(body: &Body) -> bool {
    body.abi.as_deref() == Some("vectorcall")
}

/// The IR name of `body`: its symbol's, or for a `__vectorcall` function MSVC's decorated symbol
/// (`\01scale@@16`, its bytes counted from the C types, as Clang counts them: an aggregate passed
/// by address counts its own size, which LLVM, decorating by the IR's parameters, would count as a
/// pointer's; the `\01` keeps LLVM from decorating it again).
pub(crate) fn body_name(m: &Module<'_>, body: &Body) -> String {
    if !is_vectorcall(body) {
        return global_name(&body.symbol);
    }
    global_name(&format!("\u{1}{}", vectorcall_symbol(&m.ty, body)))
}

/// `[FFI-9]` — MSVC's symbol of the `__vectorcall` function `body` (`scale@@16`).
pub(crate) fn vectorcall_symbol(types: &crate::types::Types<'_>, body: &Body) -> String {
    let values = ffi_value_params_of(types.table, body);
    let shapes: Vec<Shape> = body
        .args()
        .enumerate()
        .map(|(index, (_, decl))| types.shape(passed_ty_of(types.table, &values, index, decl.ty)))
        .collect();
    abi::vectorcall_symbol(&crate::link_symbol(&body.symbol), &shapes)
}

/// `[FFI-9]` — `__vectorcall` is Windows x64's here (the C backend's `EMBER_CC_VECTORCALL` stops a
/// build without it the same way).
fn check_convention(m: &Module<'_>, body: &Body) -> Result<()> {
    if is_vectorcall(body) && m.ty.target != abi::Target::Win64 {
        return unsupported("`extern \"vectorcall\"` off Windows x64 ([FFI-9]: the convention is x86's)");
    }
    Ok(())
}

/// `[FFI-10]` — which parameters an exported function takes by value though its body borrows
/// them: a borrowed C-safe value Ember passes by address is C's `T`, not `const T*` (the C
/// backend's `ffi_export_value_params`). Empty for any other function.
pub(crate) fn ffi_value_params(m: &Module<'_>, body: &Body) -> Vec<bool> {
    ffi_value_params_of(m.ty.table, body)
}

fn ffi_value_params_of(table: &ember_types::TypeTable, body: &Body) -> Vec<bool> {
    if body.abi.is_none() || body.is_extern_declaration {
        return Vec::new();
    }
    body.args()
        .enumerate()
        .map(|(index, (_, decl))| {
            body.param_modes.get(index) == Some(&ember_mir::ParameterMode::Borrow)
                && matches!(table.kind(decl.ty), TyKind::Ref { inner, .. } if table.is_ffi_safe(*inner) && table.passed_by_address(*inner))
        })
        .collect()
}

/// The type parameter `index` is passed as: a by-value export parameter's pointee.
fn passed_ty(m: &Module<'_>, values: &[bool], index: usize, ty: Ty) -> Ty {
    passed_ty_of(m.ty.table, values, index, ty)
}

fn passed_ty_of(table: &ember_types::TypeTable, values: &[bool], index: usize, ty: Ty) -> Ty {
    match table.kind(ty) {
        TyKind::Ref { inner, .. } if values.get(index) == Some(&true) => *inner,
        _ => ty,
    }
}

/// The type of `place` in `body`.
pub(crate) fn place_type(m: &Module<'_>, body: &Body, place: &Place) -> Ty {
    let types = m.ty.table;
    let mut ty = body.local(place.local).ty;
    let mut variant = None;
    for projection in &place.projection {
        ty = match (projection, types.kind(ty)) {
            (Projection::Downcast(v), TyKind::Enum(_)) => {
                variant = Some(*v);
                continue;
            }
            (Projection::Field(index), TyKind::Enum(id)) => {
                let fields = &types.enum_def(*id).variants[variant.take().unwrap_or(0)].fields;
                fields.get(*index).map_or(ty, |field| field.ty)
            }
            (Projection::Field(index), TyKind::Struct(id)) => types.struct_def(*id).fields.get(*index).map_or(ty, |field| field.ty),
            (Projection::Field(index), TyKind::Class(id)) => types.class_field_at(*id, *index).map_or(ty, |field| field.ty),
            (Projection::Field(index), TyKind::Tuple(items)) => items.get(*index).copied().unwrap_or(ty),
            (Projection::Field(0), TyKind::Str | TyKind::Vec { .. } | TyKind::Span { .. }) => ty,
            (Projection::Field(_), TyKind::Str | TyKind::Vec { .. } | TyKind::Span { .. }) => m.ty.usize_ty,
            (
                Projection::Index(_) | Projection::ConstIndex(_),
                TyKind::Array { elem, .. } | TyKind::Vec { elem, .. } | TyKind::Span { elem, .. },
            ) => *elem,
            (Projection::Index(_) | Projection::ConstIndex(_), TyKind::Ptr { inner, .. }) => *inner,
            (Projection::Deref, TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. }) => *inner,
            _ => ty,
        };
        variant = None;
    }
    ty
}

/// Declares a body of this program the module calls (its definition, when it has one, removes
/// the declaration).
pub(crate) fn declare_body_symbol(m: &mut Module<'_>, symbol: &str) -> Result<()> {
    let Some(body) = m.bodies.get(symbol).copied() else {
        return unsupported(format!("a call to `{symbol}`, which has no body"));
    };
    check_convention(m, body)?;
    let sig = body_signature(m, body);
    let name = body_name(m, body);
    // A foreign function is the C library's: whether it unwinds is its own.
    let attrs = if body.is_extern_declaration { "" } else { fn_attrs(m) };
    let variadic = body.variadic;
    // By the linker's symbol: a foreign function the runtime's tables also call is declared once.
    m.declare(&crate::link_symbol(symbol), || {
        let mut params = ir_params(&sig, false);
        if variadic {
            params.push("...".to_string());
        }
        format!("declare {}{} {name}({}) {attrs}", sig.conv.ir(), ret_with_attr(&sig), params.join(", "))
    });
    Ok(())
}

/// The IR parameter list of `sig`, `%p<N>` named when `named`.
pub(crate) fn ir_params(sig: &Signature, named: bool) -> Vec<String> {
    let mut out = Vec::new();
    let mut index = 0;
    let mut name = |out: &mut Vec<String>, text: String| {
        if named {
            out.push(format!("{text} %p{index}"));
        } else {
            out.push(text);
        }
        index += 1;
    };
    // The result's slot is a fresh one at every call (`lowered_call`), as C's ABI has it: nothing
    // else reaches it.
    if let Return::Sret { size, align } = sig.ret {
        name(&mut out, format!("ptr noalias sret([{size} x i8]) align {align}"));
    }
    for pass in &sig.params {
        match pass {
            Pass::Ignore => {}
            Pass::Direct(parts) => {
                for part in parts {
                    name(&mut out, format!("{}{}", part.ty, part.ext.attr()));
                }
            }
            Pass::Indirect { size, align, byval: true } => name(&mut out, format!("ptr byval([{size} x i8]) align {align}")),
            Pass::Indirect { .. } => name(&mut out, "ptr".to_string()),
        }
    }
    out
}

/// The result's type with its extension (`zeroext i1`).
pub(crate) fn ret_with_attr(sig: &Signature) -> String {
    match ret_attr(sig).as_str() {
        "" => sig.ret.ir(),
        ext => format!("{} {}", ext.trim(), sig.ret.ir()),
    }
}

pub(crate) fn ret_attr(sig: &Signature) -> String {
    match &sig.ret {
        Return::Direct(parts) if parts.len() == 1 => parts[0].ext.attr(),
        _ => String::new(),
    }
}

/// The attributes every function of the program has: no unwinding (a panic aborts), unwind
/// tables for the backtrace and the debugger, on Linux stack probes (`[RT-12]`; Windows probes
/// every frame larger than a page itself), and the CPU and features Clang gives the runtime's C
/// (`-mcx16`, `[THR-14]`). Without them, a link-time optimisation compiles the IR's functions
/// for the linker's default CPU: its features are a subset of the runtime's, LLVM never inlines a
/// function needing features its caller lacks, so no runtime inline function would be inlined
/// into the program, and a 16-byte compare-exchange would be a library call no link has.
pub(crate) fn fn_attrs(m: &Module<'_>) -> &'static str {
    match m.ty.target {
        abi::Target::SysV => {
            "nounwind uwtable \"probe-stack\"=\"inline-asm\" \"target-cpu\"=\"x86-64\" \"target-features\"=\"+cx16\" \"tune-cpu\"=\"generic\""
        }
        abi::Target::Win64 => "nounwind uwtable \"target-cpu\"=\"x86-64\" \"target-features\"=\"+cx16\" \"tune-cpu\"=\"generic\"",
        // Clang's own for `aarch64-linux-gnu`, which the runtime's C is compiled with.
        abi::Target::Aapcs64 => {
            "nounwind uwtable \"probe-stack\"=\"inline-asm\" \"target-cpu\"=\"generic\" \"target-features\"=\"+fp-armv8,+neon,+v8a,-fmv\""
        }
    }
}

pub(crate) fn emit_body<'a>(m: &mut Module<'a>, body: &'a Body) -> Result<()> {
    if body.variadic {
        return unsupported("a variadic function definition");
    }
    // `[FFI-9]` — on x86-64, `"system"` and `"fastcall"` are the platform's C convention;
    // `"vectorcall"` is its own (Windows x64).
    if body.abi.as_deref().is_some_and(|abi| !matches!(abi, "C" | "system" | "fastcall" | "vectorcall")) {
        return unsupported(format!("the calling convention `{}`", body.abi.as_deref().unwrap_or_default()));
    }
    check_convention(m, body)?;
    if is_vectorcall(body) && crate::reload::is_reloadable(m, &body.symbol) {
        return unsupported("a reloadable `extern \"vectorcall\"` function ([HR-6]'s thunk is the C convention's)");
    }
    let sums = ember_codegen_c::byte_sum_loops(body, m.ty.table, |place| place_type(m, body, place));
    // `[HR-3b]` (A1) — what a checkpoint's frames of this body still use, described beside it.
    let frames = m.reload.as_ref().is_some_and(|reload| reload.instances).then(|| crate::reload::frame_info(m, body));
    define_body(m, body, |cx| {
        if let Some(info) = frames {
            cx.push_frame_record(&info);
        }
        for index in block_order(body) {
            let block = &body.blocks[index];
            cx.start(&format!("bb{index}"));
            cx.stmts(&block.stmts)?;
            // `[HR-3b]` — the call this frame is in, for a checkpoint reached below it.
            if let Some(site) = cx.frame_record.as_ref().and_then(|record| record.sites.get(&index).copied()) {
                let record = cx.frame_record.as_ref().map(|record| record.record.clone()).expect("a frame record");
                let at = cx.gep(&record, 16);
                cx.emit(format!("store i32 {site}, ptr {at}, align 8"));
            }
            match sums.iter().find(|sum| sum.header == index) {
                Some(sum) => cx.byte_sum(sum)?,
                None => cx.terminator(&block.terminator, block.terminator_span)?,
            }
        }
        Ok(())
    })
}

/// The blocks in reverse postorder from the entry, then any no path reaches: a loop's body before
/// the step that ends its turn and the code after it. At `-O0` the machine code keeps this order,
/// so a line's first code is where its statement starts (a debugger's breakpoint on the line stops
/// before it runs, `[TST-5]`), and a forward branch is the common case.
fn block_order(body: &Body) -> Vec<usize> {
    let successors = |index: usize| -> Vec<usize> {
        match &body.blocks[index].terminator {
            Terminator::Goto(next) => vec![next.0 as usize],
            Terminator::SwitchInt { targets, otherwise, .. } => {
                targets.iter().map(|(_, target)| target.0 as usize).chain(std::iter::once(otherwise.0 as usize)).collect()
            }
            Terminator::Call { next, .. } | Terminator::Assert { next, .. } => vec![next.0 as usize],
            Terminator::Return | Terminator::Unreachable => Vec::new(),
        }
    };
    let count = body.blocks.len();
    let mut seen = vec![false; count];
    let mut post = Vec::with_capacity(count);
    // An explicit stack of (block, its successors not yet visited).
    let mut stack: Vec<(usize, Vec<usize>)> = Vec::new();
    if count > 0 {
        seen[0] = true;
        let mut next = successors(0);
        next.reverse();
        stack.push((0, next));
    }
    while let Some((block, pending)) = stack.last_mut() {
        match pending.pop() {
            Some(successor) if successor < count && !seen[successor] => {
                seen[successor] = true;
                let mut next = successors(successor);
                next.reverse();
                stack.push((successor, next));
            }
            Some(_) => {}
            None => {
                post.push(*block);
                stack.pop();
            }
        }
    }
    post.reverse();
    post.extend((0..count).filter(|index| !seen[*index]));
    post
}

/// The function `body` names, its frame laid out and its code what `build` writes from `bb0`.
pub(crate) fn define_body<'a>(m: &mut Module<'a>, body: &'a Body, build: impl FnOnce(&mut FnCx<'_, 'a>) -> Result<()>) -> Result<()> {
    let sig = body_signature(m, body);
    let optimized = m.profile != Profile::Debug;
    // `[HR-6]` — a reloadable function's symbol is its thunk's; its body has a name of its own.
    let reloadable = crate::reload::is_reloadable(m, &body.symbol);
    let defined = if reloadable { ember_codegen_c::image::body_symbol(&body.symbol) } else { body.symbol.clone() };
    let link = crate::link_symbol(&defined);
    let (subprogram, line) = m.debug.subprogram(m.map, &link, body.span, optimized);
    let folded = crate::fold::foldable(m.ty.table, body);
    let mut cx = FnCx {
        m,
        body: Some(body),
        allocas: String::new(),
        prologue: String::new(),
        out: String::new(),
        next: 0,
        addr: vec![None; body.locals.len()],
        sig,
        block: "entry".to_string(),
        span: ember_span::Span::DUMMY,
        scope: Some((subprogram, line)),
        locations: HashMap::new(),
        folded,
        frame_record: None,
    };
    cx.frame()?;
    cx.declare_locals(subprogram, line);
    build(&mut cx)?;
    let mut attrs = fn_attrs(cx.m).to_string();
    // `[CG-C-3]`, `[CG-C-3a]` — the C backend's choice: `@inline` and the standard accessors
    // forced, small bodies hinted, `@noinline` never.
    match cx.m.inline_bodies.get(&body.symbol) {
        _ if body.inline.never => attrs.push_str(" noinline"),
        Some(true) => attrs.push_str(" alwaysinline"),
        Some(false) => attrs.push_str(" inlinehint"),
        None if body.inline.always => attrs.push_str(" alwaysinline"),
        None => {}
    }
    if body.inline.cold {
        attrs.push_str(" cold");
    }
    if body.inline.hot {
        attrs.push_str(" hot");
    }
    let linkage = if body.abi.is_some() && !reloadable { "" } else { "internal " };
    let header = format!(
        "define {linkage}{}{} {}({}) {attrs} !dbg !{subprogram} {{",
        cx.sig.conv.ir(),
        ret_with_attr(&cx.sig),
        if reloadable { global_name(&defined) } else { body_name(cx.m, body) },
        ir_params(&cx.sig, true).join(", ")
    );
    let text = format!("{header}\nentry:\n{}{}  br label %bb0\n{}}}\n\n", cx.allocas, cx.prologue, cx.out);
    let sig = cx.sig.clone();
    cx.m.fns.push_str(&text);
    if reloadable {
        crate::reload::emit_thunk(m, body, &sig);
    }
    Ok(())
}

/// A parameter of a function the backend writes itself: a value of an Ember type, or one of the
/// C types its tables erase to.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Param {
    Ty(Ty),
    /// `void*` (a receiver a table erases, a callback's state).
    Ptr,
    /// C's `int`.
    Int,
    /// C's `size_t`.
    Size,
    /// The runtime's `ember_loc`.
    Loc,
}

impl Param {
    fn shape(self, m: &Module<'_>) -> Shape {
        match self {
            Param::Ty(ty) => m.ty.shape(ty),
            Param::Ptr => Shape::Scalar(abi::Scalar::Ptr),
            Param::Int => Shape::Scalar(abi::Scalar::Int { bytes: 4, signed: true }),
            Param::Size => Shape::Scalar(abi::Scalar::Int { bytes: 8, signed: false }),
            Param::Loc => runtime::c_type("$loc").shape().expect("ember_loc has a shape"),
        }
    }

    fn repr(self, m: &Module<'_>) -> Repr {
        match self {
            Param::Ty(ty) => m.ty.repr(ty),
            Param::Ptr => Repr::Scalar(Sc::Ptr),
            Param::Int => Repr::Scalar(Sc::I(32)),
            Param::Size => Repr::Scalar(Sc::I(64)),
            Param::Loc => Repr::Mem { size: 16, align: 8 },
        }
    }
}

/// A function the backend writes itself with the platform C ABI's signature for `ret` and
/// `params`: `build` gets each parameter as a value and the result's slot, and writes the body
/// up to its return.
pub(crate) fn define_c_fn<'a>(
    m: &mut Module<'a>,
    symbol: &str,
    linkage: &str,
    ret: Option<Ty>,
    params: &[Param],
    attrs: &str,
    build: impl FnOnce(&mut FnCx<'_, 'a>, &[Val], Option<(String, u64)>) -> Result<()>,
) -> Result<()> {
    let ret = ret.filter(|ty| !m.ty.is_zst(*ty));
    let shapes: Vec<Shape> = params.iter().map(|param| param.shape(m)).collect();
    let sig = c_fn_signature(m, ret, params);
    let mut cx = FnCx::glue(m);
    let mut index = 0usize;
    let ret_slot = match (&sig.ret, ret) {
        (Return::Sret { align, .. }, Some(_)) => {
            index = 1;
            Some(("%p0".to_string(), *align))
        }
        (Return::Direct(parts), Some(ty)) => {
            let (size, align) = cx.m.ty.size_align(ty);
            Some((cx.alloca(size.max(parts_span(parts)), align), align))
        }
        _ => None,
    };
    let mut values = Vec::with_capacity(params.len());
    for ((param, pass), shape) in params.iter().zip(sig.params.clone()).zip(&shapes) {
        let repr = param.repr(cx.m);
        let value = match pass {
            Pass::Ignore => Val::Zst,
            Pass::Indirect { align, .. } => {
                let value = Val::Mem(format!("%p{index}"), align);
                index += 1;
                value
            }
            Pass::Direct(parts) => match (repr, parts.as_slice()) {
                (Repr::Scalar(sc), [part]) if sc_part(sc) == Some(part.ty) => {
                    let value = Val::Imm(sc, format!("%p{index}"));
                    index += 1;
                    value
                }
                _ => {
                    let (size, align) = (shape.size(), shape.align());
                    let slot = cx.alloca(size.max(parts_span(&parts)), align);
                    for part in &parts {
                        let incoming = format!("%p{index}");
                        cx.store_part(&slot, align, part, &incoming);
                        index += 1;
                    }
                    match repr {
                        Repr::Scalar(sc) => Val::Imm(sc, cx.load(sc, &slot, align)),
                        _ => Val::Mem(slot, align),
                    }
                }
            },
        };
        values.push(value);
    }
    build(&mut cx, &values, ret_slot.clone())?;
    match &sig.ret {
        Return::Void | Return::Sret { .. } => cx.emit("ret void"),
        Return::Direct(parts) => {
            let (slot, align) = ret_slot.expect("a result in registers has a slot");
            let loaded: Vec<String> = parts.iter().map(|part| cx.load_part(&slot, align, part)).collect();
            if parts.len() == 1 {
                cx.emit(format!("ret {} {}", parts[0].ty, loaded[0]));
            } else {
                let ty = sig.ret.ir();
                let mut aggregate = "undef".to_string();
                for (index, (part, value)) in parts.iter().zip(&loaded).enumerate() {
                    aggregate = cx.value(format!("insertvalue {ty} {aggregate}, {} {value}, {index}", part.ty));
                }
                cx.emit(format!("ret {ty} {aggregate}"));
            }
        }
    }
    let base = fn_attrs(cx.m);
    let text = format!(
        "define {linkage}{} {}({}) {base}{attrs} {{\nentry:\n{}  br label %start\nstart:\n{}}}\n\n",
        ret_with_attr(&sig),
        global_name(symbol),
        ir_params(&sig, true).join(", "),
        cx.allocas,
        cx.out
    );
    cx.m.fns.push_str(&text);
    Ok(())
}

/// The bytes a value's parts reach: AAPCS64 passes a 4-byte struct as an `i64` and a 12-byte one as
/// `[2 x i64]`, so its storage is padded to them where the parts are read or written.
fn parts_span(parts: &[Part]) -> u64 {
    parts.iter().map(|part| part.offset + part.ty.size()).max().unwrap_or(0)
}

/// The ABI signature of a function the backend writes itself (as `define_c_fn` makes it).
pub(crate) fn c_fn_signature(m: &Module<'_>, ret: Option<Ty>, params: &[Param]) -> Signature {
    let ret = ret.filter(|ty| !m.ty.is_zst(*ty)).map(|ty| m.ty.shape(ty));
    let shapes: Vec<Shape> = params.iter().map(|param| param.shape(m)).collect();
    abi::classify(m.ty.target, ret.as_ref(), &shapes)
}

impl<'m, 'a> FnCx<'m, 'a> {
    /// A function the backend writes itself, with no MIR body.
    pub(crate) fn glue(m: &'m mut Module<'a>) -> FnCx<'m, 'a> {
        FnCx {
            m,
            body: None,
            allocas: String::new(),
            prologue: String::new(),
            out: String::new(),
            next: 0,
            addr: Vec::new(),
            sig: Signature { ret: Return::Void, params: Vec::new(), conv: abi::Conv::C },
            block: "start".to_string(),
            span: ember_span::Span::DUMMY,
            scope: None,
            locations: HashMap::new(),
            folded: HashMap::new(),
            frame_record: None,
        }
    }

    pub(crate) fn allocas_text(&self) -> &str {
        &self.allocas
    }

    pub(crate) fn out_text(&self) -> &str {
        &self.out
    }

    pub(crate) fn block_label(&self) -> String {
        self.block.clone()
    }

    /// Whether the block being written has ended (its last instruction a terminator).
    pub(crate) fn terminated(&self) -> bool {
        let last = self.out.trim_end().lines().last().unwrap_or("").trim_start();
        ["br ", "ret ", "ret\n", "switch ", "unreachable"].iter().any(|start| last.starts_with(start))
            || last == "ret void"
            || last.starts_with(']')
    }

    // -- writing ----------------------------------------------------------------------------------

    pub(crate) fn tmp(&mut self) -> String {
        self.next += 1;
        format!("%t{}", self.next)
    }

    /// `[TST-5]`, `[CG-LL-3]` — in the `debug` profile, each parameter and named local the debugger
    /// shows, by its Ember name and type, at its slot.
    fn declare_locals(&mut self, scope: usize, line: u32) {
        if !self.m.debug.full {
            return;
        }
        let body = self.mir();
        let by_value = ffi_value_params(self.m, body);
        let at = self.m.debug.location(line, 0, scope);
        for (index, decl) in body.locals.iter().enumerate() {
            let arg = (index >= 1 && index <= body.arg_count).then_some(index);
            let named = matches!(decl.kind, ember_mir::LocalKind::User | ember_mir::LocalKind::Arg);
            let (Some(name), Some(slot)) = (decl.name.as_deref(), self.addr[index].clone()) else { continue };
            if !named || self.m.ty.is_zst(decl.ty) || arg.is_some_and(|arg| by_value.get(arg - 1) == Some(&true)) {
                continue;
            }
            let ty = self.m.debug.describe(&self.m.ty, decl.ty);
            let variable = self.m.debug.variable(self.m.map, name, arg, scope, decl.span, ty);
            let _ = writeln!(self.prologue, "  #dbg_declare(ptr {slot}, !{variable}, !DIExpression(), !{at})");
        }
    }

    /// The result's slot, when it has bytes.
    pub(crate) fn ret_slot(&self) -> Option<String> {
        self.addr.first().cloned().flatten()
    }

    /// A call of the runtime's `name()` in the entry block, before the body.
    fn prologue_rt(&mut self, name: &str) {
        self.in_prologue(|cx| cx.call_rt(name, Vec::new(), None));
    }

    /// What `write` emits, placed in the prologue.
    fn in_prologue(&mut self, write: impl FnOnce(&mut Self)) {
        let body = std::mem::take(&mut self.out);
        write(self);
        let code = std::mem::replace(&mut self.out, body);
        self.prologue.push_str(&code);
    }

    pub(crate) fn label(&mut self) -> String {
        self.next += 1;
        format!("l{}", self.next)
    }

    pub(crate) fn emit(&mut self, line: impl AsRef<str>) {
        self.out.push_str("  ");
        self.out.push_str(line.as_ref());
        // `[CG-LL-3]` — the Ember line and column of the statement the instruction is made from
        // (the function's own line where it has none).
        if let Some((scope, own)) = self.scope {
            let span = self.span;
            let (line, col) = match self.m.map.get(span.file) {
                Some(_) if !span.is_dummy() => {
                    let at = self.m.map.line_col(span);
                    (at.line, at.col)
                }
                _ => (own, 0),
            };
            let node = match self.locations.get(&(line, col)) {
                Some(node) => *node,
                None => {
                    let node = self.m.debug.location(line, col, scope);
                    self.locations.insert((line, col), node);
                    node
                }
            };
            let _ = write!(self.out, ", !dbg !{node}");
        }
        self.out.push('\n');
    }

    pub(crate) fn start(&mut self, label: &str) {
        let _ = writeln!(self.out, "{label}:");
        self.block = label.to_string();
    }

    /// A value of its own: `%t = <rhs>`.
    pub(crate) fn value(&mut self, rhs: impl AsRef<str>) -> String {
        let name = self.tmp();
        let line = format!("{name} = {}", rhs.as_ref());
        self.emit(line);
        name
    }

    pub(crate) fn alloca(&mut self, size: u64, align: u64) -> String {
        let name = self.tmp();
        let _ = writeln!(self.allocas, "  {name} = alloca [{} x i8], align {align}", size.max(1));
        name
    }

    pub(crate) fn gep(&mut self, ptr: &str, offset: u64) -> String {
        if offset == 0 {
            return ptr.to_string();
        }
        self.value(format!("getelementptr i8, ptr {ptr}, i64 {offset}"))
    }

    pub(crate) fn load(&mut self, sc: Sc, ptr: &str, align: u64) -> String {
        let loaded = self.value(format!("load {}, ptr {ptr}, align {align}", sc.mem()));
        if sc == Sc::Bool { self.value(format!("trunc i8 {loaded} to i1")) } else { loaded }
    }

    pub(crate) fn store_scalar(&mut self, sc: Sc, value: &str, ptr: &str, align: u64) {
        let value = if sc == Sc::Bool { self.value(format!("zext i1 {value} to i8")) } else { value.to_string() };
        self.emit(format!("store {} {value}, ptr {ptr}, align {align}", sc.mem()));
    }

    pub(crate) fn copy(&mut self, to: &str, to_align: u64, from: &str, from_align: u64, size: u64, overlap: bool) {
        if size == 0 {
            return;
        }
        let what = if overlap { "memmove" } else { "memcpy" };
        let intrinsic = format!("llvm.{what}.p0.p0.i64");
        self.m.declare(&intrinsic, || format!("declare void @{intrinsic}(ptr, ptr, i64, i1)"));
        self.emit(format!(
            "call void @{intrinsic}(ptr align {to_align} {to}, ptr align {from_align} {from}, i64 {size}, i1 false)"
        ));
    }

    pub(crate) fn zero(&mut self, to: &str, align: u64, size: u64) {
        if size == 0 {
            return;
        }
        let intrinsic = "llvm.memset.p0.i64";
        self.m.declare(intrinsic, || format!("declare void @{intrinsic}(ptr, i8, i64, i1)"));
        self.emit(format!("call void @{intrinsic}(ptr align {align} {to}, i8 0, i64 {size}, i1 false)"));
    }

    /// `val` written to `ptr`, a place of `size` bytes.
    pub(crate) fn store(&mut self, val: &Val, ptr: &str, align: u64, size: u64, overlap: bool) {
        match val {
            Val::Zst => {}
            Val::Imm(sc, value) => {
                let (sc, value) = (*sc, value.clone());
                self.store_scalar(sc, &value, ptr, align);
            }
            Val::Mem(from, from_align) => {
                let (from, from_align) = (from.clone(), *from_align);
                if from != ptr {
                    self.copy(ptr, align, &from, from_align, size, overlap);
                }
            }
        }
    }

    /// `val`'s bytes in memory: its own when it has them, else a slot it is stored to.
    pub(crate) fn spill(&mut self, val: &Val, size: u64, align: u64) -> (String, u64) {
        match val {
            Val::Mem(ptr, align) => (ptr.clone(), *align),
            Val::Imm(sc, value) => {
                let slot = self.alloca(size, align);
                let (sc, value) = (*sc, value.clone());
                self.store_scalar(sc, &value, &slot, align);
                (slot, align)
            }
            Val::Zst => (self.alloca(1, 1), 1),
        }
    }

    // -- the frame --------------------------------------------------------------------------------

    fn mir(&self) -> &'a Body {
        self.body.expect("a function made from a MIR body")
    }

    fn frame(&mut self) -> Result<()> {
        let body = self.mir();
        let mut param = 0usize;
        if let Return::Sret { .. } = self.sig.ret {
            self.addr[0] = Some("%p0".to_string());
            param = 1;
        }
        let passes = self.sig.params.clone();
        let values = ffi_value_params(self.m, body);
        for (position, ((local, decl), pass)) in body.args().zip(passes).enumerate() {
            let index = local.0 as usize;
            match pass {
                Pass::Ignore => {}
                Pass::Indirect { .. } => {
                    self.addr[index] = Some(format!("%p{param}"));
                    param += 1;
                }
                Pass::Direct(parts) => {
                    let (size, align) = self.m.ty.size_align(passed_ty(self.m, &values, position, decl.ty));
                    let slot = self.alloca(size.max(parts_span(&parts)), align);
                    for part in parts {
                        let at = if part.offset == 0 { slot.clone() } else {
                            let at = self.tmp();
                            let _ = writeln!(self.prologue, "  {at} = getelementptr i8, ptr {slot}, i64 {}", part.offset);
                            at
                        };
                        let value = format!("%p{param}");
                        let part_align = align.min(offset_align(part.offset));
                        if part.ty == PartTy::I(1) {
                            let wide = self.tmp();
                            let _ = writeln!(self.prologue, "  {wide} = zext i1 {value} to i8");
                            let _ = writeln!(self.prologue, "  store i8 {wide}, ptr {at}, align {part_align}");
                        } else {
                            let _ = writeln!(self.prologue, "  store {} {value}, ptr {at}, align {part_align}", part.ty);
                        }
                        param += 1;
                    }
                    self.addr[index] = Some(slot);
                }
            }
            // The body borrows the value C passed: its reference is the value's address.
            if values.get(position) == Some(&true)
                && let Some(value) = self.addr[index].take()
            {
                let holder = self.tmp();
                let _ = writeln!(self.allocas, "  {holder} = alloca ptr, align 8");
                let _ = writeln!(self.prologue, "  store ptr {value}, ptr {holder}, align 8");
                self.addr[index] = Some(holder);
            }
        }
        // `[FFI-22]` — a host may call an export on a thread that never ran Ember: it attaches
        // before the body touches the runtime; `[FFI-33]` — a `main`-thread contract is checked.
        if body.abi.is_some() {
            self.prologue_rt("rt_thread_attach");
            if body.export_thread_policy != ember_mir::ExportThreadPolicy::Any {
                if self.m.module {
                    // `[FFI-33]` — a shared library's contract is its own: the thread of its
                    // successful `ember_module_init`, even in a runtime other modules share.
                    let creator = module_creator(self.m);
                    self.in_prologue(|cx| {
                        let id = cx.load(Sc::I(32), &creator, 4);
                        cx.call_rt("rt_check_module_thread", vec![Val::Imm(Sc::I(32), id)], None);
                    });
                } else {
                    self.prologue_rt("rt_check_main_thread");
                }
            }
            // `[HR-42]` — entering Ember code from a foreign caller: this thread's depth counter.
            if self.m.reload.is_some() {
                self.prologue_rt("reload_enter");
            }
        }
        // ADR-141 — each list's frame buffer, aligned for any element it may hold.
        for index in 0..body.stack_buffers.len() {
            let _ = writeln!(self.allocas, "  %sb{index} = alloca [64 x i8], align 16");
        }
        // `[OPT-1]` — each class object made in the frame: the whole object, header and all.
        for local in &body.frame_objects {
            if let TyKind::Class(class) = *self.m.ty.kind(body.local(*local).ty) {
                let layout = crate::glue::object_layout(self.m, class);
                let _ = writeln!(self.allocas, "  %fo{} = alloca [{} x i8], align {}", local.0, layout.size, layout.align);
            }
        }
        for (index, decl) in body.locals.iter().enumerate() {
            // A folded literal's temporary has no slot: it is written in its reader's field.
            if self.addr[index].is_some() || (index >= 1 && index <= body.arg_count) || self.folded.contains_key(&LocalId(index as u32)) {
                continue;
            }
            match self.m.ty.repr(decl.ty) {
                Repr::Zst => {}
                Repr::Scalar(sc) => {
                    let (_, align) = self.m.ty.size_align(decl.ty);
                    let name = self.tmp();
                    let _ = writeln!(self.allocas, "  {name} = alloca {}, align {}", sc.mem(), align.max(1));
                    self.addr[index] = Some(name);
                }
                Repr::Mem { size, align } => {
                    // The result's slot holds the parts it is returned in (`ret`).
                    let size = match &self.sig.ret {
                        Return::Direct(parts) if index == 0 => size.max(parts_span(parts)),
                        _ => size,
                    };
                    let slot = self.alloca(size, align);
                    self.addr[index] = Some(slot);
                }
            }
        }
        Ok(())
    }

    // -- places -----------------------------------------------------------------------------------

    fn local_ty(&self, local: LocalId) -> Ty {
        self.mir().local(local).ty
    }

    fn natural_align(&self, ty: Ty) -> u64 {
        self.m.ty.size_align(ty).1
    }

    /// The address of `place`.
    pub(crate) fn place(&mut self, place: &Place) -> Result<PlaceRef> {
        let ty = self.local_ty(place.local);
        let ptr = match &self.addr[place.local.0 as usize] {
            Some(ptr) => ptr.clone(),
            None => self.void_place(),
        };
        let mut at = PlaceRef { ptr, ty, variant: None, align: self.natural_align(ty) };
        for projection in &place.projection {
            at = self.project(at, projection)?;
        }
        Ok(at)
    }

    /// The address a borrow of a place with no bytes takes (ADR-062): one byte the program
    /// keeps for them all.
    fn void_place(&mut self) -> String {
        let name = "@ember.void_place";
        self.m.declare(name, || format!("{name} = internal global i8 0, align 1"));
        name.to_string()
    }

    fn field(&mut self, at: &PlaceRef, offset: u64, ty: Ty) -> PlaceRef {
        let ptr = self.gep(&at.ptr, offset);
        let align = at.align.min(offset_align(offset)).min(self.natural_align(ty));
        PlaceRef { ptr, ty, variant: None, align }
    }

    fn project(&mut self, at: PlaceRef, projection: &Projection) -> Result<PlaceRef> {
        let types = &self.m.ty;
        let kind = types.kind(at.ty).clone();
        Ok(match (projection, kind) {
            (Projection::Downcast(variant), TyKind::Enum(_)) => PlaceRef { variant: Some(*variant), ..at },
            (Projection::Field(index), TyKind::Enum(id)) => {
                let Some(variant) = at.variant else {
                    return unsupported("a field of an enum read without its variant");
                };
                let table = self.m.ty.table;
                let field_ty = table.enum_def(id).variants[variant].fields[*index].ty;
                if table.option_niche(id).is_some() {
                    // `Some`'s payload is the whole value.
                    return Ok(PlaceRef { ptr: at.ptr, ty: field_ty, variant: None, align: at.align });
                }
                let offset = table.variant_field_offsets(id)[variant][*index];
                self.field(&at, offset, field_ty)
            }
            (Projection::Field(index), TyKind::Struct(id)) => {
                let table = self.m.ty.table;
                if let Some(inner) = shared_inner(table, id) {
                    // `[HEAP-5]` — a `Shared`'s payload follows its object's header, at the offset
                    // its alignment gives (`ember_obj_payload`): the field is that address.
                    let field_ty = table.struct_def(id).fields[*index].ty;
                    let header = self.load(Sc::Ptr, &at.ptr, at.align.min(8));
                    let (_, align) = self.m.ty.size_align(inner);
                    let payload = self.gep(&header, self.m.ty.payload_offset(align));
                    let slot = self.alloca(8, 8);
                    self.emit(format!("store ptr {payload}, ptr {slot}, align 8"));
                    return Ok(PlaceRef { ptr: slot, ty: field_ty, variant: None, align: 8 });
                }
                let field_ty = table.struct_def(id).fields[*index].ty;
                let offset = table.layout(at.ty).field_offsets[*index];
                self.field(&at, offset, field_ty)
            }
            (Projection::Field(index), TyKind::Tuple(items)) => {
                let offset = self.m.ty.table.layout(at.ty).field_offsets[*index];
                self.field(&at, offset, items[*index])
            }
            (Projection::Field(index), TyKind::Class(id)) => {
                let table = self.m.ty.table;
                let Some(field) = table.class_field_at(id, *index) else {
                    return unsupported("a class field past the object's");
                };
                let field_ty = field.ty;
                // The object: after the handle's pointer, the runtime's header, then the fields.
                let object = self.load(Sc::Ptr, &at.ptr, at.align.min(8));
                let offset = self.m.ty.object_field_offsets(id)[*index];
                let ptr = self.gep(&object, offset);
                PlaceRef { ptr, ty: field_ty, variant: None, align: self.natural_align(field_ty).min(offset_align(offset).min(16)) }
            }
            (Projection::Field(index), TyKind::Str | TyKind::Vec { .. } | TyKind::Span { .. }) => {
                let field_ty = if *index == 0 { at.ty } else { self.m.ty.usize_ty };
                let ptr = self.gep(&at.ptr, 8 * *index as u64);
                // `.ptr` keeps the view's own type; only the words are read through it.
                PlaceRef { ptr, ty: field_ty, variant: None, align: at.align.min(8) }
            }
            (Projection::Index(_) | Projection::ConstIndex(_), TyKind::Array { elem, .. }) => {
                let (size, elem_align) = self.m.ty.size_align(elem);
                let ptr = self.index_ptr(projection, &at.ptr, size)?;
                PlaceRef { ptr, ty: elem, variant: None, align: at.align.min(elem_align) }
            }
            (Projection::Index(_) | Projection::ConstIndex(_), TyKind::Vec { elem, .. } | TyKind::Span { elem, .. }) => {
                let (size, elem_align) = self.m.ty.size_align(elem);
                let buffer = self.load(Sc::Ptr, &at.ptr, at.align.min(8));
                let ptr = self.index_ptr(projection, &buffer, size)?;
                PlaceRef { ptr, ty: elem, variant: None, align: elem_align }
            }
            (Projection::Index(_) | Projection::ConstIndex(_), TyKind::Ptr { inner, .. }) => {
                let (size, inner_align) = self.m.ty.size_align(inner);
                let base = self.load(Sc::Ptr, &at.ptr, at.align.min(8));
                let ptr = self.index_ptr(projection, &base, size)?;
                PlaceRef { ptr, ty: inner, variant: None, align: inner_align }
            }
            (Projection::Deref, TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. }) => {
                if matches!(self.m.ty.kind(inner), TyKind::Dyn { .. }) {
                    return unsupported("a `dyn` value read through its reference");
                }
                let ptr = self.load(Sc::Ptr, &at.ptr, at.align.min(8));
                PlaceRef { ptr, ty: inner, variant: None, align: self.natural_align(inner) }
            }
            (Projection::Column(_), _) => return unsupported("an `SoA` column"),
            _ => return unsupported(format!("the projection {projection:?} of `{}`", self.m.ty.table.display(at.ty))),
        })
    }

    /// The address of an index's element from `base`. `index * size` cannot overflow, since the
    /// index was checked against a length (or proved within one) and the elements fit in memory.
    fn index_ptr(&mut self, projection: &Projection, base: &str, size: u64) -> Result<String> {
        Ok(match projection {
            Projection::ConstIndex(index) => self.value(format!("getelementptr i8, ptr {base}, i64 {}", index * size)),
            Projection::Index(local) => {
                let local = *local;
                let (index, sc) = self.read_scalar(&Place::local(local))?;
                let index = match sc {
                    Sc::I(64) => index,
                    Sc::I(bits) if bits < 64 => self.value(format!("zext i{bits} {index} to i64")),
                    Sc::I(bits) => self.value(format!("trunc i{bits} {index} to i64")),
                    other => return unsupported(format!("an index of type {}", other.ir())),
                };
                self.element_at(base, &index, size)
            }
            _ => unreachable!("an index projection"),
        })
    }

    /// Element `index` of `size`-byte elements from `base`, as one scaled `getelementptr` (no
    /// `inbounds`, `[CG-LL-2]`). LLVM's cost models count it as one instruction where a `mul`
    /// and a byte `getelementptr` are two, so a loop over elements unrolls as C's does.
    pub(crate) fn element_at(&mut self, base: &str, index: &str, size: u64) -> String {
        self.value(format!("getelementptr [{size} x i8], ptr {base}, i64 {index}"))
    }

    pub(crate) fn place_ty(&self, place: &Place) -> Ty {
        place_type(self.m, self.mir(), place)
    }

    /// The object a class handle, an interface handle or a `Shared` at `place` points at.
    fn object_of(&mut self, place: &Place) -> Result<String> {
        let at = self.place(place)?;
        Ok(self.load(Sc::Ptr, &at.ptr, at.align.min(8)))
    }

    fn read_scalar(&mut self, place: &Place) -> Result<(String, Sc)> {
        let at = self.place(place)?;
        match self.m.ty.repr(at.ty) {
            Repr::Scalar(sc) => Ok((self.load(sc, &at.ptr, at.align), sc)),
            _ => unsupported(format!("a scalar read of `{}`", self.m.ty.table.display(at.ty))),
        }
    }

    // -- operands ---------------------------------------------------------------------------------

    pub(crate) fn operand_ty(&self, operand: &Operand) -> Option<Ty> {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => Some(self.place_ty(place)),
            Operand::Const(Const::Int { ty, .. } | Const::Float { ty, .. } | Const::SourceLine { ty, .. }) => Some(*ty),
            _ => None,
        }
    }

    pub(crate) fn operand(&mut self, operand: &Operand) -> Result<Val> {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => {
                let at = self.place(place)?;
                Ok(match self.m.ty.repr(at.ty) {
                    Repr::Zst => Val::Zst,
                    Repr::Scalar(sc) => Val::Imm(sc, self.load(sc, &at.ptr, at.align)),
                    Repr::Mem { .. } => Val::Mem(at.ptr, at.align),
                })
            }
            Operand::Const(constant) => self.constant(constant),
        }
    }

    /// A scalar operand's register value.
    pub(crate) fn scalar(&mut self, operand: &Operand) -> Result<(String, Sc)> {
        match self.operand(operand)? {
            Val::Imm(sc, value) => Ok((value, sc)),
            // An enum without payloads that `@align` makes wide is its discriminant (`[LAY-2]`).
            Val::Mem(ptr, align)
                if self.operand_ty(operand).is_some_and(|ty| matches!(self.m.ty.kind(ty), TyKind::Enum(id) if self.m.ty.table.enum_def(*id).is_unit_only())) =>
            {
                let ty = self.operand_ty(operand).expect("checked above");
                let (value, bits) = self.discriminant_at(&PlaceRef { ptr, ty, variant: None, align })?;
                Ok((value, Sc::I(bits)))
            }
            other => unsupported(format!("a scalar operand held as {other:?}")),
        }
    }

    fn constant(&mut self, constant: &Const) -> Result<Val> {
        Ok(match constant {
            // `[CELL-5]` — the line `at` begins on, as it is now (a body an incremental build kept
            // names its line so, `[BLD-8]`).
            Const::SourceLine { at, ty } => {
                let line = self.m.map.get(at.file).map_or(0, |file| file.line_col(at.start).line);
                return self.constant(&Const::Int { value: u128::from(line), ty: *ty });
            }
            Const::Int { value, ty } => match self.m.ty.repr(*ty) {
                Repr::Scalar(Sc::Bool) => Val::Imm(Sc::Bool, if *value != 0 { "true" } else { "false" }.to_string()),
                Repr::Scalar(Sc::I(bits)) => Val::Imm(Sc::I(bits), int_literal(*value, bits)),
                Repr::Scalar(sc @ (Sc::F32 | Sc::F64)) => Val::Imm(sc, float_literal(*value as f64, sc)),
                Repr::Scalar(Sc::Ptr) => Val::Imm(Sc::Ptr, if *value == 0 { "null".to_string() } else {
                    format!("inttoptr (i64 {} to ptr)", int_literal(*value, 64))
                }),
                Repr::Zst => Val::Zst,
                other => return unsupported(format!("an integer constant held as {other:?}")),
            },
            Const::Float { value, ty } => match self.m.ty.repr(*ty) {
                Repr::Scalar(Sc::I(16)) if self.m.ty.is_f16(*ty) => {
                    Val::Imm(Sc::I(16), int_literal(ember_types::f16_bits(*value) as u128, 16))
                }
                Repr::Scalar(sc @ (Sc::F32 | Sc::F64)) => Val::Imm(sc, float_literal(*value, sc)),
                other => return unsupported(format!("a float constant held as {other:?}")),
            },
            Const::Bool(value) => Val::Imm(Sc::Bool, value.to_string()),
            Const::Str(text) => {
                let bytes = self.m.bytes(text.as_bytes());
                let slot = self.alloca(16, 8);
                self.emit(format!("store ptr {bytes}, ptr {slot}, align 8"));
                let len = self.gep(&slot, 8);
                self.emit(format!("store i64 {}, ptr {len}, align 8", text.len()));
                Val::Mem(slot, 8)
            }
            Const::CStr(text) => Val::Imm(Sc::Ptr, self.m.bytes(text.as_bytes())),
            Const::CStrLiteral(bytes) => {
                // The literal's bytes as written; the constant's own NUL follows them.
                let bytes = bytes.strip_suffix(&[0]).unwrap_or(bytes);
                Val::Imm(Sc::Ptr, self.m.bytes(bytes))
            }
            Const::Void => Val::Zst,
            Const::OverflowShift { per_word } => {
                // `EMBER_OVERFLOW_SHIFT` is 63 for Clang (ADR-070 item 7).
                Val::Imm(Sc::I(64), if *per_word { "63" } else { "0" }.to_string())
            }
            Const::Fn(symbol) => {
                // `[FN-6]` — a native value of the function; one a C pointer holds is its address.
                if let Some(signature) = self.m.callables.natives.get(symbol.as_str()).copied() {
                    if let Some(value) = crate::reload::fn_value(self, symbol) {
                        return Ok(value);
                    }
                    return self.native_fn_value(symbol, signature);
                }
                let symbol = symbol.clone();
                if self.m.bodies.get(symbol.as_str()).is_some_and(|body| is_vectorcall(body)) {
                    return unsupported("a `__vectorcall` function as a value ([FFI-9])");
                }
                self.declare_body(&symbol)?;
                Val::Imm(Sc::Ptr, global_name(&symbol))
            }
        })
    }

    // -- statements -------------------------------------------------------------------------------

    /// A block's statements: `[EXC-19]` — an access begun and ended back to back is one check,
    /// since nothing between could observe a count.
    fn stmts(&mut self, stmts: &[ember_mir::Stmt]) -> Result<()> {
        let mut index = 0;
        while index < stmts.len() {
            let stmt = &stmts[index];
            if let StmtKind::BeginAccess { place, mutable } = &stmt.kind
                && let Some(next) = stmts.get(index + 1)
                && matches!(&next.kind, StmtKind::EndAccess { place: end, mutable: end_mutable } if end == place && end_mutable == mutable)
                && self.access_check(place, *mutable, stmt.span)?
            {
                index += 2;
                continue;
            }
            self.stmt(&stmt.kind, stmt.span)?;
            index += 1;
        }
        Ok(())
    }

    fn stmt(&mut self, kind: &StmtKind, span: ember_span::Span) -> Result<()> {
        self.span = span;
        match kind {
            StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => Ok(()),
            // `[GRM-39]` — a folded literal is written where the next literal reads it.
            StmtKind::Assign { place, .. } if place.projection.is_empty() && self.folded.contains_key(&place.local) => Ok(()),
            StmtKind::Assign { place, rvalue } => self.assign(place, rvalue, span),
            StmtKind::CheckedBinaryOp { dest, overflow, op, lhs, rhs } => self.checked(dest, overflow, *op, lhs, rhs),
            StmtKind::Drop { place, flag, .. } => self.drop_stmt(place, *flag),
            StmtKind::BeginAccess { place, mutable } => {
                let operation = if *mutable { "begin_write" } else { "begin_read" };
                self.access(place, operation, span)
            }
            StmtKind::EndAccess { place, mutable } => {
                let operation = if *mutable { "end_write" } else { "end_read" };
                self.access(place, operation, span)
            }
            StmtKind::BeginAccessTransfer { place, mutable } => {
                let operation = if *mutable { "access_begin_write" } else { "access_begin_read" };
                let object = self.object_of(place)?;
                let what = self.m.ty.table.display(self.place_ty(place));
                let what = self.m.bytes(what.as_bytes());
                let loc = self.loc(span);
                self.call_rt(operation, vec![Val::Imm(Sc::Ptr, object), Val::Imm(Sc::Ptr, what), loc], None);
                Ok(())
            }
            StmtKind::EndAccessTransfer { place, mutable } => {
                // The object's header, at the payload offset before the payload the place names.
                let operation = if *mutable { "access_end_write" } else { "access_end_read" };
                let at = self.place(place)?;
                let (_, align) = self.m.ty.size_align(at.ty);
                let offset = self.m.ty.payload_offset(align);
                let object = self.value(format!("getelementptr i8, ptr {}, i64 -{offset}", at.ptr));
                let what = self.mir().symbol.clone();
                let what = self.m.bytes(what.as_bytes());
                let loc = self.loc(span);
                self.call_rt(operation, vec![Val::Imm(Sc::Ptr, object), Val::Imm(Sc::Ptr, what), loc], None);
                Ok(())
            }
            // `[EXC-5]` (D-690) — a `mut self` method lends its object for a call.
            StmtKind::LendAccess { place, lend } => {
                let (end, begin) = if *lend { ("end_write", "begin_read") } else { ("end_read", "begin_write") };
                self.access(place, end, span)?;
                self.access(place, begin, span)
            }
        }
    }

    /// `[OWN-2]`, `[DRP-2]` — the value's life ends: behind its flag when it was moved on some paths
    /// (`[OWN-3]`); never for a handle that borrows a list's count (`[RC-3]`); not for a list
    /// still in its frame buffer (ADR-141).
    fn drop_stmt(&mut self, place: &Place, flag: Option<LocalId>) -> Result<()> {
        let body = self.mir();
        if place.projection.is_empty() && body.uncounted_handles.contains(&place.local) {
            return Ok(());
        }
        let ty = self.place_ty(place);
        if !self.m.ty.table.needs_drop(ty) {
            return Ok(());
        }
        let done = self.label();
        let mut branched = false;
        if let Some(flag) = flag {
            let (set, _) = self.read_scalar(&Place::local(flag))?;
            let go = self.label();
            self.emit(format!("br i1 {set}, label %{go}, label %{done}"));
            self.start(&go);
            branched = true;
        }
        if place.projection.is_empty()
            && let Some(buffer) = self.stack_buffer(place.local)
        {
            let at = self.place(place)?;
            let held = self.load(Sc::Ptr, &at.ptr, 8);
            let heap = self.value(format!("icmp ne ptr {held}, {buffer}"));
            let go = self.label();
            self.emit(format!("br i1 {heap}, label %{go}, label %{done}"));
            self.start(&go);
            branched = true;
        }
        let at = self.place(place)?;
        let align = if ember_mir::unaligned_place(self.m.ty.table, body, place) { 1 } else { at.align };
        self.drop_at(&at.ptr, align, ty)?;
        if branched {
            self.emit(format!("br label %{done}"));
            self.start(&done);
        }
        Ok(())
    }

    /// The frame buffer of a list local (ADR-141), if it has one.
    fn stack_buffer(&self, local: LocalId) -> Option<String> {
        self.body?.stack_buffers.iter().position(|buffer| buffer.local == local).map(|index| format!("%sb{index}"))
    }

    /// The runtime's operation on an access to `place` (`[EXC-1]`): a class object's every word
    /// (`[EXC-15]`), a class field's own word (`[EXC-19]`), or a `Shared` payload's header word
    /// (`[HEAP-5]`).
    fn access(&mut self, place: &Place, operation: &str, span: ember_span::Span) -> Result<()> {
        let ty = self.place_ty(place);
        // `[EXC-18]` — a `dyn` receiver's object, through the table's `access` entry. D-913: one a
        // class field holds is named with a `Deref` after the field, whose own place is its access
        // word (`[EXC-19]`), which a store into the field takes.
        let (carrier, behind) = match place.projection.split_last() {
            Some((Projection::Deref, prefix)) if self.is_dyn_place(self.place_ty(&Place { local: place.local, projection: prefix.to_vec() })) => {
                (Place { local: place.local, projection: prefix.to_vec() }, true)
            }
            _ => (place.clone(), false),
        };
        if behind || (self.is_dyn_place(ty) && !self.has_field_word(place)) {
            let code = match operation {
                "begin_read" => 0,
                "end_read" => 1,
                "begin_write" => 2,
                _ => 3,
            };
            // A `ref dyn I`, a `dyn I` and a `Box[dyn I]` are each the carrier itself.
            let at = self.place(&carrier)?;
            let loc = self.loc(span);
            self.dyn_access(&at, code, loc);
            return Ok(());
        }
        let loc = self.loc(span);
        if matches!(self.m.ty.kind(ty), TyKind::Class(_) | TyKind::ClassInterface(_)) {
            let object = self.object_of(place)?;
            self.call_rt(&format!("object_{operation}"), vec![Val::Imm(Sc::Ptr, object), loc], None);
            return Ok(());
        }
        if let Some((word, what)) = self.field_word(place)? {
            let helper = crate::glue::field_helper(self.m, operation);
            let what = self.m.bytes(what.as_bytes());
            let (loc_ptr, _) = self.spill(&loc, 16, 8);
            self.emit(format!("call void {helper}(ptr {word}, ptr {what}, ptr {loc_ptr})"));
            return Ok(());
        }
        let object = self.object_of(place)?;
        let what = self.m.ty.table.display(ty);
        let what = self.m.bytes(what.as_bytes());
        self.call_rt(&format!("access_{operation}"), vec![Val::Imm(Sc::Ptr, object), Val::Imm(Sc::Ptr, what), loc], None);
        Ok(())
    }

    /// `[EXC-19]` — an access nothing can overlap, as one check; `false` when `place` is neither
    /// a class field with its own word nor a class object.
    fn access_check(&mut self, place: &Place, mutable: bool, span: ember_span::Span) -> Result<bool> {
        let ty = self.place_ty(place);
        // A class field with its own word is checked there, a `dyn` one too, as the C backend's
        // `field_check_call` does: the pair brackets a store into the field (D-911: through the
        // table it read the box the store had not yet written).
        let operation = if mutable { "check_write" } else { "check_read" };
        if let Some((word, what)) = self.field_word(place)? {
            let loc = self.loc(span);
            let helper = crate::glue::field_helper(self.m, operation);
            let what = self.m.bytes(what.as_bytes());
            let (loc_ptr, _) = self.spill(&loc, 16, 8);
            self.emit(format!("call void {helper}(ptr {word}, ptr {what}, ptr {loc_ptr})"));
            return Ok(true);
        }
        if !matches!(self.m.ty.kind(ty), TyKind::Class(_) | TyKind::ClassInterface(_)) {
            return Ok(false);
        }
        let object = self.object_of(place)?;
        let loc = self.loc(span);
        if let TyKind::Class(id) = *self.m.ty.kind(ty)
            && self.m.ty.table.class_def(id).openness == ember_types::ClassOpenness::Final
        {
            // A final class's words are known: read in place, two at a time.
            let layout = crate::glue::object_layout(self.m, id);
            // An odd level's last read takes its zero pad word too.
            let pairs: Vec<u64> = layout.levels.iter().flat_map(|words| words.iter().step_by(2).copied()).collect();
            if pairs.is_empty() {
                return Ok(true);
            }
            let mut all = String::new();
            for offset in pairs {
                let at = self.gep(&object, offset);
                let align = offset_align(offset).min(4);
                let pair = self.value(format!("load i64, ptr {at}, align {align}"));
                all = if all.is_empty() { pair } else { self.value(format!("or i64 {all}, {pair}")) };
            }
            let test = if mutable {
                self.value(format!("icmp ne i64 {all}, 0"))
            } else {
                let writers = self.value(format!("and i64 {all}, -9223372034707292160"));
                self.value(format!("icmp ne i64 {writers}, 0"))
            };
            let (fail, ok) = (self.label(), self.label());
            self.emit(format!("br i1 {test}, label %{fail}, label %{ok}"));
            self.start(&fail);
            self.call_rt("object_check_failed", vec![Val::Imm(Sc::Ptr, object), Val::Imm(Sc::Bool, mutable.to_string()), loc], None);
            self.start(&ok);
            return Ok(true);
        }
        self.call_rt(&format!("object_{operation}"), vec![Val::Imm(Sc::Ptr, object), loc], None);
        Ok(true)
    }

    fn is_dyn_place(&self, ty: Ty) -> bool {
        let table = self.m.ty.table;
        match table.kind(ty) {
            TyKind::Dyn { .. } => true,
            TyKind::Ref { inner, .. } => matches!(table.kind(*inner), TyKind::Dyn { .. }),
            TyKind::Struct(id) => table.compiler_box_inner(*id).is_some_and(|inner| matches!(table.kind(inner), TyKind::Dyn { .. })),
            _ => false,
        }
    }

    /// Whether `place` is a class handle's field with its own access word (`h.f`).
    fn has_field_word(&self, place: &Place) -> bool {
        let Some((Projection::Field(index), prefix)) = place.projection.split_last() else { return false };
        let handle = Place { local: place.local, projection: prefix.to_vec() };
        matches!(*self.m.ty.kind(self.place_ty(&handle)), TyKind::Class(id) if self.m.ty.table.class_field_has_access_word(id, *index))
    }

    /// When `place` is a class handle's field with its own access word (`h.f`), the word's address
    /// and what a conflict names (`Bag.items`).
    fn field_word(&mut self, place: &Place) -> Result<Option<(String, String)>> {
        let Some((Projection::Field(index), prefix)) = place.projection.split_last() else { return Ok(None) };
        let handle = Place { local: place.local, projection: prefix.to_vec() };
        let TyKind::Class(id) = *self.m.ty.kind(self.place_ty(&handle)) else { return Ok(None) };
        let table = self.m.ty.table;
        if !table.class_field_has_access_word(id, *index) {
            return Ok(None);
        }
        let layout = crate::glue::object_layout(self.m, id);
        let Some(offset) = layout.words[*index] else { return Ok(None) };
        let field = table.class_field_at(id, *index).expect("a class field").name.to_string();
        let what = format!("{}.{field}", table.class_display_name(id));
        let object = self.read_scalar(&handle)?.0;
        let word = self.gep(&object, offset);
        Ok(Some((word, what)))
    }

    fn assign(&mut self, place: &Place, rvalue: &Rvalue, span: ember_span::Span) -> Result<()> {
        let ty = self.place_ty(place);
        if self.m.ty.is_zst(ty) {
            return Ok(());
        }
        if let Rvalue::Use(Operand::Copy(from) | Operand::Move(from)) = rvalue
            && from == place
            && !self.holds_counts(ty)
        {
            return Ok(());
        }
        let _ = span;
        // `[OWN-7]` — a copy retains what it holds before the new value becomes visible; an
        // uncounted handle borrows the list's count (`[RC-3]`).
        let counted = !(place.projection.is_empty() && self.mir().uncounted_handles.contains(&place.local));
        if counted {
            self.retain_rvalue(rvalue, ty)?;
        }
        match rvalue {
            Rvalue::Aggregate { kind, operands } => return self.aggregate(place, ty, kind, operands),
            Rvalue::Repeat { value, count } => return self.repeat(place, ty, value, *count),
            _ => {}
        }
        let value = self.rvalue(rvalue, ty)?;
        let at = self.place(place)?;
        let size = self.m.ty.size_align(ty).0;
        let overlap = match rvalue {
            Rvalue::Use(Operand::Copy(from) | Operand::Move(from)) => may_overlap(place, from),
            _ => false,
        };
        self.store(&value, &at.ptr, at.align, size, overlap);
        Ok(())
    }

    /// `[OWN-7]` — the retains a copy made by `rvalue` takes: each `Copy` operand's handles.
    fn retain_rvalue(&mut self, rvalue: &Rvalue, target: Ty) -> Result<()> {
        match rvalue {
            Rvalue::Use(Operand::Copy(from)) => {
                if self.holds_counts(target) {
                    let at = self.place(from)?;
                    self.retain_lines(&at.ptr, at.align, target)?;
                }
            }
            Rvalue::Aggregate { kind, operands } => {
                let table = self.m.ty.table;
                let types: Vec<Ty> = match kind {
                    AggregateKind::Callable => operands.iter().map(|operand| self.operand_ty(operand).unwrap_or(target)).collect(),
                    AggregateKind::Struct(id) => table.struct_def(*id).fields.iter().map(|field| field.ty).collect(),
                    AggregateKind::Tuple => match table.kind(target) {
                        TyKind::Tuple(items) => items.clone(),
                        _ => Vec::new(),
                    },
                    AggregateKind::Array => match table.kind(target) {
                        TyKind::Array { elem, len } => vec![*elem; *len as usize],
                        _ => Vec::new(),
                    },
                    AggregateKind::Enum(id, variant) => {
                        table.enum_def(*id).variants.get(*variant).map(|v| v.fields.iter().map(|f| f.ty).collect()).unwrap_or_default()
                    }
                };
                for (operand, ty) in operands.iter().zip(types) {
                    if let Operand::Copy(from) = operand
                        && self.holds_counts(ty)
                    {
                        let at = self.place(from)?;
                        self.retain_lines(&at.ptr, at.align, ty)?;
                    }
                }
            }
            Rvalue::Cast { kind: CastKind::ClassUpcast | CastKind::ClassInterfaceUpcast { .. }, operand: Operand::Copy(from), .. } => {
                let ty = self.place_ty(from);
                let (handle, _) = self.read_scalar(from)?;
                self.retain_handle(&handle, ty);
            }
            _ => {}
        }
        Ok(())
    }

    fn aggregate(&mut self, place: &Place, ty: Ty, kind: &AggregateKind, operands: &[Operand]) -> Result<()> {
        let values = self.aggregate_values(Some(place), operands)?;
        let at = self.place(place)?;
        self.write_aggregate(&at, ty, kind, operands, values)
    }

    /// A literal's operands, every one read before the first field is written (one may read the
    /// place); a folded literal's temporary (`[GRM-39]`) is written in its field's place instead.
    fn aggregate_values(&mut self, place: Option<&Place>, operands: &[Operand]) -> Result<Vec<Field>> {
        let mut values = Vec::with_capacity(operands.len());
        for operand in operands {
            if let Operand::Copy(read) | Operand::Move(read) = operand
                && read.projection.is_empty()
                && self.folded.contains_key(&read.local)
            {
                values.push(Field::Folded(read.local));
                continue;
            }
            let mut value = self.operand(operand)?;
            if let (Val::Mem(ptr, align), Operand::Copy(from) | Operand::Move(from), Some(place)) = (&value, operand, place)
                && may_overlap(place, from)
            {
                let ty = self.place_ty(from);
                let (size, _) = self.m.ty.size_align(ty);
                let slot = self.alloca(size, *align);
                let (ptr, align) = (ptr.clone(), *align);
                self.copy(&slot, align, &ptr, align, size, false);
                value = Val::Mem(slot, align);
            }
            values.push(Field::Val(value));
        }
        Ok(values)
    }

    /// A literal of `ty` written at `at`, a field at a time.
    fn write_aggregate(&mut self, at: &PlaceRef, ty: Ty, kind: &AggregateKind, operands: &[Operand], values: Vec<Field>) -> Result<()> {
        let at = at.clone();
        let mut values = values;
        let table = self.m.ty.table;
        let fields: Vec<(u64, Ty)> = match (kind, table.kind(ty).clone()) {
            (AggregateKind::Struct(id), _) => {
                if shared_inner(table, *id).is_some() {
                    return unsupported("a `Shared` value built field by field");
                }
                let offsets = table.layout(ty).field_offsets;
                table.struct_def(*id).fields.iter().zip(offsets).map(|(field, offset)| (offset, field.ty)).collect()
            }
            (AggregateKind::Tuple, TyKind::Tuple(items)) => {
                table.layout(ty).field_offsets.into_iter().zip(items.iter().copied()).collect()
            }
            // A view made of its words: the pointer, the length (and a list's capacity).
            (AggregateKind::Tuple, TyKind::Span { .. } | TyKind::Str | TyKind::Vec { .. }) => {
                (0..values.len() as u64).map(|i| (i * 8, self.m.ty.usize_ty)).collect()
            }
            (AggregateKind::Array, TyKind::Array { elem, .. }) => {
                let size = self.m.ty.size_align(elem).0;
                (0..values.len() as u64).map(|i| (i * size, elem)).collect()
            }
            (AggregateKind::Enum(id, variant), _) => {
                let def = table.enum_def(*id);
                if let Some(niche) = table.option_niche(*id) {
                    if *variant == niche.some {
                        let Some(Field::Val(value)) = values.pop() else { return unsupported("a niche `Some` of a folded literal") };
                        let size = self.m.ty.size_align(niche.payload).0;
                        self.store(&value, &at.ptr, at.align, size, false);
                    } else {
                        self.niche_none(&at, niche.payload)?;
                    }
                    return Ok(());
                }
                let tag = def.variants[*variant].discriminant;
                let repr = def.repr;
                let Repr::Scalar(sc) = self.m.ty.repr(repr) else {
                    return unsupported("an enum whose tag is not an integer");
                };
                let bits = sc.bits();
                let tag_align = at.align.min(self.natural_align(repr));
                self.store_scalar(sc, &int_literal(tag as u128, bits), &at.ptr, tag_align);
                let offsets = table.variant_field_offsets(*id)[*variant].clone();
                def.variants[*variant].fields.iter().zip(offsets).map(|(field, offset)| (offset, field.ty)).collect()
            }
            (AggregateKind::Callable, _) => return self.make_callable(operands, &at.ptr, at.align),
            (kind, other) => return unsupported(format!("an aggregate {kind:?} of {other:?}")),
        };
        for ((offset, field_ty), value) in fields.into_iter().zip(values) {
            let field = self.field(&at, offset, field_ty);
            match value {
                Field::Val(value) => {
                    let size = self.m.ty.size_align(field_ty).0;
                    self.store(&value, &field.ptr, field.align, size, false);
                }
                // `[GRM-39]` — the folded literal, with the retains its own statement made.
                Field::Folded(local) => {
                    let inner = self.folded.get(&local).cloned().expect("a folded literal");
                    let Rvalue::Aggregate { kind, operands } = &inner else { unreachable!("a folded literal is an aggregate") };
                    let inner_ty = self.mir().local(local).ty;
                    if !self.mir().uncounted_handles.contains(&local) {
                        self.retain_rvalue(&inner, inner_ty)?;
                    }
                    let values = self.aggregate_values(None, operands)?;
                    self.write_aggregate(&field, inner_ty, kind, operands, values)?;
                }
            }
        }
        Ok(())
    }

    /// `[value; count]`: a loop storing the value in each element.
    fn repeat(&mut self, place: &Place, ty: Ty, value: &Operand, count: u64) -> Result<()> {
        let TyKind::Array { elem, .. } = *self.m.ty.kind(ty) else {
            return unsupported("a repeat of a non-array");
        };
        let value = self.operand(value)?;
        let at = self.place(place)?;
        let (size, align) = self.m.ty.size_align(elem);
        if count == 0 || size == 0 {
            return Ok(());
        }
        // A zero byte pattern is one memset.
        if let Val::Imm(_, text) = &value
            && matches!(text.as_str(), "0" | "false" | "null" | "0x0000000000000000")
        {
            self.zero(&at.ptr, at.align, size * count);
            return Ok(());
        }
        let (head, body, done) = (self.label(), self.label(), self.label());
        self.emit(format!("br label %{head}"));
        let entry_label = self.block.clone();
        self.start(&head);
        let i = self.tmp();
        let next = self.tmp();
        self.emit(format!("{i} = phi i64 [ 0, %{entry_label} ], [ {next}, %{body} ]"));
        let more = self.value(format!("icmp ult i64 {i}, {count}"));
        self.emit(format!("br i1 {more}, label %{body}, label %{done}"));
        self.start(&body);
        let element = self.element_at(&at.ptr, &i, size);
        self.store(&value, &element, at.align.min(align), size, false);
        self.emit(format!("{next} = add i64 {i}, 1"));
        self.emit(format!("br label %{head}"));
        self.start(&done);
        Ok(())
    }

    /// The variant `variant` of the enum `id` marked at `at`: its tag, or for a niche `Option`'s
    /// `None` the niche (its `Some` is its payload, which the caller writes).
    pub(crate) fn write_variant(&mut self, at: &PlaceRef, id: ember_types::EnumId, variant: usize) -> Result<()> {
        let table = self.m.ty.table;
        if let Some(niche) = table.option_niche(id) {
            if variant != niche.some {
                self.niche_none(at, niche.payload)?;
            }
            return Ok(());
        }
        let def = table.enum_def(id);
        let Repr::Scalar(sc) = self.m.ty.repr(def.repr) else {
            return unsupported("an enum whose tag is not an integer");
        };
        let tag = int_literal(def.variants[variant].discriminant as u128, sc.bits());
        let align = at.align.min(self.natural_align(def.repr));
        self.store_scalar(sc, &tag, &at.ptr, align);
        Ok(())
    }

    /// `None` of a niche `Option` over `payload` written at `at`: the value its payload never holds.
    fn niche_none(&mut self, at: &PlaceRef, payload: Ty) -> Result<()> {
        let table = self.m.ty.table;
        let kind = table.kind(payload).clone();
        let ptr = at.ptr.clone();
        match kind {
            TyKind::Bool => self.emit(format!("store i8 2, ptr {ptr}, align 1")),
            TyKind::Char => self.emit(format!("store i32 1114112, ptr {ptr}, align {}", at.align.min(4))),
            TyKind::Span { .. } | TyKind::Str => {
                self.emit(format!("store ptr null, ptr {ptr}, align {}", at.align.min(8)));
                let len = self.gep(&ptr, 8);
                self.emit(format!("store i64 -1, ptr {len}, align {}", at.align.min(8)));
            }
            TyKind::Ref { inner, .. } if matches!(table.kind(inner), TyKind::Dyn { .. }) => {
                self.zero(&ptr, at.align, 16);
            }
            TyKind::Class(_) | TyKind::ClassInterface(_) | TyKind::Ref { .. } | TyKind::CStr | TyKind::Fn { abi: Some(_), .. } => {
                self.emit(format!("store ptr null, ptr {ptr}, align {}", at.align.min(8)));
            }
            TyKind::Enum(id) => {
                let sentinel = table.enum_unused_discriminant(id).expect("an enum niche has an unused discriminant");
                let Repr::Scalar(sc) = self.m.ty.repr(table.enum_def(id).repr) else {
                    return unsupported("an enum niche without an integer tag");
                };
                self.store_scalar(sc, &int_literal(sentinel as u128, sc.bits()), &ptr, at.align);
            }
            TyKind::Range(id) => {
                let repr = table.range_def(id).repr;
                // `[RNG-7]` — a float range's value is never NaN: a quiet NaN is `None`.
                if self.m.ty.is_float(repr) {
                    let (ty, bits) = match self.m.ty.repr(repr) {
                        Repr::Scalar(Sc::F32) => ("i32", "2143289344"),
                        Repr::Scalar(Sc::F64) => ("i64", "9221120237041090560"),
                        _ => ("i16", "32256"),
                    };
                    let width = ty[1..].parse::<u64>().unwrap_or(16) / 8;
                    self.emit(format!("store {ty} {bits}, ptr {ptr}, align {}", at.align.min(width)));
                    return Ok(());
                }
                let bits = table.range_unused_integer(id).expect("an integer range niche has an unused value");
                let Repr::Scalar(sc) = self.m.ty.repr(repr) else { unreachable!("a range is a scalar") };
                self.store_scalar(sc, &int_literal(bits, sc.bits()), &ptr, at.align);
            }
            TyKind::Struct(id) => {
                if let Some(inner) = table.compiler_box_inner(id) {
                    if matches!(table.kind(inner), TyKind::Dyn { .. }) {
                        self.zero(&ptr, at.align, 16);
                    } else {
                        self.emit(format!("store ptr null, ptr {ptr}, align {}", at.align.min(8)));
                    }
                } else {
                    let field = table.struct_def(id).fields[0].ty;
                    let (size, _) = self.m.ty.size_align(field);
                    self.zero(&ptr, at.align, size);
                }
            }
            other => return unsupported(format!("a niche in {other:?}")),
        }
        Ok(())
    }

    /// Whether the niche `Option` at `at` over `payload` is `None`.
    fn niche_is_none(&mut self, at: &PlaceRef, payload: Ty) -> Result<String> {
        let table = self.m.ty.table;
        let kind = table.kind(payload).clone();
        let ptr = at.ptr.clone();
        let align = at.align;
        let word = |cx: &mut Self, sc: Sc, offset: u64, value: &str| -> String {
            let at = cx.gep(&ptr, offset);
            let loaded = cx.load(sc, &at, align.min(sc.bits().div_ceil(8).max(1) as u64));
            cx.value(format!("icmp eq {} {loaded}, {value}", sc.ir()))
        };
        Ok(match kind {
            TyKind::Bool => word(self, Sc::I(8), 0, "2"),
            TyKind::Char => word(self, Sc::I(32), 0, "1114112"),
            TyKind::Span { .. } | TyKind::Str => {
                let null = word(self, Sc::Ptr, 0, "null");
                let max = word(self, Sc::I(64), 8, "-1");
                self.value(format!("and i1 {null}, {max}"))
            }
            TyKind::Ref { inner, .. } if matches!(table.kind(inner), TyKind::Dyn { .. }) => word(self, Sc::Ptr, 0, "null"),
            TyKind::Class(_) | TyKind::ClassInterface(_) | TyKind::Ref { .. } | TyKind::CStr | TyKind::Fn { abi: Some(_), .. } => {
                word(self, Sc::Ptr, 0, "null")
            }
            TyKind::Enum(id) => {
                let sentinel = table.enum_unused_discriminant(id).expect("an enum niche has an unused discriminant");
                let Repr::Scalar(sc) = self.m.ty.repr(table.enum_def(id).repr) else {
                    return unsupported("an enum niche without an integer tag");
                };
                word(self, sc, 0, &int_literal(sentinel as u128, sc.bits()))
            }
            TyKind::Range(id) => {
                let repr = table.range_def(id).repr;
                // `[RNG-7]` — NaN is `None`.
                if self.m.ty.is_f16(repr) {
                    let bits = self.load(Sc::I(16), &ptr, align.min(2));
                    let magnitude = self.value(format!("and i16 {bits}, 32767"));
                    return Ok(self.value(format!("icmp ugt i16 {magnitude}, 31744")));
                }
                if self.m.ty.is_float(repr) {
                    let Repr::Scalar(sc) = self.m.ty.repr(repr) else { unreachable!("a range is a scalar") };
                    let value = self.load(sc, &ptr, align.min(sc.bits().div_ceil(8) as u64));
                    return Ok(self.value(format!("fcmp uno {} {value}, {value}", sc.ir())));
                }
                let bits = table.range_unused_integer(id).expect("an integer range niche has an unused value");
                let Repr::Scalar(sc) = self.m.ty.repr(repr) else { unreachable!("a range is a scalar") };
                word(self, sc, 0, &int_literal(bits, sc.bits()))
            }
            TyKind::Struct(id) => {
                if table.compiler_box_inner(id).is_some() {
                    word(self, Sc::Ptr, 0, "null")
                } else {
                    let field = table.struct_def(id).fields[0].ty;
                    let Repr::Scalar(sc) = self.m.ty.repr(field) else {
                        return unsupported("a struct niche whose field is not a scalar");
                    };
                    word(self, sc, 0, if sc == Sc::Ptr { "null" } else { "0" })
                }
            }
            other => return unsupported(format!("a niche in {other:?}")),
        })
    }

    // -- rvalues ----------------------------------------------------------------------------------

    fn rvalue(&mut self, rvalue: &Rvalue, target: Ty) -> Result<Val> {
        match rvalue {
            Rvalue::Use(operand) => self.operand(operand),
            Rvalue::BinaryOp { op, lhs, rhs } => self.binary(*op, lhs, rhs, target),
            Rvalue::UnaryOp { op, operand } => {
                let ty = self.operand_ty(operand).unwrap_or(target);
                // D-316 — an `f16`'s negation flips its sign bit.
                if self.m.ty.is_f16(ty) {
                    let (bits, _) = self.scalar(operand)?;
                    return match op {
                        UnOp::Neg => Ok(Val::Imm(Sc::I(16), self.value(format!("xor i16 {bits}, -32768")))),
                        _ => unsupported("an `f16` operator other than `-`"),
                    };
                }
                let (value, sc) = self.scalar(operand)?;
                let result = match (op, sc) {
                    (UnOp::Neg, sc) if sc.is_float() => self.value(format!("fneg {} {value}", sc.ir())),
                    (UnOp::Neg, sc) => self.value(format!("sub {} 0, {value}", sc.ir())),
                    (UnOp::Not, Sc::Bool) => self.value(format!("xor i1 {value}, true")),
                    (UnOp::Not | UnOp::BitNot, sc) => self.value(format!("xor {} {value}, -1", sc.ir())),
                };
                Ok(Val::Imm(sc, result))
            }
            Rvalue::Cast { kind, operand, to } => self.cast(kind, operand, *to),
            Rvalue::Discriminant(place) => {
                let at = self.place(place)?;
                let Repr::Scalar(Sc::I(bits)) = self.m.ty.repr(target) else {
                    return unsupported("a discriminant read into a non-integer");
                };
                let value = self.discriminant_at(&at)?;
                let (value, from) = value;
                Ok(Val::Imm(Sc::I(bits), self.int_convert(&value, from, bits, true)))
            }
            Rvalue::Ref { place, .. } => {
                let ty = self.place_ty(place);
                // A value of no bytes has no slot, but one behind a pointer (an `extern type`'s,
                // `[FFI-8]`) has the pointer's address.
                if self.m.ty.is_zst(ty) && !place.projection.iter().any(|projection| matches!(projection, Projection::Deref)) {
                    return Ok(Val::Imm(Sc::Ptr, self.void_place()));
                }
                // `[TYP-14]`, `[TYP-22]` — a `Box[dyn I]` read through and borrowed (a `ref dyn I`
                // argument): its value's address and its table, the box's own two words.
                if let [.., Projection::Field(0), Projection::Deref] = place.projection.as_slice() {
                    let boxed = Place { local: place.local, projection: place.projection[..place.projection.len() - 2].to_vec() };
                    let table = self.m.ty.table;
                    if let TyKind::Struct(id) = *table.kind(self.place_ty(&boxed))
                        && table.compiler_box_inner(id).is_some_and(|inner| matches!(table.kind(inner), TyKind::Dyn { .. }))
                    {
                        let at = self.place(&boxed)?;
                        return Ok(Val::Mem(at.ptr, at.align));
                    }
                }
                if matches!(self.m.ty.repr(target), Repr::Mem { .. }) {
                    return unsupported("a borrow that makes a fat reference");
                }
                let at = self.place(place)?;
                Ok(Val::Imm(Sc::Ptr, at.ptr))
            }
            Rvalue::Aggregate { .. } | Rvalue::Repeat { .. } => unreachable!("written by `assign`"),
        }
    }

    /// The discriminant of the enum at `at` and its width.
    pub(crate) fn discriminant_at(&mut self, at: &PlaceRef) -> Result<(String, u32)> {
        let table = self.m.ty.table;
        let TyKind::Enum(id) = *table.kind(at.ty) else {
            return match self.m.ty.repr(at.ty) {
                Repr::Scalar(Sc::I(bits)) => Ok((self.load(Sc::I(bits), &at.ptr, at.align), bits)),
                _ => unsupported("the discriminant of a non-enum"),
            };
        };
        let def = table.enum_def(id);
        let Repr::Scalar(sc @ Sc::I(bits)) = self.m.ty.repr(def.repr) else {
            return unsupported("an enum whose tag is not an integer");
        };
        if let Some(niche) = table.option_niche(id) {
            let none = self.niche_is_none(at, niche.payload)?;
            let (none_tag, some_tag) = (def.variants[niche.none].discriminant, def.variants[niche.some].discriminant);
            let value = self.value(format!(
                "select i1 {none}, i{bits} {}, i{bits} {}",
                int_literal(none_tag as u128, bits),
                int_literal(some_tag as u128, bits)
            ));
            return Ok((value, bits));
        }
        Ok((self.load(sc, &at.ptr, at.align.min(self.natural_align(def.repr))), bits))
    }

    /// An integer of `from` bits as one of `to` bits.
    pub(crate) fn int_convert(&mut self, value: &str, from: u32, to: u32, signed: bool) -> String {
        match from.cmp(&to) {
            std::cmp::Ordering::Equal => value.to_string(),
            std::cmp::Ordering::Greater => self.value(format!("trunc i{from} {value} to i{to}")),
            std::cmp::Ordering::Less => {
                let ext = if signed && from > 1 { "sext" } else { "zext" };
                self.value(format!("{ext} i{from} {value} to i{to}"))
            }
        }
    }

    /// `[TYP-29]` — float `//` is `floor(a / b)` and `%` is `a - b * floor(a / b)`, each operation
    /// rounded as the machine rounds it: no contraction (the runtime's `EMBER_FLOAT_FLOOR`).
    fn float_floor(&mut self, op: BinOp, sc: Sc, a: &str, b: &str) -> String {
        let (ty, name) = (sc.ir(), if sc == Sc::F32 { "f32" } else { "f64" });
        let flags = self.fast_flags();
        let floor = format!("llvm.floor.{name}");
        self.m.declare(&floor, || format!("declare {ty} @{floor}({ty})"));
        let quotient = self.value(format!("fdiv{flags} {ty} {a}, {b}"));
        let q = self.value(format!("call{flags} {ty} @{floor}({ty} {quotient})"));
        if op == BinOp::FloorDiv {
            return q;
        }
        let m = self.value(format!("fmul{flags} {ty} {b}, {q}"));
        self.value(format!("fsub{flags} {ty} {a}, {m}"))
    }

    /// An `f16` operand as a `double`, exactly: a constant as its value, else through the
    /// runtime's conversion.
    fn half_operand(&mut self, operand: &Operand) -> Result<String> {
        if let Operand::Const(Const::Float { value, .. }) = operand {
            return Ok(float_literal(ember_types::f16_value(ember_types::f16_bits(*value)), Sc::F64));
        }
        let (bits, _) = self.scalar(operand)?;
        let wide = self.alloca(8, 8);
        self.call_rt("f16_to_f64", vec![Val::Imm(Sc::I(16), bits)], Some((&wide, 8)));
        Ok(self.load(Sc::F64, &wide, 8))
    }

    /// A `double` rounded once to `f16`, ties to even (the runtime's).
    fn to_half(&mut self, value: &str) -> String {
        let narrow = self.alloca(2, 2);
        self.call_rt("f64_to_f16", vec![Val::Imm(Sc::F64, value.to_string())], Some((&narrow, 2)));
        self.load(Sc::I(16), &narrow, 2)
    }

    /// D-316 — a numeric cast to or from `f16`, through `double`: exact from `f16`, and rounded
    /// once to it; a float to an integer saturates (`[TYP-6]`).
    fn half_cast(&mut self, operand: &Operand, from: Ty, to: Ty) -> Result<Val> {
        let types = &self.m.ty;
        if types.is_f16(from) && types.is_f16(to) {
            return self.operand(operand);
        }
        let (from_signed, to_signed) = (types.is_signed(from), types.is_signed(to));
        let to_repr = types.repr(to);
        let double = if types.is_f16(from) {
            self.half_operand(operand)?
        } else {
            let (value, sc) = self.scalar(operand)?;
            match sc {
                Sc::F64 => value,
                Sc::F32 => self.value(format!("fpext float {value} to double")),
                Sc::I(128) => {
                    let name = if from_signed { "i128_to_f64" } else { "u128_to_f64" };
                    let slot = self.alloca(8, 8);
                    self.call_rt(name, vec![Val::Imm(Sc::I(128), value)], Some((&slot, 8)));
                    self.load(Sc::F64, &slot, 8)
                }
                Sc::I(bits) if bits <= 64 => {
                    let instruction = if from_signed { "sitofp" } else { "uitofp" };
                    self.value(format!("{instruction} i{bits} {value} to double"))
                }
                _ => return unsupported("an `f16` conversion of this type"),
            }
        };
        if self.m.ty.is_f16(to) {
            return Ok(Val::Imm(Sc::I(16), self.to_half(&double)));
        }
        Ok(match to_repr {
            Repr::Scalar(Sc::F64) => Val::Imm(Sc::F64, double),
            Repr::Scalar(Sc::F32) => Val::Imm(Sc::F32, self.value(format!("fptrunc double {double} to float"))),
            Repr::Scalar(Sc::I(128)) => {
                let name = if to_signed { "ftoi_i128" } else { "ftoi_u128" };
                let slot = self.alloca(16, 16);
                self.call_rt(name, vec![Val::Imm(Sc::F64, double)], Some((&slot, 16)));
                Val::Imm(Sc::I(128), self.load(Sc::I(128), &slot, 16))
            }
            Repr::Scalar(Sc::I(bits)) if bits <= 64 => {
                let intrinsic = format!("llvm.fpto{}i.sat.i{bits}.f64", if to_signed { "s" } else { "u" });
                self.m.declare(&intrinsic, || format!("declare i{bits} @{intrinsic}(double)"));
                Val::Imm(Sc::I(bits), self.value(format!("call i{bits} @{intrinsic}(double {double})")))
            }
            _ => return unsupported("an `f16` conversion to this type"),
        })
    }

    /// D-316 — an `f16` operation: in `double`, which holds every `f16` exactly, rounded back
    /// once; a comparison compares the widened values.
    fn half_binary(&mut self, op: BinOp, lhs: &Operand, rhs: &Operand) -> Result<Val> {
        let (a, b) = (self.half_operand(lhs)?, self.half_operand(rhs)?);
        if op.is_comparison() {
            let cond = match op {
                BinOp::Eq | BinOp::Is => "oeq",
                BinOp::Ne | BinOp::IsNot => "une",
                BinOp::Lt => "olt",
                BinOp::Le => "ole",
                BinOp::Gt => "ogt",
                _ => "oge",
            };
            return Ok(Val::Imm(Sc::Bool, self.value(format!("fcmp {cond} double {a}, {b}"))));
        }
        let wide = match op {
            BinOp::Add => self.value(format!("fadd double {a}, {b}")),
            BinOp::Sub => self.value(format!("fsub double {a}, {b}")),
            BinOp::Mul => self.value(format!("fmul double {a}, {b}")),
            BinOp::Div => self.value(format!("fdiv double {a}, {b}")),
            BinOp::FloorDiv | BinOp::FloorRem => self.float_floor(op, Sc::F64, &a, &b),
            _ => return unsupported(format!("the `f16` operator `{}`", op.c_operator())),
        };
        Ok(Val::Imm(Sc::I(16), self.to_half(&wide)))
    }

    /// A comparison of values held in memory: an enum without payloads by its discriminant (one
    /// `@align` makes wide); `[FN-6]` — two callable values are equal when they call the same
    /// entry with the same state (the descriptor and the three words).
    fn memory_compare(&mut self, op: BinOp, lhs: &Operand, rhs: &Operand, ty: Ty) -> Result<Val> {
        let (size, align) = self.m.ty.size_align(ty);
        let a = self.operand(lhs)?;
        let b = self.operand(rhs)?;
        let (a, a_align) = self.spill(&a, size, align);
        let (b, b_align) = self.spill(&b, size, align);
        match self.m.ty.kind(ty).clone() {
            TyKind::Enum(id) if self.m.ty.table.enum_def(id).is_unit_only() => {
                let (x, bits) = self.discriminant_at(&PlaceRef { ptr: a, ty, variant: None, align: a_align })?;
                let (y, _) = self.discriminant_at(&PlaceRef { ptr: b, ty, variant: None, align: b_align })?;
                let signed = matches!(self.m.ty.kind(self.m.ty.table.enum_def(id).repr), TyKind::Int(_));
                let cond = match (op, signed) {
                    (BinOp::Eq | BinOp::Is, _) => "eq",
                    (BinOp::Ne | BinOp::IsNot, _) => "ne",
                    (BinOp::Lt, true) => "slt",
                    (BinOp::Lt, false) => "ult",
                    (BinOp::Le, true) => "sle",
                    (BinOp::Le, false) => "ule",
                    (BinOp::Gt, true) => "sgt",
                    (BinOp::Gt, false) => "ugt",
                    (_, true) => "sge",
                    (_, false) => "uge",
                };
                Ok(Val::Imm(Sc::Bool, self.value(format!("icmp {cond} i{bits} {x}, {y}"))))
            }
            TyKind::Fn { abi: None, .. } if matches!(op, BinOp::Eq | BinOp::Ne) => {
                let mut same = "true".to_string();
                for word in 1..5u64 {
                    let (x_at, y_at) = (self.gep(&a, 8 * word), self.gep(&b, 8 * word));
                    let x = self.load(Sc::I(64), &x_at, 8);
                    let y = self.load(Sc::I(64), &y_at, 8);
                    let equal = self.value(format!("icmp eq i64 {x}, {y}"));
                    same = self.value(format!("and i1 {same}, {equal}"));
                }
                if op == BinOp::Ne {
                    same = self.value(format!("xor i1 {same}, true"));
                }
                Ok(Val::Imm(Sc::Bool, same))
            }
            _ => unsupported(format!("a comparison of `{}`", self.m.ty.table.display(ty))),
        }
    }

    fn binary(&mut self, op: BinOp, lhs: &Operand, rhs: &Operand, target: Ty) -> Result<Val> {
        let lhs_ty = self.operand_ty(lhs).or_else(|| self.operand_ty(rhs)).unwrap_or(target);
        let types = &self.m.ty;
        if types.is_f16(lhs_ty) || types.is_f16(target) {
            return self.half_binary(op, lhs, rhs);
        }
        // `[TYP-36]` — `void` has one value: equal to itself, never less.
        let void = |operand: &Operand| matches!(operand, Operand::Const(Const::Void));
        if (types.is_zst(lhs_ty) || void(lhs) || void(rhs)) && op.is_comparison() {
            let holds = matches!(op, BinOp::Eq | BinOp::Is | BinOp::Le | BinOp::Ge);
            return Ok(Val::Imm(Sc::Bool, holds.to_string()));
        }
        if matches!(types.repr(lhs_ty), Repr::Mem { .. }) && op.is_comparison() {
            return self.memory_compare(op, lhs, rhs, lhs_ty);
        }
        let signed = types.is_signed(lhs_ty);
        let (a, sc) = self.scalar(lhs)?;
        let (b, rhs_sc) = self.scalar(rhs)?;
        let ty = sc.ir();
        if op.is_comparison() {
            let result = if sc.is_float() {
                let cond = match op {
                    BinOp::Eq | BinOp::Is => "oeq",
                    BinOp::Ne | BinOp::IsNot => "une",
                    BinOp::Lt => "olt",
                    BinOp::Le => "ole",
                    BinOp::Gt => "ogt",
                    _ => "oge",
                };
                self.value(format!("fcmp {cond} {ty} {a}, {b}"))
            } else {
                let cond = match (op, signed) {
                    (BinOp::Eq | BinOp::Is, _) => "eq",
                    (BinOp::Ne | BinOp::IsNot, _) => "ne",
                    (BinOp::Lt, true) => "slt",
                    (BinOp::Lt, false) => "ult",
                    (BinOp::Le, true) => "sle",
                    (BinOp::Le, false) => "ule",
                    (BinOp::Gt, true) => "sgt",
                    (BinOp::Gt, false) => "ugt",
                    (_, true) => "sge",
                    (_, false) => "uge",
                };
                self.value(format!("icmp {cond} {ty} {a}, {b}"))
            };
            return Ok(Val::Imm(Sc::Bool, result));
        }
        if sc.is_float() {
            let instruction = match op {
                BinOp::Add => "fadd",
                BinOp::Sub => "fsub",
                BinOp::Mul => "fmul",
                BinOp::Div => "fdiv",
                BinOp::FloorDiv | BinOp::FloorRem => return Ok(Val::Imm(sc, self.float_floor(op, sc, &a, &b))),
                _ => return unsupported(format!("the float operator `{}`", op.c_operator())),
            };
            let flags = self.fast_flags();
            return Ok(Val::Imm(sc, self.value(format!("{instruction}{flags} {ty} {a}, {b}"))));
        }
        let wide = matches!(sc, Sc::I(bits) if bits > 64);
        let result = match op {
            BinOp::Add => self.value(format!("add {ty} {a}, {b}")),
            BinOp::Sub => self.value(format!("sub {ty} {a}, {b}")),
            BinOp::Mul => self.value(format!("mul {ty} {a}, {b}")),
            BinOp::BitAnd | BinOp::And => self.value(format!("and {ty} {a}, {b}")),
            BinOp::BitOr | BinOp::Or => self.value(format!("or {ty} {a}, {b}")),
            BinOp::BitXor => self.value(format!("xor {ty} {a}, {b}")),
            // D-272 — a 128-bit division through the runtime's helper: the instruction would be a
            // compiler-rt call, which an MSVC link lacks. Floor forms are C's on unsigned values.
            BinOp::Div | BinOp::Rem | BinOp::FloorDiv | BinOp::FloorRem if wide => {
                let name = match (op, signed) {
                    (BinOp::Div, true) => "i128_div",
                    (BinOp::Rem, true) => "i128_rem",
                    (BinOp::FloorDiv, true) => "i128_floordiv",
                    (_, true) => "i128_floorrem",
                    (BinOp::Div | BinOp::FloorDiv, false) => "u128_div",
                    (_, false) => "u128_rem",
                };
                let slot = self.alloca(16, 16);
                self.call_rt(name, vec![Val::Imm(sc, a), Val::Imm(sc, b)], Some((&slot, 16)));
                return Ok(Val::Imm(sc, self.load(sc, &slot, 16)));
            }
            BinOp::Div => self.value(format!("{} {ty} {a}, {b}", if signed { "sdiv" } else { "udiv" })),
            BinOp::Rem if signed => self.signed_rem(sc, &a, &b),
            BinOp::Rem => self.value(format!("urem {ty} {a}, {b}")),
            BinOp::FloorDiv | BinOp::FloorRem if !signed => {
                self.value(format!("{} {ty} {a}, {b}", if op == BinOp::FloorDiv { "udiv" } else { "urem" }))
            }
            BinOp::FloorDiv | BinOp::FloorRem => self.floor_div(op, sc, &a, &b),
            BinOp::Shl | BinOp::Shr => {
                // The amount masked to the width: the checks before a shift bound it, and a mask
                // keeps the IR free of poison besides (`[TYP-10]`).
                let Sc::I(bits) = sc else { return unsupported("a shift of a non-integer") };
                let Sc::I(amount_bits) = rhs_sc else { return unsupported("a shift by a non-integer") };
                let amount = self.int_convert(&b, amount_bits, bits, false);
                let masked = self.value(format!("and {ty} {amount}, {}", bits - 1));
                let instruction = match (op, signed) {
                    (BinOp::Shl, _) => "shl",
                    (_, true) => "ashr",
                    (_, false) => "lshr",
                };
                self.value(format!("{instruction} {ty} {a}, {masked}"))
            }
            other => return unsupported(format!("the operator `{}`", other.c_operator())),
        };
        Ok(Val::Imm(sc, result))
    }

    /// `[TYP-28]` — floor division and modulo of signed integers: C's truncating forms, the
    /// quotient one less and the remainder plus the divisor when the remainder is not zero and
    /// its sign differs from the divisor's.
    /// Only the operator's own division: `MIN % -1` is 0 (`[TYP-28]`), and a quotient computed
    /// beside it would be `sdiv MIN, -1`, undefined in LLVM even unused (`[CG-LL-2]`). A
    /// `//`'s overflow is checked or proved before it.
    fn floor_div(&mut self, op: BinOp, sc: Sc, a: &str, b: &str) -> String {
        let ty = sc.ir();
        let remainder = self.signed_rem(sc, a, b);
        let nonzero = self.value(format!("icmp ne {ty} {remainder}, 0"));
        let signs = self.value(format!("xor {ty} {remainder}, {b}"));
        let differ = self.value(format!("icmp slt {ty} {signs}, 0"));
        let adjust = self.value(format!("and i1 {nonzero}, {differ}"));
        if op == BinOp::FloorDiv {
            let quotient = self.value(format!("sdiv {ty} {a}, {b}"));
            let less = self.value(format!("sub {ty} {quotient}, 1"));
            self.value(format!("select i1 {adjust}, {ty} {less}, {ty} {quotient}"))
        } else {
            let more = self.value(format!("add {ty} {remainder}, {b}"));
            self.value(format!("select i1 {adjust}, {ty} {more}, {ty} {remainder}"))
        }
    }

    /// A signed remainder, whose divisor `-1` is taken as `1`: every value's remainder by
    /// either is 0, and `srem MIN, -1` is undefined in LLVM (`[CG-LL-2]`; C's `%` is too, which
    /// the runtime's `floorrem` tests for). A constant divisor folds the test away.
    fn signed_rem(&mut self, sc: Sc, a: &str, b: &str) -> String {
        let ty = sc.ir();
        let minus_one = self.value(format!("icmp eq {ty} {b}, -1"));
        let divisor = self.value(format!("select i1 {minus_one}, {ty} 1, {ty} {b}"));
        self.value(format!("srem {ty} {a}, {divisor}"))
    }

    /// `[TYP-9]` — the flags this body's floating point allows: none but under `@fastmath`
    /// (every relaxation but `nnan` and `ninf`, as the C backend's flags, ODR-090) or
    /// `@fp(contract)`.
    fn fast_flags(&self) -> &'static str {
        match self.body.map_or(ember_types::FpMode::Strict, |body| body.fp) {
            ember_types::FpMode::Strict => "",
            ember_types::FpMode::Contract => " contract",
            ember_types::FpMode::Fast => " nsz arcp contract afn reassoc",
        }
    }

    fn checked(&mut self, dest: &Place, overflow: &Place, op: BinOp, lhs: &Operand, rhs: &Operand) -> Result<()> {
        let ty = self.place_ty(dest);
        let signed = self.m.ty.is_signed(ty);
        let (a, sc) = self.scalar(lhs)?;
        let (b, _) = self.scalar(rhs)?;
        let Sc::I(bits) = sc else { return unsupported("checked arithmetic on a non-integer") };
        // A 256-bit count only adds and subtracts (G8-4).
        if bits > 128 && !matches!(op, BinOp::Add | BinOp::Sub) {
            return unsupported("checked 256-bit multiplication or division");
        }
        let ity = sc.ir();
        // D-272 — the 128-bit forms that would be compiler-rt calls: the runtime's checked helpers.
        if bits == 128 && matches!(op, BinOp::Mul | BinOp::Div | BinOp::Rem | BinOp::FloorDiv | BinOp::FloorRem) && (signed || op == BinOp::Mul) {
            let name = match (op, signed) {
                (BinOp::Mul, true) => "ck_mul_i128",
                (BinOp::Mul, false) => "ck_mul_u128",
                (BinOp::Div, _) => "ck_div_i128",
                (BinOp::Rem, _) => "ck_rem_i128",
                (BinOp::FloorDiv, _) => "ck_floordiv_i128",
                _ => "ck_floorrem_i128",
            };
            let at = self.place(dest)?;
            let result = self.alloca(16, 16);
            let flag_slot = self.alloca(1, 1);
            self.call_rt(name, vec![Val::Imm(sc, a), Val::Imm(sc, b), Val::Imm(Sc::Ptr, result.clone())], Some((&flag_slot, 1)));
            self.copy(&at.ptr, at.align, &result, 16, 16, false);
            let flag = self.load(Sc::Bool, &flag_slot, 1);
            let flag_at = self.place(overflow)?;
            self.store_scalar(Sc::Bool, &flag, &flag_at.ptr, flag_at.align);
            return Ok(());
        }
        let (result, flag) = match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul => {
                let name = match op {
                    BinOp::Add => "add",
                    BinOp::Sub => "sub",
                    _ => "mul",
                };
                let intrinsic = format!("llvm.{}{name}.with.overflow.{ity}", if signed { "s" } else { "u" });
                self.m.declare(&intrinsic, || format!("declare {{ {ity}, i1 }} @{intrinsic}({ity}, {ity})"));
                let pair = self.value(format!("call {{ {ity}, i1 }} @{intrinsic}({ity} {a}, {ity} {b})"));
                let result = self.value(format!("extractvalue {{ {ity}, i1 }} {pair}, 0"));
                let flag = self.value(format!("extractvalue {{ {ity}, i1 }} {pair}, 1"));
                (result, flag)
            }
            BinOp::FloorDiv | BinOp::FloorRem | BinOp::Div | BinOp::Rem => {
                if bits == 128 {
                    // Unsigned: the runtime's helpers, which cannot overflow.
                    let name = if matches!(op, BinOp::Div | BinOp::FloorDiv) { "u128_div" } else { "u128_rem" };
                    let slot = self.alloca(16, 16);
                    self.call_rt(name, vec![Val::Imm(sc, a.clone()), Val::Imm(sc, b.clone())], Some((&slot, 16)));
                    let quotient = self.load(sc, &slot, 16);
                    let at = self.place(dest)?;
                    self.store_scalar(sc, &quotient, &at.ptr, at.align);
                    let flag_at = self.place(overflow)?;
                    self.store_scalar(Sc::Bool, "false", &flag_at.ptr, flag_at.align);
                    return Ok(());
                }
                // `MIN // -1` is the one overflow (its remainder is 0); the quotient wraps to MIN.
                if !signed {
                    let instruction = if matches!(op, BinOp::FloorDiv | BinOp::Div) { "udiv" } else { "urem" };
                    (self.value(format!("{instruction} {ity} {a}, {b}")), "false".to_string())
                } else {
                    let min = int_literal(1u128 << (bits - 1), bits);
                    let is_min = self.value(format!("icmp eq {ity} {a}, {min}"));
                    let minus_one = self.value(format!("icmp eq {ity} {b}, -1"));
                    let both = self.value(format!("and i1 {is_min}, {minus_one}"));
                    // Divide by 1 instead where the true result does not fit, so nothing divides
                    // `MIN` by `-1`.
                    let divisor = self.value(format!("select i1 {both}, {ity} 1, {ity} {b}"));
                    let value = match op {
                        BinOp::Div => self.value(format!("sdiv {ity} {a}, {divisor}")),
                        BinOp::Rem => self.value(format!("srem {ity} {a}, {divisor}")),
                        _ => self.floor_div(op, sc, &a, &divisor),
                    };
                    // `ember_ck_rem` reports `MIN % -1` (its value 0, which `MIN % 1` gives too);
                    // `ember_ck_floorrem` does not.
                    let overflow = if op == BinOp::FloorRem { "false".to_string() } else { both };
                    (value, overflow)
                }
            }
            BinOp::Shl => return unsupported("a checked shift"),
            other => return unsupported(format!("checked `{}`", other.c_operator())),
        };
        let at = self.place(dest)?;
        self.store_scalar(sc, &result, &at.ptr, at.align);
        let flag_at = self.place(overflow)?;
        self.store_scalar(Sc::Bool, &flag, &flag_at.ptr, flag_at.align);
        Ok(())
    }

    fn cast(&mut self, kind: &CastKind, operand: &Operand, to: Ty) -> Result<Val> {
        let from = self.operand_ty(operand);
        match kind {
            // `[DSJ-1]` — a proved pair is the pair itself.
            CastKind::ViewsDisjoint => return self.operand(operand),
            // `[CLS-4]`, `[OBJ-2]` — a base class's handle, and an interface's one-word handle,
            // are the derived object's own pointer (the retain a copy takes is `retain_rvalue`'s).
            // A borrowed upcast is the same reference: C needs a base-typed copy only for its
            // aliasing rules, which the IR does not have.
            CastKind::ClassUpcast | CastKind::ClassUpcastBorrowed => return self.operand(operand),
            CastKind::InterfaceUpcast { concrete, interfaces, .. } => return self.interface_upcast(*concrete, interfaces, operand),
            CastKind::ClassInterfaceUpcast { .. } => return self.class_interface_upcast(operand),
            // ADR-186 — a borrowed `Box[dyn I]`'s value: its data pointer.
            CastKind::InterfaceDowncast => {
                let (boxed, _) = self.scalar(operand)?;
                return Ok(Val::Imm(Sc::Ptr, self.load(Sc::Ptr, &boxed, 8)));
            }
            CastKind::Numeric | CastKind::Widen => {}
            // ODR-102 — a native value's C entry.
            CastKind::FnToC => {
                let span = self.span;
                return self.fn_to_c(operand, span);
            }
        }
        let Some(from) = from else { return unsupported("a cast of an untyped operand") };
        let types = &self.m.ty;
        if types.is_f16(from) || types.is_f16(to) {
            return self.half_cast(operand, from, to);
        }
        let to_repr = types.repr(to);
        let signed = types.is_signed(from);
        let (value, sc) = self.scalar(operand)?;
        let Repr::Scalar(to_sc) = to_repr else { return unsupported("a cast to a non-scalar") };
        let result = match (sc, to_sc) {
            (a, b) if a == b => value,
            (Sc::Bool, Sc::I(bits)) => self.value(format!("zext i1 {value} to i{bits}")),
            (Sc::I(from_bits), Sc::Bool) => self.value(format!("icmp ne i{from_bits} {value}, 0")),
            (Sc::I(from_bits), Sc::I(to_bits)) => self.int_convert(&value, from_bits, to_bits, signed),
            (Sc::I(128), to @ (Sc::F32 | Sc::F64)) => {
                // D-272 — through the runtime's conversion (compiler-rt's elsewhere).
                let name = match (signed, to) {
                    (true, Sc::F64) => "i128_to_f64",
                    (true, _) => "i128_to_f32",
                    (false, Sc::F64) => "u128_to_f64",
                    (false, _) => "u128_to_f32",
                };
                let slot = self.alloca(8, 8);
                self.call_rt(name, vec![Val::Imm(Sc::I(128), value)], Some((&slot, 8)));
                self.load(to, &slot, 8)
            }
            (from @ (Sc::F32 | Sc::F64), Sc::I(128)) => {
                // `[TYP-6]` — saturating, NaN to 0, through the runtime's conversion.
                let wide = if from == Sc::F32 { self.value(format!("fpext float {value} to double")) } else { value };
                let name = if self.m.ty.is_signed(to) { "ftoi_i128" } else { "ftoi_u128" };
                let slot = self.alloca(16, 16);
                self.call_rt(name, vec![Val::Imm(Sc::F64, wide)], Some((&slot, 16)));
                self.load(Sc::I(128), &slot, 16)
            }
            (Sc::I(from_bits), to @ (Sc::F32 | Sc::F64 | Sc::F80 | Sc::F128)) => {
                if from_bits > 64 {
                    return unsupported("a 256-bit integer to float conversion");
                }
                let instruction = if signed { "sitofp" } else { "uitofp" };
                self.value(format!("{instruction} i{from_bits} {value} to {}", to.ir()))
            }
            (from @ (Sc::F32 | Sc::F64 | Sc::F80 | Sc::F128), Sc::I(to_bits)) => {
                // `[TYP-6]` — saturating, NaN to 0: LLVM's `.sat` conversions exactly.
                if to_bits > 64 {
                    return unsupported("a float to 256-bit integer conversion");
                }
                let to_signed = self.m.ty.is_signed(to);
                let intrinsic = format!("llvm.fpto{}i.sat.i{to_bits}.{}", if to_signed { "s" } else { "u" }, float_suffix(from));
                let from_ir = from.ir();
                self.m.declare(&intrinsic, || format!("declare i{to_bits} @{intrinsic}({from_ir})"));
                self.value(format!("call i{to_bits} @{intrinsic}({from_ir} {value})"))
            }
            (Sc::F32, to @ (Sc::F64 | Sc::F80 | Sc::F128)) | (Sc::F64, to @ (Sc::F80 | Sc::F128)) => {
                self.value(format!("fpext {} {value} to {}", sc.ir(), to.ir()))
            }
            (from @ (Sc::F64 | Sc::F80 | Sc::F128), to @ (Sc::F32 | Sc::F64)) => {
                self.value(format!("fptrunc {} {value} to {}", from.ir(), to.ir()))
            }
            (Sc::Ptr, Sc::I(bits)) => {
                let address = self.value(format!("ptrtoint ptr {value} to i64"));
                self.int_convert(&address, 64, bits, false)
            }
            (Sc::I(bits), Sc::Ptr) => {
                let address = self.int_convert(&value, bits, 64, signed);
                self.value(format!("inttoptr i64 {address} to ptr"))
            }
            (a, b) => return unsupported(format!("a cast from {} to {}", a.ir(), b.ir())),
        };
        Ok(Val::Imm(to_sc, result))
    }

    // -- terminators ------------------------------------------------------------------------------

    /// `[SIMD-7]` — a counted loop that is a read-only byte sum (`ember_codegen_c::byte_sum_loops`,
    /// the C backend's selection), in place of its header's test: the runtime's `sum_bytes` over
    /// the bytes left, the total moved by it modulo 2^64 as the loop's unchecked operation would
    /// (its proof, or the program's `@overflow(wrap)`, is the MIR's), the counter at the limit.
    /// The loop's own blocks are then reached by no path.
    fn byte_sum(&mut self, sum: &ember_codegen_c::ByteSumLoop) -> Result<()> {
        self.span = sum.span;
        let (counter, _) = self.read_scalar(&Place::local(sum.counter))?;
        let (limit, _) = self.read_scalar(&Place::local(sum.limit))?;
        let more = self.value(format!("icmp ult i64 {counter}, {limit}"));
        let work = self.label();
        self.emit(format!("br i1 {more}, label %{work}, label %bb{}", sum.exit));
        self.start(&work);
        let element = self.place(&sum.element)?.ptr;
        let count = self.value(format!("sub i64 {limit}, {counter}"));
        let slot = self.alloca(8, 8);
        let signed = Val::Imm(Sc::Bool, sum.signed.to_string());
        self.call_rt("sum_bytes", vec![Val::Imm(Sc::Ptr, element), Val::Imm(Sc::I(64), count), signed], Some((&slot, 8)));
        let bytes = self.load(Sc::I(64), &slot, 8);
        let total = self.place(&Place::local(sum.total))?;
        let before = self.load(Sc::I(64), &total.ptr, total.align);
        let op = if sum.subtract { "sub" } else { "add" };
        let after = self.value(format!("{op} i64 {before}, {bytes}"));
        self.emit(format!("store i64 {after}, ptr {}, align {}", total.ptr, total.align));
        let at = self.place(&Place::local(sum.counter))?;
        self.emit(format!("store i64 {limit}, ptr {}, align {}", at.ptr, at.align));
        self.emit(format!("br label %bb{}", sum.exit));
        Ok(())
    }

    fn terminator(&mut self, terminator: &Terminator, span: ember_span::Span) -> Result<()> {
        self.span = span;
        match terminator {
            Terminator::Goto(target) => self.emit(format!("br label %bb{}", target.0)),
            Terminator::Unreachable => self.emit("unreachable"),
            Terminator::Return => self.ret()?,
            Terminator::SwitchInt { discr, targets, otherwise } => {
                let (mut value, mut sc) = match self.operand(discr)? {
                    Val::Imm(sc, value) => (value, sc),
                    // A wrapper of one pointer (a `Shared` an `upgrade` returned) is tested as it.
                    Val::Mem(at, align) => (self.load(Sc::Ptr, &at, align.min(8)), Sc::Ptr),
                    Val::Zst => return unsupported("a switch on a zero-sized value"),
                };
                // A pointer is tested as its address.
                if sc == Sc::Ptr {
                    value = self.value(format!("ptrtoint ptr {value} to i64"));
                    sc = Sc::I(64);
                }
                match (sc, targets.as_slice()) {
                    (Sc::Bool, [(0, target)]) => {
                        self.emit(format!("br i1 {value}, label %bb{}, label %bb{}", otherwise.0, target.0))
                    }
                    (Sc::Bool, [(1, target)]) => {
                        self.emit(format!("br i1 {value}, label %bb{}, label %bb{}", target.0, otherwise.0))
                    }
                    _ => {
                        let ty = sc.ir();
                        let bits = sc.bits();
                        let mut seen = std::collections::BTreeSet::new();
                        let arms: Vec<String> = targets
                            .iter()
                            .filter(|(case, _)| seen.insert(*case))
                            .map(|(case, target)| format!("{ty} {}, label %bb{}", int_literal(*case as u128, bits), target.0))
                            .collect();
                        self.emit(format!("switch {ty} {value}, label %bb{} [ {} ]", otherwise.0, arms.join(" ")));
                    }
                }
            }
            Terminator::Assert { cond, expected, msg, next, span } => {
                let (cond, sc) = self.scalar(cond)?;
                // A pointer or an integer holds when it is not zero, as C tests it.
                let cond = match sc {
                    Sc::Bool => cond,
                    Sc::Ptr => self.value(format!("icmp ne ptr {cond}, null")),
                    other => self.value(format!("icmp ne {} {cond}, 0", other.ir())),
                };
                let fail = self.label();
                let ok = format!("bb{}", next.0);
                if *expected {
                    self.emit(format!("br i1 {cond}, label %{ok}, label %{fail}"));
                } else {
                    self.emit(format!("br i1 {cond}, label %{fail}, label %{ok}"));
                }
                self.start(&fail);
                self.panic(msg, *span)?;
            }
            Terminator::Call { func, args, dest, next } => {
                self.call(func, args, dest, span)?;
                // A call that never returns (`process.exit`) has ended the block already.
                if !self.terminated() {
                    self.emit(format!("br label %bb{}", next.0));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn ret(&mut self) -> Result<()> {
        // `[HR-42]` — leaving Ember code for the foreign caller of an export.
        if self.m.reload.is_some() && self.body.is_some_and(|body| body.abi.is_some()) {
            self.call_rt("reload_leave", Vec::new(), None);
        }
        // `[HR-3b]` — this frame's record off its thread's chain.
        if let Some(record) = self.frame_record.as_ref().map(|record| (record.head.clone(), record.record.clone())) {
            let (head, record) = record;
            let head = self.load(Sc::Ptr, &head, 8);
            let previous = self.load(Sc::Ptr, &record, 8);
            self.emit(format!("store ptr {previous}, ptr {head}, align 8"));
        }
        match self.sig.ret.clone() {
            Return::Void | Return::Sret { .. } => self.emit("ret void"),
            Return::Direct(parts) => {
                let slot = self.addr[0].clone().expect("a result with bytes has a slot");
                let (_, align) = self.m.ty.size_align(self.mir().return_ty());
                let values: Vec<String> = parts.iter().map(|part| self.load_part(&slot, align, part)).collect();
                if parts.len() == 1 {
                    let line = format!("ret {} {}", parts[0].ty, values[0]);
                    self.emit(line);
                } else {
                    let ty = self.sig.ret.ir();
                    let mut aggregate = "undef".to_string();
                    for (index, (part, value)) in parts.iter().zip(&values).enumerate() {
                        aggregate = self.value(format!("insertvalue {ty} {aggregate}, {} {value}, {index}", part.ty));
                    }
                    self.emit(format!("ret {ty} {aggregate}"));
                }
            }
        }
        Ok(())
    }

    fn load_part(&mut self, base: &str, align: u64, part: &Part) -> String {
        let at = self.gep(base, part.offset);
        let align = align.min(offset_align(part.offset));
        if part.ty == PartTy::I(1) {
            let byte = self.value(format!("load i8, ptr {at}, align {align}"));
            return self.value(format!("trunc i8 {byte} to i1"));
        }
        self.value(format!("load {}, ptr {at}, align {align}", part.ty))
    }

    fn store_part(&mut self, base: &str, align: u64, part: &Part, value: &str) {
        let at = self.gep(base, part.offset);
        let align = align.min(offset_align(part.offset));
        if part.ty == PartTy::I(1) {
            let byte = self.value(format!("zext i1 {value} to i8"));
            self.emit(format!("store i8 {byte}, ptr {at}, align {align}"));
        } else {
            self.emit(format!("store {} {value}, ptr {at}, align {align}", part.ty));
        }
    }

    /// The IR arguments passing `values` (each with its shape's size and alignment) by `sig`.
    fn lower_args(&mut self, sig: &Signature, values: &[(Val, u64, u64)]) -> Vec<String> {
        let mut out = Vec::new();
        for (pass, (value, size, align)) in sig.params.iter().zip(values) {
            match pass {
                Pass::Ignore => {}
                Pass::Direct(parts) => {
                    if let (Val::Imm(sc, text), [part]) = (value, parts.as_slice())
                        && part.offset == 0
                        && sc_part(*sc) == Some(part.ty)
                    {
                        out.push(format!("{}{} {text}", part.ty, part.ext.attr()));
                        continue;
                    }
                    let (mut base, mut base_align) = self.spill(value, *size, *align);
                    let span = parts_span(parts);
                    if span > *size {
                        let padded = self.alloca(span, base_align.max(8));
                        self.copy(&padded, base_align.max(8), &base, base_align, *size, false);
                        (base, base_align) = (padded, base_align.max(8));
                    }
                    for part in parts {
                        let loaded = self.load_part(&base, base_align, part);
                        out.push(format!("{}{} {loaded}", part.ty, part.ext.attr()));
                    }
                }
                Pass::Indirect { size, align, byval } => {
                    let (base, base_align) = self.spill(value, *size, *align);
                    if *byval {
                        out.push(format!("ptr byval([{size} x i8]) align {align} {base}"));
                    } else {
                        // The callee's own copy.
                        let copy = self.alloca(*size, *align);
                        self.copy(&copy, *align, &base, base_align, *size, false);
                        out.push(format!("ptr {copy}"));
                    }
                }
            }
        }
        out
    }

    /// A call by `sig` to `callee` with `values`; the result, if any, written to `dest`.
    pub(crate) fn lowered_call(&mut self, callee: &str, sig: &Signature, values: &[(Val, u64, u64)], dest: Option<(&str, u64, u64)>, attrs: &str) {
        let mut args = Vec::new();
        let sret = if let Return::Sret { size, align } = sig.ret {
            let slot = self.alloca(size, align);
            args.push(format!("ptr sret([{size} x i8]) align {align} {slot}"));
            Some((slot, size, align))
        } else {
            None
        };
        args.extend(self.lower_args(sig, values));
        let ret_ir = sig.ret.ir();
        let call = format!("call {}{} {callee}({}){attrs}", sig.conv.ir(), ret_with_attr(sig), args.join(", "));
        match (&sig.ret, dest) {
            (Return::Direct(parts), Some((to, to_align, to_size))) => {
                let result = self.value(&call);
                let span = parts_span(parts);
                let (into, into_align) = if span > to_size { (self.alloca(span, to_align.max(8)), to_align.max(8)) } else { (to.to_string(), to_align) };
                if parts.len() == 1 {
                    self.store_part(&into, into_align, &parts[0], &result);
                } else {
                    for (index, part) in parts.iter().enumerate() {
                        let element = self.value(format!("extractvalue {ret_ir} {result}, {index}"));
                        self.store_part(&into, into_align, part, &element);
                    }
                }
                if into != to {
                    self.copy(to, to_align, &into, into_align, to_size, false);
                }
            }
            (Return::Sret { .. }, Some((to, to_align, _))) => {
                self.emit(call);
                let (slot, size, align) = sret.expect("an sret slot");
                self.copy(to, to_align, &slot, align, size, false);
            }
            (Return::Direct(_), None) => {
                let _ = self.value(&call);
            }
            _ => self.emit(call),
        }
    }

    // -- calls ------------------------------------------------------------------------------------

    /// Declares a body of this program the module calls (its definition, when it has one, removes
    /// the declaration).
    fn declare_body(&mut self, symbol: &str) -> Result<()> {
        declare_body_symbol(self.m, symbol)
    }

    fn call(&mut self, func: &FuncRef, args: &[Operand], dest: &Place, span: ember_span::Span) -> Result<()> {
        match func {
            FuncRef::Virtual { owner, slot, receiver, param_modes } => {
                self.retain_owned_copies(args, |index| param_modes.get(index) == Some(&ember_mir::ParameterMode::Owned))?;
                let dest = self.dest(dest)?;
                return self.virtual_call(*owner, *slot, *receiver, args, dest);
            }
            FuncRef::Interface { interfaces, interface, slot, params, ret, class_handle, param_modes, .. } => {
                self.retain_owned_copies(args, |index| {
                    index.checked_sub(1).and_then(|parameter| param_modes.get(parameter)) == Some(&ember_mir::ParameterMode::Owned)
                })?;
                let dest = self.dest(dest)?;
                return self.interface_call(interfaces, *interface, *slot, params, *ret, *class_handle, args, dest);
            }
            FuncRef::DynBoxNew { concrete, interfaces, .. } => {
                let dest = self.dest(dest)?;
                return self.dyn_box_new(*concrete, interfaces, args, dest);
            }
            FuncRef::Indirect { operand, ty, .. } => {
                if let TyKind::Fn { params, .. } = self.m.ty.kind(*ty).clone() {
                    self.retain_owned_copies(args, |index| params.get(index).is_some_and(|param| param.mode == ember_types::FnParamMode::Owned))?;
                }
                let dest = self.dest(dest)?;
                return self.indirect_call(operand, *ty, args, dest);
            }
            _ => {}
        }
        match func {
            FuncRef::Direct { symbol, .. } => {
                let Some(body) = self.m.bodies.get(symbol.as_str()).copied() else {
                    return unsupported(format!("a call to `{symbol}`, which has no body"));
                };
                // `[FFI-1]`, `[FFI-11]` — a foreign function whose contract says `threads(main)`:
                // the debug profile checks that the call is on the main thread.
                if self.m.profile == crate::Profile::Debug && body.contracts.foreign_threads == Some(ember_mir::ExportThreadPolicy::Main) {
                    let what = self.m.bytes(format!("`{}`", body.name).as_bytes());
                    self.call_rt("ffi_check_thread", vec![Val::Imm(Sc::I(32), u32::MAX.to_string()), Val::Imm(Sc::Ptr, what)], None);
                }
                if body.variadic {
                    return self.variadic_call(body, args, dest);
                }
                self.retain_owned_copies(args, |index| body.param_modes.get(index) == Some(&ember_mir::ParameterMode::Owned))?;
                let mut values = Vec::with_capacity(args.len());
                for argument in args {
                    values.push(self.operand(argument)?);
                }
                let dest = self.dest(dest)?;
                self.call_body(symbol, values, dest.as_ref().map(|(ptr, align)| (ptr.as_str(), *align)))
            }
            FuncRef::Builtin { which, arg_ty } => self.builtin(which, *arg_ty, args, dest, span),
            FuncRef::Virtual { .. } => unsupported("a virtual call ([DSP-2])"),
            FuncRef::Interface { .. } => unsupported("a call through an interface ([TYP-22])"),
            FuncRef::DynBoxNew { .. } => unsupported("a `Box[dyn]` ([TYP-22])"),
            FuncRef::Indirect { .. } => unsupported("a call through a function value ([CLO-3])"),
        }
    }

    /// `[RC-1]`, `[FN-1]` — a value copied into an owned parameter retains its handles first: the
    /// callee releases its parameter. A moved one hands its references over.
    fn retain_owned_copies(&mut self, args: &[Operand], owned: impl Fn(usize) -> bool) -> Result<()> {
        for (index, argument) in args.iter().enumerate() {
            if let Operand::Copy(place) = argument
                && owned(index)
            {
                let ty = self.place_ty(place);
                if self.holds_counts(ty) {
                    let at = self.place(place)?;
                    self.retain_lines(&at.ptr, at.align, ty)?;
                }
            }
        }
        Ok(())
    }

    /// The address a call's result is written to, or `None` when it has no bytes.
    fn dest(&mut self, dest: &Place) -> Result<Option<(String, u64)>> {
        let ty = self.place_ty(dest);
        if self.m.ty.is_zst(ty) {
            return Ok(None);
        }
        let at = self.place(dest)?;
        Ok(Some((at.ptr, at.align)))
    }

    /// A call of the program's function `symbol` with `values`; its result written to `dest`.
    pub(crate) fn call_body(&mut self, symbol: &str, values: Vec<Val>, dest: Option<(&str, u64)>) -> Result<()> {
        let Some(body) = self.m.bodies.get(symbol).copied() else {
            return unsupported(format!("a call to `{symbol}`, which has no body"));
        };
        self.declare_body(symbol)?;
        let sig = body_signature(self.m, body);
        let by_value = ffi_value_params(self.m, body);
        let mut lowered = Vec::with_capacity(values.len());
        for (index, (value, (_, decl))) in values.into_iter().zip(body.args()).enumerate() {
            let ty = passed_ty(self.m, &by_value, index, decl.ty);
            let (size, align) = self.m.ty.size_align(ty);
            // `[FFI-10]` — an export taking a borrowed value by value gets the value.
            let value = match value {
                Val::Imm(Sc::Ptr, pointer) if ty != decl.ty => Val::Mem(pointer, align),
                value => value,
            };
            lowered.push((value, size, align));
        }
        let values = lowered;
        let (size, _) = self.m.ty.size_align(body.return_ty());
        let dest = dest.map(|(ptr, align)| (ptr, align, size));
        let callee = body_name(self.m, body);
        self.lowered_call(&callee, &sig, &values, dest, "");
        Ok(())
    }

    /// `[FFI-8]` — a call of a variadic C function: its further arguments promoted as C promotes
    /// them (`float` to `double`, a narrower integer to `int`), each passed by the platform's rule.
    fn variadic_call(&mut self, body: &Body, args: &[Operand], dest: &Place) -> Result<()> {
        let symbol = body.symbol.as_str();
        self.declare_body(symbol)?;
        let fixed = body_signature(self.m, body);
        let mut shapes = Vec::with_capacity(args.len());
        let mut values = Vec::with_capacity(args.len());
        let params: Vec<Ty> = body.args().map(|(_, decl)| decl.ty).collect();
        for (index, argument) in args.iter().enumerate() {
            let value = self.operand(argument)?;
            if let Some(&ty) = params.get(index) {
                shapes.push(self.m.ty.shape(ty));
                values.push(value);
                continue;
            }
            let signed = self.operand_ty(argument).is_some_and(|ty| self.m.ty.is_signed(ty));
            let (shape, value) = match value {
                Val::Imm(Sc::F32, value) => (Shape::Scalar(abi::Scalar::F64), Val::Imm(Sc::F64, self.value(format!("fpext float {value} to double")))),
                Val::Imm(Sc::Bool, value) => (Shape::Scalar(abi::Scalar::Int { bytes: 4, signed: true }), Val::Imm(Sc::I(32), self.value(format!("zext i1 {value} to i32")))),
                Val::Imm(Sc::I(bits), value) if bits < 32 => {
                    let wide = self.int_convert(&value, bits, 32, signed);
                    (Shape::Scalar(abi::Scalar::Int { bytes: 4, signed: true }), Val::Imm(Sc::I(32), wide))
                }
                Val::Imm(Sc::Ptr, value) => (Shape::Scalar(abi::Scalar::Ptr), Val::Imm(Sc::Ptr, value)),
                Val::Imm(Sc::F64, value) => (Shape::Scalar(abi::Scalar::F64), Val::Imm(Sc::F64, value)),
                Val::Imm(Sc::I(bits), value) => (Shape::Scalar(abi::Scalar::Int { bytes: (bits / 8) as u8, signed }), Val::Imm(Sc::I(bits), value)),
                value => match self.operand_ty(argument) {
                    Some(ty) => (self.m.ty.shape(ty), value),
                    None => return unsupported("a variadic argument of no type"),
                },
            };
            shapes.push(shape);
            values.push(value);
        }
        let ret = (!self.m.ty.is_zst(body.return_ty())).then(|| self.m.ty.shape(body.return_ty()));
        let sig = abi::classify(self.m.ty.target, ret.as_ref(), &shapes);
        let mut fixed_params = ir_params(&fixed, false);
        fixed_params.push("...".to_string());
        let callee = format!("({}) {}", fixed_params.join(", "), global_name(symbol));
        let values: Vec<(Val, u64, u64)> = values.into_iter().zip(&shapes).map(|(value, shape)| (value, shape.size(), shape.align())).collect();
        let size = self.m.ty.size_align(body.return_ty()).0;
        let dest = self.dest(dest)?;
        self.lowered_call(&callee, &sig, &values, dest.as_ref().map(|(ptr, align)| (ptr.as_str(), *align, size)), "");
        Ok(())
    }

    /// A call through `callee` (a function pointer or a symbol) of the C signature `ret(params)`;
    /// its result written to `dest`.
    pub(crate) fn call_c(&mut self, callee: &str, ret: Option<Ty>, params: &[Param], values: Vec<Val>, dest: Option<(&str, u64)>) {
        let sig = c_fn_signature(self.m, ret, params);
        let values: Vec<(Val, u64, u64)> = values
            .into_iter()
            .zip(params)
            .map(|(value, param)| {
                let shape = param.shape(self.m);
                (value, shape.size(), shape.align())
            })
            .collect();
        let size = ret.map_or(0, |ty| self.m.ty.size_align(ty).0);
        let dest = dest.map(|(ptr, align)| (ptr, align, size));
        self.lowered_call(callee, &sig, &values, dest, "");
    }

    /// A call to the runtime's `name` with `values` by its C prototype; its result written to
    /// `dest` when given.
    pub(crate) fn call_rt(&mut self, name: &str, values: Vec<Val>, dest: Option<(&str, u64)>) {
        let entry = runtime::find(name);
        let ret = runtime::c_type(entry.ret).shape();
        let shapes: Vec<Shape> = entry.params.iter().map(|param| runtime::c_type(param).shape().expect("a parameter has a type")).collect();
        let sig = abi::classify(self.m.ty.target, ret.as_ref(), &shapes);
        let symbol = runtime::symbol(entry, &self.m.shim_namespace);
        if entry.inline {
            self.m.shims.insert(entry.name);
        }
        let ir_name = global_name(&symbol);
        let attrs = if entry.noreturn { " noreturn nounwind cold" } else { " nounwind" };
        self.m.declare(&symbol, || {
            format!("declare{} {} {ir_name}({}){attrs}", ret_attr(&sig), sig.ret.ir(), ir_params(&sig, false).join(", "))
        });
        let values: Vec<(Val, u64, u64)> = values.into_iter().zip(&shapes).map(|(value, shape)| (value, shape.size(), shape.align())).collect();
        let size = ret.as_ref().map_or(0, Shape::size);
        let dest = dest.map(|(ptr, align)| (ptr, align, size));
        self.lowered_call(&ir_name, &sig, &values, dest, "");
        if entry.noreturn {
            self.emit("unreachable");
        }
    }

    /// `ember_loc` for `span`: the file as the build names it, the line and the column
    /// (`ember_loc_unknown()` without one).
    pub(crate) fn loc(&mut self, span: ember_span::Span) -> Val {
        let slot = self.alloca(16, 8);
        if span.is_dummy() || self.m.map.get(span.file).is_none() {
            self.zero(&slot, 8, 16);
            return Val::Mem(slot, 8);
        }
        let file = self.m.map.file(span.file);
        let at = file.line_col(span.start);
        let path = self.m.map.artefact_path(span.file);
        let name = self.m.bytes(path.as_bytes());
        self.emit(format!("store ptr {name}, ptr {slot}, align 8"));
        let line = self.gep(&slot, 8);
        self.emit(format!("store i32 {}, ptr {line}, align 8", at.line));
        let column = self.gep(&slot, 12);
        self.emit(format!("store i32 {}, ptr {column}, align 4", at.col));
        Val::Mem(slot, 8)
    }

    fn panic(&mut self, msg: &AssertKind, span: ember_span::Span) -> Result<()> {
        let loc = self.loc(span);
        match msg {
            AssertKind::Overflow(op) => {
                let text = self.m.bytes(op.spelling().as_bytes());
                self.call_rt("panic_overflow", vec![Val::Imm(Sc::Ptr, text), loc], None);
            }
            AssertKind::SignedDivisionOverflow => {
                let text = self.m.bytes(b"/");
                self.call_rt("panic_overflow", vec![Val::Imm(Sc::Ptr, text), loc], None);
            }
            AssertKind::ShiftTooLarge => {
                let text = self.m.bytes(b"shift");
                self.call_rt("panic_overflow", vec![Val::Imm(Sc::Ptr, text), loc], None);
            }
            AssertKind::DivisionByZero => self.call_rt("panic_div_zero", vec![loc], None),
            AssertKind::Bounds { len, index } => {
                let (index, index_sc) = self.scalar(index)?;
                let (len, len_sc) = self.scalar(len)?;
                let index = self.to_size(&index, index_sc)?;
                let len = self.to_size(&len, len_sc)?;
                self.call_rt("panic_bounds", vec![Val::Imm(Sc::I(64), index), Val::Imm(Sc::I(64), len), loc], None);
            }
            AssertKind::Panic { message } => {
                let message = self.operand(message)?;
                let (base, align) = self.spill(&message, 16, 8);
                let ptr = self.load(Sc::Ptr, &base, align);
                let len_at = self.gep(&base, 8);
                let len = self.load(Sc::I(64), &len_at, align.min(8));
                self.call_rt("panic", vec![Val::Imm(Sc::Ptr, ptr), Val::Imm(Sc::I(64), len), loc], None);
            }
            AssertKind::RefCellBorrow { file, line } => {
                let (file, _) = self.scalar(file)?;
                let (line, line_sc) = self.scalar(line)?;
                let line = match line_sc {
                    Sc::I(bits) => self.int_convert(&line, bits, 32, false),
                    _ => return unsupported("a `RefCell` line that is not an integer"),
                };
                self.call_rt("panic_refcell", vec![Val::Imm(Sc::Ptr, file), Val::Imm(Sc::I(32), line), loc], None);
            }
            AssertKind::Downcast => {
                let text = self.m.bytes(b"invalid downcast");
                self.call_rt("panic", vec![Val::Imm(Sc::Ptr, text), Val::Imm(Sc::I(64), "16".to_string()), loc], None);
            }
        }
        Ok(())
    }

    /// An integer as a `size_t`.
    fn to_size(&mut self, value: &str, sc: Sc) -> Result<String> {
        match sc {
            Sc::I(bits) => Ok(self.int_convert(value, bits, 64, false)),
            other => unsupported(format!("a {} as a size", other.ir())),
        }
    }

    fn builtin(&mut self, which: &Builtin, arg_ty: Ty, args: &[Operand], dest: &Place, span: ember_span::Span) -> Result<()> {
        let _ = span;
        match which {
            Builtin::Print | Builtin::Println | Builtin::EPrint | Builtin::EPrintln | Builtin::PrintPiece { .. } => self.print(which, arg_ty, args),
            // `[CLS-1]` — the object's header from the runtime; `[OPT-1]` in the frame when the
            // compiler proved no handle outlives the function.
            Builtin::ClassNew { class_id, .. } => {
                let handle = if dest.projection.is_empty() && self.mir().frame_objects.contains(&dest.local) {
                    let info = global_name(&crate::glue::type_info(self.m, *class_id));
                    let slot = self.alloca(8, 8);
                    let storage = format!("%fo{}", dest.local.0);
                    self.call_rt("obj_new_frame", vec![Val::Imm(Sc::Ptr, storage), Val::Imm(Sc::Ptr, info)], Some((&slot, 8)));
                    self.load(Sc::Ptr, &slot, 8)
                } else {
                    self.class_new(*class_id)
                };
                let at = self.place(dest)?;
                self.store_scalar(Sc::Ptr, &handle, &at.ptr, at.align);
                Ok(())
            }
            // `[DSP-4]` — the runtime walks the object's base chain; MIR retains the result.
            Builtin::ClassDowncast { target, .. } => {
                let (object, _) = self.scalar(&args[0])?;
                let info = global_name(&crate::glue::type_info(self.m, *target));
                let dest = self.dest(dest)?;
                self.call_rt("downcast", vec![Val::Imm(Sc::Ptr, object), Val::Imm(Sc::Ptr, info)], dest.as_ref().map(|(p, a)| (p.as_str(), *a)));
                Ok(())
            }
            // An empty list; ADR-141's in its frame buffer when it has one.
            Builtin::ArrayNew | Builtin::StringNew => {
                let at = self.place(dest)?;
                if dest.projection.is_empty()
                    && let Some(index) = self.mir().stack_buffers.iter().position(|buffer| buffer.local == dest.local)
                {
                    let capacity = self.mir().stack_buffers[index].capacity;
                    return self.stack_vec(&at.ptr, &format!("%sb{index}"), capacity);
                }
                self.zero(&at.ptr, 8, 24);
                Ok(())
            }
            // `[HR-33]` — a poll the program makes itself; a runtime without reload answers that it
            // has none.
            Builtin::HotCheckpoint => {
                let status = self.alloca(4, 4);
                self.call_rt("reload_checkpoint", Vec::new(), Some((&status, 4)));
                Ok(())
            }
            other => self.builtin_call(other, arg_ty, args, dest, span),
        }
    }

    /// `[HR-3b]` — this body's frame record pushed on its thread's chain as the body starts:
    /// `{ prev, &info, site }`, the head of the chain `ember_reload_frames()`'s.
    fn push_frame_record(&mut self, info: &ember_codegen_c::image::FrameInfo) {
        let record = self.alloca(24, 8);
        let head = self.alloca(8, 8);
        let body = std::mem::take(&mut self.out);
        self.call_rt("reload_frames", Vec::new(), Some((&head, 8)));
        let chain = self.load(Sc::Ptr, &head, 8);
        let previous = self.load(Sc::Ptr, &chain, 8);
        self.emit(format!("store ptr {previous}, ptr {record}, align 8"));
        let info_at = self.gep(&record, 8);
        self.emit(format!("store ptr {}, ptr {info_at}, align 8", global_name(&info.symbol)));
        let site_at = self.gep(&record, 16);
        self.emit(format!("store i32 0, ptr {site_at}, align 8"));
        self.emit(format!("store ptr {record}, ptr {chain}, align 8"));
        let pushed = std::mem::replace(&mut self.out, body);
        self.prologue.push_str(&pushed);
        self.frame_record = Some(crate::reload::FrameRecord { head, record, sites: info.by_block.clone() });
    }

    /// ADR-141 — `ember_stack_vec(buffer, cap)`: the list in its frame buffer where the runtime
    /// can tell the buffer from a heap block (Windows x64 always; Linux when the buffer is in this
    /// thread's stack range), else empty.
    fn stack_vec(&mut self, to: &str, buffer: &str, capacity: u64) -> Result<()> {
        self.zero(to, 8, 24);
        match self.m.ty.target {
            abi::Target::Win64 => {
                self.emit(format!("store ptr {buffer}, ptr {to}, align 8"));
                let cap = self.gep(to, 16);
                self.emit(format!("store i64 {capacity}, ptr {cap}, align 8"));
            }
            abi::Target::SysV | abi::Target::Aapcs64 => {
                let (low, high) = ("@ember_stack_low", "@ember_stack_high");
                self.m.declare(low, || format!("{low} = external thread_local global i64, align 8"));
                self.m.declare(high, || format!("{high} = external thread_local global i64, align 8"));
                let known = self.load(Sc::I(64), high, 8);
                let (find, check) = (self.label(), self.label());
                let unset = self.value(format!("icmp eq i64 {known}, 0"));
                self.emit(format!("br i1 {unset}, label %{find}, label %{check}"));
                self.start(&find);
                self.call_rt("stack_find", Vec::new(), None);
                self.emit(format!("br label %{check}"));
                self.start(&check);
                let low_value = self.load(Sc::I(64), low, 8);
                let high_value = self.load(Sc::I(64), high, 8);
                let address = self.value(format!("ptrtoint ptr {buffer} to i64"));
                let from_low = self.value(format!("sub i64 {address}, {low_value}"));
                let span = self.value(format!("sub i64 {high_value}, {low_value}"));
                let inside = self.value(format!("icmp ult i64 {from_low}, {span}"));
                let (yes, done) = (self.label(), self.label());
                self.emit(format!("br i1 {inside}, label %{yes}, label %{done}"));
                self.start(&yes);
                self.emit(format!("store ptr {buffer}, ptr {to}, align 8"));
                let cap = self.gep(to, 16);
                self.emit(format!("store i64 {capacity}, ptr {cap}, align 8"));
                self.emit(format!("br label %{done}"));
                self.start(&done);
            }
        }
        Ok(())
    }

    /// `[STD-9]` — `print` and its kin through the runtime's printer for the argument's type. A
    /// print statement's piece (D-900) is written with its stream held: its first piece takes it,
    /// its last lets it go; a printer of its own holds it for itself.
    fn print(&mut self, which: &Builtin, arg_ty: Ty, args: &[Operand]) -> Result<()> {
        let (printer, piece) = match *which {
            Builtin::Println => ("println", None),
            Builtin::EPrintln => ("eprintln", None),
            Builtin::EPrint => ("eprint", None),
            Builtin::PrintPiece { stderr, first, last } => (if stderr { "eprint" } else { "print" }, Some((first, last))),
            _ => ("print", None),
        };
        let stream = Val::Imm(Sc::I(32), u8::from(printer.starts_with('e')).to_string());
        let value = self.operand(&args[0])?;
        if let Some((true, _)) = piece {
            self.call_rt("print_hold", vec![stream.clone()], None);
        }
        self.print_value(printer, piece.is_some(), value, arg_ty)?;
        if let Some((_, true)) = piece {
            self.call_rt("print_release", vec![stream], None);
        }
        Ok(())
    }
}

/// Whether writing `to` may overwrite part of `from` before it is read whole.
fn may_overlap(to: &Place, from: &Place) -> bool {
    to.local == from.local
        || to.projection.iter().any(|step| matches!(step, Projection::Deref | Projection::Index(_) | Projection::ConstIndex(_)))
        || from.projection.iter().any(|step| matches!(step, Projection::Deref | Projection::Index(_) | Projection::ConstIndex(_)))
}

pub(crate) fn shared_inner(table: &ember_types::TypeTable, id: ember_types::StructId) -> Option<Ty> {
    let def = table.struct_def(id);
    match &def.origin {
        Some((name, args)) if (name.is("Shared") || name.is("SyncShared")) && args.len() == 1 => Some(args[0]),
        _ => None,
    }
}

/// The ABI part a register scalar is, unchanged.
fn sc_part(sc: Sc) -> Option<PartTy> {
    Some(match sc {
        Sc::Bool => PartTy::I(1),
        Sc::I(bits) if bits <= 64 => PartTy::I(bits),
        Sc::F32 => PartTy::F32,
        Sc::F64 => PartTy::F64,
        Sc::F80 => PartTy::F80,
        Sc::F128 => PartTy::F128,
        Sc::Ptr => PartTy::Ptr,
        Sc::I(_) => return None,
    })
}

fn float_suffix(sc: Sc) -> &'static str {
    match sc {
        Sc::F32 => "f32",
        Sc::F64 => "f64",
        _ => "f80",
    }
}

/// An integer constant of `bits` from its two's-complement bits, as IR writes it: signed decimal.
/// A 256-bit constant is a sign-extended 128-bit one (`ember_i256_make`).
pub(crate) fn int_literal(value: u128, bits: u32) -> String {
    if bits > 128 {
        return (value as i128).to_string();
    }
    if bits == 128 {
        return (value as i128).to_string();
    }
    let mask = (1u128 << bits) - 1;
    let low = value & mask;
    if bits > 1 && low >> (bits - 1) == 1 {
        let negative = low as i128 - (1i128 << bits);
        negative.to_string()
    } else if bits == 1 {
        if low == 1 { "true".to_string() } else { "false".to_string() }
    } else {
        low.to_string()
    }
}

/// A float constant as IR writes it: the `double` bits of its value, rounded to `float` first
/// for an `f32`.
pub(crate) fn float_literal(value: f64, sc: Sc) -> String {
    let value = if sc == Sc::F32 { value as f32 as f64 } else { value };
    format!("0x{:016X}", value.to_bits())
}

/// `[FN-8]` — the C entry point: the runtime initialised, Ember's `main` called (through the
/// checker's `main_entry` when it takes `args` or returns a status), the runtime shut down.
pub(crate) fn emit_entry(m: &mut Module<'_>, bodies: &[Body], options: &Options<'_>, tests: Option<&[String]>) -> Result<()> {
    let entry_symbol = ember_branding::entry_symbol();
    let main_entry = ember_branding::generated("main_entry");
    let (allocate, deallocate) = ember_branding::global_allocator_glue();
    if bodies.iter().any(|body| [&allocate, &deallocate].contains(&&body.name.rsplit('.').next().unwrap_or_default().to_string())) {
        return unsupported("a global allocator ([ALC-3])");
    }
    let entry = bodies.iter().any(|body| body.symbol == main_entry);
    let called = if entry { main_entry.clone() } else { entry_symbol.clone() };
    let called_body = m.bodies.get(called.as_str()).copied();
    if tests.is_none() && called_body.is_none() {
        return unsupported("a program without `main`");
    }
    // The C `main` is a frame of the backtrace too, at the Ember `main`'s declaration.
    let optimized = m.profile != Profile::Debug;
    let declared = called_body.map_or(ember_span::Span::DUMMY, |body| body.span);
    let (subprogram, line) = m.debug.subprogram(m.map, "main", declared, optimized);
    let mut cx = FnCx {
        m,
        body: None,
        allocas: String::new(),
        prologue: String::new(),
        out: String::new(),
        next: 0,
        addr: Vec::new(),
        sig: Signature { ret: Return::Void, params: Vec::new(), conv: abi::Conv::C },
        block: "entry".to_string(),
        span: ember_span::Span::DUMMY,
        scope: Some((subprogram, line)),
        locations: HashMap::new(),
        folded: HashMap::new(),
        frame_record: None,
    };
    // `[TST-3]` — a test binary runs the test its one argument numbers (`2` for anything else),
    // its crash a status rather than Windows' error window (D-252).
    let which = match tests {
        Some(tests) => Some(test_argument(&mut cx, tests.len())),
        None => None,
    };
    // `ember_rt_config_default()`: its size, and nothing else set.
    let cfg = cx.alloca(config::SIZE, 8);
    cx.zero(&cfg, 8, config::SIZE);
    cx.emit(format!("store i64 {}, ptr {cfg}, align 8", config::SIZE));
    if options.leak_check {
        let flags = cx.gep(&cfg, config::FLAGS);
        cx.emit(format!("store i32 {}, ptr {flags}, align 8", config::DEBUG_OBJECTS));
    }
    if which.is_some() {
        // A test's panic is marked for the runner.
        let on_panic = cx.gep(&cfg, config::ON_PANIC);
        cx.emit(format!("store ptr @ember_ll_test_on_panic, ptr {on_panic}, align 8"));
        cx.call_rt("rt_set_args", vec![Val::Imm(Sc::I(32), "1".to_string()), Val::Imm(Sc::Ptr, "%argv".to_string())], None);
    } else {
        cx.call_rt("rt_set_args", vec![Val::Imm(Sc::I(32), "%argc".to_string()), Val::Imm(Sc::Ptr, "%argv".to_string())], None);
    }
    cx.call_rt("rt_init_v3", vec![Val::Imm(Sc::Ptr, cfg.clone()), Val::Imm(Sc::I(32), runtime::RUNTIME_ABI.to_string())], None);
    let (named, sites) = if options.profile != Profile::Release { frame_tables(cx.m, bodies) } else { (0, 0) };
    if named > 0 {
        cx.call_rt("backtrace_names", vec![Val::Imm(Sc::Ptr, "@em_frame_names".to_string()), Val::Imm(Sc::I(64), named.to_string())], None);
        if sites > 0 {
            cx.call_rt("backtrace_sites", vec![Val::Imm(Sc::Ptr, "@em_call_sites".to_string()), Val::Imm(Sc::I(64), sites.to_string())], None);
        }
    }
    let coded = code_names(cx.m, bodies);
    if coded > 0 {
        cx.call_rt("stack_names", vec![Val::Imm(Sc::Ptr, "@em_code_names".to_string()), Val::Imm(Sc::I(64), coded.to_string())], None);
    }
    // `[WK-8]` — the leak check's ownership edges.
    if options.leak_check {
        cx.register_edges()?;
    }
    if let (Some(tests), Some(which)) = (tests, which) {
        let done = cx.label();
        let labels: Vec<String> = tests.iter().map(|_| cx.label()).collect();
        let cases: Vec<String> = labels.iter().enumerate().map(|(index, label)| format!("i64 {index}, label %{label}")).collect();
        cx.emit(format!("switch i64 {which}, label %{done} [ {} ]", cases.join(" ")));
        for (test, label) in tests.iter().zip(&labels) {
            cx.start(label);
            cx.call_body(test, Vec::new(), None)?;
            cx.emit(format!("br label %{done}"));
        }
        cx.start(&done);
        cx.emit("call void @ember_ll_test_end()");
        cx.call_rt("rt_shutdown", Vec::new(), None);
        cx.emit("ret i32 0");
        let attrs = fn_attrs(cx.m);
        let text = format!("define {} {attrs} !dbg !{subprogram} {{\nentry:\n{}{}}}\n\n", entry_head(cx.m), cx.allocas, cx.out);
        cx.m.fns.push_str(&text);
        return Ok(());
    }
    let called_body = called_body.expect("checked above");
    cx.declare_body(&called)?;
    let sig = body_signature(cx.m, called_body);
    let ret_ty = called_body.return_ty();
    let status = if entry {
        let Repr::Scalar(Sc::I(bits)) = cx.m.ty.repr(ret_ty) else {
            return unsupported("a `main_entry` whose status is not an integer");
        };
        let slot = cx.alloca(bits as u64 / 8, (bits as u64 / 8).max(1));
        cx.lowered_call(&global_name(&called), &sig, &[], Some((&slot, (bits as u64 / 8).max(1), bits as u64 / 8)), "");
        let value = cx.load(Sc::I(bits), &slot, (bits as u64 / 8).max(1));
        cx.int_convert(&value, bits, 32, true)
    } else {
        cx.lowered_call(&global_name(&called), &sig, &[], None, "");
        "0".to_string()
    };
    cx.call_rt("rt_shutdown", Vec::new(), None);
    cx.emit(format!("ret i32 {status}"));
    let attrs = fn_attrs(cx.m);
    let text = format!("define {} {attrs} !dbg !{subprogram} {{\nentry:\n{}{}}}\n\n", entry_head(cx.m), cx.allocas, cx.out);
    cx.m.fns.push_str(&text);
    Ok(())
}

/// `[FFI-33]` — a shared library's creator: the thread of its successful `ember_module_init`, 0
/// before it and after `ember_module_shutdown`.
fn module_creator(m: &mut Module<'_>) -> String {
    let name = global_name(&format!("{}module_creator", ember_branding::mangle_prefix()));
    m.declare(&name, || format!("{name} = internal global i32 0, align 4"));
    name
}

/// `[FFI-28]`, `[FFI-31]`, `[ABI-1]`, `[ABI-3]`, `[FFI-33]` — a shared library's entry points, as the
/// C backend writes them (`emit_module_entry`): the marker of the module protocol it follows;
/// `ember_module_init`, which refuses a runtime of another ABI version before anything else, then
/// initialises the module in its runtime (its own copy, or the one modules share) with the host's
/// config, as much of it as the host's `size` covers and the defaults past it, and records its
/// creator thread, a second call while initialised changing nothing; `ember_module_shutdown`, which
/// shuts the module down in its runtime (the runtime's own shutdown when it is the last module).
pub(crate) fn emit_module_entry(m: &mut Module<'_>, leak_check: bool) -> Result<()> {
    let creator = module_creator(m);
    m.global(format!(
        "{} = constant i32 {}, align 4",
        global_name(&ember_branding::runtime("module_protocol_v1")),
        runtime::MODULE_PROTOCOL
    ));
    m.declare("llvm.umin.i64", || "declare i64 @llvm.umin.i64(i64, i64)".to_string());
    m.declare("llvm.memcpy.p0.p0.i64", || "declare void @llvm.memcpy.p0.p0.i64(ptr, ptr, i64, i1)".to_string());
    let mut cx = FnCx::glue(m);
    let version = cx.alloca(4, 4);
    cx.call_rt("rt_abi_version", Vec::new(), Some((&version, 4)));
    let version = cx.load(Sc::I(32), &version, 4);
    let same = cx.value(format!("icmp eq i32 {version}, {} ; EMBER_RUNTIME_ABI", runtime::RUNTIME_ABI));
    let (refused, checked) = (cx.label(), cx.label());
    cx.emit(format!("br i1 {same}, label %{checked}, label %{refused}"));
    cx.start(&refused);
    cx.emit(format!("ret i32 {}", runtime::MODULE_INIT_ABI_MISMATCH));
    cx.start(&checked);
    let current = cx.load(Sc::I(32), &creator, 4);
    let initialised = cx.value(format!("icmp ne i32 {current}, 0"));
    let (again, fresh) = (cx.label(), cx.label());
    cx.emit(format!("br i1 {initialised}, label %{again}, label %{fresh}"));
    cx.start(&again);
    cx.emit("ret i32 0");
    cx.start(&fresh);
    let settings = cx.alloca(config::SIZE, 8);
    cx.zero(&settings, 8, config::SIZE);
    cx.emit(format!("store i64 {}, ptr {settings}, align 8", config::SIZE));
    let given = cx.value("icmp ne ptr %p0, null");
    let (copy, configured) = (cx.label(), cx.label());
    cx.emit(format!("br i1 {given}, label %{copy}, label %{configured}"));
    cx.start(&copy);
    let size = cx.load(Sc::I(64), "%p0", 8);
    let count = cx.value(format!("call i64 @llvm.umin.i64(i64 {size}, i64 {})", config::SIZE));
    cx.emit(format!("call void @llvm.memcpy.p0.p0.i64(ptr align 8 {settings}, ptr align 8 %p0, i64 {count}, i1 false)"));
    cx.emit(format!("store i64 {}, ptr {settings}, align 8", config::SIZE));
    cx.emit(format!("br label %{configured}"));
    cx.start(&configured);
    if leak_check {
        let flags = cx.gep(&settings, config::FLAGS);
        let set = cx.load(Sc::I(32), &flags, 8);
        let with = cx.value(format!("or i32 {set}, {}", config::DEBUG_OBJECTS));
        cx.emit(format!("store i32 {with}, ptr {flags}, align 8"));
    }
    let status = cx.alloca(4, 4);
    cx.call_rt(
        "rt_module_init_v3",
        vec![Val::Imm(Sc::Ptr, settings.clone()), Val::Imm(Sc::I(32), runtime::RUNTIME_ABI.to_string())],
        Some((&status, 4)),
    );
    let status = cx.load(Sc::I(32), &status, 4);
    let failed = cx.value(format!("icmp ne i32 {status}, 0"));
    let (stopped, started) = (cx.label(), cx.label());
    cx.emit(format!("br i1 {failed}, label %{stopped}, label %{started}"));
    cx.start(&stopped);
    cx.emit(format!("ret i32 {status}"));
    cx.start(&started);
    if leak_check {
        cx.register_edges()?;
    }
    let thread = cx.alloca(4, 4);
    cx.call_rt("rt_thread_id", Vec::new(), Some((&thread, 4)));
    let thread = cx.load(Sc::I(32), &thread, 4);
    cx.emit(format!("store i32 {thread}, ptr {creator}, align 4"));
    cx.emit("ret i32 0");
    let attrs = fn_attrs(cx.m);
    let text = format!(
        "define i32 {}(ptr %p0) {attrs} {{\nentry:\n{}  br label %start\nstart:\n{}}}\n\n",
        global_name(&ember_branding::runtime("module_init")),
        cx.allocas,
        cx.out
    );
    cx.m.fns.push_str(&text);
    let mut cx = FnCx::glue(m);
    let current = cx.load(Sc::I(32), &creator, 4);
    let initialised = cx.value(format!("icmp ne i32 {current}, 0"));
    let (never, end) = (cx.label(), cx.label());
    cx.emit(format!("br i1 {initialised}, label %{end}, label %{never}"));
    cx.start(&never);
    cx.emit("ret void");
    cx.start(&end);
    cx.emit(format!("store i32 0, ptr {creator}, align 4"));
    cx.call_rt("rt_module_shutdown", Vec::new(), None);
    cx.emit("ret void");
    let text = format!(
        "define void {}() {attrs} {{\nentry:\n{}  br label %start\nstart:\n{}}}\n\n",
        global_name(&ember_branding::runtime("module_shutdown")),
        cx.allocas,
        cx.out
    );
    cx.m.fns.push_str(&text);
    Ok(())
}

/// The C entry's name and parameters: `main`, or in a reloadable image the image's own entry, which
/// the runtime calls (`[HR-33]`).
fn entry_head(m: &Module<'_>) -> String {
    if m.reload.is_some() {
        format!("internal i32 {}(i32 %argc, ptr %argv)", global_name(&ember_codegen_c::image::entry_symbol()))
    } else {
        "i32 @main(i32 %argc, ptr %argv)".to_string()
    }
}

/// `[TST-3]` — a test binary's argument: the number of the test it runs, in decimal, below
/// `count`; anything else ends the binary with status `2`. The C its runner needs (the error mode,
/// the markers on standard error) is the module's shim's.
fn test_argument(cx: &mut FnCx<'_, '_>, count: usize) -> String {
    let panicked = format!("{}-test: panicked", ember_branding::SYMBOL_PREFIX);
    let returned = format!("{}-test: returned", ember_branding::SYMBOL_PREFIX);
    let shim = format!(
        "\n/* [TST-3] what a test binary's runner reads. */\n\
         #if defined(_WIN32)\n__declspec(dllimport) unsigned int __stdcall SetErrorMode(unsigned int mode);\n#endif\n\
         void ember_ll_test_quiet(void) {{\n#if defined(_WIN32)\n    SetErrorMode(0x0001u | 0x0002u);\n#endif\n}}\n\
         void ember_ll_test_on_panic(const char* msg, size_t len) {{ (void)msg; (void)len; fputs(\"{panicked}\\n\", stderr); }}\n\
         void ember_ll_test_end(void) {{ fflush(stdout); fputs(\"{returned}\\n\", stderr); }}\n"
    );
    cx.m.shim_functions.insert("ember_ll_test".to_string(), shim);
    cx.m.declare("ember_ll_test_quiet", || "declare void @ember_ll_test_quiet()".to_string());
    cx.m.declare("ember_ll_test_on_panic", || "declare void @ember_ll_test_on_panic(ptr, i64)".to_string());
    cx.m.declare("ember_ll_test_end", || "declare void @ember_ll_test_end()".to_string());
    cx.emit("call void @ember_ll_test_quiet()");
    let (bad, read, head, step, end) = (cx.label(), cx.label(), cx.label(), cx.label(), cx.label());
    let few = cx.value("icmp sle i32 %argc, 1");
    cx.emit(format!("br i1 {few}, label %{bad}, label %{read}"));
    cx.start(&read);
    let at = cx.value("getelementptr ptr, ptr %argv, i64 1");
    let text = cx.value(format!("load ptr, ptr {at}, align 8"));
    let first = cx.value(format!("load i8, ptr {text}, align 1"));
    let empty = cx.value(format!("icmp eq i8 {first}, 0"));
    let entry = cx.block_label();
    cx.emit(format!("br i1 {empty}, label %{bad}, label %{head}"));
    cx.start(&head);
    let (which, digit, next_which, next_digit) = (cx.tmp(), cx.tmp(), cx.tmp(), cx.tmp());
    cx.emit(format!("{which} = phi i64 [ 0, %{entry} ], [ {next_which}, %{step} ]"));
    cx.emit(format!("{digit} = phi ptr [ {text}, %{entry} ], [ {next_digit}, %{step} ]"));
    let c = cx.value(format!("load i8, ptr {digit}, align 1"));
    let low = cx.value(format!("icmp uge i8 {c}, 48"));
    let high = cx.value(format!("icmp ule i8 {c}, 57"));
    let small = cx.value(format!("icmp slt i64 {which}, 1000000000"));
    let is_digit = cx.value(format!("and i1 {low}, {high}"));
    let goes_on = cx.value(format!("and i1 {is_digit}, {small}"));
    cx.emit(format!("br i1 {goes_on}, label %{step}, label %{end}"));
    cx.start(&step);
    let wide = cx.value(format!("zext i8 {c} to i64"));
    let value = cx.value(format!("sub i64 {wide}, 48"));
    let tens = cx.value(format!("mul i64 {which}, 10"));
    cx.emit(format!("{next_which} = add i64 {tens}, {value}"));
    cx.emit(format!("{next_digit} = getelementptr i8, ptr {digit}, i64 1"));
    cx.emit(format!("br label %{head}"));
    cx.start(&end);
    let finished = cx.value(format!("icmp eq i8 {c}, 0"));
    let known = cx.value(format!("icmp slt i64 {which}, {count}"));
    let good = cx.value(format!("and i1 {finished}, {known}"));
    let run = cx.label();
    cx.emit(format!("br i1 {good}, label %{run}, label %{bad}"));
    cx.start(&bad);
    cx.emit("ret i32 2");
    cx.start(&run);
    which
}

/// `[RT-4]`, `[CG-C-10]` — outside `release`, each function's C symbol with its Ember path and
/// declaration, and each call one makes of another: what a panic's backtrace names frames by.
fn frame_tables(m: &mut Module<'_>, bodies: &[Body]) -> (usize, usize) {
    let mut entries: Vec<(String, String, String, u32)> = Vec::new();
    let mut sites: Vec<(String, String, String, u32, u32)> = Vec::new();
    for body in bodies {
        if body.is_abstract || body.is_extern_declaration || body.blocks.is_empty() || body.span.is_dummy() {
            continue;
        }
        let Some(file) = m.map.get(body.span.file) else { continue };
        let line = file.line_col(body.span.start).line;
        let path = m.map.artefact_path(body.span.file);
        let name = if body.is_lambda { "a lambda".to_string() } else { body.path.clone().unwrap_or_else(|| body.name.clone()) };
        entries.push((body.symbol.clone(), name, path, line));
        for block in &body.blocks {
            let Terminator::Call { func, .. } = &block.terminator else { continue };
            let callee = match func {
                FuncRef::Direct { symbol, .. } => symbol.clone(),
                FuncRef::Builtin { .. } => continue,
                _ => String::new(),
            };
            let span = block.terminator_span;
            if span.is_dummy() {
                continue;
            }
            let Some(file) = m.map.get(span.file) else { continue };
            let at = file.line_col(span.start);
            sites.push((body.symbol.clone(), callee, m.map.artefact_path(span.file), at.line, at.col));
        }
    }
    entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    entries.dedup_by(|a, b| a.0 == b.0);
    sites.sort();
    sites.dedup();
    if !entries.is_empty() {
        let rows: Vec<String> = entries
            .iter()
            .map(|(symbol, name, path, line)| {
                let (symbol, name, path) = (m.bytes(symbol.as_bytes()), m.bytes(name.as_bytes()), m.bytes(path.as_bytes()));
                format!("{{ ptr, ptr, ptr, i32 }} {{ ptr {symbol}, ptr {name}, ptr {path}, i32 {line} }}")
            })
            .collect();
        m.global(format!(
            "@em_frame_names = internal constant [{} x {{ ptr, ptr, ptr, i32 }}] [{}], align 8",
            rows.len(),
            rows.join(", ")
        ));
    }
    if !sites.is_empty() {
        let rows: Vec<String> = sites
            .iter()
            .map(|(caller, callee, path, line, column)| {
                let (caller, callee, path) = (m.bytes(caller.as_bytes()), m.bytes(callee.as_bytes()), m.bytes(path.as_bytes()));
                format!("{{ ptr, ptr, ptr, i32, i32 }} {{ ptr {caller}, ptr {callee}, ptr {path}, i32 {line}, i32 {column} }}")
            })
            .collect();
        m.global(format!(
            "@em_call_sites = internal constant [{} x {{ ptr, ptr, ptr, i32, i32 }}] [{}], align 8",
            rows.len(),
            rows.join(", ")
        ));
    }
    (entries.len(), sites.len())
}

/// `[RT-12]` — each function the program can reach by its code's address, in every profile, for
/// the name a stack overflow prints.
fn code_names(m: &mut Module<'_>, bodies: &[Body]) -> usize {
    let mut entries: Vec<(String, String)> = bodies
        .iter()
        .filter(|body| !body.is_abstract && !body.is_extern_declaration && !body.blocks.is_empty() && !body.span.is_dummy())
        .map(|body| {
            let name = if body.is_lambda { "a lambda".to_string() } else { body.path.clone().unwrap_or_else(|| body.name.clone()) };
            (body.symbol.clone(), name)
        })
        .collect();
    entries.sort();
    entries.dedup_by(|a, b| a.0 == b.0);
    if entries.is_empty() {
        return 0;
    }
    let rows: Vec<String> = entries
        .iter()
        .map(|(symbol, name)| {
            let name = m.bytes(name.as_bytes());
            format!("{{ ptr, ptr }} {{ ptr {}, ptr {name} }}", global_name(symbol))
        })
        .collect();
    m.global(format!("@em_code_names = internal constant [{} x {{ ptr, ptr }}] [{}], align 8", rows.len(), rows.join(", ")));
    entries.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int_literals_are_signed_decimal() {
        assert_eq!(int_literal(255, 8), "-1");
        assert_eq!(int_literal(127, 8), "127");
        assert_eq!(int_literal(u64::MAX as u128, 64), "-1");
        assert_eq!(int_literal(1 << 63, 64), "-9223372036854775808");
        assert_eq!(int_literal(u128::MAX, 128), "-1");
        assert_eq!(int_literal(5, 32), "5");
        assert_eq!(float_literal(1.5, Sc::F64), "0x3FF8000000000000");
        assert_eq!(float_literal(0.1, Sc::F32), "0x3FB99999A0000000");
    }
}
