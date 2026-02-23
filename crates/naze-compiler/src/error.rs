use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileError {
    pub message: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub severity: Severity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_fix: Option<String>,
}

impl CompileError {
    pub fn new(
        message: String,
        file: String,
        line: usize,
        column: usize,
        severity: Severity,
    ) -> Self {
        Self {
            message,
            file,
            line,
            column,
            severity,
            error_code: None,
            suggested_fix: None,
        }
    }

    pub fn with_fix(mut self, code: &str, fix: &str) -> Self {
        self.error_code = Some(code.to_string());
        self.suggested_fix = Some(fix.to_string());
        self
    }
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sev = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        if let Some(code) = &self.error_code {
            write!(
                f,
                "{}[{}][{}:{}:{}]: {}",
                sev, code, self.file, self.line, self.column, self.message
            )
        } else {
            write!(
                f,
                "{}[{}:{}:{}]: {}",
                sev, self.file, self.line, self.column, self.message
            )
        }
    }
}

impl std::error::Error for CompileError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_returns_none_for_optional_fields() {
        let err = CompileError::new(
            "test error".into(),
            "app.naze".into(),
            1,
            1,
            Severity::Error,
        );
        assert!(err.error_code.is_none());
        assert!(err.suggested_fix.is_none());
    }

    #[test]
    fn with_fix_populates_both_fields() {
        let err = CompileError::new(
            "unknown component".into(),
            "app.naze".into(),
            5,
            4,
            Severity::Error,
        )
        .with_fix("E001", "Add 'use \"components/card\"' at top of file");
        assert_eq!(err.error_code.as_deref(), Some("E001"));
        assert_eq!(
            err.suggested_fix.as_deref(),
            Some("Add 'use \"components/card\"' at top of file")
        );
    }

    #[test]
    fn json_skips_none_fields() {
        let err = CompileError::new(
            "test".into(),
            "app.naze".into(),
            1,
            1,
            Severity::Error,
        );
        let json = serde_json::to_string(&err).unwrap();
        assert!(!json.contains("error_code"));
        assert!(!json.contains("suggested_fix"));
    }

    #[test]
    fn json_includes_some_fields() {
        let err = CompileError::new(
            "test".into(),
            "app.naze".into(),
            1,
            1,
            Severity::Error,
        )
        .with_fix("E001", "fix it");
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"error_code\":\"E001\""));
        assert!(json.contains("\"suggested_fix\":\"fix it\""));
    }
}
