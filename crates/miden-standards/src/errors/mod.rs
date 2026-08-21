/// The errors from the MASM code of the Miden standards.
#[cfg(any(feature = "testing", test))]
pub mod standards {
    include!(concat!(env!("OUT_DIR"), "/standards_errors.rs"));
}

mod generated_masm_error_messages {
    include!(concat!(env!("OUT_DIR"), "/masm_error_messages.rs"));
}

/// The error messages of all MASM errors defined by the Miden standards, keyed by their error code
/// and sorted by it.
pub const STANDARDS_MASM_ERROR_MESSAGES: &[(u64, &str)] =
    &generated_masm_error_messages::MASM_ERROR_MESSAGES;

mod code_builder_errors;

pub use code_builder_errors::CodeBuilderError;

#[cfg(test)]
mod tests {
    use miden_protocol::errors::masm_error_message_from_table;

    use super::STANDARDS_MASM_ERROR_MESSAGES;
    use super::standards::ERR_MINT_NOTE_ASSET_NOT_FROM_THIS_FAUCET;

    #[test]
    fn error_message_table_resolves_standards_error() {
        let error = ERR_MINT_NOTE_ASSET_NOT_FROM_THIS_FAUCET;
        let message = masm_error_message_from_table(
            STANDARDS_MASM_ERROR_MESSAGES,
            error.code().as_canonical_u64(),
        );

        assert_eq!(message, Some(error.message()));
    }
}
