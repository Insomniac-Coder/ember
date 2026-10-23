//! The calls the compiler provides itself (`FuncRef::Builtin`): each lowered to the runtime entry
//! the C backend's C calls, or to the few instructions the C backend's expression is.

use crate::func::{FnCx, Val};
use crate::types::{Repr, Sc, offset_align};
use crate::{Result, global_name, unsupported};

fn float_name(sc: Sc) -> &'static str {
    match sc {
        Sc::F32 => "f32",
        Sc::F80 => "f80",
        _ => "f64",
    }
}
use ember_mir::{Builtin, Operand, Place};
use ember_types::{Ty, TyKind};

impl<'a> FnCx<'_, 'a> {
    /// A pointer operand's value (a `ref`, a `ref mut`, a raw pointer, a class handle).
    pub(crate) fn pointer(&mut self, operand: &Operand) -> Result<String> {
        match self.operand(operand)? {
            Val::Imm(Sc::Ptr, value) => Ok(value),
            // A wrapper of one pointer (a `Shared`, a `Box`).
            Val::Mem(at, align) => Ok(self.load(Sc::Ptr, &at, align.min(8))),
            other => unsupported(format!("a pointer held as {other:?}")),
        }
    }

    /// An integer operand as an `i64` (a `size_t`, an `int64_t`): sign- or zero-extended by its
    /// type.
    pub(crate) fn word(&mut self, operand: &Operand) -> Result<String> {
        let ty = self.operand_ty_of(operand);
        let signed = ty.is_some_and(|ty| self.m.ty.is_signed(ty));
        match self.operand(operand)? {
            Val::Imm(Sc::I(bits), value) => Ok(self.int_convert(&value, bits, 64, signed)),
            Val::Imm(Sc::Bool, value) => Ok(self.value(format!("zext i1 {value} to i64"))),
            other => unsupported(format!("an integer held as {other:?}")),
        }
    }

    fn operand_ty_of(&self, operand: &Operand) -> Option<Ty> {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => Some(self.place_ty(place)),
            Operand::Const(ember_mir::Const::Int { ty, .. } | ember_mir::Const::Float { ty, .. }) => Some(*ty),
            _ => None,
        }
    }

    /// The element type of an `Array`, through any references (`element_of`).
    pub(crate) fn element_of(&self, ty: Ty) -> Ty {
        match self.m.ty.kind(ty) {
            TyKind::Vec { elem, .. } => *elem,
            TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. } => self.element_of(*inner),
            _ => ty,
        }
    }

    /// The element type of a view, through a reference.
    fn span_element(&self, ty: Ty) -> Ty {
        match self.m.ty.kind(ty) {
            TyKind::Span { elem, .. } | TyKind::Vec { elem, .. } => *elem,
            TyKind::Ref { inner, .. } => self.span_element(*inner),
            _ => ty,
        }
    }

    /// The address of a view operand's two words: the view itself, or where a reference to one
    /// points.
    pub(crate) fn view_words(&mut self, operand: &Operand) -> Result<(String, u64)> {
        match self.operand(operand)? {
            Val::Mem(ptr, align) => Ok((ptr, align)),
            Val::Imm(Sc::Ptr, ptr) => Ok((ptr, 8)),
            other => unsupported(format!("a view held as {other:?}")),
        }
    }

    /// Write a two-word view `{ptr, len}` to `dest`.
    fn store_view(&mut self, dest: &Place, ptr: &str, len: &str) -> Result<()> {
        let at = self.place(dest)?;
        self.emit(format!("store ptr {ptr}, ptr {}, align {}", at.ptr, at.align.min(8)));
        let len_at = self.gep(&at.ptr, 8);
        self.emit(format!("store i64 {len}, ptr {len_at}, align {}", at.align.min(8)));
        Ok(())
    }

    /// Write a scalar result to `dest`, converted to its width where the C backend's C converts.
    pub(crate) fn store_result(&mut self, dest: &Place, value: &str, sc: Sc, signed: bool) -> Result<()> {
        let at = self.place(dest)?;
        match (self.m.ty.repr(at.ty), sc) {
            (Repr::Zst, _) => {}
            (Repr::Scalar(Sc::I(to)), Sc::I(from)) if to != from => {
                let converted = self.int_convert(value, from, to, signed);
                self.store_scalar(Sc::I(to), &converted, &at.ptr, at.align);
            }
            (Repr::Scalar(Sc::Bool), Sc::I(from)) => {
                let truth = self.value(format!("icmp ne i{from} {value}, 0"));
                self.store_scalar(Sc::Bool, &truth, &at.ptr, at.align);
            }
            (Repr::Scalar(to), _) => self.store_scalar(to, value, &at.ptr, at.align),
            // A wrapper struct of the one word (a `Box`, a `Weak`).
            (Repr::Mem { size, .. }, sc) if size == u64::from(sc.bits().div_ceil(8)) => {
                self.store_scalar(sc, value, &at.ptr, at.align)
            }
            (other, _) => return unsupported(format!("a scalar result into {other:?}")),
        }
        Ok(())
    }

    /// `[HEAP-9]`, `[LAY-2]` — a list operation's size arguments: the element's size, and its
    /// alignment where that passes 16 (the `_aligned` twin).
    fn vec_sizes(&self, elem: Ty) -> (u64, Option<u64>) {
        let (size, align) = self.m.ty.size_align(elem);
        (size, (align > 16).then_some(align))
    }

    /// A call of the runtime's `name` with `args` as C converts them, its result written to
    /// `dest` (a scalar result converted to the destination's width, as C's assignment does).
    pub(crate) fn call_rt_into(&mut self, name: &str, args: Vec<Val>, dest: &Place) -> Result<()> {
        let entry = crate::runtime::find(name);
        let values = self.fit_args(entry, args)?;
        match crate::runtime::c_type(entry.ret) {
            crate::runtime::CTy::Void => {
                self.call_rt(name, values, None);
                Ok(())
            }
            crate::runtime::CTy::Scalar(scalar) if !matches!(scalar, crate::abi::Scalar::Int { bytes: 16, .. }) => {
                let size = scalar.size();
                let slot = self.alloca(size, size.max(1));
                self.call_rt(name, values, Some((&slot, size.max(1))));
                let sc = match scalar {
                    crate::abi::Scalar::Bool => Sc::Bool,
                    crate::abi::Scalar::Int { bytes, .. } => Sc::I(bytes as u32 * 8),
                    crate::abi::Scalar::F32 => Sc::F32,
                    crate::abi::Scalar::F64 => Sc::F64,
                    crate::abi::Scalar::F80 => Sc::F80,
                    crate::abi::Scalar::Ptr => Sc::Ptr,
                };
                let value = self.load(sc, &slot, size.max(1));
                let signed = matches!(scalar, crate::abi::Scalar::Int { signed: true, .. });
                self.store_result(dest, &value, sc, signed)
            }
            _ => {
                let dest = self.dest_of(dest)?;
                self.call_rt(name, values, dest.as_ref().map(|(p, a)| (p.as_str(), *a)));
                Ok(())
            }
        }
    }

    /// The built-ins the C backend writes as one runtime call of their operands in order.
    fn runtime_builtin(&self, which: &Builtin, args: &[Operand]) -> Option<&'static str> {
        use ember_mir::ParseKind;
        Some(match which {
            Builtin::StrStartsWith => "str_starts_with",
            Builtin::StrEndsWith => "str_ends_with",
            Builtin::StrFind { reverse: false } => "str_find",
            Builtin::StrFind { reverse: true } => "str_rfind",
            Builtin::StrCount => "str_count",
            Builtin::StrReplace => "str_replace",
            Builtin::StrRepeat => "str_repeat",
            Builtin::StrTrimStart => "str_trim_start",
            Builtin::StrTrimEnd => "str_trim_end",
            Builtin::StrToUpper => "str_to_upper",
            Builtin::StrToLower => "str_to_lower",
            Builtin::StrSliceOk => "str_slice_ok",
            Builtin::StrCharCount => "str_char_count",
            Builtin::StrContains => "str_contains",
            Builtin::StrContainsChar => "str_contains_char",
            Builtin::StrIsCharBoundary => "str_is_char_boundary",
            Builtin::ProcessKey => "process_key",
            Builtin::ProcessArgs => "process_args",
            // The cursor keeps its type: native text loops use `usize`, std's iterators `int`.
            Builtin::StrCharNext => {
                let cursor = self.operand_ty(&args[1])?;
                match self.m.ty.kind(cursor) {
                    TyKind::Ref { inner, .. } if matches!(self.m.ty.kind(*inner), TyKind::Uint(ember_types::UintTy::Usize)) => "str_char_next_usize",
                    _ => "str_char_next",
                }
            }
            Builtin::ParseStatus { kind } => match kind {
                ParseKind::Signed => "parse_signed_status",
                ParseKind::Unsigned => "parse_unsigned_status",
                ParseKind::I128 => "parse_i128_status",
                ParseKind::U128 => "parse_u128_status",
                ParseKind::I256 => "parse_i256_status",
                ParseKind::U256 => "parse_u256_status",
                ParseKind::F16 | ParseKind::F32 | ParseKind::F64 => "parse_float_status",
                ParseKind::Bool => "parse_bool_status",
                ParseKind::Char => "parse_char_status",
            },
            Builtin::ParseValue { kind } => match kind {
                ParseKind::Signed => "parse_signed_value",
                ParseKind::Unsigned => "parse_unsigned_value",
                ParseKind::I128 => "parse_i128_value",
                ParseKind::U128 => "parse_u128_value",
                ParseKind::I256 => "parse_i256_value",
                ParseKind::U256 => "parse_u256_value",
                ParseKind::F16 => "parse_f16_value",
                ParseKind::F32 => "parse_f32_value",
                ParseKind::F64 => "parse_f64_value",
                ParseKind::Char => "str_char_at",
                ParseKind::Bool => return None,
            },
            _ => return None,
        })
    }

    pub(crate) fn builtin_call(&mut self, which: &Builtin, arg_ty: Ty, args: &[Operand], dest: &Place, span: ember_span::Span) -> Result<()> {
        let size_word = |size: u64| Val::Imm(Sc::I(64), size.to_string());
        if let Some(name) = self.runtime_builtin(which, args) {
            let mut values = Vec::with_capacity(args.len());
            for argument in args {
                values.push(self.operand(argument)?);
            }
            // `parse`'s `Char` value is the text's first character.
            if name == "str_char_at" {
                values.push(size_word(0));
            }
            return self.call_rt_into(name, values, dest);
        }
        if let Some(done) = self.arena_builtin(which, arg_ty, args, dest, span) {
            return done;
        }
        if let Some(done) = self.concurrency_builtin(which, arg_ty, args, dest, span) {
            return done;
        }
        if let Some(done) = self.view_builtin(which, args, dest, span) {
            return done;
        }
        match which {
            // `parse`'s `Bool` value: the text is `true` or `false`, told apart by its length.
            Builtin::ParseValue { kind: ember_mir::ParseKind::Bool } => {
                let (words, align) = self.view_words(&args[0])?;
                let len_at = self.gep(&words, 8);
                let len = self.load(Sc::I(64), &len_at, align.min(8));
                let truth = self.value(format!("icmp eq i64 {len}, 4"));
                self.store_result(dest, &truth, Sc::Bool, false)
            }
            Builtin::Utf8Valid => {
                let (words, align) = self.view_words(&args[0])?;
                self.call_rt_into("utf8_valid", vec![Val::Mem(words, align)], dest)
            }
            // `[STD-26]` — the count and the values in 64 bits, signed or unsigned as the type is.
            Builtin::RangeCount | Builtin::RangeNth => {
                let wide = matches!(self.m.ty.repr(arg_ty), Repr::Scalar(Sc::I(128)));
                let unsigned = matches!(self.m.ty.kind(self.m.ty.through_range(arg_ty)), TyKind::Uint(_));
                let stem = if matches!(which, Builtin::RangeCount) { "count" } else { "nth" };
                let suffix = match (wide, unsigned) {
                    (true, true) => "u128",
                    (true, false) => "i128",
                    (false, true) => "u64",
                    (false, false) => "i64",
                };
                let mut values = Vec::with_capacity(args.len());
                for argument in args {
                    values.push(self.operand(argument)?);
                }
                self.call_rt_into(&format!("range_{stem}_{suffix}"), values, dest)
            }
            // `[TYP-37]` — floats by totalOrder, everything else by `<` (text by its bytes).
            // `[TYP-36]` — `void`'s one value is never less than itself.
            Builtin::TotalLess if self.m.ty.is_zst(arg_ty) => self.store_result(dest, "false", Sc::Bool, false),
            Builtin::TotalLess => {
                let ty = self.m.ty.through_range(arg_ty);
                let a = self.operand(&args[0])?;
                let b = self.operand(&args[1])?;
                match self.m.ty.kind(ty).clone() {
                    TyKind::Float(ember_types::FloatTy::F64) => self.call_rt_into("total_lt_f64", vec![a, b], dest),
                    TyKind::Float(ember_types::FloatTy::F32) => self.call_rt_into("total_lt_f32", vec![a, b], dest),
                    TyKind::Float(ember_types::FloatTy::F16) => self.call_rt_into("total_lt_f16", vec![a, b], dest),
                    TyKind::Str => {
                        let slot = self.alloca(4, 4);
                        self.call_rt("str_cmp", vec![a, b], Some((&slot, 4)));
                        let order = self.load(Sc::I(32), &slot, 4);
                        let less = self.value(format!("icmp slt i32 {order}, 0"));
                        self.store_result(dest, &less, Sc::Bool, false)
                    }
                    _ => {
                        let (Val::Imm(sc, a), Val::Imm(_, b)) = (a, b) else {
                            return unsupported(format!("`<` of `{}`", self.m.ty.table.display(ty)));
                        };
                        let less = if sc == Sc::Ptr {
                            let a = self.value(format!("ptrtoint ptr {a} to i64"));
                            let b = self.value(format!("ptrtoint ptr {b} to i64"));
                            self.value(format!("icmp ult i64 {a}, {b}"))
                        } else {
                            let condition = if self.m.ty.is_signed(ty) { "slt" } else { "ult" };
                            self.value(format!("icmp {condition} {} {a}, {b}", sc.ir()))
                        };
                        self.store_result(dest, &less, Sc::Bool, false)
                    }
                }
            }
            // `mem.replace` and `mem.swap` are a load and stores; `mem.forget` nothing.
            Builtin::MemReplace { elem } => {
                let target = self.pointer(&args[0])?;
                let value = self.operand(&args[1])?;
                let (size, align) = self.m.ty.size_align(*elem);
                let at = self.place(dest)?;
                self.copy(&at.ptr, at.align, &target, align, size, true);
                self.store(&value, &target, align, size, true);
                Ok(())
            }
            Builtin::MemSwap { elem } => {
                let (left, right) = (self.pointer(&args[0])?, self.pointer(&args[1])?);
                let (size, align) = self.m.ty.size_align(*elem);
                let held = self.alloca(size, align);
                self.copy(&held, align, &left, align, size, false);
                self.copy(&left, align, &right, align, size, true);
                self.copy(&right, align, &held, align, size, false);
                Ok(())
            }
            Builtin::MemForget { .. } => Ok(()),
            // `[STD-20]` — over the value's own width, zero-extended: LLVM's bit counts, which give
            // the width for zero, as the runtime's helpers do.
            Builtin::IntBits(bits) => {
                let (value, sc) = self.scalar(&args[0])?;
                let Sc::I(width) = sc else { return unsupported("a bit count of a non-integer") };
                let ty = format!("i{width}");
                let (name, extra) = match bits {
                    ember_mir::IntBits::CountOnes => ("ctpop", ""),
                    ember_mir::IntBits::LeadingZeros => ("ctlz", ", i1 false"),
                    ember_mir::IntBits::TrailingZeros => ("cttz", ", i1 false"),
                };
                let intrinsic = format!("llvm.{name}.{ty}");
                let declaration = if extra.is_empty() { format!("declare {ty} @{intrinsic}({ty})") } else { format!("declare {ty} @{intrinsic}({ty}, i1)") };
                self.m.declare(&intrinsic, || declaration);
                let count = self.value(format!("call {ty} @{intrinsic}({ty} {value}{extra})"));
                let count = self.int_convert(&count, width, 64, false);
                self.store_result(dest, &count, Sc::I(64), false)
            }
            Builtin::FloatAbs | Builtin::FloatPow | Builtin::FloatLib(_) => self.float_lib(which, arg_ty, args, dest),
            // `[RNG-3a]`, `[RNG-10]` — the total operations; `new_unchecked` is the value itself.
            Builtin::RangeClamped(id) | Builtin::RangeContains(id) => {
                let value = self.operand(&args[0])?;
                let result = if matches!(which, Builtin::RangeClamped(_)) { self.range_clamped(*id, value)? } else { self.range_contains(*id, value)? };
                let Val::Imm(sc, result) = result else { unreachable!("a scalar") };
                self.store_result(dest, &result, sc, false)
            }
            Builtin::RangeNewUnchecked(_) => {
                let value = self.operand(&args[0])?;
                let at = self.place(dest)?;
                let (size, _) = self.m.ty.size_align(at.ty);
                self.store(&value, &at.ptr, at.align, size, false);
                Ok(())
            }
            // `[FFI-15]` — a C string's bytes up to its NUL, and a `CString`'s own bytes.
            Builtin::CStrToSpan => {
                let text = self.pointer(&args[0])?;
                self.m.declare("strlen", || "declare i64 @strlen(ptr) nounwind".to_string());
                let len = self.value(format!("call i64 @strlen(ptr {text})"));
                self.store_view(dest, &text, &len)
            }
            Builtin::CStringAsCStr => {
                let string = self.pointer(&args[0])?;
                let string_ty = match self.m.ty.kind(arg_ty) {
                    TyKind::Ref { inner, .. } => *inner,
                    _ => arg_ty,
                };
                let offset = match self.m.ty.kind(string_ty) {
                    TyKind::Struct(id) => {
                        let def = self.m.ty.table.struct_def(*id);
                        let index = def.fields.iter().position(|field| field.name.as_str() == "bytes").unwrap_or(0);
                        self.m.ty.table.layout(string_ty).field_offsets.get(index).copied().unwrap_or(0)
                    }
                    _ => 0,
                };
                let bytes_at = self.gep(&string, offset);
                let bytes = self.load(Sc::Ptr, &bytes_at, 8);
                self.store_result(dest, &bytes, Sc::Ptr, false)
            }
            // D-187 — `==`, `!=` and text's orders.
            Builtin::ValueCompare { op } => {
                let a = self.operand(&args[0])?;
                let b = self.operand(&args[1])?;
                let result = self.value_compare(*op, a, b, arg_ty)?;
                self.store_result(dest, &result, Sc::Bool, false)
            }
            // `[STD-15]` — each through its helper for the element type.
            Builtin::ArraySort => {
                let elem = self.element_of(arg_ty);
                let list = self.pointer(&args[0])?;
                let sort = crate::arrays::sort_fn(self.m, elem)?;
                self.emit(format!("call void {sort}(ptr {list})"));
                Ok(())
            }
            Builtin::ArrayClear | Builtin::ArrayTruncate => {
                let elem = self.element_of(arg_ty);
                let list = self.pointer(&args[0])?;
                if matches!(which, Builtin::ArrayClear) {
                    let function = crate::arrays::helper(self.m, crate::arrays::Helper::Clear, elem)?;
                    self.emit(format!("call void {function}(ptr {list})"));
                } else {
                    let n = self.word(&args[1])?;
                    let function = crate::arrays::helper(self.m, crate::arrays::Helper::Truncate, elem)?;
                    self.emit(format!("call void {function}(ptr {list}, i64 {n})"));
                }
                Ok(())
            }
            Builtin::ArrayPop { option } => {
                let list = self.pointer(&args[0])?;
                let function = crate::arrays::helper(self.m, crate::arrays::Helper::Pop, *option)?;
                let at = self.place(dest)?;
                self.emit(format!("call void {function}(ptr {list}, ptr {})", at.ptr));
                Ok(())
            }
            Builtin::ArrayRemove | Builtin::ArraySwapRemove => {
                let elem = self.element_of(arg_ty);
                let list = self.pointer(&args[0])?;
                let index = self.word(&args[1])?;
                let kind = if matches!(which, Builtin::ArrayRemove) { crate::arrays::Helper::Remove } else { crate::arrays::Helper::SwapRemove };
                let function = crate::arrays::helper(self.m, kind, elem)?;
                let (size, align) = self.m.ty.size_align(elem);
                let at = match self.dest_of(dest)? {
                    Some((ptr, _)) => ptr,
                    None => self.alloca(size, align),
                };
                self.emit(format!("call void {function}(ptr {list}, i64 {index}, ptr {at})"));
                Ok(())
            }
            Builtin::ArrayDrain => {
                let elem = self.element_of(arg_ty);
                let list = self.pointer(&args[0])?;
                let (lo, hi) = (self.word(&args[1])?, self.word(&args[2])?);
                let function = crate::arrays::helper(self.m, crate::arrays::Helper::Drain, elem)?;
                let at = self.place(dest)?;
                self.emit(format!("call void {function}(ptr {list}, i64 {lo}, i64 {hi}, ptr {})", at.ptr));
                Ok(())
            }
            Builtin::ArrayExtend => {
                let elem = self.element_of(arg_ty);
                let list = self.pointer(&args[0])?;
                let (words, align) = self.view_words(&args[1])?;
                let elems = self.load(Sc::Ptr, &words, align.min(8));
                let len_at = self.gep(&words, 8);
                let count = self.load(Sc::I(64), &len_at, align.min(8));
                let function = crate::arrays::helper(self.m, crate::arrays::Helper::Extend, elem)?;
                self.emit(format!("call void {function}(ptr {list}, ptr {elems}, i64 {count})"));
                Ok(())
            }
            Builtin::ArrayClone { elem } | Builtin::ArraySorted { elem } => {
                let (words, align) = self.view_words(&args[0])?;
                let elems = self.load(Sc::Ptr, &words, align.min(8));
                let len_at = self.gep(&words, 8);
                let count = self.load(Sc::I(64), &len_at, align.min(8));
                let at = self.place(dest)?;
                if matches!(which, Builtin::ArrayClone { .. }) {
                    let function = crate::arrays::helper(self.m, crate::arrays::Helper::Clone, *elem)?;
                    self.emit(format!("call void {function}(ptr {elems}, i64 {count}, ptr {})", at.ptr));
                } else {
                    let (size, align) = self.m.ty.size_align(*elem);
                    if align > 16 {
                        self.call_rt("vec_from_elems_aligned", vec![size_word(size), size_word(align), Val::Imm(Sc::Ptr, elems), Val::Imm(Sc::I(64), count)], Some((&at.ptr, 8)));
                    } else {
                        self.call_rt("vec_from_elems", vec![size_word(size), Val::Imm(Sc::Ptr, elems), Val::Imm(Sc::I(64), count)], Some((&at.ptr, 8)));
                    }
                    let sort = crate::arrays::sort_fn(self.m, *elem)?;
                    self.emit(format!("call void {sort}(ptr {})", at.ptr));
                }
                Ok(())
            }
            // A structural clone, straight into the destination.
            Builtin::CloneParts { ty } => {
                let source = self.pointer(&args[0])?;
                let at = self.place(dest)?;
                let (_, align) = self.m.ty.size_align(*ty);
                self.clone_part(&at.ptr, &source, align.min(at.align), *ty)
            }
            Builtin::ArrayLen | Builtin::StringLen | Builtin::SpanLen => {
                let (words, align) = self.view_words(&args[0])?;
                let len_at = self.gep(&words, 8);
                let len = self.load(Sc::I(64), &len_at, align.min(8));
                self.store_result(dest, &len, Sc::I(64), false)
            }
            Builtin::ArrayCapacity => {
                let (words, align) = self.view_words(&args[0])?;
                let cap_at = self.gep(&words, 16);
                let cap = self.load(Sc::I(64), &cap_at, align.min(8));
                self.store_result(dest, &cap, Sc::I(64), false)
            }
            Builtin::ArrayPush => {
                let elem = self.element_of(arg_ty);
                // `[RC-1]` — the byte copy into the list is a copy: a `Copy` value's handles retained.
                if let Operand::Copy(place) = &args[1]
                    && self.holds_counts(elem)
                {
                    let at = self.place(place)?;
                    self.retain_lines(&at.ptr, at.align, elem)?;
                }
                let list = self.pointer(&args[0])?;
                let value = self.operand(&args[1])?;
                let (size, align) = self.m.ty.size_align(elem);
                let (value, _) = self.spill(&value, size, align);
                match self.vec_sizes(elem).1 {
                    Some(align) => {
                        let helper = crate::glue::vec_push_helper(self.m, true);
                        self.emit(format!("call void {helper}(ptr {list}, i64 {size}, i64 {align}, ptr {value})"));
                    }
                    None => {
                        let helper = crate::glue::vec_push_helper(self.m, false);
                        self.emit(format!("call void {helper}(ptr {list}, i64 {size}, i64 16, ptr {value})"));
                    }
                }
                Ok(())
            }
            // `[TYP-38]` — one allocation of exactly the literal's elements, moved out of the
            // fixed array.
            Builtin::ArrayFromLiteral => {
                let TyKind::Array { elem, len } = *self.m.ty.kind(arg_ty) else {
                    return unsupported("an `Array` literal not built from a fixed array");
                };
                let array = self.operand(&args[0])?;
                let (total, array_align) = self.m.ty.size_align(arg_ty);
                let (elems, _) = self.spill(&array, total, array_align);
                let dest = self.dest_of(dest)?;
                let (size, aligned) = self.vec_sizes(elem);
                match aligned {
                    Some(align) => self.call_rt(
                        "vec_from_elems_aligned",
                        vec![size_word(size), size_word(align), Val::Imm(Sc::Ptr, elems), size_word(len)],
                        dest.as_ref().map(|(p, a)| (p.as_str(), *a)),
                    ),
                    None => self.call_rt(
                        "vec_from_elems",
                        vec![size_word(size), Val::Imm(Sc::Ptr, elems), size_word(len)],
                        dest.as_ref().map(|(p, a)| (p.as_str(), *a)),
                    ),
                }
                Ok(())
            }
            // `[HEAP-8]` — `len + n` checked, never formed to wrap.
            Builtin::ArrayReserve => {
                let elem = self.element_of(arg_ty);
                let list = self.pointer(&args[0])?;
                let more = self.word(&args[1])?;
                let (size, aligned) = self.vec_sizes(elem);
                match aligned {
                    Some(align) => self.call_rt(
                        "vec_reserve_more_aligned",
                        vec![Val::Imm(Sc::Ptr, list), size_word(size), size_word(align), Val::Imm(Sc::I(64), more)],
                        None,
                    ),
                    None => self.call_rt("vec_reserve_more_inline", vec![Val::Imm(Sc::Ptr, list), size_word(size), Val::Imm(Sc::I(64), more)], None),
                }
                Ok(())
            }
            // No turn when the counter starts past the limit; the difference of the two as `u64`
            // is exact below it.
            Builtin::ArrayReserveHint { per_turn, inclusive } => {
                let elem = self.element_of(arg_ty);
                let list = self.pointer(&args[0])?;
                let counter_ty = self.operand_ty_of(&args[1]);
                let signed = counter_ty.is_some_and(|ty| self.m.ty.is_signed(ty));
                let (counter, counter_sc) = self.scalar(&args[1])?;
                let (limit, _) = self.scalar(&args[2])?;
                let Sc::I(bits) = counter_sc else { return unsupported("a reserve hint over a non-integer") };
                let test = match (inclusive, signed) {
                    (true, true) => "sle",
                    (true, false) => "ule",
                    (false, true) => "slt",
                    (false, false) => "ult",
                };
                let any = self.value(format!("icmp {test} i{bits} {counter}, {limit}"));
                let (go, done) = (self.label(), self.label());
                self.emit(format!("br i1 {any}, label %{go}, label %{done}"));
                self.start(&go);
                let limit64 = self.int_convert(&limit, bits, 64, signed);
                let counter64 = self.int_convert(&counter, bits, 64, signed);
                let mut turns = self.value(format!("sub i64 {limit64}, {counter64}"));
                if *inclusive {
                    turns = self.value(format!("add i64 {turns}, 1"));
                }
                let (size, aligned) = self.vec_sizes(elem);
                match aligned {
                    Some(align) => self.call_rt(
                        "vec_reserve_hint_aligned",
                        vec![Val::Imm(Sc::Ptr, list), size_word(size), size_word(align), Val::Imm(Sc::I(64), turns), size_word(*per_turn as u64)],
                        None,
                    ),
                    None => self.call_rt(
                        "vec_reserve_hint",
                        vec![Val::Imm(Sc::Ptr, list), size_word(size), Val::Imm(Sc::I(64), turns), size_word(*per_turn as u64)],
                        None,
                    ),
                }
                self.emit(format!("br label %{done}"));
                self.start(&done);
                Ok(())
            }
            Builtin::ArraySwap => {
                let elem = self.element_of(arg_ty);
                let list = self.pointer(&args[0])?;
                let (i, j) = (self.word(&args[1])?, self.word(&args[2])?);
                let size = self.m.ty.size_align(elem).0;
                self.call_rt("vec_swap", vec![Val::Imm(Sc::Ptr, list), size_word(size), Val::Imm(Sc::I(64), i), Val::Imm(Sc::I(64), j)], None);
                Ok(())
            }
            Builtin::ArrayInsert => {
                let elem = self.element_of(arg_ty);
                let list = self.pointer(&args[0])?;
                let value = self.operand(&args[1])?;
                let index = self.word(&args[2])?;
                let (size, align) = self.m.ty.size_align(elem);
                let (value, _) = self.spill(&value, size, align);
                match self.vec_sizes(elem).1 {
                    Some(align) => self.call_rt(
                        "vec_insert_aligned",
                        vec![Val::Imm(Sc::Ptr, list), size_word(size), size_word(align), Val::Imm(Sc::I(64), index), Val::Imm(Sc::Ptr, value)],
                        None,
                    ),
                    None => self.call_rt(
                        "vec_insert",
                        vec![Val::Imm(Sc::Ptr, list), size_word(size), Val::Imm(Sc::I(64), index), Val::Imm(Sc::Ptr, value)],
                        None,
                    ),
                }
                Ok(())
            }
            Builtin::ArrayReverse => {
                let elem = self.element_of(arg_ty);
                let list = self.pointer(&args[0])?;
                let size = self.m.ty.size_align(elem).0;
                self.call_rt("vec_reverse", vec![Val::Imm(Sc::Ptr, list), size_word(size)], None);
                Ok(())
            }
            // `[TXT-11]` — Clang's `ember_vec_extend_inline` is the library's append.
            Builtin::StringPush => {
                let text = self.pointer(&args[0])?;
                let (words, align) = self.view_words(&args[1])?;
                let bytes = self.load(Sc::Ptr, &words, align.min(8));
                let len_at = self.gep(&words, 8);
                let len = self.load(Sc::I(64), &len_at, align.min(8));
                self.call_rt("vec_extend", vec![Val::Imm(Sc::Ptr, text), Val::Imm(Sc::Ptr, bytes), Val::Imm(Sc::I(64), len)], None);
                Ok(())
            }
            Builtin::StringPushChar => {
                let text = self.pointer(&args[0])?;
                let (c, _) = self.scalar(&args[1])?;
                self.call_rt("vec_push_char", vec![Val::Imm(Sc::Ptr, text), Val::Imm(Sc::I(32), c)], None);
                Ok(())
            }
            Builtin::StringInsert => {
                let text = self.pointer(&args[0])?;
                let at = self.word(&args[1])?;
                let (c, _) = self.scalar(&args[2])?;
                let loc = self.loc(span);
                self.call_rt("string_insert", vec![Val::Imm(Sc::Ptr, text), Val::Imm(Sc::I(64), at), Val::Imm(Sc::I(32), c), loc], None);
                Ok(())
            }
            Builtin::StringRemove | Builtin::StringTruncate => {
                let text = self.pointer(&args[0])?;
                let at = self.word(&args[1])?;
                let loc = self.loc(span);
                let values = vec![Val::Imm(Sc::Ptr, text), Val::Imm(Sc::I(64), at), loc];
                if matches!(which, Builtin::StringRemove) {
                    let dest = self.dest_of(dest)?;
                    self.call_rt("string_remove", values, dest.as_ref().map(|(p, a)| (p.as_str(), *a)));
                } else {
                    self.call_rt("string_truncate", values, None);
                }
                Ok(())
            }
            // A `String`'s text is its first two words.
            Builtin::StringAsStr | Builtin::StrAsBytes => {
                let (words, align) = self.view_words(&args[0])?;
                let bytes = self.load(Sc::Ptr, &words, align.min(8));
                let len_at = self.gep(&words, 8);
                let len = self.load(Sc::I(64), &len_at, align.min(8));
                self.store_view(dest, &bytes, &len)
            }
            // `[SPN-1]` — a list's pointer and length; a fixed array's storage and its length.
            Builtin::SpanFrom { .. } => {
                let container = self.pointer(&args[0])?;
                let source = match self.m.ty.kind(arg_ty) {
                    TyKind::Ref { inner, .. } => *inner,
                    _ => arg_ty,
                };
                match *self.m.ty.kind(source) {
                    TyKind::Array { len, .. } => self.store_view(dest, &container, &len.to_string()),
                    _ => {
                        let bytes = self.load(Sc::Ptr, &container, 8);
                        let len_at = self.gep(&container, 8);
                        let len = self.load(Sc::I(64), &len_at, 8);
                        self.store_view(dest, &bytes, &len)
                    }
                }
            }
            Builtin::SpanReborrow | Builtin::SpanSharedReborrow => {
                let (words, align) = self.view_words(&args[0])?;
                let bytes = self.load(Sc::Ptr, &words, align.min(8));
                let len_at = self.gep(&words, 8);
                let len = self.load(Sc::I(64), &len_at, align.min(8));
                self.store_view(dest, &bytes, &len)
            }
            // `[SPN-2]`, `[SPN-5]` — the element's address; the bound is checked in MIR (or is the
            // caller's obligation, `[UNS-4]`).
            Builtin::SpanGetUnchecked | Builtin::SpanIterNext { .. } => {
                let elem = match which {
                    Builtin::SpanIterNext { elem, .. } => *elem,
                    _ => self.span_element(arg_ty),
                };
                let (words, align) = self.view_words(&args[0])?;
                let base = self.load(Sc::Ptr, &words, align.min(8));
                let index = self.word(&args[1])?;
                let size = self.m.ty.size_align(elem).0;
                let element = self.element_at(&base, &index, size);
                self.store_result(dest, &element, Sc::Ptr, false)
            }
            // `[SPN-8]`, `[SPN-9]` — the view's pointer.
            Builtin::SpanAsPtr { .. } => {
                let (words, align) = self.view_words(&args[0])?;
                let base = self.load(Sc::Ptr, &words, align.min(8));
                self.store_result(dest, &base, Sc::Ptr, false)
            }
            // Part VI's slice — the bounds checked in MIR; an offset of 0 keeps the pointer (an
            // empty view may have none).
            Builtin::Slice { text, .. } => {
                let (words, align) = self.view_words(&args[0])?;
                let base = self.load(Sc::Ptr, &words, align.min(8));
                let lo = self.word(&args[1])?;
                let hi = self.word(&args[2])?;
                let size = if *text { 1 } else { self.m.ty.size_align(self.span_element(arg_ty)).0 };
                let start = self.element_at(&base, &lo, size);
                let len = self.value(format!("sub i64 {hi}, {lo}"));
                self.store_view(dest, &start, &len)
            }
            // `[UNS-5]` — the raw memory primitives; `arg_ty` is the element type for `alloc`.
            Builtin::MemAlloc => {
                let elem = self.element_of(arg_ty);
                let (size, align) = self.m.ty.size_align(elem);
                let count = self.word(&args[0])?;
                let bytes = self.value(format!("mul i64 {count}, {size}"));
                let dest = self.dest_of(dest)?;
                self.call_rt("alloc", vec![Val::Imm(Sc::I(64), bytes), size_word(align)], dest.as_ref().map(|(p, a)| (p.as_str(), *a)));
                Ok(())
            }
            Builtin::MemFree => {
                let elem = self.element_of(arg_ty);
                let (size, align) = self.m.ty.size_align(elem);
                let block = self.pointer(&args[0])?;
                let count = self.word(&args[1])?;
                let bytes = self.value(format!("mul i64 {count}, {size}"));
                self.call_rt("free", vec![Val::Imm(Sc::Ptr, block), Val::Imm(Sc::I(64), bytes), size_word(align)], None);
                Ok(())
            }
            Builtin::PtrRead | Builtin::PtrWrite => {
                let pointee = match self.m.ty.kind(arg_ty) {
                    TyKind::Ptr { inner, .. } => *inner,
                    _ => return unsupported("a pointer read through a non-pointer"),
                };
                let base = self.pointer(&args[0])?;
                let index = self.word(&args[1])?;
                let (size, align) = self.m.ty.size_align(pointee);
                let element = self.element_at(&base, &index, size);
                if matches!(which, Builtin::PtrRead) {
                    let at = self.place(dest)?;
                    let value = match self.m.ty.repr(pointee) {
                        Repr::Scalar(sc) => Val::Imm(sc, self.load(sc, &element, align)),
                        Repr::Zst => Val::Zst,
                        Repr::Mem { .. } => Val::Mem(element, align),
                    };
                    self.store(&value, &at.ptr, at.align, size, true);
                } else {
                    let value = self.operand(&args[2])?;
                    self.store(&value, &element, align, size, true);
                }
                Ok(())
            }
            Builtin::PtrIsNull => {
                let pointer = self.pointer(&args[0])?;
                let null = self.value(format!("icmp eq ptr {pointer}, null"));
                self.store_result(dest, &null, Sc::Bool, false)
            }
            Builtin::PtrAdd => {
                let pointee = match self.m.ty.kind(arg_ty) {
                    TyKind::Ptr { inner, .. } => *inner,
                    _ => return unsupported("pointer arithmetic on a non-pointer"),
                };
                let base = self.pointer(&args[0])?;
                let count = self.word(&args[1])?;
                let size = self.m.ty.size_align(pointee).0;
                let moved = self.element_at(&base, &count, size);
                self.store_result(dest, &moved, Sc::Ptr, false)
            }
            Builtin::SizeOf | Builtin::AlignOf => {
                let (size, align) = self.m.ty.size_align(arg_ty);
                let value = if matches!(which, Builtin::SizeOf) { size } else { align };
                self.store_result(dest, &value.to_string(), Sc::I(64), false)
            }
            // `[ECS-4]` — a constant of the instance.
            Builtin::TypeIsSend | Builtin::TypeIsSync => {
                let table = self.m.ty.table;
                let holds = if matches!(which, Builtin::TypeIsSend) { table.is_send(arg_ty) } else { table.is_sync(arg_ty) };
                self.store_result(dest, if holds { "true" } else { "false" }, Sc::Bool, false)
            }
            // IX.1 — the value moved into a block of its own.
            Builtin::BoxNew { elem, .. } => {
                let value = self.operand(&args[0])?;
                let (size, align) = self.m.ty.size_align(*elem);
                let (source, _) = self.spill(&value, size, align);
                let dest = self.dest_of(dest)?;
                self.call_rt(
                    "box_new_copy",
                    vec![size_word(size), size_word(align), Val::Imm(Sc::Ptr, source)],
                    dest.as_ref().map(|(p, a)| (p.as_str(), *a)),
                );
                Ok(())
            }
            // `[HEAP-5]` — the value moved into a counted object of its own.
            Builtin::SharedNew { elem, shared } => {
                let TyKind::Struct(id) = *self.m.ty.kind(*shared) else {
                    return unsupported("a `Shared` that is not a struct");
                };
                let info = crate::glue::shared_type_info(self.m, id)?;
                let value = self.operand(&args[0])?;
                let (size, align) = self.m.ty.size_align(*elem);
                let (source, _) = self.spill(&value, size, align);
                let dest = self.dest_of(dest)?;
                self.call_rt(
                    "obj_new_copy",
                    vec![Val::Imm(Sc::Ptr, info), size_word(align), size_word(size), Val::Imm(Sc::Ptr, source)],
                    dest.as_ref().map(|(p, a)| (p.as_str(), *a)),
                );
                Ok(())
            }
            // `[WK-*]` — a weak handle takes a weak count; `upgrade` the runtime's compare-exchange
            // (`[RT-10]`, `[WK-12]`); an empty one holds nothing.
            Builtin::WeakNew { .. } => {
                let handle = self.pointer(&args[0])?;
                self.call_rt("weak_retain", vec![Val::Imm(Sc::Ptr, handle.clone())], None);
                self.store_result(dest, &handle, Sc::Ptr, false)
            }
            Builtin::WeakUpgrade { .. } => {
                let (words, align) = self.view_words(&args[0])?;
                let handle = self.load(Sc::Ptr, &words, align.min(8));
                let dest = self.dest_of(dest)?;
                self.call_rt("weak_upgrade", vec![Val::Imm(Sc::Ptr, handle)], dest.as_ref().map(|(p, a)| (p.as_str(), *a)));
                Ok(())
            }
            Builtin::WeakEmpty { .. } | Builtin::MaybeUninitUninit { .. } => {
                let at = self.place(dest)?;
                let size = self.m.ty.size_align(at.ty).0;
                self.zero(&at.ptr, at.align, size);
                Ok(())
            }
            // `[UNS-10]` — the cell's `value` field's address.
            Builtin::UnsafeCellGet { .. } => {
                let cell = self.pointer(&args[0])?;
                let cell_ty = match self.m.ty.kind(arg_ty) {
                    TyKind::Ref { inner, .. } => *inner,
                    _ => arg_ty,
                };
                let offset = match self.m.ty.kind(cell_ty) {
                    TyKind::Struct(id) => {
                        let def = self.m.ty.table.struct_def(*id);
                        let index = def.fields.iter().position(|field| field.name.as_str() == "value").unwrap_or(0);
                        self.m.ty.table.layout(cell_ty).field_offsets.get(index).copied().unwrap_or(0)
                    }
                    _ => 0,
                };
                let value = self.gep(&cell, offset);
                self.store_result(dest, &value, Sc::Ptr, false)
            }
            // `[LEX-19]`, `[TYP-39]` — an f-string's piece appended to its text.
            Builtin::Format | Builtin::FormatWith(_) => {
                let out = self.pointer(&args[0])?;
                let value = self.operand(&args[1])?;
                let spec = match which {
                    Builtin::FormatWith(spec) => Some(*spec),
                    _ => None,
                };
                self.format_piece(&out, value, arg_ty, spec.as_ref())
            }
            // `format_to`: a text over the caller's bytes, and how many it holds at the end.
            Builtin::FmtSinkOpen => {
                let (words, align) = self.view_words(&args[0])?;
                let bytes = self.load(Sc::Ptr, &words, align.min(8));
                let len_at = self.gep(&words, 8);
                let len = self.load(Sc::I(64), &len_at, align.min(8));
                let dest = self.dest_of(dest)?;
                self.call_rt("fmt_sink_open", vec![Val::Imm(Sc::Ptr, bytes), Val::Imm(Sc::I(64), len)], dest.as_ref().map(|(p, a)| (p.as_str(), *a)));
                Ok(())
            }
            Builtin::FmtSinkClose => {
                let sink = self.pointer(&args[0])?;
                let slot = self.alloca(8, 8);
                self.call_rt("fmt_sink_close", vec![Val::Imm(Sc::Ptr, sink)], Some((&slot, 8)));
                let held = self.load(Sc::I(64), &slot, 8);
                self.store_result(dest, &held, Sc::I(64), false)
            }
            Builtin::SpanToStr => {
                let (words, align) = self.view_words(&args[0])?;
                let bytes = self.load(Sc::Ptr, &words, align.min(8));
                let len_at = self.gep(&words, 8);
                let len = self.load(Sc::I(64), &len_at, align.min(8));
                self.store_view(dest, &bytes, &len)
            }
            // A standard function whose body is the runtime's.
            Builtin::Runtime(function) => {
                let mut values = Vec::with_capacity(args.len());
                for argument in args {
                    values.push(self.operand(argument)?);
                }
                let entry = crate::runtime::find(function.runtime());
                let values = self.fit_args(entry, values)?;
                let dest = self.dest_of(dest)?;
                self.call_rt(entry.name, values, dest.as_ref().map(|(p, a)| (p.as_str(), *a)));
                Ok(())
            }
            Builtin::SoA { op, soa } => self.soa_op(op, *soa, args, dest, span),
            // D-498 — a step whose lead byte, 0x80 or above, is already read (`ascii_runs`).
            Builtin::StrCharNextWide => {
                let (words, align) = self.view_words(&args[0])?;
                let base = self.load(Sc::Ptr, &words, align.min(8));
                let cursor = self.pointer(&args[1])?;
                let lead = self.operand(&args[2])?;
                let lead = match lead {
                    Val::Imm(Sc::I(bits), value) => Val::Imm(Sc::I(32), self.int_convert(&value, bits, 32, false)),
                    other => other,
                };
                self.call_rt_into("str_char_next_wide_usize", vec![Val::Imm(Sc::Ptr, base), Val::Imm(Sc::Ptr, cursor), lead], dest)
            }
            // `[FFI-23]` — a class object is pinned in its header (or, with an `on_relocate`
            // callback, recorded with it, `[HR-20]`); a `Box`'s value never moves: `0`.
            Builtin::RetainedPin => {
                if matches!(self.m.ty.kind(arg_ty), TyKind::Struct(_)) {
                    return self.store_result(dest, "0", Sc::I(64), false);
                }
                let object = self.pointer(&args[0])?;
                let callback = match args.get(1) {
                    None => "null".to_string(),
                    Some(argument) => match self.operand_ty(argument).map(|ty| (ty, self.m.ty.kind(ty).clone())) {
                        Some((ty, TyKind::Enum(id))) if self.m.ty.table.option_niche(id).is_none() => {
                            let value = self.operand(argument)?;
                            let (size, align) = self.m.ty.size_align(ty);
                            let (at, align) = self.spill(&value, size, align);
                            let place = crate::func::PlaceRef { ptr: at.clone(), ty, variant: None, align };
                            let (discriminant, bits) = self.discriminant_at(&place)?;
                            let some = self.m.ty.table.enum_def(id).variants[1].discriminant;
                            let is_some = self.value(format!("icmp eq i{bits} {discriminant}, {some}"));
                            let payload_at = self.gep(&at, self.m.ty.table.variant_field_offsets(id)[1][0]);
                            let payload = self.load(Sc::Ptr, &payload_at, 8);
                            self.value(format!("select i1 {is_some}, ptr {payload}, ptr null"))
                        }
                        _ => self.pointer(argument)?,
                    },
                };
                let id = self.alloca(8, 8);
                let none = self.value(format!("icmp eq ptr {callback}, null"));
                let (pin, record, done) = (self.label(), self.label(), self.label());
                self.emit(format!("br i1 {none}, label %{pin}, label %{record}"));
                self.start(&pin);
                self.call_rt("obj_pin", vec![Val::Imm(Sc::Ptr, object.clone())], None);
                self.emit(format!("store i64 0, ptr {id}, align 8"));
                self.emit(format!("br label %{done}"));
                self.start(&record);
                self.call_rt("obj_pin_relocatable", vec![Val::Imm(Sc::Ptr, object), Val::Imm(Sc::Ptr, callback)], Some((&id, 8)));
                self.emit(format!("br label %{done}"));
                self.start(&done);
                let id = self.load(Sc::I(64), &id, 8);
                self.store_result(dest, &id, Sc::I(64), false)
            }
            Builtin::RetainedUnpin => {
                let id = self.operand(&args[0])?;
                self.call_rt("obj_unpin_relocatable", vec![id], None);
                Ok(())
            }
            // The object's address: a handle's or a `Box`'s pointer, a `Box[dyn]`'s data.
            Builtin::RetainedAddress | Builtin::CppObjectAddress => {
                let address = self.pointer(&args[0])?;
                self.store_result(dest, &address, Sc::Ptr, false)
            }
            // `[UNS-5]` — through a pointer to a volatile object: each access is made.
            Builtin::VolatileRead | Builtin::VolatileWrite => {
                let TyKind::Ptr { inner, .. } = *self.m.ty.kind(arg_ty) else { return unsupported("a volatile access not through a pointer") };
                let Repr::Scalar(sc) = self.m.ty.repr(inner) else { return unsupported("a volatile access of an aggregate") };
                let (_, align) = self.m.ty.size_align(inner);
                let pointer = self.pointer(&args[0])?;
                if matches!(which, Builtin::VolatileRead) {
                    let value = self.value(format!("load volatile {}, ptr {pointer}, align {align}", sc.mem()));
                    let value = if sc == Sc::Bool { self.value(format!("trunc i8 {value} to i1")) } else { value };
                    return self.store_result(dest, &value, sc, false);
                }
                let Val::Imm(_, value) = self.operand(&args[1])? else { return unsupported("a volatile write of an aggregate") };
                let value = if sc == Sc::Bool { self.value(format!("zext i1 {value} to i8")) } else { value };
                self.emit(format!("store volatile {} {value}, ptr {pointer}, align {align}", sc.mem()));
                Ok(())
            }
            // `[FFI-8]` — a C union with one field set, the rest of its storage zero.
            Builtin::UnionLit { .. } => {
                let ty = self.place_ty(dest);
                let Some((at, align)) = self.dest_of(dest)? else { return Ok(()) };
                let (size, _) = self.m.ty.size_align(ty);
                self.zero(&at, align, size);
                let value = self.operand(&args[0])?;
                let field_ty = self.operand_ty(&args[0]);
                match value {
                    Val::Imm(sc, value) => self.store_scalar(sc, &value, &at, align),
                    Val::Mem(from, from_align) => {
                        let field_size = field_ty.map_or(size, |ty| self.m.ty.size_align(ty).0);
                        self.copy(&at, align, &from, from_align, field_size, false);
                    }
                    Val::Zst => {}
                }
                Ok(())
            }
            // `[FFI-39]`, `[FFI-17d]` — a C++ object's handle again: retained, or adopted.
            Builtin::CppObjectHandle { adopt } => {
                let object = self.pointer(&args[0])?;
                if *adopt {
                    let loc = self.loc(span);
                    return self.call_rt_into("cpp_object_adopt", vec![Val::Imm(Sc::Ptr, object), loc], dest);
                }
                self.call_rt("obj_retain", vec![Val::Imm(Sc::Ptr, object.clone())], None);
                self.store_result(dest, &object, Sc::Ptr, false)
            }
            // `[FFI-39]` (ODR-221) — the object's class overrides the method when its table's slot
            // is not the owner's own.
            Builtin::CppOverrides { owner, slot } => {
                let own = self.m.dispatch.virtual_layouts.get(owner).and_then(|slots| slots.get(*slot)).and_then(|method| method.clone());
                let Some(own) = own.filter(|method| !method.is_abstract) else {
                    return self.store_result(dest, "true", Sc::Bool, false);
                };
                crate::func::declare_body_symbol(self.m, &own.symbol)?;
                let object = self.pointer(&args[0])?;
                let entry = self.vtable_entry(&object, *slot);
                let differs = self.value(format!("icmp ne ptr {entry}, {}", crate::global_name(&own.symbol)));
                self.store_result(dest, &differs, Sc::Bool, false)
            }
            Builtin::CppObjectGive => {
                let object = self.pointer(&args[0])?;
                self.call_rt("obj_retain", vec![Val::Imm(Sc::Ptr, object.clone())], None);
                self.call_rt("obj_pin", vec![Val::Imm(Sc::Ptr, object)], None);
                Ok(())
            }
            // `[FFI-17e]` — a C++ container's elements as a view, a smart pointer's object as a
            // reference: the owner (argument 0) is what they borrow.
            Builtin::CppSpan { .. } => {
                let pointer = self.pointer(&args[1])?;
                let len = self.word(&args[2])?;
                let Some((at, align)) = self.dest_of(dest)? else { return Ok(()) };
                self.emit(format!("store ptr {pointer}, ptr {at}, align {}", align.min(8)));
                let len_at = self.gep(&at, 8);
                self.emit(format!("store i64 {len}, ptr {len_at}, align {}", align.min(8)));
                Ok(())
            }
            Builtin::CppPointee => {
                let pointer = self.pointer(&args[1])?;
                self.store_result(dest, &pointer, Sc::Ptr, false)
            }
            // `[THR-3]` — on the lock word, whose address lowering passes; `[THR-4]` — the lock
            // taken is reported, with where, for the debug profile's lock order record.
            Builtin::MutexLock | Builtin::MutexTryLock | Builtin::RwLockAcquire { .. } | Builtin::RwLockTryAcquire { .. } => {
                let word = self.pointer(&args[0])?;
                let mut values = vec![Val::Imm(Sc::Ptr, word.clone())];
                let (entry, tries) = match which {
                    Builtin::MutexLock => ("mutex_lock", false),
                    Builtin::MutexTryLock => ("mutex_try_lock", true),
                    Builtin::RwLockAcquire { write } => {
                        values.push(Val::Imm(Sc::Bool, write.to_string()));
                        ("rwlock_acquire", false)
                    }
                    Builtin::RwLockTryAcquire { write } => {
                        values.push(Val::Imm(Sc::Bool, write.to_string()));
                        ("rwlock_try_acquire", true)
                    }
                    _ => unreachable!("matched above"),
                };
                let loc = self.loc(span);
                if !tries {
                    self.call_rt(entry, values, None);
                    self.call_rt("lock_order_taken", vec![Val::Imm(Sc::Ptr, word), loc], None);
                    return Ok(());
                }
                let taken = self.alloca(1, 1);
                self.call_rt(entry, values, Some((&taken, 1)));
                let taken = self.load(Sc::Bool, &taken, 1);
                let (record, done) = (self.label(), self.label());
                self.emit(format!("br i1 {taken}, label %{record}, label %{done}"));
                self.start(&record);
                self.call_rt("lock_order_taken", vec![Val::Imm(Sc::Ptr, word), loc], None);
                self.emit(format!("br label %{done}"));
                self.start(&done);
                self.store_result(dest, &taken, Sc::Bool, false)
            }
            Builtin::Atomic { op, order } => self.atomic(*op, *order, arg_ty, args, dest, span),
            Builtin::Simd { op, vector } => self.simd(op, *vector, args, dest, span),
            // `[RFL-2]` — the description the object's own class's type information reaches.
            Builtin::ClassTypeInfo { .. } => {
                let object = self.pointer(&args[0])?;
                let info_at = self.gep(&object, 16);
                let info = self.load(Sc::Ptr, &info_at, 8);
                let reflect_at = self.gep(&info, 88);
                let reflect = self.load(Sc::Ptr, &reflect_at, 8);
                let description = self.value(format!("call ptr {reflect}()"));
                self.store_result(dest, &description, Sc::Ptr, false)
            }
            // `[CT-5]` — the constant data the evaluator computed.
            Builtin::ConstStatic { name, ty } => {
                let address = crate::statics::constant_static(self.m, name.as_str(), *ty)?;
                self.store_result(dest, &address, Sc::Ptr, false)
            }
            // `[STA-3]` — the accessor that runs the initialiser the first time any thread asks.
            Builtin::LazyStatic { name, init, ty } => {
                let accessor = crate::statics::lazy_static(self.m, name.as_str(), init.as_str(), *ty)?;
                let address = self.value(format!("call ptr {accessor}()"));
                self.store_result(dest, &address, Sc::Ptr, false)
            }
            // `[FFI-10]` — the C global's address (through its accessor where the shim has one).
            Builtin::ForeignStatic { symbol, thunk, .. } => {
                let name = crate::global_name(symbol.as_str());
                let address = if *thunk {
                    self.m.declare(&crate::link_symbol(symbol.as_str()), || format!("declare ptr {name}()"));
                    self.value(format!("call ptr {name}()"))
                } else {
                    self.m.declare(&crate::link_symbol(symbol.as_str()), || format!("{name} = external global [0 x i8]"));
                    name
                };
                self.store_result(dest, &address, Sc::Ptr, false)
            }
            // `[FFI-36a]`, `[FFI-36b]`, `[FFI-33b]` — the runtime's record of foreign objects.
            Builtin::ForeignAdopt => {
                let object = self.pointer(&args[0])?;
                let loc = self.loc(span);
                self.call_rt_into("ffi_adopt", vec![Val::Imm(Sc::Ptr, object), loc], dest)
            }
            Builtin::ForeignRelease => {
                let object = self.pointer(&args[0])?;
                let creator = self.operand(&args[1])?;
                self.call_rt("ffi_release", vec![Val::Imm(Sc::Ptr, object), creator], None);
                Ok(())
            }
            Builtin::ForeignThread => self.call_rt_into("ffi_thread", Vec::new(), dest),
            // A destructor with a result is called through its `void` thunk.
            Builtin::ForeignDestructor { symbol, ret, .. } => {
                let given = self.pointer(&args[0])?;
                let destructor = if self.m.ty.is_zst(*ret) { given } else { crate::ffi::foreign_destructor(self.m, symbol.as_str(), *ret)? };
                self.store_result(dest, &destructor, Sc::Ptr, false)
            }
            other => unsupported(format!("the built-in `{}`", other.name())),
        }
    }

    /// `[STD-27]`, `[STD-3]` — the C library's mathematics (`sqrt`, `hypot`, …; `f` for `f32`), as
    /// the C backend calls it; the predicates and `abs` by the bits, as the runtime's helpers
    /// are; an `f16` through `double`, rounded back once.
    fn float_lib(&mut self, which: &Builtin, arg_ty: Ty, args: &[Operand], dest: &Place) -> Result<()> {
        let ty = self.m.ty.through_range(arg_ty);
        let half = self.m.ty.is_f16(ty);
        let mut values = Vec::with_capacity(args.len());
        for argument in args {
            match self.operand(argument)? {
                Val::Imm(sc, value) => values.push((value, sc)),
                other => return unsupported(format!("a float held as {other:?}")),
            }
        }
        let predicate = match which {
            Builtin::FloatLib(f) if f.is_predicate() => Some(*f),
            _ => None,
        };
        if let Some(predicate) = predicate {
            let (value, sc) = values[0].clone();
            let truth = if half {
                let magnitude = self.value(format!("and i16 {value}, 32767"));
                match predicate {
                    ember_hir::FloatLib::IsNan => self.value(format!("icmp ugt i16 {magnitude}, 31744")),
                    ember_hir::FloatLib::IsInfinite => self.value(format!("icmp eq i16 {magnitude}, 31744")),
                    _ => self.value(format!("icmp ult i16 {magnitude}, 31744")),
                }
            } else {
                let ir = sc.ir();
                match predicate {
                    ember_hir::FloatLib::IsNan => self.value(format!("fcmp uno {ir} {value}, {value}")),
                    other => {
                        let intrinsic = format!("llvm.fabs.{}", float_name(sc));
                        self.m.declare(&intrinsic, || format!("declare {ir} @{intrinsic}({ir})"));
                        let magnitude = self.value(format!("call {ir} @{intrinsic}({ir} {value})"));
                        let infinity = crate::func::float_literal(f64::INFINITY, sc);
                        if other == ember_hir::FloatLib::IsInfinite {
                            self.value(format!("fcmp oeq {ir} {magnitude}, {infinity}"))
                        } else {
                            self.value(format!("fcmp olt {ir} {magnitude}, {infinity}"))
                        }
                    }
                }
            };
            return self.store_result(dest, &truth, Sc::Bool, false);
        }
        // Widen an `f16`'s operands to `double`.
        let mut operands = Vec::with_capacity(values.len());
        for (value, sc) in values {
            if half {
                let slot = self.alloca(8, 8);
                self.call_rt("f16_to_f64", vec![Val::Imm(Sc::I(16), value)], Some((&slot, 8)));
                operands.push((self.load(Sc::F64, &slot, 8), Sc::F64));
            } else {
                operands.push((value, sc));
            }
        }
        let sc = operands[0].1;
        let ir = sc.ir();
        let result = match which {
            Builtin::FloatAbs => {
                let intrinsic = format!("llvm.fabs.{}", float_name(sc));
                self.m.declare(&intrinsic, || format!("declare {ir} @{intrinsic}({ir})"));
                self.value(format!("call {ir} @{intrinsic}({ir} {})", operands[0].0))
            }
            _ => {
                let base = match which {
                    Builtin::FloatPow => "pow",
                    Builtin::FloatLib(f) => f.c_name(),
                    _ => unreachable!("a float library call"),
                };
                let name = if sc == Sc::F32 { format!("{base}f") } else { base.to_string() };
                let params = vec![ir.clone(); operands.len()].join(", ");
                let symbol = global_name(&name);
                self.m.declare(&name, || format!("declare {ir} {symbol}({params}) nounwind"));
                let arguments: Vec<String> = operands.iter().map(|(value, _)| format!("{ir} {value}")).collect();
                self.value(format!("call {ir} {symbol}({})", arguments.join(", ")))
            }
        };
        if half {
            let slot = self.alloca(2, 2);
            self.call_rt("f64_to_f16", vec![Val::Imm(Sc::F64, result)], Some((&slot, 2)));
            let bits = self.load(Sc::I(16), &slot, 2);
            return self.store_result(dest, &bits, Sc::I(16), false);
        }
        self.store_result(dest, &result, sc, false)
    }

    /// Arguments converted to the C parameter types of `entry` where C converts them implicitly:
    /// an integer to another width.
    fn fit_args(&mut self, entry: &crate::runtime::RtFn, values: Vec<Val>) -> Result<Vec<Val>> {
        let mut out = Vec::with_capacity(values.len());
        for (value, param) in values.into_iter().zip(entry.params) {
            let fitted = match (crate::runtime::c_type(param), value) {
                (crate::runtime::CTy::Scalar(crate::abi::Scalar::Int { bytes, signed }), Val::Imm(Sc::I(bits), text)) if bits != bytes as u32 * 8 => {
                    Val::Imm(Sc::I(bytes as u32 * 8), self.int_convert(&text, bits, bytes as u32 * 8, signed))
                }
                (_, value) => value,
            };
            out.push(fitted);
        }
        Ok(out)
    }

    /// Where a call's result goes, or `None` when it has no bytes.
    pub(crate) fn dest_of(&mut self, dest: &Place) -> Result<Option<(String, u64)>> {
        let ty = self.place_ty(dest);
        if self.m.ty.is_zst(ty) {
            return Ok(None);
        }
        let at = self.place(dest)?;
        Ok(Some((at.ptr, at.align.min(offset_align(0)))))
    }
}
