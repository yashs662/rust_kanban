use super::{key::Key, mouse::Mouse, InputEvent};
use log::error;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

pub struct Events {
    rx: tokio::sync::mpsc::Receiver<InputEvent>,
    _tx: tokio::sync::mpsc::Sender<InputEvent>,
    stop_capture: Arc<AtomicBool>,
}

impl Events {
    pub fn new(tick_rate: Duration) -> Events {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let stop_capture = Arc::new(AtomicBool::new(false));

        let event_tx = tx.clone();
        let event_stop_capture = stop_capture.clone();
        tokio::spawn(async move {
            loop {
                if crossterm::event::poll(tick_rate).unwrap() {
                    let event = crossterm::event::read().unwrap();
                    if let crossterm::event::Event::Mouse(mouse_action) = event {
                        let mouse_action = Mouse::from(mouse_action);
                        if let Err(err) = event_tx.send(InputEvent::MouseAction(mouse_action)).await
                        {
                            error!("Oops!, {}", err);
                        }
                    } else if let crossterm::event::Event::Key(key) = event {
                        let key = Key::from(key);
                        if let Err(err) = event_tx.send(InputEvent::KeyBoardInput(key)).await {
                            error!("Oops!, {}", err);
                        }
                    }
                }
                if let Err(err) = event_tx.send(InputEvent::Tick).await {
                    error!("Oops!, {}", err);
                }
                if event_stop_capture.load(Ordering::Relaxed) {
                    break;
                }
            }
        });

        Events {
            rx,
            _tx: tx,
            stop_capture,
        }
    }

    pub async fn next(&mut self) -> InputEvent {
        let new_event = self.rx.recv().await.unwrap_or(InputEvent::Tick);
        if new_event == InputEvent::KeyBoardInput(Key::Unknown) {
            InputEvent::Tick
        } else {
            new_event
        }
    }

    pub fn close(&mut self) {
        self.stop_capture.store(true, Ordering::Relaxed)
    }
}
