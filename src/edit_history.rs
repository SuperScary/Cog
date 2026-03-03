use crate::position::Position;

#[derive(PartialEq)]
enum CharacterClass {
    Whitespace,
    Punctuation,
    Word,
}

fn classify(character: char) -> CharacterClass {
    if character.is_whitespace() {
        CharacterClass::Whitespace
    } else if character.is_alphanumeric() || character == '_' {
        CharacterClass::Word
    } else {
        CharacterClass::Punctuation
    }
}

fn same_character_class(a: char, b: char) -> bool {
    classify(a) == classify(b)
}

#[derive(Clone, Debug)]
pub enum EditAction {
    Insert {
        position: Position,
        text: String,
        end_position: Position,
    },
    Delete {
        start: Position,
        end: Position,
        deleted_text: String,
    },
}

impl EditAction {
    pub fn inverse(&self) -> Self {
        match self {
            EditAction::Insert {
                position,
                text,
                end_position,
            } => EditAction::Delete {
                start: *position,
                end: *end_position,
                deleted_text: text.clone(),
            },
            EditAction::Delete {
                start,
                end,
                deleted_text,
            } => EditAction::Insert {
                position: *start,
                text: deleted_text.clone(),
                end_position: *end,
            },
        }
    }

    fn try_merge_with(&self, next: &EditAction) -> Option<EditAction> {
        match (self, next) {
            // Consecutive character inserts (sequential typing on the same line)
            (
                EditAction::Insert {
                    position,
                    text: previous_text,
                    end_position,
                },
                EditAction::Insert {
                    position: next_position,
                    text: next_text,
                    end_position: next_end,
                },
            ) if *next_position == *end_position
                && !previous_text.contains('\n')
                && !next_text.contains('\n')
                && previous_text
                    .chars()
                    .last()
                    .zip(next_text.chars().next())
                    .is_some_and(|(a, b)| same_character_class(a, b)) =>
            {
                Some(EditAction::Insert {
                    position: *position,
                    text: format!("{previous_text}{next_text}"),
                    end_position: *next_end,
                })
            }

            // Consecutive backward deletes (holding backspace)
            (
                EditAction::Delete {
                    start: previous_start,
                    end: previous_end,
                    deleted_text: previous_text,
                },
                EditAction::Delete {
                    start: next_start,
                    end: next_end,
                    deleted_text: next_text,
                },
            ) if *next_end == *previous_start
                && !previous_text.contains('\n')
                && !next_text.contains('\n')
                && previous_text
                    .chars()
                    .next()
                    .zip(next_text.chars().last())
                    .is_some_and(|(a, b)| same_character_class(a, b)) =>
            {
                Some(EditAction::Delete {
                    start: *next_start,
                    end: *previous_end,
                    deleted_text: format!("{next_text}{previous_text}"),
                })
            }

            // Consecutive forward deletes (holding the Delete key)
            (
                EditAction::Delete {
                    start: previous_start,
                    end: previous_end,
                    deleted_text: previous_text,
                },
                EditAction::Delete {
                    start: next_start,
                    end: next_end,
                    deleted_text: next_text,
                },
            ) if *next_start == *previous_start
                && !previous_text.contains('\n')
                && !next_text.contains('\n')
                && previous_text
                    .chars()
                    .last()
                    .zip(next_text.chars().next())
                    .is_some_and(|(a, b)| same_character_class(a, b)) =>
            {
                let additional_columns = next_end.column_index - next_start.column_index;
                Some(EditAction::Delete {
                    start: *previous_start,
                    end: Position::new(
                        previous_end.row_index,
                        previous_end.column_index + additional_columns,
                    ),
                    deleted_text: format!("{previous_text}{next_text}"),
                })
            }

            _ => None,
        }
    }
}

pub struct EditHistory {
    undo_stack: Vec<EditAction>,
    redo_stack: Vec<EditAction>,
}

impl EditHistory {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn record(&mut self, action: EditAction, merge_with_previous: bool) {
        self.redo_stack.clear();

        if merge_with_previous
            && let Some(previous) = self.undo_stack.last()
            && let Some(merged) = previous.try_merge_with(&action)
        {
            *self.undo_stack.last_mut().unwrap() = merged;
            return;
        }

        self.undo_stack.push(action);
    }

    pub fn undo(&mut self) -> Option<EditAction> {
        let forward_action = self.undo_stack.pop()?;
        let inverse_action = forward_action.inverse();
        self.redo_stack.push(forward_action);
        Some(inverse_action)
    }

    pub fn redo(&mut self) -> Option<EditAction> {
        let forward_action = self.redo_stack.pop()?;
        self.undo_stack.push(forward_action.clone());
        Some(forward_action)
    }
}
