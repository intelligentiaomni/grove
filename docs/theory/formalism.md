# Grove Formalism v0.1

## Purpose

This document defines the minimal mathematical structure governing fragment representation, transformation, evaluation, and selection within Grove.

All core modules MUST be derivable from this specification.

## 1. Fragment Definition <br>
### 1.1 Fragment Space <br>

Let *$\mathcal{F}$* be the space of all valid fragments.

A fragment is defined as:

*f* = ( *S*, *T*, *M*, *$\phi$* )

where

*S*: Structure (machine-interpretable representation)

*T*: Type (categorical role)

*M*: Metadata (contextual attributes)

*$\phi$*: Fitness function projection

### 1.2 Structure <br>

*S* $\in\mathcal{S}$

Constraints:

* Must be serializable

* Must be hashable (identity stability)

* Must satisfy schema constraints

### 1.3 Type <br>

*T* $\in\mathcal{T}$

Finite type set:

*T* = { hypothesis, heuristic, constraint, metric, artifact }

Type determines allowed transformation operators.

### 1.4 Metadata <br>

*M* = ( *c*, *v*, *τ* )

where

*c*: context

*v*: version

*$\tau$*: timestamp

Metadata does not affect fragment identity unless explicitly included in structure.

## 2. Fragment Validity

A fragment  *f*  is valid iff:

* *S* satisfies schema

* *T* $\in \mathcal{T}$

* Required metadata fields exist

* All invariants (Section 6) hold 

Invalid fragments MUST NOT enter the population.

## 3. Evolution Operators

Let *E* : $\mathcal{F}$ $\to$ $\mathcal{F}$

Evolution operators are members of set:

$\mathcal{E}$ = { *$\mu$*, *$\rho$*, *$\pi$*, *$\kappa$* }

where

$\mu$: mutation

$\rho$: recombination

$\pi$: pruning

$\kappa$: compression

### 3.1 Mutation <br>

*$\mu$* ( *f* ) = *f* ′

Constraints:

* Type preserved

* Schema preserved

* Identity hash changes

### 3.2 Recombination <br>

$\rho$ ( *$f_i$* , *$f_j$* ) = *$f_k$*

Constraints:

* Result type must be valid

* Parent fragments remain unchanged

### 3.3 Pruning

Removes fragment from population.

Does not alter fragment definition.

### 3.4 Compression <br>

$\kappa$ ( *f* ) = *f* ′

Constraints:

* Semantic equivalence preserved

* Fitness ordering preserved (see Section 4)

## 4. Fitness

Define fitness function:

*$\phi$* : *F* $\to$ $\mathcal{R}$

Properties:

* Comparable within shared context

* Deterministic under fixed evaluation conditions

* May be multi-objective (vector-valued extension allowed)

Optional extension:

*$\phi$* : *F* $\to$ $\mathcal{R}^n$

with Pareto ordering.

## 5. Robustness

Define perturbation distribution:

*$\delta$* ∼ $\mathcal{P}$

Robustness:

*R* ( *f* ) = *$E_ \delta$* ∼ *P* (*$\Phi$* ( *f* + *$\delta$*) − *$\phi$* ( *f* ))

where

* *f* + *$\delta$* denotes structural perturbation.

* Validity must hold post-perturbation.

Interpretation:

High robustness $\to$ small expected degradation.

## 6. Invariants

The following invariants MUST hold globally:

Validity Invariant

*f* $\in$ *F* $\Rightarrow$ *f* satisfies schema

Type Preservation Under Mutation

*T* (*$\mu$* ( *f* )) = *T* ( *f* )

Compression Semantic Equivalence

*$\phi$* ( *f* ) = *$\phi$* (*κ* ( *f* ))

Deterministic Evaluation

Under fixed context:

*$\phi$* ( *f* ) = constant

Additional invariants:

No operator may generate invalid fragments

Population evaluation budget is finite and conserved

## 7. Population Dynamics

Let *$P_t$* ⊂ *F* be the population at time *t*.

Selection probability:

*Pr*⁡ ( *$f_t$*+1) *$\alpha$* *$\phi$* ( *$f_t$*) *R* ( *$f_t$*)

Normalized form: 

$$Pr⁡(f_t+1) = \frac{\phi (f_t) R(f_t)}{∑_{f\in\mathcal{P}_t} \phi(f) R(f)}$$

## 8. Implementation Requirement

All core modules MUST:

* Enforce fragment validity

* Declare which operator they implement

* Not introduce new operator types without updating this document

## 9. Completion Criteria (v0.1)

Formalism v0.1 is complete when:

* Fragment schema enforces Section 1–2

* Evolutionary extractor maps strictly to operators in Section 3

* Fitness function signature matches Section 4

* Robustness experiments compute Section 5

## Status

Version: 0.1
State: Minimal Closed System
Purpose: Stabilize ontology prior to scaling infrastructure