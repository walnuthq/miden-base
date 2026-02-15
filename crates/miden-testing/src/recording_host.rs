use alloc::sync::Arc;
use alloc::vec::Vec;

use miden_processor::{
    FutureMaybeSend, Host, ProcessorState, TraceError,
    advice::AdviceMutation,
    event::EventError,
    mast::MastForest,
};
use miden_protocol::Word;
use miden_protocol::assembly::SourceFile;
use miden_protocol::assembly::debuginfo::{Location, SourceSpan};

/// A wrapper around any [`Host`] that records the [`AdviceMutation`]s returned
/// by each `on_event()` call.
///
/// After execution, the recorded mutations can be extracted and replayed in the
/// debugger's host to reproduce the same advice state without needing the real
/// transaction host.
pub(crate) struct RecordingHostWrapper<H> {
    inner: H,
    recorded_events: Vec<Vec<AdviceMutation>>,
}

impl<H: Host> RecordingHostWrapper<H> {
    /// Create a new recording wrapper around the given host.
    pub fn new(inner: H) -> Self {
        Self {
            inner,
            recorded_events: Vec::new(),
        }
    }

    /// Consume the wrapper and return the recorded event mutations.
    pub fn into_recorded_events(self) -> Vec<Vec<AdviceMutation>> {
        self.recorded_events
    }
}

impl<H: Host + Send> Host for RecordingHostWrapper<H> {
    fn get_label_and_source_file(
        &self,
        location: &Location,
    ) -> (SourceSpan, Option<Arc<SourceFile>>) {
        self.inner.get_label_and_source_file(location)
    }

    fn get_mast_forest(&self, node_digest: &Word) -> impl FutureMaybeSend<Option<Arc<MastForest>>> {
        self.inner.get_mast_forest(node_digest)
    }

    fn on_event(
        &mut self,
        process: &ProcessorState<'_>,
    ) -> impl FutureMaybeSend<Result<Vec<AdviceMutation>, EventError>> {
        // Use disjoint field borrows: `self.inner` and `self.recorded_events`
        // are separate fields, so we can borrow them independently.
        let inner = &mut self.inner;
        let recorded = &mut self.recorded_events;
        async move {
            let mutations = inner.on_event(process).await?;
            recorded.push(mutations.clone());
            Ok(mutations)
        }
    }

    fn on_trace(&mut self, process: &ProcessorState, trace_id: u32) -> Result<(), TraceError> {
        self.inner.on_trace(process, trace_id)
    }
}
