use enigo::{Enigo, KeyboardControllable};
use shortcut_autotyper::error::ATResult;
use std::{error::Error, fs::File};

use std::env::{args, var};

use shortcut_autotyper::Combinations;

/// Return path to default config file.
fn default_path() -> String {
    var("HOME").unwrap_or(String::from("~")) + "/.shortcut_autotyper.json"
}

#[derive(PartialEq)]
enum Options {
    Nothing,
    List,
    Help,
    TooMany,
}

/// Read command line arguments and return tuple
/// where:
/// - `Vec<String` is list of commands to execute
/// - `Option<String>` is path to config file
fn read_args() -> (Vec<String>, Option<String>, Options) {
    let mut option = Options::Nothing;
    let mut set_option = |new| {
        if option == new {
            return;
        }
        option = if option == Options::Nothing {
            new
        } else {
            Options::TooMany
        };
    };
    let mut read_config = false;
    let mut config = None;
    let mut commands = Vec::new();
    args().skip(1).for_each(|arg| match arg.as_str() {
        "-c" | "--config" => read_config = true,
        "-l" | "--list" => set_option(Options::List),
        "-h" | "--help" => set_option(Options::Help),
        _ => {
            if read_config {
                config = Some(arg);
                read_config = false;
            } else {
                commands.push(arg);
            }
        }
    });
    (commands, config, option)
}

fn help() -> &'static str {
    r#"shortcut-autotyper [OPTIONS] [COMMAND]...

Options:
    -c --config [PATH]  Set path to config file with sequences and combinations.
    -l --list           List all avaible commands.
    -h --help           Print this help.
"#
}

fn main() -> Result<(), Box<dyn Error>> {
    let (commands, path, option) = read_args();
    let combinations: Combinations =
        serde_json::from_reader(File::open(path.unwrap_or(default_path()))?)?;
    match option {
        Options::List => {
            combinations.list_all_commands().iter().for_each(|command| {
                println!("{command}");
            });
        }
        Options::Help => {
            println!("{}", help());
        }
        Options::TooMany => {
            println!("Invalid use of commands some of arguments cannot be used together.\n{}", help());
        }
        _ => {}
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
