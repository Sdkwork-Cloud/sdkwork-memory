# SDKWork Memory Reference Profiles Plugin

This plugin contains deterministic, provider-neutral reference runtimes for
SPI conformance and evaluation. It is an executable test fixture, not a
production storage implementation or an approved external-provider bridge.

The runtime manifest is [`sdkwork.memory.plugin.json`](sdkwork.memory.plugin.json).
The module integration contract is [`specs/component.spec.json`](specs/component.spec.json).
The external bridge intentionally fails closed until a reviewed provider
adapter is configured.

## Qualification

This plugin is **evaluation-only**. It may run in explicit `test` or
`eval_only` plugin modes through the `test-runner` target, but it is not
eligible for the SDKWork `standalone` or `cloud` deployment profiles and must
never be selected as a production implementation profile.

## Verification

```powershell
cargo test -p sdkwork-memory-plugin-reference-profiles
node --test tests/contracts/runtime_plugin_layout_contract_test.mjs
```

