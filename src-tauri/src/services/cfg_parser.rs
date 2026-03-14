use std::io::{BufRead, Write};
use crate::models::cfg::{CfgEntry, CfgFile};

pub struct CfgParser;

impl CfgParser {
    pub fn parse<R: BufRead>(reader: R, file_path: String, lang_code: String) -> CfgFile {
        let mut cfg_file = CfgFile {
            file_path,
            language_code: lang_code,
            entries: Vec::new(),
            section_order: Vec::new(),
            header_comments: Vec::new(),
        };

        let mut current_section: Option<String> = None;
        let mut current_comments = Vec::new();
        let mut is_header = true;

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l.trim().to_string(),
                Err(_) => continue,
            };

            if line.is_empty() {
                continue;
            }

            // Comment line
            if line.starts_with(';') || line.starts_with('#') {
                current_comments.push(line);
                continue;
            }

            // Section header [section-name]
            if line.starts_with('[') && line.ends_with(']') {
                let section = line[1..line.len() - 1].to_string();
                if !cfg_file.section_order.contains(&section) {
                    cfg_file.section_order.push(section.clone());
                }
                current_section = Some(section);

                if is_header && !current_comments.is_empty() {
                    cfg_file.header_comments.extend(current_comments.drain(..));
                }
                is_header = false;
                continue;
            }

            // Key=Value pair
            if let Some(section) = &current_section {
                if let Some(pos) = line.find('=') {
                    is_header = false;
                    let key = line[..pos].trim().to_string();
                    let value = line[pos + 1..].trim().to_string();
                    
                    let comment = if !current_comments.is_empty() {
                        Some(current_comments.join("\n"))
                    } else {
                        None
                    };

                    cfg_file.entries.push(CfgEntry {
                        section: section.clone(),
                        key,
                        value,
                        comment,
                    });
                    current_comments.clear();
                }
            } else if is_header {
                // If we haven't hit a section yet, treat comments as header comments
                cfg_file.header_comments.extend(current_comments.drain(..));
            }
        }

        cfg_file
    }

    pub fn write<W: Write>(cfg_file: &CfgFile, mut writer: W) -> std::io::Result<()> {
        // Write header comments
        for comment in &cfg_file.header_comments {
            writeln!(writer, "{}", comment)?;
        }

        if !cfg_file.header_comments.is_empty() {
            writeln!(writer)?;
        }

        // Write by section
        for section in &cfg_file.section_order {
            writeln!(writer, "[{}]", section)?;

            for entry in &cfg_file.entries {
                if &entry.section == section {
                    if let Some(comment) = &entry.comment {
                        writeln!(writer, "{}", comment)?;
                    }
                    writeln!(writer, "{}={}", entry.key, entry.value)?;
                }
            }
            writeln!(writer)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_should_correctly_parse_cfg_content() {
        let cfg_content = "
; Header Comment
[item-name]
; Item Comment
iron-plate=鉄板
copper-plate=銅板

[entity-name]
small-biter=小型バイター
";
        let mut cursor = Cursor::new(cfg_content);
        let result = CfgParser::parse(&mut cursor, String::new(), String::new());

        assert_eq!(result.entries.len(), 3);
        assert!(result.section_order.contains(&"item-name".to_string()));
        assert!(result.section_order.contains(&"entity-name".to_string()));
        
        let iron_plate = result.entries.iter().find(|e| e.key == "iron-plate").unwrap();
        assert_eq!(iron_plate.value, "鉄板");
        assert_eq!(iron_plate.comment, Some("; Item Comment".to_string()));

        assert_eq!(result.header_comments.len(), 1);
        assert_eq!(result.header_comments[0], "; Header Comment");
    }

    #[test]
    fn test_write_should_preserve_structure() {
        let cfg_file = CfgFile {
            file_path: String::new(),
            language_code: String::new(),
            section_order: vec!["item-name".to_string()],
            header_comments: vec!["; Header".to_string()],
            entries: vec![
                CfgEntry { section: "item-name".to_string(), key: "key1".to_string(), value: "val1".to_string(), comment: Some("; Entry Comment".to_string()) }
            ],
        };
        let mut output = Vec::new();
        CfgParser::write(&cfg_file, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("; Header"));
        assert!(output_str.contains("[item-name]"));
        assert!(output_str.contains("; Entry Comment"));
        assert!(output_str.contains("key1=val1"));
    }
}
