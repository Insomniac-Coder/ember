//! Deterministic front-end interface artifacts (`[BLD-2]`, `[LT-40]`).
//!
//! The current compiler still checks a whole loaded module graph on every
//! invocation. This module does not pretend otherwise. It establishes the
//! first real artifact boundary: resolved callable signatures and region
//! contracts are serialized, reread, independently validated, and included
//! in the dependency identity that a later item-granular cache can reuse.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ember_mir::{CallableRegionMetadata, CallableRegionMetadataCodecError};

const MAGIC: &[u8; 4] = b"EMIF";
// Schema v4 makes callable declarations authoritative even when a generic has
// no emitted body in this compilation. Older records are incompatible tooling
// cache entries, not malformed semantic metadata, and are safely invalidated
// before they can be consumed.
// Schema 5 widens the declaration-first callable section from top-level
// functions to visible members, generic owners, interfaces, and extensions.
// Schema 4 records are well-formed but omit source contracts now required by
// `[BLD-2]`, so they must be invalidated rather than treated as stale current
// metadata.
// Schema 6 records parameter modes in implicit generic Callable bounds.
// Earlier records erased `fn(mut T)` and `fn(owned T)` at the import boundary,
// so they cannot safely participate in `[FN-6a]` checking.
// Schema 7 records the `[FN-6b]` late-bound callable-boundary fact. A schema 6
// record cannot safely be reused for a callback whose invocation-local region
// behavior is part of its canonical type identity.
// Schema 8 follows ODR-024: a borrowed parameter whose type is not `Copy` is
// passed by address, which changes the ABI and the region summary of every
// function that has one, so a schema 7 record describes a different call.
const SCHEMA_VERSION: u32 = 8;
const EXTENSION: &str = "emif";

/// A BLAKE3 identity. It is kept opaque so callers cannot accidentally use a
/// formatted string as a cache key or an ABI field.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct InterfaceHash([u8; 32]);

impl InterfaceHash {
    pub fn bytes(self) -> [u8; 32] {
        self.0
    }

    pub fn hex(self) -> String {
        self.0.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    fn of_bytes(bytes: &[u8]) -> Self {
        Self(*blake3::hash(bytes).as_bytes())
    }
}

/// The passing mode of one callable parameter. This is deliberately separate
/// from its resolved type: `mut Span[T]` is an exclusive call boundary, while
/// `owned Span[T]` consumes the value.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CallableParameterMode {
    Borrow,
    Mut,
    Owned,
}

/// One parameter in an import-visible resolved callable signature.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CallableParameter {
    pub mode: CallableParameterMode,
    /// A compiler-owned canonical spelling, never diagnostic display text.
    pub ty: String,
}

/// One declared generic parameter, represented by position rather than its
/// source spelling. Bound order is canonicalized because intersection bounds
/// have no source-order semantics.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CallableGenericParameter {
    pub bounds: Vec<String>,
    /// A source `fn(A) -> R` parameter creates the existing implicit static
    /// Callable/CallableOnce bound; its shape is caller-visible too.
    pub callable: Option<CallableGenericCallableBound>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CallableGenericCallableBound {
    pub parameters: Vec<CallableParameter>,
    pub result: String,
    pub once: bool,
    pub latebound: bool,
}

/// The resolved, monomorphic callable facts currently available at the EMIF
/// boundary. Generic declaration binders/bounds, type layouts, effects, and
/// inline-body eligibility require their own verified producers and are not
/// silently approximated here.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CallableSignature {
    pub parameters: Vec<CallableParameter>,
    pub result: String,
    pub generics: Vec<CallableGenericParameter>,
    /// Zero-based parameter positions explicitly named by `@borrows`, or
    /// `None` when ordinary elision remains the contract.
    pub borrows: Option<Vec<usize>>,
    pub is_unsafe: bool,
    /// `None` denotes Ember's ordinary ABI.
    pub abi: Option<String>,
}

impl CallableSignature {
    fn validate(&self) -> Result<(), &'static str> {
        if self.result.is_empty() {
            return Err("callable result type is empty");
        }
        if self.parameters.iter().any(|parameter| parameter.ty.is_empty()) {
            return Err("callable parameter type is empty");
        }
        if self.abi.as_deref().is_some_and(str::is_empty) {
            return Err("callable ABI is empty");
        }
        for generic in &self.generics {
            if generic.bounds.iter().any(String::is_empty) {
                return Err("callable generic bound is empty");
            }
            if generic
                .bounds
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            {
                return Err("callable generic bounds are not strictly ascending");
            }
            if let Some(callable) = &generic.callable {
                if callable.result.is_empty()
                    || callable.parameters.iter().any(|parameter| parameter.ty.is_empty())
                {
                    return Err("implicit callable bound has an empty type");
                }
            }
        }
        if let Some(borrows) = &self.borrows {
            if borrows.is_empty() {
                return Err("`@borrows` must name at least one parameter");
            }
            let mut previous = None;
            for &position in borrows {
                if position >= self.parameters.len() {
                    return Err("`@borrows` position exceeds parameter count");
                }
                if previous.is_some_and(|last| last >= position) {
                    return Err("`@borrows` positions are not strictly ascending");
                }
                previous = Some(position);
            }
        }
        Ok(())
    }
}

/// The whole import-visible contract for one callable. Region summaries and
/// signatures share a single atom so stale metadata can never be matched to a
/// different declaration shape.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CallableInterfaceContract {
    pub signature: CallableSignature,
    /// Present only when this compilation has an executable non-generic body
    /// for the declaration. A generic declaration remains a complete source
    /// interface without pretending an uninstantiated MIR body exists.
    pub metadata: Option<CallableRegionMetadata>,
}

impl CallableInterfaceContract {
    fn validate(&self) -> Result<(), &'static str> {
        self.signature.validate()?;
        if self
            .metadata
            .as_ref()
            .is_some_and(|metadata| !metadata.fingerprint_is_valid())
        {
            return Err("callable-region metadata fingerprint is invalid");
        }
        Ok(())
    }
}

/// One import-visible callable contract exported by the current
/// interface-artifact slice. This includes `pub(package)` names used within
/// the package as well as `pub` names visible to dependants, but excludes
/// module-private bodies.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CallableInterfaceRecord {
    pub symbol: String,
    pub contract: CallableInterfaceContract,
}

/// Input gathered by the driver after semantic analysis but before the final
/// consumer boundary. `direct_dependencies` names loaded modules, not paths on
/// disk, keeping absolute host paths out of the artifact bytes.
#[derive(Clone, Debug)]
pub struct ModuleInterfaceInput {
    pub module: String,
    pub source: String,
    pub language_version: String,
    pub package_config: String,
    pub direct_dependencies: Vec<String>,
    pub callables: Vec<CallableInterfaceRecord>,
}

/// The schema-v7 module artifact. Its interface hash contains resolved
/// import-visible callable signatures and callable-region contracts;
/// source/cache identity already has the shape required to absorb layouts,
/// effects, and inline bodies as those compiler facts obtain real producers.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ModuleInterfaceArtifact {
    pub module: String,
    pub compiler_version: String,
    pub language_version: String,
    pub package_config: String,
    pub source_hash: InterfaceHash,
    pub dependencies: BTreeMap<String, InterfaceHash>,
    pub callables: BTreeMap<String, CallableInterfaceContract>,
    pub interface_hash: InterfaceHash,
    pub cache_key: InterfaceHash,
}

/// The cache identity for this compiler/artifact decoder pair. Cargo's package
/// version alone is not enough to distinguish a cache schema change during
/// source development, so the schema version is deliberately part of the
/// compiler-version input required by `[BLD-2]`.
pub fn compiler_identity() -> String {
    // Two builds of one version and schema can summarise the same source
    // differently (every development build does), and a cached summary that
    // disagrees is then reported as corrupt. Naming the executable itself —
    // its size and modification time, which are metadata and cheap — makes a
    // different build a different cache key, so its artifacts are rebuilt.
    let build = std::env::current_exe()
        .ok()
        .and_then(|path| std::fs::metadata(path).ok())
        .map(|meta| {
            let modified = meta
                .modified()
                .ok()
                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                .map_or(0, |since| since.as_nanos());
            format!("+{}-{modified}", meta.len())
        })
        .unwrap_or_default();
    format!("{}+emif{SCHEMA_VERSION}{build}", env!("CARGO_PKG_VERSION"))
}

/// Whether a prior artifact was reused or its dependency identity changed.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct InterfaceCacheReport {
    pub reused: usize,
    pub invalidated: usize,
    pub created: usize,
}

/// A prepared cache transaction. Artifacts are only written after semantic
/// verification succeeds, so an erroneous source program cannot replace the
/// last verified interface record.
pub struct PreparedInterfaceCache {
    artifacts: Vec<ModuleInterfaceArtifact>,
    writes: Vec<(PathBuf, Vec<u8>)>,
    report: InterfaceCacheReport,
}

impl PreparedInterfaceCache {
    pub fn artifacts(&self) -> &[ModuleInterfaceArtifact] {
        &self.artifacts
    }

    pub fn commit(self) -> Result<InterfaceCacheReport, InterfaceArtifactError> {
        for (path, bytes) in self.writes {
            // D-189 — compilations running side by side share this directory.
            // Each writes aside and renames into place, so a reader sees the
            // old record or the new one, never a half-written file.
            let temp = path.with_extension(format!("tmp{}", std::process::id()));
            std::fs::write(&temp, bytes).map_err(|source| InterfaceArtifactError::Io {
                path: temp.clone(),
                source,
            })?;
            if let Err(source) = std::fs::rename(&temp, &path) {
                let _ = std::fs::remove_file(&temp);
                // Another compilation replaced it at the same moment; its
                // record is complete, and the next read checks its key.
                if !path.exists() {
                    return Err(InterfaceArtifactError::Io { path, source });
                }
            }
        }
        Ok(self.report)
    }
}

/// Build every module's current interface artifact. Interface hashes are
/// computed before cache keys, then each module's key receives the sorted
/// transitive hashes of the modules it imports. That makes a callable-summary
/// change in an imported module invalidate every dependent key under
/// `[LT-40]`, including through an import chain.
pub fn build_artifacts(
    inputs: &[ModuleInterfaceInput],
    compiler_version: &str,
) -> Result<Vec<ModuleInterfaceArtifact>, InterfaceArtifactError> {
    let mut inputs_by_module = BTreeMap::new();
    for input in inputs {
        if inputs_by_module
            .insert(input.module.as_str(), input)
            .is_some()
        {
            return Err(InterfaceArtifactError::DuplicateModule(
                input.module.clone(),
            ));
        }
    }

    let mut preliminary = BTreeMap::new();
    for input in inputs {
        let mut callables = BTreeMap::new();
        for callable in &input.callables {
            if let Err(reason) = callable.contract.validate() {
                return Err(InterfaceArtifactError::InvalidCallableContract {
                    module: input.module.clone(),
                    symbol: callable.symbol.clone(),
                    reason,
                });
            }
            if callables
                .insert(callable.symbol.clone(), callable.contract.clone())
                .is_some()
            {
                return Err(InterfaceArtifactError::DuplicateCallable {
                    module: input.module.clone(),
                    symbol: callable.symbol.clone(),
                });
            }
        }
        let source_hash = InterfaceHash::of_bytes(input.source.as_bytes());
        let interface_hash = interface_hash(&input.module, &callables)?;
        preliminary.insert(
            input.module.clone(),
            PreliminaryArtifact {
                source_hash,
                interface_hash,
                callables,
            },
        );
    }

    let mut artifacts = Vec::with_capacity(inputs.len());
    for input in inputs {
        let mut reachable = BTreeSet::new();
        collect_transitive_dependencies(
            &input.module,
            &input.direct_dependencies,
            &inputs_by_module,
            &mut reachable,
        )?;
        reachable.remove(&input.module);
        let dependencies = reachable
            .into_iter()
            .map(|module| {
                let interface_hash = preliminary
                    .get(&module)
                    .expect("dependency existence was checked during graph traversal")
                    .interface_hash;
                (module, interface_hash)
            })
            .collect();
        let current = preliminary
            .get(&input.module)
            .expect("every interface input has a preliminary artifact");
        let cache_key = cache_key(
            current.source_hash,
            compiler_version,
            &input.language_version,
            &input.package_config,
            &dependencies,
        );
        artifacts.push(ModuleInterfaceArtifact {
            module: input.module.clone(),
            compiler_version: compiler_version.to_string(),
            language_version: input.language_version.clone(),
            package_config: input.package_config.clone(),
            source_hash: current.source_hash,
            dependencies,
            callables: current.callables.clone(),
            interface_hash: current.interface_hash,
            cache_key,
        });
    }
    artifacts.sort_by(|left, right| left.module.cmp(&right.module));
    Ok(artifacts)
}

/// Encode and decode the records even when no on-disk cache directory was
/// requested (for example `--emit mir`). This is a real producer/consumer
/// boundary, not a test-only serializer beside analysis.
pub fn round_trip_artifacts(
    artifacts: &[ModuleInterfaceArtifact],
) -> Result<Vec<ModuleInterfaceArtifact>, InterfaceArtifactError> {
    artifacts
        .iter()
        .map(|artifact| ModuleInterfaceArtifact::from_bytes(&artifact.to_bytes()?))
        .collect()
}

/// Load verified records at an on-disk cache boundary. A matching cache key
/// must also have identical semantic contents. A mismatch is stale compiler
/// metadata and is fatal; it is never widened to an unknown summary.
pub fn prepare_interface_cache(
    directory: &Path,
    fresh: &[ModuleInterfaceArtifact],
) -> Result<PreparedInterfaceCache, InterfaceArtifactError> {
    std::fs::create_dir_all(directory).map_err(|source| InterfaceArtifactError::Io {
        path: directory.to_path_buf(),
        source,
    })?;

    let mut artifacts = Vec::with_capacity(fresh.len());
    let mut writes = Vec::new();
    let mut report = InterfaceCacheReport::default();
    for artifact in fresh {
        let path = artifact_path(directory, &artifact.module);
        match std::fs::read(&path) {
            Ok(bytes) => {
                let existing = match ModuleInterfaceArtifact::from_bytes(&bytes) {
                    Ok(existing) => existing,
                    // A recognized but superseded schema is an ordinary cache
                    // invalidation, not malformed semantic metadata. It is
                    // never consumed and cannot become an unknown summary.
                    Err(InterfaceArtifactError::UnsupportedSchema(_)) => {
                        report.invalidated += 1;
                        writes.push((path, artifact.to_bytes()?));
                        artifacts.push(artifact.clone());
                        continue;
                    }
                    Err(error) => {
                        return Err(InterfaceArtifactError::StoredArtifact {
                            path: path.clone(),
                            error: Box::new(error),
                        });
                    }
                };
                if existing.module != artifact.module {
                    return Err(InterfaceArtifactError::ModulePathCollision {
                        path,
                        expected: artifact.module.clone(),
                        found: existing.module,
                    });
                }
                if existing.cache_key == artifact.cache_key {
                    if existing.interface_hash != artifact.interface_hash
                        || existing.callables != artifact.callables
                    {
                        return Err(InterfaceArtifactError::StaleSummary {
                            module: artifact.module.clone(),
                        });
                    }
                    report.reused += 1;
                    artifacts.push(existing);
                } else {
                    report.invalidated += 1;
                    writes.push((path, artifact.to_bytes()?));
                    artifacts.push(artifact.clone());
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                report.created += 1;
                writes.push((path, artifact.to_bytes()?));
                artifacts.push(artifact.clone());
            }
            Err(source) => return Err(InterfaceArtifactError::Io { path, source }),
        }
    }
    Ok(PreparedInterfaceCache {
        artifacts,
        writes,
        report,
    })
}

/// Keep different source-package roots from contending for a logical `root`
/// module under the shared `target/<profile>` directory. The package identity
/// affects only the cache path, never artifact contents or generated output.
pub fn cache_directory(profile_root: &Path, package_identity: &str) -> PathBuf {
    profile_root
        .join("interface")
        .join(InterfaceHash::of_bytes(package_identity.as_bytes()).hex())
}

impl ModuleInterfaceArtifact {
    pub fn to_bytes(&self) -> Result<Vec<u8>, InterfaceArtifactError> {
        self.validate()?;
        let mut out = Vec::new();
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&SCHEMA_VERSION.to_le_bytes());
        push_string(&mut out, &self.module);
        push_string(&mut out, &self.compiler_version);
        push_string(&mut out, &self.language_version);
        push_string(&mut out, &self.package_config);
        out.extend_from_slice(&self.source_hash.bytes());
        out.extend_from_slice(&self.interface_hash.bytes());
        out.extend_from_slice(&self.cache_key.bytes());
        push_count(&mut out, self.dependencies.len());
        for (module, hash) in &self.dependencies {
            push_string(&mut out, module);
            out.extend_from_slice(&hash.bytes());
        }
        push_count(&mut out, self.callables.len());
        for (symbol, contract) in &self.callables {
            push_string(&mut out, symbol);
            push_callable_contract(&mut out, contract)?;
        }
        Ok(out)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, InterfaceArtifactError> {
        let mut reader = ArtifactReader::new(bytes);
        if reader.fixed::<4>()? != *MAGIC {
            return Err(InterfaceArtifactError::BadMagic);
        }
        let version = reader.u32()?;
        if version != SCHEMA_VERSION {
            return Err(InterfaceArtifactError::UnsupportedSchema(version));
        }
        let module = reader.string()?;
        let compiler_version = reader.string()?;
        let language_version = reader.string()?;
        let package_config = reader.string()?;
        let source_hash = InterfaceHash(reader.fixed()?);
        let interface_hash = InterfaceHash(reader.fixed()?);
        let cache_key = InterfaceHash(reader.fixed()?);
        let dependency_count = reader.count()?;
        let mut dependencies = BTreeMap::new();
        for _ in 0..dependency_count {
            let dependency = reader.string()?;
            let hash = InterfaceHash(reader.fixed()?);
            if dependencies.insert(dependency, hash).is_some() {
                return Err(InterfaceArtifactError::NonCanonical);
            }
        }
        let callable_count = reader.count()?;
        let mut callables = BTreeMap::new();
        for _ in 0..callable_count {
            let symbol = reader.string()?;
            let contract = read_callable_contract(&mut reader)?;
            if callables.insert(symbol, contract).is_some() {
                return Err(InterfaceArtifactError::NonCanonical);
            }
        }
        if !reader.is_finished() {
            return Err(InterfaceArtifactError::TrailingBytes);
        }
        let artifact = Self {
            module,
            compiler_version,
            language_version,
            package_config,
            source_hash,
            dependencies,
            callables,
            interface_hash,
            cache_key,
        };
        artifact.validate()?;
        if artifact.to_bytes()? != bytes {
            return Err(InterfaceArtifactError::NonCanonical);
        }
        Ok(artifact)
    }

    fn validate(&self) -> Result<(), InterfaceArtifactError> {
        for (symbol, contract) in &self.callables {
            contract.validate().map_err(|reason| {
                InterfaceArtifactError::InvalidCallableContract {
                    module: self.module.clone(),
                    symbol: symbol.clone(),
                    reason,
                }
            })?;
        }
        let expected = interface_hash(&self.module, &self.callables)?;
        if expected != self.interface_hash {
            return Err(InterfaceArtifactError::StaleInterfaceHash {
                module: self.module.clone(),
            });
        }
        let expected = cache_key(
            self.source_hash,
            &self.compiler_version,
            &self.language_version,
            &self.package_config,
            &self.dependencies,
        );
        if expected != self.cache_key {
            return Err(InterfaceArtifactError::StaleCacheKey {
                module: self.module.clone(),
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct PreliminaryArtifact {
    source_hash: InterfaceHash,
    interface_hash: InterfaceHash,
    callables: BTreeMap<String, CallableInterfaceContract>,
}

fn collect_transitive_dependencies(
    owner: &str,
    direct_dependencies: &[String],
    inputs: &BTreeMap<&str, &ModuleInterfaceInput>,
    out: &mut BTreeSet<String>,
) -> Result<(), InterfaceArtifactError> {
    let mut pending: Vec<String> = direct_dependencies.to_vec();
    while let Some(module) = pending.pop() {
        if module == owner || !out.insert(module.clone()) {
            continue;
        }
        let Some(input) = inputs.get(module.as_str()) else {
            return Err(InterfaceArtifactError::UnknownDependency {
                owner: owner.to_string(),
                module,
            });
        };
        pending.extend(input.direct_dependencies.iter().cloned());
    }
    Ok(())
}

fn interface_hash(
    module: &str,
    callables: &BTreeMap<String, CallableInterfaceContract>,
) -> Result<InterfaceHash, InterfaceArtifactError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"ember-interface-v3:callable-declarations-and-regions\0");
    hash_string(&mut hasher, module);
    hash_count(&mut hasher, callables.len());
    for (symbol, contract) in callables {
        hash_string(&mut hasher, symbol);
        let mut bytes = Vec::new();
        push_callable_contract(&mut bytes, contract)?;
        hash_bytes(&mut hasher, &bytes);
    }
    Ok(InterfaceHash(*hasher.finalize().as_bytes()))
}

fn cache_key(
    source_hash: InterfaceHash,
    compiler_version: &str,
    language_version: &str,
    package_config: &str,
    dependencies: &BTreeMap<String, InterfaceHash>,
) -> InterfaceHash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"ember-cache-key-v1\0");
    hasher.update(&source_hash.bytes());
    hash_string(&mut hasher, compiler_version);
    hash_string(&mut hasher, language_version);
    hash_string(&mut hasher, package_config);
    hash_count(&mut hasher, dependencies.len());
    for (module, hash) in dependencies {
        hash_string(&mut hasher, module);
        hasher.update(&hash.bytes());
    }
    InterfaceHash(*hasher.finalize().as_bytes())
}

fn artifact_path(directory: &Path, module: &str) -> PathBuf {
    let identity = InterfaceHash::of_bytes(module.as_bytes()).hex();
    directory.join(format!("{identity}.{EXTENSION}"))
}

fn hash_count(hasher: &mut blake3::Hasher, value: usize) {
    hasher.update(&(value as u64).to_le_bytes());
}

fn hash_string(hasher: &mut blake3::Hasher, value: &str) {
    hash_bytes(hasher, value.as_bytes());
}

fn hash_bytes(hasher: &mut blake3::Hasher, value: &[u8]) {
    hasher.update(&(value.len() as u64).to_le_bytes());
    hasher.update(value);
}

fn push_count(out: &mut Vec<u8>, value: usize) {
    out.extend_from_slice(&(value as u64).to_le_bytes());
}

fn push_string(out: &mut Vec<u8>, value: &str) {
    push_bytes(out, value.as_bytes());
}

fn push_bytes(out: &mut Vec<u8>, value: &[u8]) {
    push_count(out, value.len());
    out.extend_from_slice(value);
}

fn push_callable_signature(out: &mut Vec<u8>, signature: &CallableSignature) {
    push_count(out, signature.parameters.len());
    for parameter in &signature.parameters {
        out.push(match parameter.mode {
            CallableParameterMode::Borrow => 0,
            CallableParameterMode::Mut => 1,
            CallableParameterMode::Owned => 2,
        });
        push_string(out, &parameter.ty);
    }
    push_string(out, &signature.result);
    push_count(out, signature.generics.len());
    for generic in &signature.generics {
        push_count(out, generic.bounds.len());
        for bound in &generic.bounds {
            push_string(out, bound);
        }
        match &generic.callable {
            Some(callable) => {
                out.push(1);
                push_count(out, callable.parameters.len());
                for parameter in &callable.parameters {
                    out.push(match parameter.mode {
                        CallableParameterMode::Borrow => 0,
                        CallableParameterMode::Mut => 1,
                        CallableParameterMode::Owned => 2,
                    });
                    push_string(out, &parameter.ty);
                }
                push_string(out, &callable.result);
                out.push(u8::from(callable.once));
                out.push(u8::from(callable.latebound));
            }
            None => out.push(0),
        }
    }
    out.push(u8::from(signature.is_unsafe));
    match &signature.abi {
        Some(abi) => {
            out.push(1);
            push_string(out, abi);
        }
        None => out.push(0),
    }
    match &signature.borrows {
        Some(positions) => {
            out.push(1);
            push_count(out, positions.len());
            for position in positions {
                push_count(out, *position);
            }
        }
        None => out.push(0),
    }
}

fn push_callable_contract(
    out: &mut Vec<u8>,
    contract: &CallableInterfaceContract,
) -> Result<(), InterfaceArtifactError> {
    push_callable_signature(out, &contract.signature);
    match &contract.metadata {
        Some(metadata) => {
            out.push(1);
            let metadata = metadata
                .to_interface_bytes()
                .map_err(InterfaceArtifactError::Metadata)?;
            push_bytes(out, &metadata);
        }
        None => out.push(0),
    }
    Ok(())
}

fn read_callable_signature(
    reader: &mut ArtifactReader<'_>,
) -> Result<CallableSignature, InterfaceArtifactError> {
    let parameter_count = reader.count()?;
    let mut parameters = Vec::with_capacity(parameter_count);
    for _ in 0..parameter_count {
        let mode = match reader.u8()? {
            0 => CallableParameterMode::Borrow,
            1 => CallableParameterMode::Mut,
            2 => CallableParameterMode::Owned,
            _ => return Err(InterfaceArtifactError::NonCanonical),
        };
        parameters.push(CallableParameter { mode, ty: reader.string()? });
    }
    let result = reader.string()?;
    let generic_count = reader.count()?;
    let mut generics = Vec::with_capacity(generic_count);
    for _ in 0..generic_count {
        let bound_count = reader.count()?;
        let mut bounds = Vec::with_capacity(bound_count);
        for _ in 0..bound_count {
            bounds.push(reader.string()?);
        }
        let callable = match reader.u8()? {
            0 => None,
            1 => {
                let parameter_count = reader.count()?;
                let mut parameters = Vec::with_capacity(parameter_count);
                for _ in 0..parameter_count {
                    let mode = match reader.u8()? {
                        0 => CallableParameterMode::Borrow,
                        1 => CallableParameterMode::Mut,
                        2 => CallableParameterMode::Owned,
                        _ => return Err(InterfaceArtifactError::NonCanonical),
                    };
                    parameters.push(CallableParameter { mode, ty: reader.string()? });
                }
                let result = reader.string()?;
                let once = match reader.u8()? {
                    0 => false,
                    1 => true,
                    _ => return Err(InterfaceArtifactError::NonCanonical),
                };
                let latebound = match reader.u8()? {
                    0 => false,
                    1 => true,
                    _ => return Err(InterfaceArtifactError::NonCanonical),
                };
                Some(CallableGenericCallableBound {
                    parameters,
                    result,
                    once,
                    latebound,
                })
            }
            _ => return Err(InterfaceArtifactError::NonCanonical),
        };
        generics.push(CallableGenericParameter { bounds, callable });
    }
    let is_unsafe = match reader.u8()? {
        0 => false,
        1 => true,
        _ => return Err(InterfaceArtifactError::NonCanonical),
    };
    let abi = match reader.u8()? {
        0 => None,
        1 => Some(reader.string()?),
        _ => return Err(InterfaceArtifactError::NonCanonical),
    };
    let borrows = match reader.u8()? {
        0 => None,
        1 => {
            let count = reader.count()?;
            let mut positions = Vec::with_capacity(count);
            for _ in 0..count {
                positions.push(reader.count()?);
            }
            Some(positions)
        }
        _ => return Err(InterfaceArtifactError::NonCanonical),
    };
    let signature = CallableSignature {
        parameters,
        result,
        generics,
        borrows,
        is_unsafe,
        abi,
    };
    signature
        .validate()
        .map_err(|_| InterfaceArtifactError::NonCanonical)?;
    Ok(signature)
}

fn read_callable_contract(
    reader: &mut ArtifactReader<'_>,
) -> Result<CallableInterfaceContract, InterfaceArtifactError> {
    let signature = read_callable_signature(reader)?;
    let metadata = match reader.u8()? {
        0 => None,
        1 => {
            let bytes = reader.bytes()?;
            Some(
                CallableRegionMetadata::from_interface_bytes(bytes)
                    .map_err(InterfaceArtifactError::Metadata)?,
            )
        }
        _ => return Err(InterfaceArtifactError::NonCanonical),
    };
    let contract = CallableInterfaceContract { signature, metadata };
    contract
        .validate()
        .map_err(|_| InterfaceArtifactError::NonCanonical)?;
    Ok(contract)
}

struct ArtifactReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> ArtifactReader<'a> {
    const MAX_COUNT: usize = 1_000_000;

    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn is_finished(&self) -> bool {
        self.position == self.bytes.len()
    }

    fn fixed<const N: usize>(&mut self) -> Result<[u8; N], InterfaceArtifactError> {
        let end = self
            .position
            .checked_add(N)
            .ok_or(InterfaceArtifactError::LengthOverflow)?;
        let bytes = self
            .bytes
            .get(self.position..end)
            .ok_or(InterfaceArtifactError::UnexpectedEof)?;
        self.position = end;
        bytes
            .try_into()
            .map_err(|_| InterfaceArtifactError::UnexpectedEof)
    }

    fn u32(&mut self) -> Result<u32, InterfaceArtifactError> {
        Ok(u32::from_le_bytes(self.fixed()?))
    }

    fn u8(&mut self) -> Result<u8, InterfaceArtifactError> {
        Ok(self.fixed::<1>()?[0])
    }

    fn u64(&mut self) -> Result<u64, InterfaceArtifactError> {
        Ok(u64::from_le_bytes(self.fixed()?))
    }

    fn count(&mut self) -> Result<usize, InterfaceArtifactError> {
        let count =
            usize::try_from(self.u64()?).map_err(|_| InterfaceArtifactError::LengthOverflow)?;
        if count > Self::MAX_COUNT {
            return Err(InterfaceArtifactError::LengthOverflow);
        }
        Ok(count)
    }

    fn bytes(&mut self) -> Result<&'a [u8], InterfaceArtifactError> {
        let count = self.count()?;
        let end = self
            .position
            .checked_add(count)
            .ok_or(InterfaceArtifactError::LengthOverflow)?;
        let bytes = self
            .bytes
            .get(self.position..end)
            .ok_or(InterfaceArtifactError::UnexpectedEof)?;
        self.position = end;
        Ok(bytes)
    }

    fn string(&mut self) -> Result<String, InterfaceArtifactError> {
        let bytes = self.bytes()?;
        String::from_utf8(bytes.to_vec()).map_err(|_| InterfaceArtifactError::InvalidUtf8)
    }
}

#[derive(Debug)]
pub enum InterfaceArtifactError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    DuplicateModule(String),
    UnknownDependency {
        owner: String,
        module: String,
    },
    DuplicateCallable {
        module: String,
        symbol: String,
    },
    InvalidCallableContract {
        module: String,
        symbol: String,
        reason: &'static str,
    },
    Metadata(CallableRegionMetadataCodecError),
    BadMagic,
    UnsupportedSchema(u32),
    UnexpectedEof,
    LengthOverflow,
    InvalidUtf8,
    TrailingBytes,
    NonCanonical,
    StaleInterfaceHash {
        module: String,
    },
    StaleCacheKey {
        module: String,
    },
    StaleSummary {
        module: String,
    },
    ModulePathCollision {
        path: PathBuf,
        expected: String,
        found: String,
    },
    StoredArtifact {
        path: PathBuf,
        error: Box<InterfaceArtifactError>,
    },
}

impl std::fmt::Display for InterfaceArtifactError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::DuplicateModule(module) => write!(f, "duplicate interface module `{module}`"),
            Self::UnknownDependency { owner, module } => {
                write!(
                    f,
                    "interface module `{owner}` depends on unloaded module `{module}`"
                )
            }
            Self::DuplicateCallable { module, symbol } => {
                write!(
                    f,
                    "interface module `{module}` records callable `{symbol}` twice"
                )
            }
            Self::InvalidCallableContract {
                module,
                symbol,
                reason,
            } => write!(
                f,
                "interface module `{module}` records invalid callable contract for `{symbol}`: {reason}"
            ),
            Self::Metadata(error) => write!(f, "callable-region artifact: {error}"),
            Self::BadMagic => write!(f, "not an Ember module-interface artifact"),
            Self::UnsupportedSchema(version) => {
                write!(f, "unsupported Ember module-interface schema {version}")
            }
            Self::UnexpectedEof => write!(f, "truncated module-interface artifact"),
            Self::LengthOverflow => write!(f, "module-interface artifact has an invalid length"),
            Self::InvalidUtf8 => write!(f, "module-interface artifact contains invalid UTF-8"),
            Self::TrailingBytes => write!(f, "module-interface artifact has trailing bytes"),
            Self::NonCanonical => write!(f, "module-interface artifact is not canonical"),
            Self::StaleInterfaceHash { module } => {
                write!(
                    f,
                    "module-interface hash is stale or corrupt for `{module}`"
                )
            }
            Self::StaleCacheKey { module } => {
                write!(
                    f,
                    "module-interface cache key is stale or corrupt for `{module}`"
                )
            }
            Self::StaleSummary { module } => write!(
                f,
                "cached callable-region metadata disagrees with the current verified summary for `{module}`"
            ),
            Self::ModulePathCollision {
                path,
                expected,
                found,
            } => write!(
                f,
                "{} stores interface `{found}`, expected `{expected}`",
                path.display()
            ),
            Self::StoredArtifact { path, error } => {
                write!(
                    f,
                    "{}: invalid stored module interface: {error}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for InterfaceArtifactError {}

#[cfg(test)]
mod tests {
    use super::*;
    use ember_mir::{CallableAccessSummary, ParameterFieldAccess, Projection, RegionAccessKind};

    fn metadata(field: usize) -> CallableRegionMetadata {
        CallableRegionMetadata::new(
            CallableAccessSummary::Fields(vec![ParameterFieldAccess {
                argument: 0,
                projection: vec![Projection::Field(field)],
                operations: vec![RegionAccessKind::Read],
            }]),
            None,
        )
    }

    fn signature(parameter_ty: &str) -> CallableSignature {
        CallableSignature {
            parameters: vec![CallableParameter {
                mode: CallableParameterMode::Borrow,
                ty: parameter_ty.to_string(),
            }],
            result: "Span[i32]".to_string(),
            generics: vec![CallableGenericParameter {
                bounds: Vec::new(),
                callable: Some(CallableGenericCallableBound {
                    parameters: vec![CallableParameter {
                        mode: CallableParameterMode::Borrow,
                        ty: "Span[i32]".to_string(),
                    }],
                    result: "i32".to_string(),
                    once: false,
                    latebound: true,
                }),
            }],
            borrows: Some(vec![0]),
            is_unsafe: false,
            abi: None,
        }
    }

    fn input(
        module: &str,
        source: &str,
        dependencies: &[&str],
        field: usize,
    ) -> ModuleInterfaceInput {
        ModuleInterfaceInput {
            module: module.to_string(),
            source: source.to_string(),
            language_version: "0.9.7".to_string(),
            package_config: "profile=debug".to_string(),
            direct_dependencies: dependencies
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            callables: vec![CallableInterfaceRecord {
                symbol: format!("em_{module}_f"),
                contract: CallableInterfaceContract {
                    signature: signature("Span[i32]"),
                    metadata: Some(metadata(field)),
                },
            }],
        }
    }

    #[test]
    fn callable_metadata_round_trips_through_a_module_artifact() {
        let artifacts = build_artifacts(&[input("root", "root", &[], 0)], "test").unwrap();
        let decoded = round_trip_artifacts(&artifacts).unwrap();
        assert_eq!(decoded, artifacts);
        let callable = decoded[0].callables.values().next().unwrap();
        assert!(callable.signature.generics[0].callable.as_ref().unwrap().latebound);
    }

    #[test]
    fn changing_a_dependency_summary_invalidates_the_callers_cache_key() {
        let before = build_artifacts(
            &[
                input("root", "root", &["dep"], 0),
                input("dep", "dep", &[], 0),
            ],
            "test",
        )
        .unwrap();
        let after = build_artifacts(
            &[
                input("root", "root", &["dep"], 0),
                input("dep", "dep", &[], 1),
            ],
            "test",
        )
        .unwrap();
        let before_root = before
            .iter()
            .find(|artifact| artifact.module == "root")
            .unwrap();
        let after_root = after
            .iter()
            .find(|artifact| artifact.module == "root")
            .unwrap();
        assert_ne!(before_root.cache_key, after_root.cache_key);
    }

    #[test]
    fn changing_a_dependency_signature_invalidates_the_callers_cache_key() {
        let before = build_artifacts(
            &[
                input("root", "root", &["dep"], 0),
                input("dep", "dep", &[], 0),
            ],
            "test",
        )
        .unwrap();
        let mut changed_dependency = input("dep", "dep", &[], 0);
        changed_dependency.callables[0].contract.signature.parameters[0].mode =
            CallableParameterMode::Owned;
        let after = build_artifacts(
            &[input("root", "root", &["dep"], 0), changed_dependency],
            "test",
        )
        .unwrap();
        let before_root = before
            .iter()
            .find(|artifact| artifact.module == "root")
            .unwrap();
        let after_root = after
            .iter()
            .find(|artifact| artifact.module == "root")
            .unwrap();
        assert_ne!(before_root.cache_key, after_root.cache_key);
    }

    #[test]
    fn changing_a_callable_boundary_invalidates_the_callers_cache_key() {
        let before = build_artifacts(
            &[
                input("root", "root", &["dep"], 0),
                input("dep", "dep", &[], 0),
            ],
            "test",
        )
        .unwrap();
        let mut changed_dependency = input("dep", "dep", &[], 0);
        changed_dependency.callables[0]
            .contract
            .signature
            .generics[0]
            .callable
            .as_mut()
            .unwrap()
            .latebound = false;
        let after = build_artifacts(
            &[input("root", "root", &["dep"], 0), changed_dependency],
            "test",
        )
        .unwrap();
        let before_root = before
            .iter()
            .find(|artifact| artifact.module == "root")
            .unwrap();
        let after_root = after
            .iter()
            .find(|artifact| artifact.module == "root")
            .unwrap();
        assert_ne!(before_root.cache_key, after_root.cache_key);
    }

    #[test]
    fn unchanged_artifacts_reuse_but_stale_contents_are_a_hard_failure() {
        let test_identity = InterfaceHash::of_bytes(
            std::thread::current()
                .name()
                .unwrap_or("interface-cache-test")
                .as_bytes(),
        );
        let directory = std::env::temp_dir().join(format!(
            "ember-interface-test-{}-{}",
            std::process::id(),
            test_identity.hex()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let fresh = build_artifacts(&[input("root", "root", &[], 0)], "test").unwrap();
        let prepared = prepare_interface_cache(&directory, &fresh).unwrap();
        assert_eq!(prepared.commit().unwrap().created, 1);
        let prepared = prepare_interface_cache(&directory, &fresh).unwrap();
        assert_eq!(prepared.artifacts(), fresh.as_slice());
        assert_eq!(prepared.commit().unwrap().reused, 1);

        let mut stale = fresh.clone();
        stale[0]
            .callables
            .insert(
                ember_branding::mangled("root.f"),
            CallableInterfaceContract {
                signature: signature("Span[i32]"),
                metadata: Some(metadata(1)),
                },
            );
        let error = match prepare_interface_cache(&directory, &stale) {
            Ok(_) => panic!("stale interface metadata was accepted"),
            Err(error) => error,
        };
        assert!(matches!(error, InterfaceArtifactError::StaleSummary { .. }));
        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn an_incompatible_schema_is_invalidated_without_becoming_an_unknown_contract() {
        let test_identity = InterfaceHash::of_bytes(
            std::thread::current()
                .name()
                .unwrap_or("interface-schema-test")
                .as_bytes(),
        );
        let directory = std::env::temp_dir().join(format!(
            "ember-interface-schema-test-{}-{}",
            std::process::id(),
            test_identity.hex()
        ));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        let fresh = build_artifacts(&[input("root", "root", &[], 0)], "test").unwrap();
        let path = artifact_path(&directory, "root");
        let mut incompatible = fresh[0].to_bytes().unwrap();
        incompatible[4..8].copy_from_slice(&(SCHEMA_VERSION - 1).to_le_bytes());
        std::fs::write(&path, incompatible).unwrap();

        let prepared = prepare_interface_cache(&directory, &fresh).unwrap();
        assert_eq!(prepared.artifacts(), fresh.as_slice());
        assert_eq!(prepared.commit().unwrap().invalidated, 1);
        let rewritten = ModuleInterfaceArtifact::from_bytes(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(rewritten, fresh[0]);
        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_stale_cache_key_is_rejected_at_the_artifact_boundary() {
        let artifact = build_artifacts(&[input("root", "root", &[], 0)], "test")
            .unwrap()
            .pop()
            .unwrap();
        let mut bytes = artifact.to_bytes().unwrap();

        // The cache key follows four length-prefixed strings and the two
        // fixed-size source/interface hashes in the canonical encoding.
        let mut offset = 8; // magic + schema
        for _ in 0..4 {
            let length = u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap()) as usize;
            offset += 8 + length;
        }
        offset += 32 + 32;
        bytes[offset] ^= 1;

        assert!(matches!(
            ModuleInterfaceArtifact::from_bytes(&bytes),
            Err(InterfaceArtifactError::StaleCacheKey { module }) if module == "root"
        ));
    }

    #[test]
    fn an_empty_borrows_contract_is_rejected_at_the_artifact_boundary() {
        let mut invalid = input("root", "root", &[], 0);
        invalid.callables[0].contract.signature.borrows = Some(Vec::new());

        assert!(matches!(
            build_artifacts(&[invalid], "test"),
            Err(InterfaceArtifactError::InvalidCallableContract { reason, .. })
                if reason == "`@borrows` must name at least one parameter"
        ));
    }
}
