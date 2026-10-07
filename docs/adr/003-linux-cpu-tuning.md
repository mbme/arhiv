# ADR-003: Scope Linux CPU tuning to the build distribution

- Status: Accepted
- Date: 2026-09-21

## Context

Compiler artifacts generated with `target-cpu=native` depend on the CPU features
of the machine that created them. Shared CI build caches and published binaries
need an explicit hardware contract instead of inheriting the builder's CPU.

Image previews and encryption benefit from optimized code generation, while CI
checks do not execute those production workloads. Locally compiled packages can
rely on the CPU that compiles and runs them.

## Decision

Default Cargo builds, including CI checks, target the portable x86-64 baseline.
The published Linux CLI is built explicitly for
`x86_64-unknown-linux-gnu` with `target-cpu=x86-64-v3`. Locally installed CLI
binaries and locally built Arch packages use `target-cpu=native`.

Windows and Android retain their existing target-specific build configuration.

## Consequences

- CI caches contain portable host artifacts and can be restored on different
  hosted runners.
- The published Linux CLI requires an x86-64-v3 CPU and may fail on older
  x86-64 hardware.
- Local installations and Arch packages can use the build machine's CPU
  features and are not portable across machines with different feature sets.
- The explicit Linux target changes the published CLI artifact path to
  `target/x86_64-unknown-linux-gnu/release/arhiv`.

## Alternatives considered

- Use `target-cpu=native` for all builds: rejected because shared CI caches and
  distributed binaries do not have the build machine's CPU contract.
- Use the x86-64 baseline for the published Linux CLI: rejected because the
  release policy intentionally accepts an x86-64-v3 minimum for greater runtime
  performance.
- Use x86-64-v3 for all local builds: rejected because local packages can use
  the build machine's full feature set.