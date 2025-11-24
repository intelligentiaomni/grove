# IO Lab / GROVE — Scientific Architecture

This document describes the scientific architecture of the engine. It emphasizes evidence, inference quality, simulation fidelity, and the continuous refinement of hypotheses.

The system is designed as a *continuous scientific loop* capable of generating, evaluating, and improving scientific insights across domains.

---

## High-Level Flow

```mermaid
flowchart TD
    classDef phase1 fill:#e8f5e9,stroke:#2e7d32,color:#1b5e20;
    classDef phase2 fill:#e3f2fd,stroke:#1565c0,color:#0d47a1;
    classDef phase3 fill:#fff3e0,stroke:#ef6c00,color:#e65100;
    classDef core fill:#f5f5f5,stroke:#424242,color:#212121;

    %% CORE NODES
    U[User Interaction<br/>(Builder Game, Scientific UI)]:::core
    I[Idea / Scenario Input<br/>Hypotheses · Models · Patterns]:::core

    ECORE[IO LAB ENGINE<br/><br/>
    • Simulation kernels<br/>
    • ML inference layers<br/>
    • Reasoning & planning<br/>
    • Multimodal scientific fusion]:::core

    OUT[Model & Simulation Outputs<br/>Predictions · Hypothesis Traces<br/>Design Candidates]:::core

    EVL[Evidence & Evaluation Layer<br/><br/>
    • Predictive checks<br/>
    • Golden cases & reference sets<br/>
    • Scientific error analysis<br/>
    • Foresight & scenario testing]:::phase1

    IMP[System Refinement Loop<br/><br/>
    • Update simulation fidelity<br/>
    • Improve inference & reasoning<br/>
    • Strengthen hypothesis space]:::core

    %% ADVANCED PHASE NODES
    E2[Experimental Workflow Space<br/><br/>
    • Cross-domain reasoning<br/>
    • Multi-agent inquiry<br/>
    • Novel hypothesis strategies]:::phase2

    HIP[High-Impact Scientific Targets<br/>Frontier Domains<br/>(Phase 3)]:::phase3

    %% MAIN CYCLE
    U --> I --> ECORE --> OUT --> EVL --> IMP --> U
    IMP --> HIP

    %% PHASE 2 EXTENSION
    ECORE --> E2 --> OUT
```