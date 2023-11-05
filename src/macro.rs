use std::collections::HashSet;
// use std::sync::Arc;
// use tokio::sync::Mutex;
// use tokio::sync::Semaphore;
// use tokio::sync::mpsc;

use crate::{macro_events::MacroEvent, KeyCode};
use serde::{Deserialize, Serialize};
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

// #[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
// pub struct MacroSer {
//     pub name: String,
//     pub blocks: Vec<MacroBlock>,
// }

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct Macro {
    pub name: String,
    pub blocks: Vec<MacroBlock>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct MacroBlock {
    pub hotkey: Option<Vec<KeyCode>>,
    pub events: Vec<MacroEvent>,
    pub running: bool,
}

impl Macro {
    // pub fn run(&self) {
    //     // TODO: multithread so that multiple events can run at the same time
    //     // as well as other block's hotkeys being able to be pressed
    //     for block in &self.blocks {
    //         block.run();
    //     }
    // }

    pub fn run(&self) {
        // let (tx, mut rx) = mpsc::channel(32);
        // let semaphore = Arc::new(Semaphore::new(1));

        for block in &self.blocks {
            // let tx = tx.clone();
            // let block = block.lock().await.clone();
            // let semaphore = semaphore.clone();

            // tokio::spawn(async move {
            //     block.run().await;
            //     tx.send(()).await.expect("Channel send failed");
            // });
            if block.hotkey.is_some() {
                block.wait_for_keypress();
            }
            for event in &block.events {
                event.run();
            }
        }

        // for _ in &self.blocks {
        //     rx.recv().await.expect("Channel receive failed");
        // }
    }
}

impl MacroBlock {
    fn wait_for_keypress(&self) {
        let mut keys_pressed: HashSet<KeyCode> = HashSet::new();
        loop {
            for key in self.hotkey.as_ref().unwrap() {
                let ret = unsafe { GetAsyncKeyState(*key as i32) };
                if ret & 0x8000u16 as i16 != 0 {
                    if keys_pressed.contains(key) {
                        continue;
                    }
                    println!("Key: {:?}", key);
                    keys_pressed.insert(*key);
                } else if keys_pressed.contains(key) {
                    println!("Key Up: {:?}", key);
                    keys_pressed.remove(key);
                }
            }
            if keys_pressed.len() == self.hotkey.as_ref().unwrap().len() {
                break;
            }
        }
    }
}
