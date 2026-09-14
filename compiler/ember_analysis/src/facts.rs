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

/// `[IMP-7]` — the canonical definite-initialization facts for one MIR body.
///
/// Block entries and exits are retained together so downstream consumers do
/// not independently solve the same dataflow problem. `None` denotes an
/// unreachable block. The fact verifier checks this record against the MIR
/// before any safety-critical transformation consumes or invalidates it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct InitializationFacts {
    pub(crate) body_symbol: String,
    pub(crate) block_entry: Vec<Option<Vec<InitializationState>>>,
    pub(crate) block_exit: Vec<Option<Vec<InitializationState>>>,
}

impl InitializationFacts {
    pub fn body_symbol(&self) -> &str {
        &self.body_symbol
    }

    pub fn block_entry(&self, block: usize) -> Option<&[InitializationState]> {
        self.block_entry.get(block)?.as_deref()
    }

    pub fn block_exit(&self, block: usize) -> Option<&[InitializationState]> {
        self.block_exit.get(block)?.as_deref()
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
