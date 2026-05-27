#![no_std]

pub mod concurrency;

use sha2::{Digest, Sha256};

pub use concurrency::{
    ChannelError, FiberState, KernelChannel, KernelFiber, KernelSchedulerAction,
};

pub type ContentHash = [u8; 32];
pub type ModuleId = u64;
pub type TaskId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelError {
    CapabilityMapFull,
    EmptyWeights,
    InvalidMatrixDimensions,
    InvalidWeightLength,
    ArithmeticOverflow,
    ConfigurationRejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quantization {
    Fp32,
    Fp16,
    Int8,
    UInt8,
}

impl Quantization {
    pub const fn bytes_per_weight(self) -> usize {
        match self {
            Self::Fp32 => 4,
            Self::Fp16 => 2,
            Self::Int8 | Self::UInt8 => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleClass {
    TokenParsing,
    GeneralInference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpuTaskFrame {
    pub task_id: TaskId,
    pub module_id: ModuleId,
    pub weights_hash: ContentHash,
    pub rows: u16,
    pub cols: u16,
    pub quantization: Quantization,
    pub schedule_class: ScheduleClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityEntry {
    pub module_id: ModuleId,
    pub content_hash: ContentHash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilitySnapshot<const N: usize> {
    entries: [Option<CapabilityEntry>; N],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityMap<const N: usize> {
    entries: [Option<CapabilityEntry>; N],
}

impl<const N: usize> CapabilityMap<N> {
    pub const fn new() -> Self {
        Self { entries: [None; N] }
    }

    pub const fn snapshot(&self) -> CapabilitySnapshot<N> {
        CapabilitySnapshot {
            entries: self.entries,
        }
    }

    pub fn rollback(&mut self, snapshot: CapabilitySnapshot<N>) {
        self.entries = snapshot.entries;
    }

    pub fn get(&self, module_id: ModuleId) -> Option<ContentHash> {
        self.entries.iter().find_map(|entry| match entry {
            Some(entry) if entry.module_id == module_id => Some(entry.content_hash),
            _ => None,
        })
    }

    pub fn upsert(
        &mut self,
        module_id: ModuleId,
        content_hash: ContentHash,
    ) -> Result<(), KernelError> {
        if let Some(entry) = self
            .entries
            .iter_mut()
            .flatten()
            .find(|entry| entry.module_id == module_id)
        {
            entry.content_hash = content_hash;
            return Ok(());
        }

        if let Some(slot) = self.entries.iter_mut().find(|entry| entry.is_none()) {
            *slot = Some(CapabilityEntry {
                module_id,
                content_hash,
            });
            return Ok(());
        }

        Err(KernelError::CapabilityMapFull)
    }

    pub fn transactional_upsert<F>(
        &mut self,
        module_id: ModuleId,
        content_hash: ContentHash,
        validate: F,
    ) -> Result<(), KernelError>
    where
        F: FnOnce() -> Result<(), KernelError>,
    {
        let snapshot = self.snapshot();
        self.upsert(module_id, content_hash)?;

        if let Err(err) = validate() {
            self.rollback(snapshot);
            return Err(err);
        }

        Ok(())
    }
}

impl<const N: usize> Default for CapabilityMap<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelState<const N: usize> {
    pub capabilities: CapabilityMap<N>,
    next_task_id: TaskId,
}

impl<const N: usize> KernelState<N> {
    pub const fn new() -> Self {
        Self {
            capabilities: CapabilityMap::new(),
            next_task_id: 1,
        }
    }

    fn allocate_task_id(&mut self) -> TaskId {
        let task_id = self.next_task_id;
        self.next_task_id = self.next_task_id.wrapping_add(1);
        task_id
    }
}

impl<const N: usize> Default for KernelState<N> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn content_hash(bytes: &[u8]) -> ContentHash {
    Sha256::digest(bytes).into()
}

pub fn register_token_parsing_matrix_weights<const N: usize>(
    state: &mut KernelState<N>,
    module_id: ModuleId,
    weights: &[u8],
    rows: u16,
    cols: u16,
    quantization: Quantization,
) -> Result<NpuTaskFrame, KernelError> {
    validate_matrix_weights(weights, rows, cols, quantization)?;

    let weights_hash = content_hash(weights);
    state.capabilities.upsert(module_id, weights_hash)?;

    Ok(NpuTaskFrame {
        task_id: state.allocate_task_id(),
        module_id,
        weights_hash,
        rows,
        cols,
        quantization,
        schedule_class: ScheduleClass::TokenParsing,
    })
}

fn validate_matrix_weights(
    weights: &[u8],
    rows: u16,
    cols: u16,
    quantization: Quantization,
) -> Result<(), KernelError> {
    if weights.is_empty() {
        return Err(KernelError::EmptyWeights);
    }

    if rows == 0 || cols == 0 {
        return Err(KernelError::InvalidMatrixDimensions);
    }

    let elements = usize::from(rows)
        .checked_mul(usize::from(cols))
        .ok_or(KernelError::ArithmeticOverflow)?;
    let expected_len = elements
        .checked_mul(quantization.bytes_per_weight())
        .ok_or(KernelError::ArithmeticOverflow)?;

    if weights.len() != expected_len {
        return Err(KernelError::InvalidWeightLength);
    }

    Ok(())
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::{
        content_hash, register_token_parsing_matrix_weights, KernelError, KernelState,
        Quantization, ScheduleClass,
    };

    #[test]
    fn failed_module_configuration_rolls_back_capability_hash() {
        let module_id = 42;
        let mut state = KernelState::<4>::new();
        let previous_hash = content_hash(b"known-good-token-parser");
        let staged_hash = content_hash(b"broken-token-parser");

        state
            .capabilities
            .upsert(module_id, previous_hash)
            .expect("initial capability should fit");

        let err = state
            .capabilities
            .transactional_upsert(module_id, staged_hash, || {
                Err(KernelError::ConfigurationRejected)
            })
            .expect_err("configuration should be rejected");

        assert_eq!(err, KernelError::ConfigurationRejected);
        assert_eq!(state.capabilities.get(module_id), Some(previous_hash));
    }

    #[test]
    fn registers_token_parsing_weights_as_npu_task_frame() {
        let mut state = KernelState::<4>::new();
        let weights = [1_u8, 2, 3, 4, 5, 6];

        let frame = register_token_parsing_matrix_weights(
            &mut state,
            7,
            &weights,
            2,
            3,
            Quantization::UInt8,
        )
        .expect("valid matrix should register");

        assert_eq!(frame.module_id, 7);
        assert_eq!(frame.weights_hash, content_hash(&weights));
        assert_eq!(frame.schedule_class, ScheduleClass::TokenParsing);
        assert_eq!(state.capabilities.get(7), Some(frame.weights_hash));
    }

    #[test]
    fn rejects_mismatched_weight_lengths_before_mapping_capability() {
        let mut state = KernelState::<4>::new();

        let err = register_token_parsing_matrix_weights(
            &mut state,
            11,
            &[1, 2, 3],
            2,
            3,
            Quantization::UInt8,
        )
        .expect_err("short weight buffer must be rejected");

        assert_eq!(err, KernelError::InvalidWeightLength);
        assert_eq!(state.capabilities.get(11), None);
    }
}
