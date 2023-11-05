// todo: everything
// (that means atleast partial .ahk file support)

use std::{io::BufRead, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{
    keycodes::{KeyCode, KeyUpDown, MouseFlags},
    macro_events::{KeyboardEvent, LoopEvent, MacroEvent, MouseButtonEvent, MouseMoveEvent},
    r#macro::{Macro, MacroBlock},
};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
pub enum AhkFunctions {
    None,
    Hotkey,
    // SendMode,
    // always assume SendMode Input bcus... fast.
    Send,
    // SendRaw,
    // SendInput,
    // SendPlay,
    // SendEvent,
    Sleep,
    Click,
    MouseMove,
    DllCall,
    // Reload,
    ExitApp,
    Loop,
    Run,
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct AhkFile {
    pub path: String,
    pub blocks: Vec<AhkBlock>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct AhkBlock {
    pub condition: Option<AhkFunction>,
    pub functions: Vec<AhkFunction>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct AhkFunction {
    pub func: Option<AhkFunctions>,
    pub args: Vec<String>,
    pub special_args: Vec<String>,
}

impl AhkFile {
    pub fn parse(&mut self) -> Option<Macro> {
        let mut m: Macro = Macro {
            name: self.path.clone(),
            blocks: vec![],
        };
        let file: std::fs::File = std::fs::File::open(&self.path).unwrap();
        let mut ahk_block = AhkBlock::default();
        for line in std::io::BufReader::new(file).lines() {
            let line = line.unwrap();
            let mut words = line.split_whitespace();
            let mut func = words.next().unwrap_or("").to_lowercase();
            func = func.trim_matches(',').to_string();

            if func.is_empty() || func.starts_with(';') {
                continue;
            }
            if func.contains("dllcall") {
                let mut dllcall = AhkFunction {
                    func: Some(AhkFunctions::DllCall),
                    args: vec![],
                    special_args: vec![],
                };
                let mut dllcall_args = func.split('(').collect::<Vec<&str>>();
                dllcall_args.remove(0);
                let mut dllcall_args = dllcall_args.join("(");
                dllcall_args.pop();
                dllcall_args = dllcall_args.trim_matches(&['(', ')'][..]).to_string();
                let dllcall_args = dllcall_args.split(',').collect::<Vec<&str>>();
                match dllcall_args[0] {
                    "\"mouse_event\"" => {
                        let flags: u16 = dllcall_args[2].parse::<u16>().unwrap();

                        if flags & MouseFlags::MOUSEEVENTF_MOVE as u16 != 0 {
                            dllcall.func = Some(AhkFunctions::MouseMove);
                        }

                        let x = dllcall_args[4].parse::<i32>().unwrap();
                        let y = dllcall_args[6].parse::<i32>().unwrap();

                        let absolute = flags & MouseFlags::MOUSEEVENTF_ABSOLUTE as u16 != 0;

                        dllcall.args.push(x.to_string());
                        dllcall.args.push(y.to_string());
                        dllcall.args.push(absolute.to_string());

                        ahk_block.functions.push(dllcall);
                    }
                    _ => {
                        eprintln!("DllCall function not implemented: {:?}", dllcall_args[0]);
                    }
                }
                continue;
            }
            let args = words.collect::<Vec<&str>>();
            let mut ahk_func = AhkFunction {
                func: None,
                args: vec![],
                special_args: vec![],
            };
            if func.contains("::") {
                ahk_func.func = Some(AhkFunctions::Hotkey);
                let mut hotkeys: Vec<KeyCode> = vec![];
                for char in func.trim_matches(':').chars() {
                    match char {
                        '^' => hotkeys.push(KeyCode::VK_CONTROL),
                        '!' => hotkeys.push(KeyCode::VK_MENU),
                        '+' => hotkeys.push(KeyCode::VK_SHIFT),
                        '#' => hotkeys.push(KeyCode::VK_LWIN),
                        _ => {}
                    }
                }

                let mut hotkey_no_modifiers = func
                    .trim_matches(&[':', '^', '!', '+', '#'][..])
                    .to_uppercase();
                if hotkey_no_modifiers.contains(':') {
                    let c = hotkey_no_modifiers.clone();
                    let split = c.split(':').collect::<Vec<&str>>();

                    hotkey_no_modifiers = split[0].to_string();
                    if split.len() > 1 {
                        for arg in &split[1..] {
                            if arg.is_empty() {
                                continue;
                            }
                            ahk_func.special_args.push(arg.to_string());
                        }
                    }
                }
                hotkeys.push(KeyCode::from_str(hotkey_no_modifiers.as_str()).unwrap());

                ahk_func.args.push(
                    hotkeys
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join(" "),
                );
                println!("hotkey func {:?}", ahk_func);
                ahk_block.condition = Some(ahk_func);

                continue;
            }
            let mut skip_add_func = false;
            match func.as_str() {
                "sleep" => {
                    ahk_func.func = Some(AhkFunctions::Sleep);
                    ahk_func.args.push(args[0].to_string());
                }
                "send" => {
                    ahk_func.func = Some(AhkFunctions::Send);
                    ahk_func
                        .args
                        .push(args.join(" ").trim_matches(&['{', '}'][..]).to_string());
                }
                "loop" => {
                    self.blocks.push(ahk_block);
                    ahk_block = AhkBlock::default();

                    ahk_func.func = Some(AhkFunctions::Loop);
                    ahk_func.args.push(args[0].to_string());
                    ahk_block.condition = Some(ahk_func.clone());

                    skip_add_func = true;
                }
                "{" => {
                    // TODO: figure out how to mark the start of a nested block
                    // if ahk_block.functions.last().is_some() {
                    //     self.blocks.push(ahk_block);
                    //     ahk_block = AhkBlock {
                    //         condition: None,
                    //         functions: vec![],
                    //     };
                    // }
                }
                "}" => {
                    self.blocks.push(ahk_block);
                    ahk_block = AhkBlock::default();
                }
                "click" => {
                    ahk_func.func = Some(AhkFunctions::Click);
                }
                "run" => {
                    ahk_func.func = Some(AhkFunctions::Run);
                    ahk_func.args.push(args.join(" ").to_string());
                }
                _ => {
                    eprintln!("Function not implemented: {:?} args {:?}", func, args);
                }
            }
            if ahk_func.func.is_none() && ahk_func.args.is_empty() {
                continue;
            }
            if !skip_add_func {
                ahk_block.functions.push(ahk_func);
            }
        }
        self.blocks.push(ahk_block);
        for block in &self.blocks {
            let mut macro_block = MacroBlock {
                hotkey: None,
                events: vec![],
                running: false,
            };
            if block.condition.clone().is_some() && block.condition.clone().unwrap().func.is_some()
            {
                let cond = block.condition.clone().unwrap();
                let func = cond.func.unwrap();
                println!("cond {:?} func {:?}", cond, func);
                match func {
                    AhkFunctions::Loop => {
                        let count = cond.args[0].parse::<u32>().unwrap();
                        let mut events = vec![];
                        for event in &block.functions {
                            let me = event.parse();
                            if me.is_none() {
                                continue;
                            }
                            events.push(me.unwrap());
                        }
                        macro_block
                            .events
                            .push(MacroEvent::Loop(LoopEvent { count, events }));
                    }
                    AhkFunctions::Hotkey => {
                        let hotkey = cond.args[0].split(' ').collect::<Vec<&str>>();
                        let mut hotkeys: Vec<KeyCode> = vec![];
                        for key in hotkey {
                            hotkeys.push(KeyCode::from_str(key).unwrap());
                        }
                        println!("hotkey button {:?}", hotkeys);
                        macro_block.hotkey = Some(hotkeys);
                        if !cond.special_args.is_empty() {
                            for arg in &cond.special_args {
                                if arg.to_lowercase().as_str() == "exitapp" {
                                    macro_block.events.push(MacroEvent::ExitApp);
                                }
                            }
                        }
                    }
                    _ => {
                        eprintln!("Condition not implemented: {:?}", func);
                    }
                }
            }
            for func in &block.functions {
                println!("block functions {:?}", func);
                let me = func.parse();
                if me.is_none() {
                    continue;
                }
                macro_block.events.push(me.unwrap());
            }
            m.blocks.push(macro_block);
        }
        println!("{:#?}", m);
        Some(m)
    }
}

impl AhkFunction {
    pub fn parse(&self) -> Option<MacroEvent> {
        self.func?;
        match self.func.unwrap() {
            AhkFunctions::Sleep => Some(MacroEvent::SleepMs(self.args[0].parse::<u64>().unwrap())),
            AhkFunctions::Send => {
                let input = self.args.join(" ");
                let input_vec: Vec<&str> = input.split(' ').collect();

                let vk = match input_vec[0].to_lowercase().as_str() {
                    "shift" => KeyCode::VK_SHIFT,
                    "ctrl" => KeyCode::VK_CONTROL,
                    "alt" => KeyCode::VK_MENU,
                    "space" => KeyCode::VK_SPACE,
                    _ => KeyCode::from_str(input_vec[0]).unwrap(),
                };

                let upordown = input_vec.get(1).map(|&s| {
                    match s.to_lowercase().as_str() {
                        "up" => KeyUpDown::Up,
                        "down" => KeyUpDown::Down,
                        _ => KeyUpDown::Down, // Default to "down" if an unknown value is provided
                    }
                });

                Some(MacroEvent::Keybd(KeyboardEvent {
                    key: Some(vk),
                    key_up_down: upordown,
                    custom_flags: None,
                }))
            }
            AhkFunctions::MouseMove => {
                println!("{:?}", self.args);
                let x = self.args[0].parse::<i32>().unwrap();
                let y = self.args[1].parse::<i32>().unwrap();
                let absolute = self.args[2].parse::<bool>().unwrap();
                Some(MacroEvent::MouseMove(MouseMoveEvent { x, y, absolute }))
            }
            AhkFunctions::Click => {
                if !self.args.is_empty() {
                    eprintln!("Function arguments not implemented: {:?}", self.func);
                    return None;
                }
                Some(MacroEvent::MouseBtn(MouseButtonEvent {
                    flags: MouseFlags::MOUSEEVENTF_LEFTDOWN,
                    up_down: None,
                }))
            }
            AhkFunctions::Run => {
                println!("{:?}", self.args);
                let args = self.args.join(" ");
                let mut args = args
                    .split(&[',', ' '][..])
                    .filter(|x| !x.is_empty())
                    .collect::<Vec<&str>>();

                let should_hide_cmd =
                    args.last().map(|s| s.to_lowercase().contains("hide")) == Some(true);
                if should_hide_cmd {
                    args.pop();
                }

                let command = format!("{} {}", args.remove(0).trim_matches('\\'), args.join(" "));
                Some(MacroEvent::Run(command))
            }
            AhkFunctions::ExitApp => {
                Some(MacroEvent::ExitApp)
            }
            _ => {
                eprintln!("Function not implemented: {:?}", self.func);
                None
            }
        }
    }
}
