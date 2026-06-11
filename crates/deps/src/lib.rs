mod project_manifest;
mod typedef_locator;

use std::path::{Path, PathBuf};

use stdlib::Target;

pub use project_manifest::{
    GoDependency, Manifest, ResolveReport, TrimmedVia, check_no_subpackage_deps,
    check_toolchain_version, parse_manifest, remove_go_dep, resolve_empty_via,
    trim_dead_via_parents, upsert_go_dep, validate_project_name,
};
pub use typedef_locator::{
    Bindgen, BindgenFailure, BindgenGuard, BindgenSession, BindgenSetup, DeclarationStatus,
    TypedefLocator, TypedefLocatorResult, TypedefOrigin,
};

pub fn is_third_party(pkg: &str) -> bool {
    pkg.split('/')
        .next()
        .is_some_and(|first| first.contains('.'))
}

pub fn is_stdlib(pkg: &str) -> bool {
    !is_third_party(pkg)
}

pub fn typedef_cache_dir(project_root: &Path) -> PathBuf {
    let lis_version = env!("CARGO_PKG_VERSION");
    project_root
        .join("target/.lisette/typedefs")
        .join(format!("lis@v{}", lis_version))
}

/// Directory holding materialized Go stdlib typedefs. Global (stdlib is the same
/// for every project) and keyed by target + stdlib content hash, so a compiler
/// upgrade or stdlib change lands in a fresh directory rather than reusing stale
/// files.
fn stdlib_typedef_dir(target: Target) -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join(".lisette/typedefs/go-std")
            .join(target.cache_segment())
            .join(format!("{:016x}", stdlib::GO_STD_CONTENT_HASH)),
    )
}

/// Materialize an embedded Go stdlib typedef to disk so the editor can open it as
/// a regular file, and return its path. Idempotent: the content is keyed by
/// stdlib hash, so an existing file is always current and is left untouched.
pub fn ensure_stdlib_typedef_on_disk(go_pkg: &str, source: &str, target: Target) -> Option<PathBuf> {
    let path = stdlib_typedef_dir(target)?.join(format!("{go_pkg}.d.lis"));
    if !path.exists() {
        std::fs::create_dir_all(path.parent()?).ok()?;
        atomic_write(&path, source)?;
    }
    Some(path)
}

/// Write `content` to `path` via a temp file + rename so a concurrent reader
/// never observes a partially written typedef.
fn atomic_write(path: &Path, content: &str) -> Option<()> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    std::fs::write(&tmp, content).ok()?;
    if std::fs::rename(&tmp, path).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return None;
    }
    Some(())
}

#[derive(Clone, Copy)]
pub struct GoModule<'a> {
    /// Module path, e.g. `github.com/gorilla/mux`.
    pub path: &'a str,
    /// Module version, e.g. `v1.8.0`.
    pub version: &'a str,
}

/// A Go package within a module.
pub struct GoPackage<'a> {
    /// The module that contains this package.
    pub module: GoModule<'a>,
    /// Package import path, either identical to `module.path` for the root package,
    /// or extended for subpackages (e.g. `github.com/gorilla/mux/middleware`).
    pub package: &'a str,
}

impl GoPackage<'_> {
    /// Build the path to a `.d.lis` file under a base directory.
    ///
    /// ```text
    /// <project>/target/.lisette/typedefs/lis@v0.1.6/darwin_arm64/github.com/gorilla/mux@v1.8.0/mux.d.lis
    /// <project>/target/.lisette/typedefs/lis@v0.1.6/darwin_arm64/github.com/gorilla/mux@v1.8.0/middleware/middleware.d.lis
    /// ```
    pub fn typedef_path(&self, base_dir: &Path, target: Target) -> PathBuf {
        let module_dir = base_dir
            .join(target.cache_segment())
            .join(format!("{}@{}", self.module.path, self.module.version));

        let relative = if self.package == self.module.path {
            ""
        } else {
            self.package
                .strip_prefix(self.module.path)
                .and_then(|s| s.strip_prefix('/'))
                .unwrap_or("")
        };

        let last_segment = self.package.rsplit('/').next().unwrap_or(self.package);

        let filename = format!("{}.d.lis", last_segment);

        if relative.is_empty() {
            module_dir.join(filename)
        } else {
            module_dir.join(relative).join(&filename)
        }
    }
}
