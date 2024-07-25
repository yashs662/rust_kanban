use crate::ui::text_box::{
    helper_structs::{CursorPos, TextBoxViewport},
    utils::{find_word_start_backward, find_word_start_forward},
};
use ratatui::style::Style;
use std::{
    cmp::{self, Ordering},
    fmt,
};

#[derive(Clone, Debug)]
pub enum TextBoxEditKind {
    InsertChar(char),
    DeleteChar(char),
    InsertNewline,
    DeleteNewline,
    InsertStr(String),
    DeleteStr(String),
    InsertChunk(Vec<String>),
    DeleteChunk(Vec<String>),
}

impl TextBoxEditKind {
    pub fn apply(&self, lines: &mut Vec<String>, before: &CursorPos, after: &CursorPos) {
        match self {
            TextBoxEditKind::InsertChar(c) => {
                lines[before.row].insert(before.offset, *c);
            }
            TextBoxEditKind::DeleteChar(_) => {
                lines[before.row].remove(after.offset);
            }
            TextBoxEditKind::InsertNewline => {
                let line = &mut lines[before.row];
                let next_line = line[before.offset..].to_string();
                line.truncate(before.offset);
                lines.insert(before.row + 1, next_line);
            }
            TextBoxEditKind::DeleteNewline => {
                debug_assert!(before.row > 0, "invalid pos: {:?}", before);
                let line = lines.remove(before.row);
                lines[before.row - 1].push_str(&line);
            }
            TextBoxEditKind::InsertStr(s) => {
                lines[before.row].insert_str(before.offset, s.as_str());
            }
            TextBoxEditKind::DeleteStr(s) => {
                lines[after.row].drain(after.offset..after.offset + s.len());
            }
            TextBoxEditKind::InsertChunk(c) => {
                debug_assert!(c.len() > 1, "Chunk size must be > 1: {:?}", c);

                // Handle first line of chunk
                let first_line = &mut lines[before.row];
                let mut last_line = first_line.drain(before.offset..).as_str().to_string();
                first_line.push_str(&c[0]);

                // Handle last line of chunk
                let next_row = before.row + 1;
                last_line.insert_str(0, c.last().unwrap());
                lines.insert(next_row, last_line);

                // Handle middle lines of chunk
                lines.splice(next_row..next_row, c[1..c.len() - 1].iter().cloned());
            }
            TextBoxEditKind::DeleteChunk(c) => {
                debug_assert!(c.len() > 1, "Chunk size must be > 1: {:?}", c);

                // Remove middle lines of chunk
                let mut last_line = lines
                    .drain(after.row + 1..after.row + c.len())
                    .last()
                    .unwrap();
                // Remove last line of chunk
                last_line.drain(..c[c.len() - 1].len());

                // Remove first line of chunk and concat remaining
                let first_line = &mut lines[after.row];
                first_line.truncate(after.offset);
                first_line.push_str(&last_line);
            }
        }
    }

    pub fn invert(&self) -> Self {
        use TextBoxEditKind::*;
        match self.clone() {
            InsertChar(c) => DeleteChar(c),
            DeleteChar(c) => InsertChar(c),
            InsertNewline => DeleteNewline,
            DeleteNewline => InsertNewline,
            InsertStr(s) => DeleteStr(s),
            DeleteStr(s) => InsertStr(s),
            InsertChunk(c) => DeleteChunk(c),
            DeleteChunk(c) => InsertChunk(c),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CharKind {
    Space,
    Punctuation,
    Other,
}

impl CharKind {
    pub fn new(c: char) -> Self {
        if c.is_whitespace() {
            Self::Space
        } else if c.is_ascii_punctuation() {
            Self::Punctuation
        } else {
            Self::Other
        }
    }
}

pub enum TextBoxScroll {
    Delta { rows: i16, cols: i16 },
    PageDown,
    PageUp,
}

impl TextBoxScroll {
    pub(crate) fn scroll(self, viewport: &mut TextBoxViewport) {
        let (rows, cols) = match self {
            Self::Delta { rows, cols } => (rows, cols),
            Self::PageDown => {
                let (_, _, _, height) = viewport.rect();
                (height as i16, 0)
            }
            Self::PageUp => {
                let (_, _, _, height) = viewport.rect();
                (-(height as i16), 0)
            }
        };
        viewport.scroll(rows, cols);
    }
}

impl From<(i16, i16)> for TextBoxScroll {
    fn from((rows, cols): (i16, i16)) -> Self {
        Self::Delta { rows, cols }
    }
}

#[derive(Debug, Clone)]
pub enum YankText {
    Piece(String),
    Chunk(Vec<String>),
}

impl Default for YankText {
    fn default() -> Self {
        Self::Piece(String::new())
    }
}

impl From<String> for YankText {
    fn from(s: String) -> Self {
        Self::Piece(s)
    }
}
impl From<Vec<String>> for YankText {
    fn from(mut c: Vec<String>) -> Self {
        match c.len() {
            0 => Self::default(),
            1 => Self::Piece(c.remove(0)),
            _ => Self::Chunk(c),
        }
    }
}

impl fmt::Display for YankText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Piece(s) => write!(f, "{}", s),
            Self::Chunk(ss) => write!(f, "{}", ss.join("\n")),
        }
    }
}

#[derive(Eq, PartialEq)]
pub enum Boundary {
    Cursor(Style),
    Select(Style),
    End,
}

impl PartialOrd for Boundary {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Boundary {
    fn cmp(&self, other: &Self) -> Ordering {
        fn rank(b: &Boundary) -> u8 {
            match b {
                Boundary::Cursor(_) => 2,
                Boundary::Select(_) => 1,
                Boundary::End => 0,
            }
        }
        rank(self).cmp(&rank(other))
    }
}

impl Boundary {
    pub fn style(&self) -> Option<Style> {
        match self {
            Boundary::Cursor(s) => Some(*s),
            Boundary::Select(s) => Some(*s),
            Boundary::End => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CursorMove {
    Forward,
    Back,
    Up,
    Down,
    Head,
    End,
    Top,
    Bottom,
    WordForward,
    WordBack,
    ParagraphForward,
    ParagraphBack,
    Jump(u16, u16),
    InViewport,
}

impl CursorMove {
    pub(crate) fn next_cursor(
        &self,
        (row, col): (usize, usize),
        lines: &[String],
        viewport: &TextBoxViewport,
    ) -> Option<(usize, usize)> {
        use CursorMove::*;

        fn fit_col(col: usize, line: &str) -> usize {
            cmp::min(col, line.chars().count())
        }

        match self {
            Forward if col >= lines[row].chars().count() => {
                (row + 1 < lines.len()).then_some((row + 1, 0))
            }
            Forward => Some((row, col + 1)),
            Back if col == 0 => {
                let row = row.checked_sub(1)?;
                Some((row, lines[row].chars().count()))
            }
            Back => Some((row, col - 1)),
            Up => {
                let row = row.checked_sub(1)?;
                Some((row, fit_col(col, &lines[row])))
            }
            Down => Some((row + 1, fit_col(col, lines.get(row + 1)?))),
            Head => Some((row, 0)),
            End => Some((row, lines[row].chars().count())),
            Top => Some((0, fit_col(col, &lines[0]))),
            Bottom => {
                let row = lines.len() - 1;
                Some((row, fit_col(col, &lines[row])))
            }
            WordForward => {
                if let Some(col) = find_word_start_forward(&lines[row], col) {
                    Some((row, col))
                } else if row + 1 < lines.len() {
                    Some((row + 1, 0))
                } else {
                    Some((row, lines[row].chars().count()))
                }
            }
            WordBack => {
                if let Some(col) = find_word_start_backward(&lines[row], col) {
                    Some((row, col))
                } else if row > 0 {
                    Some((row - 1, lines[row - 1].chars().count()))
                } else {
                    Some((row, 0))
                }
            }
            ParagraphForward => {
                let mut prev_is_empty = lines[row].is_empty();
                for (row, line) in lines.iter().enumerate().skip(row + 1) {
                    let is_empty = line.is_empty();
                    if !is_empty && prev_is_empty {
                        return Some((row, fit_col(col, line)));
                    }
                    prev_is_empty = is_empty;
                }
                let row = lines.len() - 1;
                Some((row, fit_col(col, &lines[row])))
            }
            ParagraphBack => {
                let row = row.checked_sub(1)?;
                let mut prev_is_empty = lines[row].is_empty();
                for row in (0..row).rev() {
                    let is_empty = lines[row].is_empty();
                    if is_empty && !prev_is_empty {
                        return Some((row + 1, fit_col(col, &lines[row + 1])));
                    }
                    prev_is_empty = is_empty;
                }
                Some((0, fit_col(col, &lines[0])))
            }
            Jump(row, col) => {
                let row = cmp::min(*row as usize, lines.len() - 1);
                let col = fit_col(*col as usize, &lines[row]);
                Some((row, col))
            }
            InViewport => {
                let (row_top, col_top, row_bottom, col_bottom) = viewport.position();

                let row = row.clamp(row_top as usize, row_bottom as usize);
                let row = cmp::min(row, lines.len() - 1);
                let col = col.clamp(col_top as usize, col_bottom as usize);
                let col = fit_col(col, &lines[row]);

                Some((row, col))
            }
        }
    }
}
