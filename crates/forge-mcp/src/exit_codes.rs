//! Exit code constants and error-to-exit-code mapping.
//!
//! Per the architecture decision DI-014, exit code mapping is a pure function
//! with no side effects. Codes follow the convention defined in BC-5.11.003.
//!
//! | Code | Meaning                                  |
//! |------|------------------------------------------|
//! | 0    | Success                                  |
//! | 1    | Test / conformance failure               |
//! | 2    | Connection error                         |
//! | 3    | Configuration error                      |
//! | 4    | Security finding detected during audit   |

use forge_core::error::CoreError;

/// Successful operation; all assertions passed.
pub const EXIT_SUCCESS: i32 = 0;

/// A conformance or `test` subcommand failure was detected.
pub const EXIT_TEST_FAILURE: i32 = 1;

/// A connection-layer error prevented the operation from completing.
pub const EXIT_CONNECTION_ERROR: i32 = 2;

/// A configuration error (missing files, parse failures) occurred.
pub const EXIT_CONFIG_ERROR: i32 = 3;

/// A security finding was detected during an `audit` run.
pub const EXIT_SECURITY_FINDING: i32 = 4;

/// Top-level CLI error type that wraps all subsystem errors so that
/// [`exit_code_for_error`] can dispatch over a single enum.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CliError {
    /// A conformance test reported failures.
    #[error("conformance test failure: {0}")]
    TestFailure(String),

    /// A connection-layer error from forge-core.
    #[error("connection error: {0}")]
    Connection(#[from] CoreError),

    /// A configuration / discovery error.
    #[error("config error: {0}")]
    Config(String),

    /// Security findings were detected; audit exits non-zero.
    #[error("security finding: {0}")]
    SecurityFinding(String),
}

/// Map a [`CliError`] to the appropriate exit code.
///
/// This is a **pure function** (DI-014): it performs no I/O, has no side
/// effects, and depends only on the variant of `err`.
///
/// When multiple errors are composed, callers should pass the most severe
/// error (EC-002: most severe exit code wins).
pub fn exit_code_for_error(err: &CliError) -> i32 {
    match err {
        CliError::TestFailure(_) => EXIT_TEST_FAILURE,
        CliError::Connection(_) => EXIT_CONNECTION_ERROR,
        CliError::Config(_) => EXIT_CONFIG_ERROR,
        CliError::SecurityFinding(_) => EXIT_SECURITY_FINDING,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // AC-002 — success constant
    #[test]
    fn test_bc_5_11_003_success_exits_0() {
        assert_eq!(EXIT_SUCCESS, 0);
    }

    // AC-003 — test failure exit code
    #[test]
    fn test_bc_5_11_003_test_failure_exits_1() {
        let err = CliError::TestFailure("server did not respond to tools/list".into());
        assert_eq!(exit_code_for_error(&err), EXIT_TEST_FAILURE);
        assert_eq!(EXIT_TEST_FAILURE, 1);
    }

    // AC-004 — connection error exit code
    #[test]
    fn test_bc_5_11_003_connection_error_exits_2() {
        let core_err = CoreError::ConnectionTimeout { seconds: 5 };
        let err = CliError::Connection(core_err);
        assert_eq!(exit_code_for_error(&err), EXIT_CONNECTION_ERROR);
        assert_eq!(EXIT_CONNECTION_ERROR, 2);
    }

    // AC-005 — config error exit code
    #[test]
    fn test_bc_5_11_003_config_error_exits_3() {
        let err = CliError::Config("no config file found".into());
        assert_eq!(exit_code_for_error(&err), EXIT_CONFIG_ERROR);
        assert_eq!(EXIT_CONFIG_ERROR, 3);
    }

    // AC-006 — security finding exit code
    #[test]
    fn test_bc_5_11_003_security_finding_exits_4() {
        let err = CliError::SecurityFinding("prompt injection detected".into());
        assert_eq!(exit_code_for_error(&err), EXIT_SECURITY_FINDING);
        assert_eq!(EXIT_SECURITY_FINDING, 4);
    }

    // Sanity: all constants have distinct values
    #[test]
    fn exit_codes_are_distinct() {
        let codes = [
            EXIT_SUCCESS,
            EXIT_TEST_FAILURE,
            EXIT_CONNECTION_ERROR,
            EXIT_CONFIG_ERROR,
            EXIT_SECURITY_FINDING,
        ];
        let unique: std::collections::HashSet<i32> = codes.iter().copied().collect();
        assert_eq!(unique.len(), codes.len(), "exit codes must be distinct");
    }
}
