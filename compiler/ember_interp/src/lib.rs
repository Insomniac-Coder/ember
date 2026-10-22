//! `[CLI-21]`, `[IMP-11]` — the evaluator: a program's MIR run as it is, with no C compiler.
//!
//! The MIR it runs is the one every check has passed: drops elaborated, generators made state
//! machines, the dynamic access checks inserted; the passes after that only make code faster.
//! Every value lives in one flat heap with the target's layout (`memory`), so a value is its
//! bytes, as in the compiled program, and what the C backend and the runtime do for a construct
//! — counting class handles, dropping a value, formatting one, growing a list — is done here the
//! same way, so a run's output is the compiled run's. What compile time cannot run (`[CT-2]`:
//! a foreign call, a thread) stops the run with `Outcome::Refused`, which the caller reports as
//! `E6010`.

mod arena;
mod builtin;
mod call;
pub mod comptime;
mod leaks;
mod memory;
mod object;
mod scalar;
mod simd;
mod soa;
mod sync;
mod system;
mod text;
mod wide;

use std::cell::{Cell, OnceCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::io::{BufRead, Write};

use ember_mir::{
    AggregateKind, AssertKind, BasicBlockId, BinOp, Body, CastKind, Const, LocalId, Operand, Place, Projection, Rvalue,
    StmtKind, Terminator, UnOp,
};
use ember_span::{SourceMap, Span};
use ember_types::{CommonTypes, FloatTy, IntTy, Layout, Ty, TyKind, TypeTable, UintTy};

pub use memory::Fault;
use memory::{Memory, WordMap};
use scalar::IntKind;

/// What the evaluator runs: every body of the program and the types they name.
pub struct Program<'a> {
    pub bodies: &'a [Body],
    pub types: &'a TypeTable,
    pub common: &'a CommonTypes,
    pub map: &'a SourceMap,
    /// The `debug` profile's runtime (`EMBER_RT_DEBUG_ACCESS`): an access conflict names where
    /// the access it meets began.
    pub debug: bool,
    /// `[RT-4]` — outside `release`, a panic prints the frames back to `main`.
    pub backtrace: bool,
    /// `[WK-15]` — the leak report at the end (`ember run` in `debug`, or `--leak-check`).
    pub leak_check: bool,
}

/// Where the program's console goes, and its arguments (its name first, `[FN-8]`).
pub struct Io<'io> {
    pub stdout: &'io mut dyn Write,
    pub stderr: &'io mut dyn Write,
    pub stdin: &'io mut dyn BufRead,
    pub args: Vec<String>,
}

/// `[CT-3]` — how far an evaluation may go: steps (a function entered or a block's end) and
/// heap bytes live at once.
#[derive(Clone, Copy)]
pub struct Limits {
    pub steps: u64,
    pub heap: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The program ended with this status: `main` returned, or `process.exit` was called.
    Exited(i32),
    /// A panic, already reported on standard error as the runtime reports one (`[RT-4]`).
    Panicked,
    /// `[CT-2]` — an operation compile time cannot run, and where (`E6010`).
    Refused { what: String, span: Span },
    /// `[CT-3]` — a limit was passed.
    Limit(String),
    /// A defect of the program's `unsafe` code or of the evaluator.
    Fault(String),
}

/// Runs the program from `entry` (the generated `main_entry`, or `main`) to its end.
pub fn run(program: &Program<'_>, io: Io<'_>, limits: Limits, entry: &str) -> Outcome {
    let mut interp = Interp::new(program, io, limits);
    let outcome = match interp.run_entry(entry) {
        // `ember_rt_shutdown`: the leak report once the program has ended.
        Ok(status) if interp.leak_check => match interp.leak_report() {
            Ok(report) => {
                let _ = interp.io.stderr.write_all(report.as_bytes());
                Outcome::Exited(status)
            }
            Err(stop) => outcome_of(stop),
        },
        Ok(status) => Outcome::Exited(status),
        Err(Stop::Exit(status)) => Outcome::Exited(status),
        Err(Stop::Panic) => Outcome::Panicked,
        Err(Stop::Refused { what, span }) => Outcome::Refused { what, span },
        Err(Stop::Limit(what)) => Outcome::Limit(what),
        Err(Stop::Fault(what)) => Outcome::Fault(what),
    };
    let _ = interp.io.stdout.flush();
    let _ = interp.io.stderr.flush();
    outcome
}

fn outcome_of(stop: Stop) -> Outcome {
    match stop {
        Stop::Exit(status) => Outcome::Exited(status),
        Stop::Panic => Outcome::Panicked,
        Stop::Refused { what, span } => Outcome::Refused { what, span },
        Stop::Limit(what) => Outcome::Limit(what),
        Stop::Fault(what) => Outcome::Fault(what),
    }
}

/// Why a run stops before its end.
#[derive(Debug)]
pub(crate) enum Stop {
    Panic,
    Exit(i32),
    Refused { what: String, span: Span },
    Limit(String),
    Fault(String),
}

impl From<Fault> for Stop {
    fn from(fault: Fault) -> Stop {
        if fault.0.contains("heap limit") { Stop::Limit(fault.0) } else { Stop::Fault(fault.0) }
    }
}

pub(crate) type R<T> = Result<T, Stop>;

/// `ember_panic_bounds`'s message: an index past `i64::MAX` is a signed one wrapped, printed back
/// signed.
pub(crate) fn bounds_message(index: u64, len: u64) -> String {
    if index > i64::MAX as u64 {
        format!("index {} is out of bounds for a length of {len}", index as i64)
    } else {
        format!("index {index} is out of bounds for a length of {len}")
    }
}

pub(crate) fn fault<T>(what: impl Into<String>) -> R<T> {
    Err(Stop::Fault(what.into()))
}

/// Where a call's result goes when its frame returns.
enum Ret {
    /// Into the caller's place, which then goes on at the block.
    Place(Place, BasicBlockId),
    /// To the evaluator itself (`Interp::invoke`).
    Value,
}

const UNKNOWN_SIZE: u64 = u64::MAX;

struct FrameLayout {
    offsets: Vec<u64>,
    size: u64,
    align: u64,
}

struct Frame<'a> {
    body: &'a Body,
    /// Each local's address.
    locals: Vec<u64>,
    /// The frame's block of memory.
    base: u64,
    block: usize,
    stmt: usize,
    ret: Ret,
    /// What to do as it returns (`call::After`).
    after: Option<call::After>,
}

/// A place: its address and type, the variant a downcast selected, or a pointer that is a
/// value rather than stored anywhere (a `Shared`'s payload).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Loc {
    pub addr: u64,
    pub ty: Ty,
    pub variant: Option<usize>,
    pub value: Option<u64>,
}

impl Loc {
    fn at(addr: u64, ty: Ty) -> Loc {
        Loc { addr, ty, variant: None, value: None }
    }
}

pub(crate) struct Interp<'a, 'io> {
    pub bodies: &'a [Body],
    pub types: &'a TypeTable,
    pub common: &'a CommonTypes,
    pub map: &'a SourceMap,
    pub mem: Memory,
    pub io: Io<'io>,
    frames: Vec<Frame<'a>>,
    pub by_symbol: WordMap<&'a str, usize>,
    /// Each type's layout and size, by `Ty::index` (the table is fixed while a program runs).
    layouts: Vec<OnceCell<Rc<Layout>>>,
    sizes: Vec<Cell<u64>>,
    /// Each type's `needs_drop`: 0 not known yet, 1 no, 2 yes.
    drops: Vec<Cell<u8>>,
    /// Each body's frame: where its locals are, its size and alignment.
    frame_layouts: Vec<OnceCell<Rc<FrameLayout>>>,
    literals: HashMap<Vec<u8>, u64>,
    steps: u64,
    step_limit: u64,
    returned: Option<Vec<u8>>,
    /// The one byte a `void` place's borrow points at (`ember_void_place`).
    void_place: u64,
    pub objects: object::Objects,
    pub calls: call::Dispatch,
    pub system: system::System,
    pub debug: bool,
    backtrace: bool,
    pub leak_check: bool,
    /// `[CT-2]` — while compiling, the package directory `read_file` reads in (`comptime`).
    comptime: Option<String>,
}

impl<'a, 'io> Interp<'a, 'io> {
    fn new(program: &Program<'a>, io: Io<'io>, limits: Limits) -> Interp<'a, 'io> {
        let mut mem = Memory::new(limits.heap);
        let void_place = mem.alloc(1, 1).expect("the first block fits");
        let by_symbol = program.bodies.iter().enumerate().map(|(index, body)| (body.symbol.as_str(), index)).collect();
        let type_count = program.types.all().count();
        let mut interp = Interp {
            bodies: program.bodies,
            types: program.types,
            common: program.common,
            map: program.map,
            mem,
            io,
            frames: Vec::new(),
            by_symbol,
            layouts: (0..type_count).map(|_| OnceCell::new()).collect(),
            sizes: (0..type_count).map(|_| Cell::new(UNKNOWN_SIZE)).collect(),
            drops: (0..type_count).map(|_| Cell::new(0)).collect(),
            frame_layouts: (0..program.bodies.len()).map(|_| OnceCell::new()).collect(),
            literals: HashMap::new(),
            steps: 0,
            step_limit: limits.steps,
            returned: None,
            void_place,
            objects: object::Objects::default(),
            calls: call::Dispatch::default(),
            system: system::System::default(),
            debug: program.debug,
            backtrace: program.backtrace,
            leak_check: program.leak_check,
            comptime: None,
        };
        interp.calls = call::Dispatch::new(&interp);
        interp
    }

    fn run_entry(&mut self, entry: &str) -> R<i32> {
        let Some(&index) = self.by_symbol.get(entry) else {
            return fault(format!("the program has no `{entry}`"));
        };
        let value = self.invoke(index, Vec::new())?;
        let body = &self.bodies[index];
        let status = match self.types.kind(body.return_ty()) {
            TyKind::Int(_) | TyKind::Uint(_) => self.int_kind(body.return_ty()).map_or(0, |kind| kind.signed_value(&value) as i32),
            _ => 0,
        };
        self.io.stdout.flush().map_err(|e| Stop::Fault(e.to_string()))?;
        Ok(status)
    }

    // -- types --------------------------------------------------------------

    pub fn layout(&self, ty: Ty) -> Rc<Layout> {
        self.layouts[ty.index()].get_or_init(|| Rc::new(self.types.layout(ty))).clone()
    }

    pub fn size(&self, ty: Ty) -> u64 {
        let cached = &self.sizes[ty.index()];
        if cached.get() == UNKNOWN_SIZE {
            cached.set(if self.is_void(ty) { 0 } else { self.layout(ty).size });
        }
        cached.get()
    }

    pub fn needs_drop(&self, ty: Ty) -> bool {
        let cached = &self.drops[ty.index()];
        if cached.get() == 0 {
            cached.set(if self.types.needs_drop(ty) { 2 } else { 1 });
        }
        cached.get() == 2
    }

    fn frame_layout(&self, index: usize) -> Rc<FrameLayout> {
        self.frame_layouts[index]
            .get_or_init(|| {
                let mut offsets = Vec::with_capacity(self.bodies[index].locals.len());
                let (mut size, mut align) = (0u64, 1u64);
                for local in &self.bodies[index].locals {
                    let a = if self.is_void(local.ty) { 1 } else { self.layout(local.ty).align.max(1) };
                    size = size.div_ceil(a) * a;
                    offsets.push(size);
                    size += self.size(local.ty);
                    align = align.max(a);
                }
                Rc::new(FrameLayout { offsets, size, align })
            })
            .clone()
    }

    pub fn is_void(&self, ty: Ty) -> bool {
        matches!(self.types.kind(ty), TyKind::Void | TyKind::Never | TyKind::Error)
    }

    /// The integer type `ty` is computed as: an integer, a `char` (its scalar value), a `bool`, a
    /// range type's representation, or a unit-only enum's tag.
    pub fn int_kind(&self, ty: Ty) -> Option<IntKind> {
        Some(match self.types.kind(ty) {
            TyKind::Int(int) => IntKind {
                bits: match int {
                    IntTy::I8 => 8,
                    IntTy::I16 => 16,
                    IntTy::I32 => 32,
                    IntTy::I64 | IntTy::Isize => 64,
                    IntTy::I128 => 128,
                    IntTy::I256 => 256,
                },
                signed: true,
            },
            TyKind::Uint(uint) => IntKind {
                bits: match uint {
                    UintTy::U8 => 8,
                    UintTy::U16 => 16,
                    UintTy::U32 => 32,
                    UintTy::U64 | UintTy::Usize => 64,
                    UintTy::U128 => 128,
                    UintTy::U256 => 256,
                },
                signed: false,
            },
            TyKind::Char => IntKind { bits: 32, signed: false },
            TyKind::Bool => IntKind { bits: 8, signed: false },
            TyKind::Range(id) => return self.int_kind(self.types.range_def(*id).repr),
            TyKind::Enum(id) if self.types.enum_def(*id).is_unit_only() => return self.int_kind(self.types.enum_def(*id).repr),
            _ => return None,
        })
    }

    /// A float type's width, through a range type.
    pub fn float_bits(&self, ty: Ty) -> Option<u32> {
        match self.types.kind(ty) {
            TyKind::Float(FloatTy::F16) => Some(16),
            TyKind::Float(FloatTy::F32) => Some(32),
            TyKind::Float(FloatTy::F64) => Some(64),
            TyKind::Range(id) => self.float_bits(self.types.range_def(*id).repr),
            _ => None,
        }
    }

    pub fn float_value(&self, ty: Ty, bytes: &[u8]) -> f64 {
        match self.float_bits(ty) {
            Some(16) => scalar::f16_to_f64(memory::uint(&bytes[..2]) as u16),
            Some(32) => f32::from_bits(memory::uint(&bytes[..4]) as u32) as f64,
            _ => f64::from_bits(memory::uint(&bytes[..8]) as u64),
        }
    }

    pub fn float_bytes(&self, bits: u32, value: f64) -> Vec<u8> {
        match bits {
            16 => scalar::f64_to_f16(value).to_le_bytes().to_vec(),
            32 => (value as f32).to_bits().to_le_bytes().to_vec(),
            _ => value.to_bits().to_le_bytes().to_vec(),
        }
    }

    /// The type `place` has in `body`, as the C backend projects it.
    pub fn place_ty(&self, body: &Body, place: &Place) -> Ty {
        let mut ty = body.locals[place.local.0 as usize].ty;
        let mut variant = None;
        for projection in &place.projection {
            (ty, variant) = self.project_ty(ty, variant, projection);
        }
        ty
    }

    fn project_ty(&self, ty: Ty, variant: Option<usize>, projection: &Projection) -> (Ty, Option<usize>) {
        match (projection, self.types.kind(ty)) {
            (Projection::Downcast(v), TyKind::Enum(_)) => (ty, Some(*v)),
            (Projection::Field(index), TyKind::Enum(id)) => {
                let Some(variant) = variant else { return (ty, None) };
                let fields = &self.types.enum_def(*id).variants[variant].fields;
                (fields.get(*index).map_or(ty, |f| f.ty), None)
            }
            (Projection::Field(index), TyKind::Struct(id)) => {
                (self.types.struct_def(*id).fields.get(*index).map_or(ty, |f| f.ty), None)
            }
            (Projection::Field(index), TyKind::Class(id)) => {
                (self.types.class_field_at(*id, *index).map_or(ty, |f| f.ty), None)
            }
            (Projection::Field(index), TyKind::Tuple(items)) => (items.get(*index).copied().unwrap_or(ty), None),
            (Projection::Index(_) | Projection::ConstIndex(_), TyKind::Array { elem, .. } | TyKind::Vec { elem, .. } | TyKind::Span { elem, .. }) => {
                (*elem, None)
            }
            (Projection::Index(_) | Projection::ConstIndex(_), TyKind::Ptr { inner, .. }) => (*inner, None),
            (Projection::Field(_), TyKind::Str | TyKind::Vec { .. } | TyKind::Span { .. }) => (self.common.usize, None),
            (Projection::Deref, TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. }) => (*inner, None),
            _ => (ty, variant),
        }
    }

    pub fn operand_ty(&self, body: &Body, operand: &Operand) -> Option<Ty> {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => Some(self.place_ty(body, place)),
            Operand::Const(Const::Int { ty, .. } | Const::Float { ty, .. }) => Some(*ty),
            Operand::Const(Const::Bool(_)) => Some(self.common.bool_),
            Operand::Const(Const::Str(_)) => Some(self.common.str_),
            Operand::Const(_) => None,
        }
    }

    // -- frames ---------------------------------------------------------------

    fn frame(&self) -> &Frame<'a> {
        self.frames.last().expect("a frame is running")
    }

    pub(crate) fn body(&self) -> &'a Body {
        self.frame().body
    }

    /// Calls body `index` with `args` (each parameter's bytes), its result going where `ret`
    /// says. The frame runs from the next step.
    pub(crate) fn enter(&mut self, index: usize, args: Vec<Vec<u8>>, ret: Ret) -> R<()> {
        let body = &self.bodies[index];
        // `[CT-2]` — a foreign function, called directly or through the wrapper that maps its
        // parameters (`[FFI-9]`), is what compile time cannot run.
        if body.is_extern_declaration || body.ffi_counted.is_some() {
            return Err(Stop::Refused { what: format!("the foreign function `{}`", body.path.as_deref().unwrap_or(&body.name)), span: body.span });
        }
        if body.is_abstract {
            return fault(format!("a call of the abstract method `{}`", body.name));
        }
        self.tick()?;
        if self.frames.len() >= 100_000 {
            return self.panic(&format!("stack overflow in {}", body.path.as_deref().unwrap_or(&body.name)), body.span);
        }
        let frame = self.frame_layout(index);
        let base = self.mem.alloc(frame.size.max(1), frame.align)?;
        let locals: Vec<u64> = frame.offsets.iter().map(|offset| base + offset).collect();
        for (at, arg) in args.iter().enumerate() {
            let local = &body.locals[at + 1];
            let expected = self.size(local.ty);
            if arg.len() as u64 != expected {
                return fault(format!("`{}`'s parameter {} takes {expected} bytes and was given {}", body.name, at + 1, arg.len()));
            }
            self.mem.write(locals[at + 1], arg)?;
        }
        self.frames.push(Frame { body, locals, base, block: 0, stmt: 0, ret, after: None });
        Ok(())
    }

    /// Runs body `index` to its return, from inside the evaluator (a builtin calling Ember code,
    /// the entry point), and gives its result's bytes.
    pub(crate) fn invoke(&mut self, index: usize, args: Vec<Vec<u8>>) -> R<Vec<u8>> {
        let depth = self.frames.len();
        self.enter(index, args, Ret::Value)?;
        while self.frames.len() > depth {
            self.step()?;
        }
        Ok(self.returned.take().unwrap_or_default())
    }

    pub(crate) fn invoke_symbol(&mut self, symbol: &str, args: Vec<Vec<u8>>) -> R<Vec<u8>> {
        let Some(&index) = self.by_symbol.get(symbol) else {
            return fault(format!("no body `{symbol}`"));
        };
        self.invoke(index, args)
    }

    fn tick(&mut self) -> R<()> {
        self.steps += 1;
        if self.steps > self.step_limit {
            return Err(Stop::Limit(format!("the evaluation passed its limit of {} steps", self.step_limit)));
        }
        Ok(())
    }

    fn step(&mut self) -> R<()> {
        let frame = self.frames.last_mut().expect("a frame is running");
        let body = frame.body;
        let block = &body.blocks[frame.block];
        if frame.stmt < block.stmts.len() {
            let stmt = &block.stmts[frame.stmt];
            frame.stmt += 1;
            return self.statement(&stmt.kind, stmt.span);
        }
        self.terminator(&block.terminator, block.terminator_span)
    }

    fn goto(&mut self, target: BasicBlockId) -> R<()> {
        let frame = self.frames.last_mut().expect("a frame is running");
        frame.block = target.0 as usize;
        frame.stmt = 0;
        Ok(())
    }

    fn return_from_frame(&mut self) -> R<()> {
        let frame = self.frames.pop().expect("a frame is running");
        let ret_ty = frame.body.return_ty();
        let value = self.mem.read(frame.locals[0], self.size(ret_ty))?.to_vec();
        self.mem.free(frame.base)?;
        if let Some(after) = frame.after {
            self.run_after(after)?;
        }
        match frame.ret {
            Ret::Value => self.returned = Some(value),
            Ret::Place(place, next) => {
                let loc = self.place(&place)?;
                self.store(loc, &value)?;
                self.goto(next)?;
            }
        }
        Ok(())
    }

    // -- places and values ------------------------------------------------------

    pub(crate) fn place(&mut self, place: &Place) -> R<Loc> {
        let frame = self.frame();
        let body = frame.body;
        let mut at = Loc::at(frame.locals[place.local.0 as usize], body.locals[place.local.0 as usize].ty);
        for projection in &place.projection {
            at = self.project(at, projection)?;
        }
        Ok(at)
    }

    fn local_index(&self, local: LocalId) -> R<u64> {
        let addr = self.frame().locals[local.0 as usize];
        Ok(self.mem.read_u64(addr)?)
    }

    /// The pointer a place holds (a reference, a pointer, a handle, a buffer's `ptr`).
    pub(crate) fn pointer_at(&self, loc: Loc) -> R<u64> {
        match loc.value {
            Some(value) => Ok(value),
            None => Ok(self.mem.read_u64(loc.addr)?),
        }
    }

    fn project(&mut self, at: Loc, projection: &Projection) -> R<Loc> {
        let (ty, variant) = self.project_ty(at.ty, at.variant, projection);
        let plain = |addr: u64| Loc { addr, ty, variant, value: None };
        Ok(match (projection, self.types.kind(at.ty)) {
            (Projection::Downcast(_), TyKind::Enum(_)) => Loc { variant, ..at },
            (Projection::Field(index), TyKind::Enum(id)) => {
                let Some(v) = at.variant else { return Ok(at) };
                if self.types.option_niche(*id).is_some() {
                    plain(at.addr)
                } else {
                    let offsets = self.types.variant_field_offsets(*id);
                    plain(at.addr + offsets[v][*index])
                }
            }
            (Projection::Field(index), TyKind::Struct(id)) => {
                if let Some(inner) = object::shared_inner(self.types, *id) {
                    let header = self.pointer_at(at)?;
                    let payload = header + object::payload_offset(self.layout(inner).align);
                    Loc { addr: 0, ty, variant: None, value: Some(payload) }
                } else if self.types.compiler_box_inner(*id).is_some() {
                    plain(at.addr)
                } else {
                    let offset = self.layout(at.ty).field_offsets.get(*index).copied().unwrap_or(0);
                    plain(at.addr + offset)
                }
            }
            (Projection::Field(index), TyKind::Class(id)) => {
                let object = self.pointer_at(at)?;
                let offsets = self.types.class_field_offsets(*id);
                plain(object + object::HEADER + offsets[*index])
            }
            (Projection::Field(index), TyKind::Str | TyKind::Vec { .. } | TyKind::Span { .. }) => plain(at.addr + 8 * (*index as u64)),
            (Projection::Field(index), TyKind::Tuple(_)) => {
                let offset = self.layout(at.ty).field_offsets.get(*index).copied().unwrap_or(0);
                plain(at.addr + offset)
            }
            (Projection::Index(_) | Projection::ConstIndex(_), kind) => {
                let index = match projection {
                    Projection::Index(local) => self.local_index(*local)?,
                    Projection::ConstIndex(i) => *i,
                    _ => unreachable!(),
                };
                let step = self.size(ty);
                match kind {
                    TyKind::Array { .. } => plain(at.addr + index.wrapping_mul(step)),
                    TyKind::Vec { .. } | TyKind::Span { .. } | TyKind::Ptr { .. } => {
                        plain(self.pointer_at(at)?.wrapping_add(index.wrapping_mul(step)))
                    }
                    _ => return fault(format!("an index into `{}`", self.types.display(at.ty))),
                }
            }
            (Projection::Deref, TyKind::Ref { .. } | TyKind::Ptr { .. }) => plain(self.pointer_at(at)?),
            (Projection::Column(_), _) => return fault("an SoA column place"),
            _ => at,
        })
    }

    pub(crate) fn load(&self, loc: Loc) -> R<Vec<u8>> {
        if let Some(value) = loc.value {
            return Ok(value.to_le_bytes().to_vec());
        }
        Ok(self.mem.read(loc.addr, self.size(loc.ty))?.to_vec())
    }

    pub(crate) fn store(&mut self, loc: Loc, bytes: &[u8]) -> R<()> {
        if loc.value.is_some() {
            return fault("a write to a computed pointer");
        }
        if self.is_void(loc.ty) {
            return Ok(());
        }
        let size = self.size(loc.ty) as usize;
        if bytes.len() != size {
            return fault(format!("a {}-byte value written to a {size}-byte `{}`", bytes.len(), self.types.display(loc.ty)));
        }
        Ok(self.mem.write(loc.addr, bytes)?)
    }

    pub(crate) fn operand(&mut self, operand: &Operand, want: Option<Ty>) -> R<Vec<u8>> {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => {
                let loc = self.place(place)?;
                self.load(loc)
            }
            Operand::Const(constant) => self.constant(constant, want),
        }
    }

    pub(crate) fn literal(&mut self, bytes: &[u8]) -> R<u64> {
        if let Some(&addr) = self.literals.get(bytes) {
            return Ok(addr);
        }
        let addr = self.mem.alloc(bytes.len() as u64 + 1, 1)?;
        self.mem.write(addr, bytes)?;
        self.literals.insert(bytes.to_vec(), addr);
        Ok(addr)
    }

    /// A `str` value: its bytes' address and length.
    pub(crate) fn str_value(&mut self, text: &[u8]) -> R<Vec<u8>> {
        let addr = if text.is_empty() { 0 } else { self.literal(text)? };
        Ok(two_words(addr, text.len() as u64))
    }

    fn constant(&mut self, constant: &Const, want: Option<Ty>) -> R<Vec<u8>> {
        Ok(match constant {
            Const::Int { value, ty } => {
                let size = self.size(*ty) as usize;
                let mut bytes = value.to_le_bytes().to_vec();
                if size > 16 {
                    let fill = if self.int_kind(*ty).is_some_and(|k| k.signed) && (*value as i128) < 0 { 0xFF } else { 0 };
                    bytes.resize(size, fill);
                }
                bytes.truncate(size);
                bytes
            }
            Const::Float { value, ty } => self.float_bytes(self.float_bits(*ty).unwrap_or(64), *value),
            Const::Bool(value) => vec![u8::from(*value)],
            Const::Str(text) => self.str_value(text.as_bytes())?,
            Const::CStr(text) => self.literal(text.as_bytes())?.to_le_bytes().to_vec(),
            Const::CStrLiteral(bytes) => self.literal(bytes)?.to_le_bytes().to_vec(),
            Const::Fn(symbol) => {
                let Some(&index) = self.by_symbol.get(symbol.as_str()) else {
                    return fault(format!("no function `{symbol}`"));
                };
                let code = memory::address(memory::Region::Code, index);
                match want.map(|ty| self.types.kind(ty)) {
                    // A function's own descriptor, as the C backend gives each one: two values are
                    // equal when they call one function with the same state.
                    Some(TyKind::Fn { abi: None, .. }) => call::fn_value(code, code, [0; 3]),
                    _ => code.to_le_bytes().to_vec(),
                }
            }
            Const::Void => Vec::new(),
            Const::OverflowShift { .. } => 0u64.to_le_bytes().to_vec(),
        })
    }

    // -- statements -------------------------------------------------------------

    fn statement(&mut self, kind: &StmtKind, span: Span) -> R<()> {
        match kind {
            StmtKind::Assign { place, rvalue } => self.assign(place, rvalue, span),
            StmtKind::Drop { place, flag, .. } => {
                if let Some(flag) = flag {
                    let addr = self.frame().locals[flag.0 as usize];
                    if self.mem.read(addr, 1)?[0] == 0 {
                        return Ok(());
                    }
                }
                let loc = self.place(place)?;
                self.drop_at(loc.addr, loc.ty)
            }
            StmtKind::CheckedBinaryOp { dest, overflow, op, lhs, rhs } => {
                let body = self.body();
                let ty = self.place_ty(body, dest);
                if let Some(kind) = self.int_kind(ty).filter(|kind| kind.bits <= 64)
                    && let (Some(x), Some(y)) = (self.scalar_word(lhs, ty)?, self.scalar_word(rhs, ty)?)
                {
                    let (x, y) = (u128::from(x) & kind.mask(), u128::from(y) & kind.mask());
                    let (value, overflowed) = scalar::int_checked_narrow(*op, kind, x, y);
                    let at = self.place(dest)?;
                    self.mem.write_word(at.addr, kind.bytes() as u64, value as u64)?;
                    let flag = self.place(overflow)?;
                    return Ok(self.mem.write_word(flag.addr, 1, u64::from(overflowed))?);
                }
                let a = self.operand(lhs, Some(ty))?;
                let b = self.operand(rhs, Some(ty))?;
                let Some(kind) = self.int_kind(ty) else {
                    return fault(format!("checked arithmetic on `{}`", self.types.display(ty)));
                };
                let (value, overflowed) = scalar::int_checked(*op, kind, &a, &b);
                let dest = self.place(dest)?;
                self.store(dest, &value)?;
                let flag = self.place(overflow)?;
                self.store(flag, &[u8::from(overflowed)])
            }
            StmtKind::BeginAccess { place, mutable } => self.access(place, *mutable, true, span),
            StmtKind::EndAccess { place, mutable } => self.access(place, *mutable, false, span),
            StmtKind::BeginAccessTransfer { place, mutable } => {
                let loc = self.place(place)?;
                let object = self.pointer_at(loc)?;
                let what = self.types.display(loc.ty);
                self.header_access(object, *mutable, true, &what, span)
            }
            StmtKind::EndAccessTransfer { place, mutable } => {
                let loc = self.place(place)?;
                let align = self.layout(loc.ty).align;
                let object = loc.addr - object::payload_offset(align);
                let what = self.body().symbol.clone();
                self.header_access(object, *mutable, false, &what, span)
            }
            StmtKind::LendAccess { place, lend } => {
                if *lend {
                    self.access(place, true, false, span)?;
                    self.access(place, false, true, span)
                } else {
                    self.access(place, false, false, span)?;
                    self.access(place, true, true, span)
                }
            }
            StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => Ok(()),
        }
    }

    fn assign(&mut self, place: &Place, rvalue: &Rvalue, span: Span) -> R<()> {
        let body = self.body();
        let ty = self.place_ty(body, place);
        if self.is_void(ty) {
            // The operand is still evaluated where it could do something: a call never is one.
            return Ok(());
        }
        // `[OWN-7]` — a class handle copied is retained before the copy is visible.
        self.retain_for_rvalue(rvalue, ty)?;
        // The common statements, with no value built: a move or copy is the bytes copied, an
        // operator on scalars words read and written.
        match rvalue {
            Rvalue::Use(Operand::Copy(from) | Operand::Move(from)) => {
                let source = self.place(from)?;
                let dest = self.place(place)?;
                let size = self.size(ty);
                if source.value.is_none() && dest.value.is_none() && self.size(source.ty) == size {
                    return Ok(self.mem.copy(source.addr, dest.addr, size)?);
                }
            }
            Rvalue::BinaryOp { op, lhs, rhs } => {
                if self.scalar_binary(*op, lhs, rhs, place, ty)? {
                    return Ok(());
                }
            }
            _ => {}
        }
        let bytes = self.rvalue(rvalue, ty, span)?;
        let loc = self.place(place)?;
        self.store(loc, &bytes)
    }

    /// A scalar operand's bits, when it is one of at most `size` bytes: a place's or a constant's.
    fn scalar_word(&mut self, operand: &Operand, ty: Ty) -> R<Option<u64>> {
        let size = self.size(ty);
        if size > 8 {
            return Ok(None);
        }
        Ok(match operand {
            Operand::Copy(place) | Operand::Move(place) => {
                let loc = self.place(place)?;
                Some(match loc.value {
                    Some(value) => value,
                    None => self.mem.read_word(loc.addr, size)?,
                })
            }
            Operand::Const(Const::Int { value, .. }) => {
                let bits = 8 * size as u32;
                Some(if bits >= 64 { *value as u64 } else { (*value as u64) & ((1u64 << bits) - 1) })
            }
            Operand::Const(Const::Bool(value)) => Some(u64::from(*value)),
            Operand::Const(Const::Float { value, .. }) => match self.float_bits(ty) {
                Some(32) => Some(u64::from((*value as f32).to_bits())),
                Some(64) => Some(value.to_bits()),
                _ => None,
            },
            Operand::Const(_) => None,
        })
    }

    /// `binary` on integers of at most 64 bits and on `f32` and `f64`, written straight to the
    /// place; `false` when the operands are not those, for `binary` to do.
    fn scalar_binary(&mut self, op: BinOp, lhs: &Operand, rhs: &Operand, place: &Place, target: Ty) -> R<bool> {
        let body = self.body();
        let Some(lty) = self.operand_ty(body, lhs).or_else(|| self.operand_ty(body, rhs)) else { return Ok(false) };
        let rty = self.operand_ty(body, rhs).unwrap_or(lty);
        let out = self.size(target);
        if out > 8 {
            return Ok(false);
        }
        let comparison = matches!(op, BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge | BinOp::Is | BinOp::IsNot);
        let result = if let (Some(kind), Some(rkind)) = (self.int_kind(lty), self.int_kind(rty)) {
            if kind.bits > 64 || rkind.bits > 64 {
                return Ok(false);
            }
            let (Some(x), Some(y)) = (self.scalar_word(lhs, lty)?, self.scalar_word(rhs, rty)?) else { return Ok(false) };
            let (x, y) = (u128::from(x) & kind.mask(), u128::from(y) & rkind.mask());
            if matches!(op, BinOp::And | BinOp::Or) {
                u64::from(if op == BinOp::And { x != 0 && y != 0 } else { x != 0 || y != 0 })
            } else if comparison {
                u64::from(scalar::int_compare_bits(op, kind, x, y))
            } else if matches!(op, BinOp::Shl | BinOp::Shr) {
                // A shift's amount has its own type.
                let amount = if rkind.signed { rkind.extend(y).max(0) as u128 } else { y };
                scalar::int_binary_bits(op, kind, x, amount.min(255) & kind.mask()) as u64
            } else {
                scalar::int_binary_bits(op, kind, x, y) as u64
            }
        } else if let (Some(bits @ (32 | 64)), Some(rbits @ (32 | 64))) = (self.float_bits(lty), self.float_bits(rty)) {
            let (Some(x), Some(y)) = (self.scalar_word(lhs, lty)?, self.scalar_word(rhs, rty)?) else { return Ok(false) };
            let float = |bits: u32, word: u64| if bits == 32 { f64::from(f32::from_bits(word as u32)) } else { f64::from_bits(word) };
            let (x, y) = (float(bits, x), float(rbits, y));
            if comparison {
                u64::from(scalar::float_compare(op, x, y))
            } else {
                let Some(value) = scalar::float_binary(op, bits, x, y) else { return Ok(false) };
                if bits == 32 { u64::from((value as f32).to_bits()) } else { value.to_bits() }
            }
        } else {
            return Ok(false);
        };
        let dest = self.place(place)?;
        if dest.value.is_some() {
            return fault("a write to a computed pointer");
        }
        self.mem.write_word(dest.addr, out, result)?;
        Ok(true)
    }

    fn retain_for_rvalue(&mut self, rvalue: &Rvalue, target: Ty) -> R<()> {
        let body = self.body();
        match rvalue {
            Rvalue::Use(Operand::Copy(place)) => {
                if self.needs_drop(target) {
                    let loc = self.place(place)?;
                    self.retain_loc(loc, target)?;
                }
            }
            Rvalue::Aggregate { kind, operands } => {
                let types: Vec<Ty> = match kind {
                    AggregateKind::Callable => operands.iter().map(|o| self.operand_ty(body, o).unwrap_or(target)).collect(),
                    AggregateKind::Struct(id) => self.types.struct_def(*id).fields.iter().map(|f| f.ty).collect(),
                    AggregateKind::Tuple => match self.types.kind(target) {
                        TyKind::Tuple(items) => items.clone(),
                        _ => Vec::new(),
                    },
                    AggregateKind::Array => match self.types.kind(target) {
                        TyKind::Array { elem, len } => vec![*elem; *len as usize],
                        _ => Vec::new(),
                    },
                    AggregateKind::Enum(id, variant) => self.types.enum_def(*id).variants.get(*variant)
                        .map(|v| v.fields.iter().map(|f| f.ty).collect()).unwrap_or_default(),
                };
                for (operand, ty) in operands.iter().zip(types) {
                    if let Operand::Copy(place) = operand {
                        let loc = self.place(place)?;
                        self.retain_loc(loc, ty)?;
                    }
                }
            }
            Rvalue::Cast { kind: CastKind::ClassUpcast | CastKind::ClassInterfaceUpcast { .. }, operand: Operand::Copy(place), .. } => {
                let loc = self.place(place)?;
                let object = self.pointer_at(loc)?;
                self.retain_object(object)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Retains what a value of type `ty` at `loc` holds.
    pub(crate) fn retain_loc(&mut self, loc: Loc, ty: Ty) -> R<()> {
        match loc.value {
            Some(pointer) => {
                if matches!(self.types.kind(ty), TyKind::Class(_) | TyKind::ClassInterface(_)) {
                    self.retain_object(pointer)?;
                }
                Ok(())
            }
            None => self.retain_at(loc.addr, ty),
        }
    }

    fn rvalue(&mut self, rvalue: &Rvalue, target: Ty, span: Span) -> R<Vec<u8>> {
        let body = self.body();
        match rvalue {
            Rvalue::Use(operand) => self.operand(operand, Some(target)),
            Rvalue::BinaryOp { op, lhs, rhs } => {
                let lty = self.operand_ty(body, lhs).or_else(|| self.operand_ty(body, rhs)).unwrap_or(target);
                let rty = self.operand_ty(body, rhs).unwrap_or(lty);
                let a = self.operand(lhs, Some(lty))?;
                let b = self.operand(rhs, Some(rty))?;
                self.binary(*op, lty, rty, &a, &b, target)
            }
            Rvalue::UnaryOp { op, operand } => {
                let ty = self.operand_ty(body, operand).unwrap_or(target);
                let value = self.operand(operand, Some(ty))?;
                match op {
                    UnOp::Not => Ok(vec![u8::from(value[0] == 0)]),
                    UnOp::Neg if self.float_bits(ty).is_some() => {
                        let bits = self.float_bits(ty).expect("a float");
                        let v = self.float_value(ty, &value);
                        Ok(self.float_bytes(bits, -v))
                    }
                    UnOp::Neg | UnOp::BitNot => {
                        let Some(kind) = self.int_kind(ty) else {
                            return fault(format!("`-` or `~` on `{}`", self.types.display(ty)));
                        };
                        if kind.bits > 128 {
                            let x = wide::W256::of(kind, &value);
                            return Ok((if *op == UnOp::Neg { x.neg() } else { x.not() }).bytes(kind.bytes()));
                        }
                        let x = kind.unsigned(&value);
                        Ok(kind.encode(if *op == UnOp::Neg { x.wrapping_neg() } else { !x }))
                    }
                }
            }
            Rvalue::Cast { kind, operand, to } => self.cast(kind, operand, *to, span),
            Rvalue::Aggregate { kind, operands } => self.aggregate(kind, operands, target),
            Rvalue::Repeat { value, count } => {
                let elem = match self.types.kind(target) {
                    TyKind::Array { elem, .. } => *elem,
                    _ => return fault("a repeat that builds no array"),
                };
                let one = self.operand(value, Some(elem))?;
                let mut bytes = Vec::with_capacity(one.len() * *count as usize);
                for _ in 0..*count {
                    bytes.extend_from_slice(&one);
                }
                bytes.resize(self.size(target) as usize, 0);
                Ok(bytes)
            }
            Rvalue::Discriminant(place) => {
                let loc = self.place(place)?;
                let tag = self.discriminant(loc)?;
                let size = self.size(target) as usize;
                Ok(tag.to_le_bytes()[..size].to_vec())
            }
            Rvalue::Ref { place, .. } => {
                let loc = self.place(place)?;
                if self.is_void(loc.ty) {
                    return Ok(self.void_place.to_le_bytes().to_vec());
                }
                if loc.value.is_some() {
                    return fault("a borrow of a computed pointer");
                }
                Ok(loc.addr.to_le_bytes().to_vec())
            }
        }
    }

    /// The discriminant of the enum at `loc`.
    pub(crate) fn discriminant(&self, loc: Loc) -> R<i128> {
        let TyKind::Enum(id) = *self.types.kind(loc.ty) else {
            // Reading a non-enum's discriminant reads the value (a unit enum through a range).
            let bytes = self.load(loc)?;
            return Ok(self.int_kind(loc.ty).map_or(0, |k| k.signed_value(&bytes)));
        };
        let def = self.types.enum_def(id);
        if let Some(niche) = self.types.option_niche(id) {
            let none = self.niche_holds(niche, loc)?;
            return Ok(def.variants[if none { niche.none } else { niche.some }].discriminant);
        }
        let repr = self.int_kind(def.repr).expect("an integer repr");
        let bytes = self.mem.read(loc.addr, repr.bytes() as u64)?;
        Ok(repr.signed_value(bytes))
    }

    fn binary(&mut self, op: BinOp, lty: Ty, rty: Ty, a: &[u8], b: &[u8], target: Ty) -> R<Vec<u8>> {
        if matches!(op, BinOp::And | BinOp::Or) {
            let (x, y) = (a.first().copied().unwrap_or(0) != 0, b.first().copied().unwrap_or(0) != 0);
            return Ok(vec![u8::from(if op == BinOp::And { x && y } else { x || y })]);
        }
        let comparison = matches!(op, BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge | BinOp::Is | BinOp::IsNot);
        // `[TYP-36]` — `void` has one value, equal to itself.
        if comparison && a.is_empty() && b.is_empty() {
            return Ok(vec![u8::from(scalar::compare(op, std::cmp::Ordering::Equal))]);
        }
        if let Some(bits) = self.float_bits(lty) {
            let (x, y) = (self.float_value(lty, a), self.float_value(rty, b));
            if comparison {
                return Ok(vec![u8::from(scalar::float_compare(op, x, y))]);
            }
            let Some(value) = scalar::float_binary(op, bits, x, y) else {
                return fault(format!("`{}` on floats", op.c_operator()));
            };
            return Ok(self.float_bytes(bits, value));
        }
        if let Some(kind) = self.int_kind(lty) {
            if comparison {
                return Ok(vec![u8::from(scalar::int_compare(op, kind, a, b))]);
            }
            // A shift's amount has its own type.
            if matches!(op, BinOp::Shl | BinOp::Shr) {
                let amount = self.int_kind(rty).map_or(0, |k| if k.signed { k.signed_value(b).max(0) as u128 } else { k.unsigned(b) });
                let amount = kind.encode(amount.min(255));
                return Ok(scalar::int_binary(op, kind, a, &amount));
            }
            return Ok(scalar::int_binary(op, kind, a, b));
        }
        if comparison {
            // A callable value is equal to another calling the same entry with the same state.
            if let TyKind::Fn { abi: None, .. } = self.types.kind(lty) {
                let same = a[8..] == b[8..];
                return Ok(vec![u8::from(if matches!(op, BinOp::Ne | BinOp::IsNot) { !same } else { same })]);
            }
            // Pointers, handles and other words, by their bits.
            let n = a.len().min(b.len());
            let ordering = memory::uint(&a[..n.min(16)]).cmp(&memory::uint(&b[..n.min(16)]));
            let _ = target;
            return Ok(vec![u8::from(scalar::compare(op, ordering))]);
        }
        fault(format!("`{}` on `{}`", op.c_operator(), self.types.display(lty)))
    }

    fn cast(&mut self, kind: &CastKind, operand: &Operand, to: Ty, span: Span) -> R<Vec<u8>> {
        let body = self.body();
        let from = self.operand_ty(body, operand).unwrap_or(to);
        let value = self.operand(operand, Some(from))?;
        match kind {
            CastKind::Numeric | CastKind::Widen => {
                let to_size = self.size(to) as usize;
                if let Some(bits) = self.float_bits(to) {
                    let v = match (self.float_bits(from), self.int_kind(from)) {
                        (Some(_), _) => self.float_value(from, &value),
                        (None, Some(k)) if k.bits <= 128 => {
                            if bits == 32 { scalar::int_to_f32(k, &value) as f64 } else { scalar::int_to_f64(k, &value) }
                        }
                        _ => return fault(format!("`{}` as `{}`", self.types.display(from), self.types.display(to))),
                    };
                    return Ok(self.float_bytes(bits, v));
                }
                if let Some(target) = self.int_kind(to) {
                    if let Some(_) = self.float_bits(from) {
                        let v = self.float_value(from, &value);
                        return Ok(scalar::float_to_int(target, v));
                    }
                    if let Some(source) = self.int_kind(from) {
                        if source.bits > 128 || target.bits > 128 {
                            let mut bytes = value.clone();
                            let fill = if source.signed && value.last().is_some_and(|b| b & 0x80 != 0) { 0xFF } else { 0 };
                            bytes.resize(to_size, fill);
                            return Ok(bytes);
                        }
                        return Ok(scalar::int_cast(source, target, &value));
                    }
                    // A pointer or handle as an integer.
                    let mut bytes = value.clone();
                    bytes.resize(to_size, 0);
                    return Ok(bytes);
                }
                // An integer as a pointer, or a pointer as another.
                let mut bytes = value;
                if let Some(source) = self.int_kind(from)
                    && source.signed
                {
                    let v = source.signed_value(&bytes) as u128;
                    bytes = v.to_le_bytes().to_vec();
                }
                bytes.resize(to_size, 0);
                Ok(bytes)
            }
            CastKind::ClassUpcast | CastKind::ClassUpcastBorrowed | CastKind::ClassInterfaceUpcast { .. } | CastKind::ViewsDisjoint => Ok(value),
            CastKind::InterfaceUpcast { concrete, interfaces, implementations, .. } => {
                let pointer = u64_of(&value);
                let data = if matches!(self.types.kind(*concrete), TyKind::Class(_)) { self.mem.read_u64(pointer)? } else { pointer };
                let table = self.calls.table(*concrete, interfaces, implementations);
                Ok(two_words(data, table))
            }
            CastKind::FnToC => {
                let descriptor = u64_of(&value[8..16]);
                if !matches!(memory::decode(descriptor), Some((memory::Region::Code, _))) {
                    return self.panic("a callable value holding captured state has no C entry", span);
                }
                Ok(value[..8].to_vec())
            }
            CastKind::InterfaceDowncast => {
                let boxed = u64_of(&value);
                Ok(self.mem.read(boxed, 8)?.to_vec())
            }
        }
    }

    fn aggregate(&mut self, kind: &AggregateKind, operands: &[Operand], target: Ty) -> R<Vec<u8>> {
        let size = self.size(target) as usize;
        let mut bytes = vec![0u8; size];
        let put = |bytes: &mut Vec<u8>, offset: u64, value: &[u8]| {
            let at = offset as usize;
            bytes[at..at + value.len()].copy_from_slice(value);
        };
        match kind {
            AggregateKind::Struct(id) => {
                let fields: Vec<Ty> = self.types.struct_def(*id).fields.iter().map(|f| f.ty).collect();
                let offsets = self.layout(target).field_offsets.clone();
                for (index, operand) in operands.iter().enumerate() {
                    let value = self.operand(operand, fields.get(index).copied())?;
                    put(&mut bytes, offsets.get(index).copied().unwrap_or(0), &value);
                }
            }
            AggregateKind::Tuple => {
                let items = match self.types.kind(target) {
                    TyKind::Tuple(items) => items.clone(),
                    _ => Vec::new(),
                };
                let offsets = self.layout(target).field_offsets.clone();
                for (index, operand) in operands.iter().enumerate() {
                    let value = self.operand(operand, items.get(index).copied())?;
                    put(&mut bytes, offsets.get(index).copied().unwrap_or(0), &value);
                }
            }
            AggregateKind::Array => {
                let elem = match self.types.kind(target) {
                    TyKind::Array { elem, .. } => *elem,
                    _ => return fault("an array aggregate of another type"),
                };
                let step = self.size(elem);
                for (index, operand) in operands.iter().enumerate() {
                    let value = self.operand(operand, Some(elem))?;
                    put(&mut bytes, index as u64 * step, &value);
                }
            }
            AggregateKind::Enum(id, variant) => {
                let def = self.types.enum_def(*id);
                let fields: Vec<Ty> = def.variants[*variant].fields.iter().map(|f| f.ty).collect();
                if let Some(niche) = self.types.option_niche(*id) {
                    if *variant == niche.some {
                        return self.operand(&operands[0], Some(niche.payload));
                    }
                    return self.niche_none(niche);
                }
                let tag = def.variants[*variant].discriminant;
                let repr = self.int_kind(def.repr).expect("an integer repr");
                put(&mut bytes, 0, &repr.encode(tag as u128));
                if !operands.is_empty() {
                    let offsets = self.types.variant_field_offsets(*id);
                    for (index, operand) in operands.iter().enumerate() {
                        let value = self.operand(operand, fields.get(index).copied())?;
                        put(&mut bytes, offsets[*variant][index], &value);
                    }
                }
            }
            AggregateKind::Callable => {
                let [Operand::Const(Const::Fn(closure)), env] = operands else {
                    return fault("an owned callable value of another shape");
                };
                let body = self.body();
                let env_ty = self.operand_ty(body, env).unwrap_or(self.common.void);
                let env_bytes = self.operand(env, Some(env_ty))?;
                return self.make_callable(closure, env_ty, &env_bytes);
            }
        }
        Ok(bytes)
    }

    // -- terminators ----------------------------------------------------------------

    fn terminator(&mut self, terminator: &Terminator, span: Span) -> R<()> {
        match terminator {
            Terminator::Goto(target) => {
                self.tick()?;
                self.goto(*target)
            }
            Terminator::SwitchInt { discr, targets, otherwise } => {
                self.tick()?;
                let body = self.body();
                let ty = self.operand_ty(body, discr);
                let kind = ty.and_then(|ty| self.int_kind(ty));
                let word = match (ty, kind) {
                    (Some(ty), Some(kind)) if kind.bits <= 64 => self.scalar_word(discr, ty)?,
                    _ => None,
                };
                let value = match (word, kind) {
                    (Some(word), Some(kind)) => kind.extend(u128::from(word) & kind.mask()),
                    _ => {
                        let bytes = self.operand(discr, ty)?;
                        match kind {
                            Some(kind) if kind.bits <= 128 => kind.signed_value(&bytes),
                            _ => memory::uint(&bytes) as i128,
                        }
                    }
                };
                let target = targets.iter().find(|(v, _)| *v == value).map_or(*otherwise, |(_, t)| *t);
                self.goto(target)
            }
            Terminator::Return => self.return_from_frame(),
            Terminator::Unreachable => fault(format!("`{}` reached an unreachable block", self.body().name)),
            Terminator::Assert { cond, expected, msg, next, span } => {
                let value = match self.scalar_word(cond, self.common.bool_)? {
                    Some(word) => word & 0xFF,
                    None => u64::from(self.operand(cond, Some(self.common.bool_))?.first().copied().unwrap_or(0)),
                };
                if (value != 0) == *expected {
                    return self.goto(*next);
                }
                let message = self.assert_message(msg)?;
                self.panic(&message, *span)
            }
            Terminator::Call { func, args, dest, next } => self.call(func, args, dest, *next, span),
        }
    }

    fn assert_message(&mut self, msg: &AssertKind) -> R<String> {
        Ok(match msg {
            AssertKind::Overflow(op) => format!("integer overflow in `{}`", op.spelling()),
            AssertKind::SignedDivisionOverflow => "integer overflow in `/`".to_string(),
            AssertKind::ShiftTooLarge => "integer overflow in `shift`".to_string(),
            AssertKind::DivisionByZero => "division by zero".to_string(),
            AssertKind::Panic { message } => {
                let value = self.operand(message, Some(self.common.str_))?;
                String::from_utf8_lossy(&self.str_bytes(&value)?).into_owned()
            }
            AssertKind::Bounds { len, index } => {
                let len = u64_of(&self.operand(len, Some(self.common.usize))?);
                let index = u64_of(&self.operand(index, Some(self.common.usize))?);
                bounds_message(index, len)
            }
            AssertKind::RefCellBorrow { file, line } => {
                let file = u64_of(&self.operand(file, None)?);
                let line = self.operand(line, None)?;
                let line = memory::uint(&line) as u32;
                let path = if file == 0 { String::new() } else { self.c_string(file)? };
                return Ok(text::refcell_message(&path, line));
            }
            AssertKind::Downcast => "invalid downcast".to_string(),
        })
    }

    /// `[RT-4]` — "panic at <file>:<line>:<col>: <message>", then the backtrace, and the run ends.
    pub(crate) fn panic<T>(&mut self, message: &str, span: Span) -> R<T> {
        let _ = self.io.stdout.flush();
        let location = self.location(span);
        let _ = match &location {
            Some((path, line, col)) => writeln!(self.io.stderr, "panic at {path}:{line}:{col}: {message}"),
            None => writeln!(self.io.stderr, "panic: {message}"),
        };
        self.print_backtrace(location);
        if self.comptime.is_some() {
            self.comptime_chain();
        }
        let _ = self.io.stderr.flush();
        Err(Stop::Panic)
    }

    /// `ember_backtrace_print`: outside `release`, each of the program's frames by its Ember path,
    /// the innermost at the panic and each other at its call of the one below, back to `main`.
    fn print_backtrace(&mut self, at: Option<(String, u32, u32)>) {
        if !self.backtrace {
            return;
        }
        let mut text = String::new();
        let mut printed = 0;
        for frame in self.frames.iter().rev() {
            let body = frame.body;
            if body.is_abstract || body.is_extern_declaration || body.blocks.is_empty() || body.span.is_dummy() || self.map.get(body.span.file).is_none() {
                continue;
            }
            let name = if body.is_lambda { "a lambda".to_string() } else { body.path.clone().unwrap_or_else(|| body.name.clone()) };
            if printed == 0 {
                text.push_str("backtrace:\n");
            }
            let place = if printed == 0 { at.clone() } else { self.location(body.blocks[frame.block].terminator_span) };
            match place {
                Some((path, line, col)) => text.push_str(&format!("  {printed}: {name} at {path}:{line}:{col}\n")),
                None => {
                    let line = self.map.file(body.span.file).line_col(body.span.start).line;
                    text.push_str(&format!("  {printed}: {name} at {}:{line}\n", self.map.artefact_path(body.span.file)));
                }
            }
            printed += 1;
            if name == "main" {
                break;
            }
        }
        let _ = self.io.stderr.write_all(text.as_bytes());
    }

    /// A panic the runtime raises at a place of its own (`{ "<arena>", 0, 0 }`).
    pub(crate) fn panic_in<T>(&mut self, file: &str, message: &str) -> R<T> {
        let _ = self.io.stdout.flush();
        let _ = writeln!(self.io.stderr, "panic at {file}:0:0: {message}");
        self.print_backtrace(Some((file.to_string(), 0, 0)));
        if self.comptime.is_some() {
            self.comptime_chain();
        }
        let _ = self.io.stderr.flush();
        Err(Stop::Panic)
    }

    pub(crate) fn location(&self, span: Span) -> Option<(String, u32, u32)> {
        if span.is_dummy() || self.map.get(span.file).is_none() {
            return None;
        }
        let lc = self.map.file(span.file).line_col(span.start);
        Some((self.map.artefact_path(span.file), lc.line, lc.col))
    }

    // -- text helpers ---------------------------------------------------------------

    /// The bytes a `str` value (pointer, length) points at.
    pub(crate) fn str_bytes(&self, value: &[u8]) -> R<Vec<u8>> {
        let (ptr, len) = (u64_of(&value[..8]), u64_of(&value[8..16]));
        if len == 0 {
            return Ok(Vec::new());
        }
        Ok(self.mem.read(ptr, len)?.to_vec())
    }

    /// A NUL-terminated C string at `addr`.
    pub(crate) fn c_string(&self, addr: u64) -> R<String> {
        let mut bytes = Vec::new();
        let mut at = addr;
        loop {
            let byte = self.mem.read(at, 1)?[0];
            if byte == 0 {
                break;
            }
            bytes.push(byte);
            at += 1;
        }
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
}

pub(crate) fn u64_of(bytes: &[u8]) -> u64 {
    memory::uint(&bytes[..bytes.len().min(8)]) as u64
}

pub(crate) fn two_words(a: u64, b: u64) -> Vec<u8> {
    let mut bytes = a.to_le_bytes().to_vec();
    bytes.extend_from_slice(&b.to_le_bytes());
    bytes
}
