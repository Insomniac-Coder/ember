//! Packages and their dependencies (`[MOD-1]`, `[MOD-4]`, `[MAN-2]`, `[VER-5]`).
//!
//! A build's packages are its root and every package its `[dependencies]` reach, each read from its
//! own `ember.toml`. A dependency is `{ path = "…" }` (relative to the manifest naming it),
//! `{ git = "…", rev = "…" }` (checked out into the build cache) or `{ git = "…", version = "1.2" }`
//! (the highest of the repository's `vMAJOR.MINOR.PATCH` tags the requirement admits, ODR-127); a
//! bare version requirement needs a registry, which this compiler does not have (`E0900`).
//!
//! A package is identified by its name and its major version: two majors of one package coexist
//! (`[VER-5]`), each with modules of its own. Module paths are a package's name followed by the
//! module's path under `src/` (`[MOD-1]`); the root package's are written without its name, as
//! they always were. The first major of a name reached keeps the bare name; another is
//! `name__<major>`, so their symbols differ (`em_geometry_area`, `em_geometry__1_area`), and an
//! import inside the package depending on it is rewritten to that path, binding the same name.
//!
//! `ember.lock` (`[MAN-2]`) records every dependency, its version, its source and a hash of its
//! contents (its manifest and every file under `src/`). A build writes it when it would change;
//! `--locked` refuses to; `ember update [package]` rewrites it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// `[VER-5]` — a semantic version, `MAJOR.MINOR.PATCH`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl Version {
    /// `MAJOR.MINOR.PATCH`, each a number; anything else is not a version.
    pub fn parse(text: &str) -> Option<Version> {
        let mut parts = text.split('.').map(|part| {
            (!part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()) && (part == "0" || !part.starts_with('0')))
                .then(|| part.parse::<u64>().ok())
                .flatten()
        });
        let version = Version { major: parts.next()??, minor: parts.next()??, patch: parts.next()?? };
        parts.next().is_none().then_some(version)
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// `[VER-5]` — whether `text` is a version requirement: one to three numbers, `"1.2"`.
fn is_requirement(text: &str) -> bool {
    let parts: Vec<&str> = text.split('.').collect();
    parts.len() <= 3 && parts.iter().all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

/// `[VER-5]` — whether `version` meets the requirement `"1.2"`: `>= 1.2.0, < 2.0.0`.
fn satisfies(version: Version, requirement: &str) -> bool {
    let parts: Vec<u64> = requirement.split('.').filter_map(|part| part.parse().ok()).collect();
    let low = Version { major: parts[0], minor: parts.get(1).copied().unwrap_or(0), patch: parts.get(2).copied().unwrap_or(0) };
    version >= low && version.major == low.major
}

/// ODR-127 — a repository's version tags, `vMAJOR.MINOR.PATCH`, each with the commit it names
/// (an annotated tag's own commit, not the tag object).
fn git_version_tags(url: &str) -> Result<Vec<(Version, String)>, String> {
    let output = std::process::Command::new("git").args(["ls-remote", "--tags", url]).output()
        .map_err(|error| format!("cannot run git for `{url}`: {error}"))?;
    if !output.status.success() {
        return Err(format!("error[E9001]: git could not list the tags of `{url}`: {}", String::from_utf8_lossy(&output.stderr).trim()));
    }
    let mut tags: BTreeMap<Version, String> = BTreeMap::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let Some((commit, reference)) = line.split_once('\t') else { continue };
        let Some(tag) = reference.strip_prefix("refs/tags/v") else { continue };
        let (tag, peeled) = match tag.strip_suffix("^{}") { Some(tag) => (tag, true), None => (tag, false) };
        let Some(version) = Version::parse(tag) else { continue };
        // The peeled line of an annotated tag names its commit; it wins over the tag object's.
        if peeled || !tags.contains_key(&version) {
            tags.insert(version, commit.to_string());
        }
    }
    Ok(tags.into_iter().collect())
}

/// `[MAN-2]` — the commit `ember.lock` records for a version dependency: kept while it still meets
/// the requirement, so a build resolves the same until `ember update`.
fn locked_commit(lock: Option<&str>, name: &str, url: &str, requirement: &str) -> Option<String> {
    let prefix = format!("git+{url}#");
    lock?.split("\n[[package]]\n").skip(1).find_map(|entry| {
        let field = |key: &str| entry.lines().find_map(|line| unquote(line.strip_prefix(&format!("{key} = "))?));
        let commit = field("source")?.strip_prefix(&prefix)?.to_string();
        (field("name")? == name && satisfies(Version::parse(&field("version")?)?, requirement)).then_some(commit)
    })
}

/// Where a package came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    Root,
    /// The directory, as the manifest naming it wrote it.
    Path(String),
    Git { url: String, rev: String },
}

impl Source {
    fn lock_text(&self) -> String {
        match self {
            Source::Root => "root".to_string(),
            Source::Path(path) => format!("path+{path}"),
            Source::Git { url, rev } => format!("git+{url}#{rev}"),
        }
    }
}

pub struct Package {
    pub name: String,
    pub version: Version,
    pub dir: PathBuf,
    pub src: PathBuf,
    /// The module path prefix: none for the root package, its namespace for a dependency.
    pub prefix: Vec<String>,
    /// Each dependency, by the name this package imports it with.
    pub deps: BTreeMap<String, usize>,
    pub source: Source,
    /// `[STD-6]` — `[package] layers`, when written: the `std` layers the package may use.
    pub layers: Option<Vec<String>>,
}

/// `[STD-6]` — the layers, in order: each depends only on the ones before it.
pub const LAYER_ORDER: [&str; 5] = ["core", "alloc", "sync", "io", "ffi"];

impl Package {
    /// Whether the package may use `layer`: every layer when it declares none.
    pub fn uses(&self, layer: &str) -> bool {
        self.layers.as_ref().is_none_or(|layers| layers.iter().any(|written| written == layer))
    }
}

/// `[STD-6]` — `layers = ["core", "alloc"]`: known layers, `core` among them, and each with the
/// ones before it (it depends on them).
fn parse_layers(value: &str, file: &Path) -> Result<Vec<String>, String> {
    let items = value.trim().strip_prefix('[').and_then(|rest| rest.strip_suffix(']'))
        .ok_or_else(|| format!("error[E9001]: `[package] layers` in {} is a list such as [\"core\", \"alloc\"]", file.display()))?;
    let layers: Vec<String> = items.split(',').map(|item| item.trim().trim_matches('"').to_string()).filter(|item| !item.is_empty()).collect();
    for layer in &layers {
        let Some(at) = LAYER_ORDER.iter().position(|known| known == layer) else {
            return Err(format!("error[E9001]: `{layer}` is not a layer of `std` ({}); the layers are {}", file.display(), LAYER_ORDER.join(", ")));
        };
        if let Some(missing) = LAYER_ORDER[..at].iter().find(|before| !layers.iter().any(|written| written == *before)) {
            return Err(format!("error[E9001]: the `{layer}` layer depends on `{missing}`, which `[package] layers` in {} leaves out [STD-6]", file.display()));
        }
    }
    if layers.is_empty() {
        return Err(format!("error[E9001]: `[package] layers` in {} names no layer: it needs at least \"core\"", file.display()));
    }
    Ok(layers)
}

/// The packages of a build, the root first.
pub struct Packages {
    pub list: Vec<Package>,
}

/// The package directory a build's root is in: the directory of the manifest above `root_dir`
/// (the source root, or a single file's directory), if there is one.
pub fn manifest_dir(root_dir: &Path) -> Option<PathBuf> {
    let mut dir = Some(root_dir);
    while let Some(candidate) = dir {
        if candidate.join(ember_branding::MANIFEST).is_file() {
            return Some(candidate.to_path_buf());
        }
        dir = candidate.parent();
    }
    None
}

/// A dependency as a manifest writes it.
#[derive(Clone, Debug)]
enum Spec {
    Path(String),
    Git { url: String, rev: String },
    /// ODR-127 — a requirement met by one of the repository's version tags.
    GitVersion { url: String, requirement: String },
    Version(String),
}

/// What one manifest says about its package and its dependencies.
struct Manifest {
    name: Option<String>,
    version: Option<String>,
    layers: Option<Vec<String>>,
    deps: Vec<(String, Spec)>,
}

/// A quoted string's text.
fn unquote(value: &str) -> Option<String> {
    let value = value.trim();
    value.strip_prefix('"').and_then(|value| value.strip_suffix('"')).map(str::to_string)
}

/// The `key = "value"` pairs of an inline table `{ path = "…" }`.
fn inline_table(value: &str) -> Option<Vec<(String, String)>> {
    let inner = value.trim().strip_prefix('{')?.strip_suffix('}')?;
    let mut pairs = Vec::new();
    for pair in inner.split(',').map(str::trim).filter(|pair| !pair.is_empty()) {
        let (key, value) = pair.split_once('=')?;
        pairs.push((key.trim().to_string(), unquote(value)?));
    }
    Some(pairs)
}

fn spec_from_pairs(name: &str, pairs: &[(String, String)], file: &Path) -> Result<Spec, String> {
    let get = |key: &str| pairs.iter().find(|(written, _)| written == key).map(|(_, value)| value.clone());
    for (key, _) in pairs {
        if !matches!(key.as_str(), "path" | "git" | "rev" | "version") {
            return Err(format!("error[E9001]: `{key}` is not a key of a dependency (`{name}` in {})\n  = help: a dependency is `{{ path = \"…\" }}`, `{{ git = \"…\", rev = \"…\" }}` or `{{ git = \"…\", version = \"…\" }}`", file.display()));
        }
    }
    match (get("path"), get("git"), get("rev"), get("version")) {
        (Some(path), None, None, None) => Ok(Spec::Path(path)),
        (None, Some(url), Some(rev), None) => Ok(Spec::Git { url, rev }),
        (None, Some(url), None, Some(requirement)) if is_requirement(&requirement) => Ok(Spec::GitVersion { url, requirement }),
        (None, Some(_), None, Some(requirement)) => Err(format!(
            "error[E9001]: dependency `{name}`'s `version = \"{requirement}\"` is not a version requirement such as \"1.2\" [VER-5]")),
        _ => Err(format!("error[E9001]: dependency `{name}` in {} is neither `{{ path }}`, `{{ git, rev }}` nor `{{ git, version }}`", file.display())),
    }
}

fn read_manifest(file: &Path) -> Result<Manifest, String> {
    let text = std::fs::read_to_string(file).map_err(|error| format!("cannot read `{}`: {error}", file.display()))?;
    let mut manifest = Manifest { name: None, version: None, layers: None, deps: Vec::new() };
    let mut section = String::new();
    let mut table_dep: Option<(String, Vec<(String, String)>)> = None;
    let finish_table = |table: &mut Option<(String, Vec<(String, String)>)>, deps: &mut Vec<(String, Spec)>| -> Result<(), String> {
        if let Some((name, pairs)) = table.take() {
            let spec = spec_from_pairs(&name, &pairs, file)?;
            deps.push((name, spec));
        }
        Ok(())
    };
    for line in text.lines() {
        let line = line.split_once('#').map_or(line, |(before, _)| before).trim();
        if let Some(name) = line.strip_prefix('[').and_then(|line| line.strip_suffix(']')) {
            finish_table(&mut table_dep, &mut manifest.deps)?;
            section = name.trim().to_string();
            if let Some(dep) = section.strip_prefix("dependencies.") {
                table_dep = Some((dep.to_string(), Vec::new()));
            }
            continue;
        }
        let Some((key, value)) = line.split_once('=') else { continue };
        let key = key.trim().trim_matches('"').to_string();
        match section.as_str() {
            "package" if key == "name" => manifest.name = unquote(value),
            "package" if key == "version" => manifest.version = unquote(value),
            "package" if key == "layers" => manifest.layers = Some(parse_layers(value, file)?),
            "dependencies" => {
                // `[MOD-1]` — `std` is implicit, and may be pinned; this compiler is its version.
                if key == ember_branding::STD_PACKAGE {
                    continue;
                }
                let spec = match (unquote(value), inline_table(value)) {
                    (Some(requirement), _) => Spec::Version(requirement),
                    (None, Some(pairs)) => spec_from_pairs(&key, &pairs, file)?,
                    (None, None) => {
                        return Err(format!("error[E9001]: dependency `{key}` in {} is not a version requirement, `{{ path }}`, `{{ git, rev }}` or `{{ git, version }}`", file.display()));
                    }
                };
                manifest.deps.push((key, spec));
            }
            _ if table_dep.is_some() => {
                if let (Some((_, pairs)), Some(value)) = (&mut table_dep, unquote(value)) {
                    pairs.push((key, value));
                }
            }
            _ => {}
        }
    }
    finish_table(&mut table_dep, &mut manifest.deps)?;
    Ok(manifest)
}

/// The directory a git dependency is checked out in, at its revision, under the build cache:
/// cloned once, then checked out per revision.
fn git_checkout(url: &str, rev: &str) -> Result<PathBuf, String> {
    let key = ember_build::content_hash(url.as_bytes());
    let root = ember_build::cache_root().join("git");
    let checkout = root.join(format!("{}-{}", &key[..16], rev));
    if checkout.join(ember_branding::MANIFEST).is_file() {
        return Ok(checkout);
    }
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let staging = root.join(format!("{}-{}.tmp{}", &key[..16], rev, std::process::id()));
    let _ = std::fs::remove_dir_all(&staging);
    let git = |args: &[&str], dir: Option<&Path>| -> Result<(), String> {
        let mut command = std::process::Command::new("git");
        command.args(args);
        if let Some(dir) = dir {
            command.current_dir(dir);
        }
        let output = command.output().map_err(|error| format!("cannot run git for `{url}`: {error}"))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(format!("error[E9001]: git could not fetch `{url}` at `{rev}`: {}", String::from_utf8_lossy(&output.stderr).trim()))
        }
    };
    let staging_text = staging.to_string_lossy().into_owned();
    git(&["clone", "--quiet", "--no-checkout", url, &staging_text], None)?;
    git(&["checkout", "--quiet", "--detach", rev], Some(&staging))?;
    let _ = std::fs::remove_dir_all(staging.join(".git"));
    if std::fs::rename(&staging, &checkout).is_err() {
        let _ = std::fs::remove_dir_all(&staging);
        if !checkout.is_dir() {
            return Err(format!("cannot place the checkout of `{url}` in {}", checkout.display()));
        }
    }
    Ok(checkout)
}

impl Packages {
    /// The root package and nothing else: a single file, or a build whose dependencies could not
    /// be resolved (which that build reports).
    pub fn alone(root_dir: &Path) -> Packages {
        Packages { list: vec![Package {
            name: String::new(), version: Version { major: 0, minor: 0, patch: 0 }, dir: root_dir.to_path_buf(),
            src: root_dir.to_path_buf(), prefix: Vec::new(), deps: BTreeMap::new(), source: Source::Root, layers: None,
        }] }
    }
}

/// Every package the root package's dependencies reach. `root_dir` is the build's source root;
/// with no manifest above it the root has no dependencies.
pub fn resolve(root_dir: &Path) -> Result<Packages, String> {
    resolve_afresh(root_dir, None)
}

/// `resolve`, with `fresh` naming the version dependency (`*`: every one) whose `ember.lock` entry is
/// set aside, so its requirement is met anew from its tags (`ember update`).
fn resolve_afresh(root_dir: &Path, fresh: Option<&str>) -> Result<Packages, String> {
    let Some(root) = manifest_dir(root_dir) else {
        return Ok(Packages::alone(root_dir));
    };
    let manifest = read_manifest(&root.join(ember_branding::MANIFEST))?;
    let lock = std::fs::read_to_string(root.join(ember_branding::LOCK_FILE)).ok().map(|text| text.replace("\r\n", "\n"));
    let mut packages = Packages { list: vec![Package {
        name: manifest.name.clone().unwrap_or_default(),
        version: manifest.version.as_deref().and_then(Version::parse).unwrap_or(Version { major: 0, minor: 0, patch: 0 }),
        dir: root.clone(),
        src: root_dir.to_path_buf(),
        prefix: Vec::new(),
        deps: BTreeMap::new(),
        source: Source::Root,
        layers: manifest.layers.clone(),
    }] };
    let mut pending = vec![(0usize, manifest.deps, root.clone())];
    while let Some((from, deps, from_dir)) = pending.pop() {
        for (name, spec) in deps {
            let mut wanted: Option<String> = None;
            let (dir, source) = match spec {
                Spec::Path(path) => {
                    let dir = from_dir.join(&path);
                    let dir = std::fs::canonicalize(&dir).map_err(|_| format!(
                        "error[E9001]: dependency `{name}`'s path `{path}` is not a directory (from {})", from_dir.join(ember_branding::MANIFEST).display()))?;
                    (dunce(dir), Source::Path(path))
                }
                Spec::Git { url, rev } => (git_checkout(&url, &rev)?, Source::Git { url, rev }),
                // ODR-127 — the highest version tag the requirement admits, or the commit the lock
                // keeps for it until `ember update`.
                Spec::GitVersion { url, requirement } => {
                    let afresh = fresh.is_some_and(|fresh| fresh == "*" || fresh == name);
                    let pinned = if afresh { None } else { locked_commit(lock.as_deref(), &name, &url, &requirement) };
                    let commit = match pinned {
                        Some(commit) => commit,
                        None => {
                            let tags = git_version_tags(&url)?;
                            let Some((_, commit)) = tags.iter().rev().find(|(version, _)| satisfies(*version, &requirement)) else {
                                let listed: Vec<String> = tags.iter().map(|(version, _)| format!("v{version}")).collect();
                                return Err(format!(
                                    "error[E9001]: no version tag of `{url}` meets `{name}`'s `version = \"{requirement}\"` [VER-5]\n  = note: its version tags: {}",
                                    if listed.is_empty() { "none".to_string() } else { listed.join(", ") }));
                            };
                            commit.clone()
                        }
                    };
                    wanted = Some(requirement);
                    (git_checkout(&url, &commit)?, Source::Git { url, rev: commit })
                }
                Spec::Version(requirement) if !is_requirement(&requirement) => {
                    return Err(format!("error[E9001]: dependency `{name} = \"{requirement}\"` is not a version requirement such as \"1.2\" [VER-5]"));
                }
                Spec::Version(requirement) => {
                    return Err(format!(
                        "error[E0900]: dependency `{name} = \"{requirement}\"` needs a package registry, which is not implemented yet\n  = help: name its repository, `{{ git = \"…\", version = \"{requirement}\" }}`, or a directory, `{{ path = \"…\" }}`"));
                }
            };
            let manifest = read_manifest(&dir.join(ember_branding::MANIFEST))?;
            let declared = manifest.name.clone().unwrap_or_default();
            if declared != name {
                return Err(format!("error[E9001]: dependency `{name}` names a package called `{declared}` ({})", dir.display()));
            }
            let version = match manifest.version.as_deref() {
                None => Version { major: 0, minor: 0, patch: 0 },
                Some(text) => Version::parse(text).ok_or_else(|| format!(
                    "error[E9001]: package `{name}`'s version `{text}` is not a semantic version `MAJOR.MINOR.PATCH` [VER-5]"))?,
            };
            // A tag whose package states another version is the repository's mistake, not a choice.
            if let Some(requirement) = &wanted
                && !satisfies(version, requirement)
            {
                return Err(format!("error[E9001]: the tag chosen for `{name}`'s `version = \"{requirement}\"` holds version {version} ({})", dir.display()));
            }
            // One package per name and major: the same directory is one package; two sources of
            // one major are a conflict, never a silent choice.
            let existing = packages.list.iter().position(|package| package.name == name && package.version.major == version.major);
            // The same directory may be written two ways (Windows' short `RUNNER~1` and long
            // `runneradmin`, `a/../b`): compared as the file system resolves them.
            let same = |a: &Path, b: &Path| a == b || matches!((std::fs::canonicalize(a), std::fs::canonicalize(b)), (Ok(a), Ok(b)) if a == b);
            let index = match existing {
                Some(index) if same(&packages.list[index].dir, &dir) => index,
                Some(index) => {
                    return Err(format!("error[E9001]: two sources of package `{name}` {}.x: {} and {}",
                        version.major, packages.list[index].dir.display(), dir.display()));
                }
                None => {
                    let namespace = if packages.list.iter().any(|package| package.name == name) {
                        format!("{name}__{}", version.major)
                    } else {
                        name.clone()
                    };
                    packages.list.push(Package {
                        name: name.clone(), version, dir: dir.clone(), src: dir.join("src"),
                        prefix: vec![namespace], deps: BTreeMap::new(), source, layers: manifest.layers.clone(),
                    });
                    pending.push((packages.list.len() - 1, manifest.deps, dir.clone()));
                    packages.list.len() - 1
                }
            };
            packages.list[from].deps.insert(name, index);
        }
    }
    // `[MOD-1]` — the root's own modules have no package prefix, so one may not share a name with
    // a dependency, whose modules start with it.
    let root_src = &packages.list[0].src;
    for name in packages.list[0].deps.keys() {
        if crate::module_file(root_src, std::slice::from_ref(name)).is_some() || root_src.join(name).is_dir() {
            return Err(format!("error[E9001]: dependency `{name}` has the name of this package's own module `{name}` ({})", root_src.display()));
        }
    }
    // `[BLD-11]` — a package that leaves a layer out cannot depend on a package that uses it.
    for package in &packages.list {
        for (name, &dep) in &package.deps {
            let dependency = &packages.list[dep];
            if let Some(layer) = LAYER_ORDER.iter().find(|layer| !package.uses(layer) && dependency.uses(layer)) {
                let who = if package.name.is_empty() { "this package".to_string() } else { format!("`{}`", package.name) };
                return Err(format!(
                    "error[E1021]: {who} leaves out the `{layer}` layer, which its dependency `{name}` uses: it is not linked in this build
  = note: a package that omits a layer cannot use it or depend on a package that does [BLD-11]
  = help: add \"{layer}\" to `[package] layers`, or depend on a package that declares fewer layers"));
            }
        }
    }
    // `[MOD-4]` — no cycle between packages.
    if let Some(cycle) = package_cycle(&packages) {
        return Err(format!("error[E1041]: the packages depend on each other in a cycle: {}\n  = note: import cycles are allowed inside a package, never between packages [MOD-4]", cycle.join(" -> ")));
    }
    Ok(packages)
}

/// A path without Windows' verbatim prefix, as the rest of the build writes paths.
fn dunce(path: PathBuf) -> PathBuf {
    let text = path.to_string_lossy();
    match text.strip_prefix(r"\\?\") {
        Some(rest) if !rest.starts_with("UNC") => PathBuf::from(rest),
        _ => path,
    }
}

fn package_cycle(packages: &Packages) -> Option<Vec<String>> {
    fn visit(at: usize, packages: &Packages, state: &mut Vec<u8>, stack: &mut Vec<usize>) -> Option<Vec<String>> {
        state[at] = 1;
        stack.push(at);
        for &next in packages.list[at].deps.values() {
            if state[next] == 1 {
                let from = stack.iter().position(|&index| index == next).unwrap_or(0);
                let mut names: Vec<String> = stack[from..].iter().map(|&index| packages.list[index].name.clone()).collect();
                names.push(packages.list[next].name.clone());
                return Some(names);
            }
            if state[next] == 0 && let Some(cycle) = visit(next, packages, state, stack) {
                return Some(cycle);
            }
        }
        stack.pop();
        state[at] = 2;
        None
    }
    let mut state = vec![0u8; packages.list.len()];
    visit(0, packages, &mut state, &mut Vec::new())
}

impl Packages {
    /// `[MOD-1]`, `[MOD-3]` — what an import of `names` inside package `from` names: the package
    /// whose module it is, the module's path in the build, and its file. The package's own
    /// module wins over a dependency of the same name; a path may start with the package's own
    /// name. `None` when neither has it (the standard library is tried after).
    pub fn resolve_import(&self, from: usize, names: &[String]) -> Option<(usize, Vec<String>, PathBuf)> {
        let package = &self.list[from];
        let own = |rest: &[String]| -> Option<(usize, Vec<String>, PathBuf)> {
            let file = if rest.is_empty() { root_module(&package.src)? } else { crate::module_file(&package.src, rest)? };
            let mut path = package.prefix.clone();
            path.extend(rest.iter().cloned());
            Some((from, path, file))
        };
        if let Some(found) = own(names).filter(|_| !names.is_empty()) {
            return Some(found);
        }
        if !package.name.is_empty() && names.first() == Some(&package.name) && (from != 0 || names.len() > 1) {
            if let Some(found) = own(&names[1..]) {
                return Some(found);
            }
        }
        let &dep = package.deps.get(names.first()?)?;
        let target = &self.list[dep];
        let rest = &names[1..];
        let file = if rest.is_empty() { root_module(&target.src)? } else { crate::module_file(&target.src, rest)? };
        let mut path = target.prefix.clone();
        path.extend(rest.iter().cloned());
        Some((dep, path, file))
    }

    /// `[MAN-2]` — the lock file this resolution writes.
    pub fn lock_text(&self) -> Result<String, String> {
        let mut text = String::from("# ember.lock: the resolved dependencies with content hashes [MAN-2].\n# Written by ember; `ember update` rewrites it.\nversion = 1\n");
        let mut entries: Vec<&Package> = self.list.iter().skip(1).collect();
        entries.sort_by(|a, b| (&a.name, a.version).cmp(&(&b.name, b.version)));
        for package in entries {
            let deps: Vec<String> = package.deps.iter()
                .map(|(name, &index)| format!("\"{name} {}\"", self.list[index].version))
                .collect();
            text.push_str(&format!(
                "\n[[package]]\nname = \"{}\"\nversion = \"{}\"\nsource = \"{}\"\nhash = \"blake3:{}\"\ndependencies = [{}]\n",
                package.name, package.version, package.source.lock_text(), content_hash(package)?, deps.join(", ")
            ));
        }
        Ok(text)
    }
}

/// A package's root module: `src/lib.em`, else `src/main.em`.
fn root_module(src: &Path) -> Option<PathBuf> {
    ["lib", "main"].iter().map(|name| src.join(ember_branding::source_file(name))).find(|file| file.is_file())
}

/// `[MAN-2]` — a package's contents: its manifest and every file under `src/`, by relative path.
fn content_hash(package: &Package) -> Result<String, String> {
    let mut files = vec![package.dir.join(ember_branding::MANIFEST)];
    let mut dirs = vec![package.src.clone()];
    while let Some(dir) = dirs.pop() {
        let entries = std::fs::read_dir(&dir).map_err(|error| format!("cannot read `{}`: {error}", dir.display()))?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
            } else {
                files.push(path);
            }
        }
    }
    files.sort();
    let mut bytes = Vec::new();
    for file in files {
        let relative = file.strip_prefix(&package.dir).unwrap_or(&file).to_string_lossy().replace('\\', "/");
        bytes.extend_from_slice(relative.as_bytes());
        bytes.push(0);
        let contents = std::fs::read(&file).map_err(|error| format!("cannot read `{}`: {error}", file.display()))?;
        // The same text whatever a checkout did to its line ends.
        let contents: Vec<u8> = contents.iter().copied().filter(|&byte| byte != b'\r').collect();
        bytes.extend_from_slice(&(contents.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&contents);
    }
    Ok(ember_build::content_hash(&bytes))
}

/// `[MAN-2]` — bring `ember.lock` in line with `packages`: with `locked`, a lock that would change
/// is an error naming it; otherwise it is written. A build without dependencies has no lock.
pub fn settle_lock(packages: &Packages, locked: bool) -> Result<(), String> {
    if packages.list.len() == 1 || packages.list[0].source != Source::Root || manifest_dir(&packages.list[0].dir).is_none() {
        if locked && packages.list.len() > 1 {
            return Err("error: `--locked` needs a package manifest".to_string());
        }
        return Ok(());
    }
    let file = packages.list[0].dir.join(ember_branding::LOCK_FILE);
    let wanted = packages.lock_text()?;
    let current = std::fs::read_to_string(&file).ok().map(|text| text.replace("\r\n", "\n"));
    if current.as_deref() == Some(wanted.as_str()) {
        return Ok(());
    }
    if locked {
        let why = if current.is_none() { "there is none" } else { "the dependencies changed since it was written" };
        return Err(format!("error: `{}` would change ({why}), and `--locked` forbids that\n  = help: run `ember update` to rewrite it, or build without `--locked`", ember_branding::LOCK_FILE));
    }
    std::fs::write(&file, wanted).map_err(|error| format!("cannot write `{}`: {error}", file.display()))?;
    Ok(())
}

/// `[VER-5]`, `[MAN-2]` — `ember update [package]`: resolve again and rewrite `ember.lock`. With a
/// package named, only its entries are taken from the new resolution; every other one stays as
/// the lock had it.
pub fn update(args: &[String]) -> Result<std::process::ExitCode, String> {
    let named: Vec<&String> = args.iter().filter(|arg| !arg.starts_with('-')).collect();
    if named.len() > 1 {
        return Err("`ember update` takes at most one package name".to_string());
    }
    let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
    let dir = manifest_dir(&cwd).ok_or("`ember update` needs a package manifest in this directory or an ancestor")?;
    let packages = resolve_afresh(&dir.join("src"), Some(named.first().map_or("*", |name| name.as_str())))?;
    let file = dir.join(ember_branding::LOCK_FILE);
    let fresh = packages.lock_text()?;
    let text = match named.first() {
        None => fresh,
        Some(name) => {
            if !packages.list.iter().skip(1).any(|package| &&package.name == name) {
                return Err(format!("error[E9003]: no dependency named `{name}`"));
            }
            let old = std::fs::read_to_string(&file).unwrap_or_default().replace("\r\n", "\n");
            merge_lock(&old, &fresh, name)
        }
    };
    std::fs::write(&file, text).map_err(|error| format!("cannot write `{}`: {error}", file.display()))?;
    eprintln!("wrote {}", file.display());
    Ok(std::process::ExitCode::SUCCESS)
}

/// The lock `fresh` with every entry but `name`'s as `old` had it (an entry only `fresh` has is
/// kept; one only `old` has is dropped).
fn merge_lock(old: &str, fresh: &str, name: &str) -> String {
    let entries = |text: &str| -> Vec<String> { text.split("\n[[package]]\n").skip(1).map(str::to_string).collect() };
    let entry_name = |entry: &str| entry.lines().next().and_then(|line| unquote(line.strip_prefix("name = ")?)).unwrap_or_default();
    let head = fresh.split("\n[[package]]\n").next().unwrap_or_default().to_string();
    let old_entries = entries(old);
    let mut text = head;
    for entry in entries(fresh) {
        let entry_name = entry_name(&entry);
        let kept = if entry_name == name {
            None
        } else {
            let version = entry.lines().nth(1).unwrap_or_default().to_string();
            old_entries.iter().find(|old| old.lines().next() == entry.lines().next() && old.lines().nth(1).unwrap_or_default() == version)
        };
        text.push_str("\n[[package]]\n");
        text.push_str(kept.unwrap_or(&entry));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `[VER-5]` — versions are `MAJOR.MINOR.PATCH`, each a number without leading zeros.
    #[test]
    fn versions_are_semantic() {
        assert_eq!(Version::parse("1.2.3"), Some(Version { major: 1, minor: 2, patch: 3 }));
        for bad in ["1.2", "1.2.3.4", "1.x.0", "01.2.3", "", "1..3"] {
            assert_eq!(Version::parse(bad), None, "{bad}");
        }
        assert!(is_requirement("1.2") && is_requirement("1") && !is_requirement("1.x") && !is_requirement("^1.2"));
        let v = |text: &str| Version::parse(text).unwrap();
        assert!(satisfies(v("1.2.0"), "1.2") && satisfies(v("1.9.3"), "1.2") && satisfies(v("1.2.5"), "1.2.5"));
        assert!(!satisfies(v("1.1.9"), "1.2") && !satisfies(v("2.0.0"), "1.2") && !satisfies(v("1.2.4"), "1.2.5"));
        assert!(satisfies(v("2.3.0"), "2") && !satisfies(v("1.9.0"), "2"));
    }

    /// `[MAN-2]` — `ember update geometry` takes geometry's entry from the new resolution and keeps
    /// every other entry as the lock had it.
    #[test]
    fn updating_one_package_keeps_the_others_locked() {
        let old = "version = 1\n\n[[package]]\nname = \"a\"\nversion = \"1.0.0\"\nhash = \"old-a\"\n\n[[package]]\nname = \"b\"\nversion = \"1.0.0\"\nhash = \"old-b\"\n";
        let fresh = "version = 1\n\n[[package]]\nname = \"a\"\nversion = \"1.0.0\"\nhash = \"new-a\"\n\n[[package]]\nname = \"b\"\nversion = \"1.0.0\"\nhash = \"new-b\"\n";
        let merged = merge_lock(old, fresh, "b");
        assert!(merged.contains("hash = \"old-a\"") && merged.contains("hash = \"new-b\""), "{merged}");
    }
}
