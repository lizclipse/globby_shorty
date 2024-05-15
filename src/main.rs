use std::{process::Command, sync::mpsc, thread};

use color_eyre::eyre;
use rdev::{grab, Event, EventType, Key};
use thiserror::Error;

const CTRL_RIGHT: u32 = 62;

struct InputHandler {
    shift_pressed: bool,
    ctrl_pressed: bool,
}

impl InputHandler {
    fn new() -> impl Fn(Event) -> Option<Event> {
        let (tx_input, rx_input) = mpsc::channel();
        let (tx_output, rx_ouput) = mpsc::channel();

        thread::spawn(move || {
            let mut handler = Self {
                shift_pressed: false,
                ctrl_pressed: false,
            };

            while let Ok(ev) = rx_input.recv() {
                let ev = handler.process_event(ev);
                if let Err(err) = tx_output.send(ev) {
                    eprintln!("Failed to send event process result: {:#?}", err);
                }
            }

            println!("Event processing stopped");
        });

        return move |ev| {
            if let Err(err) = tx_input.send(ev.clone()) {
                eprintln!("Failed to send event for processing: {:#?}", err);
            }

            match rx_ouput.recv() {
                Ok(ev) => ev,
                Err(err) => {
                    eprintln!("Failed to recv event process result: {:#?}", err);
                    Some(ev)
                }
            }
        };
    }

    fn process_event(&mut self, event: Event) -> Option<Event> {
        match &event.event_type {
            EventType::KeyPress(Key::ShiftLeft) | EventType::KeyPress(Key::ShiftRight) => {
                self.shift_pressed = true
            }

            EventType::KeyPress(Key::ControlLeft)
            | EventType::KeyPress(Key::ControlRight)
            | EventType::KeyPress(Key::Unknown(CTRL_RIGHT)) => self.ctrl_pressed = true,

            EventType::KeyRelease(Key::ShiftLeft) | EventType::KeyRelease(Key::ShiftRight) => {
                self.shift_pressed = false
            }

            EventType::KeyRelease(Key::ControlLeft)
            | EventType::KeyRelease(Key::ControlRight)
            | EventType::KeyRelease(Key::Unknown(CTRL_RIGHT)) => self.ctrl_pressed = false,

            EventType::KeyPress(key) if self.shift_pressed && self.ctrl_pressed => {
                if Self::exec_shortcut(*key) {
                    return None;
                }
            }
            _ => (),
        };

        Some(event)
    }

    fn exec_shortcut(key: Key) -> bool {
        match key {
            Key::Num1 => {
                open_app("/Applications/WezTerm.app");
                true
            }
            Key::Num2 => {
                open_app("/Applications/Safari.app");
                true
            }
            Key::Num3 => {
                open_app("/Applications/Obsidian.app");
                true
            }
            Key::Num4 => {
                open_app("/System/Applications/Music.app");
                true
            }
            Key::Num7 => {
                open_app("/Applications/Microsoft Teams (work or school).app");
                true
            }
            Key::Num8 => {
                open_app("/Applications/Microsoft Outlook.app");
                true
            }
            Key::Num9 => {
                open_app("/Applications/Slack.app");
                true
            }
            _ => false,
        }
    }
}

fn open_app(app: &str) {
    match Command::new("open").arg(app).status() {
        Ok(status) => {
            if !status.success() {
                eprintln!("Failed to open {app}");
            }
        }
        Err(err) => eprintln!("Failed to run `open`: {:#?}", err),
    }
}

fn main() -> eyre::Result<()> {
    let handle = InputHandler::new();
    grab(handle).map_err(ListenError)?;
    Ok(())
}

#[derive(Debug, Error)]
#[error("Event listening error: {0:?}")]
struct ListenError(rdev::GrabError);
