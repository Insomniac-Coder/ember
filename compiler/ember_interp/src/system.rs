//! The runtime's console, files, environment, clock and process (`[STD-22]`, `[STD-23]`,
//! `[FN-8]`), as `ember_rt.c` gives them: each operation answers 0 or the C library's `errno` for
//! what failed, which `std.io.Error` asks about. And the statics (`[STA-3]`, `[CT-5]`).

use std::collections::HashMap;
use std::io::{Read, Write};

use ember_hir::RuntimeFn;
use ember_mir::Operand;
use ember_span::Span;
use ember_types::{Ty, TyKind};

use crate::{fault, u64_of, Interp, Stop, R};

const ENOENT: i64 = 2;
const EIO: i64 = 5;
const EACCES: i64 = 13;
const EEXIST: i64 = 17;
const EINVAL: i64 = 22;

#[derive(Default)]
pub struct System {
    files: HashMap<i64, std::fs::File>,
    next_file: i64,
    statics: HashMap<String, u64>,
    /// `[ARN-1]` — each arena's chunks, by its state's address.
    pub arenas: crate::memory::WordMap<u64, crate::arena::ArenaState>,
    /// `[STA-3]` — the statics being initialised, outermost first.
    initialising: Vec<String>,
    key: Option<[u64; 2]>,
    started: Option<std::time::Instant>,
    /// `[STD-18]` — the open `format_to` buffers: each one's address and whether an append was
    /// refused.
    pub sinks: Vec<(u64, bool)>,
}

impl System {
    pub fn process_key(&mut self, which: i64) -> u64 {
        let key = *self.key.get_or_insert_with(|| [entropy(), entropy()]);
        key[(which & 1) as usize]
    }
}

/// A fresh random word, from the operating system's hash keys and the clock.
fn entropy() -> u64 {
    use std::hash::{BuildHasher, Hasher};
    let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
    hasher.write_u128(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos()));
    hasher.finish()
}

/// The C library's `errno` for an I/O error, as the runtime's calls would have set it.
fn errno(error: &std::io::Error) -> i64 {
    match error.kind() {
        std::io::ErrorKind::NotFound => ENOENT,
        std::io::ErrorKind::PermissionDenied => EACCES,
        std::io::ErrorKind::AlreadyExists => EEXIST,
        std::io::ErrorKind::InvalidInput => EINVAL,
        _ => error.raw_os_error().filter(|_| cfg!(unix)).map_or(EIO, i64::from),
    }
}

/// `strerror(code)` as the host's C library writes it.
fn strerror(code: i64) -> &'static str {
    match code {
        ENOENT => "No such file or directory",
        EACCES => "Permission denied",
        EEXIST => "File exists",
        EINVAL => "Invalid argument",
        20 => "Not a directory",
        21 => "Is a directory",
        _ if cfg!(windows) => "I/O error",
        _ => "Input/output error",
    }
}

impl Interp<'_, '_> {
    fn path(&mut self, operand: &Operand) -> R<std::path::PathBuf> {
        let value = self.operand(operand, Some(self.common.str_))?;
        let bytes = self.str_bytes(&value)?;
        Ok(std::path::PathBuf::from(String::from_utf8_lossy(&bytes).into_owned()))
    }

    fn write_out_word(&mut self, operand: &Operand, value: i64) -> R<()> {
        let at = u64_of(&self.operand(operand, None)?);
        Ok(self.mem.write_u64(at, value as u64)?)
    }

    fn status(&self, dest: Ty, code: i64) -> Vec<u8> {
        let size = self.size(dest) as usize;
        code.to_le_bytes()[..size.min(8)].to_vec()
    }

    pub(crate) fn runtime_fn(&mut self, function: RuntimeFn, args: &[Operand], dest: Ty, span: Span) -> R<Vec<u8>> {
        let _ = span;
        Ok(match function {
            RuntimeFn::Entropy => entropy().to_le_bytes().to_vec(),
            RuntimeFn::ProcessExit => {
                let code = i64::from_le_bytes(self.operand(&args[0], None)?[..8].try_into().expect("a word"));
                let _ = self.io.stdout.flush();
                let _ = self.io.stderr.flush();
                return Err(Stop::Exit(code as i32));
            }
            RuntimeFn::ProcessAbort => {
                let _ = self.io.stdout.flush();
                let _ = self.io.stderr.flush();
                return Err(Stop::Exit(if cfg!(windows) { 3 } else { 134 }));
            }
            RuntimeFn::ProcessArgs | RuntimeFn::ProcessArgsOs => self.process_args()?,
            RuntimeFn::EnvVar => {
                let name = String::from_utf8_lossy(&{
                    let value = self.operand(&args[0], Some(self.common.str_))?;
                    self.str_bytes(&value)?
                })
                .into_owned();
                let out = u64_of(&self.operand(&args[1], None)?);
                match std::env::var_os(&name) {
                    Some(value) => {
                        let text = value.to_string_lossy().into_owned();
                        self.vec_extend(out, text.as_bytes())?;
                        vec![1]
                    }
                    None => vec![0],
                }
            }
            RuntimeFn::FsRead => {
                let path = self.path(&args[0])?;
                let out = u64_of(&self.operand(&args[1], None)?);
                match std::fs::read(&path) {
                    Ok(bytes) => {
                        self.vec_extend(out, &bytes)?;
                        self.status(dest, 0)
                    }
                    Err(error) => self.status(dest, errno(&error)),
                }
            }
            RuntimeFn::FsWrite => {
                let path = self.path(&args[0])?;
                let data = self.operand(&args[1], None)?;
                let bytes = self.str_bytes(&data)?;
                let append = self.operand(&args[2], Some(self.common.bool_))?[0] != 0;
                let result = std::fs::OpenOptions::new().write(true).create(true).append(append).truncate(!append).open(&path).and_then(|mut f| f.write_all(&bytes));
                self.status(dest, result.map_or_else(|e| errno(&e), |_| 0))
            }
            RuntimeFn::FsExists => vec![u8::from(self.path(&args[0])?.exists())],
            RuntimeFn::FsMetadata => {
                let path = self.path(&args[0])?;
                match std::fs::metadata(&path) {
                    Ok(meta) => {
                        self.write_out_word(&args[1], meta.len() as i64)?;
                        let kind = if meta.is_dir() { 1 } else if meta.is_file() { 0 } else { 2 };
                        self.write_out_word(&args[2], kind)?;
                        self.status(dest, 0)
                    }
                    Err(error) => {
                        self.write_out_word(&args[2], -1)?;
                        self.status(dest, errno(&error))
                    }
                }
            }
            RuntimeFn::FsRemove => {
                let path = self.path(&args[0])?;
                let result = match std::fs::metadata(&path) {
                    Ok(meta) if meta.is_dir() => std::fs::remove_dir(&path),
                    Ok(_) => std::fs::remove_file(&path),
                    Err(error) => Err(error),
                };
                self.status(dest, result.map_or_else(|e| errno(&e), |_| 0))
            }
            RuntimeFn::FsCreateDirAll => {
                let path = self.path(&args[0])?;
                let code = if path.is_file() { EEXIST } else { std::fs::create_dir_all(&path).map_or_else(|e| errno(&e), |_| 0) };
                self.status(dest, code)
            }
            RuntimeFn::FsReadDir => {
                let path = self.path(&args[0])?;
                let names = u64_of(&self.operand(&args[1], None)?);
                match std::fs::read_dir(&path) {
                    Ok(entries) => {
                        let mut list: Vec<String> = entries.filter_map(Result::ok).map(|e| e.file_name().to_string_lossy().into_owned()).collect();
                        if cfg!(unix) {
                            // `readdir`'s order is the file system's; the evaluator's is the names'.
                            list.sort();
                        }
                        for name in list {
                            let value = self.vec_from_bytes(name.as_bytes())?;
                            self.vec_push(names, 24, &value, 16)?;
                        }
                        self.status(dest, 0)
                    }
                    Err(error) => self.status(dest, errno(&error)),
                }
            }
            RuntimeFn::FileOpen => {
                let path = self.path(&args[0])?;
                let mode = i64::from_le_bytes(self.operand(&args[1], None)?[..8].try_into().expect("a word"));
                let mut options = std::fs::OpenOptions::new();
                match mode {
                    0 => options.read(true),
                    1 => options.write(true).create(true).truncate(true),
                    _ => options.append(true).create(true),
                };
                match options.open(&path) {
                    Ok(file) => {
                        self.system.next_file += 1;
                        let handle = self.system.next_file;
                        self.system.files.insert(handle, file);
                        self.write_out_word(&args[2], handle)?;
                        self.status(dest, 0)
                    }
                    Err(error) => {
                        self.write_out_word(&args[2], 0)?;
                        self.status(dest, errno(&error))
                    }
                }
            }
            RuntimeFn::FileRead => {
                let handle = i64::from_le_bytes(self.operand(&args[0], None)?[..8].try_into().expect("a word"));
                let into = self.operand(&args[1], None)?;
                let (ptr, len) = (u64_of(&into[..8]), u64_of(&into[8..16]));
                let mut buffer = vec![0u8; len as usize];
                let Some(file) = self.system.files.get_mut(&handle) else { return fault("a read of no file") };
                let mut got = 0;
                let mut code = 0;
                while got < buffer.len() {
                    match file.read(&mut buffer[got..]) {
                        Ok(0) => break,
                        Ok(n) => got += n,
                        Err(error) => {
                            code = errno(&error);
                            break;
                        }
                    }
                }
                self.mem.write(ptr, &buffer[..got])?;
                self.write_out_word(&args[2], got as i64)?;
                self.status(dest, code)
            }
            RuntimeFn::FileWrite => {
                let handle = i64::from_le_bytes(self.operand(&args[0], None)?[..8].try_into().expect("a word"));
                let data = self.operand(&args[1], None)?;
                let bytes = self.str_bytes(&data)?;
                let Some(file) = self.system.files.get_mut(&handle) else { return fault("a write to no file") };
                let code = file.write_all(&bytes).map_or_else(|e| errno(&e), |_| 0);
                self.status(dest, code)
            }
            RuntimeFn::FileFlush => {
                let handle = i64::from_le_bytes(self.operand(&args[0], None)?[..8].try_into().expect("a word"));
                let Some(file) = self.system.files.get_mut(&handle) else { return fault("a flush of no file") };
                let code = file.flush().map_or_else(|e| errno(&e), |_| 0);
                self.status(dest, code)
            }
            RuntimeFn::FileClose => {
                let handle = i64::from_le_bytes(self.operand(&args[0], None)?[..8].try_into().expect("a word"));
                self.system.files.remove(&handle);
                self.status(dest, 0)
            }
            RuntimeFn::StdinReadLine => {
                let _ = self.io.stdout.flush();
                let out = u64_of(&self.operand(&args[0], None)?);
                let (line, read_any) = self.read_line()?;
                self.vec_extend(out, &line)?;
                self.write_out_byte(&args[1], u8::from(!read_any))?;
                self.status(dest, 0)
            }
            RuntimeFn::IoErrorKind => {
                let code = i64::from_le_bytes(self.operand(&args[0], None)?[..8].try_into().expect("a word"));
                let kind = match code {
                    ENOENT => 0,
                    EACCES | 1 => 1,
                    EEXIST => 2,
                    _ => 4,
                };
                self.status(dest, kind)
            }
            RuntimeFn::IoErrorMessage => {
                let code = i64::from_le_bytes(self.operand(&args[0], None)?[..8].try_into().expect("a word"));
                let out = u64_of(&self.operand(&args[1], None)?);
                self.vec_extend(out, strerror(code).as_bytes())?;
                return Ok(Vec::new());
            }
            RuntimeFn::TimeNow => {
                let started = *self.system.started.get_or_insert_with(std::time::Instant::now);
                let nanos = started.elapsed().as_nanos() as i64 + 1_000_000_000;
                self.status(dest, nanos)
            }
            RuntimeFn::TimeSleep => {
                let nanos = i64::from_le_bytes(self.operand(&args[0], None)?[..8].try_into().expect("a word"));
                if nanos > 0 {
                    std::thread::sleep(std::time::Duration::from_nanos(nanos as u64));
                }
                return Ok(Vec::new());
            }
            // ODR-183 — a character past ASCII in class `which` (0 alphabetic, 1 a digit, 2 numeric):
            // the runtime's runs, generated for both by tools/unicode_case.py.
            RuntimeFn::CharClass => {
                let c = u32::from_le_bytes(self.operand(&args[0], None)?[..4].try_into().expect("a char"));
                let which = i64::from_le_bytes(self.operand(&args[1], None)?[..8].try_into().expect("a word"));
                let runs = match which {
                    0 => crate::char_classes::ALPHABETIC,
                    1 => crate::char_classes::DIGIT,
                    _ => crate::char_classes::NUMERIC,
                };
                let held = runs
                    .binary_search_by(|&(first, last)| {
                        if c < first {
                            std::cmp::Ordering::Greater
                        } else if c > last {
                            std::cmp::Ordering::Less
                        } else {
                            std::cmp::Ordering::Equal
                        }
                    })
                    .is_ok();
                vec![u8::from(held)]
            }
            // `[ALC-2]` (ODR-175, ODR-239) — `Global` and `System` both allocate from the evaluator's
            // heap, which is the only memory a program run here has: null when it is full, as the
            // runtime's `try_alloc` gives.
            RuntimeFn::GlobalAllocate | RuntimeFn::SystemAllocate => {
                let size = u64_of(&self.operand(&args[0], None)?);
                let align = u64_of(&self.operand(&args[1], None)?);
                self.mem.alloc(size, align).unwrap_or(0).to_le_bytes().to_vec()
            }
            RuntimeFn::GlobalDeallocate | RuntimeFn::SystemDeallocate => {
                let p = u64_of(&self.operand(&args[0], None)?);
                if p != 0 {
                    self.mem.free(p)?;
                }
                return Ok(Vec::new());
            }
            RuntimeFn::GlobalReallocate | RuntimeFn::SystemReallocate => {
                let p = u64_of(&self.operand(&args[0], None)?);
                let old_size = u64_of(&self.operand(&args[1], None)?);
                let new_size = u64_of(&self.operand(&args[2], None)?);
                let align = u64_of(&self.operand(&args[3], None)?);
                let Ok(fresh) = self.mem.alloc(new_size, align) else { return Ok(0u64.to_le_bytes().to_vec()) };
                if p != 0 {
                    self.mem.copy(p, fresh, old_size.min(new_size))?;
                    self.mem.free(p)?;
                }
                fresh.to_le_bytes().to_vec()
            }
        })
    }

    fn write_out_byte(&mut self, operand: &Operand, value: u8) -> R<()> {
        let at = u64_of(&self.operand(operand, None)?);
        Ok(self.mem.write(at, &[value])?)
    }

    /// One line of standard input without its `\n` (or `\r\n`), and whether anything was read.
    fn read_line(&mut self) -> R<(Vec<u8>, bool)> {
        let mut line = Vec::new();
        let read = self.io.stdin.read_until(b'\n', &mut line).map_err(|e| Stop::Fault(e.to_string()))?;
        if line.last() == Some(&b'\n') {
            line.pop();
        }
        if line.last() == Some(&b'\r') {
            line.pop();
        }
        Ok((line, read > 0))
    }

    /// `[STD-10]` — `input(prompt)`: the prompt, then one line; end of input panics.
    pub(crate) fn input(&mut self, prompt: &[u8], span: Span) -> R<Vec<u8>> {
        self.io.stdout.write_all(prompt).map_err(|e| Stop::Fault(e.to_string()))?;
        let _ = self.io.stdout.flush();
        let (line, read_any) = self.read_line()?;
        if !read_any {
            return self.panic("input: end of input; `std.io.stdin().read_line()` returns it as a `Result` instead", span);
        }
        if std::str::from_utf8(&line).is_err() {
            return self.panic(
                "input: the line is not valid UTF-8; `std.io.stdin().read_line()` returns it as a `Result` instead",
                span,
            );
        }
        self.vec_from_bytes(&line)
    }

    /// `[FN-8]` — the program's arguments, its name first, as `Array[String]`.
    pub(crate) fn process_args(&mut self) -> R<Vec<u8>> {
        let args = self.io.args.clone();
        let list = self.temp(&[0; 24])?;
        for arg in args {
            let value = self.vec_from_bytes(arg.as_bytes())?;
            self.vec_push(list, 24, &value, 16)?;
        }
        let value = self.mem.read(list, 24)?.to_vec();
        self.mem.free(list)?;
        Ok(value)
    }

    // -- statics ---------------------------------------------------------------------

    /// `[STA-3]` — a static initialised at its first access: its storage's address.
    pub(crate) fn lazy_static(&mut self, name: &str, init: &str, ty: Ty) -> R<u64> {
        if let Some(&at) = self.system.statics.get(name) {
            return Ok(at);
        }
        // `ember_rt_static_begin`: one asked for again while it is being made is a cycle.
        let shown = name.rsplit('.').next().unwrap_or(name).to_string();
        if self.system.initialising.contains(&shown) {
            let mut chain: Vec<String> = self.system.initialising.iter().take(32).cloned().collect();
            chain.push(shown);
            return self.panic(&format!("static initialisation cycle: {}", chain.join(" -> ")), Span::DUMMY);
        }
        self.system.initialising.push(shown);
        let value = self.invoke_symbol(init, Vec::new());
        self.system.initialising.pop();
        let value = value?;
        let layout = self.layout(ty);
        let at = self.mem.alloc(layout.size.max(1), layout.align)?;
        self.mem.write(at, &value)?;
        self.system.statics.insert(name.to_string(), at);
        Ok(at)
    }

    /// `[CT-5]` — a static the compile-time evaluator gave a value: its storage's address.
    pub(crate) fn const_static(&mut self, name: &str, ty: Ty) -> R<u64> {
        if let Some(&at) = self.system.statics.get(name) {
            return Ok(at);
        }
        let Some((_, _, text)) = self.types.constant_statics().iter().find(|(n, _, _)| n.as_str() == name) else {
            return fault(format!("no value for the constant static `{name}`"));
        };
        let text = text.clone();
        let mut tokens = text.split_ascii_whitespace();
        let value = self.constant_value(ty, &mut tokens)?;
        let layout = self.layout(ty);
        let at = self.mem.alloc(layout.size.max(1), layout.align)?;
        self.mem.write(at, &value)?;
        self.system.statics.insert(name.to_string(), at);
        Ok(at)
    }

    /// A value of `ty` read from the evaluator's text (`constant_initializer`'s format).
    fn constant_value<'t>(&mut self, ty: Ty, tokens: &mut impl Iterator<Item = &'t str>) -> R<Vec<u8>> {
        let bad = || Stop::Fault("a constant static's text that does not read".to_string());
        let bytes_of = |text: &str| -> Option<Vec<u8>> {
            let (len, hex) = text.split_once(':')?;
            let len: usize = len.parse().ok()?;
            (0..len).map(|i| u8::from_str_radix(hex.get(i * 2..i * 2 + 2)?, 16).ok()).collect()
        };
        let size = self.size(ty) as usize;
        Ok(match self.types.kind(ty).clone() {
            TyKind::Bool => vec![u8::from(tokens.next().ok_or_else(bad)? == "b1")],
            TyKind::Char => {
                let c: u32 = tokens.next().and_then(|t| t.strip_prefix('c')).and_then(|t| t.parse().ok()).ok_or_else(bad)?;
                c.to_le_bytes().to_vec()
            }
            TyKind::Int(_) | TyKind::Uint(_) => {
                let value: i128 = tokens.next().and_then(|t| t.strip_prefix('i')).and_then(|t| t.parse::<i128>().ok().or_else(|| t.parse::<u128>().ok().map(|v| v as i128))).ok_or_else(bad)?;
                value.to_le_bytes()[..size.min(16)].to_vec()
            }
            TyKind::Float(_) => {
                let bits = tokens.next().and_then(|t| t.strip_prefix('f')).and_then(|t| u64::from_str_radix(t, 16).ok()).ok_or_else(bad)?;
                bits.to_le_bytes()[..size].to_vec()
            }
            TyKind::Range(id) => self.constant_value(self.types.range_def(id).repr, tokens)?,
            TyKind::Str => {
                let bytes = tokens.next().and_then(|t| t.strip_prefix('s')).and_then(bytes_of).ok_or_else(bad)?;
                self.str_value(&bytes)?
            }
            TyKind::Vec { text: true, .. } => {
                let bytes = tokens.next().and_then(|t| t.strip_prefix('s')).and_then(bytes_of).ok_or_else(bad)?;
                self.vec_from_bytes(&bytes)?
            }
            TyKind::Vec { elem, text: false } => {
                let len: u64 = tokens.next().and_then(|t| t.strip_prefix('v')).and_then(|t| t.parse().ok()).ok_or_else(bad)?;
                if tokens.next() != Some("(") {
                    return Err(bad());
                }
                let mut items = Vec::new();
                for _ in 0..len {
                    items.extend(self.constant_value(elem, tokens)?);
                }
                if tokens.next() != Some(")") {
                    return Err(bad());
                }
                let at = self.temp(&items)?;
                let (step, align) = (self.size(elem), self.layout(elem).align);
                let value = self.vec_from_elems(step, at, len, align)?;
                self.mem.free(at)?;
                value
            }
            TyKind::Tuple(items) => {
                let offsets = self.layout(ty).field_offsets.clone();
                self.constant_fields(&items, &offsets, size, tokens)?
            }
            TyKind::Array { elem, len } => {
                let step = self.size(elem);
                let offsets: Vec<u64> = (0..len).map(|i| i * step).collect();
                self.constant_fields(&vec![elem; len as usize], &offsets, size, tokens)?
            }
            TyKind::Struct(id) => {
                let fields: Vec<Ty> = self.types.struct_def(id).fields.iter().map(|f| f.ty).collect();
                let offsets = self.layout(ty).field_offsets.clone();
                self.constant_fields(&fields, &offsets, size, tokens)?
            }
            TyKind::Enum(id) => {
                let variant: usize = tokens.next().and_then(|t| t.strip_prefix('e')).and_then(|t| t.parse().ok()).ok_or_else(bad)?;
                let def = self.types.enum_def(id);
                let fields: Vec<Ty> = def.variants.get(variant).ok_or_else(bad)?.fields.iter().map(|f| f.ty).collect();
                if tokens.next() != Some("(") {
                    return Err(bad());
                }
                let mut values = Vec::new();
                for field in &fields {
                    if self.is_void(*field) {
                        tokens.next();
                        values.push(Vec::new());
                    } else {
                        values.push(self.constant_value(*field, tokens)?);
                    }
                }
                if tokens.next() != Some(")") {
                    return Err(bad());
                }
                if let Some(niche) = self.types.option_niche(id) {
                    if variant == niche.some {
                        return Ok(values.swap_remove(0));
                    }
                    return self.niche_none(niche);
                }
                let mut bytes = vec![0u8; size];
                let repr = self.int_kind(def.repr).expect("a repr");
                let tag = repr.encode(def.variants[variant].discriminant as u128);
                bytes[..tag.len()].copy_from_slice(&tag);
                if !values.is_empty() {
                    let offsets = self.types.variant_field_offsets(id).swap_remove(variant);
                    for (value, offset) in values.iter().zip(offsets) {
                        bytes[offset as usize..offset as usize + value.len()].copy_from_slice(value);
                    }
                }
                bytes
            }
            _ => return Err(bad()),
        })
    }

    fn constant_fields<'t>(&mut self, fields: &[Ty], offsets: &[u64], size: usize, tokens: &mut impl Iterator<Item = &'t str>) -> R<Vec<u8>> {
        if tokens.next() != Some("(") {
            return fault("a constant static's text that does not read");
        }
        let mut bytes = vec![0u8; size];
        for (field, offset) in fields.iter().zip(offsets) {
            let value = self.constant_value(*field, tokens)?;
            bytes[*offset as usize..*offset as usize + value.len()].copy_from_slice(&value);
        }
        if tokens.next() != Some(")") {
            return fault("a constant static's text that does not read");
        }
        Ok(bytes)
    }

    // -- guards and system values ---------------------------------------------------------

    /// `[THR-3]` — a `MutexGuard`'s end releases its lock: the guard points at the mutex's value,
    /// its first field, and the lock word is the mutex's `word`.
    pub(crate) fn system_drop(&mut self, addr: u64, ty: Ty, what: &str) -> R<()> {
        let _ = (addr, ty);
        Err(Stop::Refused { what: format!("a `{what}`"), span: Span::DUMMY })
    }
}
