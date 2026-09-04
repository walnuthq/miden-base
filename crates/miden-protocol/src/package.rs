use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use miden_core::mast::{MastNodeId, error_code_from_msg};
use miden_mast_package::debug_info::{DebugInfoBuilder, DebugSourceNodeId, PackageDebugInfo};
use miden_mast_package::{Package, PackageDebugInfoError};
use miden_processor::LoadedMastForest;

use crate::MastForest;
use crate::utils::serde::{
    ByteReader,
    ByteWriter,
    Deserializable,
    DeserializationError,
    Serializable,
};

// Temporary bridge while package debug loading is split between protocol and VM APIs. These helpers
// should move to miden-vm once the VM owns package-backed host libraries and debug info loading.

/// Returns package-owned debug info when it can be decoded from trusted package sections.
pub fn package_debug_info(package: &Package) -> Option<Arc<PackageDebugInfo>> {
    match package.debug_info() {
        Ok(debug_info) => debug_info.map(Arc::new),
        Err(PackageDebugInfoError::UntrustedSections) => None,
        Err(_) => None,
    }
}

/// Builds a loaded MAST forest from a MAST forest and package-owned debug info.
pub fn loaded_mast_forest(
    mast: Arc<MastForest>,
    package_debug_info: Option<Arc<PackageDebugInfo>>,
) -> LoadedMastForest {
    match package_debug_info {
        Some(package_debug_info) => {
            LoadedMastForest::with_package_debug_info(mast, Ok(Some((*package_debug_info).clone())))
        },
        None => LoadedMastForest::new(mast),
    }
}

/// Builds a loaded MAST forest from a package, including package-owned debug info when trusted.
pub fn loaded_mast_forest_from_package(package: &Package) -> LoadedMastForest {
    loaded_mast_forest(package.mast_forest().clone(), package_debug_info(package))
}

// ASSERTION ERROR MESSAGES
// ================================================================================================

/// Returns the assertion error messages of `debug_info`, keyed by their error code.
fn error_messages(debug_info: &PackageDebugInfo) -> impl Iterator<Item = (u64, Arc<str>)> + '_ {
    debug_info.error_messages().iter().filter_map(|error_message| {
        debug_info
            .get_string(error_message.message)
            .map(|message| (error_message.err_code, message))
    })
}

/// Returns debug info carrying only the assertion error messages of `debug_info`.
pub(crate) fn error_messages_only(
    debug_info: Option<&PackageDebugInfo>,
) -> Option<Arc<PackageDebugInfo>> {
    let debug_info = debug_info?;

    let mut builder = DebugInfoBuilder::<MastNodeId, DebugSourceNodeId>::default();
    let mut is_empty = true;
    for (err_code, message) in error_messages(debug_info) {
        builder.add_error_message(err_code, message);
        is_empty = false;
    }

    (!is_empty).then(|| Arc::from(builder.build()))
}

/// Writes the assertion error messages of `debug_info`. Their error codes are not written since
/// they are the hash of the message itself.
pub(crate) fn write_error_messages<W: ByteWriter>(
    debug_info: Option<&PackageDebugInfo>,
    target: &mut W,
) {
    let messages: Vec<_> = debug_info
        .map(|debug_info| error_messages(debug_info).map(|(_, message)| message).collect())
        .unwrap_or_default();

    target.write_usize(messages.len());
    for message in messages {
        message.as_ref().write_into(target);
    }
}

/// Returns the number of bytes [`write_error_messages`] writes.
pub(crate) fn error_messages_size_hint(debug_info: Option<&PackageDebugInfo>) -> usize {
    let Some(debug_info) = debug_info else {
        return 0usize.get_size_hint();
    };

    let mut num_messages = 0usize;
    let mut size = 0;
    for (_, message) in error_messages(debug_info) {
        size += message.as_ref().get_size_hint();
        num_messages += 1;
    }

    size + num_messages.get_size_hint()
}

/// Reads debug info written by [`write_error_messages`].
pub(crate) fn read_error_messages<R: ByteReader>(
    source: &mut R,
) -> Result<Option<Arc<PackageDebugInfo>>, DeserializationError> {
    let num_messages = source.read_usize()?;
    if num_messages == 0 {
        return Ok(None);
    }

    let mut builder = DebugInfoBuilder::<MastNodeId, DebugSourceNodeId>::default();
    for _ in 0..num_messages {
        let message = String::read_from(source)?;
        let err_code = error_code_from_msg(&message).as_canonical_u64();
        builder.add_error_message(err_code, Arc::from(message.as_str()));
    }

    Ok(Some(Arc::from(builder.build())))
}
