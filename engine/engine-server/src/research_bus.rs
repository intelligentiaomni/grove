use engine_core::{ResearchEvent, SchedulerAction};
use engine_ml::ContinuousResearchScheduler;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

#[derive(Debug, Clone)]
pub struct ResearchOSEventBus {
    events: mpsc::Sender<ResearchEvent>,
}

impl ResearchOSEventBus {
    pub fn new(
        channel_capacity: usize,
        scheduler: ContinuousResearchScheduler,
    ) -> (Self, mpsc::Receiver<SchedulerAction>, JoinHandle<()>) {
        let (event_tx, event_rx) = mpsc::channel(channel_capacity);
        let (action_tx, action_rx) = mpsc::channel(channel_capacity);
        let worker = tokio::spawn(run_event_loop(event_rx, action_tx, scheduler));

        (Self { events: event_tx }, action_rx, worker)
    }

    pub async fn publish(
        &self,
        event: ResearchEvent,
    ) -> Result<(), mpsc::error::SendError<ResearchEvent>> {
        self.events.send(event).await
    }
}

async fn run_event_loop(
    mut events: mpsc::Receiver<ResearchEvent>,
    actions: mpsc::Sender<SchedulerAction>,
    scheduler: ContinuousResearchScheduler,
) {
    while let Some(event) = events.recv().await {
        let action_tx = actions.clone();
        let scheduler = scheduler.clone();

        tokio::spawn(async move {
            let action = scheduler.schedule_event(&event).await;
            let _ = action_tx.send(action).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::ResearchOSEventBus;
    use engine_core::{ResearchEvent, SchedulerAction};
    use engine_ml::ContinuousResearchScheduler;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn event_bus_emits_scheduler_actions() {
        let scheduler = ContinuousResearchScheduler::new(64, 256);
        let (bus, mut actions, worker) = ResearchOSEventBus::new(8, scheduler);

        bus.publish(ResearchEvent::new(
            "thread-bus".to_string(),
            1,
            "unit".to_string(),
            "continue query expansion".to_string(),
        ))
        .await
        .expect("event bus should accept event");

        let action = timeout(Duration::from_secs(1), actions.recv())
            .await
            .expect("action should arrive")
            .expect("action channel should remain open");

        assert!(matches!(
            action,
            SchedulerAction::Continue {
                ref thread_id,
                ..
            } if thread_id == "thread-bus"
        ));

        worker.abort();
    }

    #[tokio::test]
    async fn high_entropy_event_does_not_block_adjacent_event() {
        let scheduler = ContinuousResearchScheduler::new(8, 16);
        let (bus, mut actions, worker) = ResearchOSEventBus::new(8, scheduler);

        bus.publish(ResearchEvent::new(
            "heavy".to_string(),
            1,
            "unit".to_string(),
            "this payload should fork because it is intentionally long".to_string(),
        ))
        .await
        .expect("heavy event should enqueue");
        bus.publish(ResearchEvent::new(
            "empty".to_string(),
            2,
            "unit".to_string(),
            String::new(),
        ))
        .await
        .expect("adjacent event should enqueue");

        let first = timeout(Duration::from_secs(1), actions.recv())
            .await
            .expect("first action should arrive")
            .expect("first action should exist");
        let second = timeout(Duration::from_secs(1), actions.recv())
            .await
            .expect("second action should arrive")
            .expect("second action should exist");

        let saw_heavy = matches!(first, SchedulerAction::Fork { ref thread_id, .. } if thread_id == "heavy")
            || matches!(second, SchedulerAction::Fork { ref thread_id, .. } if thread_id == "heavy");
        let saw_empty = matches!(first, SchedulerAction::Defer { ref thread_id, .. } if thread_id == "empty")
            || matches!(second, SchedulerAction::Defer { ref thread_id, .. } if thread_id == "empty");

        assert!(saw_heavy);
        assert!(saw_empty);

        worker.abort();
    }
}
