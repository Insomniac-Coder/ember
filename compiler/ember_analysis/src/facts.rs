//! Canonical semantic facts shared by MIR analyses (`[IMP-7]`).
//!
//! This is the first incremental `ARCH-096-1` slice. It names and centralizes
//! facts the compiler already proved instead of introducing a parallel safety
//! model. `TypeIdentity` and `LayoutDescriptor` remain aliases of the existing
//! interned type/layout representations; borrow checking now consumes the
//! capability and access contract below; definite initialization consumes the
//! shared lattice. Ownership graphs and effect sets are intentionally not
//! represented until their producers and consumers can move together.

use std::collections::BTreeSet;

use ember_mir::{LocalId, Place, Projection};
pub use ember_types::{LayoutDescriptor, TypeIdentity};

use crate::regions::{Point, RegionVid};

/// `[IMP-7]`, Part XIX §4.6 — the canonical definite-initialization lattice.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum InitializationState {
    Uninit,
    Init,
    /// Initialized on some paths reaching this point and not on others.
    Maybe,
}

impl InitializationState {
    pub(crate) fn join(self, other: InitializationState) -> InitializationState {
        match (self, other) {
            (InitializationState::Init, InitializationState::Init) => InitializationState::Init,
            (InitializationState::Uninit, InitializationState::Uninit) => {
                InitializationState::Uninit
            }
            _ => InitializationState::Maybe,
        }
    }

    pub(crate) fn is_readable(self) -> bool {
        self == InitializationState::Init
    }
}

/// A per-local (or per-place) dataflow lattice held sparsely (`SparseStates`): its join, and the
/// value a state holds where nothing is listed.
pub trait SparseState: Copy + Eq {
    /// The value of every local a state does not list: where a body's locals start.
    const ABSENT: Self;
    fn join_state(self, other: Self) -> Self;
}

impl SparseState for InitializationState {
    const ABSENT: Self = InitializationState::Uninit;
    fn join_state(self, other: Self) -> Self {
        self.join(other)
    }
}

/// `[IMP-7]`, ODR-281 — the state of every local (or move path) at one point, held sparsely: each
/// one whose state is not `T::ABSENT`, by number, in order. A point holds the locals live there,
/// not every local of its body, so a body's dataflow grows with the body, where a state per local
/// per block grew with its square (a 3,200-call `main`'s initialization facts were 700 MB). The
/// lattice and its join are the dense one's, local by local.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SparseStates<T>(Vec<(u32, T)>);

impl<T> Default for SparseStates<T> {
    fn default() -> Self {
        SparseStates(Vec::new())
    }
}

/// `[IMP-7]` — the initialization state of every local at one point.
pub type InitializationStates = SparseStates<InitializationState>;

impl<T: SparseState> SparseStates<T> {
    /// The state of local `local`.
    pub fn get(&self, local: usize) -> T {
        match self.0.binary_search_by_key(&(local as u32), |&(at, _)| at) {
            Ok(found) => self.0[found].1,
            Err(_) => T::ABSENT,
        }
    }

    pub(crate) fn set(&mut self, local: usize, state: T) {
        let local = local as u32;
        // Locals are written mostly in the order they were made: look at the last first.
        let found = match self.0.last() {
            Some(&(last, _)) if last < local => Err(self.0.len()),
            _ => self.0.binary_search_by_key(&local, |&(at, _)| at),
        };
        match found {
            Ok(at) if state == T::ABSENT => {
                self.0.remove(at);
            }
            Ok(at) => self.0[at].1 = state,
            Err(_) if state == T::ABSENT => {}
            Err(at) => self.0.insert(at, (local, state)),
        }
    }

    /// The join of two points' states, local by local.
    pub(crate) fn join(&self, other: &SparseStates<T>) -> SparseStates<T> {
        let (mut left, mut right) = (self.0.iter().peekable(), other.0.iter().peekable());
        let mut out = Vec::with_capacity(self.0.len().max(other.0.len()));
        loop {
            let (local, a, b) = match (left.peek(), right.peek()) {
                (Some(&&(l, a)), Some(&&(r, b))) if l == r => {
                    left.next();
                    right.next();
                    (l, a, b)
                }
                (Some(&&(l, a)), Some(&&(r, _))) if l < r => {
                    left.next();
                    (l, a, T::ABSENT)
                }
                (Some(&&(l, a)), None) => {
                    left.next();
                    (l, a, T::ABSENT)
                }
                (_, Some(&&(r, b))) => {
                    right.next();
                    (r, T::ABSENT, b)
                }
                (None, None) => break,
            };
            let joined = a.join_state(b);
            if joined != T::ABSENT {
                out.push((local, joined));
            }
        }
        SparseStates(out)
    }

    /// Every local's state, of a body of `locals` locals.
    pub fn dense(&self, locals: usize) -> Vec<T> {
        let mut out = vec![T::ABSENT; locals];
        for &(local, state) in &self.0 {
            if let Some(slot) = out.get_mut(local as usize) {
                *slot = state;
            }
        }
        out
    }

    /// The highest local listed, if any (a fact naming a local its body has not is stale).
    pub(crate) fn highest(&self) -> Option<usize> {
        self.0.last().map(|&(local, _)| local as usize)
    }
}

/// `[IMP-7]` — the canonical definite-initialization facts for one MIR body.
///
/// Block entries and exits are retained together so downstream consumers do
/// not independently solve the same dataflow problem. `None` denotes an
/// unreachable block. The fact verifier checks this record against the MIR
/// before any safety-critical transformation consumes or invalidates it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct InitializationFacts {
    pub(crate) body_symbol: String,
    pub(crate) block_entry: Vec<Option<InitializationStates>>,
    pub(crate) block_exit: Vec<Option<InitializationStates>>,
}

impl InitializationFacts {
    pub fn body_symbol(&self) -> &str {
        &self.body_symbol
    }

    pub fn block_entry(&self, block: usize) -> Option<&InitializationStates> {
        self.block_entry.get(block)?.as_ref()
    }

    pub fn block_exit(&self, block: usize) -> Option<&InitializationStates> {
        self.block_exit.get(block)?.as_ref()
    }

    pub fn block_count(&self) -> usize {
        self.block_entry.len()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ProvenanceRoot {
    Param(LocalId),
    Local(LocalId),
    Static,
}

/// Concrete storage is not a lifetime. In particular, two allocations from
/// one Arena can share provenance while retaining distinct identities.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum StorageIdentity {
    PlaceRoot(LocalId),
    ArenaAllocation { arena: LocalId, site: Point },
    Static,
}

impl StorageIdentity {
    /// Return the Arena parameter that owns this concrete allocation, when
    /// the producer has proved that relationship. Provenance and storage
    /// remain separate: this is only the concrete storage owner, not a
    /// lifetime region.
    pub fn arena_owner(self) -> Option<LocalId> {
        match self {
            StorageIdentity::ArenaAllocation { arena, .. } => Some(arena),
            StorageIdentity::PlaceRoot(_) | StorageIdentity::Static => None,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum AccessPermission {
    Shared,
    Mut,
}

impl AccessPermission {
    pub fn is_mut(self) -> bool {
        self == AccessPermission::Mut
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ReferenceKind {
    Reference,
    RawPointer,
    Value,
    View,
    Handle,
    RuntimeGuard,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum OwnershipRelation {
    Owned,
    Borrowed,
    Observing,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum AcquisitionOrCheck {
    Static,
    RuntimeChecked,
    LockAcquired,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum SafetyAuthority {
    Safe,
    Unsafe,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum SynchronizationDomain {
    None,
    ThreadConfined,
    Synchronized,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ValidityInterval(pub RegionVid);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum EscapeConstraint {
    MustNotOutliveStorage,
    MustRespectReturnProvenance,
    MustRemainBorrowCompatible,
}

/// The orthogonal access axes consumed when checking a borrow conflict.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct AccessContract {
    pub permission: AccessPermission,
    pub acquisition_or_check: AcquisitionOrCheck,
    pub safety_authority: SafetyAuthority,
    pub synchronization_domain: SynchronizationDomain,
}

/// The canonical facts for one borrow-derived capability in current MIR.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BorrowCapability {
    pub type_identity: TypeIdentity,
    pub provenance_root: ProvenanceRoot,
    pub source_place: Option<Place>,
    pub storage_identity: StorageIdentity,
    pub projection_path: Vec<Projection>,
    pub region: RegionVid,
    pub reference_kind: ReferenceKind,
    pub ownership_relation: OwnershipRelation,
    pub access: AccessContract,
    pub validity_interval: ValidityInterval,
    pub escape_constraints: BTreeSet<EscapeConstraint>,
}

impl BorrowCapability {
    #[allow(clippy::too_many_arguments)]
    pub fn statically_checked_reference(
        type_identity: TypeIdentity,
        provenance_root: ProvenanceRoot,
        source_place: Place,
        storage_identity: StorageIdentity,
        region: RegionVid,
        permission: AccessPermission,
        reference_kind: ReferenceKind,
    ) -> BorrowCapability {
        BorrowCapability {
            type_identity,
            provenance_root,
            projection_path: source_place.projection.clone(),
            source_place: Some(source_place),
            storage_identity,
            region,
            reference_kind,
            ownership_relation: OwnershipRelation::Borrowed,
            access: AccessContract {
                permission,
                acquisition_or_check: AcquisitionOrCheck::Static,
                safety_authority: SafetyAuthority::Safe,
                synchronization_domain: SynchronizationDomain::None,
            },
            validity_interval: ValidityInterval(region),
            escape_constraints: BTreeSet::from([
                EscapeConstraint::MustNotOutliveStorage,
                EscapeConstraint::MustRespectReturnProvenance,
                EscapeConstraint::MustRemainBorrowCompatible,
            ]),
        }
    }

    pub fn source_place(&self) -> Option<&Place> {
        self.source_place.as_ref()
    }

    pub fn is_mut(&self) -> bool {
        self.access.permission.is_mut()
    }

    /// Whether this capability carries the storage-survival obligation that
    /// the return escape checker enforces. Keeping the query on the canonical
    /// fact prevents a consumer from assuming that every future capability
    /// kind has identical escape obligations.
    pub fn must_not_outlive_storage(&self) -> bool {
        self.escape_constraints
            .contains(&EscapeConstraint::MustNotOutliveStorage)
    }
}
