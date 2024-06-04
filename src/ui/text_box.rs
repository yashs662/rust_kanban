// This implementation is a stripped down version inspired by https://github.com/rhysd/tui-textarea

use crate::{
    inputs::key::Key,
    util::{num_digits, replace_tabs, spaces},
};
use portable_atomic::{AtomicU64, Ordering};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget},
};
use std::{cmp, collections::VecDeque};

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
    alignment: Alignment,
    pub single_line_mode: bool,
    placeholder: Option<String>,
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
            alignment: Alignment::Left,
            single_line_mode,
            placeholder: None,
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

    pub fn set_placeholder_text<S: Into<String>>(&mut self, text: S) {
        self.placeholder = Some(text.into());
    }

    pub fn remove_placeholder_text(&mut self) {
        self.placeholder = None;
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
            Key::Ctrl('p') | Key::Up => {
                if self.single_line_mode {
                    return false;
                }
                self.move_cursor(CursorMove::Up);
                false
            }
            Key::Ctrl('f') | Key::Right => {
                self.move_cursor(CursorMove::Forward);
                false
            }
            Key::Ctrl('b') | Key::Left => {
                self.move_cursor(CursorMove::Back);
                false
            }
            Key::Ctrl('a') | Key::Home | Key::CtrlAlt('b') | Key::CtrlAltLeft => {
                self.move_cursor(CursorMove::Head);
                false
            }
            Key::Ctrl('e') | Key::End | Key::CtrlAltRight | Key::CtrlAlt('f') => {
                self.move_cursor(CursorMove::End);
                false
            }
            Key::Alt('<') | Key::CtrlAltUp | Key::CtrlAlt('p') => {
                if self.single_line_mode {
                    return false;
                }
                self.move_cursor(CursorMove::Top);
                false
            }
            Key::Alt('>') | Key::CtrlAltDown | Key::CtrlAlt('n') => {
                if self.single_line_mode {
                    return false;
                }
                self.move_cursor(CursorMove::Bottom);
                false
            }
            Key::Alt('f') | Key::CtrlRight => {
                self.move_cursor(CursorMove::WordForward);
                false
            }
            Key::Alt('b') | Key::CtrlLeft => {
                self.move_cursor(CursorMove::WordBack);
                false
            }
            Key::Alt(']') | Key::Alt('n') | Key::CtrlDown => {
                self.move_cursor(CursorMove::ParagraphForward);
                false
            }
            Key::Alt('[') | Key::Alt('p') | Key::CtrlUp => {
                self.move_cursor(CursorMove::ParagraphBack);
                false
            }
            Key::Ctrl('z') => self.undo(),
            Key::Ctrl('y') => self.redo(),
            Key::Ctrl('v') | Key::PageDown => {
                if self.single_line_mode {
                    return false;
                }
                self.scroll(TextBoxScroll::PageDown);
                false
            }
            Key::Alt('v') | Key::PageUp => {
                if self.single_line_mode {
                    return false;
                }
                self.scroll(TextBoxScroll::PageUp);
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

    fn push_history(&mut self, kind: TextBoxEditKind, cursor_before: (usize, usize)) {
        let edit = TextBoxEdit::new(kind, cursor_before, self.cursor);
        self.history.push(edit);
    }

    pub fn insert_char(&mut self, c: char) {
        let (row, col) = self.cursor;
        let line = &mut self.lines[row];
        let i = line
            .char_indices()
            .nth(col)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        line.insert(i, c);
        self.cursor.1 += 1;
        self.push_history(TextBoxEditKind::InsertChar(c, i), (row, col));
    }

    pub fn insert_str<S: Into<String>>(&mut self, s: S) -> bool {
        let s = s.into();
        if s.is_empty() {
            return false;
        }

        let (row, col) = self.cursor;
        let line = &mut self.lines[row];
        let new_insert_lines = s.chars().filter(|c| *c == '\n').count();

        if new_insert_lines > 0 {
            let mut new_lines = s
                .split('\n')
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            let new_line = new_lines.pop().unwrap();
            let i = line
                .char_indices()
                .nth(col)
                .map(|(i, _)| i)
                .unwrap_or(line.len());
            line.insert_str(i, &new_line);
            self.cursor.1 += new_line.chars().count();
            self.push_history(TextBoxEditKind::Insert(new_line, i), (row, col));
            for new_line in new_lines {
                self.lines.insert(row + 1, new_line);
                self.cursor = (row + 1, 0);
                self.push_history(TextBoxEditKind::InsertNewline(0), (row, col));
            }
            true
        } else {
            let i = line
                .char_indices()
                .nth(col)
                .map(|(i, _)| i)
                .unwrap_or(line.len());
            line.insert_str(i, &s);
            self.cursor.1 += s.chars().count();
            self.push_history(TextBoxEditKind::Insert(s, i), (row, col));
            true
        }
    }

    pub fn delete_str(&mut self, col: usize, chars: usize) -> bool {
        if chars == 0 {
            return false;
        }

        let cursor_before = self.cursor;
        let row = cursor_before.0;
        let line = &mut self.lines[row];
        if let Some((i, _)) = line.char_indices().nth(col) {
            let bytes = line[i..]
                .char_indices()
                .nth(chars)
                .map(|(i, _)| i)
                .unwrap_or_else(|| line[i..].len());
            let removed = line[i..i + bytes].to_string();
            line.replace_range(i..i + bytes, "");

            self.cursor = (row, col);
            self.push_history(TextBoxEditKind::Remove(removed.clone(), i), cursor_before);
            true
        } else {
            false
        }
    }

    pub fn insert_tab(&mut self) -> bool {
        if self.tab_len == 0 {
            return false;
        }
        let tab = if self.hard_tab_indent {
            "\t"
        } else {
            let len = self.tab_len - (self.cursor.1 % self.tab_len as usize) as u8;
            spaces(len)
        };
        self.insert_str(tab)
    }

    pub fn insert_newline(&mut self) {
        let (row, col) = self.cursor;
        let line = &mut self.lines[row];
        let idx = line
            .char_indices()
            .nth(col)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        let next_line = line[idx..].to_string();
        line.truncate(idx);

        self.lines.insert(row + 1, next_line);
        self.cursor = (row + 1, 0);
        self.push_history(TextBoxEditKind::InsertNewline(idx), (row, col));
    }

    pub fn delete_newline(&mut self) -> bool {
        let (row, col) = self.cursor;
        if row == 0 {
            return false;
        }

        let line = self.lines.remove(row);
        let prev_line = &mut self.lines[row - 1];
        let prev_line_end = prev_line.len();

        self.cursor = (row - 1, prev_line.chars().count());
        prev_line.push_str(&line);
        self.push_history(TextBoxEditKind::DeleteNewline(prev_line_end), (row, col));
        true
    }

    pub fn delete_char(&mut self) -> bool {
        let (row, col) = self.cursor;
        if col == 0 {
            return self.delete_newline();
        }

        let line = &mut self.lines[row];
        if let Some((i, c)) = line.char_indices().nth(col - 1) {
            line.remove(i);
            self.cursor.1 -= 1;
            self.push_history(TextBoxEditKind::DeleteChar(c, i), (row, col));
            true
        } else {
            false
        }
    }

    pub fn delete_next_char(&mut self) -> bool {
        let before = self.cursor;
        self.move_cursor(CursorMove::Forward);
        if before == self.cursor {
            return false;
        }
        self.delete_char()
    }

    pub fn delete_line_by_end(&mut self) -> bool {
        if self.delete_str(self.cursor.1, usize::MAX) {
            return true;
        }
        self.delete_next_char()
    }

    pub fn delete_line_by_head(&mut self) -> bool {
        if self.delete_str(0, self.cursor.1) {
            return true;
        }
        self.delete_newline()
    }

    pub fn delete_word(&mut self) -> bool {
        let (r, c) = self.cursor;
        if let Some(col) = find_word_start_backward(&self.lines[r], c) {
            self.delete_str(col, c - col)
        } else if c > 0 {
            self.delete_str(0, c)
        } else {
            self.delete_newline()
        }
    }

    pub fn delete_next_word(&mut self) -> bool {
        let (r, c) = self.cursor;
        let line = &self.lines[r];
        if let Some(col) = find_word_end_forward(line, c) {
            self.delete_str(c, col - c)
        } else {
            let end_col = line.chars().count();
            if c < end_col {
                self.delete_str(c, end_col - c)
            } else if r + 1 < self.lines.len() {
                self.cursor = (r + 1, 0);
                self.delete_newline()
            } else {
                false
            }
        }
    }

    pub fn move_cursor(&mut self, m: CursorMove) {
        if let Some(cursor) = m.next_cursor(self.cursor, &self.lines, &self.viewport) {
            self.cursor = cursor;
        }
    }

    pub fn undo(&mut self) -> bool {
        if let Some(cursor) = self.history.undo(&mut self.lines) {
            self.cursor = cursor;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(cursor) = self.history.redo(&mut self.lines) {
            self.cursor = cursor;
            true
        } else {
            false
        }
    }

    pub(crate) fn line_spans<'b>(&'b self, line: &'b str, row: usize, lnum_len: u8) -> Line<'b> {
        let mut hl = TextLineFormatter::new(line, self.cursor_style, self.tab_len);

        if let Some(style) = self.line_number_style {
            hl.line_number(row, lnum_len, style);
        }

        if row == self.cursor.0 {
            hl.cursor_line(self.cursor.1, self.cursor_line_style);
        }

        hl.into_line()
    }

    pub fn widget(&'a self) -> impl Widget + 'a {
        TextBoxRenderer::new(self)
    }

    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    pub fn style(&self) -> Style {
        self.style
    }

    pub fn set_block(&mut self, block: Block<'a>) {
        self.block = Some(block);
    }

    pub fn remove_block(&mut self) {
        self.block = None;
    }

    pub fn block<'s>(&'s self) -> Option<&'s Block<'a>> {
        self.block.as_ref()
    }

    pub fn set_tab_length(&mut self, len: u8) {
        self.tab_len = len;
    }

    pub fn tab_length(&self) -> u8 {
        self.tab_len
    }

    pub fn set_hard_tab_indent(&mut self, enabled: bool) {
        self.hard_tab_indent = enabled;
    }

    pub fn hard_tab_indent(&self) -> bool {
        self.hard_tab_indent
    }

    pub fn indent(&self) -> &'static str {
        if self.hard_tab_indent {
            "\t"
        } else {
            spaces(self.tab_len)
        }
    }

    pub fn set_max_histories(&mut self, max: usize) {
        self.history = TextBoxHistory::new(max);
    }

    pub fn max_histories(&self) -> usize {
        self.history.max_items()
    }

    pub fn set_cursor_line_style(&mut self, style: Style) {
        self.cursor_line_style = style;
    }

    pub fn cursor_line_style(&self) -> Style {
        self.cursor_line_style
    }

    pub fn set_line_number_style(&mut self, style: Style) {
        self.line_number_style = Some(style);
    }

    pub fn remove_line_number(&mut self) {
        self.line_number_style = None;
    }

    pub fn line_number_style(&self) -> Option<Style> {
        self.line_number_style
    }

    pub fn set_cursor_style(&mut self, style: Style) {
        self.cursor_style = style;
    }

    pub fn cursor_style(&self) -> Style {
        self.cursor_style
    }

    pub fn lines(&'a self) -> &'a [String] {
        &self.lines
    }

    pub fn into_lines(self) -> Vec<String> {
        self.lines
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
        scrolling.into().scroll(&mut self.viewport);
        self.move_cursor(CursorMove::InViewport);
    }
}

#[derive(Clone, Debug)]
pub enum TextBoxEditKind {
    InsertChar(char, usize),
    DeleteChar(char, usize),
    InsertNewline(usize),
    DeleteNewline(usize),
    Insert(String, usize),
    Remove(String, usize),
}

impl TextBoxEditKind {
    fn apply(&self, row: usize, lines: &mut Vec<String>) {
        match self {
            TextBoxEditKind::InsertChar(c, i) => {
                lines[row].insert(*i, *c);
            }
            TextBoxEditKind::DeleteChar(_, i) => {
                lines[row].remove(*i);
            }
            TextBoxEditKind::InsertNewline(i) => {
                let line = &mut lines[row];
                let next_line = line[*i..].to_string();
                line.truncate(*i);
                lines.insert(row + 1, next_line);
            }
            TextBoxEditKind::DeleteNewline(_) => {
                if row > 0 {
                    let line = lines.remove(row);
                    lines[row - 1].push_str(&line);
                }
            }
            TextBoxEditKind::Insert(s, i) => {
                lines[row].insert_str(*i, s.as_str());
            }
            TextBoxEditKind::Remove(s, i) => {
                let end = *i + s.len();
                lines[row].replace_range(*i..end, "");
            }
        }
    }

    fn invert(&self) -> Self {
        match self.clone() {
            TextBoxEditKind::InsertChar(c, i) => TextBoxEditKind::DeleteChar(c, i),
            TextBoxEditKind::DeleteChar(c, i) => TextBoxEditKind::InsertChar(c, i),
            TextBoxEditKind::InsertNewline(i) => TextBoxEditKind::DeleteNewline(i),
            TextBoxEditKind::DeleteNewline(i) => TextBoxEditKind::InsertNewline(i),
            TextBoxEditKind::Insert(s, i) => TextBoxEditKind::Remove(s, i),
            TextBoxEditKind::Remove(s, i) => TextBoxEditKind::Insert(s, i),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextBoxEdit {
    kind: TextBoxEditKind,
    cursor_before: (usize, usize),
    cursor_after: (usize, usize),
}

impl TextBoxEdit {
    pub fn new(
        kind: TextBoxEditKind,
        cursor_before: (usize, usize),
        cursor_after: (usize, usize),
    ) -> Self {
        Self {
            kind,
            cursor_before,
            cursor_after,
        }
    }

    pub fn redo(&self, lines: &mut Vec<String>) {
        let (row, _) = self.cursor_before;
        self.kind.apply(row, lines);
    }

    pub fn undo(&self, lines: &mut Vec<String>) {
        let (row, _) = self.cursor_after;
        self.kind.invert().apply(row, lines); // Undo is redo of inverted edit
    }

    pub fn cursor_before(&self) -> (usize, usize) {
        self.cursor_before
    }

    pub fn cursor_after(&self) -> (usize, usize) {
        self.cursor_after
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
    Punct,
    Other,
}

impl CharKind {
    fn new(c: char) -> Self {
        if c.is_whitespace() {
            Self::Space
        } else if c.is_ascii_punctuation() {
            Self::Punct
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

#[derive(Default, Debug)]
pub struct TextBoxViewport(AtomicU64);

impl Clone for TextBoxViewport {
    fn clone(&self) -> Self {
        let u = self.0.load(Ordering::Relaxed);
        TextBoxViewport(AtomicU64::new(u))
    }
}

impl TextBoxViewport {
    pub fn scroll_top(&self) -> (u16, u16) {
        let u = self.0.load(Ordering::Relaxed);
        ((u >> 16) as u16, u as u16)
    }

    pub fn rect(&self) -> (u16, u16, u16, u16) {
        let u = self.0.load(Ordering::Relaxed);
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
        self.0.store(u, Ordering::Relaxed);
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
        let lnum_len = num_digits(lines_len) - 1; // Required for correct viewport calculation
        let bottom_row = cmp::min(top_row + height, lines_len);
        let mut lines = Vec::with_capacity(bottom_row - top_row);
        if self.0.is_empty() && self.0.placeholder.is_some() {
            for (i, line) in self.0.lines()[top_row..bottom_row].iter().enumerate() {
                if i == 0 {
                    lines.push(self.0.line_spans(
                        self.0.placeholder.as_ref().unwrap(),
                        top_row,
                        lnum_len,
                    ));
                } else {
                    lines.push(self.0.line_spans(line.as_str(), top_row + i, lnum_len));
                }
            }
        } else {
            for (i, line) in self.0.lines()[top_row..bottom_row].iter().enumerate() {
                lines.push(self.0.line_spans(line.as_str(), top_row + i, lnum_len));
            }
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

        // Compensating for line numbers
        let width = if self.0.line_number_style.is_some() {
            width.saturating_sub(3)
        } else {
            width
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

        let text = self.text(top_row as usize, height as usize);
        let mut inner = Paragraph::new(text)
            .style(self.0.style())
            .alignment(self.0.alignment());
        if let Some(b) = self.0.block() {
            inner = inner.block(b.clone());
        }
        if top_col != 0 {
            inner = inner.scroll((0, top_col));
        }

        self.0.viewport.store(top_row, top_col, width, height);

        inner.render(area, buf);
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
}

impl<'a> TextLineFormatter<'a> {
    pub fn new(line: &'a str, cursor_style: Style, tab_len: u8) -> Self {
        Self {
            line,
            spans: vec![],
            boundaries: vec![],
            style_begin: Style::default(),
            cursor_at_end: false,
            cursor_style,
            tab_len,
        }
    }

    pub fn line_number(&mut self, row: usize, lnum_len: u8, style: Style) {
        let pad = spaces(lnum_len - num_digits(row + 1) + 1);
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

    pub fn into_line(self) -> Line<'a> {
        let Self {
            line,
            mut spans,
            mut boundaries,
            tab_len,
            style_begin,
            cursor_style,
            cursor_at_end,
        } = self;

        if boundaries.is_empty() {
            spans.push(Span::styled(replace_tabs(line, tab_len), style_begin));
            if cursor_at_end {
                spans.push(Span::styled(" ", cursor_style));
            }
            return Line::from(spans);
        }

        boundaries.sort_unstable_by(|(l, i), (r, j)| match i.cmp(j) {
            std::cmp::Ordering::Equal => l.cmp(r),
            o => o,
        });

        let mut boundaries = boundaries.into_iter();
        let mut style = style_begin;
        let mut start = 0;
        let mut stack = vec![];

        loop {
            if let Some((next_boundary, end)) = boundaries.next() {
                if start < end {
                    spans.push(Span::styled(
                        replace_tabs(&line[start..end], tab_len),
                        style,
                    ));
                }

                style = if let Some(s) = next_boundary.style() {
                    stack.push(style);
                    s
                } else {
                    stack.pop().unwrap_or(style_begin)
                };
                start = end;
            } else {
                if start != line.len() {
                    spans.push(Span::styled(replace_tabs(&line[start..], tab_len), style));
                }
                if cursor_at_end {
                    spans.push(Span::styled(" ", cursor_style));
                }
                return Line::from(spans);
            }
        }
    }
}

enum Boundary {
    Cursor(Style),
    End,
}

impl Boundary {
    fn cmp(&self, other: &Boundary) -> std::cmp::Ordering {
        fn rank(b: &Boundary) -> u8 {
            match b {
                Boundary::Cursor(_) => 2,
                Boundary::End => 0,
            }
        }
        rank(self).cmp(&rank(other))
    }

    fn style(&self) -> Option<Style> {
        match self {
            Boundary::Cursor(s) => Some(*s),
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
