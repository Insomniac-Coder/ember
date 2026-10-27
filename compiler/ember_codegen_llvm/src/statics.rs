//! `[STA-3]` — a lazily initialised static: its one storage, and the accessor that runs its
//! initialiser the first time any thread asks. The runtime holds one lock across a thread's chain
//! of initialisations (`rt_static_begin`), so another thread waits, and a static reached again on
//! its own chain is the initialisation-cycle panic: the C backend's `lazy_static_definitions`.

use crate::func::{FnCx, Val};
use crate::types::{Repr, Sc};
use crate::{Module, Result, escape, global_name, unsupported};
use ember_types::{FloatTy, Ty, TyKind};

/// `[CT-5]` — a value's bytes as its layout places them, and the pointers among them (their
/// offsets and the globals they point at).
struct Image {
    bytes: Vec<u8>,
    pointers: Vec<(usize, String)>,
}

impl Image {
    fn put(&mut self, at: usize, value: u128, size: usize) {
        self.bytes[at..at + size].copy_from_slice(&value.to_le_bytes()[..size]);
    }

    fn pointer(&mut self, at: usize, target: String) {
        self.pointers.push((at, target));
    }

    /// The image as an IR constant: its bytes, with each pointer in its place.
    fn constant(&self) -> (String, String) {
        let mut pointers = self.pointers.clone();
        pointers.sort_by_key(|(at, _)| *at);
        let (mut types, mut values) = (Vec::new(), Vec::new());
        let mut from = 0;
        let bytes = |types: &mut Vec<String>, values: &mut Vec<String>, from: usize, to: usize| {
            if to > from {
                types.push(format!("[{} x i8]", to - from));
                values.push(format!("[{} x i8] c\"{}\"", to - from, escape(&self.bytes[from..to])));
            }
        };
        for (at, target) in &pointers {
            bytes(&mut types, &mut values, from, *at);
            types.push("ptr".to_string());
            values.push(format!("ptr {target}"));
            from = at + 8;
        }
        bytes(&mut types, &mut values, from, self.bytes.len().max(from));
        if types.is_empty() {
            types.push("[1 x i8]".to_string());
            values.push("[1 x i8] zeroinitializer".to_string());
        }
        (format!("<{{ {} }}>", types.join(", ")), format!("<{{ {} }}>", values.join(", ")))
    }
}

/// `[CT-5]` — the constant data the evaluator's text describes for the static `name`: its heap
/// parts (an `Array`'s elements, a `String`'s bytes) constant globals of their own, which nothing
/// grows or frees. Its address.
pub(crate) fn constant_static(m: &mut Module<'_>, name: &str, ty: Ty) -> Result<String> {
    let symbol = ember_branding::item_symbol(&ember_types::item_path(name));
    let ir_name = global_name(&symbol);
    if m.helper_defined(&symbol) {
        return Ok(ir_name);
    }
    let Some((_, _, text)) = m.ty.table.constant_statics().iter().find(|(static_name, _, _)| static_name.as_str() == name).cloned() else {
        return unsupported(format!("the constant static `{name}` without its value"));
    };
    let mut tokens = text.split_ascii_whitespace();
    let (size, align) = m.ty.size_align(ty);
    let mut image = Image { bytes: vec![0; size as usize], pointers: Vec::new() };
    let mut parts = 0;
    place(m, ty, &mut tokens, &mut image, 0, &symbol, &mut parts)?;
    let (ty_text, value) = image.constant();
    m.global(format!("{ir_name} = internal constant {ty_text} {value}, align {}", align.max(1)));
    Ok(ir_name)
}

/// The value of `ty` the tokens describe, placed at `at` in `image`.
fn place<'t>(
    m: &mut Module<'_>,
    ty: Ty,
    tokens: &mut impl Iterator<Item = &'t str>,
    image: &mut Image,
    at: usize,
    symbol: &str,
    parts: &mut usize,
) -> Result<()> {
    let bad = || unsupported("a constant static's value the backend cannot read");
    let table = m.ty.table;
    let token = |tokens: &mut dyn Iterator<Item = &'t str>| tokens.next();
    let open = |tokens: &mut dyn Iterator<Item = &'t str>| tokens.next() == Some("(");
    let bytes_of = |text: &str| -> Option<Vec<u8>> {
        let (len, hex) = text.split_once(':')?;
        let len: usize = len.parse().ok()?;
        (0..len).map(|i| u8::from_str_radix(hex.get(i * 2..i * 2 + 2)?, 16).ok()).collect()
    };
    let width = |m: &Module<'_>, ty: Ty| m.ty.size_align(ty).0 as usize;
    match table.kind(ty).clone() {
        TyKind::Bool => {
            let Some(value) = token(tokens) else { return bad() };
            image.put(at, u128::from(value == "b1"), 1);
        }
        TyKind::Char => {
            let Some(value) = token(tokens).and_then(|t| t.strip_prefix('c')).and_then(|v| v.parse::<u32>().ok()) else { return bad() };
            image.put(at, u128::from(value), 4);
        }
        TyKind::Int(_) | TyKind::Uint(_) => {
            let Some(value) = token(tokens).and_then(|t| t.strip_prefix('i')).and_then(|v| v.parse::<i128>().ok()) else { return bad() };
            image.put(at, value as u128, width(m, ty));
        }
        TyKind::Float(float) => {
            let Some(bits) = token(tokens).and_then(|t| t.strip_prefix('f')).and_then(|v| u64::from_str_radix(v, 16).ok()) else { return bad() };
            let size = match float {
                FloatTy::F16 => 2,
                FloatTy::F32 => 4,
                FloatTy::F64 => 8,
            };
            image.put(at, u128::from(bits), size);
        }
        TyKind::Range(id) => return place(m, table.range_def(id).repr, tokens, image, at, symbol, parts),
        TyKind::Newtype(id) => return place(m, table.newtype_def(id).repr, tokens, image, at, symbol, parts),
        TyKind::Str => {
            let Some(bytes) = token(tokens).and_then(|t| t.strip_prefix('s')).and_then(bytes_of) else { return bad() };
            let text = m.bytes(&bytes);
            image.pointer(at, text);
            image.put(at + 8, bytes.len() as u128, 8);
        }
        TyKind::Vec { text: true, .. } => {
            let Some(bytes) = token(tokens).and_then(|t| t.strip_prefix('s')).and_then(bytes_of) else { return bad() };
            if !bytes.is_empty() {
                let text = m.bytes(&bytes);
                image.pointer(at, text);
                image.put(at + 8, bytes.len() as u128, 8);
                image.put(at + 16, bytes.len() as u128, 8);
            }
        }
        TyKind::Vec { elem, text: false } => {
            let Some(len) = token(tokens).and_then(|t| t.strip_prefix('v')).and_then(|v| v.parse::<usize>().ok()) else { return bad() };
            if !open(tokens) {
                return bad();
            }
            let (elem_size, elem_align) = m.ty.size_align(elem);
            let mut part = Image { bytes: vec![0; elem_size as usize * len], pointers: Vec::new() };
            for index in 0..len {
                place(m, elem, tokens, &mut part, index * elem_size as usize, symbol, parts)?;
            }
            if token(tokens) != Some(")") {
                return bad();
            }
            if len > 0 {
                let name = global_name(&format!("{symbol}.p{parts}"));
                *parts += 1;
                let (ty_text, value) = part.constant();
                m.global(format!("{name} = internal constant {ty_text} {value}, align {}", elem_align.max(1)));
                image.pointer(at, name);
                image.put(at + 8, len as u128, 8);
                image.put(at + 16, len as u128, 8);
            }
        }
        TyKind::Tuple(items) => {
            if !open(tokens) {
                return bad();
            }
            let offsets = table.layout(ty).field_offsets;
            for (item, offset) in items.iter().zip(offsets) {
                place(m, *item, tokens, image, at + offset as usize, symbol, parts)?;
            }
            if token(tokens) != Some(")") {
                return bad();
            }
        }
        TyKind::Array { elem, len } => {
            if !open(tokens) {
                return bad();
            }
            let size = width(m, elem);
            for index in 0..len as usize {
                place(m, elem, tokens, image, at + index * size, symbol, parts)?;
            }
            if token(tokens) != Some(")") {
                return bad();
            }
        }
        TyKind::Struct(id) => {
            if !open(tokens) {
                return bad();
            }
            let fields: Vec<Ty> = table.struct_def(id).fields.iter().map(|field| field.ty).collect();
            let offsets = table.layout(ty).field_offsets;
            for (field, offset) in fields.iter().zip(offsets) {
                place(m, *field, tokens, image, at + offset as usize, symbol, parts)?;
            }
            if token(tokens) != Some(")") {
                return bad();
            }
        }
        TyKind::Enum(id) => {
            let Some(variant) = token(tokens).and_then(|t| t.strip_prefix('e')).and_then(|v| v.parse::<usize>().ok()) else { return bad() };
            let def = table.enum_def(id).clone();
            let Some(fields) = def.variants.get(variant).map(|v| v.fields.iter().map(|field| field.ty).collect::<Vec<Ty>>()) else { return bad() };
            if !open(tokens) {
                return bad();
            }
            let niche = table.option_niche(id);
            let offsets = if niche.is_some() { vec![vec![0]; def.variants.len()] } else { table.variant_field_offsets(id) };
            for (index, field) in fields.iter().enumerate() {
                if m.ty.is_zst(*field) {
                    if token(tokens) != Some("_") {
                        return bad();
                    }
                    continue;
                }
                place(m, *field, tokens, image, at + offsets[variant][index] as usize, symbol, parts)?;
            }
            if token(tokens) != Some(")") {
                return bad();
            }
            match niche {
                // `None` of a niche `Option`: the value its payload never holds.
                Some(niche) if variant != niche.some => match table.kind(niche.payload) {
                    TyKind::Bool => image.put(at, 2, 1),
                    TyKind::Char => image.put(at, 0x11_0000, 4),
                    TyKind::Str | TyKind::Span { .. } => image.put(at + 8, u128::from(u64::MAX), 8),
                    TyKind::Enum(inner) => {
                        let sentinel = table.enum_unused_discriminant(*inner).expect("an enum niche has an unused discriminant");
                        let tag = width(m, table.enum_def(*inner).repr);
                        image.put(at, sentinel as u128, tag);
                    }
                    _ => return unsupported("a constant `None` of this niche"),
                },
                Some(_) => {}
                None => {
                    let Repr::Scalar(Sc::I(bits)) = m.ty.repr(def.repr) else { return unsupported("an enum whose tag is not an integer") };
                    image.put(at, def.variants[variant].discriminant as u128, (bits / 8) as usize);
                }
            }
        }
        _ => return unsupported(format!("a constant static of `{}`", table.display(ty))),
    }
    Ok(())
}

/// The accessor of the static `name` that `init` initialises, written the first time.
pub(crate) fn lazy_static(m: &mut Module<'_>, name: &str, init: &str, ty: Ty) -> Result<String> {
    let accessor = format!("{init}_static");
    let ir_name = global_name(&accessor);
    if m.helper_defined(&accessor) {
        return Ok(ir_name);
    }
    let (size, align) = m.ty.size_align(ty);
    let reload = m.reload.is_some();
    let (mut storage, mut state) = (global_name(&format!("{init}.storage")), global_name(&format!("{init}.state")));
    if !reload {
        m.global(format!("{storage} = internal global [{} x i8] zeroinitializer, align {}", size.max(1), align.max(1)));
        m.global(format!("{state} = internal global i32 0, align 4"));
    }
    let shown = name.rsplit('.').next().unwrap_or(name);
    let mut cx = FnCx::glue(m);
    // `[HR-4a]`, `[HR-17]` — in a reloadable image the storage and the state word are the runtime's,
    // found by the static's name, so a reload keeps the value.
    if reload {
        storage = crate::reload::static_storage(&mut cx, name);
        state = crate::reload::static_state(&mut cx, name);
    }
    let text = cx.m.bytes(shown.as_bytes());
    let first = cx.alloca(4, 4);
    cx.call_rt("rt_static_begin", vec![Val::Imm(crate::types::Sc::Ptr, state.clone()), Val::Imm(crate::types::Sc::Ptr, text)], Some((&first, 4)));
    let first = cx.load(crate::types::Sc::I(32), &first, 4);
    let runs = cx.value(format!("icmp ne i32 {first}, 0"));
    let (run, done) = (cx.label(), cx.label());
    cx.emit(format!("br i1 {runs}, label %{run}, label %{done}"));
    cx.start(&run);
    cx.call_body(init, Vec::new(), (size > 0).then_some((storage.as_str(), align)))?;
    cx.call_rt("rt_static_end", vec![Val::Imm(crate::types::Sc::Ptr, state)], None);
    cx.emit(format!("br label %{done}"));
    cx.start(&done);
    let attrs = crate::func::fn_attrs(cx.m);
    let text = format!(
        "define internal ptr {ir_name}() {attrs} {{\nentry:\n{}  br label %start\nstart:\n{}  ret ptr {storage}\n}}\n\n",
        cx.allocas_text(),
        cx.out_text()
    );
    cx.m.fns.push_str(&text);
    Ok(ir_name)
}
