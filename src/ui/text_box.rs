// This implementation is a stripped down version inspired by https://github.com/rhysd/tui-textarea,
// i have chosen this approach to allow me to independently make changes to the codebase without
// having to worry about the original codebase, and use the latest possible ratatui version
// without waiting for the original author as the original codebase is not actively maintained.

use crate::{
    inputs::key::Key,
    util::{num_digits, spaces},
};
use portable_atomic::AtomicU64;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget},
};
use std::{
    borrow::Cow,
    cmp::{self, Ordering},
    collections::VecDeque,
    fmt, iter,
};
use unicode_width::UnicodeWidthChar;

#[derive(Debug, Clone)]
pub struct CursorPos {
    pub row: usize,
    pub col: usize,
    pub offset: usize,
}

impl CursorPos {
    pub fn new(row: usize, col: usize, offset: usize) -> Self {
        Self { row, col, offset }
    }
}

#[derive(Clone, Debug)]
pub struct TextBox<'a> {
    lines: Vec<String>,
    block: Option<Block<'a>>,
    style: Style,
    cursor: (usize, usize),
    tab_len: u8,
    hard_tab_indent: bool,
    history: TextBoxHistory,
    cursor_line_style: Style,
    line_number_style: Option<Style>,
    pub(crate) viewport: TextBoxViewport,
    cursor_style: Style,
    yank: YankText,
    alignment: Alignment,
    pub single_line_mode: bool,
    pub(crate) placeholder: String,
    pub(crate) placeholder_style: Style,
    mask: Option<char>,
    selection_start: Option<(usize, usize)>,
    select_style: Style,
}

impl<'a> TextBox<'a> {
    pub fn new(mut lines: Vec<String>, single_line_mode: bool) -> Self {
        if lines.is_empty() {
            lines.push(String::new());
        }

        Self {
            lines,
            block: None,
            style: Style::default(),
            cursor: (0, 0),
            tab_len: 2,
            hard_tab_indent: false,
            history: TextBoxHistory::new(9999),
            cursor_line_style: Style::default(),
            line_number_style: None,
            viewport: TextBoxViewport::default(),
            cursor_style: Style::default(),
            yank: YankText::default(),
            alignment: Alignment::Left,
            single_line_mode,
            placeholder: String::new(),
            placeholder_style: Style::default(),
            mask: None,
            selection_start: None,
            select_style: Style::default(),
        }
    }

    pub fn from_list_of_strings(lines: Vec<String>, single_line_mode: bool) -> Self {
        Self::new(lines, single_line_mode)
    }

    pub fn from_list_of_str(lines: Vec<&'a str>, single_line_mode: bool) -> Self {
        Self::new(
            lines.into_iter().map(|s| s.to_string()).collect(),
            single_line_mode,
        )
    }

    pub fn from_string_with_newline_sep(s: String, single_line_mode: bool) -> Self {
        Self::new(
            s.split('\n').map(|s| s.to_string()).collect(),
            single_line_mode,
        )
    }

    pub fn reset(&mut self) {
        let single_line_mode = self.single_line_mode;
        *self = Self::new(vec![String::new()], single_line_mode);
    }

    pub fn get_joined_lines(&self) -> String {
        self.lines.join("\n")
    }

    pub fn get_num_lines(&self) -> usize {
        self.lines.len()
    }

    pub fn set_placeholder_text(&mut self, placeholder: impl Into<String>) {
        self.placeholder = placeholder.into();
    }

    pub fn set_mask_char(&mut self, mask: char) {
        self.mask = Some(mask);
    }

    pub fn clear_mask_char(&mut self) {
        self.mask = None;
    }

    pub fn disable_cursor(&mut self) {
        self.cursor_style = Style::default();
    }

    pub fn enable_cursor(&mut self, cursor_style: Style) {
        self.cursor_style = cursor_style;
    }

    // TODO: Add keybindings to README
    pub fn input(&mut self, input: Key) -> bool {
        match input {
            Key::Ctrl('m') | Key::Char('\n' | '\r') | Key::Enter => {
                if self.single_line_mode {
                    return false;
                }
                self.insert_newline();
                true
            }
            Key::Char(c) => {
                self.insert_char(c);
                true
            }
            Key::Tab => {
                if self.single_line_mode {
                    return false;
                }
                self.insert_tab()
            }
            Key::Ctrl('h') | Key::Backspace => self.delete_char(),
            Key::Ctrl('d') | Key::Delete => self.delete_next_char(),
            Key::Ctrl('k') => self.delete_line_by_end(),
            Key::Ctrl('j') => self.delete_line_by_head(),
            Key::Ctrl('w') | Key::Alt('h') | Key::AltBackspace => self.delete_word(),
            Key::AltDelete | Key::Alt('d') => self.delete_next_word(),
            Key::Ctrl('n') | Key::Down => {
                if self.single_line_mode {
                    return false;
                }
                self.move_cursor(CursorMove::Down);
                false
            }
            Key::ShiftDown => {
                if self.single_line_mode {
                    return false;
                }
                self.move_cursor_with_shift(CursorMove::Down, true);
                false
            }
            Key::Ctrl('p') | Key::Up => {
                if self.single_line_mode {
                    return false;
                }
                self.move_cursor(CursorMove::Up);
                false
            }
            Key::ShiftUp => {
                if self.single_line_mode {
                    return false;
                }
                self.move_cursor_with_shift(CursorMove::Up, true);
                false
            }
            Key::Ctrl('f') | Key::Right => {
                self.move_cursor(CursorMove::Forward);
                false
            }
            Key::ShiftRight => {
                self.move_cursor_with_shift(CursorMove::Forward, true);
                false
            }
            Key::Ctrl('b') | Key::Left => {
                self.move_cursor(CursorMove::Back);
                false
            }
            Key::Ctrl('a') => {
                self.select_all();
                false
            }
            Key::ShiftLeft => {
                self.move_cursor_with_shift(CursorMove::Back, true);
                false
            }
            Key::Home | Key::CtrlAlt('b') | Key::CtrlAltLeft => {
                self.move_cursor(CursorMove::Head);
                false
            }
            Key::ShiftHome | Key::CtrlAltShift('b') | Key::CtrlAltShiftLeft => {
                self.move_cursor_with_shift(CursorMove::Head, true);
                false
            }
            Key::Ctrl('e') | Key::End | Key::CtrlAltRight | Key::CtrlAlt('f') => {
                self.move_cursor(CursorMove::End);
                false
            }
            Key::CtrlShift('e')
            | Key::ShiftEnd
            | Key::CtrlAltShiftRight
            | Key::CtrlAltShift('f') => {
                self.move_cursor_with_shift(CursorMove::End, true);
                false
            }
            Key::Alt('<') | Key::CtrlAltUp | Key::CtrlAlt('p') => {
                if self.single_line_mode {
                    return false;
                }
                self.move_cursor(CursorMove::Top);
                false
            }
            Key::AltShift('<') | Key::CtrlAltShiftUp | Key::CtrlAltShift('p') => {
                if self.single_line_mode {
                    return false;
                }
                self.move_cursor_with_shift(CursorMove::Top, true);
                false
            }
            Key::Alt('>') | Key::CtrlAltDown | Key::CtrlAlt('n') => {
                if self.single_line_mode {
                    return false;
                }
                self.move_cursor(CursorMove::Bottom);
                false
            }
            Key::AltShift('>') | Key::CtrlAltShiftDown | Key::CtrlAltShift('n') => {
                if self.single_line_mode {
                    return false;
                }
                self.move_cursor_with_shift(CursorMove::Bottom, true);
                false
            }
            Key::Alt('f') | Key::CtrlRight => {
                self.move_cursor(CursorMove::WordForward);
                false
            }
            Key::AltShift('f') | Key::CtrlShiftRight => {
                self.move_cursor_with_shift(CursorMove::WordForward, true);
                false
            }
            Key::Alt('b') | Key::CtrlLeft => {
                self.move_cursor(CursorMove::WordBack);
                false
            }
            Key::AltShift('b') | Key::CtrlShiftLeft => {
                self.move_cursor_with_shift(CursorMove::WordBack, true);
                false
            }
            Key::Alt(']') | Key::Alt('n') | Key::CtrlDown => {
                self.move_cursor(CursorMove::ParagraphForward);
                false
            }
            Key::AltShift(']') | Key::AltShift('n') | Key::CtrlShiftDown => {
                self.move_cursor_with_shift(CursorMove::ParagraphForward, true);
                false
            }
            Key::Alt('[') | Key::Alt('p') | Key::CtrlUp => {
                self.move_cursor(CursorMove::ParagraphBack);
                false
            }
            Key::AltShift('[') | Key::AltShift('p') | Key::CtrlShiftUp => {
                self.move_cursor_with_shift(CursorMove::ParagraphBack, true);
                false
            }
            Key::Ctrl('z') => self.undo(),
            Key::Ctrl('y') => self.redo(),
            Key::Ctrl('c') => {
                self.copy();
                false
            }
            Key::Ctrl('x') => self.cut(),
            Key::Ctrl('v') => self.paste(),
            Key::PageDown => {
                if self.single_line_mode {
                    return false;
                }
                self.scroll(TextBoxScroll::PageDown);
                false
            }
            Key::ShiftPageDown => {
                if self.single_line_mode {
                    return false;
                }
                self.scroll_with_shift(TextBoxScroll::PageDown, true);
                false
            }
            Key::PageUp => {
                if self.single_line_mode {
                    return false;
                }
                self.scroll(TextBoxScroll::PageUp);
                false
            }
            Key::ShiftPageUp => {
                if self.single_line_mode {
                    return false;
                }
                self.scroll_with_shift(TextBoxScroll::PageUp, true);
                false
            }
            _ => false,
        }
    }

    pub fn input_without_shortcuts(&mut self, input: Key) -> bool {
        match input {
            Key::Char(c) => {
                self.insert_char(c);
                true
            }
            Key::Tab => self.insert_tab(),
            Key::Backspace => self.delete_char(),
            Key::Delete => self.delete_next_char(),
            Key::Enter => {
                self.insert_newline();
                true
            }
            _ => false,
        }
    }

    pub fn set_selection_style(&mut self, style: Style) {
        self.select_style = style;
    }

    fn line_offset(&self, row: usize, col: usize) -> usize {
        let line = self
            .lines
            .get(row)
            .unwrap_or(&self.lines[self.lines.len() - 1]);
        line.char_indices()
            .nth(col)
            .map(|(i, _)| i)
            .unwrap_or(line.len())
    }

    pub fn copy(&mut self) {
        if let Some((start, end)) = self.take_selection_range() {
            if start.row == end.row {
                self.yank = self.lines[start.row][start.offset..end.offset]
                    .to_string()
                    .into();
                return;
            }
            let mut chunk = vec![self.lines[start.row][start.offset..].to_string()];
            chunk.extend(self.lines[start.row + 1..end.row].iter().cloned());
            chunk.push(self.lines[end.row][..end.offset].to_string());
            self.yank = YankText::Chunk(chunk);
        }
    }

    pub fn cut(&mut self) -> bool {
        self.delete_selection(true)
    }

    pub fn paste(&mut self) -> bool {
        self.delete_selection(false);
        match self.yank.clone() {
            YankText::Piece(s) => self.insert_piece(s),
            YankText::Chunk(c) => self.insert_chunk(c),
        }
    }

    fn selection_range(&self) -> Option<(CursorPos, CursorPos)> {
        let (sr, sc) = self.selection_start?;
        let (er, ec) = self.cursor;
        let (so, eo) = (self.line_offset(sr, sc), self.line_offset(er, ec));
        let s = CursorPos::new(sr, sc, so);
        let e = CursorPos::new(er, ec, eo);
        match (sr, so).cmp(&(er, eo)) {
            Ordering::Less => Some((s, e)),
            Ordering::Equal => None,
            Ordering::Greater => Some((e, s)),
        }
    }

    fn take_selection_range(&mut self) -> Option<(CursorPos, CursorPos)> {
        let range = self.selection_range();
        self.cancel_selection();
        range
    }

    fn delete_range(&mut self, start: CursorPos, end: CursorPos, should_yank: bool) {
        self.cursor = (start.row, start.col);

        if start.row == end.row {
            let removed = self.lines[start.row]
                .drain(start.offset..end.offset)
                .as_str()
                .to_string();
            if should_yank {
                self.yank = removed.clone().into();
            }
            self.push_history(TextBoxEditKind::DeleteStr(removed), end, start.offset);
            return;
        }

        let mut deleted = vec![self.lines[start.row]
            .drain(start.offset..)
            .as_str()
            .to_string()];
        deleted.extend(self.lines.drain(start.row + 1..end.row));
        if start.row + 1 < self.lines.len() {
            let mut last_line = self.lines.remove(start.row + 1);
            self.lines[start.row].push_str(&last_line[end.offset..]);
            last_line.truncate(end.offset);
            deleted.push(last_line);
        }

        if should_yank {
            self.yank = YankText::Chunk(deleted.clone());
        }

        let edit = if deleted.len() == 1 {
            TextBoxEditKind::DeleteStr(deleted.remove(0))
        } else {
            TextBoxEditKind::DeleteChunk(deleted)
        };
        self.push_history(edit, end, start.offset);
    }

    fn delete_selection(&mut self, should_yank: bool) -> bool {
        if let Some((s, e)) = self.take_selection_range() {
            self.delete_range(s, e, should_yank);
            return true;
        }
        false
    }

    fn push_history(&mut self, kind: TextBoxEditKind, before: CursorPos, after_offset: usize) {
        let (row, col) = self.cursor;
        let after = CursorPos::new(row, col, after_offset);
        let edit = TextBoxEdit::new(kind, before, after);
        self.history.push(edit);
    }

    pub fn insert_char(&mut self, c: char) {
        if c == '\n' || c == '\r' {
            self.insert_newline();
            return;
        }

        self.delete_selection(false);
        let (row, col) = self.cursor;
        let line = &mut self.lines[row];
        let i = line
            .char_indices()
            .nth(col)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        line.insert(i, c);
        self.cursor.1 += 1;
        self.push_history(
            TextBoxEditKind::InsertChar(c),
            CursorPos::new(row, col, i),
            i + c.len_utf8(),
        );
    }

    pub fn insert_str<S: AsRef<str>>(&mut self, s: S) -> bool {
        let modified = self.delete_selection(false);
        let mut lines: Vec<_> = s
            .as_ref()
            .split('\n')
            .map(|s| s.strip_suffix('\r').unwrap_or(s).to_string())
            .collect();
        match lines.len() {
            0 => modified,
            1 => self.insert_piece(lines.remove(0)),
            _ => self.insert_chunk(lines),
        }
    }

    fn insert_chunk(&mut self, chunk: Vec<String>) -> bool {
        debug_assert!(chunk.len() > 1, "Chunk size must be > 1: {:?}", chunk);

        let (row, col) = self.cursor;
        let line = &mut self.lines[row];
        let i = line
            .char_indices()
            .nth(col)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        let before = CursorPos::new(row, col, i);

        let (row, col) = (
            row + chunk.len() - 1,
            chunk[chunk.len() - 1].chars().count(),
        );
        self.cursor = (row, col);

        let end_offset = chunk.last().unwrap().len();

        let edit = TextBoxEditKind::InsertChunk(chunk);
        edit.apply(
            &mut self.lines,
            &before,
            &CursorPos::new(row, col, end_offset),
        );

        self.push_history(edit, before, end_offset);
        true
    }

    fn insert_piece(&mut self, s: String) -> bool {
        if s.is_empty() {
            return false;
        }

        let (row, col) = self.cursor;
        let line = &mut self.lines[row];
        debug_assert!(
            !s.contains('\n'),
            "string given to TextArea::insert_piece must not contain newline: {:?}",
            line,
        );

        let i = line
            .char_indices()
            .nth(col)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        line.insert_str(i, &s);
        let end_offset = i + s.len();

        self.cursor.1 += s.chars().count();
        self.push_history(
            TextBoxEditKind::InsertStr(s),
            CursorPos::new(row, col, i),
            end_offset,
        );
        true
    }

    pub fn delete_str(&mut self, chars: usize) -> bool {
        if self.delete_selection(false) {
            return true;
        }
        if chars == 0 {
            return false;
        }

        let (start_row, start_col) = self.cursor;

        let mut remaining = chars;
        let mut find_end = move |line: &str| {
            let mut col = 0usize;
            for (i, _) in line.char_indices() {
                if remaining == 0 {
                    return Some((i, col));
                }
                col += 1;
                remaining -= 1;
            }
            if remaining == 0 {
                Some((line.len(), col))
            } else {
                remaining -= 1;
                None
            }
        };

        let line = &self.lines[start_row];
        let start_offset = {
            line.char_indices()
                .nth(start_col)
                .map(|(i, _)| i)
                .unwrap_or(line.len())
        };

        // First line
        if let Some((offset_delta, col_delta)) = find_end(&line[start_offset..]) {
            let end_offset = start_offset + offset_delta;
            let end_col = start_col + col_delta;
            let removed = self.lines[start_row]
                .drain(start_offset..end_offset)
                .as_str()
                .to_string();
            self.yank = removed.clone().into();
            self.push_history(
                TextBoxEditKind::DeleteStr(removed),
                CursorPos::new(start_row, end_col, end_offset),
                start_offset,
            );
            return true;
        }

        let mut r = start_row + 1;
        let mut offset = 0;
        let mut col = 0;

        while r < self.lines.len() {
            let line = &self.lines[r];
            if let Some((o, c)) = find_end(line) {
                offset = o;
                col = c;
                break;
            }
            r += 1;
        }

        let start = CursorPos::new(start_row, start_col, start_offset);
        let end = CursorPos::new(r, col, offset);
        self.delete_range(start, end, true);
        true
    }

    fn delete_piece(&mut self, col: usize, chars: usize) -> bool {
        if chars == 0 {
            return false;
        }

        #[inline]
        fn bytes_and_chars(claimed: usize, s: &str) -> (usize, usize) {
            // Note: `claimed` may be larger than characters in `s` (e.g. usize::MAX)
            let mut last_col = 0;
            for (col, (bytes, _)) in s.char_indices().enumerate() {
                if col == claimed {
                    return (bytes, claimed);
                }
                last_col = col;
            }
            (s.len(), last_col + 1)
        }

        let (row, _) = self.cursor;
        let line = &mut self.lines[row];
        if let Some((i, _)) = line.char_indices().nth(col) {
            let (bytes, chars) = bytes_and_chars(chars, &line[i..]);
            let removed = line.drain(i..i + bytes).as_str().to_string();

            self.cursor = (row, col);
            self.push_history(
                TextBoxEditKind::DeleteStr(removed.clone()),
                CursorPos::new(row, col + chars, i + bytes),
                i,
            );
            self.yank = removed.into();
            true
        } else {
            false
        }
    }

    pub fn insert_tab(&mut self) -> bool {
        let modified = self.delete_selection(false);
        if self.tab_len == 0 {
            return modified;
        }

        if self.hard_tab_indent {
            self.insert_char('\t');
            return true;
        }

        let (row, col) = self.cursor;
        let width: usize = self.lines[row]
            .chars()
            .take(col)
            .map(|c| c.width().unwrap_or(0))
            .sum();
        let len = self.tab_len - (width % self.tab_len as usize) as u8;
        self.insert_piece(spaces(len).to_string())
    }

    pub fn insert_newline(&mut self) {
        self.delete_selection(false);

        let (row, col) = self.cursor;
        let line = &mut self.lines[row];
        let offset = line
            .char_indices()
            .nth(col)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        let next_line = line[offset..].to_string();
        line.truncate(offset);

        self.lines.insert(row + 1, next_line);
        self.cursor = (row + 1, 0);
        self.push_history(
            TextBoxEditKind::InsertNewline,
            CursorPos::new(row, col, offset),
            0,
        );
    }

    pub fn delete_newline(&mut self) -> bool {
        if self.delete_selection(false) {
            return true;
        }

        let (row, _) = self.cursor;
        if row == 0 {
            return false;
        }

        let line = self.lines.remove(row);
        let prev_line = &mut self.lines[row - 1];
        let prev_line_end = prev_line.len();

        self.cursor = (row - 1, prev_line.chars().count());
        prev_line.push_str(&line);
        self.push_history(
            TextBoxEditKind::DeleteNewline,
            CursorPos::new(row, 0, 0),
            prev_line_end,
        );
        true
    }

    pub fn delete_char(&mut self) -> bool {
        if self.delete_selection(false) {
            return true;
        }

        let (row, col) = self.cursor;
        if col == 0 {
            return self.delete_newline();
        }

        let line = &mut self.lines[row];
        if let Some((offset, c)) = line.char_indices().nth(col - 1) {
            line.remove(offset);
            self.cursor.1 -= 1;
            self.push_history(
                TextBoxEditKind::DeleteChar(c),
                CursorPos::new(row, col, offset + c.len_utf8()),
                offset,
            );
            true
        } else {
            false
        }
    }

    pub fn delete_next_char(&mut self) -> bool {
        if self.delete_selection(false) {
            return true;
        }

        let before = self.cursor;
        self.move_cursor_with_shift(CursorMove::Forward, false);
        if before == self.cursor {
            return false; // Cursor didn't move, meant no character at next of cursor.
        }

        self.delete_char()
    }

    pub fn delete_line_by_end(&mut self) -> bool {
        if self.delete_selection(false) {
            return true;
        }
        if self.delete_piece(self.cursor.1, usize::MAX) {
            return true;
        }
        self.delete_next_char() // At the end of the line. Try to delete next line
    }

    pub fn delete_line_by_head(&mut self) -> bool {
        if self.delete_selection(false) {
            return true;
        }
        if self.delete_piece(0, self.cursor.1) {
            return true;
        }
        self.delete_newline()
    }

    pub fn delete_word(&mut self) -> bool {
        if self.delete_selection(false) {
            return true;
        }
        let (r, c) = self.cursor;
        if let Some(col) = find_word_start_backward(&self.lines[r], c) {
            self.delete_piece(col, c - col)
        } else if c > 0 {
            self.delete_piece(0, c)
        } else {
            self.delete_newline()
        }
    }

    pub fn delete_next_word(&mut self) -> bool {
        if self.delete_selection(false) {
            return true;
        }
        let (r, c) = self.cursor;
        let line = &self.lines[r];
        if let Some(col) = find_word_end_forward(line, c) {
            self.delete_piece(c, col - c)
        } else {
            let end_col = line.chars().count();
            if c < end_col {
                self.delete_piece(c, end_col - c)
            } else if r + 1 < self.lines.len() {
                self.cursor = (r + 1, 0);
                self.delete_newline()
            } else {
                false
            }
        }
    }

    pub fn select_all(&mut self) {
        self.move_cursor(CursorMove::Jump(u16::MAX, u16::MAX));
        self.selection_start = Some((0, 0));
    }

    pub fn move_cursor(&mut self, m: CursorMove) {
        self.move_cursor_with_shift(m, self.selection_start.is_some());
    }

    pub fn undo(&mut self) -> bool {
        if let Some(cursor) = self.history.undo(&mut self.lines) {
            self.cancel_selection();
            self.cursor = cursor;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(cursor) = self.history.redo(&mut self.lines) {
            self.cancel_selection();
            self.cursor = cursor;
            true
        } else {
            false
        }
    }

    pub(crate) fn get_formatted_line<'b>(
        &'b self,
        line: &'b str,
        row: usize,
        lnum_len: u8,
    ) -> Line<'b> {
        let mut hl = TextLineFormatter::new(
            line,
            self.cursor_style,
            self.tab_len,
            self.mask,
            self.select_style,
        );

        if let Some(style) = self.line_number_style {
            hl.line_number(row, lnum_len, style);
        }

        if row == self.cursor.0 {
            hl.cursor_line(self.cursor.1, self.cursor_line_style);
        }

        if let Some((start, end)) = self.selection_range() {
            hl.selection(row, start.row, start.offset, end.row, end.offset);
        }

        hl.into_line()
    }

    pub fn widget(&'a self) -> impl Widget + 'a {
        TextBoxRenderer::new(self)
    }

    pub fn style(&self) -> Style {
        self.style
    }

    pub fn set_block(&mut self, block: Block<'a>) {
        self.block = Some(block);
    }

    pub fn block<'s>(&'s self) -> Option<&'s Block<'a>> {
        self.block.as_ref()
    }

    pub fn set_line_number_style(&mut self, style: Style) {
        self.line_number_style = Some(style);
    }

    pub fn remove_line_number(&mut self) {
        self.line_number_style = None;
    }

    pub fn lines(&'a self) -> &'a [String] {
        &self.lines
    }

    pub fn cursor(&self) -> (usize, usize) {
        self.cursor
    }

    pub fn set_alignment(&mut self, alignment: Alignment) {
        if let Alignment::Center | Alignment::Right = alignment {
            self.line_number_style = None;
        }
        self.alignment = alignment;
    }

    pub fn alignment(&self) -> Alignment {
        self.alignment
    }

    pub fn is_empty(&self) -> bool {
        self.lines == [""]
    }

    pub fn scroll(&mut self, scrolling: impl Into<TextBoxScroll>) {
        self.scroll_with_shift(scrolling.into(), self.selection_start.is_some());
    }

    fn scroll_with_shift(&mut self, scrolling: TextBoxScroll, shift: bool) {
        if shift && self.selection_start.is_none() {
            self.selection_start = Some(self.cursor);
        }
        scrolling.scroll(&mut self.viewport);
        self.move_cursor_with_shift(CursorMove::InViewport, shift);
    }

    pub fn start_selection(&mut self) {
        self.selection_start = Some(self.cursor);
    }

    pub fn cancel_selection(&mut self) {
        self.selection_start = None;
    }

    fn move_cursor_with_shift(&mut self, m: CursorMove, shift: bool) {
        if let Some(cursor) = m.next_cursor(self.cursor, &self.lines, &self.viewport) {
            if shift {
                if self.selection_start.is_none() {
                    self.start_selection();
                }
            } else {
                self.cancel_selection();
            }
            self.cursor = cursor;
        }
    }

    pub fn get_non_ascii_aware_cursor_x_pos(&self) -> usize {
        let (row, col) = self.cursor;
        let line = &self.lines[row];
        let mut raw_length = 0;
        for c in line.chars().take(col) {
            raw_length += c.width().unwrap_or_default();
        }
        raw_length
    }
}

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
    fn apply(&self, lines: &mut Vec<String>, before: &CursorPos, after: &CursorPos) {
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

    fn invert(&self) -> Self {
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

#[derive(Clone, Debug)]
pub struct TextBoxEdit {
    kind: TextBoxEditKind,
    before: CursorPos,
    after: CursorPos,
}

impl TextBoxEdit {
    pub fn new(kind: TextBoxEditKind, before: CursorPos, after: CursorPos) -> Self {
        Self {
            kind,
            before,
            after,
        }
    }

    pub fn redo(&self, lines: &mut Vec<String>) {
        self.kind.apply(lines, &self.before, &self.after);
    }

    pub fn undo(&self, lines: &mut Vec<String>) {
        self.kind.invert().apply(lines, &self.after, &self.before); // Undo is redo of inverted edit
    }

    pub fn cursor_before(&self) -> (usize, usize) {
        (self.before.row, self.before.col)
    }

    pub fn cursor_after(&self) -> (usize, usize) {
        (self.after.row, self.after.col)
    }
}

#[derive(Clone, Debug)]
pub struct TextBoxHistory {
    index: usize,
    max_items: usize,
    edits: VecDeque<TextBoxEdit>,
}

impl TextBoxHistory {
    pub fn new(max_items: usize) -> Self {
        Self {
            index: 0,
            max_items,
            edits: VecDeque::new(),
        }
    }

    pub fn push(&mut self, edit: TextBoxEdit) {
        if self.max_items == 0 {
            return;
        }

        if self.edits.len() == self.max_items {
            self.edits.pop_front();
            self.index = self.index.saturating_sub(1);
        }

        if self.index < self.edits.len() {
            self.edits.truncate(self.index);
        }

        self.index += 1;
        self.edits.push_back(edit);
    }

    pub fn redo(&mut self, lines: &mut Vec<String>) -> Option<(usize, usize)> {
        if self.index == self.edits.len() {
            return None;
        }
        let edit = &self.edits[self.index];
        edit.redo(lines);
        self.index += 1;
        Some(edit.cursor_after())
    }

    pub fn undo(&mut self, lines: &mut Vec<String>) -> Option<(usize, usize)> {
        self.index = self.index.checked_sub(1)?;
        let edit = &self.edits[self.index];
        edit.undo(lines);
        Some(edit.cursor_before())
    }

    pub fn max_items(&self) -> usize {
        self.max_items
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum CharKind {
    Space,
    Punctuation,
    Other,
}

impl CharKind {
    fn new(c: char) -> Self {
        if c.is_whitespace() {
            Self::Space
        } else if c.is_ascii_punctuation() {
            Self::Punctuation
        } else {
            Self::Other
        }
    }
}

pub fn find_word_start_forward(line: &str, start_col: usize) -> Option<usize> {
    let mut it = line.chars().enumerate().skip(start_col);
    let mut prev = CharKind::new(it.next()?.1);
    for (col, c) in it {
        let cur = CharKind::new(c);
        if cur != CharKind::Space && prev != cur {
            return Some(col);
        }
        prev = cur;
    }
    None
}

pub fn find_word_end_forward(line: &str, start_col: usize) -> Option<usize> {
    let mut it = line.chars().enumerate().skip(start_col);
    let mut prev = CharKind::new(it.next()?.1);
    for (col, c) in it {
        let cur = CharKind::new(c);
        if prev != CharKind::Space && prev != cur {
            return Some(col);
        }
        prev = cur;
    }
    None
}

pub fn find_word_start_backward(line: &str, start_col: usize) -> Option<usize> {
    let idx = line
        .char_indices()
        .nth(start_col)
        .map(|(i, _)| i)
        .unwrap_or(line.len());
    let mut it = line[..idx].chars().rev().enumerate();
    let mut cur = CharKind::new(it.next()?.1);
    for (i, c) in it {
        let next = CharKind::new(c);
        if cur != CharKind::Space && next != cur {
            return Some(start_col - i);
        }
        cur = next;
    }
    (cur != CharKind::Space).then_some(0)
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

#[derive(Default, Debug)]
pub struct TextBoxViewport(AtomicU64);

impl Clone for TextBoxViewport {
    fn clone(&self) -> Self {
        let u = self.0.load(std::sync::atomic::Ordering::Relaxed);
        TextBoxViewport(AtomicU64::new(u))
    }
}

impl TextBoxViewport {
    pub fn scroll_top(&self) -> (u16, u16) {
        let u = self.0.load(std::sync::atomic::Ordering::Relaxed);
        ((u >> 16) as u16, u as u16)
    }

    pub fn rect(&self) -> (u16, u16, u16, u16) {
        let u = self.0.load(std::sync::atomic::Ordering::Relaxed);
        let width = (u >> 48) as u16;
        let height = (u >> 32) as u16;
        let row = (u >> 16) as u16;
        let col = u as u16;
        (row, col, width, height)
    }

    pub fn position(&self) -> (u16, u16, u16, u16) {
        let (row_top, col_top, width, height) = self.rect();
        let row_bottom = row_top.saturating_add(height).saturating_sub(1);
        let col_bottom = col_top.saturating_add(width).saturating_sub(1);

        (
            row_top,
            col_top,
            cmp::max(row_top, row_bottom),
            cmp::max(col_top, col_bottom),
        )
    }

    fn store(&self, row: u16, col: u16, width: u16, height: u16) {
        let u =
            ((width as u64) << 48) | ((height as u64) << 32) | ((row as u64) << 16) | col as u64;
        self.0.store(u, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn scroll(&mut self, rows: i16, cols: i16) {
        fn apply_scroll(pos: u16, delta: i16) -> u16 {
            if delta >= 0 {
                pos.saturating_add(delta as u16)
            } else {
                pos.saturating_sub(-delta as u16)
            }
        }

        let u = self.0.get_mut();
        let row = apply_scroll((*u >> 16) as u16, rows);
        let col = apply_scroll(*u as u16, cols);
        *u = (*u & 0xffff_ffff_0000_0000) | ((row as u64) << 16) | (col as u64);
    }
}

pub struct TextBoxRenderer<'a>(&'a TextBox<'a>);

impl<'a> TextBoxRenderer<'a> {
    pub fn new(textarea: &'a TextBox<'a>) -> Self {
        Self(textarea)
    }

    #[inline]
    fn text(&self, top_row: usize, height: usize) -> Text<'a> {
        let lines_len = self.0.lines().len();
        let line_num_len = num_digits(lines_len);
        let bottom_row = cmp::min(top_row + height, lines_len);
        let mut lines = Vec::with_capacity(bottom_row - top_row);
        for (i, line) in self.0.lines()[top_row..bottom_row].iter().enumerate() {
            lines.push(
                self.0
                    .get_formatted_line(line.as_str(), top_row + i, line_num_len),
            );
        }
        Text::from(lines)
    }
}

impl<'a> Widget for TextBoxRenderer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Rect { width, height, .. } = if let Some(b) = self.0.block() {
            b.inner(area)
        } else {
            area
        };

        fn next_scroll_top(prev_top: u16, cursor: u16, length: u16) -> u16 {
            if cursor < prev_top {
                cursor
            } else if prev_top + length <= cursor {
                cursor + 1 - length
            } else {
                prev_top
            }
        }

        let cursor = self.0.cursor();
        let (top_row, top_col) = self.0.viewport.scroll_top();
        let top_row = next_scroll_top(top_row, cursor.0 as u16, height);
        let top_col = next_scroll_top(top_col, cursor.1 as u16, width);

        let (text, style) = if !self.0.placeholder.is_empty() && self.0.is_empty() {
            let text = Text::from(self.0.placeholder.as_str());
            (text, self.0.placeholder_style)
        } else {
            (self.text(top_row as usize, height as usize), self.0.style())
        };

        let mut text_area = area;
        let mut inner = Paragraph::new(text)
            .style(style)
            .alignment(self.0.alignment());
        if let Some(b) = self.0.block() {
            text_area = b.inner(area);
            b.clone().render(area, buf)
        }
        if top_col != 0 {
            inner = inner.scroll((0, top_col));
        }

        self.0.viewport.store(top_row, top_col, width, height);

        inner.render(text_area, buf);
    }
}

pub struct TextLineFormatter<'a> {
    line: &'a str,
    spans: Vec<Span<'a>>,
    boundaries: Vec<(Boundary, usize)>,
    style_begin: Style,
    cursor_at_end: bool,
    cursor_style: Style,
    tab_len: u8,
    mask: Option<char>,
    select_at_end: bool,
    select_style: Style,
}

impl<'a> TextLineFormatter<'a> {
    pub fn new(
        line: &'a str,
        cursor_style: Style,
        tab_len: u8,
        mask: Option<char>,
        select_style: Style,
    ) -> Self {
        Self {
            line,
            spans: vec![],
            boundaries: vec![],
            style_begin: Style::default(),
            cursor_at_end: false,
            cursor_style,
            tab_len,
            mask,
            select_at_end: false,
            select_style,
        }
    }

    pub fn line_number(&mut self, row: usize, line_num_len: u8, style: Style) {
        let pad = spaces(line_num_len - num_digits(row + 1) + 1);
        self.spans
            .push(Span::styled(format!("{}{}) ", pad, row + 1), style));
    }

    pub fn cursor_line(&mut self, cursor_col: usize, style: Style) {
        if let Some((start, c)) = self.line.char_indices().nth(cursor_col) {
            self.boundaries
                .push((Boundary::Cursor(self.cursor_style), start));
            self.boundaries.push((Boundary::End, start + c.len_utf8()));
        } else {
            self.cursor_at_end = true;
        }
        self.style_begin = style;
    }

    pub fn selection(
        &mut self,
        current_row: usize,
        start_row: usize,
        start_off: usize,
        end_row: usize,
        end_off: usize,
    ) {
        let (start, end) = if current_row == start_row {
            if start_row == end_row {
                (start_off, end_off)
            } else {
                self.select_at_end = true;
                (start_off, self.line.len())
            }
        } else if current_row == end_row {
            (0, end_off)
        } else if start_row < current_row && current_row < end_row {
            self.select_at_end = true;
            (0, self.line.len())
        } else {
            return;
        };
        if start != end {
            self.boundaries
                .push((Boundary::Select(self.select_style), start));
            self.boundaries.push((Boundary::End, end));
        }
    }

    pub fn into_line(self) -> Line<'a> {
        let Self {
            line,
            mut spans,
            mut boundaries,
            tab_len,
            style_begin,
            cursor_style,
            cursor_at_end,
            mask,
            select_at_end,
            select_style,
        } = self;
        let mut builder = DisplayTextBuilder::new(tab_len, mask);

        if boundaries.is_empty() {
            let built = builder.build(line);
            if !built.is_empty() {
                spans.push(Span::styled(built, style_begin));
            }
            if cursor_at_end {
                spans.push(Span::styled(" ", cursor_style));
            } else if select_at_end {
                spans.push(Span::styled(" ", select_style));
            }
            return Line::from(spans);
        }

        boundaries.sort_unstable_by(|(l, i), (r, j)| match i.cmp(j) {
            Ordering::Equal => l.cmp(r),
            o => o,
        });

        let mut style = style_begin;
        let mut start = 0;
        let mut stack = vec![];

        for (next_boundary, end) in boundaries {
            if start < end {
                spans.push(Span::styled(builder.build(&line[start..end]), style));
            }

            style = if let Some(s) = next_boundary.style() {
                stack.push(style);
                s
            } else {
                stack.pop().unwrap_or(style_begin)
            };
            start = end;
        }

        if start != line.len() {
            spans.push(Span::styled(builder.build(&line[start..]), style));
        }

        if cursor_at_end {
            spans.push(Span::styled(" ", cursor_style));
        } else if select_at_end {
            spans.push(Span::styled(" ", select_style));
        }

        Line::from(spans)
    }
}

struct DisplayTextBuilder {
    tab_len: u8,
    width: usize,
    mask: Option<char>,
}

impl DisplayTextBuilder {
    fn new(tab_len: u8, mask: Option<char>) -> Self {
        Self {
            tab_len,
            width: 0,
            mask,
        }
    }

    fn build<'s>(&mut self, s: &'s str) -> Cow<'s, str> {
        if let Some(ch) = self.mask {
            // Note: We don't need to track width on masking text since width of tab character is fixed
            let masked = iter::repeat(ch).take(s.chars().count()).collect();
            return Cow::Owned(masked);
        }

        let tab = spaces(self.tab_len);
        let mut buf = String::new();
        for (i, c) in s.char_indices() {
            if c == '\t' {
                if buf.is_empty() {
                    buf.reserve(s.len());
                    buf.push_str(&s[..i]);
                }
                if self.tab_len > 0 {
                    let len = self.tab_len as usize - (self.width % self.tab_len as usize);
                    buf.push_str(&tab[..len]);
                    self.width += len;
                }
            } else {
                if !buf.is_empty() {
                    buf.push(c);
                }
                self.width += c.width().unwrap_or(0);
            }
        }

        if !buf.is_empty() {
            Cow::Owned(buf)
        } else {
            Cow::Borrowed(s)
        }
    }
}

#[derive(Debug, Clone)]
enum YankText {
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

enum Boundary {
    Cursor(Style),
    Select(Style),
    End,
}

impl Boundary {
    fn cmp(&self, other: &Boundary) -> std::cmp::Ordering {
        fn rank(b: &Boundary) -> u8 {
            match b {
                Boundary::Cursor(_) => 2,
                Boundary::Select(_) => 1,
                Boundary::End => 0,
            }
        }
        rank(self).cmp(&rank(other))
    }

    fn style(&self) -> Option<Style> {
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
