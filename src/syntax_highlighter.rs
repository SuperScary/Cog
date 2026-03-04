use std::collections::HashMap;
use std::sync::LazyLock;
use crossterm::style::Color;

use crate::syntax_definition::{CompiledRule, SyntaxDefinition};

pub struct HighlightSpan {
    pub byte_start: usize,
    pub byte_end: usize,
    pub color: Color,
}

/// Highlights a single line of text according to the syntax definition.
///
/// `active_span_rule_index` tracks whether we're inside a multi-line span
/// (e.g. a block comment or string that started on a previous line).
/// Returns the highlight spans for this line and the span state to carry
/// into the next line.
pub fn highlight_line(definition: &SyntaxDefinition, text: &str, active_span_rule_index: Option<usize>, ) -> (Vec<HighlightSpan>, Option<usize>) {
    let mut spans = Vec::new();
    let mut current_state = active_span_rule_index;

    if let Some(rule_index) = current_state {
        if let Some(CompiledRule::Span {
            scope,
            end_regex,
            escape_regex,
            ..
        }) = definition.rules.get(rule_index)
        {
            if let Some(end_pos) = scan_span_end(text, 0, end_regex, escape_regex.as_ref()) {
                spans.push(HighlightSpan { byte_start: 0, byte_end: end_pos, color: scope_to_color(scope) });
                current_state = None;
                find_highlights(definition, text, end_pos, &mut spans, &mut current_state);
            } else {
                spans.push(HighlightSpan { byte_start: 0, byte_end: text.len(), color: scope_to_color(scope) });
                // still inside span for next line
            }
            return (spans, current_state);
        }
    } else {
        find_highlights(definition, text, 0, &mut spans, &mut current_state);
    }

    (spans, current_state)
}

// Helper: scan to the end of a span, honoring escapes
fn scan_span_end(text: &str, mut pos: usize, end_regex: &regex::Regex, escape_regex: Option<&regex::Regex>) -> Option<usize> {
    while pos < text.len() {
        if let Some(esc) = escape_regex {
            if let Some(m) = esc.find(&text[pos..]) {
                if m.start() == 0 {
                    pos += m.end();
                    continue;
                }
            }
        }
        if let Some(m) = end_regex.find(&text[pos..]) {
            if m.start() == 0 {
                return Some(pos + m.end());
            }
        }
        pos += 1;
    }
    None
}

fn find_highlights(
    definition: &SyntaxDefinition,
    text: &str,
    from: usize,
    spans: &mut Vec<HighlightSpan>,
    active_span: &mut Option<usize>,
) {
    let mut position = from;

    while position < text.len() {
        let Some((match_start, match_end, rule_index)) =
            find_earliest_match(definition, text, position)
        else {
            break;
        };

        match &definition.rules[rule_index] {
            CompiledRule::Pattern { scope, .. } => {
                spans.push(HighlightSpan {
                    byte_start: match_start,
                    byte_end: match_end,
                    color: scope_to_color(scope),
                });
                position = match_end;
            }
            CompiledRule::Span {
                scope,
                end_regex,
                escape_regex,
                ..
            } => {
                let start_pos = match_end;
                if let Some(end_pos) = scan_span_end(text, start_pos, end_regex, escape_regex.as_ref()) {
                    spans.push(HighlightSpan { byte_start: match_start, byte_end: end_pos, color: scope_to_color(scope) });
                    position = end_pos;
                } else {
                    spans.push(HighlightSpan { byte_start: match_start, byte_end: text.len(), color: scope_to_color(scope) });
                    *active_span = Some(rule_index);
                    return;
                }
            }
        }
    }
}

fn find_earliest_match(
    definition: &SyntaxDefinition,
    text: &str,
    from: usize,
) -> Option<(usize, usize, usize)> {
    let search_text = &text[from..];
    let mut best: Option<(usize, usize, usize)> = None;

    for (index, rule) in definition.rules.iter().enumerate() {
        let regex = match rule {
            CompiledRule::Pattern { regex, .. } => regex,
            CompiledRule::Span { begin_regex, .. } => begin_regex,
        };

        if let Some(found) = regex.find(search_text) {
            let absolute_start = from + found.start();
            let absolute_end = from + found.end();

            let is_better = match best {
                None => true,
                Some((best_start, _, best_index)) => {
                    absolute_start < best_start
                        || (absolute_start == best_start && index < best_index)
                }
            };

            if is_better {
                best = Some((absolute_start, absolute_end, index));
            }
        }
    }

    best
}

fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');

    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r, g, b))
        }
        8 => { // RRGGBBAA -> ignore AA
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r, g, b))
        }
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
            Some((r * 16 + r, g * 16 + g, b * 16 + b))
        }
        4 => { // RGBA -> ignore A
            let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
            Some((r * 16 + r, g * 16 + g, b * 16 + b))
        }
        _ => None,
    }
}

fn parse_named_color(name: &str) -> Option<Color> {
    let n = name
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-', '.'], "_");

    Some(match n.as_str() {
        "reset" | "default" | "none" => Color::Reset,

        // standard 16 colors (crossterm)
        "black" => Color::Black,
        "dark_grey" | "dark_gray" => Color::DarkGrey,
        "grey" | "gray" => Color::Grey,
        "white" => Color::White,

        "red" => Color::Red,
        "dark_red" => Color::DarkRed,
        "green" => Color::Green,
        "dark_green" => Color::DarkGreen,
        "yellow" => Color::Yellow,
        "dark_yellow" => Color::DarkYellow,
        "blue" => Color::Blue,
        "dark_blue" => Color::DarkBlue,
        "magenta" => Color::Magenta,
        "dark_magenta" => Color::DarkMagenta,
        "cyan" => Color::Cyan,
        "dark_cyan" => Color::DarkCyan,

        "orange" => Color::DarkYellow,
        "purple" => Color::Magenta,
        "pink" => Color::Magenta,

        _ => return None,
    })
}

fn parse_color(color_spec: &str) -> Color {
    let s = color_spec.trim();

    // named console colors first
    if let Some(c) = parse_named_color(s) {
        return c;
    }

    // truecolor
    if let Some((r, g, b)) = hex_to_rgb(s) {
        return Color::Rgb { r, g, b };
    }

    Color::Reset
}

static COLOR_MAP: LazyLock<HashMap<String, Color>> = LazyLock::new(|| {
    let config = crate::config::configs::load_color_scheme();
    let mut map = HashMap::new();

    if let Some(section) = config.section(Some("Highlighting")) {
        for (key, value) in section.iter() {
            map.insert(key.to_string(), parse_color(value));
        }
    }
    map
});

pub fn scope_to_color(scope: &str) -> Color {
    let mut current_scope = scope;
    loop {
        if let Some(color) = COLOR_MAP.get(current_scope) {
            return *color;
        }
        if let Some(dot_index) = current_scope.rfind('.') {
            current_scope = &current_scope[..dot_index];
        } else {
            break;
        }
    }
    Color::Reset
}
