//! `[BLD-FFI-5a]` (ODR-162) — the objects of a `@sync` class an export hands to C and C++: the
//! library's functions that count and end a handle and call each `pub` method, and what the C and
//! C++ headers declare of them. A handle crosses as the object's address, with one strong count:
//! an export's result gives C++ its count, a C++ copy takes another (the counts of a `@sync` class
//! are atomic, `[THR-1]`), and the last one's end frees the object.

use super::{Emitter, RT};
use ember_mir::{Body, ParameterMode};
use ember_types::{ClassId, Ty, TyKind};
use std::collections::HashMap;

/// A `@sync` class an export hands over, as the headers and the library give it.
pub(super) struct ClassPlan {
    pub class: ClassId,
    /// Its C name: the class's own (`World`), an incomplete struct to C.
    pub name: String,
    pub retain: String,
    pub release: String,
    pub methods: Vec<MethodPlan>,
    /// Each `pub` method no C signature carries, and why.
    pub skipped: Vec<(String, String)>,
}

/// One `pub` method C and C++ call.
pub(super) struct MethodPlan {
    pub name: String,
    pub symbol: String,
    /// The C declaration's parameters and result, in the headers' names.
    pub params: Vec<String>,
    pub ret: String,
    /// The C++ method's parameters, what each passes to the C function, and the class its result
    /// is the handle of, if any.
    pub cpp_params: Vec<(String, String)>,
    pub cpp_class_result: Option<String>,
    /// The library's definition.
    pub definition: String,
    /// How the receiver and each further parameter cross, for a backend that writes the definition
    /// itself.
    pub receiver_by_address: bool,
    pub crossings: Vec<Crossed>,
    pub body: String,
}

/// `[BLD-FFI-5a]` — a `@sync` class's functions a library's header declares, for a backend that
/// writes the library itself (`ember_codegen_llvm`): the names the header gives them and how each
/// method's parameters cross. Its count's and end's are `retain` and `release` of the handle.
#[derive(Clone, Debug)]
pub struct ClassExport {
    pub class: ClassId,
    pub retain: String,
    pub release: String,
    pub methods: Vec<MethodExport>,
}

/// One `pub` method the header declares: the C function `symbol`, which calls the method's `body`
/// (through the object's method table when the method is virtual).
#[derive(Clone, Debug)]
pub struct MethodExport {
    pub symbol: String,
    pub body: String,
    /// The method takes the object by reference: its argument is the handle's address.
    pub receiver_by_address: bool,
    pub crossings: Vec<Crossed>,
}

/// How a further parameter of an exported method crosses from C.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Crossed {
    /// Another `@sync` class's handle: the object's address, given as itself or (`by_address`, a
    /// `ref` parameter) by its address; `kept`, the method owns it, so it takes a count of its own.
    Handle { by_address: bool, kept: bool },
    /// A `str` or a span: its address and its length, from which the method's view is made.
    View,
    /// A C-laid-out value the method borrows: C passes the value, the method its address.
    Borrowed,
    /// A value that crosses as itself.
    Value,
}

/// The plans as a backend that writes the library itself reads them.
pub(super) fn exported(plans: &[ClassPlan]) -> Vec<ClassExport> {
    plans
        .iter()
        .map(|plan| ClassExport {
            class: plan.class,
            retain: plan.retain.clone(),
            release: plan.release.clone(),
            methods: plan
                .methods
                .iter()
                .map(|method| MethodExport {
                    symbol: method.symbol.clone(),
                    body: method.body.clone(),
                    receiver_by_address: method.receiver_by_address,
                    crossings: method.crossings.clone(),
                })
                .collect(),
        })
        .collect()
}

/// The package's C symbol stem: its name, as an identifier.
pub(super) fn stem(package_name: &str) -> String {
    let stem: String = package_name.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
    if stem.starts_with(|c: char| c.is_ascii_digit()) { format!("_{stem}") } else { stem }
}

/// What a parameter of a method crosses as: the C declarations it takes, the C++ declaration and
/// what that passes, and the argument the method's own C takes.
struct Crossing {
    params: Vec<String>,
    cpp: (String, String),
    argument: Vec<String>,
    crossed: Crossed,
}

pub(super) fn plan(emitter: &Emitter<'_>, bodies: &[Body], package_name: &str) -> Vec<ClassPlan> {
    let stem = stem(package_name);
    let by_symbol: HashMap<&str, &Body> = bodies.iter().map(|body| (body.symbol.as_str(), body)).collect();
    let names: HashMap<ClassId, String> = emitter.types.class_exports().iter().map(|export| (export.class, class_name(emitter, export.class))).collect();
    let mut plans = Vec::new();
    for export in emitter.types.class_exports() {
        let name = names[&export.class].clone();
        let mut plan = ClassPlan {
            class: export.class,
            name: name.clone(),
            retain: format!("{stem}_{name}_retain"),
            release: format!("{stem}_{name}_release"),
            methods: Vec::new(),
            skipped: Vec::new(),
        };
        for (method, symbol) in &export.methods {
            let Some(body) = by_symbol.get(symbol.as_str()) else { continue };
            match method_plan(emitter, body, &names, &stem, &name, method) {
                Ok(made) => plan.methods.push(made),
                Err(why) => plan.skipped.push((method.clone(), why)),
            }
        }
        plans.push(plan);
    }
    plans
}

/// The class's C name: its own, without its module's path.
fn class_name(emitter: &Emitter<'_>, class: ClassId) -> String {
    let name = emitter.types.class_def(class).name;
    name.as_str().rsplit('.').next().unwrap_or(name.as_str()).to_string()
}

fn method_plan(emitter: &Emitter<'_>, body: &Body, names: &HashMap<ClassId, String>, stem: &str, class: &str, method: &str) -> Result<MethodPlan, String> {
    let symbol = format!("{stem}_{class}_{method}");
    let mut args = body.args();
    let Some((_, receiver)) = args.next() else { return Err("it takes no receiver".to_string()) };
    let receiver_by_address = match emitter.types.kind(receiver.ty) {
        TyKind::Class(_) => false,
        TyKind::Ref { inner, .. } if matches!(emitter.types.kind(*inner), TyKind::Class(_)) => true,
        _ => return Err("its receiver is not the object".to_string()),
    };
    let receiver_argument = if receiver_by_address { "&self" } else { "self" }.to_string();
    let mut crossings = Vec::new();
    let mut params = vec![format!("{class}* self")];
    let mut internal = vec![format!("{} self", emitter.c_type(class_ty(emitter, receiver.ty)))];
    let mut cpp_params = Vec::new();
    let mut arguments = vec![receiver_argument];
    let split = emitter.split_views.get(&body.symbol);
    let mut retains = Vec::new();
    for (index, (_, decl)) in args.enumerate() {
        let index = index + 1;
        let name = decl.name.clone().filter(|name| name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') && name != "self").unwrap_or_else(|| format!("a{index}"));
        let split_here = split.is_some_and(|split| split.get(index) == Some(&true));
        let owned = body.param_modes.get(index) == Some(&ParameterMode::Owned);
        let mut crossing = crossing(emitter, decl.ty, &name, names, split_here)?;
        if owned && let TyKind::Class(_) = emitter.types.kind(decl.ty) {
            // The method keeps the handle it is given: a count of its own.
            retains.push(format!("    {RT}obj_retain((struct {RT}obj_header*){name});"));
            if let Crossed::Handle { kept, .. } = &mut crossing.crossed {
                *kept = true;
            }
        }
        crossings.push(crossing.crossed);
        internal.extend(crossing.params.iter().map(|param| internal_param(emitter, decl.ty, param)));
        params.extend(crossing.params);
        cpp_params.push(crossing.cpp);
        arguments.extend(crossing.argument);
    }
    let ret_ty = body.return_ty();
    let (ret, internal_ret, cpp_class_result) = match emitter.types.kind(ret_ty) {
        TyKind::Void | TyKind::Never => ("void".to_string(), "void".to_string(), None),
        TyKind::Class(other) => match names.get(other) {
            Some(other_name) => (format!("{other_name}*"), emitter.c_type(ret_ty), Some(other_name.clone())),
            None => return Err(format!("it returns a `{}`, whose class is not `@sync`", emitter.types.display(ret_ty))),
        },
        _ if plain(emitter, ret_ty) => (emitter.c_member_type(ret_ty), emitter.c_member_type(ret_ty), None),
        _ => return Err(format!("it returns a `{}`, which C has no form of", emitter.types.display(ret_ty))),
    };
    let call = match body.class_virtual_slot {
        // Through the object's method table: an override's body when the object's class has one.
        // The table and its slot's types are those of the class that declares the method, which
        // each argument a pointer is given as (the call sites' rule).
        Some(slot) if let Some(owner) = body.class_owner => {
            let mut declaring = owner;
            let mut current = Some(owner);
            while let Some(class) = current {
                if emitter.virtual_signatures.contains_key(&(class, slot)) {
                    declaring = class;
                }
                current = emitter.types.class_def(class).base;
            }
            // The receiver as the declaring class's slot takes it (an override's own is a subclass's).
            let receiver = emitter.virtual_signatures.get(&(declaring, slot)).and_then(|signature| signature.params.first()).map(|ty| emitter.c_type(*ty));
            let mut passed = arguments.clone();
            if let (Some(c), Some(first)) = (receiver, passed.first_mut())
                && c.ends_with('*')
            {
                *first = format!("({c})({first})");
            }
            let table = emitter.class_vtable(declaring);
            format!("(((const struct {table}*)(((const {RT}obj_header*)(self))->ti->vtable))->slot{slot})({})", passed.join(", "))
        }
        _ => format!("{}({})", body.symbol, arguments.join(", ")),
    };
    let returned = if ret == "void" { format!("{call};") } else { format!("return {call};") };
    let definition = format!(
        "/* [BLD-FFI-5a] `{class}.{method}`, for C and C++. */\n{internal_ret} {symbol}({}) {{\n    {RT}rt_thread_attach();\n{}    {returned}\n}}\n",
        internal.join(", "),
        retains.iter().map(|line| format!("{line}\n")).collect::<String>()
    );
    Ok(MethodPlan {
        name: method.to_string(),
        symbol,
        params,
        ret,
        cpp_params,
        cpp_class_result,
        definition,
        receiver_by_address,
        crossings,
        body: body.symbol.clone(),
    })
}

/// A class handle's type, through a `ref`.
fn class_ty(emitter: &Emitter<'_>, ty: Ty) -> Ty {
    match emitter.types.kind(ty) {
        TyKind::Ref { inner, .. } => *inner,
        _ => ty,
    }
}

/// A parameter's declaration in the library's own C: a class's handle by its object's struct.
fn internal_param(emitter: &Emitter<'_>, ty: Ty, header: &str) -> String {
    match emitter.types.kind(class_ty(emitter, ty)) {
        TyKind::Class(_) => {
            let name = header.rsplit(' ').next().unwrap_or(header).trim_start_matches('*');
            format!("{} {name}", emitter.c_type(class_ty(emitter, ty)))
        }
        _ => header.to_string(),
    }
}

/// Whether a value crosses to C as itself: a scalar, a raw pointer, `cstr`, an `extern fn`, or a
/// C-laid-out value with nothing to drop.
fn plain(emitter: &Emitter<'_>, ty: Ty) -> bool {
    match emitter.types.kind(ty) {
        TyKind::Bool | TyKind::Char | TyKind::Int(_) | TyKind::Uint(_) | TyKind::Float(_) | TyKind::CLongDouble | TyKind::Ptr { .. } | TyKind::CStr => true,
        TyKind::Fn { abi: Some(_), .. } => true,
        TyKind::Struct(_) | TyKind::Enum(_) => emitter.types.is_ffi_safe(ty) && !emitter.types.needs_drop(ty),
        _ => false,
    }
}

fn crossing(emitter: &Emitter<'_>, ty: Ty, name: &str, names: &HashMap<ClassId, String>, split: bool) -> Result<Crossing, String> {
    let shown = || emitter.types.display(ty);
    // Another class's object: its handle, which the method borrows.
    if let TyKind::Class(other) = *emitter.types.kind(class_ty(emitter, ty)) {
        let Some(other_name) = names.get(&other) else {
            return Err(format!("it takes a `{}`, whose class is not `@sync`", emitter.types.display(class_ty(emitter, ty))));
        };
        let by_address = matches!(emitter.types.kind(ty), TyKind::Ref { .. });
        return Ok(Crossing {
            params: vec![format!("{other_name}* {name}")],
            cpp: (format!("const {other_name}& {name}"), format!("{name}.get()")),
            argument: vec![if by_address { format!("&{name}") } else { name.to_string() }],
            crossed: Crossed::Handle { by_address, kept: false },
        });
    }
    match emitter.types.kind(ty) {
        // A view: its address and its length, from which the method's view is made.
        TyKind::Str | TyKind::Span { .. } => {
            let (element, view, mutable) = match emitter.types.kind(ty) {
                TyKind::Str => ("uint8_t".to_string(), format!("{RT}str"), false),
                TyKind::Span { elem, mutable } => {
                    if !plain(emitter, *elem) {
                        return Err(format!("it takes a `{}`, whose elements C has no form of", shown()));
                    }
                    (emitter.c_member_type(*elem), if *mutable { format!("{RT}mutspan") } else { format!("{RT}span") }, *mutable)
                }
                _ => unreachable!("a view"),
            };
            let pointer = format!("{}{element}*", if mutable { "" } else { "const " });
            let cpp = if matches!(emitter.types.kind(ty), TyKind::Str) {
                (format!("std::string_view {name}"), format!("reinterpret_cast<const uint8_t*>({name}.data()), {name}.size()"))
            } else {
                (format!("std::span<{}{element}> {name}", if mutable { "" } else { "const " }), format!("{name}.data(), {name}.size()"))
            };
            let argument = if split {
                vec![name.to_string(), format!("{name}_len")]
            } else {
                vec![format!("({view}){{ {name}, {name}_len }}")]
            };
            Ok(Crossing { params: vec![format!("{pointer} {name}"), format!("size_t {name}_len")], cpp, argument, crossed: Crossed::View })
        }
        // A value the method borrows by its address.
        TyKind::Ref { inner, mutable: false } if plain(emitter, *inner) => {
            let c = emitter.c_member_type(*inner);
            Ok(Crossing {
                params: vec![format!("{c} {name}")],
                cpp: (format!("{c} {name}"), name.to_string()),
                argument: vec![format!("&{name}")],
                crossed: Crossed::Borrowed,
            })
        }
        _ if plain(emitter, ty) => {
            let c = emitter.c_member_type(ty);
            Ok(Crossing {
                params: vec![format!("{c} {name}")],
                cpp: (format!("{c} {name}"), name.to_string()),
                argument: vec![name.to_string()],
                crossed: Crossed::Value,
            })
        }
        _ => Err(format!("it takes a `{}`, which C has no form of", shown())),
    }
}

/// The library's definitions: each class's count and end, and its methods.
pub(super) fn definitions(emitter: &Emitter<'_>, plans: &[ClassPlan]) -> String {
    let mut out = String::new();
    for plan in plans {
        let object = format!("struct {}*", emitter.object_struct(plan.class));
        out.push_str(&format!(
            "\n/* [BLD-FFI-5a] `{0}` for C and C++: a handle's count, taken and given back. */\nvoid {1}({object} self) {{ {RT}obj_retain((struct {RT}obj_header*)self); }}\nvoid {2}({object} self) {{ {RT}rt_thread_attach(); {RT}obj_release((struct {RT}obj_header*)self); }}\n",
            plan.name, plan.retain, plan.release
        ));
        for method in &plan.methods {
            out.push_str(&method.definition);
        }
    }
    out
}
