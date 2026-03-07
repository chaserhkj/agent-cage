use std::path::{Path, PathBuf};

use figment::{
    Error, Metadata, Profile, Provider,
    providers::{Data, Format, Yaml},
    value::{Dict, Map, Value},
};

use delegate::delegate;

/// This figment Provider wraps the Yaml Provider, replacing all "!REL!" occurrences in
/// yaml fields with the directory path that the yaml file is in, allowing for specifying
/// relative paths in the hierarchy
pub struct YamlWithRel {
    wrapped: Data<Yaml>,
    folder_path: Option<PathBuf>,
}

impl YamlWithRel {
    pub fn new(yaml_path: &Path) -> Self {
        Self {
            wrapped: Yaml::file(yaml_path),
            folder_path: yaml_path.parent().map(|p| p.to_path_buf()),
        }
    }
}

fn replace_rel_placeholder(value: Value, path: &Path) -> Value {
    let Some(path_str) = path.to_str() else {
        return value;
    };
    match value {
        Value::String(tag, st) => Value::String(tag, st.replace("!REL!", path_str)),
        Value::Array(tag, v) => Value::Array(
            tag,
            v.into_iter()
                .map(|v| replace_rel_placeholder(v, path))
                .collect(),
        ),
        Value::Dict(tag, dict) => Value::Dict(
            tag,
            dict.into_iter()
                .map(|(k, v)| (k, replace_rel_placeholder(v, path)))
                .collect(),
        ),
        _ => value,
    }
}

impl Provider for YamlWithRel {
    delegate! {
        to self.wrapped {
            fn metadata(&self) -> Metadata;
        }
    }
    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        let wrapped_data = self.wrapped.data()?;
        let dir = self
            .folder_path
            .as_ref()
            .ok_or(Error::from("Unable to resolve contextual folder path"))?;
        Ok(wrapped_data
            .into_iter()
            .map(|(profile, dict)| {
                (
                    profile,
                    dict.into_iter()
                        .map(|(k, v)| (k, replace_rel_placeholder(v, dir)))
                        .collect(),
                )
            })
            .collect())
    }
}
