use tokio::io::{self, AsyncBufReadExt};

use serde::{Deserialize, Serialize};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, GetKeyState};

use crate::keycodes::KeyUpDown;
use crate::macro_events::{KeyboardEvent, MacroEvent};
use crate::r#macro::Macro;
use crate::KeyCode;

#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct MacroRecorder {
    pub events: Vec<MacroEvent>,
}

impl MacroRecorder {
    pub async fn start(&mut self) {
        let (tx, mut rx) = tokio::sync::broadcast::channel(1);
        tokio::spawn(async move {
            let mut input = String::new();
            let stdin = io::stdin();
            let mut reader = io::BufReader::new(stdin);

            reader
                .read_line(&mut input)
                .await
                .expect("Failed to read line");
            tx.send(()).expect("Failed to send signal");
        });
        let mut keymap: std::collections::HashMap<i32, bool> = std::collections::HashMap::new();
        let mut sleep_time = std::time::Instant::now();
        loop {
            if rx.try_recv().is_ok() {
                break;
            }
            for i in 1..256 {
                let ret = unsafe { GetAsyncKeyState(i) };
                if ret & 0x8000u16 as i16 != 0 {
                    if keymap.contains_key(&i) {
                        continue;
                    }
                    println!("Key: {:?}", KeyCode::from(i as u32));
                    println!("Sleep time: {:?}", sleep_time.elapsed().as_millis() as u32);
                    keymap.insert(i, true);
                    self.events
                        .push(MacroEvent::SleepMs(sleep_time.elapsed().as_millis() as u64));
                    self.events.push(MacroEvent::Keybd(KeyboardEvent {
                        key: Some(KeyCode::from(i as u32)),
                        key_up_down: Some(KeyUpDown::Down),
                        custom_flags: None,
                    }));
                    sleep_time = std::time::Instant::now();
                } else if keymap.contains_key(&i) {
                    println!("Key Up: {:?}", KeyCode::from(i as u32));
                    println!("Sleep time: {:?}", sleep_time.elapsed().as_millis() as u32);
                    keymap.remove(&i);
                    self.events
                        .push(MacroEvent::SleepMs(sleep_time.elapsed().as_millis() as u64));
                    self.events.push(MacroEvent::Keybd(KeyboardEvent {
                        key: Some(KeyCode::from(i as u32)),
                        key_up_down: Some(KeyUpDown::Up),
                        custom_flags: None,
                    }));
                    sleep_time = std::time::Instant::now();
                }
            }
        }
        println!("{:?}", keymap);
        self.save();
    }

    fn save(&self) {
        println!("\n\nEnter hotkey to start macro. Escape to stop recording.");
        let mut keymap: std::collections::HashMap<i32, bool> = std::collections::HashMap::new();
        loop {
            for i in 0..256 {
                let ret = unsafe { GetKeyState(i) };
                if ret & 0x8000u16 as i16 != 0 && i == KeyCode::VK_ESCAPE as i32 {
                    keymap.insert(i, true);
                    break;
                } else if ret & 0x8000u16 as i16 != 0
                    && i != 0
                    && i != KeyCode::VK_RETURN as i32
                    && i != KeyCode::VK_LBUTTON as i32
                    && i != KeyCode::VK_RBUTTON as i32
                {
                    if keymap.contains_key(&i) {
                        continue;
                    }
                    keymap.insert(i, true);
                    println!("{:?}", KeyCode::from(i as u32));
                    break;
                }
            }
            if keymap.contains_key(&(KeyCode::VK_ESCAPE as i32)) {
                keymap.remove(&(KeyCode::VK_ESCAPE as i32));
                break;
            }
        }
        let keys_pressed = keymap
            .keys()
            .map(|&x| KeyCode::from(x as u32))
            .collect::<Vec<KeyCode>>();
        let final_macro = Macro {
            name: "test".to_string(),
            blocks: vec![crate::r#macro::MacroBlock {
                hotkey: Some(keys_pressed),
                events: self.events.clone(),
                running: false,
            }],
        };
        let serialized = ron::ser::to_string_pretty(&final_macro, Default::default()).unwrap();
        std::fs::write("test_macro2.ron", serialized).unwrap();
        println!("Macro saved to test_macro2.ron");
    }
}
