# SDKWork Memory Native SQL Plugin

This directory is the production-baseline runtime plugin for the Memory SPI.
It provides the canonical SQL-backed store ports used by the standalone and
cloud application deployment profiles without making embeddings a required
dependency.

The runtime manifest is [`sdkwork.memory.plugin.json`](sdkwork.memory.plugin.json).
The module integration contract is [`specs/component.spec.json`](specs/component.spec.json),
and the Rust package-root exports listed there are the only supported
composition boundary.

## Qualification

`native_sql` is the first production implementation family. Its component
contract permits the SDKWork `standalone` and `cloud` deployment profiles and
the `server` and `container` runtime targets. The plugin manifest's
`deploymentModes` describe plugin-level execution modes; they are not a third
SDKWork deployment profile.

## Verification

```powershell
cargo test -p sdkwork-memory-plugin-native-sql
node --test tests/contracts/runtime_plugin_layout_contract_test.mjs
```

