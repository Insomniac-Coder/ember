//! Annex B — the C of a reloadable image: each reloadable function's body under a name of its own
//! and a thunk under the function's symbol that loads its target from the image's copy of the call
//! table (`[HR-5]`, `[HR-6]`); the statics' storage taken from the runtime (`[HR-4a]`); and what
//! the image says of itself (`image.rs`'s decisions: the types' identities and schemas, `[HR-11]`,
//! and the reload manifest, `[HR-8]`, which the runtime reads from the image's section and fills
//! in), written as C initialisers by `CWriter`.

use super::*;
use crate::image::{DropCase, FieldShape, FnRow, FrameInfo, Holder, InitCase, Manifest, Member, Num, Planner, Row, StaticRow, TypeRow, VariantRow, Writer, table};

/// `[HR-14]` — the C body of a range type's test over its representation at `value`: `T.checked`'s
/// decision (`[RNG-3]`), NaN in no range (`[RNG-6]`). `None` for a representation it has no test
/// for (`f16`), whose change the reload refuses (`image::range_testable`).
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

impl<'a> Emitter<'a> {
    pub(crate) fn is_reloadable(&self, symbol: &str) -> bool {
        self.reload.as_ref().is_some_and(|reload| reload.reloadable.contains(symbol))
    }

    /// The image's plan (`image::Planner`): its slots, its statics, its frames' dependencies.
    pub(crate) fn plan_reload(&mut self, bodies: &[Body]) {
        self.reload_plan = self.reload.as_ref().map(|reload| Planner::new(self.types, reload, bodies, &self.lazy_statics));
    }

    fn plan(&self) -> &Planner<'a> {
        self.reload_plan.as_ref().expect("a reloadable image has its plan")
    }

    /// The planner deciding with this emitter writing what it decides, in C.
    fn with_plan<R>(&mut self, work: impl FnOnce(&mut Planner<'a>, &mut CWriter<'_, 'a>) -> R) -> R {
        let mut plan = self.reload_plan.take().expect("a reloadable image has its plan");
        let result = work(&mut plan, &mut CWriter(self));
        self.reload_plan = Some(plan);
        result
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

    /// A body's signature under its own symbol.
    pub(crate) fn reload_body_signature(&self, body: &Body) -> String {
        let signature = self.signature(body);
        let named = format!(" {}(", body.symbol);
        assert!(signature.contains(&named), "a signature names its function");
        signature.replacen(&named, &format!(" {}(", image::body_symbol(&body.symbol)), 1)
    }

    /// The image's tables, declared before any thunk reads them.
    pub(crate) fn emit_reload_tables(&mut self) {
        if self.reload.is_none() {
            return;
        }
        let count = self.plan().slots.len().max(1);
        let statics = self.plan().static_entries().max(1);
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
        let Some(&index) = self.plan().slots.get(&body.symbol) else { return };
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

    /// `[HR-6]` — a value of a reloadable function: the descriptor the runtime made stand for it in
    /// every image, so the value is one whichever image made it, and calling it reaches the
    /// permanent thunk.
    pub(crate) fn reload_fn_value(&self, symbol: &str) -> Option<String> {
        let index = *self.reload_plan.as_ref()?.slots.get(symbol)?;
        if !self.is_reloadable(symbol) || !self.native_fn_values.contains_key(symbol) {
            return None;
        }
        let descriptor = format!("((const struct {RT}native_fn_descriptor*){}[{index}])", table("descriptors"));
        Some(format!("((struct {RT}fn_value){{{descriptor}->native, {descriptor}, {{0, 0, 0}}}})"))
    }

    /// `[HR-4a]` — a static's storage and state, the runtime's.
    pub(crate) fn reload_static_access(&self, name: &str, ty: Ty) -> String {
        format!("(({}*){}[{}])", self.c_type(ty), table("static_data"), self.plan().static_index(name))
    }

    /// `[STA-3]`'s accessor in a reloadable image: the storage and the state word are the
    /// runtime's, found by the static's name, so a reload keeps the value ([HR-17]).
    pub(crate) fn reload_lazy_static_definition(&self, init: &str, ty: Ty, name: &str) -> String {
        let c_type = self.c_type(ty);
        let shown = name.rsplit('.').next().unwrap_or(name);
        let index = self.plan().static_index(name);
        format!(
            "\n{c_type}* {init}_static(void) {{\n    {c_type}* storage = ({c_type}*){data}[{index}];\n    if ({rt}rt_static_begin({state}[{index}], \"{shown}\")) {{\n        *storage = {init}();\n        {rt}rt_static_end({state}[{index}]);\n    }}\n    return storage;\n}}\n",
            data = table("static_data"),
            state = table("static_state"),
            rt = RT,
        )
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
        image::fnv(&parts.iter().map(String::as_str).collect::<Vec<_>>())
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
                    TyKind::Class(id) => image::fnv(&["handle", &self.types.class_def(*id).name.to_string()]).to_string(),
                    _ => image::type_schema(self.types, ty, &mut Vec::new()).to_string(),
                }
            };
            parts.push(crossing(body.return_ty()));
            parts.extend((1..=body.arg_count).filter_map(|index| body.locals.get(index)).map(|local| crossing(local.ty)));
        }
        image::fnv(&parts.iter().map(String::as_str).collect::<Vec<_>>())
    }

    /// `[HR-8]` — the manifest, in the main unit (`image::Planner::manifest`).
    pub(crate) fn emit_reload_manifest(&mut self, bodies: &[Body], has_entry: bool) {
        if self.reload.is_none() || self.unit.is_some() {
            return;
        }
        self.with_plan(|plan, writer| plan.manifest(bodies, has_entry, writer));
    }

    /// `[CLO-10]`, `[HR-15]` — in a reloadable image, what a closure's descriptor says its
    /// captured state holds: `NULL` when it holds no object.
    pub(crate) fn reload_captures(&mut self, env: Ty) -> String {
        if self.reload.is_none() {
            return "NULL".to_string();
        }
        self.with_plan(|plan, writer| plan.captures(env, writer)).map_or_else(|| "NULL".to_string(), |symbol| format!("&{symbol}"))
    }

    /// `[HR-3b]` (A1) — `body`'s frame description, written beside it; its call sites by block.
    pub(crate) fn emit_frame_info(&mut self, body: &Body) -> String {
        let info = self.plan().frame_info(body, self.map);
        CWriter(self).frame_info(&info);
        self.rl_sites = info.by_block;
        info.symbol
    }

    // -- C of a row ---------------------------------------------------------------------------------

    fn holder_c(&self, holder: Holder) -> String {
        match holder {
            Holder::Ty(ty) => self.c_type(ty),
            Holder::Object(id) => format!("struct {}", ember_branding::object_struct(&self.mangled_type(TypeNode::Class(id)))),
        }
    }

    fn member_c(&self, holder: Holder, field: &Member) -> String {
        match (field, holder) {
            (Member::Field(index), Holder::Object(id)) => member(self.types.class_field_at(id, *index).expect("a class's field").name),
            (Member::Field(index), Holder::Ty(ty)) => match self.types.kind(ty) {
                TyKind::Struct(id) => member(self.types.struct_def(*id).fields[*index].name),
                _ => unreachable!("a field of a struct"),
            },
            (Member::Tuple(index), _) => format!("_{index}"),
            (Member::Payload(variant, index), Holder::Ty(ty)) => match self.types.kind(ty) {
                TyKind::Enum(id) => {
                    let variant = &self.types.enum_def(*id).variants[*variant];
                    format!("payload.{}.{}", member(variant.name), member(variant.fields[*index].name))
                }
                _ => unreachable!("a payload of an enum"),
            },
            (Member::Payload(..), Holder::Object(_)) => unreachable!("a payload of an enum"),
            (Member::Tag, _) => "tag".to_string(),
            (Member::Named(name), _) => member(name),
        }
    }

    fn num_c(&self, num: &Num) -> String {
        match num {
            Num::Zero => "0".to_string(),
            Num::Size(holder) => format!("(uint32_t)sizeof({})", self.holder_c(*holder)),
            Num::MemberSize(holder, field) => {
                let c = self.holder_c(*holder);
                format!("(uint32_t)sizeof((({c}*)0)->{})", self.member_c(*holder, field))
            }
            Num::Align(holder) => format!("(uint32_t)_Alignof({})", self.holder_c(*holder)),
            Num::VecAlign(ty) => format!("(uint32_t)EMBER_VEC_ALIGN_OF({})", self.c_type(*ty)),
            Num::Offset(holder, field) => format!("(uint32_t)offsetof({}, {})", self.holder_c(*holder), self.member_c(*holder, field)),
            Num::Within(element, path) => {
                let TyKind::Struct(mut structure) = *self.types.kind(*element) else { unreachable!("an `SoA`'s element is a struct") };
                let mut members = Vec::new();
                for &index in path {
                    let field = &self.types.struct_def(structure).fields[index];
                    members.push(member(field.name));
                    if let TyKind::Struct(inner) = *self.types.kind(field.ty) {
                        structure = inner;
                    }
                }
                format!("(uint32_t)offsetof({}, {})", self.c_type(*element), members.join("."))
            }
            Num::SizeT => "(uint32_t)sizeof(size_t)".to_string(),
        }
    }

    fn flags_c(flags: &[&str]) -> String {
        if flags.is_empty() { "0".to_string() } else { flags.join(" | ") }
    }

    /// The row's members after `flags`: what it holds, its elements', and its own tables.
    fn tail_c(&self, shape: &FieldShape) -> String {
        let element_type = if shape.element_type.is_empty() { "NULL".to_string() } else { c_string_literal(&shape.element_type) };
        let reference = |symbol: &Option<String>| symbol.as_ref().map_or_else(|| "NULL".to_string(), |symbol| format!("&{symbol}"));
        let range = shape.range.map_or_else(|| "NULL".to_string(), image::range_test_symbol);
        format!(
            "{}, {}, {}, {element_type}, {}, {}, {range}, {}, {}",
            shape.kind,
            shape.element_kind,
            self.num_c(&shape.element_size),
            reference(&shape.layout),
            reference(&shape.inner),
            self.num_c(&shape.align),
            self.num_c(&shape.within)
        )
    }

    fn row_c(&self, row: &Row) -> String {
        format!(
            "{{ {}, {}, {}, {}, {}, {}, {}, {} }}",
            c_string_literal(&row.name),
            row.renamed.as_deref().map_or_else(|| "NULL".to_string(), c_string_literal),
            c_string_literal(&row.shape.type_name),
            row.schema.map_or_else(|| "0".to_string(), |schema| format!("UINT64_C(0x{schema:016x})")),
            self.num_c(&row.offset),
            self.num_c(&row.size),
            Self::flags_c(&row.shape.flags),
            self.tail_c(&row.shape)
        )
    }

    fn type_row_c(&self, row: &TypeRow) -> String {
        let symbol = |symbol: &Option<String>| symbol.clone().unwrap_or_else(|| "NULL".to_string());
        let address = |symbol: &Option<String>| symbol.as_ref().map_or_else(|| "NULL".to_string(), |symbol| format!("&{symbol}"));
        let literal = |text: &Option<String>| text.as_deref().map_or_else(|| "NULL".to_string(), c_string_literal);
        let hash = |value: Option<u64>| value.map_or_else(|| "0".to_string(), |value| format!("UINT64_C(0x{value:016x})"));
        let list = |list: &Option<(usize, Option<String>)>| match list {
            Some((count, table)) => format!("{count}u, {}", symbol(table)),
            None => "0, NULL".to_string(),
        };
        format!(
            "{{ {}, {}, {}, {}, {}, {}, UINT64_C(0x{:016x}), {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {} }}",
            c_string_literal(&row.name),
            row.kind,
            literal(&row.base),
            Self::flags_c(&row.layout),
            self.num_c(&row.size),
            self.num_c(&row.align),
            row.schema,
            hash(row.methods),
            address(&row.type_info),
            address(&row.identity),
            list(&row.fields),
            list(&row.variants),
            symbol(&row.init),
            symbol(&row.migrate),
            literal(&row.old_type),
            symbol(&row.drop_field),
            hash(row.attributes),
            symbol(&row.columns)
        )
    }

    /// A field's place in the value at `object`, as an lvalue of its member type.
    fn field_access(&self, holder: Holder, field: &Member, ty: Ty) -> String {
        format!(
            "(*({}*)((unsigned char*)object + offsetof({}, {})))",
            self.c_member_type(ty),
            self.holder_c(holder),
            self.member_c(holder, field)
        )
    }
}

/// The C backend's `image::Writer`: what the planner decides, as C initialisers and functions
/// written where the emitter is.
pub(crate) struct CWriter<'e, 'a>(pub(crate) &'e mut Emitter<'a>);

impl Writer for CWriter<'_, '_> {
    fn range_test(&mut self, symbol: &str, id: ember_types::RangeId) {
        let def = self.0.types.range_def(id);
        let body = range_test_body(self.0.types.kind(def.repr), def).expect("a testable range");
        self.0.line(&format!("static inline int {symbol}(const void* value) {{\n    {body}\n}}"));
    }

    fn field(&mut self, symbol: &str, row: &Row) {
        let row = self.0.row_c(row);
        self.0.line(&format!("static const {RT}rl_field {symbol} = {row};"));
    }

    fn fields(&mut self, symbol: &str, rows: &[Row]) {
        self.0.line(&format!("static const {RT}rl_field {symbol}[{}] = {{", rows.len()));
        for row in rows {
            let row = self.0.row_c(row);
            self.0.line(&format!("    {row},"));
        }
        self.0.line("};");
    }

    fn tag(&mut self, symbol: &str, row: &Row) {
        let row = self.0.row_c(row);
        self.0.line(&format!("static const {RT}rl_field {symbol}[1] = {{ {row} }};"));
    }

    fn type_row(&mut self, symbol: &str, row: &TypeRow) {
        let row = self.0.type_row_c(row);
        self.0.line(&format!("static const {RT}rl_type {symbol} = {row};"));
    }

    fn variants(&mut self, symbol: &str, rows: &[VariantRow]) {
        self.0.line(&format!("static const {RT}rl_variant {symbol}[{}] = {{", rows.len()));
        for row in rows {
            self.0.line(&format!(
                "    {{ {}, {}, UINT64_C({}), {}, {}u, {} }},",
                c_string_literal(&row.name),
                row.renamed.as_deref().map_or_else(|| "NULL".to_string(), c_string_literal),
                row.discriminant,
                row.payload.map_or_else(|| "0".to_string(), |payload| format!("UINT64_C(0x{payload:016x})")),
                row.fields.0,
                row.fields.1.clone().unwrap_or_else(|| "NULL".to_string())
            ));
        }
        self.0.line("};");
    }

    fn init_fn(&mut self, symbol: &str, cases: &[InitCase]) {
        self.0.line(&format!("static void {symbol}(void* object, uint32_t index) {{"));
        self.0.line("    switch (index) {");
        for case in cases {
            let line = format!(
                "    case {}u: *({}*)((unsigned char*)object + offsetof({}, {})) = {}(); break;",
                case.index,
                self.0.c_member_type(case.ty),
                self.0.holder_c(case.holder),
                self.0.member_c(case.holder, &case.member),
                case.function
            );
            self.0.line(&line);
        }
        self.0.line("    default: break;");
        self.0.line("    }");
        self.0.line("}");
    }

    fn drop_fn(&mut self, symbol: &str, cases: &[DropCase]) {
        let mut drops = Vec::new();
        for case in cases {
            let access = self.0.field_access(case.holder, &case.member, case.ty);
            let mut lines = Vec::new();
            match self.0.drop_glue_call(&access, case.ty) {
                Some(call) => lines.push(call),
                None => self.0.drop_lines(&access, case.ty, &mut lines),
            }
            drops.push((case.index, lines));
        }
        self.0.line(&format!("static void {symbol}(void* object, uint32_t index) {{"));
        self.0.line("    switch (index) {");
        for (index, lines) in drops {
            self.0.line(&format!("    case {index}u: {{"));
            for line in lines {
                self.0.line(&format!("        {line}"));
            }
            self.0.line("        break;");
            self.0.line("    }");
        }
        self.0.line("    default: break;");
        self.0.line("    }");
        self.0.line("}");
    }

    /// `[HR-16]` — the runtime builds the old value as `Old<Name>` and passes its address; the
    /// driver's wrapper gives the new value, or `None` with the `ReloadError` in the report. 0 is
    /// success.
    fn migrate_fn(&mut self, symbol: &str, wrapper: &str, ret: Ty, option: EnumId, some: usize) {
        let e = &mut *self.0;
        let def = e.types.enum_def(option);
        let discriminant = def.variants[some].discriminant;
        let value = e.c_type(def.variants[some].fields[0].ty);
        let result = e.c_type(ret);
        let tag = e.enum_tag(option, "result");
        let payload = e.enum_member(option, "result", some, 0);
        e.line(&format!("static int {symbol}(void* out, const void* old, char* report, size_t report_size) {{"));
        e.line(&format!("    {RT}vec text = {RT}vec_empty();"));
        e.line(&format!("    {result} result = {wrapper}((void*)old, &text);"));
        e.line(&format!("    int done = {tag} == {discriminant};"));
        e.line(&format!("    if (done) {{ *({value}*)out = {payload}; }}"));
        e.line("    else if (report_size > 0) {");
        e.line("        size_t n = text.len < report_size - 1 ? text.len : report_size - 1;");
        e.line("        if (n > 0) { memcpy(report, text.ptr, n); }");
        e.line("        report[n] = '\\0';");
        e.line("    }");
        e.line(&format!("    {RT}vec_free(&text, 1);"));
        e.line("    return done ? 0 : 1;");
        e.line("}");
    }

    fn frame_info(&mut self, info: &FrameInfo) {
        let symbol = &info.symbol;
        let mut rows = Vec::new();
        for (index, (line, types)) in info.sites.iter().enumerate() {
            let list = if types.is_empty() {
                "NULL".to_string()
            } else {
                let names = format!("{symbol}_t{index}");
                let quoted: Vec<String> = types.iter().map(|name| c_string_literal(name)).collect();
                self.0.line(&format!("static const char* const {names}[{}] = {{ {} }};", types.len(), quoted.join(", ")));
                names
            };
            rows.push(format!("{{ {line}u, {}u, {list} }}", types.len()));
        }
        let sites = if rows.is_empty() {
            "NULL".to_string()
        } else {
            self.0.line(&format!("static const {RT}rl_site {symbol}_sites[{}] = {{ {} }};", rows.len(), rows.join(", ")));
            format!("{symbol}_sites")
        };
        self.0.line(&format!("static const {RT}rl_frame_info {symbol} = {{ {}, {}u, {sites} }};", c_string_literal(&info.function), rows.len()));
    }

    fn manifest(&mut self, manifest: &Manifest) {
        let e = &mut *self.0;
        let fns: Vec<String> = manifest
            .fns
            .iter()
            .map(|row| match row {
                FnRow::Reloadable { symbol, descriptor, signature, body_hash, flags } => format!(
                    "    {{ {}, (void (*)(void)){}, (void (*)(void)){symbol}, {}, UINT64_C(0x{signature:016x}), UINT64_C(0x{body_hash:016x}), {} }},",
                    c_string_literal(symbol),
                    image::body_symbol(symbol),
                    descriptor.as_ref().map_or_else(|| "NULL".to_string(), |descriptor| format!("&{descriptor}")),
                    flags.join(" | ")
                ),
                FnRow::NoReload { symbol, body_hash } => {
                    format!("    {{ {}, NULL, NULL, NULL, 0, UINT64_C(0x{body_hash:016x}), EMBER_RL_FN_NORELOAD }},", c_string_literal(symbol))
                }
            })
            .collect();
        let statics: Vec<String> = manifest
            .statics
            .iter()
            .map(|row| match row {
                StaticRow::Stored { name, ty, schema, initialiser, flags, constant, shape } => {
                    let c_type = e.c_type(*ty);
                    let initial = if *constant { format!("&{}", constant_static_symbol(name)) } else { "NULL".to_string() };
                    format!(
                        "    {{ {}, (uint32_t)sizeof({c_type}), (uint32_t)_Alignof({c_type}), UINT64_C(0x{schema:016x}), UINT64_C(0x{initialiser:016x}), {}, {initial}, NULL, &{shape} }},",
                        c_string_literal(name),
                        Emitter::flags_c(flags)
                    )
                }
                StaticRow::Folded { name, declared, initialiser, reinit } => format!(
                    "    {{ {}, 0, 1, UINT64_C(0x{declared:016x}), UINT64_C(0x{initialiser:016x}), {}, NULL, NULL, NULL }},",
                    c_string_literal(name),
                    if *reinit { "EMBER_RL_STATIC_REINIT" } else { "0" },
                ),
            })
            .collect();
        let types: Vec<String> = manifest.types.iter().map(|row| format!("    {},", e.type_row_c(row))).collect();
        let identities: Vec<String> = manifest.identities.iter().map(|identity| format!("&{identity}")).collect();
        let fns_symbol = table("fns");
        let statics_symbol = table("statics");
        let types_symbol = table("types");
        let identities_symbol = table("identities");
        e.line("/* [HR-8] the reload manifest */");
        e.line(&format!("static const {RT}rl_fn {fns_symbol}[{}] = {{", fns.len().max(1)));
        for line in &fns {
            e.line(line);
        }
        if fns.is_empty() {
            e.line("    { NULL, NULL, NULL, NULL, 0, 0, 0 },");
        }
        e.line("};");
        e.line(&format!("static const {RT}rl_static {statics_symbol}[{}] = {{", statics.len().max(1)));
        for line in &statics {
            e.line(line);
        }
        if statics.is_empty() {
            e.line("    { NULL, 0, 0, 0, 0, 0, NULL, NULL, NULL },");
        }
        e.line("};");
        e.line(&format!("static const {RT}rl_type {types_symbol}[{}] = {{", types.len().max(1)));
        for line in &types {
            e.line(line);
        }
        if types.is_empty() {
            e.line("    { NULL, 0, NULL, 0, 0, 0, 0, 0, NULL, NULL, 0, NULL, 0, NULL, NULL, NULL, NULL, NULL, 0, NULL },");
        }
        e.line("};");
        e.line(&format!("static {RT}rl_identity* const {identities_symbol}[{}] = {{", identities.len().max(1)));
        e.line(&format!("    {}", if identities.is_empty() { "NULL".to_string() } else { identities.join(", ") }));
        e.line("};");
        let headers_symbol = table("headers");
        e.line(&format!("static const {RT}rl_header {headers_symbol}[{}] = {{", manifest.headers.len().max(1)));
        for (name, hash) in &manifest.headers {
            e.line(&format!("    {{ {}, {} }},", c_string_literal(name), c_string_literal(hash)));
        }
        if manifest.headers.is_empty() {
            e.line("    { NULL, NULL },");
        }
        e.line("};");
        let layout = if manifest.instances { "obj_header_v40" } else { "obj_header_v24" };
        let entry = if manifest.has_entry { image::entry_symbol() } else { "NULL".to_string() };
        e.line(&format!("EMBER_RL_MANIFEST const {RT}rl_manifest {} = {{", image::manifest_symbol()));
        e.line("    EMBER_RELOAD_MAGIC, EMBER_RELOAD_PROTOCOL,");
        e.line(&format!("    (uint32_t)sizeof({RT}rl_manifest), (uint32_t)sizeof({RT}obj_header), {}u,", manifest.mode));
        e.line(&format!("    {},", c_string_literal(&manifest.package)));
        e.line(&format!("    {}, {},", ember_branding::runtime(layout), ember_branding::runtime("reload_protocol_v1")));
        e.line(&format!("    {}u, {fns_symbol}, {}, {}, {},", fns.len(), table("targets"), table("addresses"), table("descriptors")));
        e.line(&format!("    {}u, {statics_symbol}, {}, {},", statics.len(), table("static_data"), table("static_state")));
        e.line(&format!("    {}u, {types_symbol},", types.len()));
        e.line(&format!("    {}u, {identities_symbol},", identities.len()));
        e.line(&format!("    {entry},"));
        e.line(&format!("    {}u, {headers_symbol}", manifest.headers.len()));
        e.line("};");
        e.line("");
    }

    fn soa_columns(&mut self, soa: StructId) -> String {
        format!("{}_layout", self.0.c_named(TypeNode::Struct(soa)))
    }

    fn signature_hash(&self, body: &Body) -> u64 {
        self.0.signature_hash(body)
    }

    fn methods_hash(&self, class: ClassId) -> u64 {
        self.0.methods_schema(class)
    }

    fn descriptor(&self, symbol: &str) -> Option<String> {
        self.0.native_fn_values.get(symbol).map(|value| value.descriptor.clone())
    }

    fn interface_identities(&self) -> Vec<String> {
        self.0.reload_interface_ids.borrow().iter().cloned().collect()
    }

    fn closure_envs(&self) -> Vec<Ty> {
        self.0.closure_values.values().map(|(_, env)| *env).collect()
    }

    fn dyn_concretes(&self) -> Vec<Ty> {
        self.0.interface_adapters.values().map(|adapter| adapter.concrete).collect()
    }
}
