/// Severity for an inspectable engine diagnostic.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

/// A structured message retained with a run or artifact rather than emitted as
/// hidden process output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub message: String,
}

impl Diagnostic {
    pub fn new(
        severity: DiagnosticSeverity,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            code: code.into(),
            message: message.into(),
        }
    }
}

/// Deterministically ordered diagnostics collected during one operation.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Diagnostics {
    entries: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.entries.push(diagnostic);
    }

    pub fn info(&mut self, code: impl Into<String>, message: impl Into<String>) {
        self.push(Diagnostic::new(DiagnosticSeverity::Info, code, message));
    }

    pub fn warning(&mut self, code: impl Into<String>, message: impl Into<String>) {
        self.push(Diagnostic::new(DiagnosticSeverity::Warning, code, message));
    }

    pub fn error(&mut self, code: impl Into<String>, message: impl Into<String>) {
        self.push(Diagnostic::new(DiagnosticSeverity::Error, code, message));
    }

    pub fn entries(&self) -> &[Diagnostic] {
        &self.entries
    }

    pub fn has_errors(&self) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.severity == DiagnosticSeverity::Error)
    }
}
