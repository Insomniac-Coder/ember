//! Text as the runtime writes it: numbers (`[STD-20]`: a float's text is Python's `repr`),
//! Python's format mini-language (`[LEX-19]`), `repr` of text (`[TYP-39]`), and each value's
//! `Display` and `Debug` text as the C backend's generated functions write them
//! (`emit_fmt_fns`, `debug_stmt`, `unit_display`).

use ember_mir::FormatSpec;
use ember_types::{Ty, TyKind};

use crate::wide::W256;
use crate::{fault, Interp, R};

// -- numbers ------------------------------------------------------------------------

/// `printf("%.*e")`: `d.ddde+XX`, at least two exponent digits.
fn c_exponent(value: f64, precision: usize, upper: bool) -> String {
    let text = format!("{value:.precision$e}");
    let (mantissa, exponent) = text.split_once('e').expect("an exponent");
    let exponent: i32 = exponent.parse().expect("a number");
    let sign = if exponent < 0 { '-' } else { '+' };
    let e = if upper { 'E' } else { 'e' };
    format!("{mantissa}{e}{sign}{:02}", exponent.abs())
}

/// The value read back at `width` bits (`reads_back`).
fn reads_back(text: &str, value: f64, width: u32) -> bool {
    let parsed: f64 = text.parse().unwrap_or(f64::NAN);
    match width {
        16 => crate::scalar::f64_to_f16(parsed) == crate::scalar::f64_to_f16(value),
        32 => (parsed as f32) == (value as f32) && text.parse::<f32>().ok() == Some(value as f32),
        _ => parsed == value,
    }
}

/// `write_repr`: Python's `repr` of a float of `width` bits.
pub fn repr_float(value: f64, width: u32) -> String {
    if value.is_nan() {
        return "nan".to_string();
    }
    if value.is_infinite() {
        return if value < 0.0 { "-inf" } else { "inf" }.to_string();
    }
    let max_digits = match width {
        16 => 5,
        32 => 9,
        _ => 17,
    };
    let mut scientific = String::new();
    for digits in 1..=max_digits {
        scientific = c_exponent(value, digits - 1, false);
        if reads_back(&scientific, value, width) {
            break;
        }
    }
    let negative = scientific.starts_with('-');
    let body = scientific.trim_start_matches('-');
    let (mantissa, exponent) = body.split_once('e').expect("an exponent");
    let exponent: i32 = exponent.parse().expect("a number");
    let mut digits: Vec<u8> = mantissa.bytes().filter(|b| *b != b'.').collect();
    while digits.len() > 1 && digits.last() == Some(&b'0') {
        digits.pop();
    }
    let mut text = String::new();
    if negative {
        text.push('-');
    }
    let n = digits.len();
    if (-4..16).contains(&exponent) {
        if exponent < 0 {
            text.push_str("0.");
            for _ in 0..(-exponent - 1) {
                text.push('0');
            }
            text.push_str(std::str::from_utf8(&digits).expect("digits"));
        } else {
            let whole = (exponent + 1) as usize;
            for i in 0..whole {
                text.push(if i < n { digits[i] as char } else { '0' });
            }
            text.push('.');
            if whole >= n {
                text.push('0');
            } else {
                text.push_str(std::str::from_utf8(&digits[whole..]).expect("digits"));
            }
        }
    } else {
        text.push(digits[0] as char);
        if n > 1 {
            text.push('.');
            text.push_str(std::str::from_utf8(&digits[1..]).expect("digits"));
        }
        text.push_str(&format!("e{}{:02}", if exponent < 0 { '-' } else { '+' }, exponent.abs()));
    }
    text
}

/// `printf("%.*<letter>")` with `#` when `alternate`, for `e`, `E`, `f`, `F`, `g`, `G`.
fn c_float(letter: char, precision: usize, alternate: bool, value: f64) -> String {
    match letter {
        'e' | 'E' => {
            let text = c_exponent(value, precision, letter == 'E');
            if alternate && precision == 0 {
                let at = text.find(['e', 'E']).expect("an exponent");
                format!("{}.{}", &text[..at], &text[at..])
            } else {
                text
            }
        }
        'f' | 'F' => {
            let text = format!("{value:.precision$}");
            if alternate && precision == 0 { format!("{text}.") } else { text }
        }
        _ => {
            let p = if precision == 0 { 1 } else { precision };
            let exponent: i32 = c_exponent(value, p - 1, false).split_once('e').expect("an exponent").1.parse().expect("a number");
            let mut text = if exponent < -4 || exponent >= p as i32 {
                c_exponent(value, p - 1, letter == 'G')
            } else {
                format!("{value:.*}", (p as i32 - 1 - exponent).max(0) as usize)
            };
            if !alternate {
                let (number, tail) = match text.find(['e', 'E']) {
                    Some(at) => (text[..at].to_string(), text[at..].to_string()),
                    None => (text.clone(), String::new()),
                };
                let number = if number.contains('.') { number.trim_end_matches('0').trim_end_matches('.').to_string() } else { number };
                text = format!("{number}{tail}");
            }
            text
        }
    }
}

fn utf8_count(bytes: &[u8]) -> usize {
    bytes.iter().filter(|b| (**b & 0xC0) != 0x80).count()
}

fn push_fill(out: &mut Vec<u8>, fill: char, count: usize) {
    let mut buffer = [0u8; 4];
    let encoded = fill.encode_utf8(&mut buffer).as_bytes();
    for _ in 0..count {
        out.extend_from_slice(encoded);
    }
}

/// `emit_padded`: `prefix` then `body`, padded to the width.
fn emit_padded(out: &mut Vec<u8>, prefix: &[u8], body: &[u8], spec: &FormatSpec, default_align: char) {
    let len = prefix.len() + utf8_count(body);
    let width = spec.width as usize;
    let pad = width.saturating_sub(len);
    let align = spec.align.unwrap_or(default_align);
    let left = match align {
        '>' => pad,
        '^' => pad / 2,
        _ => 0,
    };
    push_fill(out, spec.fill, left);
    out.extend_from_slice(prefix);
    out.extend_from_slice(body);
    push_fill(out, spec.fill, pad - left);
}

fn grouped_length(digits: usize, sep: Option<char>, every: usize) -> usize {
    digits + if sep.is_some() && digits > 0 { (digits - 1) / every } else { 0 }
}

/// `emit_number`: the integer digits grouped and zero-padded, then `rest`.
fn emit_number(out: &mut Vec<u8>, prefix: &str, digits: &[u8], rest: &str, spec: &FormatSpec, every: usize) {
    let mut zeros = 0;
    if spec.zero && spec.align.is_none() {
        let need = (spec.width as usize).saturating_sub(prefix.len() + rest.len());
        while grouped_length(digits.len() + zeros, spec.grouping, every) < need {
            zeros += 1;
        }
    }
    let total = digits.len() + zeros;
    let mut body = Vec::new();
    for i in 0..total {
        if let Some(sep) = spec.grouping
            && i > 0
            && (total - i) % every == 0
        {
            body.push(sep as u8);
        }
        body.push(if i < zeros { b'0' } else { digits[i - zeros] });
    }
    body.extend_from_slice(rest.as_bytes());
    emit_padded(out, prefix.as_bytes(), &body, spec, '>');
}

fn sign_prefix(negative: bool, spec: &FormatSpec) -> String {
    if negative {
        "-".to_string()
    } else {
        match spec.sign {
            Some(sign @ ('+' | ' ')) => sign.to_string(),
            _ => String::new(),
        }
    }
}

/// `fmt_spec_integer`: an integer of this sign and magnitude under `spec`.
pub fn spec_integer(out: &mut Vec<u8>, negative: bool, magnitude: u128, spec: &FormatSpec) {
    let digits = |base: u32| match base {
        16 => format!("{magnitude:x}"),
        2 => format!("{magnitude:b}"),
        8 => format!("{magnitude:o}"),
        _ => magnitude.to_string(),
    };
    spec_digits(out, negative, magnitude as f64, digits, spec)
}

/// `spec_integer` for a magnitude of any width: its value as a float, and its digits in a base.
pub fn spec_digits(out: &mut Vec<u8>, negative: bool, as_float: f64, digits: impl Fn(u32) -> String, spec: &FormatSpec) {
    let kind = match spec.kind {
        None | Some('?') => 'd',
        Some(kind) => kind,
    };
    if "eEfFgG%".contains(kind) {
        return spec_float(out, if negative { -as_float } else { as_float }, 64, spec);
    }
    let base = match kind {
        'x' | 'X' => 16,
        'b' => 2,
        'o' => 8,
        _ => 10,
    };
    let mut digits = digits(base);
    if kind == 'X' {
        digits = digits.to_uppercase();
    }
    let mut prefix = sign_prefix(negative, spec);
    if spec.alternate && base != 10 {
        prefix.push('0');
        prefix.push(kind);
    }
    emit_number(out, &prefix, digits.as_bytes(), "", spec, if base == 10 { 3 } else { 4 });
}

/// `fmt_spec_float`: a float of `width` bits under `spec`.
pub fn spec_float(out: &mut Vec<u8>, value: f64, width: u32, spec: &FormatSpec) {
    let negative = value.is_sign_negative() && !value.is_nan();
    let magnitude = value.abs();
    let prefix = sign_prefix(negative, spec);
    let kind = spec.kind;
    let upper = matches!(kind, Some('E' | 'F' | 'G'));
    if !magnitude.is_finite() {
        let word = if magnitude.is_nan() { if upper { "NAN" } else { "nan" } } else if upper { "INF" } else { "inf" };
        let plain = FormatSpec { zero: false, ..*spec };
        return emit_padded(out, prefix.as_bytes(), word.as_bytes(), &plain, '>');
    }
    let precision = spec.precision.map_or(6, |p| p as usize);
    let text = match kind {
        None if spec.precision.is_none() => repr_float(magnitude, width),
        Some('?') => repr_float(magnitude, width),
        None => {
            let digits = if precision == 0 { 1 } else { precision };
            let scientific = c_exponent(magnitude, digits - 1, false);
            let at = scientific.find('e').expect("an exponent");
            let exponent: i32 = scientific[at + 1..].parse().expect("a number");
            if exponent < -4 || exponent >= digits as i32 - 1 {
                let mut end = at;
                let bytes = scientific.as_bytes();
                while end > 1 && bytes[end - 1] == b'0' {
                    end -= 1;
                }
                if bytes[end - 1] == b'.' {
                    end -= 1;
                }
                format!("{}{}", &scientific[..end], &scientific[at..])
            } else {
                let fixed = format!("{magnitude:.*}", (digits as i32 - 1 - exponent).max(0) as usize);
                if fixed.contains('.') {
                    let bytes = fixed.as_bytes();
                    let mut end = fixed.len();
                    while bytes[end - 1] == b'0' && bytes[end - 2] != b'.' {
                        end -= 1;
                    }
                    fixed[..end].to_string()
                } else {
                    format!("{fixed}.0")
                }
            }
        }
        Some(kind) => {
            let shown = if kind == '%' { magnitude * 100.0 } else { magnitude };
            let letter = if kind == '%' { 'f' } else { kind };
            let mut text = c_float(letter, precision, spec.alternate, shown);
            if kind == '%' {
                text.push('%');
            }
            text
        }
    };
    let n = text.bytes().take_while(u8::is_ascii_digit).count();
    emit_number(out, &prefix, &text.as_bytes()[..n], &text[n..], spec, 3);
}

/// `repr_text`: Python's `repr` of text.
pub fn repr_text(out: &mut Vec<u8>, s: &[u8]) {
    let quote = if s.contains(&b'\'') && !s.contains(&b'"') { b'"' } else { b'\'' };
    out.push(quote);
    for &c in s {
        if c == quote || c == b'\\' {
            out.extend_from_slice(&[b'\\', c]);
        } else if c == b'\n' || c == b'\r' || c == b'\t' {
            out.extend_from_slice(&[b'\\', match c { b'\n' => b'n', b'\r' => b'r', _ => b't' }]);
        } else if c < 0x20 || c == 0x7F {
            out.extend_from_slice(format!("\\x{c:02x}").as_bytes());
        } else {
            out.push(c);
        }
    }
    out.push(quote);
}

/// `fmt_spec_text`: text, or its `repr` when `quoted`, under `spec`.
pub fn spec_text(out: &mut Vec<u8>, s: &[u8], quoted: bool, spec: &FormatSpec) {
    let mut body = Vec::new();
    if quoted {
        repr_text(&mut body, s);
    } else {
        body.extend_from_slice(s);
    }
    let mut len = body.len();
    if let Some(precision) = spec.precision {
        let mut kept = 0;
        len = 0;
        while len < body.len() {
            if (body[len] & 0xC0) != 0x80 {
                if kept == precision as usize {
                    break;
                }
                kept += 1;
            }
            len += 1;
        }
    }
    emit_padded(out, b"", &body[..len], spec, '<');
}

/// `[CELL-5]` — a `RefCell` borrow that found contention (`ember_panic_refcell`).
pub fn refcell_message(file: &str, line: u32) -> String {
    if file.is_empty() {
        "RefCell already mutably borrowed (borrowed at unknown:0)".to_string()
    } else {
        format!("RefCell already mutably borrowed (borrowed at {file}:{line})")
    }
}

pub fn encode_char(c: u32) -> Vec<u8> {
    match char::from_u32(c) {
        Some(c) => c.to_string().into_bytes(),
        None => "\u{FFFD}".as_bytes().to_vec(),
    }
}

// -- values ---------------------------------------------------------------------------

impl Interp<'_, '_> {
    /// `is_display_aggregate`.
    pub(crate) fn display_aggregate(&self, ty: Ty) -> bool {
        match self.types.kind(ty) {
            TyKind::Vec { text, .. } => !text,
            TyKind::Span { .. } | TyKind::Array { .. } | TyKind::Tuple(_) => true,
            TyKind::Enum(_) | TyKind::Struct(_) => true,
            TyKind::Class(_) | TyKind::ClassInterface(_) => true,
            _ => false,
        }
    }

    /// `unit_display`: a unit-only enum's `Display`, its variant's name (a C enum's value no
    /// variant names, its number); `void`'s, `()`.
    pub(crate) fn unit_display(&self, addr: u64, ty: Ty) -> R<Option<Vec<u8>>> {
        if matches!(self.types.kind(ty), TyKind::Void) {
            return Ok(Some(b"()".to_vec()));
        }
        let TyKind::Enum(id) = *self.types.kind(ty) else { return Ok(None) };
        let def = self.types.enum_def(id);
        if !def.is_unit_only() {
            return Ok(None);
        }
        let tag = self.discriminant(crate::Loc::at(addr, ty))?;
        if let Some(variant) = def.variants.iter().find(|v| v.discriminant == tag) {
            return Ok(Some(variant.name.as_str().as_bytes().to_vec()));
        }
        Ok(Some(if def.open { tag.to_string().into_bytes() } else { Vec::new() }))
    }

    /// `printed_as` (ODR-094): what a `Box[T]`, a `Cell[T]` or a `some` wrapper prints as, and
    /// where.
    fn printed_payload(&self, addr: u64, ty: Ty) -> R<Option<(u64, Ty)>> {
        let Some(inner) = self.types.printed_as(ty) else { return Ok(None) };
        let TyKind::Struct(id) = *self.types.kind(ty) else { return Ok(None) };
        Ok(Some(match self.types.compiler_box_inner(id) {
            Some(_) => (self.mem.read_u64(addr)?, inner),
            None => (addr + self.layout(ty).field_offsets.first().copied().unwrap_or(0), inner),
        }))
    }

    fn is_text(&self, ty: Ty) -> bool {
        matches!(self.types.kind(ty), TyKind::Str | TyKind::Vec { text: true, .. })
    }

    /// A `str` or `String` value's bytes.
    fn text_at(&self, addr: u64) -> R<Vec<u8>> {
        let (ptr, len) = (self.mem.read_u64(addr)?, self.mem.read_u64(addr + 8)?);
        if len == 0 {
            return Ok(Vec::new());
        }
        Ok(self.mem.read(ptr, len)?.to_vec())
    }

    /// A scalar's text as `fmt_<suffix>` writes it.
    fn scalar_text(&self, out: &mut Vec<u8>, addr: u64, ty: Ty) -> R<()> {
        if self.is_text(ty) {
            out.extend_from_slice(&self.text_at(addr)?);
            return Ok(());
        }
        match self.types.kind(ty) {
            TyKind::Bool => out.extend_from_slice(if self.mem.read(addr, 1)?[0] != 0 { b"true" } else { b"false" }),
            TyKind::Char => out.extend_from_slice(&encode_char(self.mem.read_u32(addr)?)),
            TyKind::Range(id) => return self.scalar_text(out, addr, self.types.range_def(*id).repr),
            _ => {
                if let Some(bits) = self.float_bits(ty) {
                    let bytes = self.mem.read(addr, self.size(ty))?;
                    out.extend_from_slice(repr_float(self.float_value(ty, bytes), bits).as_bytes());
                } else if let Some(kind) = self.int_kind(ty) {
                    let bytes = self.mem.read(addr, kind.bytes() as u64)?;
                    let text = if kind.bits > 128 {
                        let w = W256::of(kind, bytes);
                        let negative = kind.signed && w.negative();
                        format!("{}{}", if negative { "-" } else { "" }, if negative { w.neg() } else { w }.digits(10))
                    } else if kind.signed {
                        kind.signed_value(bytes).to_string()
                    } else {
                        kind.unsigned(bytes).to_string()
                    };
                    out.extend_from_slice(text.as_bytes());
                } else {
                    return fault(format!("the text of `{}`", self.types.display(ty)));
                }
            }
        }
        Ok(())
    }

    /// `Format`: the `Display` text of the value of `ty` at `addr`.
    pub(crate) fn display_into(&mut self, out: &mut Vec<u8>, addr: u64, ty: Ty) -> R<()> {
        if let Some(name) = self.unit_display(addr, ty)? {
            out.extend_from_slice(&name);
            return Ok(());
        }
        if self.display_aggregate(ty) {
            return self.fmt_fn(out, addr, ty);
        }
        self.scalar_text(out, addr, ty)
    }

    /// `FormatWith`: the value's text under `spec`.
    pub(crate) fn display_with(&mut self, out: &mut Vec<u8>, addr: u64, ty: Ty, spec: &FormatSpec) -> R<()> {
        let more = FormatSpec { kind: None, ..*spec } != FormatSpec::PLAIN;
        let debug = spec.kind == Some('?');
        if self.display_aggregate(ty) || matches!(self.types.kind(ty), TyKind::Void) {
            let mut text = Vec::new();
            match (debug, self.unit_display(addr, ty)?) {
                (false, Some(name)) => text = name,
                _ => self.fmt_fn(&mut text, addr, ty)?,
            }
            if !more {
                out.extend_from_slice(&text);
            } else {
                spec_text(out, &text, false, &FormatSpec { kind: None, ..*spec });
            }
            return Ok(());
        }
        if self.is_text(ty) {
            let text = self.text_at(addr)?;
            spec_text(out, &text, debug, spec);
            return Ok(());
        }
        match self.types.kind(ty) {
            TyKind::Bool => {
                let text: &[u8] = if self.mem.read(addr, 1)?[0] != 0 { b"true" } else { b"false" };
                spec_text(out, text, false, spec);
            }
            TyKind::Char => {
                let c = encode_char(self.mem.read_u32(addr)?);
                spec_text(out, &c, debug, spec);
            }
            TyKind::Range(id) => return self.display_with(out, addr, self.types.range_def(*id).repr, spec),
            _ => {
                if let Some(bits) = self.float_bits(ty) {
                    let bytes = self.mem.read(addr, self.size(ty))?;
                    spec_float(out, self.float_value(ty, bytes), bits, spec);
                } else if let Some(kind) = self.int_kind(ty) {
                    let bytes = self.mem.read(addr, kind.bytes() as u64)?.to_vec();
                    if kind.bits > 128 {
                        let w = W256::of(kind, &bytes);
                        let negative = kind.signed && w.negative();
                        let magnitude = if negative { w.neg() } else { w };
                        spec_digits(out, negative, magnitude.to_f64(false), |base| magnitude.digits(base), spec);
                    } else if kind.signed {
                        let v = kind.signed_value(&bytes);
                        spec_integer(out, v < 0, v.unsigned_abs(), spec);
                    } else {
                        spec_integer(out, false, kind.unsigned(&bytes), spec);
                    }
                } else {
                    return fault(format!("the formatted text of `{}`", self.types.display(ty)));
                }
            }
        }
        Ok(())
    }

    /// `debug_stmt`: the `Debug` text of a part of an aggregate.
    pub(crate) fn debug_into(&mut self, out: &mut Vec<u8>, addr: u64, ty: Ty) -> R<()> {
        if self.types.debug_writer(ty).is_some() && !self.display_aggregate(ty) {
            return self.fmt_fn(out, addr, ty);
        }
        if let Some((payload, inner)) = self.printed_payload(addr, ty)? {
            return self.debug_into(out, payload, inner);
        }
        if let TyKind::Ref { inner, .. } = *self.types.kind(ty) {
            let pointee = self.mem.read_u64(addr)?;
            return self.debug_into(out, pointee, inner);
        }
        if self.is_text(ty) {
            let text = self.text_at(addr)?;
            repr_text(out, &text);
            return Ok(());
        }
        match self.types.kind(ty) {
            TyKind::Void => out.extend_from_slice(b"()"),
            TyKind::Char => repr_text(out, &encode_char(self.mem.read_u32(addr)?)),
            _ if self.display_aggregate(ty) => self.fmt_fn(out, addr, ty)?,
            _ => self.scalar_text(out, addr, ty)?,
        }
        Ok(())
    }

    /// The generated `Display` function of an aggregate (`emit_fmt_fns`).
    pub(crate) fn fmt_fn(&mut self, out: &mut Vec<u8>, addr: u64, ty: Ty) -> R<()> {
        if let Some(writer) = self.types.debug_writer(ty).map(str::to_string) {
            return self.call_debug_writer(out, &writer, addr, ty);
        }
        match self.types.kind(ty).clone() {
            TyKind::Vec { elem, .. } | TyKind::Span { elem, .. } => {
                let (ptr, len) = (self.mem.read_u64(addr)?, self.mem.read_u64(addr + 8)?);
                self.list_text(out, ptr, len, elem)
            }
            TyKind::Array { elem, len } => self.list_text(out, addr, len, elem),
            // `[TYP-36]` — `void`'s `Debug`.
            TyKind::Void => {
                out.extend_from_slice(b"()");
                Ok(())
            }
            TyKind::Class(_) | TyKind::ClassInterface(_) => {
                let object = self.mem.read_u64(addr)?;
                let name = self.object_name(object)?;
                out.extend_from_slice(format!("<{name} at {object:#x}>").as_bytes());
                Ok(())
            }
            TyKind::Tuple(items) => {
                let offsets = self.layout(ty).field_offsets.clone();
                out.push(b'(');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.extend_from_slice(b", ");
                    }
                    self.debug_into(out, addr + offsets[i], *item)?;
                }
                if items.len() == 1 {
                    out.push(b',');
                }
                out.push(b')');
                Ok(())
            }
            TyKind::Struct(id) => {
                let def = self.types.struct_def(id);
                let origin = def.origin.as_ref().map(|(name, _)| name.as_str().to_string());
                if matches!(origin.as_deref(), Some("std.collections.Map" | "std.collections.Set")) {
                    return self.map_text(out, addr, ty, origin.as_deref() == Some("std.collections.Set"));
                }
                let name = def.origin.as_ref().map(|(name, _)| name.as_str()).unwrap_or(def.name.as_str());
                let name = name.rsplit('.').next().unwrap_or(name).to_string();
                let fields: Vec<(String, Ty)> = def.fields.iter().map(|f| (f.name.to_string(), f.ty)).collect();
                let offsets = self.layout(ty).field_offsets.clone();
                out.extend_from_slice(format!("{name}(").as_bytes());
                for (i, (field, field_ty)) in fields.iter().enumerate() {
                    if i > 0 {
                        out.extend_from_slice(b", ");
                    }
                    out.extend_from_slice(format!("{field}=").as_bytes());
                    self.debug_into(out, addr + offsets[i], *field_ty)?;
                }
                out.push(b')');
                Ok(())
            }
            TyKind::Enum(id) => {
                let def = self.types.enum_def(id);
                let compiler_known = def.name.as_str().starts_with("Option_") || def.name.as_str().starts_with("Result_");
                let name = def.origin.as_ref().map(|(name, _)| name.as_str()).unwrap_or(def.name.as_str());
                let name = name.rsplit('.').next().unwrap_or(name).to_string();
                let prefix = if compiler_known { String::new() } else { format!("{name}.") };
                let tag = self.discriminant(crate::Loc::at(addr, ty))?;
                let Some(variant_index) = def.variants.iter().position(|v| v.discriminant == tag) else {
                    if def.open {
                        out.extend_from_slice(format!("{name}({tag})").as_bytes());
                    }
                    return Ok(());
                };
                let variant = &def.variants[variant_index];
                out.extend_from_slice(format!("{prefix}{}", variant.name).as_bytes());
                if !variant.fields.is_empty() {
                    let fields: Vec<(String, Ty)> = variant.fields.iter().map(|f| (f.name.to_string(), f.ty)).collect();
                    let niche = self.types.option_niche(id).is_some();
                    let offsets = if niche { Vec::new() } else { self.types.variant_field_offsets(id).swap_remove(variant_index) };
                    out.push(b'(');
                    for (i, (field, field_ty)) in fields.iter().enumerate() {
                        if i > 0 {
                            out.extend_from_slice(b", ");
                        }
                        if !field.starts_with('_') {
                            out.extend_from_slice(format!("{field}=").as_bytes());
                        }
                        let at = if niche { addr } else { addr + offsets[i] };
                        self.debug_into(out, at, *field_ty)?;
                    }
                    out.push(b')');
                }
                Ok(())
            }
            _ => fault(format!("a `Debug` function for `{}`", self.types.display(ty))),
        }
    }

    fn list_text(&mut self, out: &mut Vec<u8>, ptr: u64, len: u64, elem: Ty) -> R<()> {
        let step = self.size(elem);
        out.push(b'[');
        for i in 0..len {
            if i > 0 {
                out.extend_from_slice(b", ");
            }
            self.debug_into(out, ptr + i * step, elem)?;
        }
        out.push(b']');
        Ok(())
    }

    /// `fmt_map_body`: `{k: v, …}` and `{a, …}` in insertion order; an empty `Set` is `set()`.
    fn map_text(&mut self, out: &mut Vec<u8>, addr: u64, ty: Ty, is_set: bool) -> R<()> {
        let types = self.types;
        let TyKind::Struct(id) = *types.kind(ty) else { unreachable!() };
        let (map_ty, map_addr) = if is_set {
            let map_ty = types.struct_def(id).fields[0].ty;
            (map_ty, addr + self.layout(ty).field_offsets[0])
        } else {
            (ty, addr)
        };
        let TyKind::Struct(map_id) = *types.kind(map_ty) else { return fault("a Set holds no Map") };
        let map_def = types.struct_def(map_id);
        let map_layout = self.layout(map_ty);
        let live_index = map_def.fields.iter().position(|f| f.name.is("live"));
        let entries_ty = map_def.fields[0].ty;
        let TyKind::Vec { elem: option, .. } = *types.kind(entries_ty) else { return fault("a Map's entries are no Array") };
        let TyKind::Enum(option_id) = *types.kind(option) else { return fault("a Map entry is no Option") };
        let option_def = types.enum_def(option_id);
        let some = option_def.variants.iter().position(|v| v.name.is("Some")).expect("an Option has Some");
        let slot_ty = option_def.variants[some].fields[0].ty;
        let TyKind::Struct(slot_id) = *types.kind(slot_ty) else { return fault("a Map slot is no struct") };
        let slot_def = types.struct_def(slot_id);
        let key_index = slot_def.fields.iter().position(|f| f.name.is("key")).expect("a key");
        let value_index = slot_def.fields.iter().position(|f| f.name.is("value")).expect("a value");
        let (key_ty, value_ty) = (slot_def.fields[key_index].ty, slot_def.fields[value_index].ty);
        let slot_layout = self.layout(slot_ty);
        let niche = types.option_niche(option_id).is_some();
        let payload = if niche { 0 } else { types.variant_field_offsets(option_id)[some][0] };
        if is_set
            && let Some(live) = live_index
            && self.mem.read_u64(map_addr + map_layout.field_offsets[live])? == 0
        {
            out.extend_from_slice(b"set()");
            return Ok(());
        }
        let (ptr, len) = (self.mem.read_u64(map_addr)?, self.mem.read_u64(map_addr + 8)?);
        let step = self.size(option);
        out.push(b'{');
        let mut first = true;
        for i in 0..len {
            let entry = ptr + i * step;
            let tag = self.discriminant(crate::Loc::at(entry, option))?;
            if tag != option_def.variants[some].discriminant {
                continue;
            }
            if !first {
                out.extend_from_slice(b", ");
            }
            first = false;
            let slot = entry + payload;
            self.debug_into(out, slot + slot_layout.field_offsets[key_index], key_ty)?;
            if !is_set {
                out.extend_from_slice(b": ");
                self.debug_into(out, slot + slot_layout.field_offsets[value_index], value_ty)?;
            }
        }
        out.push(b'}');
        Ok(())
    }

    /// ADR-186 — a written `Debug`: its writer called with the value (as its borrowed parameter
    /// takes it) and the text being built, which it appends to.
    fn call_debug_writer(&mut self, out: &mut Vec<u8>, writer: &str, addr: u64, ty: Ty) -> R<()> {
        let value = if self.types.passed_by_address(ty) { addr.to_le_bytes().to_vec() } else { self.mem.read(addr, self.size(ty))?.to_vec() };
        let buffer = self.vec_from_bytes(out)?;
        let slot = self.mem.alloc(24, 8)?;
        self.mem.write(slot, &buffer)?;
        self.invoke_symbol(writer, vec![value, slot.to_le_bytes().to_vec()])?;
        let text = self.text_at(slot)?;
        self.vec_free(slot)?;
        self.mem.free(slot)?;
        *out = text;
        Ok(())
    }
}
