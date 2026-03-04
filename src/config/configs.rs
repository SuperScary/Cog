use ini::Ini;

pub fn load_user_settings() -> Ini {
    Ini::load_from_file("settings/user_settings.ini").unwrap_or_else(|_| write_user_settings())
}

fn write_user_settings() -> Ini {
    let mut config = Ini::new();
    config.set_to(None::<String>, "encoding".to_string(), "UTF-8".to_string());

    config.write_to_file("settings/user_settings.ini").unwrap();
    config
}

pub fn load_editor_settings() -> Ini {
    Ini::load_from_file("settings/editor_settings.ini").unwrap_or_else(|_| write_editor_settings())
}

fn write_editor_settings() -> Ini {
    let mut config = Ini::new();
    config.set_to(None::<String>, "encoding".to_string(), "UTF-8".to_string());

    config.set_to(Some("editor"), "show_line_numbers".to_string(), true.to_string());
    config.set_to(Some("editor"), "show_gutter".to_string(), true.to_string());
    config.set_to(Some("editor"), "show_cursor".to_string(), true.to_string());
    config.set_to(Some("editor"), "tab_size".to_string(), 4.to_string());

    config.write_to_file("settings/editor_settings.ini").unwrap();
    config
}

pub fn load_color_scheme() -> Ini {
    Ini::load_from_file("settings/color_scheme.ini").unwrap_or_else(|_| write_color_scheme())
}

fn write_color_scheme() -> Ini {
    let mut config = Ini::new();
    config.set_to(None::<String>, "encoding".to_string(), "UTF-8".to_string());

    config.set_to(Some("Highlighting"), "comment.documentation".to_string(), "#6A9955FF".to_string());
    config.set_to(Some("Highlighting"), "comment".to_string(), "#808080FF".to_string());
    config.set_to(Some("Highlighting"), "string".to_string(), "#6AAB73FF".to_string());
    config.set_to(Some("Highlighting"), "keyword.control".to_string(), "#CC7832FF".to_string());
    config.set_to(Some("Highlighting"), "keyword.declaration".to_string(), "#CC7832FF".to_string());
    config.set_to(Some("Highlighting"), "keyword".to_string(), "#CC7832FF".to_string());
    config.set_to(Some("Highlighting"), "storage.modifier".to_string(), "#BBB529FF".to_string());
    config.set_to(Some("Highlighting"), "storage.type".to_string(), "#4EC9B0FF".to_string());
    config.set_to(Some("Highlighting"), "storage".to_string(), "#4EC9B0FF".to_string());
    config.set_to(Some("Highlighting"), "constant.numeric".to_string(), "#6897BBFF".to_string());
    config.set_to(Some("Highlighting"), "constant.language".to_string(), "#9876AAFF".to_string());
    config.set_to(Some("Highlighting"), "constant".to_string(), "#6897BBFF".to_string());
    config.set_to(Some("Highlighting"), "entity.name.function".to_string(), "#FFC66DFF".to_string());
    config.set_to(Some("Highlighting"), "entity.name.type".to_string(), "#A9B7C6FF".to_string());
    config.set_to(Some("Highlighting"), "entity".to_string(), "#A9B7C6FF".to_string());
    config.set_to(Some("Highlighting"), "variable.language".to_string(), "#CC7832FF".to_string());
    config.set_to(Some("Highlighting"), "modifier.annotation".to_string(), "#BBB529FF".to_string());

    config.write_to_file("settings/color_scheme.ini").unwrap();
    config
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_load_color_scheme_creates_default_when_missing() {
        let path = "settings/color_scheme.ini";
        
        // Ensure the file does not exist before test
        if std::path::Path::new(path).exists() {
            fs::remove_file(path).expect("Failed to remove color_scheme.ini before test");
        }

        let config = load_color_scheme();
        
        // Verify the file was created
        assert!(std::path::Path::new(path).exists(), "color_scheme.ini should have been created");

        // Verify some content
        let section = config.section(Some("Highlighting")).expect("Highlighting section missing");
        assert_eq!(section.get("comment"), Some("#808080FF"));
        assert_eq!(section.get("string"), Some("#6AAB73FF"));
    }

    #[test]
    fn test_load_user_settings_creates_default_when_missing() {
        let path = "settings/user_settings.ini";
        if std::path::Path::new(path).exists() {
            fs::remove_file(path).unwrap();
        }
        let config = load_user_settings();
        assert!(std::path::Path::new(path).exists());
        assert_eq!(config.get_from(None::<String>, "encoding"), Some("UTF-8"));
    }

    #[test]
    fn test_load_editor_settings_creates_default_when_missing() {
        let path = "settings/editor_settings.ini";
        if std::path::Path::new(path).exists() {
            fs::remove_file(path).unwrap();
        }
        let config = load_editor_settings();
        assert!(std::path::Path::new(path).exists());
        assert_eq!(config.get_from(Some("editor"), "show_line_numbers"), Some("true"));
    }
}