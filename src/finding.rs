//! The uniform finding shape every failed check emits.

use std::fmt;

use serde::{Deserialize, Serialize};

/// v1 has a single level; the field exists so consumers can rely on its presence.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    #[default]
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => f.write_str("error"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// Check id: `<doc>.rule[i]`, an explicit `id:`, `load`, or an `each:` id
    /// carrying its outer match — `<doc>.rule[i].each[j]@<line>`.
    pub check: String,
    pub severity: Severity,
    /// Set by the caller; the runner takes it as given.
    pub file: String,
    pub line: usize,
    pub expected: String,
    pub actual: String,
    pub message: String,
}

impl Finding {
    pub fn new(
        check: impl Into<String>,
        file: impl Into<String>,
        line: usize,
        expected: impl Into<String>,
        actual: impl Into<String>,
        message: impl Into<String>,
    ) -> Finding {
        Finding {
            check: check.into(),
            severity: Severity::Error,
            file: file.into(),
            line,
            expected: expected.into(),
            actual: actual.into(),
            message: message.into(),
        }
    }
}
