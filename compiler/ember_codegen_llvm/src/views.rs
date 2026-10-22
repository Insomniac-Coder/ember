//! `[SPN-3]`, `[SPN-5]`, `[SPN-6]`, `[STR-*]`'s `split_once`, `[DSJ-1]` — the view operations MIR
//! leaves to the backend: a view split in two at a checked boundary, an iterator's element, a
//! chunk, text split at a separator's first match, and whether two views overlap. A pointer
//! stepped by zero stays what it was (null for an empty `String`): `getelementptr` without
//! `inbounds` makes no claim about it.

use crate::func::{FnCx, PlaceRef, Val};
use crate::types::Sc;
use crate::{Result, unsupported};
use ember_mir::{Builtin, Operand, Place};
use ember_types::{Ty, TyKind};

impl<'a> FnCx<'_, 'a> {
    /// A view operand's pointer and length.
    fn view_parts(&mut self, operand: &Operand) -> Result<(String, String)> {
        let (words, align) = self.view_words(operand)?;
        let pointer = self.load(Sc::Ptr, &words, align.min(8));
        let len_at = self.gep(&words, 8);
        let len = self.load(Sc::I(64), &len_at, align.min(8));
        Ok((pointer, len))
    }

    /// The address `index` elements of `size` bytes after `pointer`.
    fn step(&mut self, pointer: &str, index: &str, size: u64) -> String {
        if size == 0 {
            return pointer.to_string();
        }
        let offset = self.value(format!("mul i64 {index}, {size}"));
        self.value(format!("getelementptr i8, ptr {pointer}, i64 {offset}"))
    }

    /// A view `{pointer, len}` written at `at`.
    fn write_view(&mut self, at: &str, align: u64, pointer: &str, len: &str) {
        self.emit(format!("store ptr {pointer}, ptr {at}, align {}", align.min(8)));
        let len_at = self.gep(at, 8);
        self.emit(format!("store i64 {len}, ptr {len_at}, align {}", align.min(8)));
    }

    /// A view builtin, or `None` for another.
    pub(crate) fn view_builtin(&mut self, which: &Builtin, args: &[Operand], dest: &Place, span: ember_span::Span) -> Option<Result<()>> {
        Some(match which {
            // `[SPN-3]` — MIR has checked `boundary <= len`.
            Builtin::SpanSplitAt { elem, pair, .. } => (|| {
                let (pointer, len) = self.view_parts(&args[0])?;
                let boundary = self.word(&args[1])?;
                let (size, _) = self.m.ty.size_align(*elem);
                let tail = self.step(&pointer, &boundary, size);
                let rest = self.value(format!("sub i64 {len}, {boundary}"));
                let Some((at, align)) = self.dest_of(dest)? else { return Ok(()) };
                let offsets = self.m.ty.table.layout(*pair).field_offsets;
                let first = self.gep(&at, offsets[0]);
                self.write_view(&first, align, &pointer, &boundary);
                let second = self.gep(&at, offsets[1]);
                self.write_view(&second, align, &tail, &rest);
                Ok(())
            })(),
            // `[SPN-5]` — MIR has checked the cursor and advanced it.
            Builtin::SpanIterNext { elem, .. } => (|| {
                let (pointer, _) = self.view_parts(&args[0])?;
                let index = self.word(&args[1])?;
                let (size, _) = self.m.ty.size_align(*elem);
                let element = self.step(&pointer, &index, size);
                self.store_result(dest, &element, Sc::Ptr, false)
            })(),
            // `[SPN-6]` — MIR computed the bounded, non-zero advance.
            Builtin::SpanChunksNext { elem, .. } => (|| {
                let (pointer, _) = self.view_parts(&args[0])?;
                let start = self.word(&args[1])?;
                let len = self.word(&args[2])?;
                let (size, _) = self.m.ty.size_align(*elem);
                let chunk = self.step(&pointer, &start, size);
                let Some((at, align)) = self.dest_of(dest)? else { return Ok(()) };
                self.write_view(&at, align, &chunk, &len);
                Ok(())
            })(),
            // `[DSJ-1]` — two comparisons, as integers; an empty view overlaps nothing.
            Builtin::ViewsOverlap { a, b } => (|| {
                let elem = |cx: &Self, view: Ty| match cx.m.ty.kind(view) {
                    TyKind::Span { elem, .. } => Ok(cx.m.ty.size_align(*elem).0),
                    _ => unsupported("`assert_disjoint` of a non-view"),
                };
                let (size_a, size_b) = (elem(self, *a)?, elem(self, *b)?);
                let (x, x_len) = self.view_parts(&args[0])?;
                let (y, y_len) = self.view_parts(&args[1])?;
                let x = self.value(format!("ptrtoint ptr {x} to i64"));
                let y = self.value(format!("ptrtoint ptr {y} to i64"));
                let x_bytes = self.value(format!("mul i64 {x_len}, {size_a}"));
                let y_bytes = self.value(format!("mul i64 {y_len}, {size_b}"));
                let x_end = self.value(format!("add i64 {x}, {x_bytes}"));
                let y_end = self.value(format!("add i64 {y}, {y_bytes}"));
                let x_some = self.value(format!("icmp ne i64 {x_len}, 0"));
                let y_some = self.value(format!("icmp ne i64 {y_len}, 0"));
                let below = self.value(format!("icmp ult i64 {x}, {y_end}"));
                let above = self.value(format!("icmp ult i64 {y}, {x_end}"));
                let both = self.value(format!("and i1 {x_some}, {y_some}"));
                let crossing = self.value(format!("and i1 {below}, {above}"));
                let overlap = self.value(format!("and i1 {both}, {crossing}"));
                self.store_result(dest, &overlap, Sc::Bool, false)
            })(),
            Builtin::StrSplitOnce { none, some } => self.split_once(args, dest, *none, *some),
            // `[STD-10]` — `input` panics at end of input, at its own call.
            Builtin::Input => (|| {
                let prompt = self.operand(&args[0])?;
                let loc = self.loc(span);
                self.call_rt_into("input", vec![prompt, loc], dest)
            })(),
            _ => return None,
        })
    }

    /// `split_once`: `None` without a match, else `Some` of the text before the separator's
    /// first match and after it. Both inputs are read before the result is written: it may hold
    /// one of them.
    fn split_once(&mut self, args: &[Operand], dest: &Place, none: usize, some: usize) -> Result<()> {
        let (text, text_len) = self.view_parts(&args[0])?;
        let (_, needle_len) = self.view_parts(&args[1])?;
        let (text_words, text_align) = self.view_words(&args[0])?;
        let (needle_words, needle_align) = self.view_words(&args[1])?;
        let at = self.alloca(8, 8);
        self.call_rt(
            "str_search",
            vec![Val::Mem(text_words, text_align), Val::Mem(needle_words, needle_align), Val::Imm(Sc::I(64), "0".to_string())],
            Some((&at, 8)),
        );
        let found = self.load(Sc::I(64), &at, 8);
        let ty = self.place_ty(dest);
        let TyKind::Enum(id) = *self.m.ty.kind(ty) else { return unsupported("`split_once` that returns no `Option`") };
        let Some((out, align)) = self.dest_of(dest)? else { return Ok(()) };
        let place = PlaceRef { ptr: out.clone(), ty, variant: None, align };
        let missing = self.value(format!("icmp eq i64 {found}, -1"));
        let (nothing, split, done) = (self.label(), self.label(), self.label());
        self.emit(format!("br i1 {missing}, label %{nothing}, label %{split}"));
        self.start(&nothing);
        self.write_variant(&place, id, none)?;
        self.emit(format!("br label %{done}"));
        self.start(&split);
        let table = self.m.ty.table;
        let pair_at = self.gep(&out, table.variant_field_offsets(id)[some][0]);
        let pair_ty = table.enum_def(id).variants[some].fields[0].ty;
        let offsets = table.layout(pair_ty).field_offsets;
        let after_match = self.value(format!("sub i64 {text_len}, {found}"));
        let tail = self.value(format!("sub i64 {after_match}, {needle_len}"));
        let consumed = self.value(format!("sub i64 {text_len}, {tail}"));
        let rest = self.step(&text, &consumed, 1);
        let before = self.gep(&pair_at, offsets[0]);
        self.write_view(&before, align, &text, &found);
        let after = self.gep(&pair_at, offsets[1]);
        self.write_view(&after, align, &rest, &tail);
        self.write_variant(&place, id, some)?;
        self.emit(format!("br label %{done}"));
        self.start(&done);
        Ok(())
    }
}
