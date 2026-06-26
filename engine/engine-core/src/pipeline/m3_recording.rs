use crate::pipeline::m1_state::IngestionAction;

pub trait PipelineObserver {
    fn on_action_triggered(&self, action: IngestionAction);
}

pub struct AuditLogger;
impl PipelineObserver for AuditLogger {
    fn on_action_triggered(&self, action: IngestionAction) {
        let timestamp = "2026-06-25T15:00:00Z";
        println!("[TX RECORD] {} - Applied Action: {:?}", timestamp, action);
        // In full execution, this streams actions directly to an append-only WAL (Write-Ahead Log)
    }
}