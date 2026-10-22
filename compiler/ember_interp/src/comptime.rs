//! `[CT-1]`..`[CT-7]` — a compile-time evaluation on the evaluator, for a build that runs no C
//! compiler (`ember run --interp`, `[CLI-21]`). It answers as the C program
//! `ember_codegen_c::emit_comptime` builds does: the value as `const_eval::read_value` reads it on
//! standard output, and on standard error the lines the driver reads — the inputs read
//! (`ember-comptime-input`), the call chain (`ember-comptime-frame`) after a panic or a passed
//! limit (`ember-comptime-limit`), a read outside the package (`ember-comptime-outside`) — with
//! the same exit status: 86 a panic, 87 the step limit, 88 the heap limit, 89 a read outside.

use std::path::{Component, Path, PathBuf};

use ember_mir::{Operand, Place};
use ember_types::{FloatTy, Ty, TyKind};

use crate::wide::W256;
use crate::{fault, Interp, Io, Limits, Program, Stop, R};

/// What one evaluation ended with: its exit status, standard output and standard error.
pub struct Evaluation {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

/// A path made absolute and its `.` and `..` resolved by its text, with `/` between its parts
/// (`_fullpath`, `realpath`).
fn canonical(path: &Path) -> String {
    let absolute = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    let mut parts = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::ParentDir => {
                parts.pop();
            }
            Component::CurDir => {}
            other => parts.push(other.as_os_str()),
        }
    }
    let text = parts.to_string_lossy().replace('\\', "/");
    text.trim_end_matches('/').to_string()
}

/// Runs the request's function `symbol` and gives its value.
pub fn evaluate(program: &Program<'_>, symbol: &str, limits: Limits, root: &Path) -> Evaluation {
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let mut stdin = std::io::empty();
    let status = {
        let io = Io { stdout: &mut stdout, stderr: &mut stderr, stdin: &mut std::io::BufReader::new(&mut stdin), args: Vec::new() };
        let mut interp = Interp::new(program, io, limits);
        interp.backtrace = false;
        interp.comptime = Some(canonical(root));
        let outcome = match interp.by_symbol.get(symbol).copied() {
            Some(index) => interp.invoke(index, Vec::new()).and_then(|value| {
                let ty = interp.bodies[index].return_ty();
                let mut text = String::new();
                if interp.is_void(ty) {
                    text.push_str(" _");
                } else {
                    let at = interp.temp(&value)?;
                    interp.dump(&mut text, at, ty)?;
                }
                Ok(text)
            }),
            None => fault(format!("no function `{symbol}`")),
        };
        let status = match outcome {
            Ok(text) => {
                let _ = writeln!(interp.io.stdout, "{text}");
                0
            }
            Err(Stop::Panic) => 86,
            Err(Stop::Exit(status)) => status,
            Err(Stop::Limit(what)) => {
                let heap = what.contains("heap");
                let _ = writeln!(interp.io.stderr, "ember-comptime-limit {}", if heap { "heap" } else { "steps" });
                interp.comptime_chain();
                if heap { 88 } else { 87 }
            }
            Err(Stop::Refused { what, .. }) => {
                let _ = writeln!(interp.io.stderr, "the evaluator stopped: {what} cannot run while compiling");
                90
            }
            Err(Stop::Fault(what)) => {
                let _ = writeln!(interp.io.stderr, "the evaluator stopped: {what}");
                90
            }
        };
        let _ = interp.io.stdout.flush();
        status
    };
    Evaluation { status, stdout: String::from_utf8_lossy(&stdout).into_owned(), stderr: String::from_utf8_lossy(&stderr).into_owned() }
}

impl Interp<'_, '_> {
    /// `ember_ct_chain`: the frames, the request's function first.
    pub(crate) fn comptime_chain(&mut self) {
        let names: Vec<String> = self.frames.iter().take(256).map(|frame| frame.body.name.clone()).collect();
        for name in names {
            let _ = writeln!(self.io.stderr, "ember-comptime-frame {name}");
        }
    }

    /// `comptime_dump`: the value of `ty` at `addr` as `read_value` reads it.
    fn dump(&mut self, out: &mut String, addr: u64, ty: Ty) -> R<()> {
        match self.types.kind(ty).clone() {
            TyKind::Bool => out.push_str(&format!(" b{}", self.mem.read(addr, 1)?[0])),
            TyKind::Char => out.push_str(&format!(" c{}", self.mem.read_u32(addr)?)),
            TyKind::Int(_) | TyKind::Uint(_) => {
                let kind = self.int_kind(ty).expect("an integer");
                let bytes = self.mem.read(addr, kind.bytes() as u64)?.to_vec();
                let text = if kind.bits > 128 {
                    let w = W256::of(kind, &bytes);
                    let negative = kind.signed && w.negative();
                    format!("{}{}", if negative { "-" } else { "" }, if negative { w.neg() } else { w }.digits(10))
                } else if kind.signed {
                    kind.signed_value(&bytes).to_string()
                } else {
                    kind.unsigned(&bytes).to_string()
                };
                out.push_str(&format!(" i{text}"));
            }
            TyKind::Float(FloatTy::F64) => out.push_str(&format!(" f{:x}", self.mem.read_u64(addr)?)),
            TyKind::Float(FloatTy::F32) => out.push_str(&format!(" f{:x}", self.mem.read_u32(addr)?)),
            TyKind::Float(FloatTy::F16) => out.push_str(&format!(" f{:x}", self.mem.read_word(addr, 2)?)),
            TyKind::Void | TyKind::Never => out.push_str(" _"),
            TyKind::Str | TyKind::Vec { text: true, .. } => {
                let (ptr, len) = (self.mem.read_u64(addr)?, self.mem.read_u64(addr + 8)?);
                out.push_str(&format!(" s{len}:"));
                if len > 0 {
                    for byte in self.mem.read(ptr, len)? {
                        out.push_str(&format!("{byte:02x}"));
                    }
                }
            }
            TyKind::Range(id) => return self.dump(out, addr, self.types.range_def(id).repr),
            TyKind::Vec { elem, text: false } => {
                let (ptr, len) = (self.mem.read_u64(addr)?, self.mem.read_u64(addr + 8)?);
                out.push_str(&format!(" v{len} ("));
                let step = self.size(elem);
                for i in 0..len {
                    self.dump(out, ptr + i * step, elem)?;
                }
                out.push_str(" )");
            }
            TyKind::Tuple(items) => {
                out.push_str(" (");
                let offsets = self.layout(ty).field_offsets.clone();
                for (item, offset) in items.into_iter().zip(offsets) {
                    self.dump(out, addr + offset, item)?;
                }
                out.push_str(" )");
            }
            TyKind::Array { elem, len } => {
                out.push_str(" (");
                let step = self.size(elem);
                for i in 0..len {
                    self.dump(out, addr + i * step, elem)?;
                }
                out.push_str(" )");
            }
            TyKind::Struct(id) => {
                out.push_str(" (");
                let fields: Vec<Ty> = self.types.struct_def(id).fields.iter().map(|f| f.ty).collect();
                let offsets = self.layout(ty).field_offsets.clone();
                for (field, offset) in fields.into_iter().zip(offsets) {
                    self.dump(out, addr + offset, field)?;
                }
                out.push_str(" )");
            }
            TyKind::Enum(id) => {
                let def = self.types.enum_def(id);
                let tag = self.discriminant(crate::Loc::at(addr, ty))?;
                let Some(index) = def.variants.iter().position(|v| v.discriminant == tag) else {
                    out.push_str(" ?");
                    return Ok(());
                };
                out.push_str(&format!(" e{index} ("));
                let fields: Vec<Ty> = def.variants[index].fields.iter().map(|f| f.ty).collect();
                let niche = self.types.option_niche(id).is_some();
                let offsets = if niche || fields.is_empty() { Vec::new() } else { self.types.variant_field_offsets(id).swap_remove(index) };
                for (k, field) in fields.into_iter().enumerate() {
                    let at = if niche { addr } else { addr + offsets[k] };
                    self.dump(out, at, field)?;
                }
                out.push_str(" )");
            }
            _ => out.push_str(" ?"),
        }
        Ok(())
    }

    /// `ember_ct_read_file`: a file inside the package directory, as UTF-8 text.
    pub(crate) fn comptime_read_file(&mut self, path: &str) -> R<Vec<u8>> {
        let Some(root) = self.comptime.clone() else {
            return Err(Stop::Refused { what: "`comptime.read_file` at run time".to_string(), span: ember_span::Span::DUMMY });
        };
        let full = canonical(&Path::new(&root).join(path));
        let inside = full.strip_prefix(root.as_str()).is_some_and(|rest| rest.is_empty() || rest.starts_with('/'));
        if !inside {
            let _ = self.io.stdout.flush();
            let _ = writeln!(self.io.stderr, "ember-comptime-outside {path}");
            self.comptime_chain();
            return Err(Stop::Exit(89));
        }
        let _ = writeln!(self.io.stderr, "ember-comptime-input file {full}");
        let Ok(bytes) = std::fs::read(&full) else {
            return self.panic_in("<comptime>", &format!("comptime.read_file: cannot read `{path}`"));
        };
        if std::str::from_utf8(&bytes).is_err() {
            return self.panic_in("<comptime>", &format!("comptime.read_file: `{path}` is not UTF-8 text"));
        }
        self.vec_from_bytes(&bytes)
    }

    /// `ember_ct_env_<Option>`: a build-environment variable, written to `dest` as an
    /// `Option[String]`.
    pub(crate) fn comptime_env(&mut self, option: Ty, args: &[Operand], dest: &Place) -> R<()> {
        let name = String::from_utf8_lossy(&self.text_operand(&args[0])?).into_owned();
        let _ = writeln!(self.io.stderr, "ember-comptime-input env {name}");
        let value = match std::env::var_os(&name) {
            Some(value) => Some(self.vec_from_bytes(value.to_string_lossy().as_bytes())?),
            None => None,
        };
        let TyKind::Enum(id) = *self.types.kind(option) else { return fault("`comptime.env` gives no Option") };
        let at = self.place(dest)?.addr;
        let def = self.types.enum_def(id);
        let none = def.variants.iter().position(|v| v.fields.is_empty()).expect("an Option has a None");
        let some = def.variants.iter().position(|v| v.fields.len() == 1).expect("an Option has a Some");
        self.mem.write(at, &vec![0; self.size(option) as usize])?;
        if let Some(niche) = self.types.option_niche(id) {
            let bytes = match value {
                Some(bytes) => bytes,
                None => self.niche_none(niche)?,
            };
            return Ok(self.mem.write(at, &bytes)?);
        }
        let repr = self.int_kind(def.repr).expect("an integer tag");
        let variant = if value.is_some() { some } else { none };
        self.mem.write(at, &repr.encode(def.variants[variant].discriminant as u128))?;
        if let Some(bytes) = value {
            let offset = self.types.variant_field_offsets(id)[some][0];
            self.mem.write(at + offset, &bytes)?;
        }
        Ok(())
    }
}
