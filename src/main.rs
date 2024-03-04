use shortcut_autotyper::error::ErrType;
use std::process::Command as sysCommand;
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
fn read_args(
    option: &mut Options,
    config: &mut Option<String>,
    delay: &mut u64,
    typer: &mut String,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut iter = args().skip(1);
    let mut set_option = |new| {
        if *option == new {
            return;
        }
        *option = if *option == Options::Nothing {
            new
        } else {
            Options::TooMany
        };
    };
    let mut arguments = Vec::new();
    while let Some(arg) = iter.next() {
        let mut get_value = || match iter.next() {
            Some(value) => Ok(value),
            None => ErrType::ArgumentMissing(arg.clone()).into(),
        };
        match arg.as_str() {
            "-c" | "--config" => *config = Some(get_value()?),
            "-d" | "--delay" => *delay = get_value()?.parse()?,
            "-t" | "--typer" => *typer = get_value()?.parse()?,
            "-l" | "--list" => set_option(Options::List),
            "-L" | "--list-full" => set_option(Options::ListFull),
            "-h" | "--help" => set_option(Options::Help),
            _ => arguments.push(arg),
        };
    }
    if *option == Options::Nothing && arguments.is_empty() {
        return Err("Error: Command must be set.".into());
    }
    Ok(arguments)
}

fn help() -> &'static str {
    r#"shortcut-autotyper [OPTIONS] [COMMAND]...

Options:
    -c --config [PATH]  Set path to config file with sequences and combinations.
    -l --list           List all avaible commands.
    -L --list-full      List all avaible commands with output.
    -h --help           Print this help.
    -d --delay          Set delay between two key strokes, default is 50.
    -t --typer          Binary to send text to terminal.
"#
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut option = Options::Nothing;
    let mut config = None;
    let mut delay = 50;
    let mut typer = String::from("wtype");

    let commands = read_args(&mut option, &mut config, &mut delay, &mut typer)?;
    let get_combinations = || -> Result<Combinations, Box<dyn Error>> {
        Ok(serde_json::from_reader(File::open(
            config.unwrap_or(default_path()),
        )?)?)
    };

    match option {
        Options::List => {
            get_combinations()?
                .list_all_commands()
                .iter()
                .for_each(|command| {
                    println!("{command}");
                });
        }
        Options::ListFull => {
            let combinations = get_combinations()?;
            combinations.list_all_commands().iter().for_each(|command| {
                println!(
                    "{command}: {}",
                    combinations
                        .get_sequence(command, &Vec::new())
                        .unwrap()
                        .replace("\n", "\\n")
                );
            });
        }
        Options::Help => {
            println!("{}", help());
            return Ok(());
        }
        Options::TooMany => {
            println!(
                "Invalid use of commands some of arguments cannot be used together.\n{}",
                help()
            );
        }
        Options::Nothing => {
            let mut sys_comand = sysCommand::new(typer);
            sys_comand.args([
                "-d",
                &delay.to_string(),
                &get_combinations()?.get_sequence(&commands[0], &commands)?,
            ]);
            sys_comand.spawn()?;
        }
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e)
    }
}
