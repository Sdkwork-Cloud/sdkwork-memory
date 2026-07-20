# SDKWork Memory PC Source Configuration

`sdkwork.deployment.config.json` delegates deployment topology and public origins to the repository-level Memory deployment authority. This PC renderer does not own a competing topology.

`browser.runtime.json` declares browser-safe binding names. The templates under `browser/` materialize exactly one active lifecycle environment and deployment profile into `/runtime-env.json`; they contain no tokens, API keys, database settings, or private service endpoints.

Local overrides matching `etc/**/*.local.*` stay untracked. Validate with:

```powershell
node ../../../sdkwork-specs/tools/check-source-config-standard.mjs --root .
```
