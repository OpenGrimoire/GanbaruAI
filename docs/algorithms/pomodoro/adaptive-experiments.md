# Adaptive Pomodoro experiments

The current adaptive engine contains seven bounded run-level experiment lanes. Each lane compares a control rhythm with one treatment under narrow eligibility. Assignment, context, selected values, outcomes, analysis, terminal state, and cooldown are persisted locally.

Stable objectives and shared guardrails are defined in [Adaptive policy](adaptive-policy.md). This document records the current experiment catalog so implementation and tests can remain discoverable without repeating the entire analysis pipeline in schema documentation.

## Rhythm notation

Compact rhythm notation uses:

focus minutes / short-break minutes / long-break minutes / long-break cadence

For example, 40/5/10/C4 means 40 minute focus, 5 minute short break, 10 minute long break, and a long break after four focus positions.

## Experiment catalog

| Lane | Control | Treatment | Narrow eligibility | Primary evidence | Main guardrails |
| --- | --- | --- | --- | --- | --- |
| Focus duration | 40 minute focus | 45 minute focus | Policy already selects capacity growth, comparable history exists, and strain, recovery debt, avoidance, blocker, and break-drift risk are low | Conservative clean-focus gain | Completion, stop rate, blocker pressure, missed planned work, next-day return |
| Short-break duration | 5 minutes | 7 minutes | Repeated low-risk short-break return drift without broader recovery or blocker pressure | Reduced short-break overtime | Completion, blockers, skipped breaks, missed planned work, next-day return, no worse drift |
| Long-break duration | 10 minutes | 15 minutes | Clean long-break return drift, low broader risk, and no blocked attempt during long-break overtime | Reduced long-break overtime | Completion, blockers, skipped breaks, missed planned work, next-day return, no worse drift |
| Earlier long-break cadence | C4 | C3 | Mild late-cycle recovery pressure after several focus positions without high strain, recovery debt, or avoidance | Reduced blocker pressure or conservative completion improvement | Clean focus, completion, skipped breaks, missed planned work, next-day return |
| Later long-break cadence | C4 | C5 | Very clean high-momentum work, at least 12 completed focus periods, completed breaks, no skipped breaks, blocked attempts, break drift, or focus failures, and enough comparable history | Conservative clean-focus gain | Completion, blockers, skipped breaks, long-break return, missed planned work, next-day return |
| Focus plus short-break bundle | 40/5/10/C4 | 45/7/10/C4 | Strong clean momentum, stable short-break return, substantial completed focus and break history, comparable evidence, and low risk | Conservative clean-focus gain | Completion, stop rate, blockers, skipped breaks, short-break drift, missed planned work, next-day return |
| Long-recovery bundle | 40/5/15/C4 | 40/5/15/C3 | Clean long-break drift, stable completed focus and break history, comparable evidence, no blocked attempt during overtime, and low risk | Reduced blocker pressure, reduced long-break drift, or conservative completion improvement | Clean focus, completion, stop rate, blockers, skipped breaks, missed planned work, next-day return |

## Shared eligibility exclusions

An experiment does not start when:

- adaptive mode is not explicitly enabled;
- the relevant lane is in cooldown;
- the run is recovery, guardrail, or low-confidence posture;
- context or history quality is insufficient;
- the treatment would exceed product or user-pinned bounds;
- another incompatible run-start experiment already owns the comparable short window;
- current configuration does not match the lane's supported control family;
- a required outcome cannot be observed without using disallowed data.

Eligibility is evaluated before deterministic assignment. Failing eligibility keeps the current safe rhythm and creates no fake control observation.

## Assignment lifecycle

1. Build a coarse context and policy snapshot from canonical local history.
2. Select the one eligible lane, if any, using deterministic policy priority.
3. Reuse an existing assignment for the same command or run identity.
4. Otherwise assign control or treatment from the persisted deterministic seed and exploration balance.
5. Commit assignment, run snapshot, chosen values, and first active segment atomically.
6. Attach later phase, run, same-day, and next-day outcomes only when each observation matures.
7. Aggregate by experiment, variant, and coarse context for analysis.
8. Persist terminal result and cooldown without creating a synthetic extra assignment.

A run participates in one run-start lane at a time. Component experiments and bundle experiments remain distinct evidence. A bundle result is not decomposed into causal claims about each component.

## Outcome maturity

Immediate phase outcomes can include completion, stop, focus failure, clean focus, break skip, break overtime, and blocker pressure.

Run outcomes attach after the run closes. Same-day missed planned work and blocker pressure attach only after the relevant local day can no longer change under the outcome rule. Next-day return attaches only after the following observation window passes.

Missing mature data remains unknown. A user who has not opened the app on the next day is interpreted only according to the explicit next-day observation rule, not automatically as treatment harm.

Outcome writers are idempotent. Re-running maturation updates or inserts the same logical observation rather than increasing sample size.

## Analysis contract

Both variants require a minimum observation count before preference. Exact-context paired evidence is preferred. Sparse paired context may borrow a small discounted prior from similar neighboring contexts, then broader evidence for the same lane, then global lane evidence.

Binary outcomes use conservative interval comparisons. Numeric outcomes use observation count, sum, and squared sum so variance affects confidence. Treatment preference requires meaningful primary improvement and preserved guardrails.

Any severe direct harm can stop treatment before ordinary confidence is reached. Conservative guardrail harm also selects control. Otherwise the result stays inconclusive.

The analyzer records which evidence tier supported the result. It does not silently combine unmatched contexts until a desired answer appears.

## Lane-specific result rules

### Focus duration

Treatment wins only with conservative clean-focus gain and preserved completion and next-day behavior. Completion decline, increased stops, blockers, missed work, or next-day avoidance returns to 40 minutes.

### Short break

Treatment wins only when 7 minutes materially reduces short-break overtime without increasing skipped breaks or broader risk. More allotted break time without improved return is not a win.

### Long break

Treatment wins only when 15 minutes reduces long-break overtime while preserving later completion and return. A longer break that produces still more drift or blocker pressure loses.

### Earlier long-break cadence

C3 wins when earlier recovery conservatively reduces late-cycle blocker pressure or improves completion without sacrificing clean focus. More frequent long breaks that increase skips, drift, missed work, or next-day avoidance lose.

### Later long-break cadence

C5 wins only when delayed long recovery increases clean focus in already stable high-momentum contexts without degrading return, blockers, skips, missed work, or next-day behavior.

### Focus plus short-break bundle

The 45/7 treatment must improve clean focus while its longer short break prevents the support cost from becoming drift. A harmful result applies to the combined shape, not automatically to 45 minute focus or 7 minute breaks tested separately.

### Long-recovery bundle

The C3 treatment inside the shared 15 minute long-break rhythm must reduce blocker pressure, long-break drift, or improve completion while preserving clean focus and all shared guardrails. Harm blocks only the combined 15 minute plus C3 shape.

## Terminal states and cooldown

Treatment preference records completed. Guardrail-forced control records abandoned. An inconclusive analysis remains active only while exploration budget and eligibility permit more observations.

Current terminal cooldown is 14 days per lane. Abandoned cooldown holds the control. Completed treatment may be selected in compatible contexts but does not become an unconditional global default.

Leaving adaptive mode stops new assignments. Historical observations remain interpretable. Returning later respects current policy version and cooldown rather than pretending the old experiment never happened.

## Deterministic replay

Diagnostics may replay policy and analysis from persisted snapshots to explain a prior assignment or compare a candidate policy version. Replay must:

- use the historical policy and experiment version unless explicitly evaluating a candidate version;
- read canonical assignments and outcomes without rewriting them;
- produce bounded reason codes, evidence tier, guardrails, and candidate result;
- avoid raw diary, Notes, Chat, or calendar-title content;
- distinguish a historical explanation from a hypothetical candidate result.

Replay is a debugging and evaluation tool. It does not retroactively change the rhythm that a historical run used.

## Required tests

Each lane protects:

- exact eligibility and every high-risk exclusion;
- deterministic assignment and command replay;
- control and treatment value snapshots;
- outcome maturity and idempotence;
- minimum samples in both arms;
- exact, neighboring, broader, and global evidence order;
- primary outcome improvement;
- every named guardrail and severe-harm stop;
- terminal result, 14-day cooldown, and abandoned control hold;
- independence of component and bundle results;
- versioned deterministic replay.
