use super::session::Session;
use super::turn_context::TurnContext;
use codex_protocol::models::ResponseItem;
use std::sync::Arc;

impl Session {
    /// Returns the input if there is no active turn to inject into.
    pub async fn inject_if_running(
        &self,
        input: Vec<ResponseItem>,
    ) -> Result<(), Vec<ResponseItem>> {
        self.input_queue
            .inject_if_running(&self.active_turn, input)
            .await
    }

    /// Injects items into active work, or queues them and starts a turn when idle.
    pub(crate) async fn inject_starts_turn(self: &Arc<Self>, items: Vec<ResponseItem>) {
        if let Err(items) = self.inject_if_running(items).await {
            self.input_queue
                .queue_response_items_for_next_turn(items)
                .await;
            self.maybe_start_turn_for_pending_work().await;
        }
    }

    /// Injects items into active work, or records them without starting a turn.
    pub(crate) async fn inject_no_new_turn(
        &self,
        items: Vec<ResponseItem>,
        current_turn_context: Option<&TurnContext>,
    ) {
        let Err(items) = self.inject_if_running(items).await else {
            return;
        };
        match current_turn_context {
            Some(turn_context) => {
                self.record_conversation_items(turn_context, &items).await;
            }
            None => {
                let turn_context = self.new_default_turn().await;
                self.record_conversation_items(turn_context.as_ref(), &items)
                    .await;
            }
        }
    }
}
