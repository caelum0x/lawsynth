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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NetworkSurface {
    NotImplemented,
}

impl NetworkSurface {
    pub const fn supports_http(self) -> bool {
        false
    }
    pub const fn reason(self) -> &'static str {
        "lawsynth-artifact implements a local core only; no HTTP transport is linked"
    }
}
