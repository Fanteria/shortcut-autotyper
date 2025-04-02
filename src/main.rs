use clap::{Parser, ValueEnum};
use shortcut_autotyper::{
    typer::{TypeText, Wtype, XDoTool},
    Combinations,
};
use std::{env::var, error::Error, fs::File, process::exit};

const CONFIG_NAME: &str = "/.shortcut_autotyper.json";
const DEFAULT_DELAY: usize = 50;

#[derive(ValueEnum, Clone, Debug)]
pub enum Typer {
    Xdotool,
    Wtype,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Set path to config file with sequences and combinations.
    #[arg(short, long)]
    #[arg(default_value_t = (||{ var("HOME").unwrap_or("~".into()) + CONFIG_NAME})())]
    pub config: String,

    /// List all avaible commands.
    #[arg(long)]
    list: bool,

    /// List all avaible commands with output.
    #[arg(long)]
    list_full: bool,

    /// Set delay between two key strokes. [default: 50]
    #[arg(short, long)]
    delay: Option<usize>,

    /// Binary to send text to terminal.
    #[arg(short, long, default_value = "xdotool")]
    typer: Typer,

    // HERE
    commands: Vec<String>,
}

impl Args {
    // TODO new is not right name
    pub fn run() -> Result<Self, Box<dyn Error>> {
        let args = Self::parse();
        if args.list {
            args.get_combinations()?
                .list_all_commands()
                .iter()
                .filter(|command| !command.starts_with("_"))
                .for_each(|command| {
                    println!("{command}");
                });
            exit(0);
        }
        if args.list_full {
            let combinations = args.get_combinations()?;
            combinations
                .list_all_commands()
                .iter()
                .filter(|command| !command.starts_with("_"))
                .for_each(|command| {
                    println!(
                        "{command}: {}",
                        combinations
                            .get_sequence(command, &Vec::new())
                            .unwrap()
                            .replace("\n", "\\n")
                    );
                });
            exit(0);
        }
        Ok(args)
    }

    fn get_combinations(&self) -> Result<Combinations, Box<dyn Error>> {
        Ok(serde_json::from_reader(File::open(&self.config)?)?)
    }

    fn type_text(&self) -> Result<(), Box<dyn Error>> {
        let c = self.get_combinations()?;
        let sequence = c.get_sequence(&self.commands[0], &self.commands)?;
        let delay = self
            .delay
            .or_else(|| c.get_delay(&self.commands[0]))
            .unwrap_or(DEFAULT_DELAY);
        match &self.typer {
            Typer::Xdotool => XDoTool::type_text(sequence, delay)?,
            Typer::Wtype => Wtype::type_text(sequence, delay)?,
        }
        Ok(())
    }
}

fn main() {
    if let Err(e) = Args::run().and_then(|a| a.type_text()) {
        eprintln!("{}", e)
    }
}
