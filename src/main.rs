use std::{env, fs, path::Path, process::Command, sync::mpsc, thread};

use color_eyre::eyre::{self, Context};
use rdev::{grab, Event, EventType, Key};
use serde::Deserialize;
use thiserror::Error;

// dont ask why we need this, mac is weird i guess
const CTRL_RIGHT: u32 = 62;

fn main() -> eyre::Result<()> {
    let config = Config::load()?;
    println!("Mapping the following shortcuts:");
    config.print();
    let handle = InputHandler::new(config);
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

#[derive(Deserialize)]
struct Config {
    num1: Option<String>,
    num2: Option<String>,
    num3: Option<String>,
    num4: Option<String>,
    num5: Option<String>,
    num6: Option<String>,
    num7: Option<String>,
    num8: Option<String>,
    num9: Option<String>,
}

impl Config {
    pub fn load() -> eyre::Result<Config> {
        let path = Path::new(&env::var("HOME").context("HOME env var not available")?)
            .join(".config/globby_shorty.toml");
        let config = fs::read_to_string(&path)
            .with_context(|| format!("failed to load config from {}", path.display()))?;
        Ok(toml::from_str(&config)?)
    }

    pub fn print(&self) {
        if let Some(path) = &self.num1 {
            println!("  Num 1 => {path}")
        }
        if let Some(path) = &self.num2 {
            println!("  Num 2 => {path}")
        }
        if let Some(path) = &self.num3 {
            println!("  Num 3 => {path}")
        }
        if let Some(path) = &self.num4 {
            println!("  Num 4 => {path}")
        }
        if let Some(path) = &self.num5 {
            println!("  Num 5 => {path}")
        }
        if let Some(path) = &self.num6 {
            println!("  Num 6 => {path}")
        }
        if let Some(path) = &self.num7 {
            println!("  Num 7 => {path}")
        }
        if let Some(path) = &self.num8 {
            println!("  Num 8 => {path}")
        }
        if let Some(path) = &self.num9 {
            println!("  Num 9 => {path}")
        }
    }
}

struct InputHandler {
    config: Config,
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
    fn new(config: Config) -> impl Fn(Event) -> Option<Event> {
        let (tx_input, rx_input) = mpsc::channel();
        let (tx_output, rx_ouput) = mpsc::channel();

        thread::spawn(move || {
            let mut handler = Self {
                config,
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
                if self.exec_shortcut(*key) {
                    return None;
                }
            }

            _ => (),
        };

        Some(event)
    }

    fn exec_shortcut(&self, key: Key) -> bool {
        match key {
            Key::Num1 => {
                let Some(path) = &self.config.num1 else {
                    return false;
                };

                open_app(&path);
                true
            }
            Key::Num2 => {
                let Some(path) = &self.config.num2 else {
                    return false;
                };

                open_app(&path);
                true
            }
            Key::Num3 => {
                let Some(path) = &self.config.num3 else {
                    return false;
                };

                open_app(&path);
                true
            }
            Key::Num4 => {
                let Some(path) = &self.config.num4 else {
                    return false;
                };

                open_app(&path);
                true
            }
            Key::Num5 => {
                let Some(path) = &self.config.num5 else {
                    return false;
                };

                open_app(&path);
                true
            }
            Key::Num6 => {
                let Some(path) = &self.config.num6 else {
                    return false;
                };

                open_app(&path);
                true
            }
            Key::Num7 => {
                let Some(path) = &self.config.num7 else {
                    return false;
                };

                open_app(&path);
                true
            }
            Key::Num8 => {
                let Some(path) = &self.config.num8 else {
                    return false;
                };

                open_app(&path);
                true
            }
            Key::Num9 => {
                let Some(path) = &self.config.num9 else {
                    return false;
                };

                open_app(&path);
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
