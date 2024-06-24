use std::{process::Command, sync::mpsc, thread};

use color_eyre::eyre;
use rdev::{grab, Event, EventType, Key};
use thiserror::Error;

// dont ask why we need this, mac is weird i guess
const CTRL_RIGHT: u32 = 62;

fn main() -> eyre::Result<()> {
    let handle = InputHandler::new();
    grab(handle).map_err(ListenError)?;
    Ok(())
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

struct InputHandler {
    shift_left: KeyMonitor,
    shift_right: KeyMonitor,
    ctrl_left: KeyMonitor,
    ctrl_right: KeyMonitor,
    ctrl_unknown: KeyMonitor,
    alt: KeyMonitor,
    alt_gr: KeyMonitor,
    cmd_left: KeyMonitor,
    cmd_right: KeyMonitor,
}

impl InputHandler {
    fn new() -> impl Fn(Event) -> Option<Event> {
        let (tx_input, rx_input) = mpsc::channel();
        let (tx_output, rx_ouput) = mpsc::channel();

        thread::spawn(move || {
            let mut handler = Self {
                shift_left: KeyMonitor::new(Key::ShiftLeft),
                shift_right: KeyMonitor::new(Key::ShiftRight),
                ctrl_left: KeyMonitor::new(Key::ControlLeft),
                ctrl_right: KeyMonitor::new(Key::ControlRight),
                ctrl_unknown: KeyMonitor::new(Key::Unknown(CTRL_RIGHT)),
                alt: KeyMonitor::new(Key::Alt),
                alt_gr: KeyMonitor::new(Key::AltGr),
                cmd_left: KeyMonitor::new(Key::MetaLeft),
                cmd_right: KeyMonitor::new(Key::MetaRight),
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

    fn shift_pressed(&self) -> bool {
        self.shift_left.is_pressed() || self.shift_right.is_pressed()
    }

    fn ctrl_pressed(&self) -> bool {
        self.ctrl_left.is_pressed()
            || self.ctrl_right.is_pressed()
            || self.ctrl_unknown.is_pressed()
    }

    fn alt_pressed(&self) -> bool {
        self.alt.is_pressed() || self.alt_gr.is_pressed()
    }

    fn cmd_pressed(&self) -> bool {
        self.cmd_left.is_pressed() || self.cmd_right.is_pressed()
    }

    fn process_event(&mut self, event: Event) -> Option<Event> {
        self.shift_left.process_event(&event);
        self.shift_right.process_event(&event);
        self.ctrl_left.process_event(&event);
        self.ctrl_right.process_event(&event);
        self.ctrl_unknown.process_event(&event);
        self.alt.process_event(&event);
        self.alt_gr.process_event(&event);
        self.cmd_left.process_event(&event);
        self.cmd_right.process_event(&event);

        match &event.event_type {
            // Shortcut-key
            EventType::KeyPress(key)
                if self.shift_pressed()
                    && self.ctrl_pressed()
                    && !self.alt_pressed()
                    && !self.cmd_pressed() =>
            {
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
            Key::Num5 => {
                open_app("/Applications/Discord.app");
                true
            }
            Key::Num6 => {
                open_app("/Applications/Firefox.app");
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

struct KeyMonitor {
    key: Key,
    pressed: bool,
}

impl KeyMonitor {
    pub fn new(key: Key) -> Self {
        KeyMonitor {
            key,
            pressed: false,
        }
    }

    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    pub fn process_event(&mut self, event: &Event) {
        match event.event_type {
            EventType::KeyPress(key) if key == self.key => self.pressed = true,
            EventType::KeyRelease(key) if key == self.key => self.pressed = false,
            _ => (),
        }
    }
}

#[derive(Debug, Error)]
#[error("Event listening error: {0:?}")]
struct ListenError(rdev::GrabError);
