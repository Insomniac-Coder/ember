//! One MIR body as one IR function.
//!
//! Every local lives in a stack slot of its layout (an `alloca` in the entry block, which
//! `mem2reg` and SROA promote at `-O1` and above), a place is a pointer reached by byte offsets,
//! and a scalar operand is loaded into a register. Arguments and results cross by the platform's C
//! ABI (`crate::abi`): a value passed behind a pointer is the callee's own copy, so the parameter
//! is that copy's slot, and an `sret` result is written in place.

use std::fmt::Write as _;

use crate::abi::{self, Part, PartTy, Pass, Return, Shape, Signature};
use crate::runtime;
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
}

/// The ABI signature of a function returning `ret` and taking `params`.
fn signature(m: &Module<'_>, ret: Ty, params: impl Iterator<Item = Ty>) -> Signature {
    let ret = (!m.ty.is_zst(ret)).then(|| m.ty.shape(ret));
    let params: Vec<Shape> = params.map(|ty| m.ty.shape(ty)).collect();
    abi::classify(m.ty.target, ret.as_ref(), &params)
}

pub(crate) fn body_signature(m: &Module<'_>, body: &Body) -> Signature {
    signature(m, body.return_ty(), body.args().map(|(_, decl)| decl.ty))
}

/// Declares a body of this program the module calls (its definition, when it has one, removes
/// the declaration).
pub(crate) fn declare_body_symbol(m: &mut Module<'_>, symbol: &str) -> Result<()> {
    let Some(body) = m.bodies.get(symbol).copied() else {
        return unsupported(format!("a call to `{symbol}`, which has no body"));
    };
    let sig = body_signature(m, body);
    let name = global_name(symbol);
    let attrs = fn_attrs(m);
    m.declare(symbol, || {
        let ret = sig.ret.ir();
        let ext = ret_attr(&sig);
        format!("declare{ext} {ret} {name}({}) {attrs}", ir_params(&sig, false).join(", "))
    });
    Ok(())
}

/// The IR parameter list of `sig`, `%p<N>` named when `named`.
fn ir_params(sig: &Signature, named: bool) -> Vec<String> {
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
    if let Return::Sret { size, align } = sig.ret {
        name(&mut out, format!("ptr sret([{size} x i8]) align {align}"));
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
fn ret_with_attr(sig: &Signature) -> String {
    match ret_attr(sig) {
        "" => sig.ret.ir(),
        ext => format!("{} {}", ext.trim(), sig.ret.ir()),
    }
}

fn ret_attr(sig: &Signature) -> &'static str {
    match &sig.ret {
        Return::Direct(parts) if parts.len() == 1 => parts[0].ext.attr(),
        _ => "",
    }
}

/// The attributes every function of the program has: no unwinding (a panic aborts), unwind
/// tables for the backtrace and the debugger, and on Linux stack probes (`[RT-12]`; Windows
/// probes every frame larger than a page itself).
pub(crate) fn fn_attrs(m: &Module<'_>) -> &'static str {
    match m.ty.target {
        abi::Target::SysV => "nounwind uwtable \"probe-stack\"=\"inline-asm\"",
        abi::Target::Win64 => "nounwind uwtable",
    }
}

pub(crate) fn emit_body<'a>(m: &mut Module<'a>, body: &'a Body) -> Result<()> {
    if body.variadic {
        return unsupported("a variadic function definition");
    }
    if body.abi.as_deref().is_some_and(|abi| abi != "C") {
        return unsupported(format!("the calling convention `{}`", body.abi.as_deref().unwrap_or_default()));
    }
    let sig = body_signature(m, body);
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
    };
    cx.frame()?;
    for (index, block) in body.blocks.iter().enumerate() {
        cx.start(&format!("bb{index}"));
        cx.stmts(&block.stmts)?;
        cx.terminator(&block.terminator, block.terminator_span)?;
    }
    let mut attrs = fn_attrs(cx.m).to_string();
    if body.inline.always {
        attrs.push_str(" alwaysinline");
    } else if body.inline.never {
        attrs.push_str(" noinline");
    }
    if body.inline.cold {
        attrs.push_str(" cold");
    }
    if body.inline.hot {
        attrs.push_str(" hot");
    }
    let linkage = if body.abi.is_some() { "" } else { "internal " };
    let header = format!(
        "define {linkage}{} {}({}) {attrs} {{",
        ret_with_attr(&cx.sig),
        global_name(&body.symbol),
        ir_params(&cx.sig, true).join(", ")
    );
    let text = format!("{header}\nentry:\n{}{}  br label %bb0\n{}}}\n\n", cx.allocas, cx.prologue, cx.out);
    cx.m.fns.push_str(&text);
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
    /// The runtime's `ember_loc`.
    Loc,
}

impl Param {
    fn shape(self, m: &Module<'_>) -> Shape {
        match self {
            Param::Ty(ty) => m.ty.shape(ty),
            Param::Ptr => Shape::Scalar(abi::Scalar::Ptr),
            Param::Int => Shape::Scalar(abi::Scalar::Int { bytes: 4, signed: true }),
            Param::Loc => runtime::c_type("$loc").shape().expect("ember_loc has a shape"),
        }
    }

    fn repr(self, m: &Module<'_>) -> Repr {
        match self {
            Param::Ty(ty) => m.ty.repr(ty),
            Param::Ptr => Repr::Scalar(Sc::Ptr),
            Param::Int => Repr::Scalar(Sc::I(32)),
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
        (Return::Direct(_), Some(ty)) => {
            let (size, align) = cx.m.ty.size_align(ty);
            Some((cx.alloca(size, align), align))
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
                    let slot = cx.alloca(size, align);
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
            sig: Signature { ret: Return::Void, params: Vec::new() },
            block: "start".to_string(),
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

    // -- writing ----------------------------------------------------------------------------------

    pub(crate) fn tmp(&mut self) -> String {
        self.next += 1;
        format!("%t{}", self.next)
    }

    pub(crate) fn label(&mut self) -> String {
        self.next += 1;
        format!("l{}", self.next)
    }

    pub(crate) fn emit(&mut self, line: impl AsRef<str>) {
        self.out.push_str("  ");
        self.out.push_str(line.as_ref());
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
        for ((local, decl), pass) in body.args().zip(passes) {
            let index = local.0 as usize;
            match pass {
                Pass::Ignore => {}
                Pass::Indirect { .. } => {
                    self.addr[index] = Some(format!("%p{param}"));
                    param += 1;
                }
                Pass::Direct(parts) => {
                    let (size, align) = self.m.ty.size_align(decl.ty);
                    let slot = self.alloca(size, align);
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
            if self.addr[index].is_some() || (index >= 1 && index <= body.arg_count) {
                continue;
            }
            match self.m.ty.repr(decl.ty) {
                Repr::Zst => {}
                Repr::Scalar(sc) => {
                    let (size, align) = self.m.ty.size_align(decl.ty);
                    let name = self.tmp();
                    let _ = writeln!(self.allocas, "  {name} = alloca {}, align {}", sc.mem(), align.max(1));
                    let _ = size;
                    self.addr[index] = Some(name);
                }
                Repr::Mem { size, align } => {
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
                    let _ = inner;
                    return unsupported("a `Shared` payload");
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
                let offset = 24 + table.class_field_offsets(id)[*index];
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
                let offset = self.index_offset(projection, size)?;
                let ptr = self.value(format!("getelementptr i8, ptr {}, i64 {offset}", at.ptr));
                PlaceRef { ptr, ty: elem, variant: None, align: at.align.min(elem_align) }
            }
            (Projection::Index(_) | Projection::ConstIndex(_), TyKind::Vec { elem, .. } | TyKind::Span { elem, .. }) => {
                let (size, elem_align) = self.m.ty.size_align(elem);
                let buffer = self.load(Sc::Ptr, &at.ptr, at.align.min(8));
                let offset = self.index_offset(projection, size)?;
                let ptr = self.value(format!("getelementptr i8, ptr {buffer}, i64 {offset}"));
                PlaceRef { ptr, ty: elem, variant: None, align: elem_align }
            }
            (Projection::Index(_) | Projection::ConstIndex(_), TyKind::Ptr { inner, .. }) => {
                let (size, inner_align) = self.m.ty.size_align(inner);
                let base = self.load(Sc::Ptr, &at.ptr, at.align.min(8));
                let offset = self.index_offset(projection, size)?;
                let ptr = self.value(format!("getelementptr i8, ptr {base}, i64 {offset}"));
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

    /// The byte offset of an index: `index * size`, which cannot overflow, since the index was
    /// checked against a length (or proved within one) and the elements fit in memory.
    fn index_offset(&mut self, projection: &Projection, size: u64) -> Result<String> {
        Ok(match projection {
            Projection::ConstIndex(index) => (index * size).to_string(),
            Projection::Index(local) => {
                let local = *local;
                let (index, sc) = self.read_scalar(&Place::local(local))?;
                let index = match sc {
                    Sc::I(64) => index,
                    Sc::I(bits) if bits < 64 => self.value(format!("zext i{bits} {index} to i64")),
                    Sc::I(bits) => self.value(format!("trunc i{bits} {index} to i64")),
                    other => return unsupported(format!("an index of type {}", other.ir())),
                };
                if size == 1 { index } else { self.value(format!("mul i64 {index}, {size}")) }
            }
            _ => unreachable!("an index projection"),
        })
    }

    pub(crate) fn place_ty(&self, place: &Place) -> Ty {
        let types = self.m.ty.table;
        let mut ty = self.local_ty(place.local);
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
                (Projection::Field(index), TyKind::Struct(id)) => {
                    types.struct_def(*id).fields.get(*index).map_or(ty, |field| field.ty)
                }
                (Projection::Field(index), TyKind::Class(id)) => types.class_field_at(*id, *index).map_or(ty, |field| field.ty),
                (Projection::Field(index), TyKind::Tuple(items)) => items.get(*index).copied().unwrap_or(ty),
                (Projection::Field(0), TyKind::Str | TyKind::Vec { .. } | TyKind::Span { .. }) => ty,
                (Projection::Field(_), TyKind::Str | TyKind::Vec { .. } | TyKind::Span { .. }) => self.m.ty.usize_ty,
                (Projection::Index(_) | Projection::ConstIndex(_), TyKind::Array { elem, .. } | TyKind::Vec { elem, .. } | TyKind::Span { elem, .. }) => *elem,
                (Projection::Index(_) | Projection::ConstIndex(_), TyKind::Ptr { inner, .. }) => *inner,
                (Projection::Deref, TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. }) => *inner,
                _ => ty,
            };
            variant = None;
        }
        ty
    }

    fn read_scalar(&mut self, place: &Place) -> Result<(String, Sc)> {
        let at = self.place(place)?;
        match self.m.ty.repr(at.ty) {
            Repr::Scalar(sc) => Ok((self.load(sc, &at.ptr, at.align), sc)),
            _ => unsupported(format!("a scalar read of `{}`", self.m.ty.table.display(at.ty))),
        }
    }

    // -- operands ---------------------------------------------------------------------------------

    fn operand_ty(&self, operand: &Operand) -> Option<Ty> {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => Some(self.place_ty(place)),
            Operand::Const(Const::Int { ty, .. } | Const::Float { ty, .. }) => Some(*ty),
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
            other => unsupported(format!("a scalar operand held as {other:?}")),
        }
    }

    fn constant(&mut self, constant: &Const) -> Result<Val> {
        Ok(match constant {
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
                // A function the program calls through a C pointer is its address.
                let symbol = symbol.clone();
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
        match kind {
            StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => Ok(()),
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
                let object = self.read_scalar(place)?.0;
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
                let offset = (24 + align - 1) & !(align - 1);
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
        if self.is_dyn_place(ty) {
            // `[EXC-18]` — through the table's `access` entry.
            let code = match operation {
                "begin_read" => 0,
                "end_read" => 1,
                "begin_write" => 2,
                _ => 3,
            };
            // A `ref dyn I`, a `dyn I` and a `Box[dyn I]` are each the carrier itself.
            let at = self.place(place)?;
            let loc = self.loc(span);
            self.dyn_access(&at, code, loc);
            return Ok(());
        }
        let loc = self.loc(span);
        if matches!(self.m.ty.kind(ty), TyKind::Class(_) | TyKind::ClassInterface(_)) {
            let object = self.read_scalar(place)?.0;
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
        let object = self.read_scalar(place)?.0;
        let what = self.m.ty.table.display(ty);
        let what = self.m.bytes(what.as_bytes());
        self.call_rt(&format!("access_{operation}"), vec![Val::Imm(Sc::Ptr, object), Val::Imm(Sc::Ptr, what), loc], None);
        Ok(())
    }

    /// `[EXC-19]` — an access nothing can overlap, as one check; `false` when `place` is neither
    /// a class field with its own word nor a class object.
    fn access_check(&mut self, place: &Place, mutable: bool, span: ember_span::Span) -> Result<bool> {
        let ty = self.place_ty(place);
        if self.is_dyn_place(ty) {
            return Ok(false);
        }
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
        let object = self.read_scalar(place)?.0;
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
        // Read every operand before the first field is written: one may read the place.
        let mut values = Vec::with_capacity(operands.len());
        for operand in operands {
            let mut value = self.operand(operand)?;
            if let (Val::Mem(ptr, align), Operand::Copy(from) | Operand::Move(from)) = (&value, operand)
                && may_overlap(place, from)
            {
                let ty = self.place_ty(from);
                let (size, _) = self.m.ty.size_align(ty);
                let slot = self.alloca(size, *align);
                let (ptr, align) = (ptr.clone(), *align);
                self.copy(&slot, align, &ptr, align, size, false);
                value = Val::Mem(slot, align);
            }
            values.push(value);
        }
        let at = self.place(place)?;
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
            (AggregateKind::Array, TyKind::Array { elem, .. }) => {
                let size = self.m.ty.size_align(elem).0;
                (0..values.len() as u64).map(|i| (i * size, elem)).collect()
            }
            (AggregateKind::Enum(id, variant), _) => {
                let def = table.enum_def(*id);
                if let Some(niche) = table.option_niche(*id) {
                    if *variant == niche.some {
                        let value = values.pop().expect("`Some` has its payload");
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
            (AggregateKind::Callable, _) => return unsupported("an owned callable value ([CLO-10])"),
            (kind, other) => return unsupported(format!("an aggregate {kind:?} of {other:?}")),
        };
        for ((offset, field_ty), value) in fields.into_iter().zip(values) {
            let field = self.field(&at, offset, field_ty);
            let size = self.m.ty.size_align(field_ty).0;
            self.store(&value, &field.ptr, field.align, size, false);
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
        let offset = self.value(format!("mul i64 {i}, {size}"));
        let element = self.value(format!("getelementptr i8, ptr {}, i64 {offset}", at.ptr));
        self.store(&value, &element, at.align.min(align), size, false);
        self.emit(format!("{next} = add i64 {i}, 1"));
        self.emit(format!("br label %{head}"));
        self.start(&done);
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
                if self.m.ty.is_float(repr) {
                    return unsupported("a float range's niche");
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
                if self.m.ty.is_float(repr) {
                    return unsupported("a float range's niche");
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
                if self.m.ty.is_f16(ty) {
                    return unsupported("`f16` arithmetic");
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
                if self.m.ty.is_zst(ty) {
                    return Ok(Val::Imm(Sc::Ptr, self.void_place()));
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

    fn binary(&mut self, op: BinOp, lhs: &Operand, rhs: &Operand, target: Ty) -> Result<Val> {
        let lhs_ty = self.operand_ty(lhs).or_else(|| self.operand_ty(rhs)).unwrap_or(target);
        let types = &self.m.ty;
        if types.is_f16(lhs_ty) || types.is_f16(target) {
            return unsupported("`f16` arithmetic");
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
            BinOp::Div | BinOp::Rem | BinOp::FloorDiv | BinOp::FloorRem if wide && self.m.ty.target == abi::Target::Win64 => {
                return unsupported("a 128-bit division on Windows");
            }
            BinOp::Div => self.value(format!("{} {ty} {a}, {b}", if signed { "sdiv" } else { "udiv" })),
            BinOp::Rem => self.value(format!("{} {ty} {a}, {b}", if signed { "srem" } else { "urem" })),
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
    fn floor_div(&mut self, op: BinOp, sc: Sc, a: &str, b: &str) -> String {
        let ty = sc.ir();
        let quotient = self.value(format!("sdiv {ty} {a}, {b}"));
        let remainder = self.value(format!("srem {ty} {a}, {b}"));
        let nonzero = self.value(format!("icmp ne {ty} {remainder}, 0"));
        let signs = self.value(format!("xor {ty} {remainder}, {b}"));
        let differ = self.value(format!("icmp slt {ty} {signs}, 0"));
        let adjust = self.value(format!("and i1 {nonzero}, {differ}"));
        if op == BinOp::FloorDiv {
            let less = self.value(format!("sub {ty} {quotient}, 1"));
            self.value(format!("select i1 {adjust}, {ty} {less}, {ty} {quotient}"))
        } else {
            let more = self.value(format!("add {ty} {remainder}, {b}"));
            self.value(format!("select i1 {adjust}, {ty} {more}, {ty} {remainder}"))
        }
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
        if bits > 128 {
            return unsupported("checked 256-bit arithmetic");
        }
        let ity = sc.ir();
        let (result, flag) = match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul => {
                if op == BinOp::Mul && bits == 128 {
                    return unsupported("a checked 128-bit multiplication");
                }
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
                if bits == 128 && self.m.ty.target == abi::Target::Win64 {
                    return unsupported("a 128-bit division on Windows");
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
            other => return unsupported(format!("the cast {}", cast_name(other))),
        }
        let Some(from) = from else { return unsupported("a cast of an untyped operand") };
        let types = &self.m.ty;
        if types.is_f16(from) || types.is_f16(to) {
            return unsupported("an `f16` conversion");
        }
        let (from_repr, to_repr) = (types.repr(from), types.repr(to));
        let signed = types.is_signed(from);
        let (value, sc) = self.scalar(operand)?;
        let Repr::Scalar(to_sc) = to_repr else { return unsupported("a cast to a non-scalar") };
        let _ = from_repr;
        let result = match (sc, to_sc) {
            (a, b) if a == b => value,
            (Sc::Bool, Sc::I(bits)) => self.value(format!("zext i1 {value} to i{bits}")),
            (Sc::I(from_bits), Sc::Bool) => self.value(format!("icmp ne i{from_bits} {value}, 0")),
            (Sc::I(from_bits), Sc::I(to_bits)) => self.int_convert(&value, from_bits, to_bits, signed),
            (Sc::I(from_bits), to @ (Sc::F32 | Sc::F64 | Sc::F80)) => {
                if from_bits > 64 && self.m.ty.target == abi::Target::Win64 {
                    return unsupported("a 128-bit integer to float conversion on Windows");
                }
                let instruction = if signed { "sitofp" } else { "uitofp" };
                self.value(format!("{instruction} i{from_bits} {value} to {}", to.ir()))
            }
            (from @ (Sc::F32 | Sc::F64 | Sc::F80), Sc::I(to_bits)) => {
                // `[TYP-6]` — saturating, NaN to 0: LLVM's `.sat` conversions exactly.
                if to_bits > 64 && self.m.ty.target == abi::Target::Win64 {
                    return unsupported("a float to 128-bit integer conversion on Windows");
                }
                let to_signed = self.m.ty.is_signed(to);
                let intrinsic = format!("llvm.fpto{}i.sat.i{to_bits}.{}", if to_signed { "s" } else { "u" }, float_suffix(from));
                let from_ir = from.ir();
                self.m.declare(&intrinsic, || format!("declare i{to_bits} @{intrinsic}({from_ir})"));
                self.value(format!("call i{to_bits} @{intrinsic}({from_ir} {value})"))
            }
            (Sc::F32, to @ (Sc::F64 | Sc::F80)) | (Sc::F64, to @ Sc::F80) => {
                self.value(format!("fpext {} {value} to {}", sc.ir(), to.ir()))
            }
            (from @ (Sc::F64 | Sc::F80), to @ (Sc::F32 | Sc::F64)) => {
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

    fn terminator(&mut self, terminator: &Terminator, span: ember_span::Span) -> Result<()> {
        match terminator {
            Terminator::Goto(target) => self.emit(format!("br label %bb{}", target.0)),
            Terminator::Unreachable => self.emit("unreachable"),
            Terminator::Return => self.ret()?,
            Terminator::SwitchInt { discr, targets, otherwise } => {
                let (mut value, mut sc) = self.scalar(discr)?;
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
                if !self.out.ends_with("  unreachable\n") {
                    self.emit(format!("br label %bb{}", next.0));
                }
            }
        }
        Ok(())
    }

    fn ret(&mut self) -> Result<()> {
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
                    let (base, base_align) = self.spill(value, *size, *align);
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
        let ret_ext = ret_attr(sig);
        let call = format!("call{ret_ext} {ret_ir} {callee}({}){attrs}", args.join(", "));
        match (&sig.ret, dest) {
            (Return::Direct(parts), Some((to, to_align, _))) => {
                let result = self.value(&call);
                if parts.len() == 1 {
                    self.store_part(to, to_align, &parts[0], &result);
                } else {
                    for (index, part) in parts.iter().enumerate() {
                        let element = self.value(format!("extractvalue {ret_ir} {result}, {index}"));
                        self.store_part(to, to_align, part, &element);
                    }
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
            _ => {}
        }
        match func {
            FuncRef::Direct { symbol, .. } => {
                let Some(body) = self.m.bodies.get(symbol.as_str()).copied() else {
                    return unsupported(format!("a call to `{symbol}`, which has no body"));
                };
                if body.is_extern_declaration {
                    return unsupported("a foreign call ([FFI-10])");
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
        let values: Vec<(Val, u64, u64)> = values
            .into_iter()
            .zip(body.args())
            .map(|(value, (_, decl))| {
                let (size, align) = self.m.ty.size_align(decl.ty);
                (value, size, align)
            })
            .collect();
        let (size, _) = self.m.ty.size_align(body.return_ty());
        let dest = dest.map(|(ptr, align)| (ptr, align, size));
        self.lowered_call(&global_name(symbol), &sig, &values, dest, "");
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
        let symbol = runtime::symbol(entry);
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
        let call_attrs = if entry.noreturn { " #0" } else { "" };
        let _ = call_attrs;
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
            Builtin::Print | Builtin::Println | Builtin::EPrint | Builtin::EPrintln => self.print(which, arg_ty, args),
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
            other => self.builtin_call(other, arg_ty, args, dest, span),
        }
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
            abi::Target::SysV => {
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

    /// `[STD-9]` — `print` and its kin through the runtime's printer for the argument's type.
    fn print(&mut self, which: &Builtin, arg_ty: Ty, args: &[Operand]) -> Result<()> {
        let printer = match which {
            Builtin::Println => "println",
            Builtin::EPrintln => "eprintln",
            Builtin::EPrint => "eprint",
            _ => "print",
        };
        let table = self.m.ty.table;
        let ty = self.m.ty.through_range(arg_ty);
        let value = self.operand(&args[0])?;
        // `[TYP-36]` — `void` prints `()`, a unit-only enum its variant's name.
        if matches!(table.kind(ty), TyKind::Void) {
            let text = self.str_value(b"()");
            self.call_rt(&format!("{printer}_str"), vec![text], None);
            return Ok(());
        }
        if let TyKind::Enum(id) = *table.kind(ty) {
            let def = table.enum_def(id);
            if !def.is_unit_only() || !table.enum_is_bare(id) || def.open {
                return unsupported("printing an enum value ([TYP-36])");
            }
            let Val::Imm(Sc::I(bits), tag) = value else { return unsupported("a unit enum held in memory") };
            let empty = self.m.bytes(b"");
            let (mut ptr, mut len) = (empty, "0".to_string());
            for variant in def.variants.iter().rev() {
                let name = variant.name.as_str().as_bytes().to_vec();
                let is = self.value(format!("icmp eq i{bits} {tag}, {}", int_literal(variant.discriminant as u128, bits)));
                let text = self.m.bytes(&name);
                ptr = self.value(format!("select i1 {is}, ptr {text}, ptr {ptr}"));
                len = self.value(format!("select i1 {is}, i64 {}, i64 {len}", name.len()));
            }
            let slot = self.alloca(16, 8);
            self.emit(format!("store ptr {ptr}, ptr {slot}, align 8"));
            let len_at = self.gep(&slot, 8);
            self.emit(format!("store i64 {len}, ptr {len_at}, align 8"));
            self.call_rt(&format!("{printer}_str"), vec![Val::Mem(slot, 8)], None);
            return Ok(());
        }
        let (suffix, value) = match (table.kind(ty), value) {
            (TyKind::Bool, value) => ("bool", value),
            (TyKind::Char, value) => ("char", value),
            (TyKind::Str, value) => ("str", value),
            (TyKind::Float(ember_types::FloatTy::F64), value) => ("f64", value),
            (TyKind::Float(ember_types::FloatTy::F32), value) => ("f32", value),
            (TyKind::Float(ember_types::FloatTy::F16), value) => ("f16", value),
            (TyKind::Int(ember_types::IntTy::I128), value) => ("i128", value),
            (TyKind::Uint(ember_types::UintTy::U128), value) => ("u128", value),
            (TyKind::Int(ember_types::IntTy::I256), value) => ("i256", value),
            (TyKind::Uint(ember_types::UintTy::U256), value) => ("u256", value),
            (TyKind::Int(_), Val::Imm(Sc::I(bits), v)) => ("i64", Val::Imm(Sc::I(64), self.int_convert(&v, bits, 64, true))),
            (TyKind::Uint(_), Val::Imm(Sc::I(bits), v)) => ("u64", Val::Imm(Sc::I(64), self.int_convert(&v, bits, 64, false))),
            _ => return unsupported(format!("printing `{}`", table.display(arg_ty))),
        };
        let value = match (suffix, value) {
            // A 256-bit count is the runtime's struct of words, passed in memory.
            ("i256" | "u256", Val::Imm(sc, v)) => {
                let (slot, align) = self.spill(&Val::Imm(sc, v), 32, 16);
                Val::Mem(slot, align)
            }
            (_, value) => value,
        };
        self.call_rt(&format!("{printer}_{suffix}"), vec![value], None);
        Ok(())
    }

    /// A `str` of these bytes.
    fn str_value(&mut self, bytes: &[u8]) -> Val {
        let ptr = self.m.bytes(bytes);
        let slot = self.alloca(16, 8);
        self.emit(format!("store ptr {ptr}, ptr {slot}, align 8"));
        let len = self.gep(&slot, 8);
        self.emit(format!("store i64 {}, ptr {len}, align 8", bytes.len()));
        Val::Mem(slot, 8)
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

fn cast_name(kind: &CastKind) -> &'static str {
    match kind {
        CastKind::FnToC => "of a function value to its C entry",
        CastKind::Numeric | CastKind::Widen => "of a number",
        CastKind::ClassUpcast | CastKind::ClassUpcastBorrowed => "of a class handle to its base ([CLS-4])",
        CastKind::InterfaceUpcast { .. } => "to `dyn` ([TYP-22])",
        CastKind::ClassInterfaceUpcast { .. } => "to an interface handle ([OBJ-2])",
        CastKind::ViewsDisjoint => "of disjoint views",
        CastKind::InterfaceDowncast => "of `Box[dyn]` to its value",
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
pub(crate) fn emit_entry(m: &mut Module<'_>, bodies: &[Body], options: &Options<'_>) -> Result<()> {
    let entry_symbol = ember_branding::entry_symbol();
    let main_entry = ember_branding::generated("main_entry");
    let (allocate, deallocate) = ember_branding::global_allocator_glue();
    if bodies.iter().any(|body| [&allocate, &deallocate].contains(&&body.name.rsplit('.').next().unwrap_or_default().to_string())) {
        return unsupported("a global allocator ([ALC-3])");
    }
    let entry = bodies.iter().any(|body| body.symbol == main_entry);
    let called = if entry { main_entry.clone() } else { entry_symbol.clone() };
    let Some(called_body) = m.bodies.get(called.as_str()).copied() else {
        return unsupported("a program without `main`");
    };
    let mut cx = FnCx {
        m,
        body: None,
        allocas: String::new(),
        prologue: String::new(),
        out: String::new(),
        next: 0,
        addr: Vec::new(),
        sig: Signature { ret: Return::Void, params: Vec::new() },
        block: "entry".to_string(),
    };
    // `ember_rt_config_default()`: its size, and nothing else set.
    let cfg = cx.alloca(48, 8);
    cx.zero(&cfg, 8, 48);
    cx.emit(format!("store i64 48, ptr {cfg}, align 8"));
    if options.leak_check {
        let flags = cx.gep(&cfg, 40);
        // `EMBER_RT_DEBUG_OBJECTS`
        cx.emit(format!("store i32 1, ptr {flags}, align 8"));
    }
    cx.call_rt("rt_set_args", vec![Val::Imm(Sc::I(32), "%argc".to_string()), Val::Imm(Sc::Ptr, "%argv".to_string())], None);
    cx.call_rt("rt_init_v3", vec![Val::Imm(Sc::Ptr, cfg.clone()), Val::Imm(Sc::I(32), "3".to_string())], None);
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
    let text = format!(
        "define i32 @main(i32 %argc, ptr %argv) {attrs} {{\nentry:\n{}{}}}\n\n",
        cx.allocas, cx.out
    );
    cx.m.fns.push_str(&text);
    Ok(())
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
