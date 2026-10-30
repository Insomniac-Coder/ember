//! `[CG-C-9]`, `[CG-LL-3]` — the debugger visualisers of a program the LLVM backend built: a
//! `.natvis` for Visual Studio and WinDbg, a GDB pretty-printer script and an LLDB formatter script,
//! each showing the program's values as Ember values, as the C backend's do (`ember_codegen_c`'s
//! `visualisers`): a `str` or `String` as its text, an `Array`, `Span` or `MutSpan` as its
//! elements, an enum with payloads as its variant and payload, a struct, tuple or class object by
//! its Ember name and fields.
//!
//! The debug information names each type (`debug::debug_name`) and types each collection's buffer
//! pointer with its element type, so one entry shows every `Array` whatever it holds; the types
//! the debug information describes (`debug::Shown`) have one each.

use ember_codegen_c::Visualisers;

use crate::debug::Shown;

/// The three scripts, for the types `shown` describes: each once, though several units described it
/// (`[BLD-4]`).
pub(crate) fn render(shown: &[Shown]) -> Visualisers {
    let mut seen = std::collections::BTreeSet::new();
    let shown: Vec<&Shown> = shown
        .iter()
        .filter(|item| seen.insert(match item {
            Shown::Fields { name, .. } | Shown::Enum { name, .. } => name.as_str(),
        }))
        .collect();
    Visualisers { natvis: natvis(&shown), gdb: python(&shown, GDB), lldb: python(&shown, LLDB) }
}

/// The collections whose buffers a visualiser shows as elements, by the start of their names.
const COLLECTIONS: [&str; 3] = ["Array", "Span", "MutSpan"];

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

/// Fields shown inline at `prefix` (`payload.Some.`): `a: {a}, b: {b}`, a positional one bare.
fn inline(fields: &[String], prefix: &str) -> String {
    let shown: Vec<String> = fields
        .iter()
        .map(|field| {
            let value = format!("{{{}}}", xml(&format!("{prefix}{field}")));
            if positional(field) { value } else { format!("{}: {value}", literal(field)) }
        })
        .collect();
    shown.join(", ")
}

fn natvis(shown: &[&Shown]) -> String {
    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<!-- [CG-C-9] The program's values as Ember values, for Visual Studio and WinDbg. Written by the build. -->\n<AutoVisualizer xmlns=\"http://schemas.microsoft.com/vstudio/debugger/natvis/2010\">\n",
    );
    for text in ["str", "String"] {
        out.push_str(&format!(
            "  <Type Name=\"{text}\">\n    <DisplayString>{{(char*)ptr,[len]s8}}</DisplayString>\n    <StringView>(char*)ptr,[len]s8</StringView>\n  </Type>\n"
        ));
    }
    for collection in COLLECTIONS {
        out.push_str(&format!(
            "  <Type Name=\"{collection}&lt;*&gt;\">\n    <DisplayString>[{{len}}]</DisplayString>\n    <Expand>\n      <ArrayItems>\n        <Size>len</Size>\n        <ValuePointer>ptr</ValuePointer>\n      </ArrayItems>\n    </Expand>\n  </Type>\n"
        ));
    }
    for item in shown {
        match item {
            Shown::Fields { name, ember, fields, tuple, .. } => {
                let display = if *tuple {
                    format!("({})", inline(fields, ""))
                } else if fields.is_empty() {
                    literal(ember)
                } else {
                    format!("{} {{{{ {} }}}}", literal(ember), inline(fields, ""))
                };
                out.push_str(&format!("  <Type Name=\"{}\">\n    <DisplayString>{display}</DisplayString>\n    <Expand>\n", xml(name)));
                for field in fields {
                    out.push_str(&format!("      <Item Name=\"{}\">{}</Item>\n", xml(field), xml(field)));
                }
                out.push_str("    </Expand>\n  </Type>\n");
            }
            Shown::Enum { name, variants, .. } => {
                out.push_str(&format!("  <Type Name=\"{}\">\n", xml(name)));
                for (discriminant, variant, fields) in variants {
                    let display = if fields.is_empty() {
                        literal(variant)
                    } else {
                        format!("{}({})", literal(variant), inline(fields, &format!("payload.{variant}.")))
                    };
                    out.push_str(&format!("    <DisplayString Condition=\"tag == {discriminant}\">{display}</DisplayString>\n"));
                }
                out.push_str("    <Expand>\n");
                for (discriminant, variant, fields) in variants {
                    for field in fields {
                        out.push_str(&format!(
                            "      <Item Name=\"{}\" Condition=\"tag == {discriminant}\">{}</Item>\n",
                            xml(field),
                            xml(&format!("payload.{variant}.{field}"))
                        ));
                    }
                }
                out.push_str("    </Expand>\n  </Type>\n");
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

fn py_list(items: &[String]) -> String {
    format!("[{}]", items.iter().map(|item| py(item)).collect::<Vec<_>>().join(", "))
}

/// The scripts' common description of the types, `TYPES`, by the name the debug information gives
/// each, and the collections' names, `COLLECTIONS`.
fn python(shown: &[&Shown], template: &str) -> String {
    let mut entries = Vec::new();
    for item in shown {
        entries.push(match item {
            Shown::Fields { name, ember, fields, class, .. } => format!(
                "    {}: {{\"kind\": \"fields\", \"name\": {}, \"fields\": {}, \"class\": {}}},",
                py(name),
                py(ember),
                py_list(fields),
                if *class { "True" } else { "False" }
            ),
            Shown::Enum { name, ember, variants } => {
                let variants: Vec<String> = variants
                    .iter()
                    .map(|(discriminant, variant, fields)| format!("{discriminant}: ({}, {})", py(variant), py_list(fields)))
                    .collect();
                format!("    {}: {{\"kind\": \"enum\", \"name\": {}, \"variants\": {{{}}}}},", py(name), py(ember), variants.join(", "))
            }
        });
    }
    let collections: Vec<String> = COLLECTIONS.iter().map(|collection| py(&format!("{collection}["))).collect();
    let types = format!("TYPES = {{\n{}\n}}\nCOLLECTIONS = ({},)\n", entries.join("\n"), collections.join(", "));
    template.replace("@TYPES@\n", &types)
}

/// The GDB script: printers for the types `TYPES` describes, the texts, the collections and class
/// handles.
const GDB: &str = r#"# [CG-C-9] GDB pretty-printers showing this Ember program's values as Ember values. Written by
# the build beside the program as `<program>-gdb.py`, which GDB loads with the program when its
# auto-load safe path allows; otherwise `source` this file.
import gdb

@TYPES@

def _name(type_):
    return type_.tag or type_.name or ""


def _text(value):
    length = int(value["len"])
    if length == 0:
        return '""'
    data = bytes(gdb.selected_inferior().read_memory(int(value["ptr"]), length))
    return '"' + data.decode("utf-8", "replace") + '"'


class _Text:
    def __init__(self, value):
        self.value = value

    def to_string(self):
        return _text(self.value)


class _Elements:
    def __init__(self, value):
        self.value = value

    def to_string(self):
        return "[" + str(int(self.value["len"])) + "]"

    def display_hint(self):
        return "array"

    def children(self):
        pointer = self.value["ptr"]
        for index in range(int(self.value["len"])):
            yield "[%d]" % index, (pointer + index).dereference()


class _Fields:
    def __init__(self, value, described):
        self.value, self.described = value, described

    def to_string(self):
        return self.described["name"]

    def children(self):
        for name in self.described["fields"]:
            yield name, self.value[name]


class _Enum:
    def __init__(self, value, described):
        self.value = value
        self.variant = described["variants"].get(int(value["tag"]), ("?", []))

    def to_string(self):
        return self.variant[0]

    def children(self):
        if not self.variant[1]:
            return
        payload = self.value["payload"][self.variant[0]]
        for name in self.variant[1]:
            yield name, payload[name]


class _Handle:
    """A class handle: the object it points at, or `null`."""

    def __init__(self, value, described):
        self.value, self.described = value, described

    def to_string(self):
        return "null" if int(self.value) == 0 else self.described["name"]

    def children(self):
        if int(self.value) == 0:
            return
        target = self.value.dereference()
        for name in self.described["fields"]:
            yield name, target[name]


def _lookup(value):
    stripped = value.type.strip_typedefs()
    if stripped.code == gdb.TYPE_CODE_PTR:
        described = TYPES.get(_name(stripped.target().strip_typedefs()))
        if described is not None and described["kind"] == "fields" and described["class"]:
            return _Handle(value, described)
        return None
    name = _name(stripped)
    if name in ("str", "String"):
        return _Text(value)
    if name.startswith(COLLECTIONS):
        return _Elements(value)
    described = TYPES.get(name)
    if described is None:
        return None
    if described["kind"] == "enum":
        return _Enum(value, described)
    return _Fields(value, described)


_objfile = gdb.current_objfile()
(_objfile.pretty_printers if _objfile is not None else gdb.pretty_printers).append(_lookup)
"#;

/// The LLDB script: summaries and synthetic children for the texts, the collections and the types
/// `TYPES` describes.
const LLDB: &str = r#"# [CG-C-9] LLDB formatters showing this Ember program's values as Ember values. Written by the
# build beside the program as `<program>.lldb.py`: `command script import <this file>`.
import re

import lldb

@TYPES@

def _key(value):
    name = value.GetType().GetUnqualifiedType().GetName()
    return name[len("struct "):] if name.startswith("struct ") else name


def _length(value):
    return value.GetChildMemberWithName("len").GetValueAsUnsigned()


def _text(value):
    length = _length(value)
    if length == 0:
        return '""'
    error = lldb.SBError()
    data = value.GetProcess().ReadMemory(value.GetChildMemberWithName("ptr").GetValueAsUnsigned(), length, error)
    return '"' + (data.decode("utf-8", "replace") if error.Success() else "?") + '"'


def summary(value, internal_dict):
    key = _key(value)
    if key in ("str", "String"):
        return _text(value)
    if key.startswith(COLLECTIONS):
        return "[" + str(_length(value)) + "]"
    described = TYPES.get(key)
    if described is None:
        return None
    if described["kind"] == "enum":
        tag = value.GetChildMemberWithName("tag").GetValueAsUnsigned()
        return described["variants"].get(tag, ("?", []))[0]
    return described["name"]


class Children:
    def __init__(self, value, internal_dict):
        self.value = value
        self.items = []

    def update(self):
        key = _key(self.value)
        self.items = []
        if key.startswith(COLLECTIONS):
            pointer = self.value.GetChildMemberWithName("ptr")
            element = pointer.GetType().GetPointeeType()
            length = _length(self.value)
            if length > 0 and element.IsValid():
                elements = self.value.CreateValueFromAddress("elements", pointer.GetValueAsUnsigned(), element.GetArrayType(length))
                self.items = [elements.GetChildAtIndex(index) for index in range(length)]
            return False
        described = TYPES.get(key)
        if described is None:
            return False
        if described["kind"] == "enum":
            tag = self.value.GetChildMemberWithName("tag").GetValueAsUnsigned()
            variant, fields = described["variants"].get(tag, ("?", []))
            payload = self.value.GetChildMemberWithName("payload").GetChildMemberWithName(variant)
            self.items = [payload.GetChildMemberWithName(field) for field in fields]
        else:
            self.items = [self.value.GetChildMemberWithName(field) for field in described["fields"]]
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
    category = debugger.CreateCategory("ember")
    # By regular expression: a name such as `Array[i64]` is not one LLDB matches exactly.
    patterns = ["^str$", "^String$"] + ["^" + re.escape(collection) for collection in COLLECTIONS]
    patterns += ["^" + re.escape(name) + "$" for name in TYPES]
    for index, pattern in enumerate(patterns):
        named = lldb.SBTypeNameSpecifier(pattern, True)
        category.AddTypeSummary(named, lldb.SBTypeSummary.CreateWithFunctionName(module + ".summary"))
        if index >= 2:
            category.AddTypeSynthetic(named, lldb.SBTypeSynthetic.CreateWithClassName(module + ".Children"))
    category.SetEnabled(True)
"#;
