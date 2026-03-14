use std::fs;
use std::io::Read;
use std::path::Path;
use serde::Deserialize;
use crate::models::mod_info::ModInfo;
use crate::models::enums::ModSourceType;
use crate::services::cfg_parser::CfgParser;
use log::info;

#[derive(Debug, Deserialize)]
struct InfoJson {
    name: Option<String>,
    version: Option<String>,
    title: Option<String>,
    author: Option<String>,
    factorio_version: Option<String>,
}

pub struct ModLoader;

impl ModLoader {
    pub fn load_from_folder(path: &str) -> Result<ModInfo, String> {
        info!("ModLoader: Loading from folder: {}", path);
        let root_path = Path::new(path);
        if !root_path.exists() || !root_path.is_dir() {
            return Err(format!("Directory not found: {}", path));
        }

        let mut mod_info = ModInfo {
            name: String::new(),
            version: String::new(),
            title: String::new(),
            author: String::new(),
            source_path: path.to_string(),
            source_type: ModSourceType::Folder,
            factorio_version: None,
            locale_files: Vec::new(),
        };

        // Read info.json
        let info_path = root_path.join("info.json");
        if info_path.exists() {
            if let Ok(content) = fs::read_to_string(info_path) {
                Self::populate_metadata(&mut mod_info, &content);
            }
        }

        // Find locale folder
        let locale_path = root_path.join("locale");
        if locale_path.exists() && locale_path.is_dir() {
            if let Ok(entries) = fs::read_dir(locale_path) {
                for entry in entries.flatten() {
                    let lang_dir = entry.path();
                    if lang_dir.is_dir() {
                        let lang_code = lang_dir.file_name().unwrap_or_default().to_string_lossy().to_string();
                        if let Ok(files) = fs::read_dir(&lang_dir) {
                            for file_entry in files.flatten() {
                                let cfg_path = file_entry.path();
                                if cfg_path.extension().and_then(|s| s.to_str()) == Some("cfg") {
                                    if let Ok(file) = fs::File::open(&cfg_path) {
                                        let reader = std::io::BufReader::new(file);
                                        let cfg_file = CfgParser::parse(
                                            reader,
                                            cfg_path.to_string_lossy().to_string(),
                                            lang_code.clone(),
                                        );
                                        mod_info.locale_files.push(cfg_file);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(mod_info)
    }

    pub fn load_from_zip(zip_path: &str) -> Result<ModInfo, String> {
        info!("ModLoader: Loading from zip: {}", zip_path);
        let path = Path::new(zip_path);
        if !path.exists() {
            return Err(format!("Zip file not found: {}", zip_path));
        }

        let file = fs::File::open(path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

        let mut mod_info = ModInfo {
            name: String::new(),
            version: String::new(),
            title: String::new(),
            author: String::new(),
            source_path: zip_path.to_string(),
            source_type: ModSourceType::Zip,
            factorio_version: None,
            locale_files: Vec::new(),
        };

        if archive.len() == 0 {
            return Err("Zip is empty".to_string());
        }

        // Factorio mods in zip usually have a top-level folder: modname_version/
        let first_entry_name = archive.by_index(0).map_err(|e| e.to_string())?.name().to_string();
        let root_folder = first_entry_name.split('/').next().unwrap_or_default().to_string() + "/";

        // metadata
        let info_name = format!("{}info.json", root_folder);
        if let Ok(mut info_entry) = archive.by_name(&info_name) {
            let mut content = String::new();
            if info_entry.read_to_string(&mut content).is_ok() {
                Self::populate_metadata(&mut mod_info, &content);
            }
        }

        // locale
        let mut locale_entries = Vec::new();
        for i in 0..archive.len() {
            if let Ok(entry) = archive.by_index(i) {
                let name = entry.name().to_string();
                if name.starts_with(&format!("{}locale/", root_folder)) && name.ends_with(".cfg") {
                    locale_entries.push(name);
                }
            }
        }

        for entry_name in locale_entries {
            if let Ok(mut entry) = archive.by_name(&entry_name) {
                let parts: Vec<&str> = entry_name.split('/').collect();
                if parts.len() >= 4 {
                    let lang_code = parts[parts.len() - 2].to_string();
                    let mut content = Vec::new();
                    if entry.read_to_end(&mut content).is_ok() {
                        let reader = std::io::BufReader::new(&content[..]);
                        let cfg_file = CfgParser::parse(reader, entry_name.clone(), lang_code);
                        mod_info.locale_files.push(cfg_file);
                    }
                }
            }
        }

        Ok(mod_info)
    }

    fn populate_metadata(mod_info: &mut ModInfo, json_content: &str) {
        if let Ok(info) = serde_json::from_str::<InfoJson>(json_content) {
            if let Some(n) = info.name { mod_info.name = n; }
            if let Some(v) = info.version { mod_info.version = v; }
            if let Some(t) = info.title { mod_info.title = t; }
            if let Some(a) = info.author { mod_info.author = a; }
            mod_info.factorio_version = info.factorio_version;
        }
    }
}
