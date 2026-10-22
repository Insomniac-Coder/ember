//! `[CG-LL-3]`, `[RT-4]`, `[CG-C-10]` — the module's debug information: CodeView on Windows, DWARF
//! elsewhere, in every profile (a release panic still names its frames). Each function made from a
//! MIR body is a `DISubprogram` named by its symbol, as the C backend's C function is, and each of
//! its instructions carries the Ember line and column of the statement it was made from, as the C
//! backend's `#line`s give them: the runtime's backtrace finds a frame's function and call line in
//! the program's symbols. Line tables only for now: a debugger steps through Ember lines and names
//! functions; locals are not described yet.

use std::collections::HashMap;
use std::fmt::Write as _;

use ember_span::{FileId, SourceMap, Span};
use ember_types::{Ty, TyKind};

use crate::abi::Target;
use crate::escape;
use crate::types::Types;

pub(crate) struct Debug {
    /// The metadata nodes, by number.
    nodes: Vec<String>,
    unit: usize,
    subroutine: usize,
    files: HashMap<FileId, usize>,
    /// `[TST-5]` — in the `debug` profile: locals and their types too, by their Ember names.
    pub full: bool,
    /// Each type's description, and each basic one's by name.
    types: HashMap<Ty, usize>,
    basics: HashMap<String, usize>,
}

impl Debug {
    pub(crate) fn new(map: &SourceMap, module_name: &str, optimized: bool, full: bool) -> Debug {
        let mut debug = Debug {
            nodes: Vec::new(),
            unit: 0,
            subroutine: 0,
            files: HashMap::new(),
            full,
            types: HashMap::new(),
            basics: HashMap::new(),
        };
        let main = map.files().next().map(|file| file.path.display().to_string().replace('\\', "/")).unwrap_or_else(|| module_name.to_string());
        let (directory, name) = split(&main);
        let file = debug.node(format!("!DIFile(filename: \"{}\", directory: \"{}\")", escape(name.as_bytes()), escape(directory.as_bytes())));
        let kind = if full { "FullDebug" } else { "LineTablesOnly" };
        debug.unit = debug.node(format!(
            "distinct !DICompileUnit(language: DW_LANG_C99, file: !{file}, producer: \"Ember\", isOptimized: {optimized}, runtimeVersion: 0, emissionKind: {kind})"
        ));
        let types = debug.node("!{null}".to_string());
        debug.subroutine = debug.node(format!("!DISubroutineType(types: !{types})"));
        debug
    }

    fn node(&mut self, text: String) -> usize {
        self.nodes.push(text);
        self.nodes.len() - 1
    }

    fn file(&mut self, map: &SourceMap, id: FileId) -> usize {
        if let Some(node) = self.files.get(&id) {
            return *node;
        }
        let path = map.artefact_path(id);
        let (directory, name) = split(&path);
        let node = self.node(format!("!DIFile(filename: \"{}\", directory: \"{}\")", escape(name.as_bytes()), escape(directory.as_bytes())));
        self.files.insert(id, node);
        node
    }

    /// The `DISubprogram` of the function `symbol` declared at `span`, and its line.
    pub(crate) fn subprogram(&mut self, map: &SourceMap, symbol: &str, span: Span, optimized: bool) -> (usize, u32) {
        let (file, line) = if span.is_dummy() || map.get(span.file).is_none() {
            (self.files.values().next().copied().unwrap_or(0), 0)
        } else {
            (self.file(map, span.file), map.line_col(span).line)
        };
        let flags = if optimized { "DISPFlagDefinition | DISPFlagOptimized" } else { "DISPFlagDefinition" };
        let node = self.node(format!(
            "distinct !DISubprogram(name: \"{}\", scope: !{file}, file: !{file}, line: {line}, type: !{}, scopeLine: {line}, spFlags: {flags}, unit: !{})",
            escape(symbol.as_bytes()),
            self.subroutine,
            self.unit
        ));
        (node, line)
    }

    /// A location in `scope`.
    pub(crate) fn location(&mut self, line: u32, col: u32, scope: usize) -> usize {
        self.node(format!("!DILocation(line: {line}, column: {col}, scope: !{scope})"))
    }

    /// `[TST-5]` — the local `name` of `scope` (parameter `arg`, from 1, where it is one),
    /// declared at `span`, of the type `ty` describes.
    pub(crate) fn variable(&mut self, map: &SourceMap, name: &str, arg: Option<usize>, scope: usize, span: Span, ty: usize) -> usize {
        let (file, line) = if span.is_dummy() || map.get(span.file).is_none() {
            (0, 0)
        } else {
            (self.file(map, span.file), map.line_col(span).line)
        };
        let arg = arg.map_or(String::new(), |arg| format!(", arg: {arg}"));
        self.node(format!(
            "!DILocalVariable(name: \"{}\"{arg}, scope: !{scope}, file: !{file}, line: {line}, type: !{ty})",
            escape(name.as_bytes())
        ))
    }

    fn basic(&mut self, name: &str, bits: u64, encoding: &str) -> usize {
        if let Some(node) = self.basics.get(name) {
            return *node;
        }
        let node = self.node(format!("!DIBasicType(name: \"{}\", size: {bits}, encoding: {encoding})", escape(name.as_bytes())));
        self.basics.insert(name.to_string(), node);
        node
    }

    fn pointer(&mut self, base: usize) -> usize {
        self.node(format!("!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{base}, size: 64)"))
    }

    /// A structure named `name` of `size` bytes holding `members` (name, type, offset in bytes).
    fn structure(&mut self, at: usize, name: &str, size: u64, members: &[(String, usize, u64, u64)]) {
        let mut elements = Vec::with_capacity(members.len());
        for (member, ty, size, offset) in members {
            elements.push(self.node(format!(
                "!DIDerivedType(tag: DW_TAG_member, name: \"{}\", baseType: !{ty}, size: {}, offset: {})",
                escape(member.as_bytes()),
                size * 8,
                offset * 8
            )));
        }
        let elements: Vec<String> = elements.iter().map(|element| format!("!{element}")).collect();
        self.nodes[at] = format!(
            "distinct !DICompositeType(tag: DW_TAG_structure_type, name: \"{}\", size: {}, elements: !{{{}}})",
            escape(name.as_bytes()),
            size * 8,
            elements.join(", ")
        );
    }

    /// `[CG-LL-3]` — the description of `ty`, by its Ember name: a number by its width and
    /// signedness, a reference or pointer as a pointer, a list or view by its words, a struct,
    /// tuple or class object by its fields, an enum without payloads by its variants.
    pub(crate) fn describe(&mut self, types: &Types<'_>, ty: Ty) -> usize {
        if let Some(node) = self.types.get(&ty) {
            return *node;
        }
        let table = types.table;
        let name = table.display(ty).to_string();
        let (size, _) = types.size_align(ty);
        let bits = size * 8;
        let node = match table.kind(ty).clone() {
            TyKind::Bool => self.basic("bool", 8, "DW_ATE_boolean"),
            TyKind::Char => self.basic("char", 32, "DW_ATE_unsigned"),
            TyKind::Int(_) => self.basic(&name, bits, "DW_ATE_signed"),
            TyKind::Uint(_) => self.basic(&name, bits, "DW_ATE_unsigned"),
            TyKind::Float(_) if !types.is_f16(ty) => self.basic(&name, bits, "DW_ATE_float"),
            TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. } => {
                // Reserved first: a type may reach itself through a pointer.
                let at = self.node(String::new());
                self.types.insert(ty, at);
                let base = self.describe(types, inner);
                self.nodes[at] = format!("!DIDerivedType(tag: DW_TAG_pointer_type, baseType: !{base}, size: 64)");
                return at;
            }
            TyKind::Class(id) => {
                let object = self.node(String::new());
                let at = self.pointer(object);
                self.types.insert(ty, at);
                let offsets = table.class_field_offsets(id);
                let mut members = Vec::new();
                for index in 0..table.class_field_count(id) {
                    if let Some(field) = table.class_field_at(id, index) {
                        let (field_size, _) = types.size_align(field.ty);
                        let described = self.describe(types, field.ty);
                        members.push((field.name.to_string(), described, field_size, 24 + offsets[index]));
                    }
                }
                let object_size = members.iter().map(|(_, _, size, offset)| offset + size).max().unwrap_or(24);
                self.structure(object, &table.class_def(id).name.to_string(), object_size, &members);
                return at;
            }
            TyKind::Enum(id) if table.enum_def(id).is_unit_only() => {
                let def = table.enum_def(id).clone();
                let enumerators: Vec<usize> = def
                    .variants
                    .iter()
                    .map(|variant| self.node(format!("!DIEnumerator(name: \"{}\", value: {})", escape(variant.name.as_str().as_bytes()), variant.discriminant)))
                    .collect();
                let base = self.describe(types, def.repr);
                let elements: Vec<String> = enumerators.iter().map(|node| format!("!{node}")).collect();
                self.node(format!(
                    "!DICompositeType(tag: DW_TAG_enumeration_type, name: \"{}\", baseType: !{base}, size: {bits}, elements: !{{{}}})",
                    escape(name.as_bytes()),
                    elements.join(", ")
                ))
            }
            kind => {
                let at = self.node(String::new());
                self.types.insert(ty, at);
                let word = self.basic("u64", 64, "DW_ATE_unsigned");
                let byte = self.basic("u8", 8, "DW_ATE_unsigned");
                let members: Vec<(String, usize, u64, u64)> = match kind {
                    TyKind::Struct(id) => {
                        let offsets = table.layout(ty).field_offsets;
                        let fields = table.struct_def(id).fields.clone();
                        let mut members = Vec::new();
                        for (field, offset) in fields.iter().zip(offsets) {
                            let (field_size, _) = types.size_align(field.ty);
                            if field_size > 0 {
                                let described = self.describe(types, field.ty);
                                members.push((field.name.to_string(), described, field_size, offset));
                            }
                        }
                        members
                    }
                    TyKind::Tuple(items) => {
                        let offsets = table.layout(ty).field_offsets;
                        let mut members = Vec::new();
                        for (index, (item, offset)) in items.iter().zip(offsets).enumerate() {
                            let (item_size, _) = types.size_align(*item);
                            if item_size > 0 {
                                let described = self.describe(types, *item);
                                members.push((format!("_{index}"), described, item_size, offset));
                            }
                        }
                        members
                    }
                    TyKind::Vec { elem, .. } | TyKind::Span { elem, .. } => {
                        let element = self.describe(types, elem);
                        let pointer = self.pointer(element);
                        let mut members = vec![("ptr".to_string(), pointer, 8, 0), ("len".to_string(), word, 8, 8)];
                        if matches!(table.kind(ty), TyKind::Vec { .. }) {
                            members.push(("cap".to_string(), word, 8, 16));
                        }
                        members
                    }
                    TyKind::Str => {
                        let pointer = self.pointer(byte);
                        vec![("ptr".to_string(), pointer, 8, 0), ("len".to_string(), word, 8, 8)]
                    }
                    _ => Vec::new(),
                };
                self.structure(at, &name, size, &members);
                return at;
            }
        };
        self.types.insert(ty, node);
        node
    }

    /// The module's named metadata and nodes.
    pub(crate) fn write(&self, target: Target, out: &mut String) {
        let flags = self.nodes.len();
        let format = match target {
            Target::Win64 => "!{i32 2, !\"CodeView\", i32 1}",
            Target::SysV => "!{i32 7, !\"Dwarf Version\", i32 5}",
        };
        let _ = writeln!(out, "\n!llvm.dbg.cu = !{{!{}}}", self.unit);
        let _ = writeln!(out, "!llvm.module.flags = !{{!{flags}, !{}}}", flags + 1);
        for (index, node) in self.nodes.iter().enumerate() {
            let _ = writeln!(out, "!{index} = {node}");
        }
        let _ = writeln!(out, "!{flags} = {format}");
        let _ = writeln!(out, "!{} = !{{i32 2, !\"Debug Info Version\", i32 3}}", flags + 1);
    }
}

/// A path's directory and file name.
fn split(path: &str) -> (&str, &str) {
    match path.rfind('/') {
        Some(at) => (&path[..at], &path[at + 1..]),
        None => ("", path),
    }
}
