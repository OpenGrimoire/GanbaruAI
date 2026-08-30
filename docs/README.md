# Documentation

Ganbaru AI documentation is organized by the kind of decision it records. Start here instead of opening the largest specification directly.

## Start here

| Area | Read this first | Purpose |
| --- | --- | --- |
| Product | [Product direction](product/README.md) | Product identity, principles, scope, and system ownership |
| Features | [Feature index](features/README.md) | Current capability status and user-facing feature contracts |
| Roadmap | [Roadmap](ROADMAP.md) | Remaining work, dependencies, and deferred areas |
| Architecture | [Architecture](architecture/README.md) | Current system boundaries, technology choices, and platform composition |
| Platforms | [Platforms](platforms/README.md) | Platform capabilities, permissions, native services, and release validation |
| Data | [Data index](data/README.md) | Sources of truth, schema domains, invariants, hazards, security, and sync |
| Algorithms | [Algorithm index](algorithms/README.md) | Stable pure-logic rules and worked examples |
| Interoperability | [Interoperability](interop/README.md) | Standards scope, preservation, conformance, and client behavior |
| Performance | [Performance](performance/README.md) | Measurement rules, benchmark harness, and recorded results |
| Testing | [Testing](testing/README.md) | Validation gates, test design, and resource constraints |
| Project operations | [Operations](operations/README.md) | Repository rules, releases, and release notes |

## Status language

Feature documentation can describe both shipped behavior and the intended end state. Use these labels so readers do not have to infer which is which:

- **Implemented:** present in the current repository and available on the stated platform.
- **Partial:** a useful slice exists, but the documented feature still has material gaps.
- **Planned:** accepted product direction with no complete user-facing implementation.
- **Deferred:** intentionally outside the active delivery sequence.
- **Reference:** a normative rule, decision, standard, or historical measurement. It is not a feature-status claim.

A feature's main document should state its current status near the beginning. Target behavior belongs in clearly named sections such as `Target behavior`, `Planned work`, or `Open decisions`.

## Where information belongs

- `product/` explains what Ganbaru AI is, why it exists, and which system owns each cross-feature responsibility.
- `features/` specifies user-facing behavior. Large domains have one `README.md` and a small set of focused supporting documents.
- `architecture/` describes current code boundaries and durable technology decisions. Exact dependency versions remain authoritative in manifests.
- `data/` describes sources of truth, durable storage, authorization, migration rules, and cross-domain invariants. Exact SQL remains authoritative in migrations.
- `algorithms/` contains logic that should remain understandable independently of UI and persistence code.
- `interop/` records external standards, preservation rules, fixtures, and client observations.
- `platforms/` records operating-system capabilities, lifecycle, permissions, native services, and acceptance requirements.
- `performance/` separates methodology from immutable recorded measurements.
- `testing/` defines validation gates, infrastructure, test design, and manual acceptance matrices.
- `operations/` contains contributor and maintainer procedures.

## Maintenance rules

Prefer durable intent over a tour of the current implementation. Record the behavior, boundary, invariant, or reason that future changes must preserve. Link to a source directory when ownership matters, but do not mirror every component, command, table column, or private helper in prose.

Split a document when it combines independent audiences or workflows, not only because it has many lines. A long conformance checklist or decision log can remain one file when splitting would make it harder to use. A feature document should usually split when product behavior, platform behavior, data details, and validation procedures have become separate references.

Avoid repeating the same contract in multiple places. Choose one normative document and link to it. In particular:

- Feature docs own user behavior.
- Data docs own persistence and authorization rules.
- Algorithm docs own pure decision rules.
- The roadmap owns delivery order, not detailed feature design.
- Source code, manifests, migrations, and generated platform projects own exact implementation shape.

When a current-state claim changes, update the feature index, the domain's main document, and the roadmap only if the delivery sequence also changed. Recorded benchmark rows and dated architecture decisions are historical records and must not be rewritten to resemble the present.
