use crate::{
    keycodes::{KeyboardFlags, MouseData, MouseFlags, KeyUpDown},
    KeyCode,
};
use serde::{Deserialize, Serialize};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT,
    KEYBD_EVENT_FLAGS, MOUSEINPUT, MOUSE_EVENT_FLAGS, VIRTUAL_KEY,
};

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum MacroEvent {
    SleepMs(u64),
    Keybd(KeyboardEvent),
    MouseMove(MouseMoveEvent),
    MouseBtn(MouseButtonEvent),
    Run(String),
    Loop(LoopEvent),
    ExitApp,
    PreciseSleep(u64),
    LossySleep(u64),
}

impl MacroEvent {
    pub fn run(&self) {
        let (elapsed_time, event_type) = match self {
            MacroEvent::LossySleep(ms) => {
                let start = std::time::Instant::now();
                std::thread::sleep(std::time::Duration::from_millis(*ms));
                (start.elapsed().as_micros(), "LossySleep")
            }
            MacroEvent::SleepMs(ms) |
            MacroEvent::PreciseSleep(ms) => {
                let start = std::time::Instant::now();
                let mut elapsed = 0;
                while elapsed < *ms {
                    elapsed = start.elapsed().as_millis() as u64;
                }
                (start.elapsed().as_micros(), "Sleep")
            }
            MacroEvent::Keybd(keybd_event) => {
                let start = std::time::Instant::now();
                keybd_event.run();
                (start.elapsed().as_micros(), "Keybd")
            }
            MacroEvent::MouseMove(mouse_move_event) => {
                let start = std::time::Instant::now();
                mouse_move_event.run();
                (start.elapsed().as_micros(), "MouseMove")
            }
            MacroEvent::MouseBtn(mouse_btn_event) => {
                let start = std::time::Instant::now();
                mouse_btn_event.run();
                (start.elapsed().as_micros(), "MouseBtn")
            }
            MacroEvent::Run(cmd) => {
                let start = std::time::Instant::now();
                let _ = std::process::Command::new("cmd")
                    .args([cmd])
                    .output()
                    .expect("Failed to execute command.");
                (start.elapsed().as_micros(), "Run")
                // println!("Output: {}", String::from_utf8_lossy(&output.stdout));
            }
            MacroEvent::Loop(event) => {
                for _ in 0..event.count {
                    for event in &event.events {
                        event.run();
                    }
                }
                (0, "Loop")
            }
            MacroEvent::ExitApp => {
                std::process::exit(0);
            }
        };
        println!("{}: {}us", event_type, elapsed_time);
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct LoopEvent {
    pub count: u32,
    pub events: Vec<MacroEvent>,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct KeyboardEvent {
    pub key: Option<KeyCode>,
    // none to tap, down to press and hold, up to release
    pub key_up_down: Option<KeyUpDown>,
    pub custom_flags: Option<KeyboardFlags>,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct MouseMoveEvent {
    pub x: i32,
    pub y: i32,
    pub absolute: bool,
}

// #[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
// #[repr(u32)]
// pub enum MouseButton {
//     Left = KeyCode::VK_LBUTTON as u32,
//     Right = KeyCode::VK_RBUTTON as u32,
//     Middle = KeyCode::VK_MBUTTON as u32,
//     XButton1 = KeyCode::VK_XBUTTON1 as u32,
//     XButton2 = KeyCode::VK_XBUTTON2 as u32,
// }

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct MouseButtonEvent {
    pub flags: MouseFlags,
    pub up_down: Option<KeyUpDown>,
}

impl KeyboardEvent {
    pub fn run(&self) {
        let mut inputs = vec![];
        let mut flags = self.custom_flags.unwrap_or(KeyboardFlags::NONE);

        if let Some(key_up_down) = self.key_up_down {
            if key_up_down == KeyUpDown::Up {
                flags |= KeyboardFlags::KEYEVENTF_KEYUP;
            }
        }

        let virtual_key = self.key.unwrap_or(KeyCode::VK_NONE) as u16;

        // Function to create an INPUT structure for a key event
        fn create_input(dw_flags: KeyboardFlags, virtual_key: u16) -> INPUT {
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(virtual_key),
                        wScan: 0,
                        dwFlags: KEYBD_EVENT_FLAGS(dw_flags as u32),
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            }
        }

        inputs.push(create_input(flags, virtual_key));

        // Use the number of inputs in the vector
        let num_inputs = inputs.len();
        unsafe {
            SendInput(&inputs, (std::mem::size_of::<INPUT>() * num_inputs) as i32);
        }

        if self.key_up_down.is_none() {
            std::thread::sleep(std::time::Duration::from_micros(10));
            let key_up_inputs = vec![create_input(flags | KeyboardFlags::KEYEVENTF_KEYUP, virtual_key)];
            unsafe {
                SendInput(&key_up_inputs, std::mem::size_of::<INPUT>() as i32);
            }
        }
    }
}


impl MouseMoveEvent {
    pub fn run(&self) {
        let mut inputs = vec![INPUT::default()];
        let mut flags = MouseFlags::MOUSEEVENTF_MOVE;

        if self.absolute {
            flags |= MouseFlags::MOUSEEVENTF_ABSOLUTE;
        }

        let input = &mut inputs[0];
        input.r#type = INPUT_MOUSE;
        input.Anonymous = INPUT_0 {
            mi: MOUSEINPUT {
                dx: self.x,
                dy: self.y,
                mouseData: MouseData::NONE as i32,
                dwFlags: MOUSE_EVENT_FLAGS(flags as u32),
                time: 0,
                dwExtraInfo: 0,
            },
        };
        let len = inputs.len();
        unsafe {
            SendInput(&inputs, (std::mem::size_of::<INPUT>() * len) as i32);
        }
    }
}

impl MouseButtonEvent {
    pub fn run(&self) {
        let mut inputs = vec![INPUT::default()];
        let right = self.flags & MouseFlags::MOUSEEVENTF_RIGHTDOWN == MouseFlags::MOUSEEVENTF_RIGHTDOWN || self.flags & MouseFlags::MOUSEEVENTF_RIGHTUP == MouseFlags::MOUSEEVENTF_RIGHTUP;

        
        let input = &mut inputs[0];
        input.r#type = INPUT_MOUSE;
        if self.up_down.is_some() {
            input.Anonymous = INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: MouseData::NONE as i32,
                    dwFlags: MOUSE_EVENT_FLAGS(self.flags as u32),
                    time: 0,
                    dwExtraInfo: 0,
                },
            };
        } else {
            let flags = self.flags | if right { MouseFlags::MOUSEEVENTF_RIGHTUP } else { MouseFlags::MOUSEEVENTF_LEFTUP };
            input.Anonymous = INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: MouseData::NONE as i32,
                    dwFlags: MOUSE_EVENT_FLAGS(flags as u32),
                    time: 0,
                    dwExtraInfo: 0,
                },
            };
        }

        let len = inputs.len();
        unsafe {
            SendInput(&inputs, (std::mem::size_of::<INPUT>() * len) as i32);
        }
    }
}
