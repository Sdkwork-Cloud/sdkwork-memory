> Migrated from `docs/superpowers/plans/2026-06-10-memory-implementation-family-baseline.md` on 2026-06-24.
> Owner: SDKWork maintainers

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move all SDKWork Memory implementation families from design-only declarations to executable, test-covered baseline profiles.

**Architecture:** Keep `native_sql` and `local_embedded` in `sdkwork-memory-plugin-native-sql`, because the existing plugin already owns the SQLite/PostgreSQL phase 1 store. Add one provider-neutral reference plugin for `event_sourced`, `search_first`, `graph_temporal`, `external_provider_bridge`, and `hybrid_platform`; it exposes explicit builders and fail-closed/provider-neutral ports without binding a graph/search/external vendor.

**Tech Stack:** Rust 2021, Cargo workspace crates, `sdkwork-memory-spi`, `sdkwork-memory-runtime`, JSON plugin manifests, contract tests, Node phase 1 verification.

---

### Task 1: Add Family Coverage Tests

**Files:**
- Modify: `crates/sdkwork-memory-spi/tests/manifest_contract.rs`
- Modify: `crates/sdkwork-memory-spi/tests/registry_contract.rs`
- Modify: `crates/sdkwork-memory-runtime/tests/profile_resolution_contract.rs`

- [ ] Write failing tests that require every implementation family to have a manifest/profile path.
- [ ] Run focused tests and confirm they fail because non-native profiles are missing.

### Task 2: Implement Reference Profile Manifests

**Files:**
- Modify: `Cargo.toml`
- Create: `plugins/sdkwork-memory-plugin-reference-profiles/Cargo.toml`
- Create: `plugins/sdkwork-memory-plugin-reference-profiles/sdkwork.memory.plugin.json`
- Create: `plugins/sdkwork-memory-plugin-reference-profiles/src/lib.rs`
- Create: `plugins/sdkwork-memory-plugin-reference-profiles/src/manifest.rs`
- Create: `plugins/sdkwork-memory-plugin-reference-profiles/tests/manifest_matches_json.rs`
- Modify: `crates/sdkwork-memory-spi/src/manifest.rs`

- [ ] Add manifest constructors for reference profile support.
- [ ] Add a workspace member for the reference profile plugin.
- [ ] Ensure the JSON manifest and Rust manifest match.
- [ ] Run plugin and SPI manifest tests.

### Task 3: Implement Runtime Profiles

**Files:**
- Modify: `crates/sdkwork-memory-runtime/src/profile.rs`
- Modify: `crates/sdkwork-memory-runtime/tests/profile_resolution_contract.rs`

- [ ] Add `local_embedded_phase1`, `event_sourced_phase1`, `search_first_phase1`, `graph_temporal_phase1`, `external_provider_bridge_eval`, and `hybrid_platform_phase1` profile constructors.
- [ ] Ensure each profile validates supported implementation kinds and required ports before serving.
- [ ] Run runtime profile tests.

### Task 4: Tighten Conformance Status

**Files:**
- Modify: `crates/sdkwork-memory-test-support/src/lib.rs`
- Modify: `crates/sdkwork-memory-test-support/tests/conformance_harness_contract.rs`
- Modify: `plugins/sdkwork-memory-plugin-native-sql/sdkwork.memory.plugin.json`
- Modify: `crates/sdkwork-memory-spi/src/manifest.rs`

- [ ] Replace blanket pending native SQL checks with manifest-capability-aware checks.
- [ ] Stop advertising native SQL candidate/habit behavior as implemented until executable ports exist.
- [ ] Add tests proving reference profiles are baseline/eval-only where appropriate.

### Task 5: Verify End To End

**Files:**
- Rust workspace and phase 1 contract artifacts.

- [ ] Run `cargo fmt`.
- [ ] Run `cargo test --workspace`.
- [ ] Run `node tools\materialize_phase1_contracts.mjs`.
- [ ] Run `powershell -ExecutionPolicy Bypass -File tools\verify_phase1.ps1`.
- [ ] Report exact evidence and remaining production gaps.

