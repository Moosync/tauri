use std::{collections::HashMap, fs, path::PathBuf, str::FromStr};

use fs_extra::dir::CopyOptions;
use regex::Regex;
use types::{
    errors::{MoosyncError, Result},
    themes::ThemeDetails,
};

#[derive(Debug)]
pub struct ThemeHolder {
    pub theme_dir: PathBuf,
    pub tmp_dir: PathBuf,
}

impl ThemeHolder {
    #[tracing::instrument(level = "trace", skip(theme_dir, tmp_dir))]
    pub fn new(theme_dir: PathBuf, tmp_dir: PathBuf) -> Self {
        Self { theme_dir, tmp_dir }
    }

    #[tracing::instrument(level = "trace", skip(self, theme))]
    pub fn save_theme(&self, theme: ThemeDetails) -> Result<()> {
        let theme_path = self.theme_dir.join(theme.id.clone());

        if !theme_path.exists() {
            fs::create_dir_all(&theme_path)?;
        }
        let theme_config = theme_path.join("config.json");
        fs::write(theme_config, serde_json::to_string(&theme)?)?;

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, id))]
    pub fn remove_theme(&self, id: String) -> Result<()> {
        let theme_path = self.theme_dir.join(id.clone());
        if theme_path.exists() {
            fs::remove_dir_all(&theme_path)?;
        }

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, id))]
    pub fn load_theme(&self, id: String) -> Result<ThemeDetails> {
        if id == "default" {
            return Ok(ThemeDetails::default());
        }
        let theme_config = self.theme_dir.join(id.clone()).join("config.json");
        if theme_config.exists() {
            let data = fs::read_to_string(theme_config)?;
            return Ok(serde_json::from_str(&data)?);
        }

        Err(MoosyncError::String("Theme not found".to_string()))
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn load_all_themes(&self) -> Result<HashMap<String, ThemeDetails>> {
        let theme_dir = self.theme_dir.clone();
        let entries = fs::read_dir(theme_dir)?;
        let mut ret = HashMap::new();
        ret.insert("default".into(), ThemeDetails::default());
        for theme_dir in entries.flatten() {
            if theme_dir.path().is_dir() {
                let id = theme_dir.file_name().to_str().unwrap().to_string();
                let theme = self.load_theme(id.clone())?;
                ret.insert(id, theme);
            }
        }

        Ok(ret)
    }

    #[tracing::instrument(level = "trace", skip(self, css_path, root))]
    pub fn transform_css(&self, css_path: String, root: Option<String>) -> Result<String> {
        let parsed_path = if let Some(root) = root {
            PathBuf::from(root).join(css_path)
        } else {
            PathBuf::from(css_path)
        };

        if !parsed_path.exists() {
            return Err(MoosyncError::String("CSS path does not exist".to_string()));
        }

        let mut css = fs::read_to_string(parsed_path.clone())?;
        let import_regex = Regex::new(r"@import\s(.*)").unwrap();
        let cloned_css = css.clone();
        let matches = import_regex.captures_iter(cloned_css.as_str());
        for mat in matches {
            let path = mat.get(1);
            if let Some(path) = path {
                let path = path.as_str().replace('"', "").as_str().to_string();
                let transformed_css = self.transform_css(
                    path,
                    parsed_path
                        .parent()
                        .map(|v| v.as_os_str().to_string_lossy().to_string()),
                )?;

                css = css.replace(mat.get(0).unwrap().as_str(), transformed_css.as_str());
            }
        }

        let theme_dir = parsed_path.parent().unwrap();
        css = css.replace("%themeDir%", theme_dir.to_str().unwrap());

        Ok(css)
    }

    #[tracing::instrument(level = "trace", skip(self, theme_path))]
    pub fn import_theme(&self, theme_path: String) -> Result<()> {
        let extract_dir = self
            .tmp_dir
            .join(format!("moosync_theme_{}", uuid::Uuid::new_v4()));

        let theme_path = PathBuf::from_str(&theme_path).unwrap();
        zip_extensions::zip_extract(&theme_path, &extract_dir.clone())?;

        for item in extract_dir.read_dir()? {
            if item.is_ok() {
                let item = item.unwrap().path();
                if item.is_file() && item.file_name().unwrap().to_string_lossy() == "config.json" {
                    let config = fs::read(item)?;
                    let parsed: ThemeDetails = serde_json::from_slice(config.as_slice())?;
                    let final_theme_path = self.theme_dir.join(parsed.id);
                    let options = CopyOptions::default().overwrite(true);

                    fs::create_dir_all(final_theme_path.clone())?;

                    let mut item_list = vec![];
                    for items in extract_dir.read_dir()? {
                        item_list.push(items.unwrap().path());
                    }
                    tracing::info!("Moving from {:?} to {:?}", item_list, final_theme_path);
                    fs_extra::move_items(item_list.as_slice(), final_theme_path, &options)?;

                    return Ok(());
                }
            }
        }
        Err(MoosyncError::String("Failed to parse theme".to_string()))
    }

    #[tracing::instrument(level = "trace", skip(self, id, export_path))]
    pub fn export_theme(&self, id: String, export_path: PathBuf) -> Result<()> {
        let mut theme = self.load_theme(id.clone())?;
        let theme_dir = self.tmp_dir.join(format!("theme-unpacked-{}", id));
        if !theme_dir.exists() {
            fs::create_dir_all(theme_dir.clone())?;
        }

        let config_path = theme_dir.clone().join("config.json");

        if let Some(custom_css) = theme.theme.custom_css {
            let transformed = self.transform_css(custom_css, None)?;
            let custom_css_path = theme_dir.join("custom.css");
            fs::write(custom_css_path.clone(), transformed)?;
            theme.theme.custom_css = Some(custom_css_path.to_string_lossy().to_string());
        }

        fs::write(config_path.clone(), serde_json::to_string_pretty(&theme)?)?;

        zip_extensions::zip_create_from_directory(&export_path, &theme_dir)?;

        if let Some(custom_css_path) = theme.theme.custom_css {
            fs::remove_file(custom_css_path)?;
        }
        fs::remove_file(config_path)?;
        fs::remove_dir(theme_dir)?;

        Ok(())
    }
}
