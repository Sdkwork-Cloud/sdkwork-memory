use std::collections::BTreeMap;

use crate::{
    MemoryExecutablePluginRuntime, MemoryImplementationKind, MemoryPluginManifest, MemorySpiError,
    MemorySpiResult,
};

#[derive(Debug, Default)]
pub struct MemoryPluginRegistry {
    manifests: Vec<MemoryPluginManifest>,
    executable_runtimes: BTreeMap<String, MemoryExecutablePluginRuntime>,
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

    pub fn register_executable(
        &mut self,
        manifest: MemoryPluginManifest,
        runtime: MemoryExecutablePluginRuntime,
    ) -> MemorySpiResult<()> {
        Self::validate_executable_ports_are_declared(&manifest, &runtime)?;
        let plugin_id = manifest.plugin_id.clone();
        self.register(manifest)?;
        self.executable_runtimes.insert(plugin_id, runtime);
        Ok(())
    }

    pub fn register_executable_runtime(
        &mut self,
        plugin_id: &str,
        runtime: MemoryExecutablePluginRuntime,
    ) -> MemorySpiResult<()> {
        let manifest = self
            .get(plugin_id)
            .ok_or_else(|| MemorySpiError::PluginNotFound(plugin_id.to_string()))?;
        Self::validate_executable_ports_are_declared(manifest, &runtime)?;
        if self.executable_runtimes.contains_key(plugin_id) {
            return Err(MemorySpiError::DuplicateExecutableRuntime(
                plugin_id.to_string(),
            ));
        }
        self.executable_runtimes
            .insert(plugin_id.to_string(), runtime);
        Ok(())
    }

    pub fn get(&self, plugin_id: &str) -> Option<&MemoryPluginManifest> {
        self.manifests
            .iter()
            .find(|manifest| manifest.plugin_id == plugin_id)
    }

    pub fn executable_runtime(&self, plugin_id: &str) -> Option<&MemoryExecutablePluginRuntime> {
        self.executable_runtimes.get(plugin_id)
    }

    pub fn require_executable_runtime(
        &self,
        plugin_id: &str,
    ) -> MemorySpiResult<&MemoryExecutablePluginRuntime> {
        self.executable_runtime(plugin_id)
            .ok_or_else(|| MemorySpiError::ExecutableRuntimeMissing {
                plugin_id: plugin_id.to_string(),
            })
    }

    pub fn has_executable_runtime(&self, plugin_id: &str) -> bool {
        self.executable_runtimes.contains_key(plugin_id)
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

    fn validate_executable_ports_are_declared(
        manifest: &MemoryPluginManifest,
        runtime: &MemoryExecutablePluginRuntime,
    ) -> MemorySpiResult<()> {
        for port in runtime.ports().port_names() {
            if !manifest
                .port_exports
                .iter()
                .any(|export| export.port == port)
            {
                return Err(MemorySpiError::ExecutablePortUndeclared {
                    plugin_id: manifest.plugin_id.clone(),
                    port: port.to_string(),
                });
            }
        }
        Ok(())
    }
}
