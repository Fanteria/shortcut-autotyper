use enigo::{Enigo, KeyboardControllable};
use shortcut_autotyper::error::{ATResult, ErrType};
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
    ListFull,
    Help,
    TooMany,
}

/// Read command line arguments and return tuple
/// where:
/// - `Vec<String` is list of commands to execute
/// - `Option<String>` is path to config file
fn read_args() -> Result<(Vec<String>, Option<String>, Options, u64), Box<dyn Error>> {
    let mut option = Options::Nothing;
    let mut iter = args().skip(1);
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
    let mut config = None;
    let mut delay = 50_000;
    let mut arguments = Vec::new();
    while let Some(arg) = iter.next() {
        let mut get_value = || match iter.next() {
            Some(value) => Ok(value),
            None => ErrType::ArgumentMissing(arg.clone()).into(),
        };
        match arg.as_str() {
            "-c" | "--config" => config = Some(get_value()?),
            "-d" | "--delay" => delay = get_value()?.parse()?,
            "-l" | "--list" => set_option(Options::List),
            "-L" | "--list-full" => set_option(Options::ListFull),
            "-h" | "--help" => set_option(Options::Help),
            _ => arguments.push(arg),
        };
    }
    Ok((arguments, config, option, delay))
}

fn help() -> &'static str {
    r#"shortcut-autotyper [OPTIONS] [COMMAND]...

Options:
    -c --config [PATH]  Set path to config file with sequences and combinations.
    -l --list           List all avaible commands.
    -L --list-full      List all avaible commands with output.
    -h --help           Print this help.
    -d --delay          Set delay between two key strokes, default is 50 000.
"#
}

fn run() -> Result<(), Box<dyn Error>> {
    let (commands, path, option, delay) = read_args()?;
    let combinations: Combinations =
        serde_json::from_reader(File::open(path.unwrap_or(default_path()))?)?;
    match option {
        Options::List => {
            combinations.list_all_commands().iter().for_each(|command| {
                println!("{command}");
            });
        }
        Options::ListFull => {
            combinations.list_all_commands().iter().for_each(|command| {
                println!("{command}: {}", combinations.get_sequence(command, &Vec::new()).unwrap().replace("\n", "\\n"));
            });
        }
        Options::Help => {
            println!("{}", help());
        }
        Options::TooMany => {
            println!(
                "Invalid use of commands some of arguments cannot be used together.\n{}",
                help()
            );
        }
        _ => {}
    }
    let mut enigo = Enigo::new();
    enigo.set_delay(delay);
    enigo.key_sequence(&combinations.get_sequence(&commands[0], &commands)?);
    // commands
    //     .iter()
    //     .map(|command| combinations.get_sequence(command, &Vec::new()))
    //     .collect::<ATResult<Vec<_>>>()?
    //     .iter()
    //     .for_each(|sequence| {
    //         enigo.key_sequence(sequence);
    //     });
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("{}", e)
    }
}
