# SDKWork Memory Source Configuration

`sdkwork.deployment.config.json` is the source-controlled profile index for SDKWork Memory. It selects one profile under `topology/`; the topology contract is `../specs/topology.spec.json`.

Supported profiles are `standalone.development`, `standalone.production`, `cloud.development`, and `cloud.production`. Standalone development owns the local unified gateway. Cloud development starts no local service and consumes explicit deployed development URLs.

The cloud gateway TOML files are gateway composition handoff templates. Production secrets, database credentials, signing material, tokens, and local overrides are injected by the deployment platform and are never committed under `etc/`.

Validate with:

```powershell
node ../sdkwork-specs/tools/check-source-config-standard.mjs --root .
pnpm topology:validate
```
