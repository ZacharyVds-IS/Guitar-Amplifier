---
date: April 2026
title: "![arc42](images/arc42-logo.png) Programming Language Choice"
---

# Introduction and Goals {#section-introduction-and-goals}

## Requirements Overview {#_requirements_overview}

The backend language must support realtime audio processing for a guitar amplifier workflow.
- low-latency audio path for live input/output
- continuous stream processing without unstable pauses
- direct integration with OS audio drivers
- practical maintainability for a team with initially low Rust/C++ experience

## Quality Goals {#_quality_goals}

1. **Latency**: keep end-to-end delay acceptable for amplifier usage.
2. **Performance**: sustain realtime processing under continuous load.
3. **Stability**: avoid glitches caused by unpredictable runtime behavior.
4. **Safety**: reduce memory-related failure modes in native audio code.
5. **Maintainability**: keep the codebase evolvable while adding DSP features.

## Stakeholders {#_stakeholders}

| Role/Name      | contact                      | Expectations                                                                            |
|----------------|------------------------------|-----------------------------------------------------------------------------------------|
| Realtime audio | Internal                     | Choose a backend language that can achieve low latency and still workable for the team. |
| Latency        | N/A                          | Responsive audio and reliable amplifier controls during playback.                       |

# Architecture Constraints {#section-architecture-constraints}

- Low latency is a hard requirement for usable amplifier behavior.
- Language choice must support direct access to platform audio I/O.
- Team experience is limited in both Rust and C++, so learning curve matters.
- Solution must balance performance and development risk in an internship context.

# Context and Scope {#section-context-and-scope}

## Business Context {#_business_context}

**Language choice context (from research comparison)**

| Criterion | C++ | Go | Rust |
|---|---|---|---|
| Performance benchmark (100 mil loops) | 657.26 ms | 2132.87 ms | 654.54 ms |
| Audio input latency (team observation) | Minor latency | More latency than C++ | Similar to C++ |
| Learning curve | Steep | Easier | Steep/front-loaded |

## Technical Context {#_technical_context}

**Language-choice technical context**

| Technical element | Pre-development finding                                        |
|---|----------------------------------------------------------------|
| Performance | Rust is near C++ and faster than Go in the compared benchmark  |
| Latency suitability | Rust and C++ are better aligned with low-latency needs than Go |
| Runtime behavior | Rust and C++ have no runtime garbage collector pauses          |

**Mapping Input/Output to Channels**
- Input: live audio capture from input device
- Processing: gain and tone shaping in backend pipeline
- Output: processed signal to output device

# Solution Strategy {#section-solution-strategy}
Choose Rust for the backend audio processor and keep UI responsibilities outside the realtime path.

Why this strategy matches the provided evidence:

- Research benchmark places Rust near C++ and well ahead of Go for tested workload.
- Research notes Rust has no runtime/garbage collector and supports low-latency goals.
- Research identifies CPAL as Rust option for cross-platform audio driver access.
- Rust best fits the low-latency and safety goals while remaining practical for project learning goals.
- Rust is not the standard for DSP making it an interesting choice for the project and a good learning opportunity.

# Building Block View {#section-building-block-view}
## Whitebox Overall System {#_whitebox_overall_system}

# Deployment View {#section-deployment-view}
## Infrastructure Level 1 {#_infrastructure_level_1}
# Cross-cutting Concepts {#section-concepts}

## Realtime-first processing

- Audio processing decisions prioritize low and stable latency over convenience.
- Language and tooling are evaluated on predictability under continuous load.

## Safety as a default

- Native performance is required, but memory and concurrency safety are also primary concerns.
- The chosen language should reduce classes of runtime failures common in low-level audio software.

## Minimal runtime overhead

- The processing path should avoid runtime behaviors that can introduce unpredictable pauses.
- This concept directly supports the low-latency and stability goals.

## Cross-platform audio access

- The language ecosystem must support practical access to platform audio drivers.
- Cross-platform capability is treated as a baseline requirement, not a later optimization.

## Learning investment vs long-term maintainability

- A steeper early learning curve is acceptable when it leads to safer and more maintainable code.
- The language choice balances short-term onboarding cost with long-term project quality.


# Architecture Decisions {#section-design-decisions}

## Decision

Rust is selected for the backend realtime audio processor.

## Considered Alternatives

### Go
- Easier initial learning experience.
- Research data: benchmark result 2132.87 ms and observed higher audio latency in team tests.
- Research notes Go path used PortAudio binding layer for input handling in this context.

### C++
- Strong performance and established audio ecosystem.
- Research data: benchmark result 657.26 ms and low-latency suitability.
- Team context in research: steeper overall difficulty for current experience level.

### Rust (Chosen)
- Research data: benchmark result 654.54 ms and observed latency similar to C++.
- Research statement: Rust has no runtime or garbage collector.
- Research note: CPAL provides cross-platform access to OS audio drivers.
- Rust offers the best balance of latency fit and safer systems programming for this project.

## Rationale

Based on the research findings, Rust is the option that best matches the low-latency target while providing a safer native development model than manual-memory alternatives.

# Quality Requirements {#section-quality-scenarios}

## Quality Requirements Overview {#_quality_requirements_overview}

Highest priority quality attributes are latency and stability, followed by maintainability.

## Quality Scenarios {#_quality_scenarios}

1. Given live guitar input, when audio is processed in realtime, then perceived latency stays low enough for practical amplifier use.
2. Given continuous playback, when control values change, then output remains stable without audible interruption.
3. Given language and tooling constraints, when adding new processing features, then implementation remains maintainable by the project team.

# Risks and Technical Debts {#section-technical-risks}

- Rust learning curve can slow early delivery.
- Latency is also affected by hardware and driver configuration.
- Realtime safety depends on implementation practices, not language alone.
- Ongoing profiling and tuning remain necessary technical work.

# Glossary {#section-glossary}

| Term | Definition |
|---|---|
| Realtime audio | Audio processing where input-to-output delay must remain low and stable. |
| Latency | End-to-end delay from captured input to audible output. |
| DSP | Digital signal processing applied to audio samples. |
| CPAL | Rust audio crate used to create cross-platform input and output streams. |
