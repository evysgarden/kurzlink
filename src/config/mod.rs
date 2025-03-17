use anyhow::Context;
use minijinja::Environment;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    config::{
        network::Network, shortlink::Shortlink, tag::Tag, templating::write_html, url::AbsoluteUrl,
    },
    error::ValidationError,
    utils::find_duplicates,
};

use self::templating::TEMPLATE;

mod network;
mod shortlink;
mod tag;
mod templating;
pub mod url;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub shortlinks: Vec<Shortlink>,
    pub tags: HashMap<String, Tag>,
    pub network: Network,
    pub index: Option<AbsoluteUrl>,
    pub files: Option<Vec<String>>,
}

impl Config {
    pub fn new(config_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let mut config: Config = serde_yaml::from_str(
            &fs::read_to_string(&config_path).with_context(|| "config not found".to_string())?,
        )?;
        if let Some(ref additional_paths) = config.files {
            let base_path = config_path.as_ref().parent().unwrap();
            let actual_paths: Vec<PathBuf> = additional_paths
                .iter()
                .map(|path_str| {
                    let path = PathBuf::from(&path_str);
                    if path.is_absolute() {
                        path
                    } else {
                        base_path.join(path)
                    }
                })
                .collect();
            for path in actual_paths {
                config
                    .shortlinks
                    .append(&mut serde_yaml::from_str(&fs::read_to_string(&path)?)?);
            }
        }
        Ok(config)
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        self.check_duplicates()?;
        // self.check_links()?;
        Ok(())
    }

    fn check_duplicates(&self) -> anyhow::Result<()> {
        if let Some(duplicates) = find_duplicates(self.shortlinks.iter().flat_map(|v| &v.sources)) {
            return Err(ValidationError::DuplicateSources(duplicates).into());
        }

        if let Some(duplicates) = find_duplicates(self.shortlinks.iter().map(|v| &v.destination)) {
            return Err(ValidationError::DuplicateDestinations(duplicates).into());
        }

        Ok(())
    }

    pub fn render_files(&self, output_path: impl AsRef<Path>) -> anyhow::Result<()> {
        if !output_path.as_ref().exists() {
            fs::create_dir(&output_path).with_context(|| "Couldn't create output dir")?;
        }

        let mut env = Environment::new();

        env.add_template("redirect", TEMPLATE)?;
        let template = env.get_template("redirect")?;

        if let Some(index) = &self.index {
            let index_link = Shortlink {
                check: None,
                description: None,
                destination: index.clone(),
                sources: Vec::new(),
                tags: None,
            };
            write_html(
                &output_path,
                &index_link.checked_html(template, &self.network)?,
            )?;
        }

        for shortlink in &self.shortlinks {
            let target_render = shortlink.checked_html(template, &self.network)?;
            for source in &shortlink.sources {
                write_html(output_path.as_ref().join(source.inner()), &target_render)?;
            }
        }

        Ok(())
    }

    pub fn write_vanity(&self, vanity_path: impl AsRef<Path>) -> anyhow::Result<()> {
        let vanity = json!({
            "index": &self.index,
            "shortlinks": &self.shortlinks,
            "tags": &self.tags
        })
        .to_string();

        fs::write(vanity_path, vanity)?;

        Ok(())
    }

    pub fn write_redirect_list(&self, rewrite_path: impl AsRef<Path>) -> anyhow::Result<()> {
        let mut file_content = String::from("");

        if let Some(index) = &self.index {
            file_content += format!("/ {} 302\n", index.inner()).as_str();
        }

        for shortlink in &self.shortlinks {
            for link in shortlink.sources.iter() {
                file_content +=
                    format!("/{} {} 302\n", link.inner(), shortlink.destination.inner()).as_str();
            }
        }

        fs::write(rewrite_path, file_content)?;

        Ok(())
    }
}
