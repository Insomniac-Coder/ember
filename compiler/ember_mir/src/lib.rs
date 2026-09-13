//! MIR: a control-flow graph over places and operands (Part XVIII §4).
//!
//! MIR is where the borrow checker, drop elaboration and the effect analysis
//! will run (Phases 2–4), so it exists from Phase 0 even though Phase 0 only
//! lowers straight-line code. Codegen reads MIR, never HIR — writing the C
//! backend against HIR would mean writing it twice.

use ember_span::Span;
use ember_types::{EnumId, StructId, Ty};

pub mod lower;
pub mod verify;

pub use lower::lower;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct BasicBlockId(pub u32);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct LocalId(pub u32);

/// The slot a function's return value is written into. Rust MIR's convention,
/// kept because it makes `Return` need no operand.
pub const RETURN_LOCAL: LocalId = LocalId(0);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum LocalKind {
    /// The return slot, local 0.
    Return,
    /// A parameter. Parameters occupy locals `1..=arg_count`.
    Arg,
    /// A user-written local.
    User,
    /// A compiler-introduced temporary.
    Temp,
}

#[derive(Clone, Debug)]
pub struct LocalDecl {
    pub ty: Ty,
    pub kind: LocalKind,
    /// The name as written, for readable C and for diagnostics.
    pub name: Option<String>,
    pub span: Span,
}

#[derive(Debug)]
pub struct Body {
    pub name: String,
    /// The mangled C symbol (`[MNG-1]`).
    pub symbol: String,
    pub locals: Vec<LocalDecl>,
    pub blocks: Vec<BasicBlock>,
    pub arg_count: usize,
    pub span: Span,
    /// `[LT-1a]` — the parameter positions `@borrows(…)` names. `None` means
    /// `[LT-1]`'s elision decides which parameters the return may point into.
    pub borrows: Option<Vec<usize>>,
    /// `[FN-1]` — the parameters this body borrows rather than owns, as MIR
    /// locals. A borrowed parameter arrives as a bitwise copy of the caller's
    /// value with no loan behind it, so no borrow analysis can see that the
    /// caller still owns (and drops) it: a `Move` out of one of these places
    /// double-destroys across the call boundary, which `[EXP-6]` reports as
    /// `E3013` (D-041). Populated from the HIR parameter modes at lowering.
    /// Only `Borrow` is listed: `Owned` takes ownership, and `Mut` arrives as
    /// `ref mut`, whose moves already carry a `Deref` projection.
    pub borrowed_params: Vec<LocalId>,
    /// `[CTL-2]` — iterator locals synthesized specifically for source `for`
    /// loops. HIR has already desugared those loops to ordinary control flow;
    /// retaining this semantic fact lets diagnostics distinguish E3020/B2
    /// from a manually held iterator's ordinary E3021/B3 overlap.
    pub for_iterators: Vec<LocalId>,
    /// `[MIR-REG-1]` — compiler-internal callable access and result-provenance
    /// metadata. Lowering leaves this absent; the region analysis derives it
    /// from MIR before borrow checking. The unconditional code-generation
    /// verifier rejects a missing or corrupt record.
    pub callable_regions: Option<CallableRegionMetadata>,
}

impl Body {
    pub fn local(&self, id: LocalId) -> &LocalDecl {
        &self.locals[id.0 as usize]
    }

    pub fn block(&self, id: BasicBlockId) -> &BasicBlock {
        &self.blocks[id.0 as usize]
    }

    pub fn return_ty(&self) -> Ty {
        self.locals[0].ty
    }

    /// Parameters, in declaration order.
    pub fn args(&self) -> impl Iterator<Item = (LocalId, &LocalDecl)> {
        (1..=self.arg_count).map(move |i| (LocalId(i as u32), &self.locals[i]))
    }
}

#[derive(Debug)]
pub struct BasicBlock {
    pub stmts: Vec<Stmt>,
    pub terminator: Terminator,
    /// Where the terminator came from. A call, a branch condition and a loop
    /// test are all reads that an analysis has to be able to point at.
    pub terminator_span: Span,
}

/// A memory location: a local with a chain of projections.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Place {
    pub local: LocalId,
    pub projection: Vec<Projection>,
}

impl Place {
    pub fn local(local: LocalId) -> Place {
        Place { local, projection: Vec::new() }
    }

    pub fn field(mut self, index: usize) -> Place {
        self.projection.push(Projection::Field(index));
        self
    }

    /// `[BRW-5]` — an index the compiler knows. Kept apart from `index` so
    /// `overlaps` can tell two constant indices apart, which is the whole of
    /// that rule's exemption.
    pub fn const_index(mut self, value: u64) -> Place {
        self.projection.push(Projection::ConstIndex(value));
        self
    }

    pub fn index(mut self, local: LocalId) -> Place {
        self.projection.push(Projection::Index(local));
        self
    }

    pub fn downcast(mut self, variant: usize) -> Place {
        self.projection.push(Projection::Downcast(variant));
        self
    }

    pub fn is_local(&self) -> bool {
        self.projection.is_empty()
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Projection {
    Field(usize),
    /// Indexing by a local's value, with the bounds `Assert` already lowered
    /// beside it.
    Index(LocalId),
    ConstIndex(u64),
    Deref,
    /// Read an enum place as one particular variant, so that `Field` after it
    /// names that variant's payload. Only valid where the tag has already been
    /// tested — `match` puts it after the `SwitchInt` that proved it.
    Downcast(usize),
    /// An `SoA` column (`[SOA-2]`). Reserved; Phase 6 emits it.
    Column(usize),
}

/// One input region named by an inferred multi-region result slot.
/// Paths are relative to the callable's public parameter types, not MIR
/// locals, so this is suitable for an interface artifact.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ResultRegionSource {
    View { argument: usize, projection: Vec<Projection> },
    /// `[LT-4a]`'s narrow non-view Arena provenance source.
    Arena { argument: usize },
}

/// `[LT-22]` — the exact input regions from which one returned borrowed field
/// derives.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ResultFieldProvenance {
    pub result_projection: Vec<Projection>,
    pub sources: Vec<ResultRegionSource>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ResultProvenanceSummary {
    pub fields: Vec<ResultFieldProvenance>,
}

/// The operations `[LT-35]` permits a callable summary to name.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum RegionAccessKind {
    Read,
    Write,
    BorrowShared,
    BorrowMut,
    Move,
    Return,
    Publish,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ParameterFieldAccess {
    pub argument: usize,
    pub projection: Vec<Projection>,
    pub operations: Vec<RegionAccessKind>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CallableAccessSummary {
    /// Opaque/unresolved dispatch: every field and operation is possible.
    All,
    /// Exact verified direct-call accesses. Empty means no borrowed field is
    /// accessed and is deliberately distinct from unknown.
    Fields(Vec<ParameterFieldAccess>),
}

/// Canonical `[MIR-REG-1]` metadata carried by a callable's MIR artifact.
/// `result` is absent for a callable with no exact multi-region result
/// relation. The ordinary one-region elision contract remains signature data.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CallableRegionMetadata {
    pub access: CallableAccessSummary,
    pub result: Option<ResultProvenanceSummary>,
    fingerprint: u64,
}

impl CallableRegionMetadata {
    pub fn new(
        mut access: CallableAccessSummary,
        mut result: Option<ResultProvenanceSummary>,
    ) -> Self {
        if let CallableAccessSummary::Fields(fields) = &mut access {
            for field in &mut *fields {
                field.operations.sort();
                field.operations.dedup();
            }
            fields.sort_by(|left, right| {
                (&left.argument, &left.projection).cmp(&(&right.argument, &right.projection))
            });
            let mut merged: Vec<ParameterFieldAccess> = Vec::with_capacity(fields.len());
            for field in fields.drain(..) {
                if let Some(previous) = merged.last_mut()
                    && previous.argument == field.argument
                    && previous.projection == field.projection
                {
                    previous.operations.extend(field.operations);
                    previous.operations.sort();
                    previous.operations.dedup();
                } else {
                    merged.push(field);
                }
            }
            *fields = merged;
        }
        if let Some(result) = &mut result {
            for field in &mut result.fields {
                field.sources.sort();
                field.sources.dedup();
            }
            result
                .fields
                .sort_by(|left, right| left.result_projection.cmp(&right.result_projection));
            let mut merged: Vec<ResultFieldProvenance> = Vec::with_capacity(result.fields.len());
            for field in result.fields.drain(..) {
                if let Some(previous) = merged.last_mut()
                    && previous.result_projection == field.result_projection
                {
                    previous.sources.extend(field.sources);
                    previous.sources.sort();
                    previous.sources.dedup();
                } else {
                    merged.push(field);
                }
            }
            result.fields = merged;
        }
        let mut metadata = Self { access, result, fingerprint: 0 };
        metadata.fingerprint = metadata.compute_fingerprint();
        metadata
    }

    /// Stable summary input for `[LT-40]`'s future interface/cache hash. This
    /// deliberately does not use Rust's implementation-defined hashers.
    pub fn fingerprint(&self) -> u64 {
        self.fingerprint
    }

    pub fn fingerprint_is_valid(&self) -> bool {
        self.fingerprint == self.compute_fingerprint()
    }

    #[cfg(test)]
    pub(crate) fn corrupt_fingerprint_for_test(&mut self) {
        self.fingerprint ^= 1;
    }

    fn compute_fingerprint(&self) -> u64 {
        let mut hash = StableFingerprint::new();
        match &self.access {
            CallableAccessSummary::All => hash.byte(0),
            CallableAccessSummary::Fields(fields) => {
                hash.byte(1);
                hash.usize(fields.len());
                for field in fields {
                    hash.usize(field.argument);
                    hash.projections(&field.projection);
                    hash.usize(field.operations.len());
                    for operation in &field.operations {
                        hash.byte(match operation {
                            RegionAccessKind::Read => 0,
                            RegionAccessKind::Write => 1,
                            RegionAccessKind::BorrowShared => 2,
                            RegionAccessKind::BorrowMut => 3,
                            RegionAccessKind::Move => 4,
                            RegionAccessKind::Return => 5,
                            RegionAccessKind::Publish => 6,
                        });
                    }
                }
            }
        }
        match &self.result {
            None => hash.byte(0),
            Some(result) => {
                hash.byte(1);
                hash.usize(result.fields.len());
                for field in &result.fields {
                    hash.projections(&field.result_projection);
                    hash.usize(field.sources.len());
                    for source in &field.sources {
                        match source {
                            ResultRegionSource::View { argument, projection } => {
                                hash.byte(0);
                                hash.usize(*argument);
                                hash.projections(projection);
                            }
                            ResultRegionSource::Arena { argument } => {
                                hash.byte(1);
                                hash.usize(*argument);
                            }
                        }
                    }
                }
            }
        }
        hash.finish()
    }
}

/// Fixed FNV-1a encoding used only for compiler metadata identity.
struct StableFingerprint(u64);

impl StableFingerprint {
    fn new() -> Self {
        Self(0xcbf29ce484222325)
    }

    fn byte(&mut self, value: u8) {
        self.0 ^= u64::from(value);
        self.0 = self.0.wrapping_mul(0x100000001b3);
    }

    fn usize(&mut self, value: usize) {
        for byte in (value as u64).to_le_bytes() {
            self.byte(byte);
        }
    }

    fn projections(&mut self, projections: &[Projection]) {
        self.usize(projections.len());
        for projection in projections {
            match projection {
                Projection::Field(index) => {
                    self.byte(0);
                    self.usize(*index);
                }
                Projection::Index(local) => {
                    self.byte(1);
                    self.usize(local.0 as usize);
                }
                Projection::ConstIndex(index) => {
                    self.byte(2);
                    for byte in index.to_le_bytes() {
                        self.byte(byte);
                    }
                }
                Projection::Deref => self.byte(3),
                Projection::Downcast(variant) => {
                    self.byte(4);
                    self.usize(*variant);
                }
                Projection::Column(column) => {
                    self.byte(5);
                    self.usize(*column);
                }
            }
        }
    }

    fn finish(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug)]
pub enum Operand {
    /// Read a place without disturbing it. Only valid for `Copy` types.
    Copy(Place),
    /// Read a place and leave it uninitialised.
    Move(Place),
    Const(Const),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Const {
    Int { value: u128, ty: Ty },
    Float { value: f64, ty: Ty },
    Bool(bool),
    /// A string literal with static region (`[LEX-20]`).
    Str(String),
    /// `[CELL-5]` — a C string literal for a `RefCell` conflicting-borrow
    /// location. Renders as `"path"` (a `const char*`), stored into the cell's
    /// `borrow_file` field (a `*u8`, via a cast in the backend) so the panic
    /// names the conflicting borrow's source location in debug and release.
    /// Kept apart from `Str` (which is an `ember_str` view with a region)
    /// because the file field must not make the cell a view type (`[TYP-15]`).
    CStr(String),
    /// `[FN-6]` — a named function as a value: its mangled symbol, which in C
    /// is the function's address.
    Fn(String),
    /// The unit value.
    Void,
}

#[derive(Clone, Debug)]
pub enum Rvalue {
    Use(Operand),
    BinaryOp { op: BinOp, lhs: Operand, rhs: Operand },
    UnaryOp { op: UnOp, operand: Operand },
    Cast { kind: CastKind, operand: Operand, to: Ty },
    /// Building a struct, tuple or array from its elements, in order.
    Aggregate { kind: AggregateKind, operands: Vec<Operand> },
    /// `[value; count]`. Kept apart from `Aggregate` so that a large array
    /// stays one statement instead of `count` operands — `[0; 4096]` would
    /// otherwise be four thousand entries in the IR and in the emitted C.
    Repeat { value: Operand, count: u64 },
    /// The tag of an enum value, as its repr integer. This is what `match`
    /// switches on and what `as` on a unit-only enum reads (`[ENM-3]`).
    Discriminant(Place),
    /// The address of a place. A `mut` argument is passed this way, so the
    /// callee writes through to the caller's variable.
    Ref { place: Place, mutable: bool },
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum AggregateKind {
    Struct(StructId),
    Tuple,
    Array,
    /// One variant of an enum, with its payload in field order. The whole
    /// value — tag and payload together — is built in one statement.
    Enum(EnumId, usize),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CastKind {
    /// `[TYP-6]` — truncation, float-to-int saturation, int-to-float rounding.
    Numeric,
    /// A lossless widening inserted implicitly (`[TYP-5]`).
    Widen,
}

pub use ember_hir::{BinOp, Builtin, UnOp};

/// One statement, with the source location it came from.
#[derive(Debug)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

impl Stmt {
    pub fn new(kind: StmtKind, span: Span) -> Stmt {
        Stmt { kind, span }
    }
}

#[derive(Debug)]
pub enum StmtKind {
    Assign { place: Place, rvalue: Rvalue },
    /// Arithmetic that reports whether it overflowed.
    ///
    /// Rust MIR models this as an rvalue producing a `(T, bool)` tuple. Here
    /// it writes two places instead, which maps directly onto the C helper
    /// `bool ember_ck_add_i32(a, b, &dest)` and needs no tuple support in the
    /// backend. The `Assert` on `overflow` follows in the terminator.
    CheckedBinaryOp { dest: Place, overflow: Place, op: BinOp, lhs: Operand, rhs: Operand },
    /// A local comes into scope. Drives `[DRP-2]`'s reverse-order drops and,
    /// from Phase 2, the borrow checker's loan-kill analysis.
    StorageLive(LocalId),
    StorageDead(LocalId),
    /// `[OWN-2]`, `[DRP-2]` — the value in `place` reaches the end of its
    /// life here. Drop elaboration (Part XVIII §4.9) turns this into nothing,
    /// into the type's drop glue, or into a test of `flag` first.
    ///
    /// `flag` is `[OWN-3]`'s drop flag: a local that says whether the value
    /// is still there on this path. `None` means it always is.
    Drop { place: Place, flag: Option<LocalId> },
    Nop,
}

impl StmtKind {
    /// Statements that record scope rather than execute source. `[CG-C-8]`
    /// requires a `#line` for every statement the backend emits, and these
    /// emit nothing, so the verifier does not demand a span for them.
    pub fn is_bookkeeping(&self) -> bool {
        matches!(
            self,
            StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop
        )
    }

    pub fn describe(&self) -> &'static str {
        match self {
            StmtKind::Assign { .. } => "an assignment",
            StmtKind::CheckedBinaryOp { .. } => "a checked arithmetic statement",
            StmtKind::StorageLive(_) => "a storage-live marker",
            StmtKind::StorageDead(_) => "a storage-dead marker",
            StmtKind::Drop { .. } => "a drop",
            StmtKind::Nop => "a nop",
        }
    }
}

#[derive(Debug)]
pub enum Terminator {
    Goto(BasicBlockId),
    /// A conditional branch on an integer or boolean discriminant.
    /// Case values are signed: an enum discriminant may be negative
    /// (`Forward = -1`), and rendering one through `u128` would print a huge
    /// positive number into the emitted `switch`.
    SwitchInt { discr: Operand, targets: Vec<(i128, BasicBlockId)>, otherwise: BasicBlockId },
    Return,
    Unreachable,
    Call { func: FuncRef, args: Vec<Operand>, dest: Place, next: BasicBlockId },
    /// A runtime check. Control reaches `next` when `cond` equals `expected`;
    /// otherwise the program panics with `msg`.
    ///
    /// Kept as a terminator rather than a statement so that the borrow checker
    /// and the effect analysis both see the branch: a function containing one
    /// carries the `Panic` effect (`[EFF-*]`).
    Assert {
        cond: Operand,
        expected: bool,
        msg: AssertKind,
        next: BasicBlockId,
        /// The operation being checked, so the panic names the right line.
        span: Span,
    },
}

/// What a failed [`Terminator::Assert`] panics about. Each maps to one
/// `ember_panic_*` entry point in the runtime (Part XVIII §9).
#[derive(Clone, Debug)]
pub enum AssertKind {
    /// `[TYP-8]` — an overflowing `+`, `-`, `*` or `<<` under
    /// `OverflowPolicy::Panic`.
    Overflow(BinOp),
    /// `[TYP-8]` — `/` or `%` by zero. Always checked, whatever the policy.
    DivisionByZero,
    /// `[TYP-8]` — `T.MIN / -1`, whose true result is not representable.
    /// Always checked.
    SignedDivisionOverflow,
    /// `[TYP-10]` — a shift amount at or past the type's width.
    ShiftTooLarge,
    /// Bounds. Phase 2 emits these; the shape is here so the backend needs no
    /// change then.
    Bounds { len: Operand, index: Operand },
    /// `[CELL-5]` — a `RefCell` borrow found contention. `file`/`line` are the
    /// conflicting borrow's source location, loaded from the cell's location
    /// fields; the panic names them in debug and release (`[CELL-9]`). Like
    /// `Bounds`, the operands travel here so the backend needs no new call
    /// shape for them.
    RefCellBorrow { file: Operand, line: Operand },
}

impl AssertKind {
    /// The runtime function a failure calls. `[RT-5]` — the prefix comes
    /// from `ember_branding`, so it is spelled once in the workspace.
    pub fn runtime_entry(&self) -> String {
        let name = match self {
            AssertKind::Overflow(_) => "panic_overflow",
            AssertKind::DivisionByZero => "panic_div_zero",
            AssertKind::SignedDivisionOverflow => "panic_overflow",
            AssertKind::ShiftTooLarge => "panic_overflow",
            AssertKind::Bounds { .. } => "panic_bounds",
            AssertKind::RefCellBorrow { .. } => "panic_refcell",
        };
        ember_branding::runtime(name)
    }
}

#[derive(Clone, Debug)]
pub enum FuncRef {
    /// A direct call to a body in this compilation unit.
    Direct { symbol: String },
    /// `[CLO-3]` — a call through a value of function type. The operand holds
    /// the callee, so the region machinery sees it as an ordinary read.
    Indirect(Operand),
    /// A call the compiler provides itself, lowered to an `ember_rt` entry.
    Builtin { which: ember_hir::Builtin, arg_ty: Ty },
}

/// `--emit=mir`: a stable textual form, for snapshot tests.
pub fn dump(bodies: &[Body], types: &ember_types::TypeTable) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    for body in bodies {
        let _ = writeln!(out, "fn {}:", body.symbol);
        if let Some(metadata) = &body.callable_regions {
            let _ = writeln!(
                out,
                "  callable-regions {:016x}:",
                metadata.fingerprint()
            );
            match &metadata.access {
                CallableAccessSummary::All => {
                    let _ = writeln!(out, "    access all-fields");
                }
                CallableAccessSummary::Fields(fields) if fields.is_empty() => {
                    let _ = writeln!(out, "    access none");
                }
                CallableAccessSummary::Fields(fields) => {
                    for field in fields {
                        let operations = field
                            .operations
                            .iter()
                            .map(|operation| match operation {
                                RegionAccessKind::Read => "read",
                                RegionAccessKind::Write => "write",
                                RegionAccessKind::BorrowShared => "borrow_shared",
                                RegionAccessKind::BorrowMut => "borrow_mut",
                                RegionAccessKind::Move => "move",
                                RegionAccessKind::Return => "return",
                                RegionAccessKind::Publish => "publish",
                            })
                            .collect::<Vec<_>>()
                            .join(",");
                        let _ = writeln!(
                            out,
                            "    access arg{}{} = {operations}",
                            field.argument,
                            dump_region_path(&field.projection)
                        );
                    }
                }
            }
            if let Some(result) = &metadata.result {
                for field in &result.fields {
                    let sources = field
                        .sources
                        .iter()
                        .map(dump_region_source)
                        .collect::<Vec<_>>()
                        .join(" & ");
                    let _ = writeln!(
                        out,
                        "    result{} <- {sources}",
                        dump_region_path(&field.result_projection)
                    );
                }
            } else {
                let _ = writeln!(out, "    result signature-elision");
            }
        }
        for (i, local) in body.locals.iter().enumerate() {
            let name = local.name.as_deref().unwrap_or("");
            let _ = writeln!(
                out,
                "  let _{i}: {}   // {:?} {name}",
                types.display(local.ty),
                local.kind
            );
        }
        for (i, block) in body.blocks.iter().enumerate() {
            let _ = writeln!(out, "  bb{i}:");
            for stmt in &block.stmts {
                let _ = writeln!(out, "    {}", dump_stmt(stmt, types));
            }
            let _ = writeln!(out, "    {}", dump_terminator(&block.terminator, types));
        }
    }
    out
}

fn dump_region_source(source: &ResultRegionSource) -> String {
    match source {
        ResultRegionSource::View { argument, projection } => {
            format!("arg{argument}{}", dump_region_path(projection))
        }
        ResultRegionSource::Arena { argument } => format!("arena-arg{argument}"),
    }
}

fn dump_region_path(projections: &[Projection]) -> String {
    let mut out = String::new();
    for projection in projections {
        match projection {
            Projection::Field(index) => out.push_str(&format!(".{index}")),
            Projection::Index(local) => out.push_str(&format!("[_{}]", local.0)),
            Projection::ConstIndex(index) => out.push_str(&format!("[{index}]")),
            Projection::Deref => out.push_str(".*"),
            Projection::Downcast(variant) => out.push_str(&format!(" as variant {variant}")),
            Projection::Column(column) => out.push_str(&format!(".col{column}")),
        }
    }
    out
}

fn dump_stmt(stmt: &Stmt, types: &ember_types::TypeTable) -> String {
    match &stmt.kind {
        StmtKind::Assign { place, rvalue } => {
            format!("{} = {}", dump_place(place), dump_rvalue(rvalue, types))
        }
        StmtKind::CheckedBinaryOp { dest, overflow, op, lhs, rhs } => format!(
            "({}, {}) = checked {} {} {}",
            dump_place(dest),
            dump_place(overflow),
            dump_operand(lhs, types),
            op.c_operator(),
            dump_operand(rhs, types)
        ),
        StmtKind::StorageLive(l) => format!("StorageLive(_{})", l.0),
        StmtKind::StorageDead(l) => format!("StorageDead(_{})", l.0),
        StmtKind::Drop { place, flag } => match flag {
            Some(flag) => format!("drop({}) if _{}", dump_place(place), flag.0),
            None => format!("drop({})", dump_place(place)),
        },
        StmtKind::Nop => "nop".to_string(),
    }
}

fn dump_place(place: &Place) -> String {
    let mut out = format!("_{}", place.local.0);
    for projection in &place.projection {
        match projection {
            Projection::Field(i) => out.push_str(&format!(".{i}")),
            Projection::Index(l) => out.push_str(&format!("[_{}]", l.0)),
            Projection::ConstIndex(i) => out.push_str(&format!("[{i}]")),
            Projection::Deref => out = format!("(*{out})"),
            Projection::Downcast(v) => out.push_str(&format!(" as variant {v}")),
            Projection::Column(i) => out.push_str(&format!(".col{i}")),
        }
    }
    out
}

fn dump_operand(operand: &Operand, types: &ember_types::TypeTable) -> String {
    match operand {
        Operand::Copy(p) => format!("copy {}", dump_place(p)),
        Operand::Move(p) => format!("move {}", dump_place(p)),
        Operand::Const(c) => match c {
            Const::Int { value, ty } => format!("const {value}_{}", types.display(*ty)),
            Const::Float { value, ty } => format!("const {value:?}_{}", types.display(*ty)),
            Const::Bool(b) => format!("const {b}"),
            Const::Str(s) => format!("const {s:?}"),
            Const::CStr(s) => format!("const cstr {s:?}"),
            Const::Fn(symbol) => format!("const fn {symbol}"),
            Const::Void => "const ()".to_string(),
        },
    }
}

fn dump_rvalue(rvalue: &Rvalue, types: &ember_types::TypeTable) -> String {
    match rvalue {
        Rvalue::Use(o) => dump_operand(o, types),
        Rvalue::BinaryOp { op, lhs, rhs } => format!(
            "{} {} {}",
            dump_operand(lhs, types),
            op.c_operator(),
            dump_operand(rhs, types)
        ),
        Rvalue::UnaryOp { op, operand } => format!("{op:?} {}", dump_operand(operand, types)),
        Rvalue::Cast { kind, operand, to } => {
            format!("{:?}({}) as {}", kind, dump_operand(operand, types), types.display(*to))
        }
        Rvalue::Aggregate { kind, operands } => {
            let inner: Vec<String> = operands.iter().map(|o| dump_operand(o, types)).collect();
            let name = match kind {
                AggregateKind::Struct(id) => types.struct_def(*id).name.to_string(),
                AggregateKind::Tuple => "tuple".to_string(),
                AggregateKind::Array => "array".to_string(),
                AggregateKind::Enum(id, variant) => {
                    let def = types.enum_def(*id);
                    format!("{}.{}", def.name, def.variants[*variant].name)
                }
            };
            format!("{name}({})", inner.join(", "))
        }
        Rvalue::Repeat { value, count } => {
            format!("[{}; {count}]", dump_operand(value, types))
        }
        Rvalue::Discriminant(place) => format!("discriminant({})", dump_place(place)),
        Rvalue::Ref { place, mutable } => {
            let kind = if *mutable { "&mut " } else { "&" };
            format!("{kind}{}", dump_place(place))
        }
    }
}

fn dump_terminator(terminator: &Terminator, types: &ember_types::TypeTable) -> String {
    match terminator {
        Terminator::Goto(bb) => format!("goto bb{}", bb.0),
        Terminator::SwitchInt { discr, targets, otherwise } => {
            let arms: Vec<String> =
                targets.iter().map(|(v, bb)| format!("{v} -> bb{}", bb.0)).collect();
            format!(
                "switchInt({}) [{}, otherwise -> bb{}]",
                dump_operand(discr, types),
                arms.join(", "),
                otherwise.0
            )
        }
        Terminator::Assert { cond, expected, msg, next, .. } => format!(
            "assert({}{}) -> [success: bb{}, {:?}]",
            if *expected { "" } else { "!" },
            dump_operand(cond, types),
            next.0,
            msg
        ),
        Terminator::Return => "return".to_string(),
        Terminator::Unreachable => "unreachable".to_string(),
        Terminator::Call { func, args, dest, next } => {
            let inner: Vec<String> = args.iter().map(|a| dump_operand(a, types)).collect();
            let name = match func {
                FuncRef::Direct { symbol } => symbol.clone(),
                FuncRef::Builtin { which, .. } => which.name().to_string(),
                FuncRef::Indirect(operand) => dump_operand(operand, types),
            };
            format!("{} = {name}({}) -> bb{}", dump_place(dest), inner.join(", "), next.0)
        }
    }
}
