pub mod ahk;
pub mod keycodes;
pub mod r#macro;
pub mod macro_events;
pub mod recorder;
// use windows::Win32::Foundation::ERROR_BLOCK_SHARED;

use crate::ahk::AhkFile;
use crate::r#macro::Macro;
use crate::recorder::MacroRecorder;

use crate::keycodes::KeyCode;
use std::io::{Read, Write};

fn usage() -> String {
    format!(
        "Usage: {} [OPTIONS] [MACRO_FILE]
Options:
    -r, --record    Record a new macro
    -h, --help      Print this help message and exit
    -v, --version   Print version information and exit",
        std::env::args().next().unwrap()
    )
}

#[derive(Debug)]
enum Argument {
    Record,
    Help,
    Version,
    MacroFile(String),
}

#[tokio::main]
async fn main() -> anyhow::Result<(), anyhow::Error> {
    // let mut ahk = AhkFile {
    //     path: "test.ahk".to_string(),
    //     blocks: vec![],
    //     converted_macro: None,
    // };

    // let m = ahk.parse().unwrap();

    // // println!("{:#?}", m);

    // let ronstr = ron::ser::to_string_pretty(&m, Default::default()).unwrap();
    // let file = std::fs::File::create(format!("{}.ron", m.name)).unwrap();
    // let mut writer = std::io::BufWriter::new(file);
    // writer.write_all(ronstr.as_bytes()).unwrap();
    
    // return Ok(());


    let mut arguments = vec![];
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-r" | "--record" => arguments.push(Argument::Record),
            "-h" | "--help" => arguments.push(Argument::Help),
            "-v" | "--version" => arguments.push(Argument::Version),
            _ => arguments.push(Argument::MacroFile(arg)),
        }
    }

    let mut record = false;
    let mut macro_file = None;

    for argument in arguments {
        match argument {
            Argument::Record => record = true,
            Argument::Help => {
                println!("{}", usage());
                return Ok(());
            }
            Argument::Version => {
                println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            Argument::MacroFile(file) => macro_file = Some(file),
        }
    }

    if record {
        let mut macro_recorder = MacroRecorder::default();
        println!("Recording macro. Press enter to stop recording.");
        macro_recorder.start().await;
        return Ok(());
    }

    if macro_file.is_none() {
        println!("{}", usage());
        return Ok(());
    }

    let ma: Option<Macro>;

    if macro_file.clone().unwrap().ends_with(".ahk") {
        let mut ahk = AhkFile {
            path: macro_file.unwrap(),
            blocks: vec![],
        };
        ma = ahk.parse();
        // cant use .map() because of the async block
        // let blocks_ser = ahk.blocks.iter().map(|x| x.lock().await.clone()).collect::<Vec<MacroSer>>();
        let mut blocks_ser = vec![];
        for block in ma.as_ref().unwrap().blocks.iter() {
            blocks_ser.push(block.clone());
        }
        // let macro_ser = MacroSer {
        //     name: ma.as_ref().unwrap().name.clone(),
        //     blocks: blocks_ser,
        // };

        let ronstr = ron::ser::to_string_pretty(&ma, Default::default()).unwrap();
        let file = std::fs::File::create(format!("{}.ron", ma.as_ref().unwrap().name)).unwrap();
        let mut writer = std::io::BufWriter::new(file);
        writer.write_all(ronstr.as_bytes()).unwrap();
    } else {
        let mut file = std::fs::File::open(macro_file.unwrap()).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        ma = Some(ron::de::from_str(&contents).unwrap());
    }

    if ma.is_none() {
        println!("Failed to parse macro file.");
        return Ok(());
    }

    let ma = ma.unwrap();
    println!("Running macro: {}", ma.name);
    let start = std::time::Instant::now();
    ma.run();
    println!("Total time elapsed: {:?}ms", start.elapsed().as_millis());

    Ok(())
}
