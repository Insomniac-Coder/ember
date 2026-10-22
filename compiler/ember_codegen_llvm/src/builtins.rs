//! The calls the compiler provides itself (`FuncRef::Builtin`): each lowered to the runtime entry
//! the C backend's C calls, or to the few instructions the C backend's expression is.

use crate::func::{FnCx, Val};
use crate::types::{Repr, Sc, offset_align};
use crate::{Result, unsupported};
use ember_mir::{Builtin, Operand, Place};
use ember_types::{Ty, TyKind};

impl<'a> FnCx<'_, 'a> {
    /// A pointer operand's value (a `ref`, a `ref mut`, a raw pointer, a class handle).
    pub(crate) fn pointer(&mut self, operand: &Operand) -> Result<String> {
        match self.operand(operand)? {
            Val::Imm(Sc::Ptr, value) => Ok(value),
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
    fn element_of(&self, ty: Ty) -> Ty {
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
    fn view_words(&mut self, operand: &Operand) -> Result<(String, u64)> {
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
    fn store_result(&mut self, dest: &Place, value: &str, sc: Sc, signed: bool) -> Result<()> {
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

    pub(crate) fn builtin_call(&mut self, which: &Builtin, arg_ty: Ty, args: &[Operand], dest: &Place, span: ember_span::Span) -> Result<()> {
        let size_word = |size: u64| Val::Imm(Sc::I(64), size.to_string());
        match which {
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
                let offset = self.value(format!("mul i64 {index}, {size}"));
                let element = self.value(format!("getelementptr i8, ptr {base}, i64 {offset}"));
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
                let offset = self.value(format!("mul i64 {lo}, {size}"));
                let start = self.value(format!("getelementptr i8, ptr {base}, i64 {offset}"));
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
                let offset = self.value(format!("mul i64 {index}, {size}"));
                let element = self.value(format!("getelementptr i8, ptr {base}, i64 {offset}"));
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
                let offset = self.value(format!("mul i64 {count}, {size}"));
                let moved = self.value(format!("getelementptr i8, ptr {base}, i64 {offset}"));
                self.store_result(dest, &moved, Sc::Ptr, false)
            }
            Builtin::SizeOf | Builtin::AlignOf => {
                let (size, align) = self.m.ty.size_align(arg_ty);
                let value = if matches!(which, Builtin::SizeOf) { size } else { align };
                self.store_result(dest, &value.to_string(), Sc::I(64), false)
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
            other => unsupported(format!("the built-in `{}`", other.name())),
        }
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
