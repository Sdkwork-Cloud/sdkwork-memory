use crate::{MemoryImplementationKind, MemoryPluginManifest, MemorySpiError, MemorySpiResult};

#[derive(Debug, Default)]
pub struct MemoryPluginRegistry {
    manifests: Vec<MemoryPluginManifest>,
}

impl MemoryPluginRegistry {
    pub fn register(&mut self, manifest: MemoryPluginManifest) -> MemorySpiResult<()> {
        manifest.validate()?;

        if self
            .manifests
            .iter()
            .any(|existing| existing.plugin_id == manifest.plugin_id)
        {
            return Err(MemorySpiError::DuplicatePluginId(manifest.plugin_id));
        }

        self.manifests.push(manifest);
        Ok(())
    }

    pub fn get(&self, plugin_id: &str) -> Option<&MemoryPluginManifest> {
        self.manifests
            .iter()
            .find(|manifest| manifest.plugin_id == plugin_id)
    }

    pub fn plugins_for_implementation(
        &self,
        implementation_kind: MemoryImplementationKind,
    ) -> Vec<&MemoryPluginManifest> {
        self.manifests
            .iter()
            .filter(|manifest| manifest.implementation_kinds.contains(&implementation_kind))
            .collect()
    }

    pub fn validate_required_ports(
        &self,
        plugin_id: &str,
        required_ports: &[&str],
    ) -> MemorySpiResult<()> {
        let manifest = self
            .get(plugin_id)
            .ok_or_else(|| MemorySpiError::PluginNotFound(plugin_id.to_string()))?;

        for required_port in required_ports {
            let found = manifest
                .port_exports
                .iter()
                .any(|export| export.port == *required_port);

            if !found {
                return Err(MemorySpiError::RequiredPortMissing {
                    plugin_id: plugin_id.to_string(),
                    port: (*required_port).to_string(),
                });
            }
        }

        Ok(())
    }
}
