#[cfg(test)]
mod tests {
    use crate::syntax_definition::{CompiledRule, SyntaxDefinition};
    use crate::syntax_highlighter::{highlight_line, HighlightSpan};
    use regex::Regex;
    use crossterm::style::Color;

    #[test]
    fn test_escape_sequence_in_span() {
        let definition = SyntaxDefinition {
            name: "Test".to_string(),
            file_extensions: vec!["test".to_string()],
            rules: vec![CompiledRule::Span {
                scope: "string".to_string(),
                begin_regex: Regex::new("\"").unwrap(),
                end_regex: Regex::new("\"").unwrap(),
                escape_regex: Some(Regex::new(r"\\.").unwrap()),
            }],
        };

        // Escaped quote: "a\"b"
        let text = "\"a\\\"b\"";
        let (spans, state) = highlight_line(&definition, text, None);

        assert_eq!(state, None);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].byte_start, 0);
        assert_eq!(spans[0].byte_end, 6); // Entire "a\"b"
    }

    #[test]
    fn test_multi_line_escape_sequence() {
        let definition = SyntaxDefinition {
            name: "Test".to_string(),
            file_extensions: vec!["test".to_string()],
            rules: vec![CompiledRule::Span {
                scope: "string".to_string(),
                begin_regex: Regex::new("\"").unwrap(),
                end_regex: Regex::new("\"").unwrap(),
                escape_regex: Some(Regex::new(r"\\.").unwrap()),
            }],
        };

        // First line: "a\
        let line1 = "\"a\\";
        let (spans1, state1) = highlight_line(&definition, line1, None);
        assert!(state1.is_some());
        assert_eq!(spans1.len(), 1);
        assert_eq!(spans1[0].byte_end, 3);

        // Second line: "b"
        // In some languages, \ at end of line escapes the newline.
        // Here, if line1 was "a\" (quote, a, backslash), and line2 is "b",
        // it's still inside the string.
        let line2 = "\"b\""; // This line has its own quotes, but we are ALREADY in a string
        let (spans2, state2) = highlight_line(&definition, line2, state1);
        
        // If " is escaped by something on this line?
        // Wait, if I have:
        // "line1 \
        // line2"
        // In line1, the \ at the end might escape something? 
        // Our current logic: if it's in state1, it starts from index 0 of line2.
    }
    
    #[test]
    fn test_escape_at_end_of_line() {
         let definition = SyntaxDefinition {
            name: "Test".to_string(),
            file_extensions: vec!["test".to_string()],
            rules: vec![CompiledRule::Span {
                scope: "string".to_string(),
                begin_regex: Regex::new("\"").unwrap(),
                end_regex: Regex::new("\"").unwrap(),
                escape_regex: Some(Regex::new(r"\\.").unwrap()),
            }],
        };

        // Line 1: "a\
        let (spans1, state1) = highlight_line(&definition, "\"a\\", None);
        assert!(state1.is_some());

        // Line 2: \"b"
        // The first \" on Line 2 should be skipped if we want to support multi-line escapes that escape the start of the line? 
        // Actually, most escapes are single-line.
        // If line 2 is: \"b"
        let (spans2, state2) = highlight_line(&definition, "\\\"b\"", state1);
        assert_eq!(state2, None);
        assert_eq!(spans2.len(), 1);
        assert_eq!(spans2[0].byte_end, 4); // entire \"b"
    }
}
