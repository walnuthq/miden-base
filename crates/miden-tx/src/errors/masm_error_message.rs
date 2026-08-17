use alloc::sync::Arc;

use miden_agglayer::errors::AGGLAYER_MASM_ERROR_MESSAGES;
use miden_processor::ExecutionError;
use miden_processor::operation::OperationError;
use miden_protocol::Felt;
use miden_protocol::errors::{PROTOCOL_MASM_ERROR_MESSAGES, masm_error_message_from_table};
use miden_standards::errors::STANDARDS_MASM_ERROR_MESSAGES;

/// Attaches the message of a failed assertion to the provided error, if it is missing.
///
/// The VM resolves an error code through the debug info of the package that raised it, which
/// serializing account code and scripts drops. Errors of unknown code are left unchanged.
pub(crate) fn resolve_masm_error_message(error: ExecutionError) -> ExecutionError {
    match error {
        ExecutionError::OperationError { label, source_file, err } => {
            ExecutionError::OperationError {
                label,
                source_file,
                err: resolve_operation_error_message(err),
            }
        },
        error => error,
    }
}

/// Attaches the message of a failed assertion to the provided operation error, if it is missing.
fn resolve_operation_error_message(err: OperationError) -> OperationError {
    match err {
        OperationError::FailedAssertion { err_code, err_msg: None } => {
            OperationError::FailedAssertion { err_msg: message_of(err_code), err_code }
        },
        OperationError::U32AssertionFailed { err_code, err_msg: None, invalid_values } => {
            OperationError::U32AssertionFailed {
                err_msg: message_of(err_code),
                err_code,
                invalid_values,
            }
        },
        OperationError::MerklePathVerificationFailed { mut inner } if inner.err_msg.is_none() => {
            inner.err_msg = message_of(inner.err_code);
            OperationError::MerklePathVerificationFailed { inner }
        },
        err => err,
    }
}

/// Returns the message of the MASM error with the provided code, if it is a known error.
fn message_of(err_code: Felt) -> Option<Arc<str>> {
    let err_code = err_code.as_canonical_u64();

    masm_error_message_from_table(PROTOCOL_MASM_ERROR_MESSAGES, err_code)
        .or_else(|| masm_error_message_from_table(STANDARDS_MASM_ERROR_MESSAGES, err_code))
        .or_else(|| masm_error_message_from_table(AGGLAYER_MASM_ERROR_MESSAGES, err_code))
        .map(Arc::from)
}
