//! The calls the compiler provides itself (`ember_hir::Builtin`), each done as the runtime and
//! the C backend do it (`call_expression`, `ember_rt.c`): lists and text in the runtime's buffer
//! (`ember_vec`: pointer, length, capacity), printing and formatting, counted values, raw memory.

use ember_hir::{Builtin, FloatLib, IntBits, ParseKind};
use ember_mir::{Operand, Place};
use ember_span::Span;
use ember_types::{Ty, TyKind};

use crate::memory::{self, Region};
use crate::scalar::IntKind;
use crate::{fault, two_words, u64_of, Interp, Loc, Stop, R};

/// `EMBER_VEC_ALIGN`: every list buffer's alignment.
const VEC_ALIGN: u64 = 16;

impl Interp<'_, '_> {
    /// A value's bytes in a block of their own, for the operations that work on an address.
    pub(crate) fn temp(&mut self, bytes: &[u8]) -> R<u64> {
        let addr = self.mem.alloc(bytes.len().max(1) as u64, 16)?;
        self.mem.write(addr, bytes)?;
        Ok(addr)
    }

    /// The pointer an operand holds (a `ref` to a list, a raw pointer).
    pub(crate) fn pointer(&mut self, operand: &Operand) -> R<u64> {
        Ok(u64_of(&self.operand(operand, None)?))
    }

    pub(crate) fn word(&mut self, operand: &Operand) -> R<u64> {
        let bytes = self.operand(operand, Some(self.common.usize))?;
        Ok(u64_of(&bytes))
    }

    pub(crate) fn signed(&mut self, operand: &Operand) -> R<i64> {
        let body = self.body();
        let ty = self.operand_ty(body, operand);
        let bytes = self.operand(operand, ty)?;
        Ok(match ty.and_then(|ty| self.int_kind(ty)) {
            Some(kind) if kind.bits <= 64 => kind.signed_value(&bytes) as i64,
            _ => u64_of(&bytes) as i64,
        })
    }

    /// The `str` (or `String`, read as one) an operand holds.
    pub(crate) fn text_operand(&mut self, operand: &Operand) -> R<Vec<u8>> {
        let value = self.operand(operand, None)?;
        self.str_bytes(&value)
    }

    /// An integer result of the destination's type.
    pub(crate) fn int_result(&self, dest: Ty, value: i128) -> Vec<u8> {
        let size = self.size(dest) as usize;
        value.to_le_bytes()[..size.min(16)].to_vec()
    }

    // -- the list buffer (`ember_vec`) -------------------------------------------------

    /// `vec_reserve_at`: room for `want` elements, doubling (four at least), exactly `want`
    /// past double.
    pub(crate) fn vec_reserve(&mut self, vec: u64, elem: u64, want: u64, align: u64) -> R<()> {
        let (ptr, cap) = (self.mem.read_u64(vec)?, self.mem.read_u64(vec + 16)?);
        if want <= cap {
            return Ok(());
        }
        let limit = if elem == 0 { i64::MAX as u64 } else { i64::MAX as u64 / elem };
        if want > limit {
            return self.panic_in("<array>", "capacity overflow");
        }
        let mut next = if cap < 2 { 4 } else { cap * 2 };
        next = next.min(limit).max(want);
        let block = self.mem.alloc((next * elem).max(1), align.max(VEC_ALIGN))?;
        if ptr != 0 {
            let len = self.mem.read_u64(vec + 8)?;
            self.mem.copy(ptr, block, len * elem)?;
            self.mem.free(ptr)?;
        }
        self.mem.write_u64(vec, block)?;
        self.mem.write_u64(vec + 16, next)?;
        Ok(())
    }

    fn vec_reserve_more(&mut self, vec: u64, elem: u64, additional: u64, align: u64) -> R<()> {
        let len = self.mem.read_u64(vec + 8)?;
        let limit = if elem == 0 { i64::MAX as u64 } else { i64::MAX as u64 / elem };
        if len > limit || additional > limit - len {
            return self.panic_in("<array>", "capacity overflow");
        }
        self.vec_reserve(vec, elem, len + additional, align)
    }

    pub(crate) fn vec_push(&mut self, vec: u64, elem: u64, value: &[u8], align: u64) -> R<()> {
        let (len, cap) = (self.mem.read_u64(vec + 8)?, self.mem.read_u64(vec + 16)?);
        if len == cap {
            self.vec_reserve(vec, elem, len + 1, align)?;
        }
        let ptr = self.mem.read_u64(vec)?;
        self.mem.write(ptr + len * elem, value)?;
        self.mem.write_u64(vec + 8, len + 1)?;
        Ok(())
    }

    /// `ember_vec_extend`: `bytes` appended to a `String`.
    pub(crate) fn vec_extend(&mut self, vec: u64, bytes: &[u8]) -> R<()> {
        if bytes.is_empty() {
            return Ok(());
        }
        let (ptr, len, cap) = (self.mem.read_u64(vec)?, self.mem.read_u64(vec + 8)?, self.mem.read_u64(vec + 16)?);
        if bytes.len() as u64 > cap - len
            && let Some(sink) = self.system.sinks.iter_mut().rev().find(|(base, _)| *base == ptr)
        {
            sink.1 = true;
            return Ok(());
        }
        self.vec_reserve_more(vec, 1, bytes.len() as u64, VEC_ALIGN)?;
        let (ptr, len) = (self.mem.read_u64(vec)?, self.mem.read_u64(vec + 8)?);
        self.mem.write(ptr + len, bytes)?;
        self.mem.write_u64(vec + 8, len + bytes.len() as u64)?;
        Ok(())
    }

    pub(crate) fn vec_free(&mut self, vec: u64) -> R<()> {
        let ptr = self.mem.read_u64(vec)?;
        if ptr != 0 {
            self.mem.free(ptr)?;
        }
        Ok(())
    }

    /// A new list holding exactly `count` elements copied from `elems` (`vec_from_elems`).
    pub(crate) fn vec_from_elems(&mut self, elem: u64, elems: u64, count: u64, align: u64) -> R<Vec<u8>> {
        if count == 0 {
            return Ok(vec![0; 24]);
        }
        let block = self.mem.alloc((count * elem).max(1), align.max(VEC_ALIGN))?;
        self.mem.copy(elems, block, count * elem)?;
        let mut value = two_words(block, count);
        value.extend_from_slice(&count.to_le_bytes());
        Ok(value)
    }

    /// A `String` holding `bytes`.
    pub(crate) fn vec_from_bytes(&mut self, bytes: &[u8]) -> R<Vec<u8>> {
        if bytes.is_empty() {
            return Ok(vec![0; 24]);
        }
        let block = self.mem.alloc(bytes.len() as u64, VEC_ALIGN)?;
        self.mem.write(block, bytes)?;
        let mut value = two_words(block, bytes.len() as u64);
        value.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
        Ok(value)
    }

    fn element_of(&self, ty: Ty) -> Ty {
        match self.types.kind(ty) {
            TyKind::Vec { elem, .. } | TyKind::Span { elem, .. } | TyKind::Array { elem, .. } => *elem,
            TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. } => self.element_of(*inner),
            _ => ty,
        }
    }

    fn vec_align(&self, elem: Ty) -> u64 {
        self.layout(elem).align.max(VEC_ALIGN)
    }

    // -- the builtins --------------------------------------------------------------------

    /// One builtin call: its result's bytes for the destination, or `None` when it wrote the
    /// destination itself (or has no result).
    pub(crate) fn builtin(&mut self, which: &Builtin, arg_ty: Ty, args: &[Operand], dest: &Place, span: Span) -> R<Option<Vec<u8>>> {
        let body = self.body();
        let dest_ty = self.place_ty(body, dest);
        Ok(Some(match which {
            Builtin::Println | Builtin::Print | Builtin::EPrintln | Builtin::EPrint => {
                self.print(which, arg_ty, &args[0])?;
                return Ok(None);
            }
            Builtin::SoA { op, soa } => return self.soa(*op, *soa, args, dest, span),
            // `[CT-2]` — the evaluator's own inputs while compiling.
            Builtin::ComptimeReadFile => {
                let path = String::from_utf8_lossy(&self.text_operand(&args[0])?).into_owned();
                self.comptime_read_file(&path)?
            }
            Builtin::ComptimeEnv { option } => {
                self.comptime_env(*option, args, dest)?;
                return Ok(None);
            }
            Builtin::Simd { op, vector } => return self.simd(*op, *vector, args, dest_ty, span),
            Builtin::Atomic { op, order } => return self.atomic(*op, *order, arg_ty, args, span),
            Builtin::MutexLock | Builtin::MutexTryLock | Builtin::RwLockAcquire { .. } | Builtin::RwLockTryAcquire { .. } => {
                let word = self.pointer(&args[0])?;
                let (write, rwlock, trying) = match which {
                    Builtin::MutexLock => (true, false, false),
                    Builtin::MutexTryLock => (true, false, true),
                    Builtin::RwLockAcquire { write } => (*write, true, false),
                    Builtin::RwLockTryAcquire { write } => (*write, true, true),
                    _ => unreachable!(),
                };
                let taken = self.lock(word, write, rwlock, trying)?;
                if !trying {
                    return Ok(None);
                }
                vec![u8::from(taken)]
            }
            Builtin::ArenaWithCapacity
            | Builtin::ArenaAlloc { .. }
            | Builtin::ArenaAllocUninit { .. }
            | Builtin::ArenaAllocArrayZeroed { .. }
            | Builtin::ArenaAllocArrayDefault { .. }
            | Builtin::ArenaReset
            | Builtin::FixedArenaAlloc { .. }
            | Builtin::FixedArenaReset
            | Builtin::ArenaScope { .. }
            | Builtin::ScopedArenaAlloc { .. }
            | Builtin::ArenaArrayWithCapacity { .. }
            | Builtin::ArenaMapWithCapacity { .. } => return self.arena_builtin(which, arg_ty, args, dest_ty),
            // `emit_str_split_once`: only the live variant's tag is written for `None`.
            Builtin::StrSplitOnce { none, some } => {
                let text = self.operand(&args[0], None)?;
                let separator = self.operand(&args[1], None)?;
                let (ptr, len) = (u64_of(&text[..8]), u64_of(&text[8..16]));
                let needle = u64_of(&separator[8..16]);
                let found = search(&self.str_bytes(&text)?, &self.str_bytes(&separator)?, 0);
                let out = self.place(dest)?;
                let TyKind::Enum(option) = *self.types.kind(out.ty) else { return fault("split_once gives no Option") };
                let def = self.types.enum_def(option);
                let repr = self.int_kind(def.repr).expect("an integer tag");
                let Some(at) = found.map(|at| at as u64) else {
                    self.mem.write(out.addr, &repr.encode(def.variants[*none].discriminant as u128))?;
                    return Ok(None);
                };
                let tail = (len - at) - needle;
                let consumed = len - tail;
                self.mem.write(out.addr, &repr.encode(def.variants[*some].discriminant as u128))?;
                let pair = def.variants[*some].fields[0].ty;
                let payload = out.addr + self.types.variant_field_offsets(option)[*some][0];
                let offsets = self.layout(pair).field_offsets.clone();
                self.mem.write(payload + offsets[0], &two_words(ptr, at))?;
                let after = if consumed == 0 { ptr } else { ptr + consumed };
                self.mem.write(payload + offsets[1], &two_words(after, tail))?;
                return Ok(None);
            }
            Builtin::CStrToSpan => {
                let at = self.pointer(&args[0])?;
                let len = self.c_string(at)?.len() as u64;
                two_words(at, len)
            }
            Builtin::CStringAsCStr => {
                let at = self.pointer(&args[0])?;
                let cstring = match self.operand_ty(body, &args[0]).map(|ty| self.types.kind(ty)) {
                    Some(TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. }) => *inner,
                    _ => return fault("`as_cstr` of no `CString`"),
                };
                let bytes = at + self.field_offset(cstring, "bytes")?;
                self.mem.read_u64(bytes)?.to_le_bytes().to_vec()
            }
            // `[DSJ-1]` — two comparisons; an empty view overlaps nothing.
            Builtin::ViewsOverlap { a, b } => {
                let (x, y) = (self.operand(&args[0], None)?, self.operand(&args[1], None)?);
                let size = |interp: &Self, view: Ty| match interp.types.kind(view) {
                    TyKind::Span { elem, .. } => interp.size(*elem),
                    _ => 1,
                };
                let (xp, xl, yp, yl) = (u64_of(&x[..8]), u64_of(&x[8..16]), u64_of(&y[..8]), u64_of(&y[8..16]));
                let overlap = xl != 0 && yl != 0 && xp < yp + yl * size(self, *b) && yp < xp + xl * size(self, *a);
                vec![u8::from(overlap)]
            }
            // `ember_str_char_next_wide_usize`: the character a lead byte of two or more bytes
            // begins, and the cursor past it.
            Builtin::StrCharNextWide => {
                let text = self.operand(&args[0], None)?;
                let cursor = self.pointer(&args[1])?;
                let lead = self.word(&args[2])? as u8;
                let at = self.mem.read_u64(cursor)?;
                let width = utf8_width(lead) as u64;
                let bytes = self.mem.read(u64_of(&text[..8]) + at, width)?.to_vec();
                self.mem.write_u64(cursor, at + width)?;
                decode_char(&bytes).to_le_bytes().to_vec()
            }
            Builtin::Format | Builtin::FormatWith(_) => {
                let out = self.pointer(&args[0])?;
                let ty = self.operand_ty(body, &args[1]).unwrap_or(arg_ty);
                let value = self.operand(&args[1], Some(ty))?;
                let at = self.temp(&value)?;
                let mut text = Vec::new();
                let (at_unwrapped, printed) = self.unwrap_printed(at, ty)?;
                match which {
                    Builtin::FormatWith(spec) => self.display_with(&mut text, at_unwrapped, printed, spec)?,
                    _ => self.display_into(&mut text, at_unwrapped, printed)?,
                }
                self.mem.free(at)?;
                self.vec_extend(out, &text)?;
                return Ok(None);
            }
            Builtin::ArrayNew | Builtin::StringNew => vec![0; 24],
            Builtin::ArrayLen | Builtin::StringLen | Builtin::SpanLen => {
                let value = self.operand(&args[0], None)?;
                let value = if value.len() == 8 { self.mem.read(u64_of(&value), 16)?.to_vec() } else { value };
                self.int_result(dest_ty, u64_of(&value[8..16]) as i128)
            }
            Builtin::ArrayCapacity => {
                let value = self.operand(&args[0], None)?;
                let value = if value.len() == 8 { self.mem.read(u64_of(&value), 24)?.to_vec() } else { value };
                self.int_result(dest_ty, u64_of(&value[16..24]) as i128)
            }
            Builtin::ArrayPush => {
                let vec = self.pointer(&args[0])?;
                let elem = self.element_of(arg_ty);
                // `[RC-1]` — a copied value pushed is retained, as any copy.
                if let Operand::Copy(place) = &args[1] {
                    let loc = self.place(place)?;
                    self.retain_loc(loc, elem)?;
                }
                let value = self.operand(&args[1], Some(elem))?;
                let (size, align) = (self.size(elem), self.vec_align(elem));
                self.vec_push(vec, size, &value, align)?;
                return Ok(None);
            }
            Builtin::ArrayFromLiteral => {
                let TyKind::Array { elem, len } = *self.types.kind(arg_ty) else {
                    return fault("an Array literal from no fixed array");
                };
                let value = self.operand(&args[0], Some(arg_ty))?;
                let at = self.temp(&value)?;
                let (size, align) = (self.size(elem), self.vec_align(elem));
                let result = self.vec_from_elems(size, at, len, align)?;
                self.mem.free(at)?;
                result
            }
            Builtin::StringPush => {
                let vec = self.pointer(&args[0])?;
                let text = self.text_operand(&args[1])?;
                self.vec_extend(vec, &text)?;
                return Ok(None);
            }
            Builtin::StringPushChar => {
                let vec = self.pointer(&args[0])?;
                let c = u64_of(&self.operand(&args[1], Some(self.common.char_))?) as u32;
                let bytes = crate::text::encode_char(c);
                self.vec_extend(vec, &bytes)?;
                return Ok(None);
            }
            Builtin::StringAsStr => {
                let vec = self.pointer(&args[0])?;
                self.mem.read(vec, 16)?.to_vec()
            }
            Builtin::StringInsert => {
                let vec = self.pointer(&args[0])?;
                let at = self.word(&args[1])?;
                let c = u64_of(&self.operand(&args[2], Some(self.common.char_))?) as u32;
                let len = self.mem.read_u64(vec + 8)?;
                if at > len {
                    return self.panic(&format!("index {at} is out of bounds for a length of {len}"), span);
                }
                self.string_boundary(vec, at, span)?;
                let bytes = crate::text::encode_char(c);
                let len = self.mem.read_u64(vec + 8)?;
                self.vec_reserve_more(vec, 1, bytes.len() as u64, VEC_ALIGN)?;
                let ptr = self.mem.read_u64(vec)?;
                self.mem.copy(ptr + at, ptr + at + bytes.len() as u64, len - at)?;
                self.mem.write(ptr + at, &bytes)?;
                self.mem.write_u64(vec + 8, len + bytes.len() as u64)?;
                return Ok(None);
            }
            Builtin::StringRemove => {
                let vec = self.pointer(&args[0])?;
                let at = self.word(&args[1])?;
                let (ptr, len) = (self.mem.read_u64(vec)?, self.mem.read_u64(vec + 8)?);
                if at >= len {
                    return self.panic(&format!("index {at} is out of bounds for a length of {len}"), span);
                }
                self.string_boundary(vec, at, span)?;
                let rest = self.mem.read(ptr + at, len - at)?.to_vec();
                let text = String::from_utf8_lossy(&rest);
                let c = text.chars().next().expect("a character");
                let width = c.len_utf8() as u64;
                self.mem.copy(ptr + at + width, ptr + at, len - at - width)?;
                self.mem.write_u64(vec + 8, len - width)?;
                self.int_result(dest_ty, c as i128)
            }
            Builtin::StringTruncate => {
                let vec = self.pointer(&args[0])?;
                let at = self.word(&args[1])?;
                let len = self.mem.read_u64(vec + 8)?;
                if at < len {
                    self.string_boundary(vec, at, span)?;
                    self.mem.write_u64(vec + 8, at)?;
                }
                return Ok(None);
            }
            Builtin::SpanFrom { .. } => {
                let container = self.pointer(&args[0])?;
                let source = self.element_of_ref(arg_ty);
                match *self.types.kind(source) {
                    TyKind::Array { len, .. } => two_words(container, len),
                    _ => self.mem.read(container, 16)?.to_vec(),
                }
            }
            Builtin::Slice { text, nonempty } => {
                let view = self.operand(&args[0], None)?;
                let (ptr, _) = (u64_of(&view[..8]), u64_of(&view[8..16]));
                let lo = self.word(&args[1])?;
                let hi = self.word(&args[2])?;
                let step = if *text { 1 } else { self.size(self.element_of(arg_ty)) };
                let start = if lo == 0 && !nonempty { ptr } else { ptr + lo * step };
                two_words(start, hi - lo)
            }
            Builtin::StrAsBytes | Builtin::SpanToStr | Builtin::SpanReborrow | Builtin::SpanSharedReborrow => {
                let view = self.operand(&args[0], None)?;
                let view = if view.len() == 8 { self.mem.read(u64_of(&view), 16)?.to_vec() } else { view };
                view[..16].to_vec()
            }
            Builtin::SpanAsPtr { .. } => {
                let view = self.operand(&args[0], None)?;
                let view = if view.len() == 8 { self.mem.read(u64_of(&view), 16)?.to_vec() } else { view };
                view[..8].to_vec()
            }
            Builtin::SpanGetUnchecked => {
                let view = self.operand(&args[0], None)?;
                let i = self.word(&args[1])?;
                let step = self.size(self.element_of(arg_ty));
                (u64_of(&view[..8]) + i * step).to_le_bytes().to_vec()
            }
            Builtin::SpanIterNext { elem, .. } => {
                let iter = self.operand(&args[0], None)?;
                let i = self.word(&args[1])?;
                let step = self.size(*elem);
                (u64_of(&iter[..8]) + i * step).to_le_bytes().to_vec()
            }
            Builtin::SpanChunksNext { elem, .. } => {
                let source = self.operand(&args[0], None)?;
                let start = self.word(&args[1])?;
                let len = self.word(&args[2])?;
                let step = self.size(*elem);
                let ptr = u64_of(&source[..8]);
                two_words(if start == 0 { ptr } else { ptr + start * step }, len)
            }
            Builtin::SpanSplitAt { elem, pair, .. } => {
                let view = self.operand(&args[0], None)?;
                let view = if view.len() == 8 { self.mem.read(u64_of(&view), 16)?.to_vec() } else { view };
                let at = self.word(&args[1])?;
                let (ptr, len) = (u64_of(&view[..8]), u64_of(&view[8..16]));
                let step = self.size(*elem);
                let mut bytes = vec![0u8; self.size(*pair) as usize];
                let offsets = self.layout(*pair).field_offsets.clone();
                bytes[offsets[0] as usize..offsets[0] as usize + 16].copy_from_slice(&two_words(ptr, at));
                let tail = if at == 0 { ptr } else { ptr + at * step };
                bytes[offsets[1] as usize..offsets[1] as usize + 16].copy_from_slice(&two_words(tail, len - at));
                bytes
            }
            Builtin::StrIsCharBoundary => {
                let text = self.text_operand(&args[0])?;
                let i = self.word(&args[1])? as usize;
                vec![u8::from(i == 0 || i == text.len() || (i < text.len() && (text[i] & 0xC0) != 0x80))]
            }
            Builtin::StrCharCount => {
                let text = self.text_operand(&args[0])?;
                let count = text.iter().filter(|b| (**b & 0xC0) != 0x80).count();
                self.int_result(dest_ty, count as i128)
            }
            Builtin::StrCharNext => {
                let text = self.operand(&args[0], None)?;
                let cursor = self.pointer(&args[1])?;
                let at = self.mem.read_u64(cursor)?;
                let ptr = u64_of(&text[..8]);
                let lead = self.mem.read(ptr + at, 1)?[0];
                let width = if lead < 0x80 { 1 } else if lead < 0xE0 { 2 } else if lead < 0xF0 { 3 } else { 4 };
                let bytes = self.mem.read(ptr + at, width)?.to_vec();
                let c = decode_char(&bytes);
                self.mem.write_u64(cursor, at + width)?;
                self.int_result(dest_ty, c as i128)
            }
            Builtin::StrStartsWith | Builtin::StrEndsWith | Builtin::StrContains => {
                let s = self.text_operand(&args[0])?;
                let needle = self.text_operand(&args[1])?;
                let result = match which {
                    Builtin::StrStartsWith => s.starts_with(&needle),
                    Builtin::StrEndsWith => s.ends_with(&needle),
                    _ => search(&s, &needle, 0).is_some(),
                };
                vec![u8::from(result)]
            }
            Builtin::StrContainsChar => {
                let s = self.text_operand(&args[0])?;
                let c = u64_of(&self.operand(&args[1], Some(self.common.char_))?) as u32;
                let needle = crate::text::encode_char(c);
                vec![u8::from(search(&s, &needle, 0).is_some())]
            }
            Builtin::StrFind { reverse } => {
                let s = self.text_operand(&args[0])?;
                let needle = self.text_operand(&args[1])?;
                let at = if *reverse { rsearch(&s, &needle) } else { search(&s, &needle, 0) };
                self.int_result(dest_ty, at.map_or(-1, |at| at as i128))
            }
            Builtin::StrCount => {
                let s = self.text_operand(&args[0])?;
                let needle = self.text_operand(&args[1])?;
                let count = if needle.is_empty() {
                    s.iter().filter(|b| (**b & 0xC0) != 0x80).count() + 1
                } else {
                    let (mut count, mut from) = (0, 0);
                    while let Some(at) = search(&s, &needle, from) {
                        count += 1;
                        from = at + needle.len();
                    }
                    count
                };
                self.int_result(dest_ty, count as i128)
            }
            Builtin::StrReplace => {
                let s = String::from_utf8_lossy(&self.text_operand(&args[0])?).into_owned();
                let from = String::from_utf8_lossy(&self.text_operand(&args[1])?).into_owned();
                let to = String::from_utf8_lossy(&self.text_operand(&args[2])?).into_owned();
                let replaced = if from.is_empty() { python_replace_empty(&s, &to) } else { s.replace(&from, &to) };
                self.vec_from_bytes(replaced.as_bytes())?
            }
            Builtin::StrRepeat => {
                let s = self.text_operand(&args[0])?;
                let n = self.signed(&args[1])?;
                let repeated = if n < 1 { Vec::new() } else { s.repeat(n as usize) };
                self.vec_from_bytes(&repeated)?
            }
            Builtin::StrTrimStart | Builtin::StrTrimEnd => {
                let s = String::from_utf8_lossy(&self.text_operand(&args[0])?).into_owned();
                let offset = if matches!(which, Builtin::StrTrimStart) {
                    s.len() - s.trim_start().len()
                } else {
                    s.trim_end().len()
                };
                self.int_result(dest_ty, offset as i128)
            }
            Builtin::StrSliceOk => {
                let s = self.text_operand(&args[0])?;
                let start = self.signed(&args[1])?;
                let end = self.signed(&args[2])?;
                let inclusive = self.operand(&args[3], Some(self.common.bool_))?[0] != 0;
                let boundary = |i: i64| i == 0 || i as usize == s.len() || (i > 0 && (i as usize) < s.len() && (s[i as usize] & 0xC0) != 0x80);
                let end_ok = if inclusive {
                    end >= 0 && (end as usize) < s.len() && boundary(end) && {
                        let e = end as usize;
                        let width = utf8_width(s[e]);
                        e + width <= s.len()
                    }
                } else {
                    boundary(end)
                };
                vec![u8::from(start >= 0 && end >= start && (end as usize) <= s.len() && boundary(start) && end_ok)]
            }
            Builtin::StrToUpper | Builtin::StrToLower => {
                let s = String::from_utf8_lossy(&self.text_operand(&args[0])?).into_owned();
                let mapped = if matches!(which, Builtin::StrToUpper) { s.to_uppercase() } else { s.to_lowercase() };
                self.vec_from_bytes(mapped.as_bytes())?
            }
            Builtin::Utf8Valid => {
                let view = self.operand(&args[0], None)?;
                let bytes = self.str_bytes(&view)?;
                vec![u8::from(std::str::from_utf8(&bytes).is_ok())]
            }
            Builtin::ParseStatus { kind } => {
                let s = self.text_operand(&args[0])?;
                let max = if args.len() > 1 { Some(u64_of(&self.operand(&args[1], None)?)) } else { None };
                vec![parse_status(*kind, &s, max)]
            }
            Builtin::ParseValue { kind } => {
                let s = String::from_utf8_lossy(&self.text_operand(&args[0])?).into_owned();
                self.parse_value(*kind, &s, dest_ty)?
            }
            Builtin::TotalLess => {
                let a = self.operand(&args[0], Some(arg_ty))?;
                let b = self.operand(&args[1], Some(arg_ty))?;
                vec![u8::from(self.total_less(arg_ty, &a, &b)?)]
            }
            Builtin::ValueCompare { op } => {
                let a = self.operand(&args[0], Some(arg_ty))?;
                let b = self.operand(&args[1], Some(arg_ty))?;
                let result = if self.is_text_ty(arg_ty) {
                    let (x, y) = (self.str_bytes(&a)?, self.str_bytes(&b)?);
                    crate::scalar::compare(*op, x.cmp(&y))
                } else {
                    let (x, y) = (self.temp(&a)?, self.temp(&b)?);
                    let equal = self.equal(x, y, arg_ty)?;
                    self.mem.free(x)?;
                    self.mem.free(y)?;
                    if matches!(op, ember_mir::BinOp::Ne) { !equal } else { equal }
                };
                vec![u8::from(result)]
            }
            Builtin::ClassNew { class_id, .. } => self.new_object(*class_id)?.to_le_bytes().to_vec(),
            Builtin::ClassDowncast { target, .. } => {
                let object = self.pointer(&args[0])?;
                self.downcast(object, *target)?.to_le_bytes().to_vec()
            }
            Builtin::BoxNew { elem, .. } => {
                let value = self.operand(&args[0], Some(*elem))?;
                let layout = self.layout(*elem);
                let block = self.mem.alloc(layout.size.max(1), layout.align)?;
                self.mem.write(block, &value)?;
                block.to_le_bytes().to_vec()
            }
            Builtin::SharedNew { elem, shared } => {
                let value = self.operand(&args[0], Some(*elem))?;
                let TyKind::Struct(id) = *self.types.kind(*shared) else { return fault("a Shared that is no struct") };
                self.new_shared(id, *elem, &value)?.to_le_bytes().to_vec()
            }
            Builtin::WeakNew { .. } => {
                let object = self.pointer(&args[0])?;
                self.weak_retain(object)?;
                object.to_le_bytes().to_vec()
            }
            Builtin::WeakUpgrade { .. } => {
                let weak = self.operand(&args[0], None)?;
                let weak = if self.is_ref(self.operand_ty(body, &args[0])) { self.mem.read(u64_of(&weak), 8)?.to_vec() } else { weak };
                self.weak_upgrade(u64_of(&weak))?.to_le_bytes().to_vec()
            }
            Builtin::WeakEmpty { weak } => vec![0; self.size(*weak) as usize],
            Builtin::MemReplace { elem } => {
                let target = self.pointer(&args[0])?;
                let value = self.operand(&args[1], Some(*elem))?;
                let old = self.mem.read(target, self.size(*elem))?.to_vec();
                self.mem.write(target, &value)?;
                old
            }
            Builtin::MemSwap { elem } => {
                let (a, b) = (self.pointer(&args[0])?, self.pointer(&args[1])?);
                let size = self.size(*elem);
                let x = self.mem.read(a, size)?.to_vec();
                let y = self.mem.read(b, size)?.to_vec();
                self.mem.write(a, &y)?;
                self.mem.write(b, &x)?;
                return Ok(None);
            }
            Builtin::MemForget { .. } => {
                let _ = self.operand(&args[0], None)?;
                return Ok(None);
            }
            Builtin::SizeOf => self.int_result(dest_ty, self.size(arg_ty) as i128),
            Builtin::TypeIsSend => vec![u8::from(self.types.is_send(arg_ty))],
            Builtin::TypeIsSync => vec![u8::from(self.types.is_sync(arg_ty))],
            Builtin::AlignOf => self.int_result(dest_ty, self.layout(arg_ty).align as i128),
            Builtin::MemAlloc => {
                let n = self.word(&args[0])?;
                let elem = self.element_of(arg_ty);
                let layout = self.layout(elem);
                self.mem.alloc((n * layout.size).max(1), layout.align)?.to_le_bytes().to_vec()
            }
            Builtin::MemFree => {
                let p = self.pointer(&args[0])?;
                if p != 0 {
                    self.mem.free(p)?;
                }
                return Ok(None);
            }
            Builtin::PtrRead => {
                let p = self.pointer(&args[0])?;
                let i = self.word(&args[1])?;
                let elem = self.element_of(arg_ty);
                let size = self.size(elem);
                self.mem.read(p + i * size, size)?.to_vec()
            }
            Builtin::PtrWrite => {
                let p = self.pointer(&args[0])?;
                let i = self.word(&args[1])?;
                let elem = self.element_of(arg_ty);
                let value = self.operand(&args[2], Some(elem))?;
                let size = self.size(elem);
                self.mem.write(p + i * size, &value)?;
                return Ok(None);
            }
            Builtin::PtrIsNull => vec![u8::from(self.pointer(&args[0])? == 0)],
            Builtin::PtrAdd => {
                let p = self.pointer(&args[0])?;
                let i = self.signed(&args[1])?;
                let size = self.size(self.element_of(arg_ty));
                (p as i64 + i * size as i64).to_le_bytes().to_vec()
            }
            Builtin::RangeCount => {
                let kind = self.int_kind(arg_ty).ok_or_else(|| Stop::Fault("a range of no integers".into()))?;
                let start = self.operand(&args[0], Some(arg_ty))?;
                let stop = self.operand(&args[1], Some(arg_ty))?;
                let step = self.operand(&args[2], Some(arg_ty))?;
                let count = range_count(kind, &start, &stop, &step);
                self.int_result(dest_ty, count as i128)
            }
            Builtin::RangeNth => {
                let kind = self.int_kind(arg_ty).ok_or_else(|| Stop::Fault("a range of no integers".into()))?;
                let start = self.operand(&args[0], Some(arg_ty))?;
                let step = self.operand(&args[1], Some(arg_ty))?;
                let k = self.word(&args[2])? as i128;
                let value = if kind.signed {
                    kind.signed_value(&start).wrapping_add(kind.signed_value(&step).wrapping_mul(k))
                } else {
                    (kind.unsigned(&start).wrapping_add(kind.unsigned(&step).wrapping_mul(k as u128))) as i128
                };
                kind.encode(value as u128)
            }
            Builtin::FloatAbs => {
                let bits = self.float_bits(arg_ty).unwrap_or(64);
                let v = self.operand(&args[0], Some(arg_ty))?;
                let v = self.float_value(arg_ty, &v);
                self.float_bytes(bits, v.abs())
            }
            Builtin::FloatPow => {
                let bits = self.float_bits(arg_ty).unwrap_or(64);
                let a = self.operand(&args[0], Some(arg_ty))?;
                let b = self.operand(&args[1], Some(arg_ty))?;
                let (a, b) = (self.float_value(arg_ty, &a), self.float_value(arg_ty, &b));
                let result = if bits == 32 { (a as f32).powf(b as f32) as f64 } else { a.powf(b) };
                self.float_bytes(bits, result)
            }
            Builtin::FloatLib(f) => self.float_lib(*f, arg_ty, args)?,
            Builtin::IntBits(bits) => {
                let kind = self.int_kind(arg_ty).ok_or_else(|| Stop::Fault("bits of no integer".into()))?;
                let value = self.operand(&args[0], Some(arg_ty))?;
                let x = kind.unsigned(&value);
                let width = kind.bits.min(128);
                let result = match bits {
                    IntBits::CountOnes => x.count_ones(),
                    IntBits::LeadingZeros => if x == 0 { width } else { x.leading_zeros() - (128 - width) },
                    IntBits::TrailingZeros => if x == 0 { width } else { x.trailing_zeros() },
                };
                self.int_result(dest_ty, result as i128)
            }
            Builtin::ArrayClear => {
                let vec = self.pointer(&args[0])?;
                let elem = self.element_of(arg_ty);
                self.drop_elements(vec, elem, 0)?;
                self.mem.write_u64(vec + 8, 0)?;
                return Ok(None);
            }
            Builtin::ArrayTruncate => {
                let vec = self.pointer(&args[0])?;
                let n = self.word(&args[1])?;
                let len = self.mem.read_u64(vec + 8)?;
                if n < len {
                    let elem = self.element_of(arg_ty);
                    self.drop_elements(vec, elem, n)?;
                    self.mem.write_u64(vec + 8, n)?;
                }
                return Ok(None);
            }
            Builtin::ArrayPop { option } => {
                let vec = self.pointer(&args[0])?;
                let TyKind::Enum(id) = *self.types.kind(*option) else { return fault("pop returns no Option") };
                let def = self.types.enum_def(id);
                let some = def.variants[1].fields[0].ty;
                let len = self.mem.read_u64(vec + 8)?;
                let size = self.size(some);
                if let Some(niche) = self.types.option_niche(id) {
                    if len == 0 {
                        return Ok(Some(self.niche_none(niche)?));
                    }
                    let ptr = self.mem.read_u64(vec)?;
                    self.mem.write_u64(vec + 8, len - 1)?;
                    self.mem.read(ptr + (len - 1) * size, size)?.to_vec()
                } else {
                    let mut bytes = vec![0u8; self.size(*option) as usize];
                    let repr = self.int_kind(def.repr).expect("a repr");
                    if len == 0 {
                        bytes[..repr.bytes()].copy_from_slice(&repr.encode(def.variants[0].discriminant as u128));
                        return Ok(Some(bytes));
                    }
                    let ptr = self.mem.read_u64(vec)?;
                    self.mem.write_u64(vec + 8, len - 1)?;
                    let value = self.mem.read(ptr + (len - 1) * size, size)?.to_vec();
                    let offset = self.types.variant_field_offsets(id)[1][0] as usize;
                    bytes[offset..offset + value.len()].copy_from_slice(&value);
                    bytes[..repr.bytes()].copy_from_slice(&repr.encode(def.variants[1].discriminant as u128));
                    bytes
                }
            }
            Builtin::ArrayRemove | Builtin::ArraySwapRemove => {
                let vec = self.pointer(&args[0])?;
                let i = self.word(&args[1])?;
                let elem = self.element_of(arg_ty);
                let size = self.size(elem);
                let (ptr, len) = (self.mem.read_u64(vec)?, self.mem.read_u64(vec + 8)?);
                let at = ptr + i * size;
                let value = self.mem.read(at, size)?.to_vec();
                if matches!(which, Builtin::ArrayRemove) {
                    self.mem.copy(at + size, at, (len - i - 1) * size)?;
                } else {
                    self.mem.copy(ptr + (len - 1) * size, at, size)?;
                }
                self.mem.write_u64(vec + 8, len - 1)?;
                value
            }
            Builtin::ArrayDrain => {
                let vec = self.pointer(&args[0])?;
                let lo = self.word(&args[1])?;
                let hi = self.word(&args[2])?;
                if lo == hi {
                    return Ok(Some(vec![0; 24]));
                }
                let elem = self.element_of(arg_ty);
                let (size, align) = (self.size(elem), self.vec_align(elem));
                let (ptr, len) = (self.mem.read_u64(vec)?, self.mem.read_u64(vec + 8)?);
                let at = ptr + lo * size;
                let drained = self.vec_from_elems(size, at, hi - lo, align)?;
                self.mem.copy(at + (hi - lo) * size, at, (len - hi) * size)?;
                self.mem.write_u64(vec + 8, len - (hi - lo))?;
                drained
            }
            Builtin::ArrayInsert => {
                let vec = self.pointer(&args[0])?;
                let elem = self.element_of(arg_ty);
                let value = self.operand(&args[1], Some(elem))?;
                let index = self.word(&args[2])?;
                let (size, align) = (self.size(elem), self.vec_align(elem));
                let len = self.mem.read_u64(vec + 8)?;
                self.vec_reserve(vec, size, len + 1, align)?;
                let ptr = self.mem.read_u64(vec)?;
                self.mem.copy(ptr + index * size, ptr + (index + 1) * size, (len - index) * size)?;
                self.mem.write(ptr + index * size, &value)?;
                self.mem.write_u64(vec + 8, len + 1)?;
                return Ok(None);
            }
            Builtin::ArrayExtend => {
                let vec = self.pointer(&args[0])?;
                let elem = self.element_of(arg_ty);
                let source = self.operand(&args[1], None)?;
                let (elems, count) = (u64_of(&source[..8]), u64_of(&source[8..16]));
                let (size, align) = (self.size(elem), self.vec_align(elem));
                self.vec_reserve_more(vec, size, count, align)?;
                let (ptr, len) = (self.mem.read_u64(vec)?, self.mem.read_u64(vec + 8)?);
                let to = ptr + len * size;
                self.mem.copy(elems, to, count * size)?;
                for i in 0..count {
                    self.clone_in_place(to + i * size, elems + i * size, elem)?;
                }
                self.mem.write_u64(vec + 8, len + count)?;
                return Ok(None);
            }
            Builtin::ArrayReserve => {
                let vec = self.pointer(&args[0])?;
                let n = self.word(&args[1])?;
                let elem = self.element_of(arg_ty);
                let (size, align) = (self.size(elem), self.vec_align(elem));
                self.vec_reserve_more(vec, size, n, align)?;
                return Ok(None);
            }
            Builtin::ArrayReserveHint { .. } => return Ok(None),
            Builtin::ArraySwap => {
                let vec = self.pointer(&args[0])?;
                let (i, j) = (self.word(&args[1])?, self.word(&args[2])?);
                let size = self.size(self.element_of(arg_ty));
                let ptr = self.mem.read_u64(vec)?;
                if i != j {
                    let x = self.mem.read(ptr + i * size, size)?.to_vec();
                    let y = self.mem.read(ptr + j * size, size)?.to_vec();
                    self.mem.write(ptr + i * size, &y)?;
                    self.mem.write(ptr + j * size, &x)?;
                }
                return Ok(None);
            }
            Builtin::ArrayReverse => {
                let vec = self.pointer(&args[0])?;
                let size = self.size(self.element_of(arg_ty));
                let (ptr, len) = (self.mem.read_u64(vec)?, self.mem.read_u64(vec + 8)?);
                let mut elements: Vec<Vec<u8>> = (0..len).map(|i| self.mem.read(ptr + i * size, size).map(<[u8]>::to_vec)).collect::<Result<_, _>>()?;
                elements.reverse();
                for (i, e) in elements.iter().enumerate() {
                    self.mem.write(ptr + i as u64 * size, e)?;
                }
                return Ok(None);
            }
            Builtin::ArraySort => {
                let vec = self.pointer(&args[0])?;
                let elem = self.element_of(arg_ty);
                let (ptr, len) = (self.mem.read_u64(vec)?, self.mem.read_u64(vec + 8)?);
                self.sort_elements(ptr, len, elem)?;
                return Ok(None);
            }
            Builtin::ArraySorted { elem } => {
                let source = self.operand(&args[0], None)?;
                let (elems, count) = (u64_of(&source[..8]), u64_of(&source[8..16]));
                let (size, align) = (self.size(*elem), self.vec_align(*elem));
                let sorted = self.vec_from_elems(size, elems, count, align)?;
                let ptr = u64_of(&sorted[..8]);
                self.sort_elements(ptr, count, *elem)?;
                sorted
            }
            Builtin::ArrayClone { elem } => {
                let source = self.operand(&args[0], None)?;
                let (elems, count) = (u64_of(&source[..8]), u64_of(&source[8..16]));
                let (size, align) = (self.size(*elem), self.vec_align(*elem));
                let copy = self.vec_from_elems(size, elems, count, align)?;
                let ptr = u64_of(&copy[..8]);
                for i in 0..count {
                    self.clone_in_place(ptr + i * size, elems + i * size, *elem)?;
                }
                copy
            }
            Builtin::CloneParts { ty } => {
                let source = self.pointer(&args[0])?;
                let loc = self.place(dest)?;
                let size = self.size(*ty);
                self.mem.copy(source, loc.addr, size)?;
                self.clone_parts(loc.addr, source, *ty)?;
                return Ok(None);
            }
            Builtin::LazyStatic { name, init, ty } => self.lazy_static(name.as_str(), init.as_str(), *ty)?.to_le_bytes().to_vec(),
            Builtin::ConstStatic { name, ty } => self.const_static(name.as_str(), *ty)?.to_le_bytes().to_vec(),
            Builtin::Runtime(function) => return self.runtime_fn(*function, args, dest_ty, span).map(Some),
            Builtin::ProcessArgs => self.process_args()?,
            Builtin::Input => {
                let prompt = self.text_operand(&args[0])?;
                self.input(&prompt, span)?
            }
            Builtin::HotCheckpoint => return Ok(None),
            Builtin::ProcessKey => {
                let which = self.signed(&args[0])?;
                let key = self.system.process_key(which);
                self.int_result(dest_ty, key as i128)
            }
            Builtin::MaybeUninitUninit { .. } => vec![0; self.size(arg_ty) as usize],
            Builtin::UnsafeCellGet { .. } => self.pointer(&args[0])?.to_le_bytes().to_vec(),
            Builtin::RangeContains(id) => {
                let def = self.types.range_def(*id).clone();
                let value = self.operand(&args[0], Some(def.repr))?;
                vec![u8::from(self.range_contains(&def, &value))]
            }
            Builtin::RangeClamped(id) => {
                let def = self.types.range_def(*id).clone();
                let value = self.operand(&args[0], Some(def.repr))?;
                self.range_clamp(&def, &value)
            }
            Builtin::RangeNewUnchecked(_) => self.operand(&args[0], None)?,
            // `[RFL-2]`, `[RT-3]` — the description the object's own class keeps.
            Builtin::ClassTypeInfo { .. } => {
                let object = self.pointer(&args[0])?;
                let class = self.object_class(object)?;
                let Some((holder, ty)) = self.types.class_reflection(class) else {
                    return fault(format!("`{}` keeps no description", self.types.class_def(class).name));
                };
                self.const_static(holder.as_str(), ty)?.to_le_bytes().to_vec()
            }
            // `[STD-18]` — `format_to`'s text over the caller's buffer, which never grows: an
            // append that does not fit is refused and remembered.
            Builtin::FmtSinkOpen => {
                let view = self.operand(&args[0], None)?;
                let (ptr, len) = (u64_of(&view[..8]), u64_of(&view[8..16]));
                if self.system.sinks.len() == 64 {
                    return self.panic_in("<format_to>", "format_to: more than 64 calls inside one another (a `Display` that calls `format_to` without end?)");
                }
                // An empty buffer still needs an address no other text has.
                let base = if len == 0 { self.mem.alloc(1, 1)? } else { ptr };
                self.system.sinks.push((base, false));
                let mut value = two_words(base, 0);
                value.extend_from_slice(&len.to_le_bytes());
                value
            }
            Builtin::FmtSinkClose => {
                let sink = self.pointer(&args[0])?;
                let len = self.mem.read_u64(sink + 8)?;
                let (_, refused) = self.system.sinks.pop().unwrap_or((0, false));
                self.int_result(dest_ty, if refused { u64::MAX as i128 } else { len as i128 })
            }
            other => return self.system_builtin(other, arg_ty, args, dest, span),
        }))
    }

    fn element_of_ref(&self, ty: Ty) -> Ty {
        match self.types.kind(ty) {
            TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. } => *inner,
            _ => ty,
        }
    }

    fn is_ref(&self, ty: Option<Ty>) -> bool {
        ty.is_some_and(|ty| matches!(self.types.kind(ty), TyKind::Ref { .. } | TyKind::Ptr { .. }))
    }

    pub(crate) fn is_text_ty(&self, ty: Ty) -> bool {
        matches!(self.types.kind(ty), TyKind::Str | TyKind::Vec { text: true, .. })
    }

    /// `ember_string_boundary`.
    fn string_boundary(&mut self, vec: u64, at: u64, span: Span) -> R<()> {
        let (ptr, len) = (self.mem.read_u64(vec)?, self.mem.read_u64(vec + 8)?);
        let ok = at == 0 || at == len || (at < len && (self.mem.read(ptr + at, 1)?[0] & 0xC0) != 0x80);
        if !ok {
            return self.panic("String byte offset is not on a character boundary", span);
        }
        Ok(())
    }

    /// The elements of a list from `from` on, dropped.
    fn drop_elements(&mut self, vec: u64, elem: Ty, from: u64) -> R<()> {
        if !self.types.needs_drop(elem) {
            return Ok(());
        }
        let (ptr, len) = (self.mem.read_u64(vec)?, self.mem.read_u64(vec + 8)?);
        let size = self.size(elem);
        for i in from..len {
            self.drop_at(ptr + i * size, elem)?;
        }
        Ok(())
    }

    /// `print`/`println` and their standard-error twins (ODR-094: a `Box` or `Cell` prints as
    /// its payload).
    fn print(&mut self, which: &Builtin, arg_ty: Ty, operand: &Operand) -> R<()> {
        let body = self.body();
        let ty = self.operand_ty(body, operand).unwrap_or(arg_ty);
        let value = self.operand(operand, Some(ty))?;
        let at = self.temp(&value)?;
        let (unwrapped, printed) = self.unwrap_printed(at, ty)?;
        let mut text = Vec::new();
        self.display_into(&mut text, unwrapped, printed)?;
        self.mem.free(at)?;
        if matches!(which, Builtin::Println | Builtin::EPrintln) {
            text.push(b'\n');
        }
        let stream: &mut dyn std::io::Write = if matches!(which, Builtin::EPrint | Builtin::EPrintln) { self.io.stderr } else { self.io.stdout };
        stream.write_all(&text).map_err(|e| Stop::Fault(e.to_string()))
    }

    /// The payload a `Box`, a `Cell` or a `some` wrapper prints as, repeatedly.
    fn unwrap_printed(&self, mut at: u64, mut ty: Ty) -> R<(u64, Ty)> {
        while let Some(inner) = self.types.printed_as(ty) {
            let TyKind::Struct(id) = *self.types.kind(ty) else { break };
            at = match self.types.compiler_box_inner(id) {
                Some(_) => self.mem.read_u64(at)?,
                None => at + self.layout(ty).field_offsets.first().copied().unwrap_or(0),
            };
            ty = inner;
        }
        Ok((at, ty))
    }

    /// `[TYP-37]` — `Ord`'s `<`: totalOrder on floats, bytes on text, `<` otherwise.
    fn total_less(&mut self, ty: Ty, a: &[u8], b: &[u8]) -> R<bool> {
        if let Some(_) = self.float_bits(ty) {
            let (x, y) = (self.float_value(ty, a), self.float_value(ty, b));
            return Ok(x.total_cmp(&y) == std::cmp::Ordering::Less);
        }
        if self.is_text_ty(ty) {
            let (x, y) = (self.str_bytes(a)?, self.str_bytes(b)?);
            return Ok(x < y);
        }
        match self.int_kind(ty) {
            Some(kind) if kind.bits <= 128 => Ok(crate::scalar::int_compare(ember_mir::BinOp::Lt, kind, a, b)),
            _ => Ok(crate::memory::uint(a) < crate::memory::uint(b)),
        }
    }

    /// `[STD-15]` — a stable sort by `Ord`'s order (equal elements that are the same value cannot
    /// show their order, so the stable sort serves every type).
    fn sort_elements(&mut self, ptr: u64, len: u64, elem: Ty) -> R<()> {
        let size = self.size(elem);
        if size == 0 || len < 2 {
            return Ok(());
        }
        let mut elements: Vec<Vec<u8>> = (0..len).map(|i| self.mem.read(ptr + i * size, size).map(<[u8]>::to_vec)).collect::<Result<_, _>>()?;
        let mut failure = None;
        // A merge sort by `less`, which is a strict total order.
        let mut sorted = Vec::with_capacity(elements.len());
        merge_sort(&mut elements, &mut |a, b| match self.total_less(elem, a, b) {
            Ok(less) => less,
            Err(stop) => {
                failure.get_or_insert(stop);
                false
            }
        });
        if let Some(stop) = failure {
            return Err(stop);
        }
        sorted.extend(elements);
        for (i, e) in sorted.iter().enumerate() {
            self.mem.write(ptr + i as u64 * size, e)?;
        }
        Ok(())
    }

    /// `clone_part_lines`: the element copied bitwise from `source` to `dest` made a clone:
    /// a handle retained, a list cloned, a written `clone` called, a tuple or enum part by part.
    pub(crate) fn clone_in_place(&mut self, dest: u64, source: u64, ty: Ty) -> R<()> {
        if self.types.is_copy(ty) {
            return self.retain_at(dest, ty);
        }
        if let TyKind::Vec { elem, .. } = *self.types.kind(ty) {
            let (elems, count) = (self.mem.read_u64(source)?, self.mem.read_u64(source + 8)?);
            let (size, align) = (self.size(elem), self.vec_align(elem));
            let copy = self.vec_from_elems(size, elems, count, align)?;
            let ptr = u64_of(&copy[..8]);
            for i in 0..count {
                self.clone_in_place(ptr + i * size, elems + i * size, elem)?;
            }
            return Ok(self.mem.write(dest, &copy)?);
        }
        if let Some((symbol, by_address)) = self.clone_fn(ty) {
            let argument = if by_address { source.to_le_bytes().to_vec() } else { self.mem.read(source, self.size(ty))?.to_vec() };
            let value = self.invoke_symbol(&symbol, vec![argument])?;
            return Ok(self.mem.write(dest, &value)?);
        }
        if matches!(self.types.kind(ty), TyKind::Tuple(_) | TyKind::Enum(_))
            || matches!(*self.types.kind(ty), TyKind::Struct(id) if self.types.compiler_box_inner(id).is_some())
        {
            return self.clone_parts(dest, source, ty);
        }
        fault(format!("a clone of `{}`", self.types.display(ty)))
    }

    /// `emit_clone_parts_fns`: a tuple or an enum cloned part by part into `dest`, which holds a
    /// bitwise copy of `source`.
    fn clone_parts(&mut self, dest: u64, source: u64, ty: Ty) -> R<()> {
        match self.types.kind(ty).clone() {
            TyKind::Tuple(items) => {
                let offsets = self.layout(ty).field_offsets.clone();
                for (i, item) in items.into_iter().enumerate() {
                    self.clone_in_place(dest + offsets[i], source + offsets[i], item)?;
                }
                Ok(())
            }
            TyKind::Enum(id) => {
                let def = self.types.enum_def(id);
                let tag = self.discriminant(Loc::at(source, ty))?;
                let Some(variant) = def.variants.iter().position(|v| v.discriminant == tag) else { return Ok(()) };
                let fields: Vec<Ty> = def.variants[variant].fields.iter().map(|f| f.ty).collect();
                let niche = self.types.option_niche(id).is_some();
                let offsets = if niche { Vec::new() } else { self.types.variant_field_offsets(id).swap_remove(variant) };
                for (i, field) in fields.into_iter().enumerate() {
                    let off = if niche { 0 } else { offsets[i] };
                    self.clone_in_place(dest + off, source + off, field)?;
                }
                Ok(())
            }
            // ODR-237 — a new box holding a clone of the value; a `Box[dyn I]`'s through its
            // table's concrete type (a class handle's copy is the handle, retained).
            TyKind::Struct(id) if self.types.compiler_box_inner(id).is_some() => {
                let inner = self.types.compiler_box_inner(id).expect("a box");
                let (data, concrete) = if matches!(self.types.kind(inner), TyKind::Dyn { .. }) {
                    let table = self.mem.read_u64(source + 8)?;
                    let Some((Region::Table, index)) = memory::decode(table) else { return fault("a dynamic box with no table") };
                    (self.mem.read_u64(source)?, self.calls.tables[index].concrete)
                } else {
                    (self.mem.read_u64(source)?, inner)
                };
                if matches!(self.types.kind(concrete), TyKind::Class(_)) {
                    return self.clone_in_place(dest, source, concrete);
                }
                let layout = self.layout(concrete);
                let block = self.mem.alloc(layout.size.max(1), layout.align)?;
                self.mem.copy(data, block, layout.size)?;
                self.mem.write(dest, &block.to_le_bytes())?;
                self.clone_in_place(block, data, concrete)
            }
            _ => fault("CloneParts of no tuple, enum or box"),
        }
    }

    /// A type's written `clone(self) -> Self`: its symbol and whether it takes its receiver's
    /// address.
    fn clone_fn(&self, ty: Ty) -> Option<(String, bool)> {
        self.bodies.iter().find_map(|body| {
            if body.arg_count != 1 || body.name != "clone" || body.locals.get(1).is_none_or(|l| l.name.as_deref() != Some("self")) {
                return None;
            }
            let receiver = body.locals[1].ty;
            let (owner, by_address) = match self.types.kind(receiver) {
                TyKind::Ref { inner, .. } => (*inner, true),
                _ => (receiver, false),
            };
            (owner == ty && body.return_ty() == owner).then(|| (body.symbol.clone(), by_address))
        })
    }

    /// D-187 — `==` on aggregates: IEEE on floats, identity on handles, bytes on text, field-wise
    /// on the rest (`eq_expr`, `emit_eq_fns`).
    pub(crate) fn equal(&mut self, a: u64, b: u64, ty: Ty) -> R<bool> {
        if self.is_void(ty) {
            return Ok(true);
        }
        if self.is_text_ty(ty) {
            let x = self.mem.read(a, 16)?.to_vec();
            let y = self.mem.read(b, 16)?.to_vec();
            return Ok(self.str_bytes(&x)? == self.str_bytes(&y)?);
        }
        if let Some(_) = self.float_bits(ty) {
            let size = self.size(ty);
            let x = self.mem.read(a, size)?.to_vec();
            let y = self.mem.read(b, size)?.to_vec();
            return Ok(self.float_value(ty, &x) == self.float_value(ty, &y));
        }
        match self.types.kind(ty).clone() {
            TyKind::Fn { abi: None, .. } => Ok(self.mem.read(a + 8, 32)? == self.mem.read(b + 8, 32)?),
            // IX.1 (ODR-241) — two boxes compare what they hold; two `Box[dyn I]` of one kind (one
            // table) through the comparing function of the type the table was made for, of two
            // kinds never.
            TyKind::Struct(id) if self.types.compiler_box_inner(id).is_some() => {
                let inner = self.types.compiler_box_inner(id).expect("a box");
                if !matches!(self.types.kind(inner), TyKind::Dyn { .. }) {
                    let (x, y) = (self.mem.read_u64(a)?, self.mem.read_u64(b)?);
                    return self.equal(x, y, inner);
                }
                let (data_a, table_a) = (self.mem.read_u64(a)?, self.mem.read_u64(a + 8)?);
                let (data_b, table_b) = (self.mem.read_u64(b)?, self.mem.read_u64(b + 8)?);
                if table_a != table_b {
                    return Ok(false);
                }
                let Some((memory::Region::Table, index)) = memory::decode(table_a) else {
                    return fault("a `Box[dyn I]` with no table");
                };
                let concrete = self.calls.tables[index].concrete;
                let Some(symbol) = self.types.dyn_eq_symbol(concrete).map(str::to_string) else {
                    return fault("a table compared with no comparing function");
                };
                let Some(&body) = self.by_symbol.get(symbol.as_str()) else {
                    return fault(format!("no body `{symbol}`"));
                };
                let class = matches!(self.types.kind(concrete), TyKind::Class(_));
                let mut args = Vec::new();
                for (local, data) in [(1, data_a), (2, data_b)] {
                    let param = self.bodies[body].locals[local].ty;
                    args.push(if class || matches!(self.types.kind(param), TyKind::Ref { .. }) {
                        data.to_le_bytes().to_vec()
                    } else {
                        let size = self.size(concrete);
                        self.mem.read(data, size)?.to_vec()
                    });
                }
                let result = self.invoke(body, args)?;
                Ok(result.first().is_some_and(|&byte| byte != 0))
            }
            TyKind::Struct(id) => {
                let fields: Vec<Ty> = self.types.struct_def(id).fields.iter().map(|f| f.ty).collect();
                let offsets = self.layout(ty).field_offsets.clone();
                for (i, field) in fields.into_iter().enumerate() {
                    if !self.equal(a + offsets[i], b + offsets[i], field)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            TyKind::Tuple(items) => {
                let offsets = self.layout(ty).field_offsets.clone();
                for (i, item) in items.into_iter().enumerate() {
                    if !self.equal(a + offsets[i], b + offsets[i], item)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            TyKind::Array { elem, len } => {
                let step = self.size(elem);
                for i in 0..len {
                    if !self.equal(a + i * step, b + i * step, elem)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            TyKind::Vec { elem, .. } => {
                let (pa, la) = (self.mem.read_u64(a)?, self.mem.read_u64(a + 8)?);
                let (pb, lb) = (self.mem.read_u64(b)?, self.mem.read_u64(b + 8)?);
                if la != lb {
                    return Ok(false);
                }
                let step = self.size(elem);
                for i in 0..la {
                    if !self.equal(pa + i * step, pb + i * step, elem)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            TyKind::Enum(id) if !self.types.enum_is_bare(id) => {
                let (ta, tb) = (self.discriminant(Loc::at(a, ty))?, self.discriminant(Loc::at(b, ty))?);
                if ta != tb {
                    return Ok(false);
                }
                let def = self.types.enum_def(id);
                let Some(variant) = def.variants.iter().position(|v| v.discriminant == ta) else { return Ok(true) };
                let fields: Vec<Ty> = def.variants[variant].fields.iter().map(|f| f.ty).collect();
                let niche = self.types.option_niche(id).is_some();
                let offsets = if niche { Vec::new() } else { self.types.variant_field_offsets(id).swap_remove(variant) };
                for (i, field) in fields.into_iter().enumerate() {
                    let off = if niche { 0 } else { offsets[i] };
                    if !self.equal(a + off, b + off, field)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            _ => {
                let size = self.size(ty);
                Ok(self.mem.read(a, size)? == self.mem.read(b, size)?)
            }
        }
    }

    fn float_lib(&mut self, f: FloatLib, ty: Ty, args: &[Operand]) -> R<Vec<u8>> {
        let bits = self.float_bits(ty).unwrap_or(64);
        let mut values = Vec::new();
        for arg in args {
            let bytes = self.operand(arg, Some(ty))?;
            values.push(self.float_value(ty, &bytes));
        }
        let x = values[0];
        if f.is_predicate() {
            return Ok(vec![u8::from(match f {
                FloatLib::IsNan => x.is_nan(),
                FloatLib::IsFinite => x.is_finite(),
                _ => x.is_infinite(),
            })]);
        }
        let result = if bits == 32 {
            let x = x as f32;
            let y = values.get(1).map_or(0.0, |v| *v as f32);
            let z = values.get(2).map_or(0.0, |v| *v as f32);
            (match f {
                FloatLib::Sqrt => x.sqrt(),
                FloatLib::Cbrt => x.cbrt(),
                FloatLib::Exp => x.exp(),
                FloatLib::Exp2 => x.exp2(),
                FloatLib::Ln => x.ln(),
                FloatLib::Log2 => x.log2(),
                FloatLib::Log10 => x.log10(),
                FloatLib::Sin => x.sin(),
                FloatLib::Cos => x.cos(),
                FloatLib::Tan => x.tan(),
                FloatLib::Asin => x.asin(),
                FloatLib::Acos => x.acos(),
                FloatLib::Atan => x.atan(),
                FloatLib::Sinh => x.sinh(),
                FloatLib::Cosh => x.cosh(),
                FloatLib::Tanh => x.tanh(),
                FloatLib::Floor => x.floor(),
                FloatLib::Ceil => x.ceil(),
                FloatLib::Trunc => x.trunc(),
                FloatLib::Round => x.round_ties_even(),
                FloatLib::RoundHalfAway => x.round(),
                FloatLib::Atan2 => x.atan2(y),
                FloatLib::Hypot => x.hypot(y),
                FloatLib::Pow => x.powf(y),
                FloatLib::Copysign => x.copysign(y),
                FloatLib::MulAdd => x.mul_add(y, z),
                _ => unreachable!("a predicate"),
            }) as f64
        } else {
            let y = values.get(1).copied().unwrap_or(0.0);
            let z = values.get(2).copied().unwrap_or(0.0);
            match f {
                FloatLib::Sqrt => x.sqrt(),
                FloatLib::Cbrt => x.cbrt(),
                FloatLib::Exp => x.exp(),
                FloatLib::Exp2 => x.exp2(),
                FloatLib::Ln => x.ln(),
                FloatLib::Log2 => x.log2(),
                FloatLib::Log10 => x.log10(),
                FloatLib::Sin => x.sin(),
                FloatLib::Cos => x.cos(),
                FloatLib::Tan => x.tan(),
                FloatLib::Asin => x.asin(),
                FloatLib::Acos => x.acos(),
                FloatLib::Atan => x.atan(),
                FloatLib::Sinh => x.sinh(),
                FloatLib::Cosh => x.cosh(),
                FloatLib::Tanh => x.tanh(),
                FloatLib::Floor => x.floor(),
                FloatLib::Ceil => x.ceil(),
                FloatLib::Trunc => x.trunc(),
                FloatLib::Round => x.round_ties_even(),
                FloatLib::RoundHalfAway => x.round(),
                FloatLib::Atan2 => x.atan2(y),
                FloatLib::Hypot => x.hypot(y),
                FloatLib::Pow => x.powf(y),
                FloatLib::Copysign => x.copysign(y),
                FloatLib::MulAdd => x.mul_add(y, z),
                _ => unreachable!("a predicate"),
            }
        };
        Ok(self.float_bytes(bits, result))
    }

    fn range_contains(&self, def: &ember_types::RangeDef, value: &[u8]) -> bool {
        let as_f64 = |bound: ember_types::Bound| match bound {
            ember_types::Bound::Float(v) => v,
            ember_types::Bound::Int(v) => v as f64,
        };
        if let Some(_) = self.float_bits(def.repr) {
            let v = self.float_value(def.repr, value);
            let (lo, hi) = (as_f64(def.lo), as_f64(def.hi));
            return v >= lo && if def.inclusive { v <= hi } else { v < hi };
        }
        let kind = self.int_kind(def.repr).expect("an integer range");
        let v = if kind.signed { kind.signed_value(value) } else { kind.unsigned(value) as i128 };
        let bound = |b: ember_types::Bound| match b {
            ember_types::Bound::Int(v) => v,
            ember_types::Bound::Float(v) => v as i128,
        };
        v >= bound(def.lo) && if def.inclusive { v <= bound(def.hi) } else { v < bound(def.hi) }
    }

    fn range_clamp(&self, def: &ember_types::RangeDef, value: &[u8]) -> Vec<u8> {
        let as_f64 = |bound: ember_types::Bound| match bound {
            ember_types::Bound::Float(v) => v,
            ember_types::Bound::Int(v) => v as f64,
        };
        if let Some(bits) = self.float_bits(def.repr) {
            let v = self.float_value(def.repr, value);
            let (lo, hi) = (as_f64(def.lo), as_f64(def.hi));
            let hi = if def.inclusive { hi } else { next_below(hi, bits) };
            let clamped = if v.is_nan() { lo } else { v.max(lo).min(hi) };
            return self.float_bytes(bits, clamped);
        }
        let kind = self.int_kind(def.repr).expect("an integer range");
        let v = if kind.signed { kind.signed_value(value) } else { kind.unsigned(value) as i128 };
        let bound = |b: ember_types::Bound| match b {
            ember_types::Bound::Int(v) => v,
            ember_types::Bound::Float(v) => v as i128,
        };
        let (lo, hi) = (bound(def.lo), if def.inclusive { bound(def.hi) } else { bound(def.hi) - 1 });
        kind.encode(v.clamp(lo, hi) as u128)
    }

    fn parse_value(&mut self, kind: ParseKind, s: &str, dest: Ty) -> R<Vec<u8>> {
        Ok(match kind {
            ParseKind::Signed | ParseKind::I128 => {
                let value: i128 = s.parse().unwrap_or(0);
                self.int_result(dest, value)
            }
            ParseKind::Unsigned | ParseKind::U128 => {
                let value: u128 = s.trim_start_matches('+').parse().unwrap_or(0);
                let size = self.size(dest) as usize;
                value.to_le_bytes()[..size.min(16)].to_vec()
            }
            ParseKind::F64 => parse_float(s).to_bits().to_le_bytes().to_vec(),
            // Each rounded once, from the text (D-316): through an `f64` it would be twice.
            ParseKind::F32 => s.parse::<f32>().unwrap_or(0.0).to_bits().to_le_bytes().to_vec(),
            ParseKind::F16 => {
                let (negative, magnitude) = match s.strip_prefix('-') {
                    Some(rest) => (true, rest),
                    None => (false, s.strip_prefix('+').unwrap_or(s)),
                };
                let lower = magnitude.to_ascii_lowercase();
                let bits = if lower == "inf" || lower == "infinity" {
                    0x7C00
                } else if lower == "nan" {
                    0x7E00
                } else {
                    ember_types::f16_bits_of_decimal(magnitude)
                };
                (bits | if negative { 0x8000 } else { 0 }).to_le_bytes().to_vec()
            }
            ParseKind::Bool => vec![u8::from(s.len() == 4)],
            ParseKind::Char => (s.chars().next().map_or(0, |c| c as u32)).to_le_bytes().to_vec(),
            ParseKind::I256 | ParseKind::U256 => {
                let kind = IntKind { bits: 256, signed: kind == ParseKind::I256 };
                crate::wide::W256::parse(kind, s.as_bytes()).unwrap_or_default().bytes(32)
            }
        })
    }

    fn system_builtin(&mut self, which: &Builtin, arg_ty: Ty, args: &[Operand], dest: &Place, span: Span) -> R<Option<Vec<u8>>> {
        let _ = (arg_ty, args, dest);
        let what = match which {
            Builtin::ThreadSpawn | Builtin::ThreadJoin | Builtin::ThreadScope | Builtin::ScopeSpawn { .. } | Builtin::ScopedJoin => "a thread",
            Builtin::JobScope | Builtin::JobScopeSubmit { .. } | Builtin::JobSubmit { .. } | Builtin::JobWait | Builtin::ParallelFor { .. } => {
                "a job on the pool"
            }
            Builtin::ForeignStatic { .. } | Builtin::ForeignAdopt | Builtin::ForeignRelease | Builtin::ForeignThread | Builtin::ForeignDestructor { .. } => {
                "a foreign object"
            }
            Builtin::CppObjectAddress | Builtin::CppObjectHandle { .. } | Builtin::CppObjectGive | Builtin::CppSpan { .. } | Builtin::CppPointee => {
                "a C++ object"
            }
            Builtin::RetainedPin | Builtin::RetainedAddress => "a pinned object's address",
            // `[JOB-5]` — the evaluator runs no job (one is refused above), so every call is outside one.
            Builtin::JobLocalArena { .. } => {
                return self.panic("`jobs.local_arena()` is called outside a job: the arena is the running job's [JOB-5]", span);
            }
            _ => return fault(format!("the builtin `{}` is not run by the evaluator yet", which.name())),
        };
        Err(Stop::Refused { what: what.to_string(), span })
    }
}

/// `ember_str_search`: the first `needle` at or after `from`.
fn search(s: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if needle.is_empty() {
        return (from <= s.len()).then_some(from);
    }
    (from..=s.len().saturating_sub(needle.len())).find(|&at| at + needle.len() <= s.len() && &s[at..at + needle.len()] == needle)
}

fn rsearch(s: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.len() > s.len() {
        return None;
    }
    (0..=s.len() - needle.len()).rev().find(|&at| &s[at..at + needle.len()] == needle)
}

/// Python's `s.replace("", to)`: `to` between every character and at both ends.
fn python_replace_empty(s: &str, to: &str) -> String {
    let mut out = String::from(to);
    for c in s.chars() {
        out.push(c);
        out.push_str(to);
    }
    out
}

fn utf8_width(lead: u8) -> usize {
    if lead < 0x80 { 1 } else if lead < 0xE0 { 2 } else if lead < 0xF0 { 3 } else { 4 }
}

fn decode_char(bytes: &[u8]) -> u32 {
    let b = bytes[0] as u32;
    match bytes.len() {
        1 => b,
        2 => ((b & 0x1F) << 6) | (bytes[1] as u32 & 0x3F),
        3 => ((b & 0x0F) << 12) | ((bytes[1] as u32 & 0x3F) << 6) | (bytes[2] as u32 & 0x3F),
        _ => ((b & 0x07) << 18) | ((bytes[1] as u32 & 0x3F) << 12) | ((bytes[2] as u32 & 0x3F) << 6) | (bytes[3] as u32 & 0x3F),
    }
}

/// `[TXT-10]` (ODR-029) — 0 for a whole literal that fits, 1 empty, 2 invalid, 3 out of range.
fn parse_status(kind: ParseKind, s: &[u8], max: Option<u64>) -> u8 {
    let Ok(text) = std::str::from_utf8(s) else { return 2 };
    if text.is_empty() {
        return 1;
    }
    match kind {
        ParseKind::I256 | ParseKind::U256 => {
            let signed = kind == ParseKind::I256;
            let digits = if signed { text.strip_prefix(['-', '+']) } else { text.strip_prefix('+') }.unwrap_or(text);
            if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                return 2;
            }
            if crate::wide::W256::parse(IntKind { bits: 256, signed }, s).is_some() { 0 } else { 3 }
        }
        ParseKind::Signed | ParseKind::I128 => {
            let digits = text.strip_prefix(['-', '+']).unwrap_or(text);
            if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                return 2;
            }
            match text.parse::<i128>() {
                Ok(v) => {
                    let max = max.map_or(i128::MAX, |m| m as i128);
                    let min = -max - 1;
                    if v > max || v < min { 3 } else { 0 }
                }
                Err(_) => 3,
            }
        }
        ParseKind::Unsigned | ParseKind::U128 => {
            let digits = text.strip_prefix('+').unwrap_or(text);
            if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                return 2;
            }
            match digits.parse::<u128>() {
                Ok(v) => if v > max.map_or(u128::MAX, u128::from) { 3 } else { 0 },
                Err(_) => 3,
            }
        }
        ParseKind::F16 | ParseKind::F32 | ParseKind::F64 => if text.parse::<f64>().is_ok() { 0 } else { 2 },
        ParseKind::Bool => if text == "true" || text == "false" { 0 } else { 2 },
        ParseKind::Char => if text.chars().count() == 1 { 0 } else { 2 },
    }
}

fn parse_float(s: &str) -> f64 {
    s.parse().unwrap_or(0.0)
}

/// `ember_range_count_*`: how many values `start..stop` by `step` holds.
fn range_count(kind: IntKind, start: &[u8], stop: &[u8], step: &[u8]) -> u128 {
    if kind.signed {
        let (a, b, s) = (kind.signed_value(start), kind.signed_value(stop), kind.signed_value(step));
        if s > 0 && a < b {
            ((b - a - 1) / s + 1) as u128
        } else if s < 0 && a > b {
            ((a - b - 1) / -s + 1) as u128
        } else {
            0
        }
    } else {
        let (a, b, s) = (kind.unsigned(start), kind.unsigned(stop), kind.unsigned(step));
        if s > 0 && a < b { (b - a - 1) / s + 1 } else { 0 }
    }
}

/// The greatest float of `bits` below `value`.
fn next_below(value: f64, bits: u32) -> f64 {
    match bits {
        32 => {
            let v = value as f32;
            f32::from_bits(if v > 0.0 { v.to_bits() - 1 } else if v == 0.0 { 0x8000_0001 } else { v.to_bits() + 1 }) as f64
        }
        16 => {
            let h = crate::scalar::f64_to_f16(value);
            let below = if value > 0.0 { h - 1 } else if value == 0.0 { 0x8001 } else { h + 1 };
            crate::scalar::f16_to_f64(below)
        }
        _ => f64::from_bits(if value > 0.0 { value.to_bits() - 1 } else if value == 0.0 { 0x8000_0000_0000_0001 } else { value.to_bits() + 1 }),
    }
}

/// A stable merge sort over `less`.
fn merge_sort(items: &mut Vec<Vec<u8>>, less: &mut dyn FnMut(&[u8], &[u8]) -> bool) {
    if items.len() < 2 {
        return;
    }
    let mut right = items.split_off(items.len() / 2);
    merge_sort(items, less);
    merge_sort(&mut right, less);
    let left = std::mem::take(items);
    let (mut i, mut j) = (0, 0);
    while i < left.len() && j < right.len() {
        if less(&right[j], &left[i]) {
            items.push(right[j].clone());
            j += 1;
        } else {
            items.push(left[i].clone());
            i += 1;
        }
    }
    items.extend_from_slice(&left[i..]);
    items.extend_from_slice(&right[j..]);
}
