use std::{collections::HashMap, env, error::Error, path::PathBuf};

use config::{Config, Environment as DeprecatedEnvironment, File};
use convert_case::{Case, Casing};
use eyre::{bail, Context, Result};
use itertools::Itertools;
use serde::Deserialize;

use super::deprecated_parser::DeprecatedRawSettings;
use crate::settings::loader::deprecated_arguments::DeprecatedCommandLineArguments;

mod arguments;
mod deprecated_arguments;
mod environment;

/// Load a settings object from the config locations.
/// Further documentation can be found in the `settings` module.
pub(crate) fn load_settings_object<'de, T, S>(
    agent_prefix: &str,
    ignore_prefixes: &[S],
) -> Result<T>
where
    T: Deserialize<'de> + AsMut<DeprecatedRawSettings>,
    S: AsRef<str>,
{
    // Derive additional prefix from agent name
    let prefix = format!("HYP_{}", agent_prefix).to_ascii_uppercase();

    let filtered_env: HashMap<String, String> = env::vars()
        .filter(|(k, _v)| {
            !ignore_prefixes
                .iter()
                .any(|prefix| k.starts_with(prefix.as_ref()))
        })
        .collect();

    let mut base_config_sources = vec![];
    let mut builder = Config::builder();

    // Always load the default config files (`rust/config/*.json`)
    for entry in PathBuf::from("./config")
        .read_dir()
        .expect("Failed to open config directory")
        .map(Result::unwrap)
    {
        if !entry.file_type().unwrap().is_file() {
            continue;
        }

        let fname = entry.file_name();
        let ext = fname.to_str().unwrap().split('.').last().unwrap_or("");
        if ext == "json" {
            base_config_sources.push(format!("{:?}", entry.path()));
            builder = builder.add_source(File::from(entry.path()));
        }
    }

    // Load a set of additional user specified config files
    let config_file_paths: Vec<String> = env::var("CONFIG_FILES")
        .map(|s| s.split(',').map(|s| s.to_owned()).collect())
        .unwrap_or_default();

    for path in &config_file_paths {
        let p = PathBuf::from(path);
        if p.is_file() {
            if p.extension() == Some("json".as_ref()) {
                builder = builder.add_source(File::from(p));
            } else {
                bail!("Provided config path via CONFIG_FILES is of an unsupported type ({p:?})")
            }
        } else if !p.exists() {
            bail!("Provided config path via CONFIG_FILES does not exist ({p:?})")
        } else {
            bail!("Provided config path via CONFIG_FILES is not a file ({p:?})")
        }
    }

    let config_deserializer = builder
        // Use a base configuration env variable prefix
        .add_source(
            DeprecatedEnvironment::with_prefix("HYP_BASE")
                .separator("_")
                .source(Some(filtered_env.clone())),
        )
        .add_source(
            DeprecatedEnvironment::with_prefix(&prefix)
                .separator("_")
                .source(Some(filtered_env)),
        )
        .add_source(DeprecatedCommandLineArguments::default().separator("."))
        .build()?;

    let formatted_config = {
        let f = format!("{config_deserializer:#?}");
        if env::var("ONELINE_BACKTRACES")
            .map(|v| v.to_lowercase())
            .as_deref()
            == Ok("true")
        {
            f.replace('\n', "\\n")
        } else {
            f
        }
    };

    match Config::try_deserialize::<T>(config_deserializer) {
        Ok(mut cfg) => {
            cfg.as_mut();
            Ok(cfg)
        }
        Err(err) => {
            let mut err = if let Some(source_err) = err.source() {
                let source = format!("Config error source: {source_err}");
                Err(err).context(source)
            } else {
                Err(err.into())
            };

            for cfg_path in base_config_sources.iter().chain(config_file_paths.iter()) {
                err = err.with_context(|| format!("Config loaded: {cfg_path}"));
            }

            println!("Error during deserialization, showing the config for debugging: {formatted_config}");

            err.context("Config deserialization error, please check the config reference (https://docs.hyperlane.xyz/docs/operators/agent-configuration/configuration-reference)")
        }
    }
}

/// Load a settings object from the config locations and re-join the components with the standard
/// `config` crate separator `.`.
fn split_and_recase_key(sep: &str, case: Option<Case>, key: String) -> String {
    if let Some(case) = case {
        // if case is given, replace case of each key component and separate them with `.`
        key.split(sep).map(|s| s.to_case(case)).join(".")
    } else if !sep.is_empty() && sep != "." {
        // Just standardize the separator to `.`
        key.replace(sep, ".")
    } else {
        // no changes needed if there was no separator defined and we are preserving case.
        key
    }
}
