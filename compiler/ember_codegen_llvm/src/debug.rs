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

use crate::abi::Target;
use crate::escape;

pub(crate) struct Debug {
    /// The metadata nodes, by number.
    nodes: Vec<String>,
    unit: usize,
    subroutine: usize,
    files: HashMap<FileId, usize>,
}

impl Debug {
    pub(crate) fn new(map: &SourceMap, target: Target, module_name: &str, optimized: bool) -> Debug {
        let mut debug = Debug { nodes: Vec::new(), unit: 0, subroutine: 0, files: HashMap::new() };
        let main = map.files().next().map(|file| file.path.display().to_string().replace('\\', "/")).unwrap_or_else(|| module_name.to_string());
        let (directory, name) = split(&main);
        let file = debug.node(format!("!DIFile(filename: \"{}\", directory: \"{}\")", escape(name.as_bytes()), escape(directory.as_bytes())));
        debug.unit = debug.node(format!(
            "distinct !DICompileUnit(language: DW_LANG_C99, file: !{file}, producer: \"Ember\", isOptimized: {optimized}, runtimeVersion: 0, emissionKind: LineTablesOnly)"
        ));
        let types = debug.node("!{null}".to_string());
        debug.subroutine = debug.node(format!("!DISubroutineType(types: !{types})"));
        let _ = target;
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
