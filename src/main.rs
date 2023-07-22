use enigo::{Enigo, KeyboardControllable};
use shortcut_autotyper::error::ATResult;
use std::{error::Error, fs::File};

use std::env::{args, var};

use shortcut_autotyper::combinations::Combinations;

fn default_path() -> String {
    var("HOME").unwrap_or(String::from("~")) + "/.shortcut_autotyper.json"
}

fn read_args() -> (Vec<String>, Option<String>) {
    let mut read_config = false;
    let mut config = None;
    let mut commands = Vec::new();
    args().skip(1).for_each(|arg| match arg.as_str() {
        "-c" => read_config = true,
        _ => {
            if read_config {
                config = Some(arg);
            } else {
                commands.push(arg);
            }
        }
    });
    (commands, config)
}

fn main() -> Result<(), Box<dyn Error>> {
    let (commands, path) = read_args();
    let mut enigo = Enigo::new();
    enigo.set_delay(50_000);
    let combinations: Combinations =
        serde_json::from_reader(File::open(path.unwrap_or(default_path()))?)?;
    commands
        .iter()
        .map(|command| combinations.get_sequence(command))
        .collect::<ATResult<Vec<_>>>()?
        .iter()
        .for_each(|sequence| {
            enigo.key_sequence(sequence);
        });
    Ok(())
}
