use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde::Deserialize;
use toml_edit::de;

// Keeping forc since some ways we handle forc is slightly different.
pub const FORC: &str = "forc";
pub const FUELUP: &str = "fuelup";

const COMPONENTS_TOML: &str = include_str!("../../components.toml");

#[derive(Debug, Deserialize)]
pub struct Components {
    pub component: HashMap<String, Component>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Component {
    pub name: String,
    pub is_plugin: Option<bool>,
    pub tarball_prefix: String,
    pub executables: Vec<String>,
    pub repository_name: String,
    pub targets: Vec<String>,
    pub publish: Option<bool>,
}

impl Component {
    pub fn from_name(name: &str) -> Result<Self> {
        if name == FUELUP {
            return Ok(Component {
                name: FUELUP.to_string(),
                tarball_prefix: FUELUP.to_string(),
                executables: vec![FUELUP.to_string()],
                repository_name: FUELUP.to_string(),
                targets: vec![FUELUP.to_string()],
                is_plugin: Some(false),
                publish: Some(true),
            });
        }

        let components = Components::collect().expect("Could not collect components");

        components
            .component
            .get(name)
            .ok_or_else(|| anyhow!("component with name '{}' does not exist", name))
            .and_then(|c| Ok(c.clone()))
    }
}

#[derive(Debug)]
pub struct Plugin {
    pub name: String,
    pub executables: Vec<String>,
}

impl Plugin {
    pub fn is_main_executable(&self) -> bool {
        self.executables.len() == 1 && self.name == self.executables[0]
    }
}

impl Components {
    pub fn from_toml(toml: &str) -> Result<Self> {
        let components: Components = de::from_str(toml)?;
        Ok(components)
    }

    pub fn collect() -> Result<Components> {
        let components = Self::from_toml(COMPONENTS_TOML)?;
        Ok(components)
    }

    pub fn contains_published(name: &str) -> bool {
        Self::collect_publishables()
            .expect("Failed to collect publishable components")
            .iter()
            .map(|c| c.name.clone())
            .collect::<String>()
            .contains(name)
    }

    pub fn collect_publishables() -> Result<Vec<Component>> {
        let components = Self::from_toml(COMPONENTS_TOML)?;

        let mut publishables: Vec<Component> = components
            .component
            .keys()
            .map(|c| {
                components
                    .component
                    .get(c)
                    .expect("Failed to parse components.toml")
            })
            .filter_map(|c| c.publish.and_then(|_| Some(c.clone())))
            .collect();

        publishables.sort_by_key(|c| c.name.clone());
        Ok(publishables)
    }

    pub fn collect_exclude_plugins() -> Result<Vec<Component>> {
        let components = Self::from_toml(COMPONENTS_TOML)?;

        let mut main_components: Vec<Component> = components
            .component
            .keys()
            .map(|c| {
                components
                    .component
                    .get(c)
                    .expect("Failed to parse components.toml")
            })
            .filter_map(|c| c.is_plugin.is_none().then(|| c.clone()))
            .collect();

        main_components.sort_by_key(|c| c.name.clone());

        Ok(main_components)
    }

    pub fn collect_plugins() -> Result<Vec<Plugin>> {
        let components = Self::from_toml(COMPONENTS_TOML)?;

        let mut plugins: Vec<Plugin> = components
            .component
            .keys()
            .map(|c| {
                components
                    .component
                    .get(c)
                    .expect("Failed to parse components.toml")
            })
            .filter(|&c| c.is_plugin.unwrap_or_default())
            .map(|p| Plugin {
                name: p.name.clone(),
                executables: p.executables.clone(),
            })
            .collect();
        plugins.sort_by_key(|p| p.name.clone());

        Ok(plugins)
    }

    pub fn collect_plugin_executables() -> Result<Vec<String>> {
        let plugins = Self::collect_plugins()?;
        let mut executables = vec![];

        for plugin in plugins.iter() {
            executables.extend(plugin.executables.clone().into_iter())
        }

        Ok(executables)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_toml() -> Result<()> {
        const TOML: &str = r#"
[component.forc-fmt]
name = "forc-fmt"
is_plugin = true
tarball_prefix = "forc-binaries"
executables = ["forc-fmt"]
repository_name = "sway"
targets = ["linux_amd64", "linux_arm64", "darwin_amd64", "darwin_arm64"]
"#;

        let components = Components::from_toml(TOML)?;

        assert_eq!(components.component["forc-fmt"].name, "forc-fmt");
        assert_eq!(components.component["forc-fmt"].is_plugin, Some(true));
        assert_eq!(
            components.component["forc-fmt"].tarball_prefix,
            "forc-binaries"
        );
        assert_eq!(components.component["forc-fmt"].executables, ["forc-fmt"]);
        assert_eq!(
            components.component["forc-fmt"].repository_name,
            "https://github.com/FuelLabs/sway"
        );
        assert_eq!(
            components.component["forc-fmt"].targets,
            ["linux_amd64", "linux_arm64", "darwin_amd64", "darwin_arm64"]
        );

        Ok(())
    }

    #[test]
    fn test_collect_exclude_plugins() -> Result<()> {
        let components = Components::collect_exclude_plugins().unwrap();
        let actual = components
            .iter()
            .map(|c| c.name.clone())
            .collect::<Vec<String>>();
        let mut expected = ["forc", "fuel-core"];
        expected.sort();
        assert_eq!(components.len(), 2);
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_collect_plugins() {
        assert!(Components::collect_plugins().is_ok());
    }

    #[test]
    fn test_collect_plugin_executables() {
        assert!(Components::collect_plugin_executables().is_ok());
    }
}