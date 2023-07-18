use crate::error::{ATResult, ErrAutoType, ErrType};
use rand::Rng;
use std::{ops::Range, str::FromStr};
use std::fmt::{self, Display};

#[derive(Debug, PartialEq, Eq)]
enum Times {
    Number(usize),
    Range(Range<usize>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Command {
    name: String,
    times: Option<Times>,
}

impl Command {
    pub fn is_valid(&self) -> ATResult<()> {
        Self::is_valid_name(&self.name)
    }

    pub fn is_valid_name(name: &str) -> ATResult<()> {
        if name.is_empty() {
            return Err(ErrAutoType::new(ErrType::KeyCannotBeEmpty));
        }

        if name.chars().position(|c| !c.is_alphabetic()).is_none() {
            Ok(())
        } else {
            Err(ErrAutoType::new(ErrType::InvalidKeyFormat(String::from(
                name,
            ))))
        }
    }

    pub fn get_times(&self) -> usize {
        let x = 1..2;
        match &self.times {
            Some(Times::Number(n)) => *n,
            Some(Times::Range(r)) => rand::thread_rng().gen_range(r.start..r.end),
            None => 1,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

impl FromStr for Command {
    type Err = ErrAutoType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().position(|c| c.is_digit(10)) {
            Some(i) => {
                Self::is_valid_name(&s[..i])?;
                Ok(Command {
                    name: String::from(&s[..i]),
                    times: Some(Times::from_str(&s[i..])?),
                })
            }
            None => {
                Self::is_valid_name(s)?;
                Ok(Command {
                    name: String::from(s),
                    times: None,
                })
            }
        }
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.times {
            Some(times) => write!(f, "{}{}", self.name, times),
            None => write!(f, "{}", self.name),
        }
    }
}

impl FromStr for Times {
    type Err = ErrAutoType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(times) = s.parse::<usize>() {
            return Ok(Times::Number(times));
        };
        let index = match s.find("..") {
            Some(i) => i,
            None => return Err(ErrAutoType::new(ErrType::WrongSequenceArg(String::from(s)))),
        };
        match (s[..index].parse::<usize>(), s[index + 2..].parse::<usize>()) {
            (Ok(start), Ok(end)) if start <= end => Ok(Times::Range(start..end)),
            (Ok(start), Ok(end)) => Err(ErrAutoType::new(ErrType::RangeMustNotBeEmpty(start..end))),
            _ => Err(ErrAutoType::new(ErrType::WrongSequenceArg(String::from(s)))),
        }
    }
}

impl Display for Times {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Times::Number(n) => write!(f, "{n}"),
            Times::Range(r) => write!(f, "{}..{}", r.start, r.end),
        }
        
    }
}

#[cfg(test)]
mod tests {
    use crate::error::ATResult;

    use super::*;

    fn new_command(name: &str) -> Command {
        Command {
            name: String::from(name),
            times: None,
        }
    }

    fn new_command_number(name: &str, num: usize) -> Command {
        Command {
            name: String::from(name),
            times: Some(Times::Number(num)),
        }
    }

    fn new_command_range(name: &str, range: Range<usize>) -> Command {
        Command {
            name: String::from(name),
            times: Some(Times::Range(range)),
        }
    }

    #[test]
    fn is_valid_name() -> ATResult<()> {
        Command::is_valid_name("A")?;
        Command::is_valid_name("BCDE")?;
        assert_eq!(
            Command::is_valid_name(""),
            Err(ErrAutoType::new(ErrType::KeyCannotBeEmpty))
        );
        assert_eq!(
            Command::is_valid_name("1"),
            Err(ErrAutoType::new(ErrType::InvalidKeyFormat(String::from(
                "1"
            ))))
        );
        assert_eq!(
            Command::is_valid_name("A4"),
            Err(ErrAutoType::new(ErrType::InvalidKeyFormat(String::from(
                "A4"
            ))))
        );
        assert_eq!(
            Command::is_valid_name("/A"),
            Err(ErrAutoType::new(ErrType::InvalidKeyFormat(String::from(
                "/A"
            ))))
        );
        assert_eq!(
            Command::is_valid_name("B A"),
            Err(ErrAutoType::new(ErrType::InvalidKeyFormat(String::from(
                "B A"
            ))))
        );
        Ok(())
    }

    #[test]
    fn times_from_str() -> ATResult<()> {
        assert_eq!(Times::from_str("1")?, Times::Number(1));
        assert_eq!(Times::from_str("5")?, Times::Number(5));
        assert_eq!(Times::from_str("57")?, Times::Number(57));
        assert_eq!(Times::from_str("5..7")?, Times::Range(5..7));
        assert!(Times::from_str("5..").is_err());
        assert!(Times::from_str("..7").is_err());
        assert!(Times::from_str("5..=7").is_err());
        assert!(Times::from_str("57a37").is_err());
        assert!(Times::from_str("57..7").is_err());
        Ok(())
    }

    #[test]
    fn command_from_str() -> ATResult<()> {
        assert_eq!(Command::from_str("A")?, new_command("A"));
        assert_eq!(Command::from_str("AB")?, new_command("AB"));
        assert_eq!(Command::from_str("A1")?, new_command_number("A", 1));
        assert_eq!(Command::from_str("CDE5")?, new_command_number("CDE", 5));
        assert_eq!(Command::from_str("A3..6")?, new_command_range("A", 3..6));
        assert!(Command::from_str("").is_err());
        assert!(Command::from_str("A B").is_err());
        assert!(Command::from_str("A 5").is_err());
        assert!(Command::from_str("4").is_err());
        Ok(())
    }

    #[test]
    fn get_times() {
        let range_check = |start, end| {
            for _ in 0..100 {
                let times = new_command_range("", start..end).get_times();
                assert!(times >= start);
                assert!(times < end);
            }
        };
        assert_eq!(new_command("").get_times(), 1);
        assert_eq!(new_command_number("", 1).get_times(), 1);
        assert_eq!(new_command_number("", 5).get_times(), 5);
        assert_eq!(new_command_number("", 66).get_times(), 66);
        range_check(10, 100);
        range_check(0, 3);
        range_check(3, 7);
    }
}
