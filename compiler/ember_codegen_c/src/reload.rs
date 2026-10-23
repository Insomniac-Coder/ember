//! Annex B — the C of a reloadable image: each reloadable function's body under a name of its own
//! and a thunk under the function's symbol that loads its target from the image's copy of the call
//! table (`[HR-5]`, `[HR-6]`); the statics' storage taken from the runtime (`[HR-4a]`); the types'
//! identities and schemas (`[HR-11]`); and the reload manifest (`[HR-8]`), which the runtime reads
//! from the image's section and fills in.

use super::*;

/// What the driver decides of a reloadable build, which the backend writes into the image.
#[derive(Clone, Debug, Default)]
pub struct ReloadImage {
    /// `[build] reload`'s mode as the manifest records it: 1 `opt-in`, 2 `all`, 3 `bodies`.
    pub mode: u32,
    /// `[HR-12]` — the 40-byte object header with its live-instance list.
    pub instances: bool,
    /// The package's name, which the manifest carries.
    pub package: String,
    /// `[HR-5]` — every function reached through a thunk, by symbol.
    pub reloadable: BTreeSet<String>,
    /// `[HR-10]` — every `@noreload` function, by symbol, with its body's hash.
    pub noreload: BTreeMap<String, u64>,
    /// `[HR-17]`, `[HR-17a]` — every static the package declares, by its name: its initialiser's
    /// hash, its declared type's, and whether it is `@reinit_on_reload`. One whose value the
    /// compiler folds into the code has no storage, but an edit of it is still refused.
    pub statics: BTreeMap<String, (u64, u64, bool)>,
    /// The package's own source files: the types declared in them have schemas ([HR-11]).
    pub files: BTreeSet<ember_span::FileId>,
    /// `[HR-14]` — each field with a declared default, by its type's name and its own, with the
    /// name of the function the driver wrote to compute the default.
    pub defaults: BTreeMap<(String, String), String>,
    /// `[HR-14]` — `@renamed_from("old")` on a field or a variant, by its type's name and its own.
    pub renamed: BTreeMap<(String, String), String>,
    /// `[HR-16]` — each type that declares `migrate_from`, with the name of the adapter the driver
    /// wrote, and the name of the struct its old schema became.
    pub migrations: BTreeMap<String, (String, String)>,
    /// `[HR-23]` — each C++ header the package imports, by name, with the content hash of it and
    /// every file it includes.
    pub headers: BTreeMap<String, String>,
}

/// What a field holds, for the runtime's migration and reference walk ([HR-14], [HR-15]).
/// `layout` and `inner` are C expressions, `NULL` or the address of a table this image describes
/// a tuple's fields or an element's or pointee's value with.
struct FieldShape {
    kind: &'static str,
    flags: Vec<&'static str>,
    type_name: String,
    element_kind: &'static str,
    element_size: String,
    element_type: String,
    layout: String,
    inner: String,
    /// `[HR-14]` — for a range type, the function telling whether a value of its representation
    /// is in its range (`T.checked`'s test).
    range: String,
    /// `[HR-13]` — for a container's element row, the alignment its buffer is allocated with.
    align: String,
    /// `[HR-13]` — for an `SoA`'s column, where its value sits in a whole element; `0` otherwise.
    within: String,
}

impl FieldShape {
    fn plain(type_name: String) -> FieldShape {
        FieldShape {
            kind: "EMBER_RL_KIND_PLAIN",
            flags: Vec::new(),
            type_name,
            element_kind: "EMBER_RL_KIND_PLAIN",
            element_size: "0".to_string(),
            element_type: String::new(),
            layout: "NULL".to_string(),
            inner: "NULL".to_string(),
            range: "NULL".to_string(),
            align: "0".to_string(),
            within: "0".to_string(),
        }
    }

    /// The row's members after `flags`: what it holds, its elements', and its own tables.
    fn tail(&self) -> String {
        let element_type = if self.element_type.is_empty() { "NULL".to_string() } else { c_string_literal(&self.element_type) };
        format!(
            "{}, {}, {}, {element_type}, {}, {}, {}, {}, {}",
            self.kind, self.element_kind, self.element_size, self.layout, self.inner, self.range, self.align, self.within
        )
    }
}

/// `[HR-14]` — the symbol of range type `id`'s test (`emit_range_tests`).
fn range_test_symbol(id: ember_types::RangeId) -> String {
    table(&format!("in_range_{}", id.0))
}

/// `[HR-14]` — the C body of a range type's test over its representation at `value`: `T.checked`'s
/// decision (`[RNG-3]`), NaN in no range (`[RNG-6]`). `None` for a representation it has no test
/// for (`f16`), whose change the reload refuses.
fn range_test_body(kind: &TyKind, def: &ember_types::RangeDef) -> Option<String> {
    use ember_types::Bound;
    let upper = if def.inclusive { "<=" } else { "<" };
    let signed = |v: i128| if v == i128::from(i64::MIN) { "INT64_MIN".to_string() } else { format!("INT64_C({v})") };
    // A 128-bit value as its two halves, compared high first: the bytes a little-endian `__int128`
    // has, which is how the runtime lays one out.
    let wide = |high: &str, lo: i128, hi: i128| {
        let half = |v: i128| (format!("({high})INT64_C({})", (v >> 64) as i64), format!("UINT64_C({})", v as u64));
        let ((lo_h, lo_l), (hi_h, hi_l)) = (half(lo), half(hi));
        let below = if def.inclusive { "<=" } else { "<" };
        format!(
            "uint64_t l; {high} h; memcpy(&l, value, 8); memcpy(&h, (const unsigned char*)value + 8, 8);\n    \
             return (h > {lo_h} || (h == {lo_h} && l >= {lo_l})) && (h < {hi_h} || (h == {hi_h} && l {below} {hi_l}));"
        )
    };
    Some(match (kind, def.lo, def.hi) {
        (TyKind::Int(IntTy::I128), Bound::Int(lo), Bound::Int(hi)) => wide("int64_t", lo, hi),
        (TyKind::Uint(UintTy::U128), Bound::Int(lo), Bound::Int(hi)) => wide("uint64_t", lo, hi),
        (TyKind::Int(IntTy::I8 | IntTy::I16 | IntTy::I32 | IntTy::I64), Bound::Int(lo), Bound::Int(hi)) => {
            let width = match kind {
                TyKind::Int(IntTy::I8) => 8,
                TyKind::Int(IntTy::I16) => 16,
                TyKind::Int(IntTy::I32) => 32,
                _ => 64,
            };
            format!("int{width}_t n; memcpy(&n, value, sizeof n);\n    int64_t v = n;\n    return v >= {} && v {upper} {};", signed(lo), signed(hi))
        }
        (TyKind::Uint(UintTy::U8 | UintTy::U16 | UintTy::U32 | UintTy::U64), Bound::Int(lo), Bound::Int(hi)) => {
            let width = match kind {
                TyKind::Uint(UintTy::U8) => 8,
                TyKind::Uint(UintTy::U16) => 16,
                TyKind::Uint(UintTy::U32) => 32,
                _ => 64,
            };
            // A lower end of zero is no test: an unsigned value is never below it.
            let low = if lo <= 0 { String::new() } else { format!("v >= UINT64_C({lo}) && ") };
            format!("uint{width}_t n; memcpy(&n, value, sizeof n);\n    uint64_t v = n;\n    return {low}v {upper} UINT64_C({hi});")
        }
        (TyKind::Float(FloatTy::F32 | FloatTy::F64), Bound::Float(lo), Bound::Float(hi)) => {
            let float = |v: f64| if v.is_infinite() { (if v < 0.0 { "-HUGE_VAL" } else { "HUGE_VAL" }).to_string() } else { format!("{v:e}") };
            let c = if matches!(kind, TyKind::Float(FloatTy::F32)) { "float" } else { "double" };
            format!("{c} n; memcpy(&n, value, sizeof n);\n    double v = n;\n    return v >= {} && v {upper} {};", float(lo), float(hi))
        }
        _ => return None,
    })
}

/// The suffix of a reloadable body's own symbol; the function's symbol is its thunk's.
const BODY_SUFFIX: &str = "__rl";

/// FNV-1a: a schema or signature hash, stable across builds and hosts.
pub(crate) fn fnv(parts: &[&str]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for part in parts {
        for byte in part.bytes().chain(std::iter::once(0xff)) {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
        }
    }
    hash
}

/// The image's own tables and manifest, named from the mangling prefix.
fn table(name: &str) -> String {
    format!("{}rl_{name}", ember_branding::mangle_prefix())
}

/// The manifest's symbol, which the image's link keeps (`[HR-8]`).
pub fn manifest_symbol() -> String {
    format!("{}reload_manifest", ember_branding::mangle_prefix())
}

impl Emitter<'_> {
    pub(crate) fn is_reloadable(&self, symbol: &str) -> bool {
        self.reload.as_ref().is_some_and(|reload| reload.reloadable.contains(symbol))
    }

    /// The manifest's functions, in order: the reloadable ones and the `@noreload` ones, each
    /// with its index into the image's tables.
    pub(crate) fn plan_reload_slots(&mut self, bodies: &[Body]) {
        let Some(reload) = &self.reload else { return };
        let mut slots = BTreeMap::new();
        for body in bodies {
            if reload.reloadable.contains(&body.symbol) && !slots.contains_key(&body.symbol) {
                let next = slots.len();
                slots.insert(body.symbol.clone(), next);
            }
        }
        // `[HR-10]` — every `@noreload` function, whether or not its body was inlined away: a
        // reload that changes one is refused.
        for symbol in reload.noreload.keys() {
            let next = slots.len();
            slots.entry(symbol.clone()).or_insert(next);
        }
        self.reload_slots = slots;
    }

    /// A body's C parameters, each its type and its name, as `signature` writes them.
    pub(crate) fn c_params(&self, body: &Body) -> Vec<(String, String)> {
        let ffi_values = self.ffi_export_value_params.get(&body.symbol);
        let split = self.split_views.get(&body.symbol);
        let mut out = Vec::new();
        for (index, (id, decl)) in body.args().enumerate() {
            if split.is_some_and(|split| split.get(index) == Some(&true)) {
                let pointer = match self.types.kind(decl.ty) {
                    TyKind::Str => "const unsigned char*",
                    TyKind::Span { mutable: true, .. } => "void*",
                    _ => "const void*",
                };
                out.push((pointer.to_string(), format!("_{}_p", id.0)));
                out.push(("size_t".to_string(), format!("_{}_n", id.0)));
            } else if ffi_values.is_some_and(|values| values.get(index) == Some(&true)) {
                let TyKind::Ref { inner, .. } = self.types.kind(decl.ty) else { unreachable!() };
                out.push((self.c_member_type(*inner), format!("_ffi{}", id.0)));
            } else {
                out.push((self.c_member_type(decl.ty), format!("_{}", id.0)));
            }
        }
        out
    }

    /// The symbol a reloadable body is defined under.
    pub(crate) fn reload_body_symbol(symbol: &str) -> String {
        format!("{symbol}{BODY_SUFFIX}")
    }

    /// A body's signature under its own symbol.
    pub(crate) fn reload_body_signature(&self, body: &Body) -> String {
        let signature = self.signature(body);
        let named = format!(" {}(", body.symbol);
        assert!(signature.contains(&named), "a signature names its function");
        signature.replacen(&named, &format!(" {}(", Self::reload_body_symbol(&body.symbol)), 1)
    }

    /// The image's tables, declared before any thunk reads them.
    pub(crate) fn emit_reload_tables(&mut self) {
        if self.reload.is_none() {
            return;
        }
        let count = self.reload_slots.len().max(1);
        let statics = self.reload_static_entries().max(1);
        match self.shared_definition() {
            Some(linkage) => {
                self.line(&format!("{linkage}void (*{}[{count}])(void);", table("targets")));
                self.line(&format!("{linkage}void (*{}[{count}])(void);", table("addresses")));
                self.line(&format!("{linkage}const void* {}[{count}];", table("descriptors")));
                self.line(&format!("{linkage}void* {}[{statics}];", table("static_data")));
                self.line(&format!("{linkage}uint32_t* {}[{statics}];", table("static_state")));
            }
            None => {
                self.line(&format!("extern void (*{}[{count}])(void);", table("targets")));
                self.line(&format!("extern void (*{}[{count}])(void);", table("addresses")));
                self.line(&format!("extern const void* {}[{count}];", table("descriptors")));
                self.line(&format!("extern void* {}[{statics}];", table("static_data")));
                self.line(&format!("extern uint32_t* {}[{statics}];", table("static_state")));
            }
        }
        self.line("");
    }

    /// `[HR-6]` — the thunk: the function's own symbol, which loads the current target from the
    /// image's copy of the call table and calls it, a tail call the C compiler makes a jump.
    pub(crate) fn emit_thunk(&mut self, body: &Body) {
        let Some(&index) = self.reload_slots.get(&body.symbol) else { return };
        let ret = self.c_type(body.return_ty());
        let params = self.c_params(body);
        let declared = if params.is_empty() {
            "void".to_string()
        } else {
            params.iter().map(|(ty, name)| format!("{ty} {name}")).collect::<Vec<_>>().join(", ")
        };
        let types = if params.is_empty() {
            "void".to_string()
        } else {
            params.iter().map(|(ty, _)| ty.clone()).collect::<Vec<_>>().join(", ")
        };
        let names = params.iter().map(|(_, name)| name.clone()).collect::<Vec<_>>().join(", ");
        let call = format!("(({ret} (*)({types})){}[{index}])({names})", table("targets"));
        self.line(&format!("/* [HR-6] the thunk of `{}` */", body.name.replace("*/", "* /")));
        self.line(&format!("{ret} {}({declared}) {{", body.symbol));
        if self.is_void(body.return_ty()) {
            self.line(&format!("    {call};"));
        } else {
            self.line(&format!("    return {call};"));
        }
        self.line("}");
        self.line("");
    }

    /// The statics the image keeps in the runtime's storage, by name, each with its type and,
    /// for a lazily initialised one, its initialiser's symbol.
    pub(crate) fn reload_static_names(&self) -> Vec<(String, Ty, Option<String>)> {
        let mut out: Vec<(String, Ty, Option<String>)> = self
            .types
            .constant_statics()
            .iter()
            .map(|(name, ty, _)| (name.to_string(), *ty, None))
            .collect();
        out.extend(self.lazy_statics.iter().map(|(init, (ty, name))| (name.clone(), *ty, Some(init.clone()))));
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out.dedup_by(|a, b| a.0 == b.0);
        out
    }

    /// The manifest's statics: those with storage, then those the compiler folded away.
    fn reload_static_entries(&self) -> usize {
        let stored = self.reload_static_names();
        let folded = self.reload.as_ref().map_or(0, |reload| {
            reload.statics.keys().filter(|name| !stored.iter().any(|(known, _, _)| known == *name)).count()
        });
        stored.len() + folded
    }

    pub(crate) fn reload_static_index(&self, name: &str) -> usize {
        self.reload_static_names().iter().position(|(known, _, _)| known == name).expect("a static of this image")
    }

    /// `[HR-6]` — a value of a reloadable function: the descriptor the runtime made stand for it in
    /// every image, so the value is one whichever image made it, and calling it reaches the
    /// permanent thunk.
    pub(crate) fn reload_fn_value(&self, symbol: &str) -> Option<String> {
        let index = *self.reload_slots.get(symbol)?;
        if !self.is_reloadable(symbol) || !self.native_fn_values.contains_key(symbol) {
            return None;
        }
        let descriptor = format!("((const struct {RT}native_fn_descriptor*){}[{index}])", table("descriptors"));
        Some(format!("((struct {RT}fn_value){{{descriptor}->native, {descriptor}, {{0, 0, 0}}}})"))
    }

    /// `[HR-4a]` — a static's storage and state, the runtime's.
    pub(crate) fn reload_static_access(&self, name: &str, ty: Ty) -> String {
        format!("(({}*){}[{}])", self.c_type(ty), table("static_data"), self.reload_static_index(name))
    }

    /// `[STA-3]`'s accessor in a reloadable image: the storage and the state word are the
    /// runtime's, found by the static's name, so a reload keeps the value ([HR-17]).
    pub(crate) fn reload_lazy_static_definition(&self, init: &str, ty: Ty, name: &str) -> String {
        let c_type = self.c_type(ty);
        let shown = name.rsplit('.').next().unwrap_or(name);
        let index = self.reload_static_index(name);
        format!(
            "\n{c_type}* {init}_static(void) {{\n    {c_type}* storage = ({c_type}*){data}[{index}];\n    if ({rt}rt_static_begin({state}[{index}], \"{shown}\")) {{\n        *storage = {init}();\n        {rt}rt_static_end({state}[{index}]);\n    }}\n    return storage;\n}}\n",
            data = table("static_data"),
            state = table("static_state"),
            rt = RT,
        )
    }

    /// A type's schema hash ([HR-11]): its kind, name, layout and each field's or variant's name
    /// and schema, so equal hashes mean nothing to migrate. A type met again inside itself (behind
    /// a pointer) counts by its name. `[LAY-2]` — its layout attributes too, when it has any: a
    /// change to them is a schema change, which migrates by name, or is refused for a type a raw
    /// pointer can reach (`[HR-14]`'s last row, through `[HR-13b]`).
    pub(crate) fn type_schema(&self, ty: Ty, seen: &mut Vec<Ty>) -> u64 {
        if seen.contains(&ty) {
            return fnv(&["again", &self.types.display(ty)]);
        }
        seen.push(ty);
        let hash = match self.types.kind(ty) {
            TyKind::Struct(id) => {
                let def = self.types.struct_def(*id);
                let mut parts = vec!["struct".to_string(), def.name.to_string()];
                parts.extend(self.layout_schema(ty));
                for field in &def.fields {
                    parts.push(field.name.to_string());
                    parts.push(self.type_schema(field.ty, seen).to_string());
                }
                fnv(&parts.iter().map(String::as_str).collect::<Vec<_>>())
            }
            TyKind::Enum(id) => {
                let def = self.types.enum_def(*id);
                let mut parts = vec!["enum".to_string(), def.name.to_string(), self.types.display(def.repr)];
                parts.extend(self.layout_schema(ty));
                for variant in &def.variants {
                    parts.push(variant.name.to_string());
                    for field in &variant.fields {
                        parts.push(field.name.to_string());
                        parts.push(self.type_schema(field.ty, seen).to_string());
                    }
                }
                fnv(&parts.iter().map(String::as_str).collect::<Vec<_>>())
            }
            // Held inside another value, a class is a handle: its own layout is not the holder's.
            TyKind::Class(id) if seen.len() > 1 => fnv(&["handle", &self.types.class_def(*id).name.to_string()]),
            TyKind::Class(id) => {
                let mut parts = vec!["class".to_string(), self.types.class_def(*id).name.to_string()];
                if let Some(base) = self.types.class_def(*id).base {
                    parts.push(self.types.class_def(base).name.to_string());
                }
                for index in 0..self.types.class_field_count(*id) {
                    if let Some(field) = self.types.class_field_at(*id, index) {
                        parts.push(field.name.to_string());
                        parts.push(self.type_schema(field.ty, seen).to_string());
                    }
                }
                fnv(&parts.iter().map(String::as_str).collect::<Vec<_>>())
            }
            TyKind::Tuple(items) => {
                let mut parts = vec!["tuple".to_string()];
                for item in items.clone() {
                    parts.push(self.type_schema(item, seen).to_string());
                }
                fnv(&parts.iter().map(String::as_str).collect::<Vec<_>>())
            }
            TyKind::Ptr { inner, .. } => {
                let inner = *inner;
                fnv(&["pointer", &self.types.display(ty), &self.type_schema(inner, seen).to_string()])
            }
            // `[HR-13]` — an `Array`'s or a fixed array's elements are in its schema: a container of
            // a type that changed holds values to migrate. A class element is a handle (above).
            TyKind::Vec { elem, text: false } => {
                let elem = *elem;
                fnv(&["array", &self.type_schema(elem, seen).to_string()])
            }
            TyKind::Array { elem, len } => {
                let (elem, len) = (*elem, *len);
                fnv(&["fixed", &format!("{len:?}"), &self.type_schema(elem, seen).to_string()])
            }
            // `[HR-14]` — a range type's range is in its schema: a value of the old one may lie
            // outside the new one.
            TyKind::Range(id) => {
                let def = self.types.range_def(*id);
                fnv(&["range", &def.name.to_string(), &self.types.display(def.repr), &format!("{:?}", def.lo), &format!("{:?}", def.hi), if def.inclusive { "..=" } else { ".." }])
            }
            _ => fnv(&[&self.types.display(ty)]),
        };
        seen.pop();
        hash
    }

    /// `[LAY-2]` — a type's layout attributes in its schema; nothing for a type without them, so
    /// every other schema hash is as it was.
    fn layout_schema(&self, ty: Ty) -> Option<String> {
        let attrs = self.types.layout_attrs(ty);
        (!attrs.is_none()).then(|| format!("layout packed={} align={}", attrs.packed, attrs.align))
    }

    /// A class's method tables' layout: each slot's method and its C type, and the interfaces it
    /// implements. Instances keep their data when it changes, but not their type information.
    fn methods_schema(&self, id: ClassId) -> u64 {
        let mut parts = Vec::new();
        if let Some(slots) = self.virtual_tables.get(&id) {
            for slot in slots {
                match slot {
                    Some(method) => {
                        parts.push(method.name.clone());
                        parts.extend(method.params.iter().map(|ty| self.c_type(*ty)));
                        parts.push(self.c_type(method.ret));
                    }
                    None => parts.push("-".to_string()),
                }
            }
        }
        for (interface, _) in self.class_itable_entries(id) {
            parts.push(interface);
        }
        fnv(&parts.iter().map(String::as_str).collect::<Vec<_>>())
    }

    /// A function's signature hash: its C type, which a slot keeps while its address may be held.
    fn signature_hash(&self, body: &Body) -> u64 {
        let ret = self.c_type(body.return_ty());
        let params = self.c_params(body);
        let mut parts = vec![ret];
        parts.extend(params.into_iter().map(|(ty, _)| ty));
        // `[HR-21]` — a C-ABI function's values cross by their layout: an aggregate whose layout
        // changed changes what the foreign side must pass, which only a rebuilt host does.
        if body.abi.is_some() {
            // A class crosses as its handle, whose own layout is not the foreign side's; a parameter
            // borrowed by default crosses by value.
            let crossing = |ty: Ty| {
                let ty = match self.types.kind(ty) {
                    TyKind::Ref { inner, .. } => *inner,
                    _ => ty,
                };
                match self.types.kind(ty) {
                    TyKind::Class(id) => fnv(&["handle", &self.types.class_def(*id).name.to_string()]).to_string(),
                    _ => self.type_schema(ty, &mut Vec::new()).to_string(),
                }
            };
            parts.push(crossing(body.return_ty()));
            parts.extend((1..=body.arg_count).filter_map(|index| body.locals.get(index)).map(|local| crossing(local.ty)));
        }
        fnv(&parts.iter().map(String::as_str).collect::<Vec<_>>())
    }

    /// `[HR-8]` — the manifest: every reloadable function by mangled name, every `@noreload` one
    /// with its body's hash, every static, every type's schema and every identity, with the
    /// protocol's version, in the image's own section. The tables it points at are the runtime's
    /// to fill.
    pub(crate) fn emit_reload_manifest(&mut self, bodies: &[Body], has_entry: bool) {
        // The main unit's alone: no other unit copies the image's description to write nothing.
        if self.unit.is_some() {
            return;
        }
        let Some(reload) = self.reload.clone() else { return };
        // `[HR-14]` — each range type's test, which a field's row names.
        let ranges: Vec<(ember_types::RangeId, ember_types::RangeDef)> = self.types.ranges().map(|(id, def)| (id, def.clone())).collect();
        for (id, def) in ranges {
            if let Some(body) = range_test_body(self.types.kind(def.repr), &def) {
                self.line(&format!("static inline int {}(const void* value) {{\n    {body}\n}}", range_test_symbol(id)));
            }
        }
        let address_taken = address_taken_symbols(bodies);
        let mut fns = Vec::new();
        for body in bodies {
            let Some(&index) = self.reload_slots.get(&body.symbol) else { continue };
            if fns.len() != index || !reload.reloadable.contains(&body.symbol) {
                continue;
            }
            let reloadable = true;
            let mut flags = if reloadable { "EMBER_RL_FN_RELOADABLE" } else { "EMBER_RL_FN_NORELOAD" }.to_string();
            if address_taken.contains(body.symbol.as_str()) || body.abi.is_some() {
                flags.push_str(" | EMBER_RL_FN_ADDRESS");
            }
            // `[HR-21]` — a host reaches an export through its permanent thunk.
            if body.abi.is_some() {
                flags.push_str(" | EMBER_RL_FN_EXPORT");
            }
            let (code, thunk) = if reloadable {
                (
                    format!("(void (*)(void)){}", Self::reload_body_symbol(&body.symbol)),
                    format!("(void (*)(void)){}", body.symbol),
                )
            } else {
                ("NULL".to_string(), "NULL".to_string())
            };
            let descriptor = match self.native_fn_values.get(&body.symbol) {
                Some(value) if reloadable => format!("&{}", value.descriptor),
                _ => "NULL".to_string(),
            };
            fns.push(format!(
                "    {{ {}, {code}, {thunk}, {descriptor}, UINT64_C(0x{:016x}), UINT64_C(0x{:016x}), {flags} }},",
                c_string_literal(&body.symbol),
                self.signature_hash(body),
                reload.noreload.get(&body.symbol).copied().unwrap_or(0),
            ));
        }
        let mut fixed: Vec<(&String, &u64)> = reload.noreload.iter().collect();
        fixed.sort_by_key(|(symbol, _)| self.reload_slots.get(*symbol).copied());
        for (symbol, hash) in fixed {
            if self.reload_slots.get(symbol) != Some(&fns.len()) {
                continue;
            }
            fns.push(format!(
                "    {{ {}, NULL, NULL, NULL, 0, UINT64_C(0x{hash:016x}), EMBER_RL_FN_NORELOAD }},",
                c_string_literal(symbol)
            ));
        }
        let stored = self.reload_static_names();
        // `[HR-15]` — what each stored static's value holds, as a field at offset 0.
        let mut shapes = Vec::new();
        for (index, (name, ty, _)) in stored.iter().enumerate() {
            let shape = self.field_shape(*ty, &reload);
            let symbol = table(&format!("shape_{index}"));
            let flags = if shape.flags.is_empty() { "0".to_string() } else { shape.flags.join(" | ") };
            self.line(&format!(
                "static const {RT}rl_field {symbol} = {{ {}, NULL, {}, UINT64_C(0x{:016x}), 0, (uint32_t)sizeof({}), {flags}, {} }};",
                c_string_literal(name),
                c_string_literal(&shape.type_name),
                self.type_schema(*ty, &mut Vec::new()),
                self.c_type(*ty),
                shape.tail(),
            ));
            shapes.push(format!("&{symbol}"));
        }
        let mut statics: Vec<String> = stored
            .clone()
            .into_iter()
            .enumerate()
            .map(|(index, (name, ty, init))| {
                let (initialiser, _, reinit) = reload.statics.get(&name).copied().unwrap_or((0, 0, false));
                let c_type = self.c_type(ty);
                let mut flags = Vec::new();
                if reinit {
                    flags.push("EMBER_RL_STATIC_REINIT");
                }
                let initial = if init.is_none() {
                    flags.push("EMBER_RL_STATIC_CONSTANT");
                    format!("&{}", constant_static_symbol(&name))
                } else {
                    "NULL".to_string()
                };
                let flags = if flags.is_empty() { "0".to_string() } else { flags.join(" | ") };
                format!(
                    "    {{ {}, (uint32_t)sizeof({c_type}), (uint32_t)_Alignof({c_type}), UINT64_C(0x{:016x}), UINT64_C(0x{:016x}), {flags}, {initial}, NULL, {} }},",
                    c_string_literal(&name),
                    self.type_schema(ty, &mut Vec::new()),
                    initialiser,
                    shapes[index],
                )
            })
            .collect();
        for (name, (initialiser, declared, reinit)) in &reload.statics {
            if stored.iter().any(|(known, _, _)| known == name) {
                continue;
            }
            // `[HR-17a]` — a folded static's code holds its value and nothing else: a value that
            // changed with its initialiser's text unchanged (a function it calls was edited) is a
            // changed initialiser too, refused unless the static is `@reinit_on_reload`.
            let value = self.types.folded_statics().iter().find(|(known, _)| known.as_str() == name).map(|(_, value)| value.as_str());
            let initialiser = &value.map_or(*initialiser, |value| fnv(&[&initialiser.to_string(), value]));
            statics.push(format!(
                "    {{ {}, 0, 1, UINT64_C(0x{declared:016x}), UINT64_C(0x{initialiser:016x}), {}, NULL, NULL, NULL }},",
                c_string_literal(name),
                if *reinit { "EMBER_RL_STATIC_REINIT" } else { "0" },
            ));
        }
        let abi = self.abi_types(bodies, &reload);
        let (types, mut identities) = self.emit_type_schemas(&reload, &abi, bodies);
        identities.extend(self.reload_interface_ids.borrow().iter().map(|id| format!("&{id}")));

        let fns_symbol = table("fns");
        let statics_symbol = table("statics");
        let types_symbol = table("types");
        let identities_symbol = table("identities");
        self.line("/* [HR-8] the reload manifest */");
        self.line(&format!("static const {RT}rl_fn {fns_symbol}[{}] = {{", fns.len().max(1)));
        for line in &fns {
            self.line(line);
        }
        if fns.is_empty() {
            self.line("    { NULL, NULL, NULL, NULL, 0, 0, 0 },");
        }
        self.line("};");
        self.line(&format!("static const {RT}rl_static {statics_symbol}[{}] = {{", statics.len().max(1)));
        for line in &statics {
            self.line(line);
        }
        if statics.is_empty() {
            self.line("    { NULL, 0, 0, 0, 0, 0, NULL, NULL, NULL },");
        }
        self.line("};");
        self.line(&format!("static const {RT}rl_type {types_symbol}[{}] = {{", types.len().max(1)));
        for line in &types {
            self.line(line);
        }
        if types.is_empty() {
            self.line("    { NULL, 0, NULL, 0, 0, 0, 0, 0, NULL, NULL, 0, NULL, 0, NULL, NULL, NULL, NULL, NULL, 0, NULL },");
        }
        self.line("};");
        self.line(&format!("static {RT}rl_identity* const {identities_symbol}[{}] = {{", identities.len().max(1)));
        self.line(&format!("    {}", if identities.is_empty() { "NULL".to_string() } else { identities.join(", ") }));
        self.line("};");
        let headers_symbol = table("headers");
        self.line(&format!("static const {RT}rl_header {headers_symbol}[{}] = {{", reload.headers.len().max(1)));
        for (name, hash) in &reload.headers {
            self.line(&format!("    {{ {}, {} }},", c_string_literal(name), c_string_literal(hash)));
        }
        if reload.headers.is_empty() {
            self.line("    { NULL, NULL },");
        }
        self.line("};");
        let layout = if reload.instances { "obj_header_v40" } else { "obj_header_v24" };
        let entry = if has_entry { table("entry") } else { "NULL".to_string() };
        self.line(&format!("EMBER_RL_MANIFEST const {RT}rl_manifest {} = {{", manifest_symbol()));
        self.line("    EMBER_RELOAD_MAGIC, EMBER_RELOAD_PROTOCOL,");
        self.line(&format!("    (uint32_t)sizeof({RT}rl_manifest), (uint32_t)sizeof({RT}obj_header), {}u,", reload.mode));
        self.line(&format!("    {},", c_string_literal(&reload.package)));
        self.line(&format!("    {}, {},", ember_branding::runtime(layout), ember_branding::runtime("reload_protocol_v1")));
        self.line(&format!("    {}u, {fns_symbol}, {}, {}, {},", fns.len(), table("targets"), table("addresses"), table("descriptors")));
        self.line(&format!("    {}u, {statics_symbol}, {}, {},", statics.len(), table("static_data"), table("static_state")));
        self.line(&format!("    {}u, {types_symbol},", types.len()));
        self.line(&format!("    {}u, {identities_symbol},", identities.len()));
        self.line(&format!("    {entry},"));
        self.line(&format!("    {}u, {headers_symbol}", reload.headers.len()));
        self.line("};");
        self.line("");
    }

    /// Whether a value of `ty` can hold a reference to an object, which a reload must follow.
    fn holds_objects(&self, ty: Ty, seen: &mut Vec<Ty>) -> bool {
        if seen.contains(&ty) {
            return false;
        }
        seen.push(ty);
        let holds = match self.types.kind(ty) {
            TyKind::Class(_) | TyKind::ClassInterface(_) | TyKind::Dyn { .. } | TyKind::Fn { .. } => true,
            TyKind::Struct(id) => {
                self.shared_inner_id(*id).is_some()
                    || self.weak_inner_id(*id).is_some()
                    || self.box_inner_id(*id).is_some_and(|inner| self.holds_objects(inner, seen))
                    || self.types.struct_def(*id).fields.iter().map(|field| field.ty).collect::<Vec<_>>().into_iter().any(|field| self.holds_objects(field, seen))
            }
            TyKind::Enum(id) => self
                .types
                .enum_def(*id)
                .variants
                .iter()
                .flat_map(|variant| variant.fields.iter().map(|field| field.ty))
                .collect::<Vec<_>>()
                .into_iter()
                .any(|field| self.holds_objects(field, seen)),
            TyKind::Vec { elem, .. } | TyKind::Array { elem, .. } => self.holds_objects(*elem, seen),
            TyKind::Tuple(items) => items.clone().into_iter().any(|item| self.holds_objects(item, seen)),
            _ => false,
        };
        seen.pop();
        holds
    }

    /// `[HR-13]` — the package structs a value of which sits inside something that is not a
    /// package type's field: an `Array`, a tuple, a payload without a schema, a standard type. A
    /// reload cannot yet migrate such values, so a change to one is refused. Only a holder whose
    /// values can outlive a call counts: one held by another type, a closure's captured state or a
    /// static's value; a tuple a function makes and takes apart (`(self, default)` in a `match`) is
    /// gone before any poll.
    fn contained_structs(&self, reload: &ReloadImage) -> BTreeSet<String> {
        // A struct's or an enum's: an enum's values a reload cannot reach keep their old
        // discriminants as surely as a struct's keep their old layout.
        let schema_struct = |ty: Ty| -> Option<String> {
            match self.types.kind(ty) {
                TyKind::Struct(id) if self.has_schema(ty, reload) => Some(self.types.struct_def(*id).name.to_string()),
                TyKind::Enum(id) if self.has_schema(ty, reload) => Some(self.types.enum_def(*id).name.to_string()),
                _ => None,
            }
        };
        let mut held: BTreeSet<Ty> = self.closure_values.values().map(|(_, env)| *env).collect();
        held.extend(self.reload_static_names().into_iter().map(|(_, ty, _)| ty));
        for (_, kind) in self.types.all() {
            match kind {
                TyKind::Vec { elem, .. } | TyKind::Array { elem, .. } => {
                    held.insert(*elem);
                }
                TyKind::Tuple(items) => held.extend(items.iter().copied()),
                TyKind::Struct(id) => held.extend(self.types.struct_def(*id).fields.iter().map(|field| field.ty)),
                TyKind::Enum(id) => held.extend(self.types.enum_def(*id).variants.iter().flat_map(|variant| variant.fields.iter().map(|field| field.ty))),
                TyKind::Class(id) => held.extend(self.types.class_def(*id).fields.iter().map(|field| field.ty)),
                _ => {}
            }
        }
        // What a value of `ty` holds by value: an `SoA[T]`'s columns are its `T`s.
        let members = |ty: Ty| -> Vec<Ty> {
            match self.types.kind(ty) {
                TyKind::Vec { elem, text: false } | TyKind::Array { elem, .. } => vec![*elem],
                TyKind::Tuple(items) => items.clone(),
                TyKind::Enum(id) => self.types.enum_def(*id).variants.iter().flat_map(|variant| variant.fields.iter().map(|field| field.ty)).collect(),
                TyKind::Struct(id) => match &self.types.struct_def(*id).origin {
                    Some((origin, args)) if matches!(origin.as_str(), "SoA" | "ArenaSoA") && args.len() == 1 => args.clone(),
                    _ => self.types.struct_def(*id).fields.iter().map(|field| field.ty).collect(),
                },
                TyKind::Class(id) => self.types.class_def(*id).fields.iter().map(|field| field.ty).collect(),
                _ => Vec::new(),
            }
        };
        // Where a reload cannot reach: inside a tuple, or a struct, an enum or a class with no
        // schema. A struct, an enum with a schema and an `Array` or a fixed array migrate what
        // they hold (`[HR-14]`, `[HR-13]`), unless they sit where a reload cannot reach themselves.
        // A value made an interface value (`Box[dyn I]`) is behind its interface's methods, which a
        // reload repoints to code built for the new layout; the runtime cannot tell its type there.
        let mut unreached: Vec<Ty> = self
            .interface_adapters
            .values()
            .map(|adapter| adapter.concrete)
            .filter(|concrete| !matches!(self.types.kind(*concrete), TyKind::Class(_)))
            .collect();
        for (ty, kind) in self.types.all() {
            // An instance of a class lives on the heap, held or not.
            if !held.contains(&ty) && !matches!(kind, TyKind::Class(_)) {
                continue;
            }
            let migrates = match kind {
                TyKind::Tuple(_) => false,
                TyKind::Class(id) => self.class_has_schema(*id, reload),
                TyKind::Struct(_) => self.has_schema(ty, reload) || self.soa_element(ty, reload).is_some(),
                TyKind::Enum(_) => self.has_schema(ty, reload),
                _ => true,
            };
            if !migrates {
                unreached.extend(members(ty));
            }
        }
        let mut seen = BTreeSet::new();
        let mut out = BTreeSet::new();
        while let Some(ty) = unreached.pop() {
            if !seen.insert(ty) {
                continue;
            }
            out.extend(schema_struct(ty));
            // A class is a handle: its own fields migrate with its instances.
            if !matches!(self.types.kind(ty), TyKind::Class(_)) {
                unreached.extend(members(ty));
            }
        }
        out
    }

    /// Whether a type mentions one the package declares, by value or through a container.
    fn mentions_package(&self, ty: Ty, reload: &ReloadImage, seen: &mut Vec<Ty>) -> bool {
        if seen.contains(&ty) {
            return false;
        }
        seen.push(ty);
        let mentions = match self.types.kind(ty) {
            TyKind::Class(id) => reload.files.contains(&self.types.class_def(*id).span.file),
            TyKind::Struct(id) => {
                let def = self.types.struct_def(*id);
                // An `SoA[T]` holds its `T`s in columns, behind its fields' pointers.
                let soa = match &def.origin {
                    Some((origin, args)) if origin.as_str() == "SoA" => args.clone(),
                    _ => Vec::new(),
                };
                reload.files.contains(&def.span.file)
                    || def.fields.iter().map(|field| field.ty).collect::<Vec<_>>().into_iter().any(|field| self.mentions_package(field, reload, seen))
                    || soa.into_iter().any(|element| self.mentions_package(element, reload, seen))
            }
            TyKind::Enum(id) => {
                let def = self.types.enum_def(*id);
                reload.files.contains(&def.span.file)
                    || def.variants.iter().flat_map(|variant| variant.fields.iter().map(|field| field.ty)).collect::<Vec<_>>().into_iter()
                        .any(|field| self.mentions_package(field, reload, seen))
            }
            TyKind::Vec { elem, .. } | TyKind::Array { elem, .. } => self.mentions_package(*elem, reload, seen),
            TyKind::Tuple(items) => items.clone().into_iter().any(|item| self.mentions_package(item, reload, seen)),
            _ => false,
        };
        seen.pop();
        mentions
    }

    /// `[HR-13b]` — the package types a raw pointer can reach: a value behind one may sit in
    /// foreign memory, which no reload can find, so a change to its schema is refused.
    fn pointed_structs(&self, reload: &ReloadImage, bodies: &[Body]) -> BTreeSet<String> {
        // `[HR-13]` — a pointer type that only an `SoA`'s own struct holds is its column, whose
        // values migrate through it: every other mention counts, in a type or a body's local.
        // A reference to a pointer is a place's (a column projected while it is read), not a
        // value's: the pointer it names is counted where it is held.
        let mut mentioned: BTreeSet<Ty> = bodies.iter().flat_map(|body| body.locals.iter().map(|local| local.ty)).collect();
        for (_, kind) in self.types.all() {
            match kind {
                TyKind::Struct(id) if matches!(&self.types.struct_def(*id).origin, Some((origin, _)) if origin.as_str() == "SoA") => {}
                TyKind::Struct(id) => mentioned.extend(self.types.struct_def(*id).fields.iter().map(|field| field.ty)),
                TyKind::Class(id) => mentioned.extend(self.types.class_def(*id).fields.iter().map(|field| field.ty)),
                TyKind::Enum(id) => mentioned.extend(self.types.enum_def(*id).variants.iter().flat_map(|variant| variant.fields.iter().map(|field| field.ty))),
                TyKind::Tuple(items) => mentioned.extend(items.iter().copied()),
                TyKind::Vec { elem, .. } | TyKind::Array { elem, .. } | TyKind::Ptr { inner: elem, .. } => {
                    mentioned.insert(*elem);
                }
                TyKind::Fn { params, ret, .. } => {
                    mentioned.extend(params.iter().map(|param| param.ty));
                    mentioned.insert(*ret);
                }
                _ => {}
            }
        }
        let mut out = BTreeSet::new();
        for (ty, kind) in self.types.all() {
            let TyKind::Ptr { inner, .. } = kind else { continue };
            if !mentioned.contains(&ty) {
                continue;
            }
            match self.types.kind(*inner) {
                TyKind::Struct(id) if self.has_schema(*inner, reload) => {
                    out.insert(self.types.struct_def(*id).name.to_string());
                }
                TyKind::Class(id) if reload.files.contains(&self.types.class_def(*id).span.file) => {
                    out.insert(self.types.class_def(*id).name.to_string());
                }
                _ => {}
            }
        }
        out
    }

    /// Whether a type has a schema in this image: a struct, a class or a unit enum the package
    /// declares, and `[HR-13a]` a standard-library struct holding the package's types (a
    /// `Mutex[Array[Particle]]`), whose schema the instantiating package emits so a reload can
    /// follow and migrate what it holds.
    fn has_schema(&self, ty: Ty, reload: &ReloadImage) -> bool {
        match self.types.kind(ty) {
            TyKind::Struct(id) => {
                let def = self.types.struct_def(*id);
                // An `SoA`'s struct is its columns' pointers: its elements migrate by their own
                // schema (`soa_element`), never by its fields.
                !is_compiler_struct(&def.name.to_string())
                    && !matches!(&def.origin, Some((origin, _)) if matches!(origin.as_str(), "SoA" | "ArenaSoA"))
                    && self.structural_type_emitted(ty)
                    && (reload.files.contains(&def.span.file) || self.mentions_package(ty, reload, &mut Vec::new()))
            }
            TyKind::Enum(id) => {
                let def = self.types.enum_def(*id);
                if self.types.enum_is_bare(*id) {
                    reload.files.contains(&def.span.file)
                } else {
                    // `[HR-15]` — a payload enum (`Option[MapSlot[int, Particle]]` in a `Map`) has
                    // a schema too, so a reload can follow the references its payloads hold.
                    self.types.option_niche(*id).is_none()
                        && (reload.files.contains(&def.span.file) || self.mentions_package(ty, reload, &mut Vec::new()))
                }
            }
            _ => false,
        }
    }

    /// What a field of type `ty` holds: its kind, its flags, its type's name, and for an `Array`
    /// its elements'.
    fn field_shape(&mut self, ty: Ty, reload: &ReloadImage) -> FieldShape {
        let mut shape = FieldShape::plain(self.types.display(ty));
        // `[HR-15]` — a tuple, a `Box` and a callable value are described by this image's own
        // tables (ODR-126); as an `Array`'s element too, through `inner`.
        if let Some(own) = self.described_shape(ty, reload) {
            return own;
        }
        let element = |this: &Self, elem: Ty| -> (&'static str, String) {
            match this.types.kind(elem) {
                TyKind::Class(_) => ("EMBER_RL_KIND_HANDLE", String::new()),
                TyKind::Enum(id) if this.types.option_niche(*id).is_some_and(|niche| matches!(this.types.kind(niche.payload), TyKind::Class(_))) => {
                    ("EMBER_RL_KIND_OPTION_HANDLE", String::new())
                }
                TyKind::Struct(id) if this.weak_inner_id(*id).is_some() => ("EMBER_RL_KIND_WEAK", String::new()),
                TyKind::Struct(id) if this.shared_inner_id(*id).is_some() => ("EMBER_RL_KIND_HANDLE", String::new()),
                TyKind::Struct(id) if this.has_schema(elem, reload) => {
                    ("EMBER_RL_KIND_STRUCT", this.types.struct_def(*id).name.to_string())
                }
                TyKind::Enum(id) if !this.types.enum_is_bare(*id) && this.has_schema(elem, reload) => {
                    ("EMBER_RL_KIND_PAYLOAD", this.types.enum_def(*id).name.to_string())
                }
                _ if this.holds_objects(elem, &mut Vec::new()) => ("EMBER_RL_KIND_OPAQUE", String::new()),
                _ => ("EMBER_RL_KIND_PLAIN", String::new()),
            }
        };
        match self.types.kind(ty) {
            TyKind::Int(_) => shape.flags.extend(["EMBER_RL_FIELD_SIGNED", "EMBER_RL_FIELD_ZERO"]),
            TyKind::Uint(_) => shape.flags.extend(["EMBER_RL_FIELD_UNSIGNED", "EMBER_RL_FIELD_ZERO"]),
            TyKind::Float(_) => shape.flags.extend(["EMBER_RL_FIELD_FLOAT", "EMBER_RL_FIELD_ZERO"]),
            TyKind::Bool => shape.flags.push("EMBER_RL_FIELD_ZERO"),
            // `[HR-14]` — a range type is its representation's number with a test of its range,
            // and never `Default` (`[RNG-8]`).
            TyKind::Range(id) => {
                let def = self.types.range_def(*id);
                let repr = self.types.kind(def.repr);
                if range_test_body(repr, def).is_some() {
                    shape.flags.push(match repr {
                        TyKind::Uint(_) => "EMBER_RL_FIELD_UNSIGNED",
                        TyKind::Float(_) => "EMBER_RL_FIELD_FLOAT",
                        _ => "EMBER_RL_FIELD_SIGNED",
                    });
                    shape.range = range_test_symbol(*id);
                }
            }
            TyKind::Class(_) => shape.kind = "EMBER_RL_KIND_HANDLE",
            TyKind::Vec { text: true, .. } => shape.flags.push("EMBER_RL_FIELD_ZERO"),
            TyKind::Vec { elem, text: false } => {
                let elem = *elem;
                shape.kind = "EMBER_RL_KIND_ARRAY";
                shape.flags.push("EMBER_RL_FIELD_ZERO");
                shape.element_size = format!("(uint32_t)sizeof({})", self.c_type(elem));
                let nested = matches!(self.types.kind(elem), TyKind::Vec { text: false, .. }) && self.holds_objects(elem, &mut Vec::new());
                if let Some(own) = self.described_shape(elem, reload) {
                    shape.element_kind = own.kind;
                    shape.inner = self.reload_shape_table(elem, reload);
                } else if self.element_migrates(elem, reload) {
                    // `[HR-13]` — the elements migrate through their container, by the element's
                    // schema, which the element's row names.
                    let (_, name) = element(self, elem);
                    let (inner, kind) = self.reload_element_table(elem, reload);
                    shape.element_kind = kind;
                    shape.element_type = name;
                    shape.inner = inner;
                } else if nested {
                    // An `Array` of `Array`s: each element described by this image's own table.
                    shape.inner = self.reload_shape_table(elem, reload);
                    shape.element_kind = "EMBER_RL_KIND_ARRAY";
                } else {
                    let (kind, name) = element(self, elem);
                    shape.element_kind = kind;
                    shape.element_type = name;
                }
            }
            TyKind::Enum(id) => {
                if self.types.option_niche(*id).is_some_and(|niche| matches!(self.types.kind(niche.payload), TyKind::Class(_))) {
                    shape.kind = "EMBER_RL_KIND_OPTION_HANDLE";
                } else if self.has_schema(ty, reload) {
                    shape.kind = if self.types.enum_is_bare(*id) { "EMBER_RL_KIND_ENUM" } else { "EMBER_RL_KIND_PAYLOAD" };
                    shape.type_name = self.types.enum_def(*id).name.to_string();
                } else if self.holds_objects(ty, &mut Vec::new()) {
                    // `[HR-15]` — no schema, but this image describes what it holds, so the
                    // references in it are followed (a standard type's, a `Map`'s slots).
                    shape = self.contained_shape(ty, reload);
                }
            }
            TyKind::Struct(id) => {
                if self.weak_inner_id(*id).is_some() {
                    shape.kind = "EMBER_RL_KIND_WEAK";
                } else if self.shared_inner_id(*id).is_some() {
                    shape.kind = "EMBER_RL_KIND_HANDLE";
                } else if let Some(element) = self.soa_element(ty, reload) {
                    // `[HR-13]` — an `SoA[T]`'s elements migrate through it, column by column.
                    shape.kind = "EMBER_RL_KIND_SOA";
                    shape.element_type = self.types.struct_def(element).name.to_string();
                    shape.layout = self.reload_soa_table(ty, element, reload);
                } else if self.has_schema(ty, reload) {
                    shape.kind = "EMBER_RL_KIND_STRUCT";
                    shape.type_name = self.types.struct_def(*id).name.to_string();
                } else if self.holds_objects(ty, &mut Vec::new()) {
                    // `[HR-15]` — described by this image, so the references in it are followed.
                    shape = self.contained_shape(ty, reload);
                }
            }
            // `[HR-13]` — a fixed array whose elements migrate holds them in place, each by the
            // element's row; any other is plain bytes, or opaque when it holds objects.
            TyKind::Array { elem, .. } if self.element_migrates(*elem, reload) && self.described_shape(*elem, reload).is_none() => {
                let elem = *elem;
                shape.kind = "EMBER_RL_KIND_FIXED";
                shape.element_size = format!("(uint32_t)sizeof({})", self.c_type(elem));
                let (_, name) = element(self, elem);
                let (inner, kind) = self.reload_element_table(elem, reload);
                shape.element_kind = kind;
                shape.element_type = name;
                shape.inner = inner;
            }
            _ if self.holds_objects(ty, &mut Vec::new()) => shape.kind = "EMBER_RL_KIND_OPAQUE",
            _ => {}
        }
        shape
    }

    /// `[HR-15]` — what a tuple, a `Box` or a callable value holds, when it holds objects: `None`
    /// for every other type, which `field_shape` describes by its schema's name.
    fn described_shape(&mut self, ty: Ty, reload: &ReloadImage) -> Option<FieldShape> {
        if !self.holds_objects(ty, &mut Vec::new()) {
            return None;
        }
        let boxed = |this: &Self, ty: Ty| match this.types.kind(ty) {
            TyKind::Struct(id) => this.box_inner_id(*id).filter(|inner| !matches!(this.types.kind(*inner), TyKind::Dyn { .. })),
            _ => None,
        };
        let pointee = match self.types.kind(ty) {
            TyKind::Struct(_) => boxed(self, ty),
            TyKind::Enum(id) => self.types.option_niche(*id).and_then(|niche| boxed(self, niche.payload)),
            _ => None,
        };
        let mut shape = FieldShape::plain(self.types.display(ty));
        match (self.types.kind(ty).clone(), pointee) {
            (_, Some(pointee)) => {
                shape.kind = "EMBER_RL_KIND_BOX";
                shape.inner = self.reload_shape_table(pointee, reload);
            }
            (TyKind::Tuple(_), None) => {
                shape.kind = "EMBER_RL_KIND_TUPLE";
                shape.layout = self.reload_layout_table(ty, reload);
            }
            (TyKind::Fn { abi: None, .. }, None) => shape.kind = "EMBER_RL_KIND_CALLABLE",
            _ => return None,
        }
        Some(shape)
    }

    /// `[HR-15]` — a value's description that names no type, for what a tuple, a `Box` or a
    /// closure's captured state holds (ODR-126): a struct is described by its fields here too, so
    /// the runtime reads the value by the layout of the image that made it, and an enum's variants
    /// are listed with their payloads. A type met again inside itself is `OPAQUE`: a reload that
    /// must move what it holds refuses ([HR-15a]).
    fn contained_shape(&mut self, ty: Ty, reload: &ReloadImage) -> FieldShape {
        let mut shape = FieldShape::plain(self.types.display(ty));
        if !self.holds_objects(ty, &mut Vec::new()) {
            return shape;
        }
        shape.kind = "EMBER_RL_KIND_OPAQUE";
        if self.reload_shapes_open.contains(&ty) {
            return shape;
        }
        if let Some(own) = self.described_shape(ty, reload) {
            return own;
        }
        match self.types.kind(ty).clone() {
            TyKind::Class(_) => shape.kind = "EMBER_RL_KIND_HANDLE",
            // A niche `Option` is its payload's bits, null for `None`, which a handle's or a `Box`'s
            // walk passes over.
            TyKind::Enum(id) => match self.types.option_niche(id) {
                Some(niche) => {
                    let own = self.contained_shape(niche.payload, reload);
                    if matches!(own.kind, "EMBER_RL_KIND_HANDLE" | "EMBER_RL_KIND_WEAK" | "EMBER_RL_KIND_BOX" | "EMBER_RL_KIND_OPTION_HANDLE") {
                        return FieldShape { type_name: shape.type_name, ..own };
                    }
                }
                None => {
                    shape.kind = "EMBER_RL_KIND_PAYLOAD";
                    shape.layout = self.reload_layout_table(ty, reload);
                }
            },
            TyKind::Struct(id) if self.weak_inner_id(id).is_some() => shape.kind = "EMBER_RL_KIND_WEAK",
            TyKind::Struct(id) if self.shared_inner_id(id).is_some() => shape.kind = "EMBER_RL_KIND_HANDLE",
            TyKind::Struct(id) if self.box_inner_id(id).is_none() => {
                shape.kind = "EMBER_RL_KIND_TUPLE";
                shape.layout = self.reload_layout_table(ty, reload);
            }
            TyKind::Vec { elem, text: false } => {
                let own = self.contained_shape(elem, reload);
                if own.kind != "EMBER_RL_KIND_PLAIN" && own.kind != "EMBER_RL_KIND_OPAQUE" {
                    shape.kind = "EMBER_RL_KIND_ARRAY";
                    shape.element_kind = own.kind;
                    shape.element_size = format!("(uint32_t)sizeof({})", self.c_type(elem));
                    shape.inner = self.reload_shape_table(elem, reload);
                }
            }
            _ => {}
        }
        shape
    }

    /// `[HR-11]`, `[HR-13a]` — whether a class has a schema in this image: one the package declares,
    /// or a standard-library class holding the package's types (an ECS storage of a component),
    /// whose schema the instantiating package emits so its instances migrate and are followed; or
    /// `[HR-15]` one holding references (an ECS `World`'s storages), so a reload follows them.
    fn class_has_schema(&self, id: ClassId, reload: &ReloadImage) -> bool {
        let def = self.types.class_def(id);
        reload.files.contains(&def.span.file)
            || (0..self.types.class_field_count(id)).filter_map(|index| self.types.class_field_at(id, index)).any(|field| {
                self.mentions_package(field.ty, reload, &mut Vec::new())
                    || self.soa_element(field.ty, reload).is_some()
                    || self.holds_objects(field.ty, &mut Vec::new())
            })
    }

    /// `[HR-13]` — whether a container's elements of type `ty` migrate through it: a struct or an
    /// enum with a schema, or a container of such.
    fn element_migrates(&self, ty: Ty, reload: &ReloadImage) -> bool {
        match self.types.kind(ty) {
            TyKind::Struct(id) => self.structural_type_emitted(ty) && self.box_inner_id(*id).is_none() && self.has_schema(ty, reload),
            TyKind::Enum(_) => self.has_schema(ty, reload),
            TyKind::Vec { elem, text: false } | TyKind::Array { elem, .. } => self.element_migrates(*elem, reload),
            _ => false,
        }
    }

    /// `[HR-13]` — the address of the row describing one element of type `ty` as a field at offset
    /// 0 is (`field_shape`), with the alignment its container's buffer has; emitted once. With it,
    /// the row's kind.
    fn reload_element_table(&mut self, ty: Ty, reload: &ReloadImage) -> (String, &'static str) {
        if let Some(known) = self.reload_elements.get(&ty) {
            return known.clone();
        }
        let mut shape = self.field_shape(ty, reload);
        let c_type = self.c_type(ty);
        shape.align = format!("(uint32_t)EMBER_VEC_ALIGN_OF({c_type})");
        let symbol = table(&format!("element_{}", self.reload_elements.len()));
        let flags = if shape.flags.is_empty() { "0".to_string() } else { shape.flags.join(" | ") };
        self.line(&format!(
            "static const {RT}rl_field {symbol} = {{ \"\", NULL, {}, UINT64_C(0x{:016x}), 0, (uint32_t)sizeof({c_type}), {flags}, {} }};",
            c_string_literal(&shape.type_name),
            self.type_schema(ty, &mut Vec::new()),
            shape.tail(),
        ));
        let known = (format!("&{symbol}"), shape.kind);
        self.reload_elements.insert(ty, known.clone());
        known
    }

    /// `[HR-13]` — an `SoA[T]` whose element `T` has a schema: `T`'s struct, which its columns
    /// split.
    fn soa_element(&self, ty: Ty, reload: &ReloadImage) -> Option<StructId> {
        let TyKind::Struct(id) = self.types.kind(ty) else { return None };
        let (origin, args) = self.types.struct_def(*id).origin.as_ref()?;
        if origin.as_str() != "SoA" || args.len() != 1 || !self.has_schema(args[0], reload) {
            return None;
        }
        match self.types.kind(args[0]) {
            TyKind::Struct(element) => Some(*element),
            _ => None,
        }
    }

    /// `[HR-13]` — the address of the description of an `SoA[T]`'s own struct, emitted once: each
    /// column's pointer (where it lives in the struct, the size and alignment of its values, and
    /// where its value sits in a whole `T`, `within`), then `len` and `cap`; and the function
    /// laying out its one block (`<SoA>_layout`), which a reload rebuilding the block calls.
    fn reload_soa_table(&mut self, ty: Ty, element: StructId, reload: &ReloadImage) -> String {
        if let Some(symbol) = self.reload_shapes.get(&(ty, true)) {
            return symbol.clone();
        }
        let TyKind::Struct(id) = *self.types.kind(ty) else { unreachable!("an `SoA` is a struct") };
        let soa_c = self.c_type(ty);
        let element_ty = self.types.find(&TyKind::Struct(element)).expect("an `SoA`'s element is interned");
        let element_c = self.c_type(element_ty);
        let mut rows = Vec::new();
        for column in self.types.soa_columns(element) {
            let mut structure = element;
            let mut path = Vec::new();
            for &index in &column.path {
                let field = &self.types.struct_def(structure).fields[index];
                path.push(member(field.name));
                if let TyKind::Struct(inner) = *self.types.kind(field.ty) {
                    structure = inner;
                }
            }
            let mut shape = self.field_shape(column.ty, reload);
            let column_c = self.c_type(column.ty);
            shape.align = format!("(uint32_t)_Alignof({column_c})");
            shape.within = format!("(uint32_t)offsetof({element_c}, {})", path.join("."));
            let flags = if shape.flags.is_empty() { "0".to_string() } else { shape.flags.join(" | ") };
            let pointer = member(Symbol::intern(&format!("col_{}", column.name)));
            rows.push(format!(
                "    {{ {}, NULL, {}, UINT64_C(0x{:016x}), (uint32_t)offsetof({soa_c}, {pointer}), (uint32_t)sizeof({column_c}), {flags}, {} }},",
                c_string_literal(&format!("col_{}", column.name)),
                c_string_literal(&shape.type_name),
                self.type_schema(column.ty, &mut Vec::new()),
                shape.tail(),
            ));
        }
        for count in ["len", "cap"] {
            rows.push(format!(
                "    {{ \"{count}\", NULL, \"usize\", 0, (uint32_t)offsetof({soa_c}, {count}), (uint32_t)sizeof(size_t), 0, EMBER_RL_KIND_PLAIN, EMBER_RL_KIND_PLAIN, 0, NULL, NULL, NULL, NULL, 0, 0 }},"
            ));
        }
        let symbol = table(&format!("soa_{}", self.reload_shapes.len()));
        self.line(&format!("static const {RT}rl_field {symbol}_fields[{}] = {{", rows.len()));
        for row in &rows {
            self.line(row);
        }
        self.line("};");
        let layout = format!("{}_layout", self.c_named(TypeNode::Struct(id)));
        self.line(&format!(
            "static const {RT}rl_type {symbol} = {{ {}, EMBER_RL_TYPE_STRUCT, NULL, 0, (uint32_t)sizeof({soa_c}), (uint32_t)_Alignof({soa_c}), UINT64_C(0x{:016x}), 0, NULL, NULL, {}u, {symbol}_fields, 0, NULL, NULL, NULL, NULL, NULL, 0, {layout} }};",
            c_string_literal(&self.types.display(ty)),
            self.type_schema(ty, &mut Vec::new()),
            rows.len(),
        ));
        let address = format!("&{symbol}");
        self.reload_shapes.insert((ty, true), address.clone());
        address
    }

    /// The address of a table describing one value of `ty` (`contained_shape`), emitted once.
    fn reload_shape_table(&mut self, ty: Ty, reload: &ReloadImage) -> String {
        if let Some(symbol) = self.reload_shapes.get(&(ty, false)) {
            return symbol.clone();
        }
        let shape = self.contained_shape(ty, reload);
        let symbol = table(&format!("value_{}", self.reload_shapes.len()));
        self.line(&format!(
            "static const {RT}rl_field {symbol} = {{ \"\", NULL, {}, UINT64_C(0x{:016x}), 0, (uint32_t)sizeof({}), 0, {} }};",
            c_string_literal(&shape.type_name),
            self.type_schema(ty, &mut Vec::new()),
            self.c_type(ty),
            shape.tail(),
        ));
        let address = format!("&{symbol}");
        self.reload_shapes.insert((ty, false), address.clone());
        address
    }

    /// The address of a table listing the fields of a tuple or a struct that hold objects, each
    /// described by `contained_shape`, emitted once: what a `TUPLE` value's walk reads.
    fn reload_layout_table(&mut self, ty: Ty, reload: &ReloadImage) -> String {
        if let Some(symbol) = self.reload_shapes.get(&(ty, true)) {
            return symbol.clone();
        }
        // A struct's or a tuple's members; an enum's, by variant: its discriminant, and each payload
        // field's name and its C member, `payload.<variant>.<field>`.
        let mut groups: Vec<(Option<(String, u64)>, Vec<(String, String, Ty)>)> = Vec::new();
        match self.types.kind(ty).clone() {
            TyKind::Tuple(items) => groups.push((None, items.into_iter().enumerate()
                .map(|(index, item)| (format!("_{index}"), format!("_{index}"), item)).collect())),
            TyKind::Struct(id) => groups.push((None, self.types.struct_def(id).fields.iter()
                .map(|field| (field.name.to_string(), member(field.name), field.ty)).collect())),
            TyKind::Enum(id) => {
                for variant in self.types.enum_def(id).variants.clone() {
                    let members = variant.fields.iter()
                        .map(|field| (field.name.to_string(), format!("payload.{}.{}", member(variant.name), member(field.name)), field.ty))
                        .collect();
                    groups.push((Some((variant.name.to_string(), variant.discriminant as u64)), members));
                }
            }
            _ => {}
        }
        self.reload_shapes_open.insert(ty);
        let c_type = self.c_type(ty);
        let mut tables = Vec::new();
        let mut described = Vec::new();
        for (index, (variant, members)) in groups.into_iter().enumerate() {
            let mut rows = Vec::new();
            for (name, member, item) in members {
                if self.is_void(item) || !self.holds_objects(item, &mut Vec::new()) {
                    continue;
                }
                let shape = self.contained_shape(item, reload);
                rows.push(format!(
                    "    {{ {}, NULL, {}, UINT64_C(0x{:016x}), (uint32_t)offsetof({c_type}, {member}), (uint32_t)sizeof((({c_type}*)0)->{member}), 0, {} }},",
                    c_string_literal(&name),
                    c_string_literal(&shape.type_name),
                    self.type_schema(item, &mut Vec::new()),
                    shape.tail(),
                ));
            }
            described.push((variant, index, rows));
        }
        // Named once what it holds is described: a table emitted for a member (a type met inside
        // this one) takes a name of its own first.
        let symbol = table(&format!("layout_{}", self.reload_shapes.len()));
        for (variant, index, rows) in described {
            tables.push((variant, format!("{symbol}_fields_{index}"), rows));
        }
        self.reload_shapes_open.remove(&ty);
        for (_, name, rows) in &tables {
            if !rows.is_empty() {
                self.line(&format!("static const {RT}rl_field {name}[{}] = {{", rows.len()));
                for row in rows {
                    self.line(row);
                }
                self.line("};");
            }
        }
        let reference = |name: &String, rows: &Vec<String>| if rows.is_empty() { "NULL".to_string() } else { name.clone() };
        let (fields, variants) = if tables.iter().any(|(variant, _, _)| variant.is_some()) {
            // `[HR-15]` — an enum with payloads: its tag as its one field, and each variant's payload,
            // which the walk reads for the variant the tag names.
            self.line(&format!(
                "static const {RT}rl_field {symbol}_tag[1] = {{ {{ \"tag\", NULL, \"tag\", 0, (uint32_t)offsetof({c_type}, tag), (uint32_t)sizeof((({c_type}*)0)->tag), 0, EMBER_RL_KIND_PLAIN, EMBER_RL_KIND_PLAIN, 0, NULL, NULL, NULL, NULL, 0, 0 }} }};"
            ));
            self.line(&format!("static const {RT}rl_variant {symbol}_variants[{}] = {{", tables.len()));
            for (variant, name, rows) in &tables {
                let (variant, discriminant) = variant.clone().expect("every group of an enum is a variant");
                self.line(&format!("    {{ {}, NULL, UINT64_C({discriminant}), 0, {}u, {} }},", c_string_literal(&variant), rows.len(), reference(name, rows)));
            }
            self.line("};");
            (format!("1u, {symbol}_tag"), format!("{}u, {symbol}_variants", tables.len()))
        } else {
            let (_, name, rows) = &tables[0];
            (format!("{}u, {}", rows.len(), reference(name, rows)), "0, NULL".to_string())
        };
        self.line(&format!(
            "static const {RT}rl_type {symbol} = {{ {}, EMBER_RL_TYPE_STRUCT, NULL, 0, (uint32_t)sizeof({c_type}), (uint32_t)_Alignof({c_type}), UINT64_C(0x{:016x}), 0, NULL, NULL, {fields}, {variants}, NULL, NULL, NULL, NULL, 0, NULL }};",
            c_string_literal(&self.types.display(ty)),
            self.type_schema(ty, &mut Vec::new()),
        ));
        let address = format!("&{symbol}");
        self.reload_shapes.insert((ty, true), address.clone());
        address
    }

    /// `[CLO-10]`, `[HR-15]` — in a reloadable image, what a closure's descriptor says its
    /// captured state holds: `NULL` when it holds no object.
    pub(crate) fn reload_captures(&mut self, env: Ty) -> String {
        if self.reload.is_none() || !self.holds_objects(env, &mut Vec::new()) {
            return "NULL".to_string();
        }
        let reload = self.reload.clone().expect("a reloadable image");
        self.reload_shape_table(env, &reload)
    }

    /// `[HR-11]` — every package type's schema: a class's and a struct's fields (name, renamed-from
    /// name, type, schema hash, offset, size, what it holds), a unit enum's variants; and, for a
    /// type with declared field defaults, the function that writes one into a new instance
    /// (`[HR-14]`). The entries for the manifest's type table, and every identity.
    /// `[HR-14]` — the package types that cross a foreign boundary by value: a parameter or the
    /// result of a function with a C ABI (an export, a foreign declaration, a C function pointer's
    /// type), and what such a value holds by value. The foreign side was compiled for their layout
    /// attributes, so a change to those is refused.
    fn abi_types(&self, bodies: &[Body], reload: &ReloadImage) -> BTreeSet<String> {
        let mut roots: Vec<Ty> = Vec::new();
        for body in bodies.iter().filter(|body| body.abi.is_some() || body.is_extern_declaration) {
            roots.push(body.return_ty());
            roots.extend((1..=body.arg_count).filter_map(|index| body.locals.get(index).map(|local| local.ty)));
        }
        for (_, kind) in self.types.all() {
            if let TyKind::Fn { abi: Some(_), params, ret, .. } = kind {
                roots.extend(params.iter().map(|param| param.ty));
                roots.push(*ret);
            }
        }
        let mut out = BTreeSet::new();
        let mut seen = BTreeSet::new();
        while let Some(ty) = roots.pop() {
            if !seen.insert(ty) {
                continue;
            }
            match self.types.kind(ty) {
                TyKind::Struct(id) if self.has_schema(ty, reload) => {
                    out.insert(self.types.struct_def(*id).name.to_string());
                    roots.extend(self.types.struct_def(*id).fields.iter().map(|field| field.ty));
                }
                TyKind::Enum(id) if self.has_schema(ty, reload) => {
                    out.insert(self.types.enum_def(*id).name.to_string());
                    roots.extend(self.types.enum_def(*id).variants.iter().flat_map(|variant| variant.fields.iter().map(|field| field.ty)));
                }
                TyKind::Array { elem, .. } => roots.push(*elem),
                TyKind::Tuple(items) => roots.extend(items.iter().copied()),
                // A parameter borrowed by default crosses by value, and one the foreign side reads
                // through a pointer relies on the layout as much.
                TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. } => roots.push(*inner),
                _ => {}
            }
        }
        out
    }

    /// `[HR-14]`, `[LAY-2]` — a type's layout attributes alone, hashed: 0 for none.
    fn attributes_hash(&self, ty: Ty) -> u64 {
        let repr = match self.types.kind(ty) {
            TyKind::Enum(id) => self.types.display(self.types.enum_def(*id).repr),
            _ => String::new(),
        };
        match self.layout_schema(ty) {
            None if repr.is_empty() => 0,
            layout => fnv(&[&repr, layout.as_deref().unwrap_or("")]),
        }
    }

    fn emit_type_schemas(&mut self, reload: &ReloadImage, abi: &BTreeSet<String>, bodies: &[Body]) -> (Vec<String>, Vec<String>) {
        let mut types = Vec::new();
        let mut identities = Vec::new();
        let contained = self.contained_structs(reload);
        let foreign = self.pointed_structs(reload, bodies);
        // A field's row and, when it has a declared default, its default's case; each type with its
        // mangled form (`[MNG-1]`), which its tables are named by, and the type itself.
        let mut layouts: Vec<(Option<ClassId>, String, String, Vec<(String, Ty)>, String, Option<Ty>)> = Vec::new();
        let classes: Vec<ClassId> = self.types.runtime_classes().map(|(id, _)| id).collect();
        for id in classes {
            let def = self.types.class_def(id);
            let name = def.name.to_string();
            let mangled = self.mangled_type(TypeNode::Class(id));
            identities.push(format!("&{}", class_identity_symbol(&mangled)));
            if !self.class_has_schema(id, reload) {
                continue;
            }
            let fields: Vec<(String, Ty)> = (0..self.types.class_field_count(id))
                .filter_map(|index| self.types.class_field_at(id, index))
                .map(|field| (field.name.to_string(), field.ty))
                .collect();
            let ty = self.types.find(&TyKind::Class(id));
            layouts.push((Some(id), name.clone(), mangled.clone(), fields, format!("struct {}", ember_branding::object_struct(&mangled)), ty));
        }
        let structs: Vec<(StructId, String)> = self.types.structs().map(|(id, def)| (id, def.name.to_string())).collect();
        for (id, name) in structs {
            let Some(ty) = self.types.find(&TyKind::Struct(id)) else { continue };
            if !self.has_schema(ty, reload) {
                continue;
            }
            let fields: Vec<(String, Ty)> =
                self.types.struct_def(id).fields.iter().map(|field| (field.name.to_string(), field.ty)).collect();
            layouts.push((None, name, self.mangled_type(TypeNode::Struct(id)), fields, self.c_type(ty), Some(ty)));
        }
        for (class, name, mangled, fields, c_type, ty) in layouts {
            let table_name = table(&format!("fields_{mangled}"));
            let init_name = table(&format!("init_{mangled}"));
            let mut rows = Vec::new();
            let mut cases = Vec::new();
            for (index, (field, ty)) in fields.iter().enumerate() {
                let c_field = member(field);
                let shape = self.field_shape(*ty, reload);
                let mut flags = shape.flags.clone();
                // A declared default's function, or `[HR-14]` a `Default` type's, which the checker
                // built only where the type is `Default`.
                if let Some(symbol) = reload.defaults.get(&(name.clone(), field.clone())).and_then(|function| self.reload_function_symbol(function)) {
                    flags.push("EMBER_RL_FIELD_DEFAULT");
                    let member_type = self.c_member_type(*ty);
                    cases.push(format!(
                        "    case {index}u: *({member_type}*)((unsigned char*)object + offsetof({c_type}, {c_field})) = {symbol}(); break;"
                    ));
                }
                let renamed = reload.renamed.get(&(name.clone(), field.clone())).map_or("NULL".to_string(), |old| c_string_literal(old));
                let flags = if flags.is_empty() { "0".to_string() } else { flags.join(" | ") };
                rows.push(format!(
                    "    {{ {}, {renamed}, {}, UINT64_C(0x{:016x}), (uint32_t)offsetof({c_type}, {c_field}), (uint32_t)sizeof((({c_type}*)0)->{c_field}), {flags}, {} }},",
                    c_string_literal(field),
                    c_string_literal(&shape.type_name),
                    self.type_schema(*ty, &mut Vec::new()),
                    shape.tail(),
                ));
            }
            if !rows.is_empty() {
                self.line(&format!("static const {RT}rl_field {table_name}[{}] = {{", rows.len()));
                for row in &rows {
                    self.line(row);
                }
                self.line("};");
            }
            let init = if cases.is_empty() {
                "NULL".to_string()
            } else {
                self.line(&format!("static void {init_name}(void* object, uint32_t index) {{"));
                self.line("    switch (index) {");
                for case in &cases {
                    self.line(case);
                }
                self.line("    default: break;");
                self.line("    }");
                self.line("}");
                init_name.clone()
            };
            let fields_ref = if rows.is_empty() { "NULL".to_string() } else { table_name };
            let migrate = self.reload_migrate_adapter(&name, &mangled, reload);
            let old_type = reload.migrations.get(&name).map_or("NULL".to_string(), |(_, old)| c_string_literal(old));
            // `[HR-2a]` — reclaim drops the fields an edit removed, one by one.
            let drop_name = table(&format!("drop_{mangled}"));
            let mut drops = Vec::new();
            for (index, (field, ty)) in fields.iter().enumerate() {
                let c_field = member(field);
                if !self.types.needs_drop(*ty) {
                    continue;
                }
                let access = format!("(*({}*)((unsigned char*)object + offsetof({c_type}, {c_field})))", self.c_member_type(*ty));
                let mut lines = Vec::new();
                match self.drop_glue_call(&access, *ty) {
                    Some(call) => lines.push(call),
                    None => self.drop_lines(&access, *ty, &mut lines),
                }
                drops.push((index, lines));
            }
            let drop_field = if drops.is_empty() {
                "NULL".to_string()
            } else {
                self.line(&format!("static void {drop_name}(void* object, uint32_t index) {{"));
                self.line("    switch (index) {");
                for (index, lines) in drops {
                    self.line(&format!("    case {index}u: {{"));
                    for line in lines {
                        self.line(&format!("        {line}"));
                    }
                    self.line("        break;");
                    self.line("    }");
                }
                self.line("    default: break;");
                self.line("    }");
                self.line("}");
                drop_name
            };
            let mut layout = match (class.is_none() && contained.contains(&name), foreign.contains(&name)) {
                (_, true) => "EMBER_RL_LAYOUT_FOREIGN",
                (true, false) => "EMBER_RL_LAYOUT_CONTAINED",
                (false, false) => "0",
            }
            .to_string();
            if abi.contains(&name) {
                layout = if layout == "0" { "EMBER_RL_LAYOUT_ABI".to_string() } else { format!("{layout} | EMBER_RL_LAYOUT_ABI") };
            }
            let schema = ty.map_or(0, |ty| self.type_schema(ty, &mut Vec::new()));
            if let Some(id) = class {
                let def = self.types.class_def(id);
                let base = def.base.map_or("NULL".to_string(), |base| c_string_literal(&self.types.class_def(base).name.to_string()));
                types.push(format!(
                    "    {{ {}, EMBER_RL_TYPE_CLASS, {base}, 0, (uint32_t)sizeof({c_type}), (uint32_t)_Alignof({c_type}), UINT64_C(0x{schema:016x}), UINT64_C(0x{:016x}), &{}, &{}, {}u, {fields_ref}, 0, NULL, {init}, {migrate}, {old_type}, {drop_field}, 0, NULL }},",
                    c_string_literal(&name),
                    self.methods_schema(id),
                    ember_branding::type_info(&mangled),
                    class_identity_symbol(&mangled),
                    rows.len(),
                ));
            } else {
                types.push(format!(
                    "    {{ {}, EMBER_RL_TYPE_STRUCT, NULL, {layout}, (uint32_t)sizeof({c_type}), (uint32_t)_Alignof({c_type}), UINT64_C(0x{schema:016x}), 0, NULL, NULL, {}u, {fields_ref}, 0, NULL, {init}, {migrate}, {old_type}, {drop_field}, UINT64_C(0x{:016x}), NULL }},",
                    c_string_literal(&name),
                    rows.len(),
                    ty.map_or(0, |ty| self.attributes_hash(ty)),
                ));
            }
        }
        let enums: Vec<(EnumId, String)> = self.types.enums().map(|(id, def)| (id, def.name.to_string())).collect();
        for (id, name) in enums {
            let Some(ty) = self.types.find(&TyKind::Enum(id)) else { continue };
            let def = self.types.enum_def(id).clone();
            if !reload.files.contains(&def.span.file) && !self.has_schema(ty, reload) {
                continue;
            }
            let c_type = self.c_type(ty);
            let mangled = self.mangled_type(TypeNode::Enum(id));
            // `[HR-11]`, `[HR-15]` — a payload enum's schema: its tag as its one field, and each
            // variant's payload fields, which a reload walks for the variant the tag names.
            let payloads = !self.types.enum_is_bare(id) && self.has_schema(ty, reload);
            let tag_name = table(&format!("tag_{mangled}"));
            if payloads {
                self.line(&format!(
                    "static const {RT}rl_field {tag_name}[1] = {{ {{ \"tag\", NULL, \"tag\", 0, (uint32_t)offsetof({c_type}, tag), (uint32_t)sizeof((({c_type}*)0)->tag), 0, EMBER_RL_KIND_PLAIN, EMBER_RL_KIND_PLAIN, 0, NULL, NULL, NULL, NULL, 0, 0 }} }};"
                ));
            }
            let mut rows = Vec::new();
            // `[HR-14]` — a payload's fields migrate by the rows a type's take: each one's default
            // (its type's `Default`, `init_field`) and its drop (`drop_field`) are numbered over
            // every variant's payload fields in turn.
            let mut cases = Vec::new();
            let mut drops = Vec::new();
            let mut numbered = 0u32;
            for variant in &def.variants {
                let renamed = reload.renamed.get(&(name.clone(), variant.name.to_string())).map_or("NULL".to_string(), |old| c_string_literal(old));
                let payload: Vec<String> = variant.fields.iter().map(|field| self.type_schema(field.ty, &mut Vec::new()).to_string()).collect();
                let mut fields = Vec::new();
                let held: Vec<_> = variant.fields.iter().filter(|field| payloads && !self.is_void(field.ty)).cloned().collect();
                for field in &held {
                    let shape = self.field_shape(field.ty, reload);
                    let member = format!("payload.{}.{}", member(variant.name), member(field.name));
                    let mut flags = shape.flags.clone();
                    let key = (name.clone(), format!("{}.{}", variant.name, field.name));
                    if let Some(symbol) = reload.defaults.get(&key).and_then(|function| self.reload_function_symbol(function)) {
                        flags.push("EMBER_RL_FIELD_DEFAULT");
                        let member_type = self.c_member_type(field.ty);
                        cases.push(format!("    case {numbered}u: *({member_type}*)((unsigned char*)object + offsetof({c_type}, {member})) = {symbol}(); break;"));
                    }
                    if self.types.needs_drop(field.ty) {
                        let access = format!("(*({}*)((unsigned char*)object + offsetof({c_type}, {member})))", self.c_member_type(field.ty));
                        let mut lines = Vec::new();
                        match self.drop_glue_call(&access, field.ty) {
                            Some(call) => lines.push(call),
                            None => self.drop_lines(&access, field.ty, &mut lines),
                        }
                        drops.push((numbered, lines));
                    }
                    numbered += 1;
                    let flags = if flags.is_empty() { "0".to_string() } else { flags.join(" | ") };
                    let renamed = reload.renamed.get(&key).map_or("NULL".to_string(), |old| c_string_literal(old));
                    fields.push(format!(
                        "    {{ {}, {renamed}, {}, UINT64_C(0x{:016x}), (uint32_t)offsetof({c_type}, {member}), (uint32_t)sizeof((({c_type}*)0)->{member}), {flags}, {} }},",
                        c_string_literal(&field.name.to_string()),
                        c_string_literal(&shape.type_name),
                        self.type_schema(field.ty, &mut Vec::new()),
                        shape.tail(),
                    ));
                }
                let payload_table = table(&format!("payload_{mangled}_{}", ember_branding::mangled_path(variant.name.as_str())));
                if !fields.is_empty() {
                    self.line(&format!("static const {RT}rl_field {payload_table}[{}] = {{", fields.len()));
                    for row in &fields {
                        self.line(row);
                    }
                    self.line("};");
                }
                rows.push(format!(
                    "    {{ {}, {renamed}, UINT64_C({}), UINT64_C(0x{:016x}), {}u, {} }},",
                    c_string_literal(&variant.name.to_string()),
                    variant.discriminant as u64,
                    fnv(&payload.iter().map(String::as_str).collect::<Vec<_>>()),
                    fields.len(),
                    if fields.is_empty() { "NULL".to_string() } else { payload_table },
                ));
            }
            let table_name = table(&format!("variants_{mangled}"));
            if !rows.is_empty() {
                self.line(&format!("static const {RT}rl_variant {table_name}[{}] = {{", rows.len()));
                for row in &rows {
                    self.line(row);
                }
                self.line("};");
            }
            let init = if cases.is_empty() {
                "NULL".to_string()
            } else {
                let init_name = table(&format!("init_{mangled}"));
                self.line(&format!("static void {init_name}(void* object, uint32_t index) {{\n    switch (index) {{"));
                for case in &cases {
                    self.line(case);
                }
                self.line("    default: break;\n    }\n}");
                init_name
            };
            let drop_field = if drops.is_empty() {
                "NULL".to_string()
            } else {
                let drop_name = table(&format!("drop_{mangled}"));
                self.line(&format!("static void {drop_name}(void* object, uint32_t index) {{\n    switch (index) {{"));
                for (index, lines) in drops {
                    self.line(&format!("    case {index}u: {{"));
                    for line in lines {
                        self.line(&format!("        {line}"));
                    }
                    self.line("        break;\n    }");
                }
                self.line("    default: break;\n    }\n}");
                drop_name
            };
            let mut layout = if self.types.enum_is_bare(id) { "EMBER_RL_LAYOUT_UNIT" } else { "0" }.to_string();
            for (flag, holds) in [("EMBER_RL_LAYOUT_ABI", abi.contains(&name)), ("EMBER_RL_LAYOUT_CONTAINED", contained.contains(&name))] {
                if holds {
                    layout = if layout == "0" { flag.to_string() } else { format!("{layout} | {flag}") };
                }
            }
            let attributes = self.attributes_hash(ty);
            types.push(format!(
                "    {{ {}, EMBER_RL_TYPE_ENUM, NULL, {}, (uint32_t)sizeof({c_type}), (uint32_t)_Alignof({c_type}), UINT64_C(0x{:016x}), 0, NULL, NULL, {}, {}u, {}, {init}, NULL, NULL, {drop_field}, UINT64_C(0x{attributes:016x}), NULL }},",
                c_string_literal(&name),
                layout,
                self.type_schema(ty, &mut Vec::new()),
                if payloads { format!("1u, {tag_name}") } else { "0, NULL".to_string() },
                rows.len(),
                if rows.is_empty() { "NULL".to_string() } else { table_name },
            ));
        }
        (types, identities)
    }

    /// A function the driver wrote for the reload (a field's default, a migration's wrapper),
    /// found by its name in the package: its symbol and its return type.
    fn reload_function(&self, name: &str) -> Option<(String, Ty)> {
        let dotted = format!(".{name}");
        self.reload_bodies
            .iter()
            .find(|(body_name, _, _)| body_name == name || body_name.ends_with(&dotted))
            .map(|(_, symbol, ret)| (symbol.clone(), *ret))
    }

    fn reload_function_symbol(&self, name: &str) -> Option<String> {
        self.reload_function(name).map(|(symbol, _)| symbol)
    }

    /// `[HR-16]` — the C adapter between the runtime and a type's `migrate_from`: the runtime builds
    /// the old value as `Old<Name>` from the old instance, field by name, and passes its address;
    /// the driver's wrapper (`fn(old: ref Old<Name>, report: mut String) -> Option[<Name>]`) calls
    /// `migrate_from` and gives the new value, or `None` with the `ReloadError` written into the
    /// report. 0 is success. "NULL" for a type without `migrate_from`.
    fn reload_migrate_adapter(&mut self, name: &str, mangled: &str, reload: &ReloadImage) -> String {
        let Some((wrapper, _)) = reload.migrations.get(name) else { return "NULL".to_string() };
        let Some((symbol, ret)) = self.reload_function(wrapper) else { return "NULL".to_string() };
        let TyKind::Enum(option) = *self.types.kind(ret) else { return "NULL".to_string() };
        let def = self.types.enum_def(option);
        let Some(some) = def.variants.iter().position(|variant| variant.name.as_str() == "Some") else { return "NULL".to_string() };
        let discriminant = def.variants[some].discriminant;
        let value = self.c_type(def.variants[some].fields[0].ty);
        let result = self.c_type(ret);
        let tag = self.enum_tag(option, "result");
        let payload = self.enum_member(option, "result", some, 0);
        let adapter = table(&format!("migrate_{mangled}"));
        self.line(&format!("static int {adapter}(void* out, const void* old, char* report, size_t report_size) {{"));
        self.line(&format!("    {RT}vec text = {RT}vec_empty();"));
        self.line(&format!("    {result} result = {symbol}((void*)old, &text);"));
        self.line(&format!("    int done = {tag} == {discriminant};"));
        self.line(&format!("    if (done) {{ *({value}*)out = {payload}; }}"));
        self.line("    else if (report_size > 0) {");
        self.line("        size_t n = text.len < report_size - 1 ? text.len : report_size - 1;");
        self.line("        if (n > 0) { memcpy(report, text.ptr, n); }");
        self.line("        report[n] = '\\0';");
        self.line("    }");
        self.line(&format!("    {RT}vec_free(&text, 1);"));
        self.line("    return done ? 0 : 1;");
        self.line("}");
        adapter
    }

    /// Whether the program defines this struct in its C (not a view, an alias or an erased one).
    fn structural_type_emitted(&self, ty: Ty) -> bool {
        let TyKind::Struct(id) = self.types.kind(ty) else { return false };
        self.box_inner_id(*id).is_none() && self.shared_inner_id(*id).is_none() && self.weak_inner_id(*id).is_none()
    }
}

/// A class's identity record across images, by the class's mangled form (`[MNG-1]`).
pub(crate) fn class_identity_symbol(mangled_class: &str) -> String {
    format!("{}_{mangled_class}", ember_branding::generated("tid"))
}

/// A struct the compiler supplies, which no edit changes.
fn is_compiler_struct(name: &str) -> bool {
    ["Box_", "Shared_", "Weak_", "Cell_", "RefCell_", "UnsafeCell_", "MaybeUninit_", "closure", "Option_", "Result_"]
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

/// `[HR-7]` — the functions whose address the program takes: a function value, a table a `dyn`
/// value points at. Data may hold their thunks, so removing one is refused.
fn address_taken_symbols(bodies: &[Body]) -> BTreeSet<&str> {
    let mut taken = BTreeSet::new();
    for body in bodies {
        for block in &body.blocks {
            for stmt in &block.stmts {
                if let StmtKind::Assign { rvalue, .. } = &stmt.kind {
                    match rvalue {
                        Rvalue::Use(Operand::Const(Const::Fn(symbol))) => {
                            taken.insert(symbol.as_str());
                        }
                        Rvalue::Aggregate { operands, .. } => {
                            for operand in operands {
                                if let Operand::Const(Const::Fn(symbol)) = operand {
                                    taken.insert(symbol.as_str());
                                }
                            }
                        }
                        Rvalue::Cast { kind: CastKind::InterfaceUpcast { implementations, .. } | CastKind::ClassInterfaceUpcast { implementations, .. }, .. } => {
                            taken.extend(implementations.iter().flatten().map(|method| method.symbol.as_str()));
                        }
                        _ => {}
                    }
                }
            }
            if let Terminator::Call { func, args, .. } = &block.terminator {
                if let FuncRef::DynBoxNew { implementations, .. } = func {
                    taken.extend(implementations.iter().flatten().map(|method| method.symbol.as_str()));
                }
                for arg in args {
                    if let Operand::Const(Const::Fn(symbol)) = arg {
                        taken.insert(symbol.as_str());
                    }
                }
            }
        }
    }
    taken
}
