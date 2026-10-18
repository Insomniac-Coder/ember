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
struct FieldShape {
    kind: &'static str,
    flags: Vec<&'static str>,
    type_name: String,
    element_kind: &'static str,
    element_size: String,
    element_type: String,
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
    /// a pointer) counts by its name.
    pub(crate) fn type_schema(&self, ty: Ty, seen: &mut Vec<Ty>) -> u64 {
        if seen.contains(&ty) {
            return fnv(&["again", &self.types.display(ty)]);
        }
        seen.push(ty);
        let hash = match self.types.kind(ty) {
            TyKind::Struct(id) => {
                let def = self.types.struct_def(*id);
                let mut parts = vec!["struct".to_string(), def.name.to_string()];
                for field in &def.fields {
                    parts.push(field.name.to_string());
                    parts.push(self.type_schema(field.ty, seen).to_string());
                }
                fnv(&parts.iter().map(String::as_str).collect::<Vec<_>>())
            }
            TyKind::Enum(id) => {
                let def = self.types.enum_def(*id);
                let mut parts = vec!["enum".to_string(), def.name.to_string(), self.types.display(def.repr)];
                for variant in &def.variants {
                    parts.push(variant.name.to_string());
                    for field in &variant.fields {
                        parts.push(field.name.to_string());
                        parts.push(self.type_schema(field.ty, seen).to_string());
                    }
                }
                fnv(&parts.iter().map(String::as_str).collect::<Vec<_>>())
            }
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
            _ => fnv(&[&self.types.display(ty)]),
        };
        seen.pop();
        hash
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
            if address_taken.contains(body.symbol.as_str()) || body.abi.as_deref() == Some("C") {
                flags.push_str(" | EMBER_RL_FN_ADDRESS");
            }
            // `[HR-21]` — a host reaches an export through its permanent thunk.
            if body.abi.as_deref() == Some("C") {
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
            let element_type = if shape.element_type.is_empty() { "NULL".to_string() } else { c_string_literal(&shape.element_type) };
            self.line(&format!(
                "static const {RT}rl_field {symbol} = {{ {}, NULL, {}, UINT64_C(0x{:016x}), 0, (uint32_t)sizeof({}), {flags}, {}, {}, {}, {element_type} }};",
                c_string_literal(name),
                c_string_literal(&shape.type_name),
                self.type_schema(*ty, &mut Vec::new()),
                self.c_type(*ty),
                shape.kind,
                shape.element_kind,
                shape.element_size,
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
                reload.files.contains(&def.span.file) && def.is_unit_only()
            }
            _ => false,
        }
    }

    /// What a field of type `ty` holds: its kind, its flags, its type's name, and for an `Array`
    /// its elements'.
    fn field_shape(&self, ty: Ty, reload: &ReloadImage) -> FieldShape {
        let mut shape = FieldShape {
            kind: "EMBER_RL_KIND_PLAIN",
            flags: Vec::new(),
            type_name: self.types.display(ty),
            element_kind: "EMBER_RL_KIND_PLAIN",
            element_size: "0".to_string(),
            element_type: String::new(),
        };
        let element = |this: &Self, elem: Ty| -> (&'static str, String) {
            match this.types.kind(elem) {
                TyKind::Class(_) => ("EMBER_RL_KIND_HANDLE", String::new()),
                TyKind::Enum(id) if this.types.option_niche(*id).is_some_and(|niche| matches!(this.types.kind(niche.payload), TyKind::Class(_))) => {
                    ("EMBER_RL_KIND_OPTION_HANDLE", String::new())
                }
                TyKind::Struct(id) if this.has_schema(elem, reload) => {
                    ("EMBER_RL_KIND_STRUCT", this.types.struct_def(*id).name.to_string())
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
                let (kind, name) = element(self, *elem);
                shape.kind = "EMBER_RL_KIND_ARRAY";
                shape.flags.push("EMBER_RL_FIELD_ZERO");
                shape.element_kind = kind;
                shape.element_size = format!("(uint32_t)sizeof({})", self.c_type(*elem));
                shape.element_type = name;
            }
            TyKind::Enum(id) => {
                if self.types.option_niche(*id).is_some_and(|niche| matches!(self.types.kind(niche.payload), TyKind::Class(_))) {
                    shape.kind = "EMBER_RL_KIND_OPTION_HANDLE";
                } else if self.has_schema(ty, reload) {
                    shape.kind = "EMBER_RL_KIND_ENUM";
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

    /// `[HR-11]` — every package type's schema: a class's and a struct's fields (name, renamed-from
    /// name, type, schema hash, offset, size, what it holds), a unit enum's variants; and, for a
    /// type with declared field defaults, the function that writes one into a new instance
    /// (`[HR-14]`). The entries for the manifest's type table, and every identity.
    fn emit_type_schemas(&mut self, reload: &ReloadImage) -> (Vec<String>, Vec<String>) {
        let mut types = Vec::new();
        let mut identities = Vec::new();
        let contained = self.contained_structs(reload);
        // A field's row and, when it has a declared default, its default's case.
        let mut layouts: Vec<(String, String, Vec<(String, Ty)>, String)> = Vec::new();
        let classes: Vec<ClassId> = self.types.runtime_classes().map(|(id, _)| id).collect();
        for id in classes {
            let def = self.types.class_def(id);
            let name = def.name.to_string();
            identities.push(format!("&{}", class_identity_symbol(&name)));
            if !reload.files.contains(&def.span.file) {
                continue;
            }
            let fields: Vec<(String, Ty)> = (0..self.types.class_field_count(id))
                .filter_map(|index| self.types.class_field_at(id, index))
                .map(|field| (field.name.to_string(), field.ty))
                .collect();
            layouts.push(("class".to_string(), name.clone(), fields, format!("struct {}", ember_branding::object_struct(&name))));
        }
        let structs: Vec<(StructId, String)> = self.types.structs().map(|(id, def)| (id, def.name.to_string())).collect();
        for (id, name) in structs {
            let Some(ty) = self.types.find(&TyKind::Struct(id)) else { continue };
            if !self.has_schema(ty, reload) {
                continue;
            }
            let fields: Vec<(String, Ty)> =
                self.types.struct_def(id).fields.iter().map(|field| (field.name.to_string(), field.ty)).collect();
            layouts.push(("struct".to_string(), name, fields, self.c_type(ty)));
        }
        for (kind, name, fields, c_type) in layouts {
            let table_name = table(&format!("fields_{}", c_name(&name)));
            let init_name = table(&format!("init_{}", c_name(&name)));
            let mut rows = Vec::new();
            let mut cases = Vec::new();
            for (index, (field, ty)) in fields.iter().enumerate() {
                let shape = self.field_shape(*ty, reload);
                let mut flags = shape.flags.clone();
                if let Some(function) = reload.defaults.get(&(name.clone(), field.clone())) {
                    flags.push("EMBER_RL_FIELD_DEFAULT");
                    let symbol = self.reload_function_symbol(function);
                    if let Some(symbol) = symbol {
                        let member_type = self.c_member_type(*ty);
                        cases.push(format!(
                            "    case {index}u: *({member_type}*)((unsigned char*)object + offsetof({c_type}, {field})) = {symbol}(); break;"
                        ));
                    }
                }
                let renamed = reload.renamed.get(&(name.clone(), field.clone())).map_or("NULL".to_string(), |old| c_string_literal(old));
                let flags = if flags.is_empty() { "0".to_string() } else { flags.join(" | ") };
                let element_type = if shape.element_type.is_empty() { "NULL".to_string() } else { c_string_literal(&shape.element_type) };
                rows.push(format!(
                    "    {{ {}, {renamed}, {}, UINT64_C(0x{:016x}), (uint32_t)offsetof({c_type}, {field}), (uint32_t)sizeof((({c_type}*)0)->{field}), {flags}, {}, {}, {}, {element_type} }},",
                    c_string_literal(field),
                    c_string_literal(&shape.type_name),
                    self.type_schema(*ty, &mut Vec::new()),
                    shape.kind,
                    shape.element_kind,
                    shape.element_size,
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
            let migrate = self.reload_migrate_adapter(&name, reload);
            let old_type = reload.migrations.get(&name).map_or("NULL".to_string(), |(_, old)| c_string_literal(old));
            // `[HR-2a]` — reclaim drops the fields an edit removed, one by one.
            let drop_name = table(&format!("drop_{}", c_name(&name)));
            let mut drops = Vec::new();
            for (index, (field, ty)) in fields.iter().enumerate() {
                if !self.types.needs_drop(*ty) {
                    continue;
                }
                let access = format!("(*({}*)((unsigned char*)object + offsetof({c_type}, {field})))", self.c_member_type(*ty));
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
            let layout = if kind == "struct" && contained.contains(&name) { "EMBER_RL_LAYOUT_CONTAINED" } else { "0" };
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
                    ember_branding::type_info(&name),
                    class_identity_symbol(&name),
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
            let def = self.types.enum_def(id);
            if !reload.files.contains(&def.span.file) {
                continue;
            }
            let mut rows = Vec::new();
            for variant in &def.variants {
                let renamed = reload.renamed.get(&(name.clone(), variant.name.to_string())).map_or("NULL".to_string(), |old| c_string_literal(old));
                let payload: Vec<String> = variant.fields.iter().map(|field| self.type_schema(field.ty, &mut Vec::new()).to_string()).collect();
                rows.push(format!(
                    "    {{ {}, {renamed}, UINT64_C({}), UINT64_C(0x{:016x}) }},",
                    c_string_literal(&variant.name.to_string()),
                    variant.discriminant as u64,
                    fnv(&payload.iter().map(String::as_str).collect::<Vec<_>>()),
                ));
            }
            let table_name = table(&format!("variants_{}", c_name(&name)));
            if !rows.is_empty() {
                self.line(&format!("static const {RT}rl_variant {table_name}[{}] = {{", rows.len()));
                for row in &rows {
                    self.line(row);
                }
                self.line("};");
            }
            let c_type = self.c_type(ty);
            types.push(format!(
                "    {{ {}, EMBER_RL_TYPE_ENUM, NULL, {}, (uint32_t)sizeof({c_type}), (uint32_t)_Alignof({c_type}), UINT64_C(0x{:016x}), 0, NULL, NULL, 0, NULL, {}u, {}, NULL, NULL, NULL, NULL }},",
                c_string_literal(&name),
                if def.is_unit_only() { "EMBER_RL_LAYOUT_UNIT" } else { "0" },
                self.type_schema(ty, &mut Vec::new()),
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
    fn reload_migrate_adapter(&mut self, name: &str, reload: &ReloadImage) -> String {
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
        let adapter = table(&format!("migrate_{}", c_name(name)));
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

/// A class's identity record across images.
pub(crate) fn class_identity_symbol(class: &str) -> String {
    ember_branding::mangled(&format!("tid_{}", class.replace('.', "_")))
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
