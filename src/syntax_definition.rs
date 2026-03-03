use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;
use serde::Deserialize;

#[derive(Deserialize)]
struct SyntaxDefinitionFile {
    name: String,
    file_extensions: Vec<String>,
    rules: Vec<RuleEntry>,
}

#[derive(Deserialize)]
struct RuleEntry {
    scope: String,
    #[serde(default)]
    pattern: Option<String>,
    #[serde(default)]
    begin: Option<String>,
    #[serde(default)]
    end: Option<String>,
}

pub enum CompiledRule {
    Pattern {
        scope: String,
        regex: Regex,
    },
    Span {
        scope: String,
        begin_regex: Regex,
        end_regex: Regex,
    },
}

pub struct SyntaxDefinition {
    pub name: String,
    pub file_extensions: Vec<String>,
    pub rules: Vec<CompiledRule>,
}

impl SyntaxDefinition {
    pub fn load(path: &Path) -> Option<Self> {
        let text = fs::read_to_string(path).ok()?;
        let file: SyntaxDefinitionFile = serde_json::from_str(&text).ok()?;

        let mut rules = Vec::new();
        for entry in &file.rules {
            if let Some(pattern) = &entry.pattern {
                if let Ok(regex) = Regex::new(pattern) {
                    rules.push(CompiledRule::Pattern {
                        scope: entry.scope.clone(),
                        regex,
                    });
                }
            } else if let (Some(begin), Some(end)) = (&entry.begin, &entry.end) {
                if let (Ok(begin_regex), Ok(end_regex)) = (Regex::new(begin), Regex::new(end)) {
                    rules.push(CompiledRule::Span {
                        scope: entry.scope.clone(),
                        begin_regex,
                        end_regex,
                    });
                }
            }
        }

        Some(SyntaxDefinition {
            name: file.name,
            file_extensions: file.file_extensions,
            rules,
        })
    }

    pub fn find_for_extension(extension: &str) -> Option<Self> {
        let languages_directory = find_languages_directory()?;

        let entries = fs::read_dir(&languages_directory).ok()?;
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let syntax_path = entry.path().join("syntax.json");
            if syntax_path.exists() {
                if let Some(definition) = Self::load(&syntax_path) {
                    if definition
                        .file_extensions
                        .iter()
                        .any(|ext| ext == extension)
                    {
                        return Some(definition);
                    }
                }
            }
        }

        None
    }
}

fn find_languages_directory() -> Option<PathBuf> {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_directory) = exe_path.parent() {
            let candidate = exe_directory.join("languages");
            if candidate.is_dir() {
                return Some(candidate);
            }
            // Two levels up handles the cargo target/debug/ layout
            if let Some(project_root) = exe_directory.parent().and_then(|p| p.parent()) {
                let candidate = project_root.join("languages");
                if candidate.is_dir() {
                    return Some(candidate);
                }
            }
        }
    }

    let candidate = PathBuf::from("languages");
    if candidate.is_dir() {
        return Some(candidate);
    }

    None
}
