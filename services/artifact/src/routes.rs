/// Operations available from the local API. They are intentionally not HTTP routes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalOperation {
    Health,
    Upload,
    Download,
    Metadata,
    CompleteMultipart,
    CollectGarbage,
}

/// Honest capability declaration for adapters deciding whether they can bind a listener.
///
/// `Http` is served by [`crate::ArtifactServer`]; `NotImplemented` continues to
/// describe transports (for example gRPC) that this crate does not link.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NetworkSurface {
    Http,
    NotImplemented,
}

impl NetworkSurface {
    pub const fn supports_http(self) -> bool {
        matches!(self, Self::Http)
    }
    pub const fn reason(self) -> &'static str {
        match self {
            Self::Http => "lawsynth-artifact serves HTTP/1.1 via the std-only ArtifactServer",
            Self::NotImplemented => {
                "lawsynth-artifact implements a local core only; no HTTP transport is linked"
            }
        }
    }
}
