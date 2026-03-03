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
pub fn highlight_line(
    definition: &SyntaxDefinition,
    text: &str,
    active_span_rule_index: Option<usize>,
) -> (Vec<HighlightSpan>, Option<usize>) {
    let mut spans = Vec::new();
    let mut current_state = active_span_rule_index;

    if let Some(rule_index) = current_state {
        if let Some(CompiledRule::Span {
            scope, end_regex, ..
        }) = definition.rules.get(rule_index)
        {
            if let Some(end_match) = end_regex.find(text) {
                spans.push(HighlightSpan {
                    byte_start: 0,
                    byte_end: end_match.end(),
                    color: scope_to_color(scope),
                });
                current_state = None;
                find_highlights(
                    definition,
                    text,
                    end_match.end(),
                    &mut spans,
                    &mut current_state,
                );
            } else {
                spans.push(HighlightSpan {
                    byte_start: 0,
                    byte_end: text.len(),
                    color: scope_to_color(scope),
                });
                return (spans, current_state);
            }
        }
    } else {
        find_highlights(definition, text, 0, &mut spans, &mut current_state);
    }

    (spans, current_state)
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
                scope, end_regex, ..
            } => {
                if let Some(end_match) = end_regex.find(&text[match_end..]) {
                    let span_end = match_end + end_match.end();
                    spans.push(HighlightSpan {
                        byte_start: match_start,
                        byte_end: span_end,
                        color: scope_to_color(scope),
                    });
                    position = span_end;
                } else {
                    spans.push(HighlightSpan {
                        byte_start: match_start,
                        byte_end: text.len(),
                        color: scope_to_color(scope),
                    });
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

pub fn scope_to_color(scope: &str) -> Color {
    if scope.eq("comment.documentation") {
        Color::Green
    } else if scope.starts_with("comment") {
        Color::DarkGrey
    } else if scope.starts_with("string") {
        Color::Green
    } else if scope.starts_with("keyword.control") {
        Color::Magenta
    } else if scope.starts_with("keyword.declaration") {
        Color::Blue
    } else if scope.starts_with("keyword") {
        Color::Magenta
    } else if scope.starts_with("storage.modifier") {
        Color::Cyan
    } else if scope.starts_with("storage.type") {
        Color::Cyan
    } else if scope.starts_with("storage") {
        Color::Cyan
    } else if scope.starts_with("constant.numeric") {
        Color::Yellow
    } else if scope.starts_with("constant.language") {
        Color::DarkYellow
    } else if scope.starts_with("constant") {
        Color::Yellow
    } else if scope.starts_with("entity.name.function") {
        Color::Yellow
    } else if scope.starts_with("entity.name.type") {
        Color::DarkCyan
    } else if scope.starts_with("entity") {
        Color::Cyan
    } else if scope.starts_with("variable.language") {
        Color::Red
    } else if scope.starts_with("modifier.annotation") {
        Color::Yellow
    } else {
        Color::Reset
    }
}
