//! MIR: a control-flow graph over places and operands (Part XVIII §4).
//!
//! MIR is where the borrow checker, drop elaboration and the effect analysis
//! will run (Phases 2–4), so it exists from Phase 0 even though Phase 0 only
//! lowers straight-line code. Codegen reads MIR, never HIR — writing the C
//! backend against HIR would mean writing it twice.

use ember_span::{Span, Symbol};
use ember_types::{EnumId, StructId, Ty};

pub mod lower;
pub mod verify;

pub use lower::lower;
/// Exact source-level parameter mode retained by MIR for callable contracts.
/// Consumers of MIR should name this boundary fact rather than depending on
/// HIR solely to construct or inspect a MIR body.
pub use ember_hir::Mode as ParameterMode;

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
    /// The declaration's explicit unsafe-call boundary.
    pub is_unsafe: bool,
    /// The declared external ABI, if any; `None` denotes Ember's ordinary
    /// callable ABI.
    pub abi: Option<String>,
    pub locals: Vec<LocalDecl>,
    pub blocks: Vec<BasicBlock>,
    pub arg_count: usize,
    /// `[FN-1]` — the exact declared mode for every parameter, in the same
    /// order as locals `1..=arg_count`. `borrowed_params` remains the compact
    /// move-checker subset; it cannot distinguish `mut` from `owned` and is
    /// therefore insufficient for a public callable signature.
    pub param_modes: Vec<ParameterMode>,
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
    /// Compiler-internal identity of the synthesized environment struct for a
    /// capturing closure. This preserves the proof boundary between the
    /// closure body and the synthetic capture borrows in its creator; it has
    /// no source-level, ABI, layout, or generated-C meaning.
    pub closure_environment: Option<StructId>,
    /// Whether `closure_environment`, when present, owns its capture fields
    /// because the source closure was written `owned fn`. This drives the
    /// `[LT-42]` static-region storage check at the environment construction
    /// site; it never becomes ABI data.
    pub closure_captures_by_move: bool,
    /// `[DSP-2]` — compiler-only class method identity used to build and
    /// select vtables.  It is not runtime metadata in the Ember value ABI.
    pub class_owner: Option<ember_types::ClassId>,
    pub class_virtual_slot: Option<usize>,
    /// See `ember_hir::Function::is_abstract`. This body is declaration
    /// metadata for virtual-table layout, not executable code.
    pub is_abstract: bool,
    /// `[EXC-3a]` — dynamic accesses removed by a verified static proof.
    /// These records are compiler metadata only; the C backend serializes
    /// them into the `[EFF-10]` safety side table.
    pub elided_accesses: Vec<ElidedAccess>,
    /// `[EXC-8]`–`[EXC-12]` — accesses whose runtime bracket is one
    /// compiler-internal loop interval rather than one interval per iteration.
    /// This is MIR metadata only: it neither creates a source-level value nor
    /// changes the Ember aliasing model.
    pub hoisted_accesses: Vec<HoistedAccess>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElidedAccess {
    pub span: Span,
    pub reason: AccessElisionReason,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccessElisionReason {
    UniqueHandle,
}

impl AccessElisionReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UniqueHandle => "unique_handle",
        }
    }
}

/// Evidence retained when a dynamic class-exclusivity check is moved from a
/// canonical counted-loop body to its checked preheader/postheader interval.
/// The block identifiers are compiler-internal anchors for code generation;
/// user-facing inspection renders source locations instead.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HoistedAccess {
    pub span: Span,
    pub loop_span: Span,
    pub preheader: BasicBlockId,
    pub preheader_statement: usize,
    pub postheader: BasicBlockId,
    pub place: Place,
    pub mutable: bool,
    pub proof: HoistedAccessProof,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HoistedAccessProof {
    StableReceiverDirectCall,
}

impl HoistedAccessProof {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::StableReceiverDirectCall => "stable_receiver_direct_call",
        }
    }
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

#[derive(Clone, Debug)]
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
        Place {
            local,
            projection: Vec::new(),
        }
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
    View {
        argument: usize,
        projection: Vec<Projection>,
    },
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
        let mut metadata = Self {
            access,
            result,
            fingerprint: 0,
        };
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
                            ResultRegionSource::View {
                                argument,
                                projection,
                            } => {
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

    /// Encode the canonical `[MIR-REG-1]` summary for a module interface
    /// artifact. This deliberately uses a small explicit format rather than
    /// Rust object serialization: interface artifacts cross compiler runs and
    /// must not inherit implementation-defined layout or map iteration order.
    ///
    /// The trailing FNV-1a value is redundant with the payload on purpose. It
    /// makes a stale or damaged record fail at the artifact boundary before a
    /// caller treats it as a region contract.
    pub fn to_interface_bytes(&self) -> Result<Vec<u8>, CallableRegionMetadataCodecError> {
        if !self.fingerprint_is_valid() {
            return Err(CallableRegionMetadataCodecError::StaleFingerprint);
        }
        let mut out = self.canonical_payload_bytes();
        out.extend_from_slice(&self.fingerprint.to_le_bytes());
        Ok(out)
    }

    /// Decode one complete canonical summary from an interface artifact.
    /// Rejecting non-canonical encodings matters: two byte representations of
    /// the same contract would make `[LT-40]` cache identity depend on an
    /// incidental writer rather than the semantic summary.
    pub fn from_interface_bytes(bytes: &[u8]) -> Result<Self, CallableRegionMetadataCodecError> {
        let mut reader = InterfaceReader::new(bytes);
        let access = match reader.byte()? {
            0 => CallableAccessSummary::All,
            1 => {
                let count = reader.count()?;
                let mut fields = Vec::with_capacity(count);
                for _ in 0..count {
                    let argument = reader.usize()?;
                    let projection = reader.projections()?;
                    let operation_count = reader.count()?;
                    let mut operations = Vec::with_capacity(operation_count);
                    for _ in 0..operation_count {
                        operations.push(reader.region_access_kind()?);
                    }
                    fields.push(ParameterFieldAccess {
                        argument,
                        projection,
                        operations,
                    });
                }
                CallableAccessSummary::Fields(fields)
            }
            tag => {
                return Err(CallableRegionMetadataCodecError::InvalidTag {
                    kind: "access",
                    tag,
                });
            }
        };
        let result = match reader.byte()? {
            0 => None,
            1 => {
                let count = reader.count()?;
                let mut fields = Vec::with_capacity(count);
                for _ in 0..count {
                    let result_projection = reader.projections()?;
                    let source_count = reader.count()?;
                    let mut sources = Vec::with_capacity(source_count);
                    for _ in 0..source_count {
                        let source = match reader.byte()? {
                            0 => ResultRegionSource::View {
                                argument: reader.usize()?,
                                projection: reader.projections()?,
                            },
                            1 => ResultRegionSource::Arena {
                                argument: reader.usize()?,
                            },
                            tag => {
                                return Err(CallableRegionMetadataCodecError::InvalidTag {
                                    kind: "result source",
                                    tag,
                                });
                            }
                        };
                        sources.push(source);
                    }
                    fields.push(ResultFieldProvenance {
                        result_projection,
                        sources,
                    });
                }
                Some(ResultProvenanceSummary { fields })
            }
            tag => {
                return Err(CallableRegionMetadataCodecError::InvalidTag {
                    kind: "result",
                    tag,
                });
            }
        };

        let payload_end = reader.position();
        let stored_fingerprint = reader.u64()?;
        if !reader.is_finished() {
            return Err(CallableRegionMetadataCodecError::TrailingBytes);
        }

        let metadata = Self::new(access, result);
        if metadata.fingerprint != stored_fingerprint {
            return Err(CallableRegionMetadataCodecError::StaleFingerprint);
        }
        if metadata.canonical_payload_bytes() != bytes[..payload_end] {
            return Err(CallableRegionMetadataCodecError::NonCanonical);
        }
        Ok(metadata)
    }

    fn canonical_payload_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        match &self.access {
            CallableAccessSummary::All => out.push(0),
            CallableAccessSummary::Fields(fields) => {
                out.push(1);
                push_count(&mut out, fields.len());
                for field in fields {
                    push_usize(&mut out, field.argument);
                    push_projections(&mut out, &field.projection);
                    push_count(&mut out, field.operations.len());
                    for operation in &field.operations {
                        out.push(match operation {
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
            None => out.push(0),
            Some(result) => {
                out.push(1);
                push_count(&mut out, result.fields.len());
                for field in &result.fields {
                    push_projections(&mut out, &field.result_projection);
                    push_count(&mut out, field.sources.len());
                    for source in &field.sources {
                        match source {
                            ResultRegionSource::View {
                                argument,
                                projection,
                            } => {
                                out.push(0);
                                push_usize(&mut out, *argument);
                                push_projections(&mut out, projection);
                            }
                            ResultRegionSource::Arena { argument } => {
                                out.push(1);
                                push_usize(&mut out, *argument);
                            }
                        }
                    }
                }
            }
        }
        out
    }
}

/// The interface-artifact codec's errors are deliberately distinguished from
/// ordinary source diagnostics. A malformed or stale compiler artifact is a
/// compiler/build failure under `[LT-40]`, never an optimisation fallback.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CallableRegionMetadataCodecError {
    UnexpectedEof,
    LengthOverflow,
    InvalidTag { kind: &'static str, tag: u8 },
    StaleFingerprint,
    NonCanonical,
    TrailingBytes,
}

impl std::fmt::Display for CallableRegionMetadataCodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "truncated callable-region metadata"),
            Self::LengthOverflow => write!(f, "callable-region metadata has an invalid length"),
            Self::InvalidTag { kind, tag } => {
                write!(f, "callable-region metadata has invalid {kind} tag {tag}")
            }
            Self::StaleFingerprint => {
                write!(
                    f,
                    "callable-region metadata fingerprint is stale or corrupt"
                )
            }
            Self::NonCanonical => write!(f, "callable-region metadata is not canonical"),
            Self::TrailingBytes => write!(f, "callable-region metadata has trailing bytes"),
        }
    }
}

impl std::error::Error for CallableRegionMetadataCodecError {}

fn push_count(out: &mut Vec<u8>, value: usize) {
    out.extend_from_slice(&(value as u64).to_le_bytes());
}

fn push_usize(out: &mut Vec<u8>, value: usize) {
    out.extend_from_slice(&(value as u64).to_le_bytes());
}

fn push_projections(out: &mut Vec<u8>, projections: &[Projection]) {
    push_count(out, projections.len());
    for projection in projections {
        match projection {
            Projection::Field(field) => {
                out.push(0);
                push_usize(out, *field);
            }
            Projection::Index(local) => {
                out.push(1);
                out.extend_from_slice(&local.0.to_le_bytes());
            }
            Projection::ConstIndex(index) => {
                out.push(2);
                out.extend_from_slice(&index.to_le_bytes());
            }
            Projection::Deref => out.push(3),
            Projection::Downcast(variant) => {
                out.push(4);
                push_usize(out, *variant);
            }
            Projection::Column(column) => {
                out.push(5);
                push_usize(out, *column);
            }
        }
    }
}

struct InterfaceReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> InterfaceReader<'a> {
    const MAX_COUNT: usize = 1_000_000;

    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn position(&self) -> usize {
        self.position
    }

    fn is_finished(&self) -> bool {
        self.position == self.bytes.len()
    }

    fn byte(&mut self) -> Result<u8, CallableRegionMetadataCodecError> {
        let byte = *self
            .bytes
            .get(self.position)
            .ok_or(CallableRegionMetadataCodecError::UnexpectedEof)?;
        self.position += 1;
        Ok(byte)
    }

    fn fixed<const N: usize>(&mut self) -> Result<[u8; N], CallableRegionMetadataCodecError> {
        let end = self
            .position
            .checked_add(N)
            .ok_or(CallableRegionMetadataCodecError::LengthOverflow)?;
        let bytes = self
            .bytes
            .get(self.position..end)
            .ok_or(CallableRegionMetadataCodecError::UnexpectedEof)?;
        self.position = end;
        bytes
            .try_into()
            .map_err(|_| CallableRegionMetadataCodecError::UnexpectedEof)
    }

    fn u64(&mut self) -> Result<u64, CallableRegionMetadataCodecError> {
        Ok(u64::from_le_bytes(self.fixed()?))
    }

    fn usize(&mut self) -> Result<usize, CallableRegionMetadataCodecError> {
        usize::try_from(self.u64()?).map_err(|_| CallableRegionMetadataCodecError::LengthOverflow)
    }

    fn count(&mut self) -> Result<usize, CallableRegionMetadataCodecError> {
        let count = self.usize()?;
        if count > Self::MAX_COUNT {
            return Err(CallableRegionMetadataCodecError::LengthOverflow);
        }
        Ok(count)
    }

    fn projections(&mut self) -> Result<Vec<Projection>, CallableRegionMetadataCodecError> {
        let count = self.count()?;
        let mut projections = Vec::with_capacity(count);
        for _ in 0..count {
            let projection = match self.byte()? {
                0 => Projection::Field(self.usize()?),
                1 => Projection::Index(LocalId(u32::from_le_bytes(self.fixed()?))),
                2 => Projection::ConstIndex(self.u64()?),
                3 => Projection::Deref,
                4 => Projection::Downcast(self.usize()?),
                5 => Projection::Column(self.usize()?),
                tag => {
                    return Err(CallableRegionMetadataCodecError::InvalidTag {
                        kind: "projection",
                        tag,
                    });
                }
            };
            projections.push(projection);
        }
        Ok(projections)
    }

    fn region_access_kind(&mut self) -> Result<RegionAccessKind, CallableRegionMetadataCodecError> {
        match self.byte()? {
            0 => Ok(RegionAccessKind::Read),
            1 => Ok(RegionAccessKind::Write),
            2 => Ok(RegionAccessKind::BorrowShared),
            3 => Ok(RegionAccessKind::BorrowMut),
            4 => Ok(RegionAccessKind::Move),
            5 => Ok(RegionAccessKind::Return),
            6 => Ok(RegionAccessKind::Publish),
            tag => Err(CallableRegionMetadataCodecError::InvalidTag {
                kind: "operation",
                tag,
            }),
        }
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
    Int {
        value: u128,
        ty: Ty,
    },
    Float {
        value: f64,
        ty: Ty,
    },
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
    BinaryOp {
        op: BinOp,
        lhs: Operand,
        rhs: Operand,
    },
    UnaryOp {
        op: UnOp,
        operand: Operand,
    },
    Cast {
        kind: CastKind,
        operand: Operand,
        to: Ty,
    },
    /// Building a struct, tuple or array from its elements, in order.
    Aggregate {
        kind: AggregateKind,
        operands: Vec<Operand>,
    },
    /// `[value; count]`. Kept apart from `Aggregate` so that a large array
    /// stays one statement instead of `count` operands — `[0; 4096]` would
    /// otherwise be four thousand entries in the IR and in the emitted C.
    Repeat {
        value: Operand,
        count: u64,
    },
    /// The tag of an enum value, as its repr integer. This is what `match`
    /// switches on and what `as` on a unit-only enum reads (`[ENM-3]`).
    Discriminant(Place),
    /// The address of a place. A `mut` argument is passed this way, so the
    /// callee writes through to the caller's variable.
    Ref {
        place: Place,
        mutable: bool,
    },
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

/// One concrete implementation used by a compiler-generated dynamic-interface
/// adapter. The erased interface signature stays on the table layout; this
/// value is the concrete callable identity.
#[derive(Clone, Debug)]
pub struct InterfaceAdapterMethod {
    pub symbol: String,
    pub receiver: ember_hir::Mode,
}

#[derive(Clone, Debug)]
pub enum CastKind {
    /// `[TYP-6]` — truncation, float-to-int saturation, int-to-float rounding.
    Numeric,
    /// A lossless widening inserted implicitly (`[TYP-5]`).
    Widen,
    /// `[CLS-4]` — a derived class handle coerced to an inherited base
    /// handle. The representation is pointer-compatible because base fields
    /// occupy the prefix of the single-inheritance object layout; this is
    /// distinct from numeric casts so later verification can audit it.
    ClassUpcast,
    /// `[CLS-4]` constructor plumbing — a derived handle viewed as its base
    /// while calling `super.init`. Unlike `ClassUpcast`, this is a borrowed
    /// pointer adjustment and MUST NOT retain the object: the scratch handle
    /// is not an owning source local.
    ClassUpcastBorrowed,
    /// `[TYP-22]` — turn a concrete borrow into the fixed `{data*, vtable*}`
    /// carrier for one checked type/interface
    /// implementation. This is not a numeric conversion: the backend must
    /// emit the corresponding concrete adapter table and preserve the source
    /// borrow without a retain or transfer.
    InterfaceUpcast {
        concrete: Ty,
        interface: Symbol,
        layout: Vec<Option<ember_hir::InterfaceSlot>>,
        implementations: Vec<Option<InterfaceAdapterMethod>>,
    },
}

pub use ember_hir::{BinOp, Builtin, UnOp};

/// One statement, with the source location it came from.
#[derive(Clone, Debug)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

impl Stmt {
    pub fn new(kind: StmtKind, span: Span) -> Stmt {
        Stmt { kind, span }
    }
}

#[derive(Clone, Debug)]
pub enum StmtKind {
    Assign {
        place: Place,
        rvalue: Rvalue,
    },
    /// `[CLS-7]`/`[EXC-1]` — a class `mut self` method owns a dynamically
    /// checked write access for the duration of the method.  The place is the
    /// dereferenced receiver handle, so the backend can pass its object header
    /// to the runtime without inventing a source-level value or ABI field.
    BeginAccess {
        place: Place,
        mutable: bool,
    },
    /// Begin an access whose reference is returned to the caller. The caller
    /// closes it through the returned payload; this preserves the exact
    /// selected object when a callee returns one of several `Shared` owners.
    BeginAccessTransfer {
        place: Place,
        mutable: bool,
    },
    /// End the access opened by [`StmtKind::BeginAccess`].  These are explicit
    /// MIR operations rather than backend-only instrumentation so borrow,
    /// verifier, inspection, and code-generation phases see the same interval.
    EndAccess {
        place: Place,
        mutable: bool,
    },
    /// Close an access started in a callee through a returned payload place.
    /// The C backend recovers the counted-object header from that payload's
    /// alignment-defined offset.
    EndAccessTransfer {
        place: Place,
        mutable: bool,
    },
    /// Arithmetic that reports whether it overflowed.
    ///
    /// Rust MIR models this as an rvalue producing a `(T, bool)` tuple. Here
    /// it writes two places instead, which maps directly onto the C helper
    /// `bool ember_ck_add_i32(a, b, &dest)` and needs no tuple support in the
    /// backend. The `Assert` on `overflow` follows in the terminator.
    CheckedBinaryOp {
        dest: Place,
        overflow: Place,
        op: BinOp,
        lhs: Operand,
        rhs: Operand,
    },
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
    Drop {
        place: Place,
        flag: Option<LocalId>,
    },
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
            StmtKind::BeginAccess { .. } => "a begin-access operation",
            StmtKind::BeginAccessTransfer { .. } => "a transferred begin-access operation",
            StmtKind::EndAccess { .. } => "an end-access operation",
            StmtKind::EndAccessTransfer { .. } => "a transferred end-access operation",
            StmtKind::CheckedBinaryOp { .. } => "a checked arithmetic statement",
            StmtKind::StorageLive(_) => "a storage-live marker",
            StmtKind::StorageDead(_) => "a storage-dead marker",
            StmtKind::Drop { .. } => "a drop",
            StmtKind::Nop => "a nop",
        }
    }
}

#[derive(Clone, Debug)]
pub enum Terminator {
    Goto(BasicBlockId),
    /// A conditional branch on an integer or boolean discriminant.
    /// Case values are signed: an enum discriminant may be negative
    /// (`Forward = -1`), and rendering one through `u128` would print a huge
    /// positive number into the emitted `switch`.
    SwitchInt {
        discr: Operand,
        targets: Vec<(i128, BasicBlockId)>,
        otherwise: BasicBlockId,
    },
    Return,
    Unreachable,
    Call {
        func: FuncRef,
        args: Vec<Operand>,
        dest: Place,
        next: BasicBlockId,
    },
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
    /// `[DSP-4]` — `as!` failed its runtime class-chain check. The v1 panic
    /// model aborts; there is no recovery edge after this assertion fails.
    Downcast,
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
            AssertKind::Downcast => "panic",
        };
        ember_branding::runtime(name)
    }
}

#[derive(Clone, Debug)]
pub enum FuncRef {
    /// A direct call to a body in this compilation unit. `latebound` is a
    /// compile-time callable-boundary fact retained for region analysis and
    /// erased before code generation.
    Direct { symbol: String, latebound: bool },
    /// `[DSP-2]` — a call through the receiver's class vtable. `owner` is
    /// the declaration that established the slot, so the C backend can use
    /// the stable base signature even when a derived override supplies the
    /// implementation.
    Virtual { owner: ember_types::ClassId, slot: usize },
    /// `[TYP-22]` — a call through a `ref dyn I` carrier. The receiver is
    /// represented by the first operand; the remaining operands use the
    /// interface declaration's parameter types. `layout` carries every slot
    /// so the C backend can emit the canonical table shape without reaching
    /// back into type-checker-only interface definitions. A `None` slot is a
    /// `Self: Sized` default: its ordinal is retained, but it has no dyn-call
    /// ABI at this boundary.
    Interface {
        interface: ember_span::Symbol,
        slot: usize,
        params: Vec<Ty>,
        ret: Ty,
        layout: Vec<Option<ember_hir::InterfaceSlot>>,
    },
    /// `[TYP-22]` — allocate a concrete payload and return its owning
    /// two-word `Box[dyn I]` carrier.
    DynBoxNew {
        concrete: Ty,
        boxed: Ty,
        interface: ember_span::Symbol,
        layout: Vec<Option<ember_hir::InterfaceSlot>>,
        implementations: Vec<Option<InterfaceAdapterMethod>>,
    },
    /// `[CLO-3]`, `[FN-6b]` — a call through a value of function type. The
    /// operand holds the callee; `latebound` is the expected callable-boundary
    /// fact and is erased before code generation.
    Indirect { operand: Operand, latebound: bool },
    /// A call the compiler provides itself, lowered to an `ember_rt` entry.
    Builtin {
        which: ember_hir::Builtin,
        arg_ty: Ty,
    },
}

/// `--emit=mir`: a stable textual form, for snapshot tests.
pub fn dump(bodies: &[Body], types: &ember_types::TypeTable) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    for body in bodies {
        let _ = writeln!(out, "fn {}:", body.symbol);
        if let Some(metadata) = &body.callable_regions {
            let _ = writeln!(out, "  callable-regions {:016x}:", metadata.fingerprint());
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
        ResultRegionSource::View {
            argument,
            projection,
        } => {
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
        StmtKind::BeginAccess { place, mutable } => {
            let mode = if *mutable { "write" } else { "read" };
            format!("begin_access_{mode}({})", dump_place(place))
        }
        StmtKind::BeginAccessTransfer { place, mutable } => {
            let mode = if *mutable { "write" } else { "read" };
            format!("begin_access_{mode}_transfer({})", dump_place(place))
        }
        StmtKind::EndAccess { place, mutable } => {
            let mode = if *mutable { "write" } else { "read" };
            format!("end_access_{mode}({})", dump_place(place))
        }
        StmtKind::EndAccessTransfer { place, mutable } => {
            let mode = if *mutable { "write" } else { "read" };
            format!("end_access_{mode}_transfer({})", dump_place(place))
        }
        StmtKind::CheckedBinaryOp {
            dest,
            overflow,
            op,
            lhs,
            rhs,
        } => format!(
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
            format!(
                "{:?}({}) as {}",
                kind,
                dump_operand(operand, types),
                types.display(*to)
            )
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
        Terminator::SwitchInt {
            discr,
            targets,
            otherwise,
        } => {
            let arms: Vec<String> = targets
                .iter()
                .map(|(v, bb)| format!("{v} -> bb{}", bb.0))
                .collect();
            format!(
                "switchInt({}) [{}, otherwise -> bb{}]",
                dump_operand(discr, types),
                arms.join(", "),
                otherwise.0
            )
        }
        Terminator::Assert {
            cond,
            expected,
            msg,
            next,
            ..
        } => format!(
            "assert({}{}) -> [success: bb{}, {:?}]",
            if *expected { "" } else { "!" },
            dump_operand(cond, types),
            next.0,
            msg
        ),
        Terminator::Return => "return".to_string(),
        Terminator::Unreachable => "unreachable".to_string(),
        Terminator::Call {
            func,
            args,
            dest,
            next,
        } => {
            let inner: Vec<String> = args.iter().map(|a| dump_operand(a, types)).collect();
            let (boundary, name) = match func {
                FuncRef::Direct { symbol, latebound } => {
                    (if *latebound { "@latebound " } else { "" }, symbol.clone())
                }
                FuncRef::Builtin { which, .. } => ("", which.name().to_string()),
                FuncRef::Virtual { owner, slot } => {
                    ("@virtual ", format!("class#{}::slot{}", owner.0, slot))
                }
                FuncRef::Interface { interface, slot, .. } => {
                    ("@dyn ", format!("{interface}::slot{slot}"))
                }
                FuncRef::DynBoxNew { interface, .. } => {
                    ("", format!("Box[dyn {interface}]"))
                }
                FuncRef::Indirect { operand, latebound } => (
                    if *latebound { "@latebound " } else { "" },
                    dump_operand(operand, types),
                ),
            };
            format!(
                "{} = {boundary}{name}({}) -> bb{}",
                dump_place(dest),
                inner.join(", "),
                next.0
            )
        }
    }
}
