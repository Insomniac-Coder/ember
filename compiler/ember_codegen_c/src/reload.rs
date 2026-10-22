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
        }
    }

    /// The row's members after `flags`: what it holds, its elements', and its own tables.
    fn tail(&self) -> String {
        let element_type = if self.element_type.is_empty() { "NULL".to_string() } else { c_string_literal(&self.element_type) };
        format!("{}, {}, {}, {element_type}, {}, {}", self.kind, self.element_kind, self.element_size, self.layout, self.inner)
    }
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
        fnv(&parts.iter().map(String::as_str).collect::<Vec<_>>())
    }

    /// `[HR-8]` — the manifest: every reloadable function by mangled name, every `@noreload` one
    /// with its body's hash, every static, every type's schema and every identity, with the
    /// protocol's version, in the image's own section. The tables it points at are the runtime's
    /// to fill.
    pub(crate) fn emit_reload_manifest(&mut self, bodies: &[Body], has_entry: bool) {
        let Some(reload) = self.reload.clone() else { return };
        if self.unit.is_some() {
            return;
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
            statics.push(format!(
                "    {{ {}, 0, 1, UINT64_C(0x{declared:016x}), UINT64_C(0x{initialiser:016x}), {}, NULL, NULL, NULL }},",
                c_string_literal(name),
                if *reinit { "EMBER_RL_STATIC_REINIT" } else { "0" },
            ));
        }
        let (types, mut identities) = self.emit_type_schemas(&reload);
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
            self.line("    { NULL, 0, NULL, 0, 0, 0, 0, 0, NULL, NULL, 0, NULL, 0, NULL, NULL, NULL, NULL, NULL },");
        }
        self.line("};");
        self.line(&format!("static {RT}rl_identity* const {identities_symbol}[{}] = {{", identities.len().max(1)));
        self.line(&format!("    {}", if identities.is_empty() { "NULL".to_string() } else { identities.join(", ") }));
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
        self.line(&format!("    {entry}"));
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
    /// package type's field: an `Array`, an `Option`, a tuple, a payload, a standard type. A reload
    /// cannot yet migrate such values, so a change to one is refused.
    fn contained_structs(&self, reload: &ReloadImage) -> BTreeSet<String> {
        let schema_struct = |ty: Ty| -> Option<String> {
            match self.types.kind(ty) {
                TyKind::Struct(id) if self.has_schema(ty, reload) => Some(self.types.struct_def(*id).name.to_string()),
                _ => None,
            }
        };
        let mut out = BTreeSet::new();
        for (ty, kind) in self.types.all() {
            let inside: Vec<Ty> = match kind {
                TyKind::Vec { elem, .. } | TyKind::Array { elem, .. } => vec![*elem],
                TyKind::Tuple(items) => items.clone(),
                TyKind::Enum(id) => {
                    self.types.enum_def(*id).variants.iter().flat_map(|variant| variant.fields.iter().map(|field| field.ty)).collect()
                }
                // A struct with a schema migrates its fields depth-first; any other holds them
                // where a reload cannot reach.
                TyKind::Struct(id) => {
                    if self.has_schema(ty, reload) {
                        Vec::new()
                    } else {
                        self.types.struct_def(*id).fields.iter().map(|field| field.ty).collect()
                    }
                }
                _ => Vec::new(),
            };
            out.extend(inside.into_iter().filter_map(schema_struct));
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
                reload.files.contains(&def.span.file)
                    || def.fields.iter().map(|field| field.ty).collect::<Vec<_>>().into_iter().any(|field| self.mentions_package(field, reload, seen))
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
    fn pointed_structs(&self, reload: &ReloadImage) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        for (_, kind) in self.types.all() {
            let TyKind::Ptr { inner, .. } = kind else { continue };
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
                !is_compiler_struct(&def.name.to_string())
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
                    shape.kind = "EMBER_RL_KIND_OPAQUE";
                }
            }
            TyKind::Struct(id) => {
                if self.weak_inner_id(*id).is_some() {
                    shape.kind = "EMBER_RL_KIND_WEAK";
                } else if self.shared_inner_id(*id).is_some() {
                    shape.kind = "EMBER_RL_KIND_HANDLE";
                } else if self.has_schema(ty, reload) {
                    shape.kind = "EMBER_RL_KIND_STRUCT";
                    shape.type_name = self.types.struct_def(*id).name.to_string();
                } else if self.holds_objects(ty, &mut Vec::new()) {
                    shape.kind = "EMBER_RL_KIND_OPAQUE";
                }
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
        let symbol = table(&format!("layout_{}", self.reload_shapes.len()));
        let mut tables = Vec::new();
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
                "static const {RT}rl_field {symbol}_tag[1] = {{ {{ \"tag\", NULL, \"tag\", 0, (uint32_t)offsetof({c_type}, tag), (uint32_t)sizeof((({c_type}*)0)->tag), 0, EMBER_RL_KIND_PLAIN, EMBER_RL_KIND_PLAIN, 0, NULL, NULL, NULL }} }};"
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
            "static const {RT}rl_type {symbol} = {{ {}, EMBER_RL_TYPE_STRUCT, NULL, 0, (uint32_t)sizeof({c_type}), (uint32_t)_Alignof({c_type}), UINT64_C(0x{:016x}), 0, NULL, NULL, {fields}, {variants}, NULL, NULL, NULL, NULL }};",
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
        let Some(reload) = self.reload.clone() else { return "NULL".to_string() };
        if !self.holds_objects(env, &mut Vec::new()) {
            return "NULL".to_string();
        }
        self.reload_shape_table(env, &reload)
    }

    /// `[HR-11]` — every package type's schema: a class's and a struct's fields (name, renamed-from
    /// name, type, schema hash, offset, size, what it holds), a unit enum's variants; and, for a
    /// type with declared field defaults, the function that writes one into a new instance
    /// (`[HR-14]`). The entries for the manifest's type table, and every identity.
    fn emit_type_schemas(&mut self, reload: &ReloadImage) -> (Vec<String>, Vec<String>) {
        let mut types = Vec::new();
        let mut identities = Vec::new();
        let contained = self.contained_structs(reload);
        let foreign = self.pointed_structs(reload);
        // A field's row and, when it has a declared default, its default's case; each type with its
        // mangled form (`[MNG-1]`), which its tables are named by.
        let mut layouts: Vec<(String, String, String, Vec<(String, Ty)>, String)> = Vec::new();
        let classes: Vec<ClassId> = self.types.runtime_classes().map(|(id, _)| id).collect();
        for id in classes {
            let def = self.types.class_def(id);
            let name = def.name.to_string();
            let mangled = self.mangled_type(TypeNode::Class(id));
            identities.push(format!("&{}", class_identity_symbol(&mangled)));
            if !reload.files.contains(&def.span.file) {
                continue;
            }
            let fields: Vec<(String, Ty)> = (0..self.types.class_field_count(id))
                .filter_map(|index| self.types.class_field_at(id, index))
                .map(|field| (field.name.to_string(), field.ty))
                .collect();
            layouts.push(("class".to_string(), name.clone(), mangled.clone(), fields, format!("struct {}", ember_branding::object_struct(&mangled))));
        }
        let structs: Vec<(StructId, String)> = self.types.structs().map(|(id, def)| (id, def.name.to_string())).collect();
        for (id, name) in structs {
            let Some(ty) = self.types.find(&TyKind::Struct(id)) else { continue };
            if !self.has_schema(ty, reload) {
                continue;
            }
            let fields: Vec<(String, Ty)> =
                self.types.struct_def(id).fields.iter().map(|field| (field.name.to_string(), field.ty)).collect();
            layouts.push(("struct".to_string(), name, self.mangled_type(TypeNode::Struct(id)), fields, self.c_type(ty)));
        }
        for (kind, name, mangled, fields, c_type) in layouts {
            let table_name = table(&format!("fields_{mangled}"));
            let init_name = table(&format!("init_{mangled}"));
            let mut rows = Vec::new();
            let mut cases = Vec::new();
            for (index, (field, ty)) in fields.iter().enumerate() {
                let c_field = member(field);
                let shape = self.field_shape(*ty, reload);
                let mut flags = shape.flags.clone();
                if let Some(function) = reload.defaults.get(&(name.clone(), field.clone())) {
                    flags.push("EMBER_RL_FIELD_DEFAULT");
                    let symbol = self.reload_function_symbol(function);
                    if let Some(symbol) = symbol {
                        let member_type = self.c_member_type(*ty);
                        cases.push(format!(
                            "    case {index}u: *({member_type}*)((unsigned char*)object + offsetof({c_type}, {c_field})) = {symbol}(); break;"
                        ));
                    }
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
            let layout = match (kind == "struct" && contained.contains(&name), foreign.contains(&name)) {
                (_, true) => "EMBER_RL_LAYOUT_FOREIGN",
                (true, false) => "EMBER_RL_LAYOUT_CONTAINED",
                (false, false) => "0",
            };
            let ty = if kind == "class" {
                self.types.classes().find(|(_, def)| def.name.to_string() == name).and_then(|(id, _)| self.types.find(&TyKind::Class(id)))
            } else {
                self.types.structs().find(|(_, def)| def.name.to_string() == name).and_then(|(id, _)| self.types.find(&TyKind::Struct(id)))
            };
            let schema = ty.map_or(0, |ty| self.type_schema(ty, &mut Vec::new()));
            if kind == "class" {
                let id = self.types.classes().find(|(_, def)| def.name.to_string() == name).map(|(id, _)| id).expect("a class");
                let def = self.types.class_def(id);
                let base = def.base.map_or("NULL".to_string(), |base| c_string_literal(&self.types.class_def(base).name.to_string()));
                types.push(format!(
                    "    {{ {}, EMBER_RL_TYPE_CLASS, {base}, 0, (uint32_t)sizeof({c_type}), (uint32_t)_Alignof({c_type}), UINT64_C(0x{schema:016x}), UINT64_C(0x{:016x}), &{}, &{}, {}u, {fields_ref}, 0, NULL, {init}, {migrate}, {old_type}, {drop_field} }},",
                    c_string_literal(&name),
                    self.methods_schema(id),
                    ember_branding::type_info(&mangled),
                    class_identity_symbol(&mangled),
                    rows.len(),
                ));
            } else {
                types.push(format!(
                    "    {{ {}, EMBER_RL_TYPE_STRUCT, NULL, {layout}, (uint32_t)sizeof({c_type}), (uint32_t)_Alignof({c_type}), UINT64_C(0x{schema:016x}), 0, NULL, NULL, {}u, {fields_ref}, 0, NULL, {init}, {migrate}, {old_type}, {drop_field} }},",
                    c_string_literal(&name),
                    rows.len(),
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
                    "static const {RT}rl_field {tag_name}[1] = {{ {{ \"tag\", NULL, \"tag\", 0, (uint32_t)offsetof({c_type}, tag), (uint32_t)sizeof((({c_type}*)0)->tag), 0, EMBER_RL_KIND_PLAIN, EMBER_RL_KIND_PLAIN, 0, NULL, NULL, NULL }} }};"
                ));
            }
            let mut rows = Vec::new();
            for variant in &def.variants {
                let renamed = reload.renamed.get(&(name.clone(), variant.name.to_string())).map_or("NULL".to_string(), |old| c_string_literal(old));
                let payload: Vec<String> = variant.fields.iter().map(|field| self.type_schema(field.ty, &mut Vec::new()).to_string()).collect();
                let mut fields = Vec::new();
                let held: Vec<_> = variant.fields.iter().filter(|field| payloads && !self.is_void(field.ty)).cloned().collect();
                for field in &held {
                    let shape = self.field_shape(field.ty, reload);
                    let member = format!("payload.{}.{}", member(variant.name), member(field.name));
                    fields.push(format!(
                        "    {{ {}, NULL, {}, UINT64_C(0x{:016x}), (uint32_t)offsetof({c_type}, {member}), (uint32_t)sizeof((({c_type}*)0)->{member}), 0, {} }},",
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
            types.push(format!(
                "    {{ {}, EMBER_RL_TYPE_ENUM, NULL, {}, (uint32_t)sizeof({c_type}), (uint32_t)_Alignof({c_type}), UINT64_C(0x{:016x}), 0, NULL, NULL, {}, {}u, {}, NULL, NULL, NULL, NULL }},",
                c_string_literal(&name),
                if self.types.enum_is_bare(id) { "EMBER_RL_LAYOUT_UNIT" } else { "0" },
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
