use enigo::{Enigo, KeyboardControllable};
use shortcut_autotyper::error::ATResult;
use std::{error::Error, fs::File};

use std::env::{args, var};

use shortcut_autotyper::Combinations;

/// Return path to default config file.
fn default_path() -> String {
    var("HOME").unwrap_or(String::from("~")) + "/.shortcut_autotyper.json"
}

/// Read command line arguments and return tuple
/// where:
/// - `Vec<String` is list of commands to execute
/// - `Option<String>` is path to config file
fn read_args() -> (Vec<String>, Option<String>, bool) {
    let mut read_config = false;
    let mut print_list = false;
    let mut config = None;
    let mut commands = Vec::new();
    args().skip(1).for_each(|arg| match arg.as_str() {
        "-c" | "--config" => read_config = true,
        "-l" | "--list" => print_list = true,
        _ => {
            if read_config {
                config = Some(arg);
                read_config = false;
            } else {
                commands.push(arg);
            }
        }
    });
    (commands, config, print_list)
}

fn main() -> Result<(), Box<dyn Error>> {
    let (commands, path, print_list) = read_args();
    let combinations: Combinations =
        serde_json::from_reader(File::open(path.unwrap_or(default_path()))?)?;
    if print_list {
        combinations.list_all_commands().iter().for_each(|command| {
            println!("{command}");
        });
    }
    let mut enigo = Enigo::new();
    enigo.set_delay(50_000);
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
