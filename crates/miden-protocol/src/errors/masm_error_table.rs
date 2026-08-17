mod generated {
    include!(concat!(env!("OUT_DIR"), "/masm_error_messages.rs"));
}

/// The messages of all MASM errors defined by the transaction kernel and the protocol library,
/// keyed by their error code and sorted by it.
pub const PROTOCOL_MASM_ERROR_MESSAGES: &[(u64, &str)] = &generated::MASM_ERROR_MESSAGES;

/// Returns the message registered for `err_code` in the provided table, which must be sorted by
/// error code.
pub fn masm_error_message_from_table(
    table: &[(u64, &'static str)],
    err_code: u64,
) -> Option<&'static str> {
    table
        .binary_search_by_key(&err_code, |(code, _)| *code)
        .ok()
        .map(|index| table[index].1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::tx_kernel::ERR_ACCOUNT_CODE_COMMITMENT_MISMATCH;

    #[test]
    fn error_message_table_is_sorted_by_code() {
        assert!(PROTOCOL_MASM_ERROR_MESSAGES.is_sorted_by_key(|(code, _)| *code));
    }

    #[test]
    fn error_message_table_resolves_kernel_error() {
        let error = ERR_ACCOUNT_CODE_COMMITMENT_MISMATCH;
        let message = masm_error_message_from_table(
            PROTOCOL_MASM_ERROR_MESSAGES,
            error.code().as_canonical_u64(),
        );

        assert_eq!(message, Some(error.message()));
    }
}
