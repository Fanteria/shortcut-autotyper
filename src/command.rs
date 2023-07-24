use crate::error::{ATResult, ATVecResult, ErrAutoType, ErrType};
use rand::Rng;
use std::fmt::{self, Display};
use std::{ops::Range, str::FromStr};

#[derive(Debug, PartialEq, Eq)]
enum Times {
    Number(usize),
    Range(Range<usize>),
}

/// Basic structure containing name and number of repetition.
#[derive(Debug, PartialEq, Eq)]
pub struct Command {
    name: String,
    times: Option<Times>,
}

impl Command {
    /// Create new `Command` with only one repetition.
    ///
    /// - `name` is given name and does not have to be valid
    pub fn new(name: &str) -> Command {
        Command {
            name: String::from(name),
            times: None,
        }
    }

    /// Create new `Command` with given number of repetitions.
    ///
    /// - `name` is given name and does not have to be valid
    /// - `num` is the number of repetitions
    pub fn new_number(name: &str, num: usize) -> Command {
        Command {
            name: String::from(name),
            times: Some(Times::Number(num)),
        }
    }

    /// Create new `Command` with number of repetitions randomly
    /// selected from given range.
    ///
    /// - `name` is given name and have not to be valid
    /// - `range` is a half-open range of possible repetitions
    ///
    /// ```
    /// # use shortcut_autotyper::Command;
    /// let cmd = Command::new_range("A", 3..5);
    /// let times = cmd.get_times();
    /// assert!(times >= 3);
    /// assert!(times < 5);
    /// ```
    pub fn new_range(name: &str, range: Range<usize>) -> Command {
        Command {
            name: String::from(name),
            times: Some(Times::Range(range)),
        }
    }

    /// Check if `self` structure have valid name.
    pub fn valid(&self) -> ATResult<()> {
        Self::valid_name(&self.name)
    }

    /// Check if given `name` is valid. A valid name can consist
    /// only of alphabetical characters. If given name is not valid,
    /// then it returns an error with [`ErrType::InvalidKeyFormat`].
    ///
    /// ```
    /// # use shortcut_autotyper::Command;
    /// # use shortcut_autotyper::error::ErrType;
    /// assert_eq!(Command::valid_name("A"), Ok(()));
    /// assert_eq!(
    ///     Command::valid_name("A~"),
    ///     Err(ErrType::InvalidKeyFormat(String::from("A~")).into())
    /// );
    /// assert_eq!(
    ///     Command::valid_name("A B"),
    ///     Err(ErrType::InvalidKeyFormat(String::from("A B")).into())
    /// );
    /// ```
    pub fn valid_name(name: &str) -> ATResult<()> {
        if name.is_empty() {
            return ErrType::KeyCannotBeEmpty.into();
        }

        if !name.chars().any(|c| !c.is_alphabetic()) {
            Ok(())
        } else {
            ErrType::InvalidKeyFormat(String::from(name)).into()
        }
    }

    /// Check if given names are valid same as [`Command::valid_name()`]
    /// and returns a `Vec` of [`ErrType::InvalidKeyFormat`].
    ///
    /// - `iter` is iterator of given names
    pub fn are_valid_names<'a, I: Iterator<Item = &'a String>>(iter: I) -> ATVecResult<()> {
        let errors: Vec<_> = iter
            .filter_map(|name| Command::valid_name(name).err())
            .collect();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Return number of repetition of command, for range return one of
    /// random possible options.
    pub fn get_times(&self) -> usize {
        match &self.times {
            Some(Times::Number(n)) => *n,
            Some(Times::Range(r)) => rand::thread_rng().gen_range(r.start..r.end),
            None => 1,
        }
    }

    /// Return reference to name of the command.
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

impl FromStr for Command {
    type Err = ErrAutoType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().position(|c| c.is_ascii_digit()) {
            Some(i) => {
                Self::valid_name(&s[..i])?;
                Ok(Command {
                    name: String::from(&s[..i]),
                    times: Some(Times::from_str(&s[i..])?),
                })
            }
            None => {
                Self::valid_name(s)?;
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
            None => return ErrType::WrongSequenceArg(String::from(s)).into(),
        };
        match (s[..index].parse::<usize>(), s[index + 2..].parse::<usize>()) {
            (Ok(start), Ok(end)) if start <= end => Ok(Times::Range(start..end)),
            (Ok(start), Ok(end)) => ErrType::RangeMustNotBeEmpty(start..end).into(),
            _ => ErrType::WrongSequenceArg(String::from(s)).into(),
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

    #[test]
    fn valid_name() -> ATResult<()> {
        Command::valid_name("A")?;
        Command::valid_name("BCDE")?;
        assert_eq!(Command::valid_name(""), ErrType::KeyCannotBeEmpty.into());
        assert_eq!(
            Command::valid_name("1"),
            ErrType::InvalidKeyFormat(String::from("1")).into()
        );
        assert_eq!(
            Command::valid_name("A4"),
            ErrType::InvalidKeyFormat(String::from("A4")).into()
        );
        assert_eq!(
            Command::valid_name("/A"),
            ErrType::InvalidKeyFormat(String::from("/A")).into()
        );
        assert_eq!(
            Command::valid_name("B A"),
            ErrType::InvalidKeyFormat(String::from("B A")).into()
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
        assert_eq!(Command::from_str("A")?, Command::new("A"));
        assert_eq!(Command::from_str("AB")?, Command::new("AB"));
        assert_eq!(Command::from_str("A1")?, Command::new_number("A", 1));
        assert_eq!(Command::from_str("CDE5")?, Command::new_number("CDE", 5));
        assert_eq!(Command::from_str("A3..6")?, Command::new_range("A", 3..6));
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
                let times = Command::new_range("", start..end).get_times();
                assert!(times >= start);
                assert!(times < end);
            }
        };
        assert_eq!(Command::new("").get_times(), 1);
        assert_eq!(Command::new_number("", 1).get_times(), 1);
        assert_eq!(Command::new_number("", 5).get_times(), 5);
        assert_eq!(Command::new_number("", 66).get_times(), 66);
        range_check(10, 100);
        range_check(0, 3);
        range_check(3, 7);
    }
}
