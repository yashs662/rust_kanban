// This is a stripped down version of the tui_logger crate to be used in rust-kanban,
// i have not written this code myself. link to github: https://github.com/gin66/tui-logger
// This is done to avoid future conflicts with the ratatui crate having different versions of the same crate.

use chrono::{DateTime, Local};
use log::{Level, LevelFilter, Log, Metadata, Record};
use parking_lot::Mutex;
use ratatui::widgets::ListState;
use std::{
    collections::{
        hash_map::{Iter, Keys},
        HashMap,
    },
    iter, mem,
};

#[derive(Clone, Debug)]
pub struct CircularBuffer<T> {
    pub buffer: Vec<T>,
    next_write_pos: usize,
}

#[derive(Default, Debug)]
pub struct LevelConfig {
    config: HashMap<String, LevelFilter>,
    generation: u64,
    default_display_level: Option<LevelFilter>,
}

#[derive(Clone, Debug)]
pub struct ExtLogRecord {
    pub timestamp: DateTime<Local>,
    pub level: Level,
    pub msg: String,
}

#[derive(Debug)]
struct HotSelect {
    hashtable: HashMap<u64, LevelFilter>,
    default: LevelFilter,
}

#[derive(Debug)]
pub struct HotLog {
    pub events: CircularBuffer<ExtLogRecord>,
    pub state: ListState,
}

#[derive(Debug)]
pub struct RustKanbanLoggerInner {
    hot_depth: usize,
    pub events: CircularBuffer<ExtLogRecord>,
    pub total_events: usize,
    default: LevelFilter,
    targets: LevelConfig,
}

#[derive(Debug)]
pub struct RustKanbanLogger {
    hot_select: Mutex<HotSelect>,
    pub hot_log: Mutex<HotLog>,
    pub inner: Mutex<RustKanbanLoggerInner>,
}

impl RustKanbanLogger {
    pub fn move_events(&self) {
        // If there are no new events, then just return
        if self.hot_log.lock().events.total_elements() == 0 {
            return;
        }
        // Exchange new event buffer with the hot buffer
        let mut received_events = {
            let new_circular = CircularBuffer::new(self.inner.lock().hot_depth);
            let mut hl = self.hot_log.lock();
            mem::replace(&mut hl.events, new_circular)
        };
        let mut tli = self.inner.lock();
        let total = received_events.total_elements();
        let elements = received_events.len();
        tli.total_events += total;
        let mut consumed = received_events.take();
        let mut reversed = Vec::with_capacity(consumed.len() + 1);
        while let Some(log_entry) = consumed.pop() {
            reversed.push(log_entry);
        }
        if total > elements {
            // Too many events received, so some have been lost
            let new_log_entry = ExtLogRecord {
                timestamp: reversed[reversed.len() - 1].timestamp,
                level: Level::Warn,
                msg: format!(
                    "There have been {} events lost, {} recorded out of {}",
                    total - elements,
                    elements,
                    total
                ),
            };
            reversed.push(new_log_entry);
        }
        while let Some(log_entry) = reversed.pop() {
            tli.events.push(log_entry);
        }
    }
}

impl<T> CircularBuffer<T> {
    pub fn new(max_depth: usize) -> CircularBuffer<T> {
        CircularBuffer {
            buffer: Vec::with_capacity(max_depth),
            next_write_pos: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
    pub fn push(&mut self, elem: T) {
        let max_depth = self.buffer.capacity();
        if self.buffer.len() < max_depth {
            self.buffer.push(elem);
        } else {
            self.buffer[self.next_write_pos % max_depth] = elem;
        }
        self.next_write_pos += 1;
    }
    pub fn take(&mut self) -> Vec<T> {
        let mut consumed = vec![];
        let max_depth = self.buffer.capacity();
        if self.buffer.len() < max_depth {
            consumed.append(&mut self.buffer);
        } else {
            let pos = self.next_write_pos % max_depth;
            let mut xvec = self.buffer.split_off(pos);
            consumed.append(&mut xvec);
            consumed.append(&mut self.buffer)
        }
        self.next_write_pos = 0;
        consumed
    }
    pub fn total_elements(&self) -> usize {
        self.next_write_pos
    }
    pub fn has_wrapped(&self) -> bool {
        self.next_write_pos > self.buffer.capacity()
    }
    pub fn iter(&mut self) -> iter::Chain<std::slice::Iter<T>, std::slice::Iter<T>> {
        let max_depth = self.buffer.capacity();
        if self.next_write_pos <= max_depth {
            // If buffer is not completely filled, then just iterate through it
            self.buffer.iter().chain(self.buffer[..0].iter())
        } else {
            let wrap = self.next_write_pos % max_depth;
            let it_end = self.buffer[..wrap].iter();
            let it_start = self.buffer[wrap..].iter();
            it_start.chain(it_end)
        }
    }
    pub fn rev_iter(
        &mut self,
    ) -> iter::Chain<std::iter::Rev<std::slice::Iter<T>>, std::iter::Rev<std::slice::Iter<T>>> {
        let max_depth = self.buffer.capacity();
        if self.next_write_pos <= max_depth {
            // If buffer is not completely filled, then just iterate through it
            self.buffer
                .iter()
                .rev()
                .chain(self.buffer[..0].iter().rev())
        } else {
            let wrap = self.next_write_pos % max_depth;
            let it_end = self.buffer[..wrap].iter().rev();
            let it_start = self.buffer[wrap..].iter().rev();
            it_end.chain(it_start)
        }
    }
}

impl LevelConfig {
    pub fn new() -> LevelConfig {
        LevelConfig {
            config: HashMap::new(),
            generation: 0,
            default_display_level: None,
        }
    }
    pub fn set(&mut self, target: &str, level: LevelFilter) {
        if let Some(lev) = self.config.get_mut(target) {
            if *lev != level {
                *lev = level;
                self.generation += 1;
            }
            return;
        }
        self.config.insert(target.to_string(), level);
        self.generation += 1;
    }
    pub fn set_default_display_level(&mut self, level: LevelFilter) {
        self.default_display_level = Some(level);
    }
    pub fn keys(&self) -> Keys<String, LevelFilter> {
        self.config.keys()
    }
    pub fn get(&self, target: &str) -> Option<&LevelFilter> {
        self.config.get(target)
    }
    pub fn iter(&self) -> Iter<String, LevelFilter> {
        self.config.iter()
    }
}

pub fn set_hot_buffer_depth(depth: usize) {
    RUST_KANBAN_LOGGER.inner.lock().hot_depth = depth;
}

pub fn move_events() {
    RUST_KANBAN_LOGGER.move_events();
}

pub fn set_default_level(levelfilter: LevelFilter) {
    RUST_KANBAN_LOGGER.hot_select.lock().default = levelfilter;
    RUST_KANBAN_LOGGER.inner.lock().default = levelfilter;
}

pub fn set_level_for_target(target: &str, levelfilter: LevelFilter) {
    let h = fxhash::hash64(&target);
    RUST_KANBAN_LOGGER
        .inner
        .lock()
        .targets
        .set(target, levelfilter);
    let mut hs = RUST_KANBAN_LOGGER.hot_select.lock();
    hs.hashtable.insert(h, levelfilter);
}

impl RustKanbanLogger {
    fn raw_log(&self, record: &Record) {
        let log_entry = ExtLogRecord {
            timestamp: chrono::Local::now(),
            level: record.level(),
            msg: format!("{}", record.args()),
        };
        let mut hot_log = self.hot_log.lock();
        hot_log.events.push(log_entry);
        let last_index = hot_log.events.len() - 1;
        hot_log.state.select(Some(last_index));
    }
}

impl Log for RustKanbanLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let h = fxhash::hash64(metadata.target());
        let hs = self.hot_select.lock();
        if let Some(&levelfilter) = hs.hashtable.get(&h) {
            metadata.level() <= levelfilter
        } else {
            metadata.level() <= hs.default
        }
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            self.raw_log(record)
        }
    }

    fn flush(&self) {}
}

lazy_static! {
    pub static ref RUST_KANBAN_LOGGER: RustKanbanLogger = {
        let hs = HotSelect {
            hashtable: HashMap::with_capacity(1000),
            default: LevelFilter::Info,
        };
        let hl = HotLog {
            events: CircularBuffer::new(1000),
            state: ListState::default(),
        };
        let tli = RustKanbanLoggerInner {
            hot_depth: 1000,
            events: CircularBuffer::new(10000),
            total_events: 0,
            default: LevelFilter::Info,
            targets: LevelConfig::new(),
        };
        RustKanbanLogger {
            hot_select: Mutex::new(hs),
            hot_log: Mutex::new(hl),
            inner: Mutex::new(tli),
        }
    };
}

pub fn init_logger(max_level: LevelFilter) -> Result<(), log::SetLoggerError> {
    log::set_max_level(max_level);
    log::set_logger(&*RUST_KANBAN_LOGGER)
}

pub fn get_logs() -> CircularBuffer<ExtLogRecord> {
    RUST_KANBAN_LOGGER.hot_log.lock().events.clone()
}

pub fn get_selected_index() -> usize {
    RUST_KANBAN_LOGGER
        .hot_log
        .lock()
        .state
        .selected()
        .unwrap_or(0)
}
