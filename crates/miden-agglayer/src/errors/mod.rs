// Include generated error constants
#[cfg(any(feature = "testing", test))]
include!(concat!(env!("OUT_DIR"), "/agglayer_errors.rs"));

mod generated_masm_error_messages {
    include!(concat!(env!("OUT_DIR"), "/masm_error_messages.rs"));
}

/// The error messages of all MASM errors defined by the Miden agglayer, keyed by their error code
/// and sorted by it.
pub const AGGLAYER_MASM_ERROR_MESSAGES: &[(u64, &str)] =
    &generated_masm_error_messages::MASM_ERROR_MESSAGES;
