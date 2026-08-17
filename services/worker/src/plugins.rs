//! An honest plugin-execution seam.
//!
//! HONESTY BOUNDARY: this worker does not link a plugin runtime. Real plugin
//! execution lives behind `lawsynth-plugin-api` (the stable, dependency-free
//! protocol and request/response contracts) and `lawsynth-plugin-host` (the
//! isolated host that spawns out-of-process or WASI plugins over a framed
//! `RpcChannel`, enforces a `PermissionPolicy`, and meters `ResourceLimits`).
//! Neither crate is a dependency of this service, and this module deliberately
//! does **not** re-implement or fake any of it.
//!
//! What this module provides is the *seam*: a validated description of the
//! dispatch a worker would make, and a [`PluginDispatch`] trait whose default
//! implementation reports [`WorkerError::Plugin`] "not linked" for every
//! request. That keeps the absence of a plugin runtime explicit and testable --
//! a caller can never mistake a missing host for a successful, silent no-op --
//! while leaving a precise place to wire the real host when this service is
//! built with it.

use crate::WorkerError;

/// The kind of plugin a dispatch targets, mirroring `lawsynth_plugin_api`'s
/// plugin kinds without depending on that crate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PluginKind {
    Algorithm,
    Simulator,
    DataAdapter,
}

impl PluginKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Algorithm => "algorithm",
            Self::Simulator => "simulator",
            Self::DataAdapter => "data-adapter",
        }
    }
}

/// A validated description of a plugin dispatch the worker would make. Building
/// one performs the input validation a host requires up front, so the seam is
/// honest about *what* would be dispatched even though it dispatches nothing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginRequest {
    pub plugin_id: String,
    pub kind: PluginKind,
    pub capabilities: Vec<String>,
}

impl PluginRequest {
    /// Maximum capabilities a single request may declare, matching the spirit of
    /// the host's bounded capability set.
    const MAX_CAPABILITIES: usize = 64;

    pub fn new(
        plugin_id: impl Into<String>,
        kind: PluginKind,
        capabilities: Vec<String>,
    ) -> Result<Self, WorkerError> {
        let plugin_id = plugin_id.into();
        if plugin_id.is_empty()
            || plugin_id.len() > 128
            || !plugin_id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        {
            return Err(WorkerError::Plugin(
                "plugin id must be URL-safe and no longer than 128 bytes".into(),
            ));
        }
        if capabilities.len() > Self::MAX_CAPABILITIES {
            return Err(WorkerError::Plugin(format!(
                "a request may declare at most {} capabilities",
                Self::MAX_CAPABILITIES
            )));
        }
        if capabilities.iter().any(|capability| {
            capability.is_empty() || capability.len() > 128 || capability.contains('\0')
        }) {
            return Err(WorkerError::Plugin(
                "each capability must be a non-empty control-free token of at most 128 bytes"
                    .into(),
            ));
        }
        Ok(Self { plugin_id, kind, capabilities })
    }
}

/// How a worker would hand a validated request to a plugin runtime.
///
/// Implementors are the boundary at which a real `lawsynth-plugin-host` would be
/// wired. The trait is object-safe so a worker could hold a `Box<dyn
/// PluginDispatch>` chosen at construction.
pub trait PluginDispatch {
    /// Whether a plugin runtime is actually linked and reachable.
    fn is_linked(&self) -> bool;

    /// Dispatches a validated request. The default seam never fakes a result: it
    /// returns [`WorkerError::Plugin`] describing that no host is linked.
    fn dispatch(&self, request: &PluginRequest) -> Result<PluginOutcome, WorkerError>;
}

/// The outcome surface of a dispatch. Only honest outcomes are representable:
/// there is deliberately no "faked success" variant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PluginOutcome {
    /// No plugin runtime is linked; the request was validated but not executed.
    NotLinked { detail: String },
}

/// The default dispatch seam for a worker built without a plugin runtime.
#[derive(Clone, Copy, Debug, Default)]
pub struct PluginSeam;

impl PluginSeam {
    pub fn new() -> Self {
        Self
    }

    /// Describes, for operators and docs, the real dispatch path this seam
    /// stands in for.
    pub const fn describe() -> &'static str {
        "plugin execution requires lawsynth-plugin-host (isolated out-of-process or WASI \
         dispatch over a framed RpcChannel with an enforced PermissionPolicy and metered \
         ResourceLimits); it is not linked into this worker build"
    }
}

impl PluginDispatch for PluginSeam {
    fn is_linked(&self) -> bool {
        false
    }

    fn dispatch(&self, request: &PluginRequest) -> Result<PluginOutcome, WorkerError> {
        // The request is already validated by construction; surface an honest,
        // non-faked "unsupported" that names the missing runtime.
        Err(WorkerError::Plugin(format!(
            "cannot dispatch {} plugin '{}': {}",
            request.kind.as_str(),
            request.plugin_id,
            Self::describe()
        )))
    }
}
