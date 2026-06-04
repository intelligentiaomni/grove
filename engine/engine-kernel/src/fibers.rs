use std::sync::mpsc::{Sender, Receiver};

pub struct KernelFiber {
    pub id: u64,
    pub execution_state: &'static str,
}

pub struct KernelPayload {
    pub text_pointer: String,       // Inbound stream block
    pub state_hash: String,         // Current operational state signature
}

pub struct KernelChannel {
    pub tx: Sender<KernelPayload>,
    pub rx: Receiver<KernelPayload>,
}