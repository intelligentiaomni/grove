use engine_core::{ResearchEvent, ResearchThread, SchedulerAction};

#[derive(Debug, Clone)]
pub struct ContinuousResearchScheduler {
    high_entropy_payload_bytes: usize,
    fork_payload_bytes: usize,
}

impl ContinuousResearchScheduler {
    pub const fn new(high_entropy_payload_bytes: usize, fork_payload_bytes: usize) -> Self {
        Self {
            high_entropy_payload_bytes,
            fork_payload_bytes,
        }
    }

    pub const fn default() -> Self {
        Self {
            high_entropy_payload_bytes: 32 * 1024,
            fork_payload_bytes: 128 * 1024,
        }
    }

    pub async fn schedule_event(&self, event: &ResearchEvent) -> SchedulerAction {
        if event.payload.len() >= self.high_entropy_payload_bytes {
            tokio::task::yield_now().await;
        }

        if event.payload.trim().is_empty() {
            return SchedulerAction::Defer {
                thread_id: event.thread_id.clone(),
                reason: "empty research event".to_string(),
            };
        }

        if event.payload.len() >= self.fork_payload_bytes {
            return SchedulerAction::Fork {
                thread_id: event.thread_id.clone(),
                child_thread_id: fork_thread_id(event),
                reason: "high entropy payload requires isolated continuation".to_string(),
            };
        }

        if has_completion_signal(&event.payload) {
            return SchedulerAction::Complete {
                thread_id: event.thread_id.clone(),
                summary: compact_summary(&event.payload),
            };
        }

        SchedulerAction::Continue {
            thread_id: event.thread_id.clone(),
            next_query: compact_query(&event.payload),
        }
    }

    pub async fn schedule_thread(&self, thread: &ResearchThread) -> SchedulerAction {
        match thread.events.last() {
            Some(event) => self.schedule_event(event).await,
            None => SchedulerAction::Defer {
                thread_id: thread.id.clone(),
                reason: "research thread has no events".to_string(),
            },
        }
    }
}

impl Default for ContinuousResearchScheduler {
    fn default() -> Self {
        Self::new(32 * 1024, 128 * 1024)
    }
}

fn has_completion_signal(payload: &str) -> bool {
    payload
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .any(|token| {
            matches!(
                token.to_ascii_lowercase().as_str(),
                "resolved" | "complete" | "done"
            )
        })
}

fn compact_query(payload: &str) -> String {
    const LIMIT: usize = 192;
    payload.chars().take(LIMIT).collect()
}

fn compact_summary(payload: &str) -> String {
    const LIMIT: usize = 160;
    payload.chars().take(LIMIT).collect()
}

fn fork_thread_id(event: &ResearchEvent) -> String {
    let mut id = String::with_capacity(event.thread_id.len() + 21);
    id.push_str(&event.thread_id);
    id.push_str("-fork-");
    id.push_str(&event.sequence.to_string());
    id
}

#[cfg(test)]
mod tests {
    use super::ContinuousResearchScheduler;
    use engine_core::{ResearchEvent, ResearchThread, SchedulerAction};

    #[tokio::test]
    async fn schedules_continue_for_normal_research_event() {
        let scheduler = ContinuousResearchScheduler::new(64, 256);
        let event = ResearchEvent::new(
            "thread-a".to_string(),
            1,
            "unit".to_string(),
            "investigate vector database recall drift".to_string(),
        );

        let action = scheduler.schedule_event(&event).await;

        assert!(matches!(
            action,
            SchedulerAction::Continue {
                ref thread_id,
                ..
            } if thread_id == "thread-a"
        ));
    }

    #[tokio::test]
    async fn forks_high_entropy_payloads_after_async_yield() {
        let scheduler = ContinuousResearchScheduler::new(8, 16);
        let event = ResearchEvent::new(
            "thread-b".to_string(),
            9,
            "unit".to_string(),
            "this payload is intentionally large enough to fork".to_string(),
        );

        let action = scheduler.schedule_event(&event).await;

        assert_eq!(
            action,
            SchedulerAction::Fork {
                thread_id: "thread-b".to_string(),
                child_thread_id: "thread-b-fork-9".to_string(),
                reason: "high entropy payload requires isolated continuation".to_string(),
            }
        );
    }

    #[tokio::test]
    async fn schedules_from_thread_tail_event() {
        let scheduler = ContinuousResearchScheduler::default();
        let mut thread = ResearchThread::new("thread-c".to_string(), "demo".to_string());
        thread.events.push(ResearchEvent::new(
            "thread-c".to_string(),
            2,
            "unit".to_string(),
            "resolved after final synthesis".to_string(),
        ));

        let action = scheduler.schedule_thread(&thread).await;

        assert!(matches!(action, SchedulerAction::Complete { .. }));
    }
}
