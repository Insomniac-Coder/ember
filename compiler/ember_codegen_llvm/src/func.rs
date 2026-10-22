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
struct PlaceRef {
    ptr: String,
    ty: Ty,
    variant: Option<usize>,
    align: u64,
}

pub(crate) struct FnCx<'m, 'a> {
    m: &'m mut Module<'a>,
    body: &'a Body,
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

fn body_signature(m: &Module<'_>, body: &Body) -> Signature {
    signature(m, body.return_ty(), body.args().map(|(_, decl)| decl.ty))
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
fn fn_attrs(m: &Module<'_>) -> &'static str {
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
        body,
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
        for stmt in &block.stmts {
            cx.stmt(&stmt.kind, stmt.span)?;
        }
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

impl<'m, 'a> FnCx<'m, 'a> {
    // -- writing ----------------------------------------------------------------------------------

    fn tmp(&mut self) -> String {
        self.next += 1;
        format!("%t{}", self.next)
    }

    fn label(&mut self) -> String {
        self.next += 1;
        format!("l{}", self.next)
    }

    fn emit(&mut self, line: impl AsRef<str>) {
        self.out.push_str("  ");
        self.out.push_str(line.as_ref());
        self.out.push('\n');
    }

    fn start(&mut self, label: &str) {
        let _ = writeln!(self.out, "{label}:");
        self.block = label.to_string();
    }

    /// A value of its own: `%t = <rhs>`.
    fn value(&mut self, rhs: impl AsRef<str>) -> String {
        let name = self.tmp();
        let line = format!("{name} = {}", rhs.as_ref());
        self.emit(line);
        name
    }

    fn alloca(&mut self, size: u64, align: u64) -> String {
        let name = self.tmp();
        let _ = writeln!(self.allocas, "  {name} = alloca [{} x i8], align {align}", size.max(1));
        name
    }

    fn gep(&mut self, ptr: &str, offset: u64) -> String {
        if offset == 0 {
            return ptr.to_string();
        }
        self.value(format!("getelementptr i8, ptr {ptr}, i64 {offset}"))
    }

    fn load(&mut self, sc: Sc, ptr: &str, align: u64) -> String {
        let loaded = self.value(format!("load {}, ptr {ptr}, align {align}", sc.mem()));
        if sc == Sc::Bool { self.value(format!("trunc i8 {loaded} to i1")) } else { loaded }
    }

    fn store_scalar(&mut self, sc: Sc, value: &str, ptr: &str, align: u64) {
        let value = if sc == Sc::Bool { self.value(format!("zext i1 {value} to i8")) } else { value.to_string() };
        self.emit(format!("store {} {value}, ptr {ptr}, align {align}", sc.mem()));
    }

    fn copy(&mut self, to: &str, to_align: u64, from: &str, from_align: u64, size: u64, overlap: bool) {
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

    fn zero(&mut self, to: &str, align: u64, size: u64) {
        if size == 0 {
            return;
        }
        let intrinsic = "llvm.memset.p0.i64";
        self.m.declare(intrinsic, || format!("declare void @{intrinsic}(ptr, i8, i64, i1)"));
        self.emit(format!("call void @{intrinsic}(ptr align {align} {to}, i8 0, i64 {size}, i1 false)"));
    }

    /// `val` written to `ptr`, a place of `size` bytes.
    fn store(&mut self, val: &Val, ptr: &str, align: u64, size: u64, overlap: bool) {
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
    fn spill(&mut self, val: &Val, size: u64, align: u64) -> (String, u64) {
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

    fn frame(&mut self) -> Result<()> {
        let body = self.body;
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
        self.body.local(local).ty
    }

    fn natural_align(&self, ty: Ty) -> u64 {
        self.m.ty.size_align(ty).1
    }

    /// The address of `place`.
    fn place(&mut self, place: &Place) -> Result<PlaceRef> {
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

    fn place_ty(&self, place: &Place) -> Ty {
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

    fn operand(&mut self, operand: &Operand) -> Result<Val> {
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
    fn scalar(&mut self, operand: &Operand) -> Result<(String, Sc)> {
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

    fn stmt(&mut self, kind: &StmtKind, span: ember_span::Span) -> Result<()> {
        match kind {
            StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => Ok(()),
            StmtKind::Assign { place, rvalue } => self.assign(place, rvalue, span),
            StmtKind::CheckedBinaryOp { dest, overflow, op, lhs, rhs } => self.checked(dest, overflow, *op, lhs, rhs),
            StmtKind::Drop { place, .. } => {
                let ty = self.place_ty(place);
                if self.m.ty.table.needs_drop(ty) {
                    return unsupported(format!("a drop of `{}`", self.m.ty.table.display(ty)));
                }
                Ok(())
            }
            StmtKind::BeginAccess { .. }
            | StmtKind::BeginAccessTransfer { .. }
            | StmtKind::EndAccess { .. }
            | StmtKind::EndAccessTransfer { .. }
            | StmtKind::LendAccess { .. } => unsupported("a dynamic exclusivity check ([EXC-1])"),
        }
    }

    /// Whether a copy of a value of `ty` must retain what it holds (`[OWN-7]`).
    fn holds_handles(&self, ty: Ty) -> bool {
        let table = self.m.ty.table;
        table.is_copy(ty) && table.needs_drop(ty)
    }

    fn assign(&mut self, place: &Place, rvalue: &Rvalue, span: ember_span::Span) -> Result<()> {
        let ty = self.place_ty(place);
        if self.m.ty.is_zst(ty) {
            return Ok(());
        }
        if let Rvalue::Use(Operand::Copy(from) | Operand::Move(from)) = rvalue
            && from == place
        {
            return Ok(());
        }
        if matches!(rvalue, Rvalue::Use(Operand::Copy(_))) && self.holds_handles(ty) {
            return unsupported("a copy of a value holding class handles ([OWN-7])");
        }
        let _ = span;
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
                let value = self.discriminant(&at)?;
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
    fn discriminant(&mut self, at: &PlaceRef) -> Result<(String, u32)> {
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
    fn int_convert(&mut self, value: &str, from: u32, to: u32, signed: bool) -> String {
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
        match self.body.fp {
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
            CastKind::ViewsDisjoint => return self.operand(operand),
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
                let (value, sc) = self.scalar(discr)?;
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
                let (cond, _) = self.scalar(cond)?;
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
                self.emit(format!("br label %bb{}", next.0));
            }
        }
        Ok(())
    }

    fn ret(&mut self) -> Result<()> {
        match self.sig.ret.clone() {
            Return::Void | Return::Sret { .. } => self.emit("ret void"),
            Return::Direct(parts) => {
                let slot = self.addr[0].clone().expect("a result with bytes has a slot");
                let (_, align) = self.m.ty.size_align(self.body.return_ty());
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
    fn lowered_call(&mut self, callee: &str, sig: &Signature, values: &[(Val, u64, u64)], dest: Option<(&str, u64, u64)>, attrs: &str) {
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
        let Some(body) = self.m.bodies.get(symbol).copied() else {
            return unsupported(format!("a call to `{symbol}`, which has no body"));
        };
        let sig = body_signature(self.m, body);
        let name = global_name(symbol);
        let attrs = fn_attrs(self.m);
        self.m.declare(symbol, || {
            let ret = sig.ret.ir();
            let ext = ret_attr(&sig);
            format!("declare{ext} {ret} {name}({}) {attrs}", ir_params(&sig, false).join(", "))
        });
        Ok(())
    }

    fn call(&mut self, func: &FuncRef, args: &[Operand], dest: &Place, span: ember_span::Span) -> Result<()> {
        match func {
            FuncRef::Direct { symbol, .. } => {
                let Some(body) = self.m.bodies.get(symbol.as_str()).copied() else {
                    return unsupported(format!("a call to `{symbol}`, which has no body"));
                };
                if body.is_extern_declaration {
                    return unsupported("a foreign call ([FFI-10])");
                }
                for (index, argument) in args.iter().enumerate() {
                    if let Operand::Copy(place) = argument
                        && body.param_modes.get(index) == Some(&ember_mir::ParameterMode::Owned)
                        && self.holds_handles(self.place_ty(place))
                    {
                        return unsupported("a class handle copied into an owned parameter ([RC-1])");
                    }
                }
                self.declare_body(symbol)?;
                let sig = body_signature(self.m, body);
                let params: Vec<Ty> = body.args().map(|(_, decl)| decl.ty).collect();
                let mut values = Vec::with_capacity(args.len());
                for (argument, ty) in args.iter().zip(params) {
                    let value = self.operand(argument)?;
                    let (size, align) = self.m.ty.size_align(ty);
                    values.push((value, size, align));
                }
                let dest_ty = self.place_ty(dest);
                let target = if self.m.ty.is_zst(dest_ty) {
                    None
                } else {
                    let at = self.place(dest)?;
                    let (size, _) = self.m.ty.size_align(dest_ty);
                    Some((at.ptr, at.align, size))
                };
                let callee = global_name(symbol);
                self.lowered_call(&callee, &sig, &values, target.as_ref().map(|(p, a, s)| (p.as_str(), *a, *s)), "");
                Ok(())
            }
            FuncRef::Builtin { which, arg_ty } => self.builtin(which, *arg_ty, args, dest, span),
            FuncRef::Virtual { .. } => unsupported("a virtual call ([DSP-2])"),
            FuncRef::Interface { .. } => unsupported("a call through an interface ([TYP-22])"),
            FuncRef::DynBoxNew { .. } => unsupported("a `Box[dyn]` ([TYP-22])"),
            FuncRef::Indirect { .. } => unsupported("a call through a function value ([CLO-3])"),
        }
    }

    /// A call to the runtime's `name` with `values` by its C prototype; its result written to
    /// `dest` when given.
    fn call_rt(&mut self, name: &str, values: Vec<Val>, dest: Option<(&str, u64)>) {
        let entry = runtime::find(name);
        let ret = runtime::c_type(entry.ret).shape();
        let shapes: Vec<Shape> = entry.params.iter().map(|param| runtime::c_type(param).shape().expect("a parameter has a type")).collect();
        let sig = abi::classify(self.m.ty.target, ret.as_ref(), &shapes);
        let symbol = ember_branding::runtime(name);
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
    fn loc(&mut self, span: ember_span::Span) -> Val {
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
        let _ = (dest, span);
        match which {
            Builtin::Print | Builtin::Println | Builtin::EPrint | Builtin::EPrintln => self.print(which, arg_ty, args),
            other => unsupported(format!("the built-in `{}`", other.name())),
        }
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

fn shared_inner(table: &ember_types::TypeTable, id: ember_types::StructId) -> Option<Ty> {
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
        body: called_body,
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
