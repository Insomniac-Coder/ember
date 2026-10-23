//! `[CG-C-9]` — the build's debugger visualisers: a `.natvis` for Visual Studio and WinDbg, a GDB
//! pretty-printer script and an LLDB formatter script, each showing the program's values as Ember
//! values: a struct or a class object by its Ember name and fields, an enum (`Option` and `Result`
//! among them) as its variant and payload, a unit enum as its variant, a `str` or `String` as its
//! text, and an `Array`, `Span` or `MutSpan` as its elements.
//!
//! All three are written from one description of the types the C defines (`describe`). In C one
//! struct serves every `Array[T]` and `String` (`ember_vec`) and every view (`ember_span`), so a
//! collection's elements are known where its Ember type is: a field of a struct, a class or a
//! variant, and a local, which is declared as its type's own struct around the buffer (ODR-178,
//! `collection_wrappers`): the debugger finds it by that name, as CodeView keeps no `typedef`. A
//! parameter holding a collection is the buffer itself (the C signature is not changed), shown by
//! its length.

use super::{member, Emitter, TypeNode};
use ember_types::{Ty, TyKind};

/// The three scripts.
#[derive(Clone, Debug, Default)]
pub struct Visualisers {
    pub natvis: String,
    pub gdb: String,
    pub lldb: String,
}

/// How a debugger shows a value of one Ember type.
#[derive(Clone, Debug)]
enum Shown {
    /// By its own C type (a number, or a type described here).
    Value,
    /// `str`: `ember_str`, its text.
    Str,
    /// `String`: an `ember_vec` of UTF-8 bytes, its text.
    Text,
    /// `Array[T]`, `Span[T]`, `MutSpan[T]`: its elements, of the C type given.
    Elements(String),
    /// A unit enum: an integer, which this C type name (the enum's) describes.
    Unit(String),
}

/// A field as a debugger shows it: its Ember name, where it is from the value (`payload.Some._0`)
/// and how it is shown.
#[derive(Clone, Debug)]
struct Field {
    name: String,
    at: String,
    shown: Shown,
}

#[derive(Clone, Debug)]
enum Described {
    /// A struct, a tuple (shown as one: `(1, 2)`), or a class's object (its header left out).
    Fields { c: String, name: String, fields: Vec<Field>, tuple: bool },
    /// An enum with payloads: `tag` and a union of its variants' payloads.
    Enum { c: String, name: String, variants: Vec<(i128, String, Vec<Field>)> },
    /// A unit-only enum, an integer type.
    Unit { c: String, name: String, variants: Vec<(i128, String)> },
    /// `[CG-C-9]` (ODR-178) — a collection local's struct: its buffer, `v`, shown as its text or
    /// its elements.
    Collection { c: String, name: String, shown: Shown },
}

/// `[CG-C-9]` — the visualisers of the types `emitter` defined.
pub(super) fn render(emitter: &Emitter<'_>) -> Visualisers {
    let described = describe(emitter);
    Visualisers { natvis: natvis(&described), gdb: python(&described, GDB), lldb: python(&described, LLDB) }
}

fn describe(emitter: &Emitter<'_>) -> Vec<Described> {
    let types = emitter.types;
    let mut out = Vec::new();
    for &node in &emitter.order {
        let c = emitter.node_name(node);
        match node {
            TypeNode::Struct(id) => {
                // Compiler-known wrappers are their payload's (`Box`, `Shared`, `Weak`, `Cell`), and a
                // vector of `std.simd` its lanes: a debugger shows them as C does.
                if emitter.box_inner_id(id).is_some()
                    || emitter.weak_inner_id(id).is_some()
                    || emitter.shared_inner_id(id).is_some()
                    || emitter.cell_inner_id(id).is_some()
                    || emitter.simd_shape_of(id).is_some()
                {
                    continue;
                }
                let Some(ty) = types.find(&TyKind::Struct(id)) else { continue };
                let fields = types
                    .struct_def(id)
                    .fields
                    .iter()
                    .map(|field| Field { name: field.name.to_string(), at: member(field.name), shown: shown(emitter, field.ty) })
                    .collect();
                out.push(Described::Fields { c, name: types.display(ty), fields, tuple: false });
            }
            TypeNode::Class(id) => {
                let mut fields = Vec::new();
                let mut index = 0;
                while let Some(field) = types.class_field_at(id, index) {
                    fields.push(Field { name: field.name.to_string(), at: member(field.name), shown: shown(emitter, field.ty) });
                    index += 1;
                }
                out.push(Described::Fields { c, name: types.class_def(id).name.to_string(), fields, tuple: false });
            }
            TypeNode::Enum(id) => {
                let def = types.enum_def(id);
                let Some(ty) = types.find(&TyKind::Enum(id)) else { continue };
                let name = types.display(ty);
                if types.enum_is_bare(id) {
                    out.push(Described::Unit { c, name, variants: def.variants.iter().map(|v| (v.discriminant, v.name.to_string())).collect() });
                    continue;
                }
                // `[TYP-13]` — an `Option` with a niche is its payload, which shows itself.
                if types.option_niche(id).is_some() {
                    continue;
                }
                let variants = def
                    .variants
                    .iter()
                    .map(|variant| {
                        let fields = variant
                            .fields
                            .iter()
                            .filter(|field| !matches!(types.kind(field.ty), TyKind::Void))
                            .map(|field| Field {
                                name: field.name.to_string(),
                                at: format!("payload.{}.{}", member(variant.name), member(field.name)),
                                shown: shown(emitter, field.ty),
                            })
                            .collect();
                        (variant.discriminant, variant.name.to_string(), fields)
                    })
                    .collect();
                out.push(Described::Enum { c, name, variants });
            }
            TypeNode::Structural(ty) => {
                let TyKind::Tuple(items) = types.kind(ty) else { continue };
                let fields = items
                    .iter()
                    .enumerate()
                    .map(|(index, &item)| Field { name: format!("_{index}"), at: format!("_{index}"), shown: shown(emitter, item) })
                    .collect();
                out.push(Described::Fields { c, name: types.display(ty), fields, tuple: true });
            }
        }
    }
    for (&ty, c) in emitter.collection_wrappers() {
        let shown = shown(emitter, ty);
        if matches!(shown, Shown::Text | Shown::Elements(_)) {
            out.push(Described::Collection { c: c.clone(), name: types.display(ty), shown });
        }
    }
    out
}

/// How a value of `ty` is shown where its Ember type is known.
fn shown(emitter: &Emitter<'_>, ty: Ty) -> Shown {
    let types = emitter.types;
    match types.kind(ty) {
        TyKind::Str => Shown::Str,
        TyKind::Vec { text: true, .. } => Shown::Text,
        TyKind::Vec { elem, .. } | TyKind::Span { elem, .. } if !matches!(types.kind(*elem), TyKind::Void) => {
            Shown::Elements(emitter.c_type(*elem))
        }
        TyKind::Enum(id) if types.enum_is_bare(*id) => Shown::Unit(emitter.c_type(ty)),
        _ => Shown::Value,
    }
}

/// A positional field (`_0`) is shown without its name: `Some(3)`, not `Some(_0: 3)`.
fn positional(name: &str) -> bool {
    name.strip_prefix('_').is_some_and(|rest| !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit()))
}

fn xml(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Literal text in a natvis display string, whose braces are doubled.
fn literal(text: &str) -> String {
    xml(&text.replace('{', "{{").replace('}', "}}"))
}

/// A C type as natvis casts to it: a struct by its tag alone.
fn natvis_type(c: &str) -> String {
    xml(&c.replace("struct ", ""))
}

/// A field's value inside its owner's display string.
fn natvis_inline(field: &Field) -> String {
    let at = xml(&field.at);
    match &field.shown {
        Shown::Value | Shown::Str | Shown::Unit(_) => format!("{{{at}}}"),
        Shown::Text => format!("{{(char*){at}.ptr,[{at}.len]s8}}"),
        Shown::Elements(_) => format!("[{{{at}.len}}]"),
    }
}

/// A field's item in its owner's expansion, under `condition` when given.
fn natvis_item(field: &Field, condition: Option<&str>, units: &[Described]) -> String {
    let at = xml(&field.at);
    let name = xml(&field.name);
    let when = condition.map_or(String::new(), |condition| format!(" Condition=\"{}\"", xml(condition)));
    let both = |extra: &str| match condition {
        Some(condition) => format!(" Condition=\"{} &amp;&amp; {extra}\"", xml(condition)),
        None => format!(" Condition=\"{extra}\""),
    };
    match &field.shown {
        Shown::Value | Shown::Str => format!("      <Item Name=\"{name}\"{when}>{at}</Item>\n"),
        Shown::Text => format!("      <Item Name=\"{name}\"{when}>(char*){at}.ptr,[{at}.len]s8</Item>\n"),
        Shown::Elements(elem) => format!(
            "      <Synthetic Name=\"{name}\"{when}>\n        <DisplayString>[{{{at}.len}}]</DisplayString>\n        <Expand>\n          <ArrayItems>\n            <Size>{at}.len</Size>\n            <ValuePointer>({}*){at}.ptr</ValuePointer>\n          </ArrayItems>\n        </Expand>\n      </Synthetic>\n",
            natvis_type(elem)
        ),
        Shown::Unit(c) => {
            let variants = units.iter().find_map(|described| match described {
                Described::Unit { c: unit, variants, .. } if unit == c => Some(variants),
                _ => None,
            });
            match variants {
                Some(variants) => variants
                    .iter()
                    .map(|(discriminant, variant)| {
                        format!(
                            "      <Synthetic Name=\"{name}\"{}>\n        <DisplayString>{}</DisplayString>\n      </Synthetic>\n",
                            both(&format!("{at} == {discriminant}")),
                            literal(variant)
                        )
                    })
                    .collect(),
                None => format!("      <Item Name=\"{name}\"{when}>{at}</Item>\n"),
            }
        }
    }
}

fn natvis(described: &[Described]) -> String {
    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<!-- [CG-C-9] The program's values as Ember values, for Visual Studio and WinDbg. Written by the build. -->\n<AutoVisualizer xmlns=\"http://schemas.microsoft.com/vstudio/debugger/natvis/2010\">\n",
    );
    let runtime = ember_branding::RUNTIME_PREFIX;
    out.push_str(&format!(
        "  <Type Name=\"{runtime}str\">\n    <DisplayString>{{(char*)ptr,[len]s8}}</DisplayString>\n    <StringView>(char*)ptr,[len]s8</StringView>\n  </Type>\n"
    ));
    for buffer in ["vec", "span", "mutspan"] {
        out.push_str(&format!(
            "  <Type Name=\"{runtime}{buffer}\">\n    <DisplayString>[{{len}}]</DisplayString>\n    <Expand>\n      <Item Name=\"len\">len</Item>\n    </Expand>\n  </Type>\n"
        ));
    }
    for item in described {
        match item {
            Described::Fields { c, name, fields, tuple } => {
                let inline: Vec<String> = fields
                    .iter()
                    .map(|field| if positional(&field.name) { natvis_inline(field) } else { format!("{}: {}", literal(&field.name), natvis_inline(field)) })
                    .collect();
                let shown = if *tuple {
                    format!("({})", inline.join(", "))
                } else if fields.is_empty() {
                    literal(name)
                } else {
                    format!("{} {{{{ {} }}}}", literal(name), inline.join(", "))
                };
                out.push_str(&format!("  <Type Name=\"{}\">\n    <DisplayString>{shown}</DisplayString>\n    <Expand>\n", xml(c)));
                for field in fields {
                    out.push_str(&natvis_item(field, None, described));
                }
                out.push_str("    </Expand>\n  </Type>\n");
            }
            Described::Enum { c, name: _, variants } => {
                out.push_str(&format!("  <Type Name=\"{}\">\n", xml(c)));
                for (discriminant, variant, fields) in variants {
                    let shown = if fields.is_empty() {
                        literal(variant)
                    } else {
                        let inline: Vec<String> = fields
                            .iter()
                            .map(|field| if positional(&field.name) { natvis_inline(field) } else { format!("{}: {}", literal(&field.name), natvis_inline(field)) })
                            .collect();
                        format!("{}({})", literal(variant), inline.join(", "))
                    };
                    out.push_str(&format!("    <DisplayString Condition=\"tag == {discriminant}\">{shown}</DisplayString>\n"));
                }
                out.push_str("    <Expand>\n");
                for (discriminant, _, fields) in variants {
                    for field in fields {
                        out.push_str(&natvis_item(field, Some(&format!("tag == {discriminant}")), described));
                    }
                }
                out.push_str("    </Expand>\n  </Type>\n");
            }
            // A unit enum is an integer type in C, which no visualiser can tell from another; it is
            // shown by its variant where it is a field (`natvis_item`).
            Described::Unit { .. } => {}
            Described::Collection { c, shown, .. } => {
                out.push_str(&format!("  <Type Name=\"{}\">\n", xml(c)));
                match shown {
                    Shown::Text => out.push_str("    <DisplayString>{(char*)v.ptr,[v.len]s8}</DisplayString>\n    <StringView>(char*)v.ptr,[v.len]s8</StringView>\n"),
                    Shown::Elements(elem) => out.push_str(&format!(
                        "    <DisplayString>[{{v.len}}]</DisplayString>\n    <Expand>\n      <ArrayItems>\n        <Size>v.len</Size>\n        <ValuePointer>({}*)v.ptr</ValuePointer>\n      </ArrayItems>\n    </Expand>\n",
                        natvis_type(elem)
                    )),
                    _ => {}
                }
                out.push_str("  </Type>\n");
            }
        }
    }
    out.push_str("</AutoVisualizer>\n");
    out
}

/// A Python string literal.
fn py(text: &str) -> String {
    format!("{text:?}")
}

fn py_shown(shown: &Shown) -> String {
    match shown {
        Shown::Value => "(\"value\",)".to_string(),
        Shown::Str => "(\"str\",)".to_string(),
        Shown::Text => "(\"text\",)".to_string(),
        Shown::Elements(elem) => format!("(\"elements\", {})", py(elem)),
        Shown::Unit(c) => format!("(\"unit\", {})", py(c)),
    }
}

fn py_fields(fields: &[Field]) -> String {
    let items: Vec<String> =
        fields.iter().map(|field| format!("({}, {}, {})", py(&field.name), py(&field.at), py_shown(&field.shown))).collect();
    format!("[{}]", items.join(", "))
}

/// The scripts' common description of the types, `TYPES`, by C type name.
fn python(described: &[Described], template: &str) -> String {
    let mut entries = Vec::new();
    for item in described {
        entries.push(match item {
            Described::Fields { c, name, fields, .. } => {
                format!("    {}: {{\"kind\": \"fields\", \"name\": {}, \"fields\": {}}},", py(c), py(name), py_fields(fields))
            }
            Described::Enum { c, name, variants } => {
                let variants: Vec<String> = variants
                    .iter()
                    .map(|(discriminant, variant, fields)| format!("{discriminant}: ({}, {})", py(variant), py_fields(fields)))
                    .collect();
                format!("    {}: {{\"kind\": \"enum\", \"name\": {}, \"variants\": {{{}}}}},", py(c), py(name), variants.join(", "))
            }
            Described::Unit { c, name, variants } => {
                let variants: Vec<String> = variants.iter().map(|(discriminant, variant)| format!("{discriminant}: {}", py(variant))).collect();
                format!("    {}: {{\"kind\": \"unit\", \"name\": {}, \"variants\": {{{}}}}},", py(c), py(name), variants.join(", "))
            }
            Described::Collection { c, name, shown } => {
                format!("    {}: {{\"kind\": \"collection\", \"name\": {}, \"shown\": {}}},", py(c), py(name), py_shown(shown))
            }
        });
    }
    let types = format!("TYPES = {{\n{}\n}}\n", entries.join("\n"));
    template
        .replace("@TYPES@\n", &types)
        .replace("@RUNTIME@", ember_branding::RUNTIME_PREFIX)
        .replace("@OBJECT@", &ember_branding::object_struct(""))
}

/// The GDB script: printers for the types `TYPES` describes, `str`, the buffers, and class handles.
const GDB: &str = r#"# [CG-C-9] GDB pretty-printers showing this Ember program's values as Ember values. Written by
# the build beside the program as `<program>-gdb.py`, which GDB loads with the program when its
# auto-load safe path allows; otherwise `source` this file.
import gdb

@TYPES@

def _text(pointer, length):
    length = int(length)
    if length == 0:
        return '""'
    data = bytes(gdb.selected_inferior().read_memory(int(pointer), length))
    return '"' + data.decode("utf-8", "replace") + '"'


def _pointer_to(c_type):
    return gdb.parse_and_eval("(" + c_type + "*)0").type


def _member(value, path):
    for name in path.split("."):
        value = value[name]
    return value


def _shown(value, shown):
    """A field's value as a child: its text, its elements as an array, or the value itself."""
    kind = shown[0]
    if kind == "text":
        return _text(value["ptr"], value["len"])
    if kind == "elements":
        length = int(value["len"])
        if length == 0:
            return "[]"
        element = _pointer_to(shown[1]).target()
        return value["ptr"].cast(element.array(length - 1).pointer()).dereference()
    if kind == "unit":
        variants = TYPES.get(shown[1], {}).get("variants", {})
        return variants.get(int(value), str(int(value)))
    return value


class _Fields:
    def __init__(self, value, described):
        self.value, self.described = value, described

    def to_string(self):
        return self.described["name"]

    def children(self):
        for name, path, shown in self.described["fields"]:
            yield name, _shown(_member(self.value, path), shown)


class _Enum:
    def __init__(self, value, described):
        self.value, self.described = value, described
        self.variant = described["variants"].get(int(value["tag"]), ("?", []))

    def to_string(self):
        return self.variant[0]

    def children(self):
        for name, path, shown in self.variant[1]:
            yield name, _shown(_member(self.value, path), shown)


class _Unit:
    def __init__(self, value, described):
        self.value, self.described = value, described

    def to_string(self):
        return self.described["variants"].get(int(self.value), str(int(self.value)))


class _Handle:
    """A class handle: the object it points at, or `null`."""

    def __init__(self, value, described):
        self.value, self.described = value, described

    def to_string(self):
        return "null" if int(self.value) == 0 else self.described["name"]

    def children(self):
        if int(self.value) == 0:
            return
        for name, path, shown in self.described["fields"]:
            yield name, _shown(_member(self.value.dereference(), path), shown)


class _Str:
    def __init__(self, value):
        self.value = value

    def to_string(self):
        return _text(self.value["ptr"], self.value["len"])


class _Buffer:
    def __init__(self, value):
        self.value = value

    def to_string(self):
        return "[" + str(int(self.value["len"])) + "]"


class _Collection:
    """A collection local's struct: its buffer `v` as its text or its elements."""

    def __init__(self, value, described):
        self.value, self.described = value, described

    def to_string(self):
        return _shown(self.value["v"], self.described["shown"])


def _lookup(value):
    named = value.type.name
    if named in TYPES and TYPES[named]["kind"] == "unit":
        return _Unit(value, TYPES[named])
    stripped = value.type.strip_typedefs()
    if stripped.code == gdb.TYPE_CODE_PTR:
        target = stripped.target().strip_typedefs()
        described = TYPES.get(target.tag or "")
        if described is not None and described["kind"] == "fields" and target.tag.startswith("@OBJECT@"):
            return _Handle(value, described)
        return None
    tag = stripped.tag or ""
    if tag == "@RUNTIME@str":
        return _Str(value)
    if tag in ("@RUNTIME@vec", "@RUNTIME@span", "@RUNTIME@mutspan"):
        return _Buffer(value)
    described = TYPES.get(tag)
    if described is None:
        return None
    if described["kind"] == "enum":
        return _Enum(value, described)
    if described["kind"] == "fields":
        return _Fields(value, described)
    if described["kind"] == "collection":
        return _Collection(value, described)
    return None


_objfile = gdb.current_objfile()
(_objfile.pretty_printers if _objfile is not None else gdb.pretty_printers).append(_lookup)
"#;

/// The LLDB script: summaries and synthetic children for the types `TYPES` describes.
const LLDB: &str = r#"# [CG-C-9] LLDB formatters showing this Ember program's values as Ember values. Written by the
# build beside the program as `<program>.lldb.py`: `command script import <this file>`.
import lldb

@TYPES@

def _key(value):
    name = value.GetType().GetUnqualifiedType().GetName()
    return name[len("struct "):] if name.startswith("struct ") else name


def _text(value):
    length = value.GetChildMemberWithName("len").GetValueAsUnsigned()
    if length == 0:
        return '""'
    error = lldb.SBError()
    data = value.GetProcess().ReadMemory(value.GetChildMemberWithName("ptr").GetValueAsUnsigned(), length, error)
    return '"' + (data.decode("utf-8", "replace") if error.Success() else "?") + '"'


def _find(value, c_type):
    target = value.GetTarget()
    base = c_type.replace("struct ", "").replace("const ", "").strip()
    stars = len(base) - len(base.rstrip("*"))
    found = target.FindFirstType(base.rstrip("* "))
    for _ in range(stars):
        found = found.GetPointerType()
    return found


def _child(value, name, path, shown):
    field = value.GetValueForExpressionPath("." + path)
    kind = shown[0]
    if kind == "elements":
        length = field.GetChildMemberWithName("len").GetValueAsUnsigned()
        element = _find(value, shown[1])
        if length == 0 or not element.IsValid():
            return field
        address = field.GetChildMemberWithName("ptr").GetValueAsUnsigned()
        return value.CreateValueFromAddress(name, address, element.GetArrayType(length))
    if kind == "text":
        length = field.GetChildMemberWithName("len").GetValueAsUnsigned()
        address = field.GetChildMemberWithName("ptr").GetValueAsUnsigned()
        char = value.GetTarget().GetBasicType(lldb.eBasicTypeChar)
        return value.CreateValueFromAddress(name, address, char.GetArrayType(max(length, 1)))
    return field.Clone(name) if field.IsValid() else field


def _described(value):
    return TYPES.get(_key(value))


def summary(value, internal_dict):
    key = _key(value)
    if key == "@RUNTIME@str":
        return _text(value)
    if key in ("@RUNTIME@vec", "@RUNTIME@span", "@RUNTIME@mutspan"):
        return "[" + str(value.GetChildMemberWithName("len").GetValueAsUnsigned()) + "]"
    described = TYPES.get(key)
    if described is None:
        return None
    if described["kind"] == "unit":
        number = value.GetValueAsUnsigned()
        return described["variants"].get(number, str(number))
    if described["kind"] == "collection":
        buffer = value.GetChildMemberWithName("v")
        if described["shown"][0] == "text":
            return _text(buffer)
        return "[" + str(buffer.GetChildMemberWithName("len").GetValueAsUnsigned()) + "]"
    if described["kind"] == "enum":
        tag = value.GetChildMemberWithName("tag").GetValueAsUnsigned()
        return described["variants"].get(tag, ("?", []))[0]
    return described["name"]


class Children:
    def __init__(self, value, internal_dict):
        self.value = value
        self.items = []

    def update(self):
        described = _described(self.value)
        fields = []
        if described is not None and described["kind"] == "fields":
            fields = described["fields"]
        elif described is not None and described["kind"] == "enum":
            tag = self.value.GetChildMemberWithName("tag").GetValueAsUnsigned()
            fields = described["variants"].get(tag, ("?", []))[1]
        elif described is not None and described["kind"] == "collection" and described["shown"][0] == "elements":
            elements = _child(self.value, "v", "v", described["shown"])
            self.items = [elements.GetChildAtIndex(index) for index in range(elements.GetNumChildren())] if elements.GetType().IsArrayType() else []
            return False
        self.items = [_child(self.value, name, path, shown) for name, path, shown in fields]
        return False

    def num_children(self):
        return len(self.items)

    def get_child_index(self, name):
        for index, item in enumerate(self.items):
            if item.GetName() == name:
                return index
        return -1

    def get_child_at_index(self, index):
        return self.items[index] if 0 <= index < len(self.items) else None

    def has_children(self):
        return True


def __lldb_init_module(debugger, internal_dict):
    module = __name__
    for name in ["@RUNTIME@str", "@RUNTIME@vec", "@RUNTIME@span", "@RUNTIME@mutspan"] + list(TYPES):
        debugger.HandleCommand('type summary add -w ember -F %s.summary "%s"' % (module, name))
    for name, described in TYPES.items():
        if described["kind"] != "unit":
            debugger.HandleCommand('type synthetic add -w ember -l %s.Children "%s"' % (module, name))
    debugger.HandleCommand("type category enable ember")
"#;
