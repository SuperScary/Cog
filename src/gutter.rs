use std::io;

use crossterm::{
    queue,
    style::{Attribute, Color, Print, SetAttribute, SetForegroundColor},
};

pub struct Gutter {
    digit_count: usize,
}

impl Gutter {
    pub fn for_line_count(total_lines: usize) -> Self {
        let digit_count = if total_lines == 0 {
            1
        } else {
            total_lines.ilog10() as usize + 1
        };
        Self { digit_count }
    }

    pub fn width(&self) -> usize {
        self.digit_count + 1
    }

    pub fn render_line_number(
        &self,
        stdout: &mut io::Stdout,
        line_number: Option<usize>,
    ) -> io::Result<()> {
        match line_number {
            Some(number) => {
                let formatted = format!("{:>width$} ", number, width = self.digit_count);
                queue!(
                    stdout,
                    SetForegroundColor(Color::DarkGrey),
                    SetAttribute(Attribute::Dim),
                    Print(formatted),
                    SetAttribute(Attribute::Reset),
                )?;
            }
            None => {
                queue!(stdout, Print(" ".repeat(self.width())))?;
            }
        }
        Ok(())
    }
}
