use crate::{
    ui::text_box::{
        helper_enums::{Boundary, TextBoxEditKind},
        TextBox,
    },
    util::{num_digits, spaces},
};
use portable_atomic::AtomicU64;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span, Text},
    widgets::{Paragraph, Widget},
};
use std::{
    borrow::Cow,
    cmp::{self, Ordering},
    collections::VecDeque,
    iter,
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
