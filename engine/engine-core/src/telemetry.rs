use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryPayload {
    pub twin_id: String,
    pub metric: String,
    pub value: f32,
    pub timestamp_utc: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryBounds {
    pub metric: String,
    pub min: f32,
    pub max: f32,
    pub max_delta_per_ms: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TelemetryDecision {
    Clean,
    Imputed,
    Quarantine,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryValidationReport {
    pub decision: TelemetryDecision,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TwinKind {
    SoftwareSupplyChain,
    FactoryAutomation,
    RegionalEnergyGrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TwinState {
    pub resilience: f32,
    pub throughput: f32,
    pub risk: f32,
}

pub fn validate_telemetry(
    current: &TelemetryPayload,
    previous: Option<&TelemetryPayload>,
    bounds: &[TelemetryBounds],
) -> TelemetryValidationReport {
    if current.twin_id.trim().is_empty() || current.metric.trim().is_empty() {
        return quarantine("missing twin id or metric");
    }

    if !is_iso8601_utc(&current.timestamp_utc) {
        return quarantine("timestamp must be explicit ISO 8601 UTC");
    }

    if !current.value.is_finite() {
        return quarantine("value must be a finite float");
    }

    let Some(bound) = bounds.iter().find(|bound| bound.metric == current.metric) else {
        return quarantine("metric has no declared physical bounds");
    };

    if current.value < bound.min || current.value > bound.max {
        return quarantine("value exceeds physical range constraints");
    }

    if let (Some(previous), Some(max_delta)) = (previous, bound.max_delta_per_ms) {
        if previous.metric == current.metric {
            let delta = (current.value - previous.value).abs();
            if delta > max_delta {
                return TelemetryValidationReport {
                    decision: TelemetryDecision::Imputed,
                    reason: "rate-of-change exceeded; substitute proxy stream".to_string(),
                };
            }
        }
    }

    TelemetryValidationReport {
        decision: TelemetryDecision::Clean,
        reason: "telemetry passed structural and boundary validation".to_string(),
    }
}

pub fn simulate_twin(
    kind: TwinKind,
    baseline: TwinState,
    intervention: f32,
    steps: u32,
) -> TwinState {
    let (resilience_gain, throughput_gain, risk_gain) = match kind {
        TwinKind::SoftwareSupplyChain => (0.018, 0.012, -0.020),
        TwinKind::FactoryAutomation => (0.010, 0.020, -0.012),
        TwinKind::RegionalEnergyGrid => (0.014, 0.009, -0.016),
    };
    let scale = intervention.clamp(-1.0, 1.0) * steps as f32;

    TwinState {
        resilience: (baseline.resilience + resilience_gain * scale).clamp(0.0, 1.0),
        throughput: (baseline.throughput + throughput_gain * scale).clamp(0.0, 1.0),
        risk: (baseline.risk + risk_gain * scale).clamp(0.0, 1.0),
    }
}

pub fn residual_error(predicted: TwinState, actual: TwinState) -> f32 {
    let resilience = predicted.resilience - actual.resilience;
    let throughput = predicted.throughput - actual.throughput;
    let risk = predicted.risk - actual.risk;
    ((resilience * resilience) + (throughput * throughput) + (risk * risk)).sqrt()
}

fn quarantine(reason: &str) -> TelemetryValidationReport {
    TelemetryValidationReport {
        decision: TelemetryDecision::Quarantine,
        reason: reason.to_string(),
    }
}

fn is_iso8601_utc(value: &str) -> bool {
    value.len() >= 20
        && value.ends_with('Z')
        && value.as_bytes().get(4) == Some(&b'-')
        && value.as_bytes().get(7) == Some(&b'-')
        && value.as_bytes().get(10) == Some(&b'T')
        && value.as_bytes().get(13) == Some(&b':')
        && value.as_bytes().get(16) == Some(&b':')
}

#[cfg(test)]
mod tests {
    use super::{
        residual_error, simulate_twin, validate_telemetry, TelemetryBounds, TelemetryDecision,
        TelemetryPayload, TwinKind, TwinState,
    };

    #[test]
    fn quarantines_non_physical_telemetry() {
        let payload = TelemetryPayload {
            twin_id: "factory".to_string(),
            metric: "temperature_c".to_string(),
            value: 140.0,
            timestamp_utc: "2026-06-04T12:00:00Z".to_string(),
        };
        let bounds = [TelemetryBounds {
            metric: "temperature_c".to_string(),
            min: -50.0,
            max: 85.0,
            max_delta_per_ms: None,
        }];

        let report = validate_telemetry(&payload, None, &bounds);

        assert_eq!(report.decision, TelemetryDecision::Quarantine);
    }

    #[test]
    fn twin_simulation_is_deterministic() {
        let baseline = TwinState {
            resilience: 0.5,
            throughput: 0.5,
            risk: 0.5,
        };

        let a = simulate_twin(TwinKind::RegionalEnergyGrid, baseline, 0.8, 10);
        let b = simulate_twin(TwinKind::RegionalEnergyGrid, baseline, 0.8, 10);

        assert_eq!(a, b);
        assert_eq!(residual_error(a, a), 0.0);
    }
}
