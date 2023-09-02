use crate::{
    command::Command,
    error::{ATResult, ATVecResult, ErrAutoType, ErrType},
    sequence::Sequences,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};

/// Combinations of existing [`Sequences`].
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Combinations {
    combinations: HashMap<String, String>,
    sequences: Sequences,
}

impl Combinations {
    /// Create new instance of [`Combinations`] if identification name is valid
    /// and unique in combinations and `sequences`. Otherwise returns error.
    ///
    /// ```
    /// # use shortcut_autotyper::error::ErrType;
    /// # use shortcut_autotyper::*;
    /// let seq = Sequences::new(&[("A", "seq a"), ("B", "b")]).unwrap();
    /// let comb = Combinations::new(seq, &[("X", "A2 B3")]);
    /// assert!(comb.is_ok());
    /// ```
    pub fn new(sequences: Sequences, combinations: &[(&str, &str)]) -> ATResult<Combinations> {
        let mut comb = Combinations {
            combinations: HashMap::new(),
            sequences,
        };
        for (key, value) in combinations.iter() {
            comb.insert(key, value)?;
        }
        Ok(comb)
    }

    /// Works similarly as [`Combinations::get_sequence()`], only takes reference
    /// to [`Command`] instead of `key`.
    pub fn get_sequence_cmd(&self, command: &Command, args: &Vec<String>) -> ATResult<String> {
        match self.combinations.get(command.get_name()) {
            Some(sequence) => {
                let commands = Self::decompose(sequence)?;
                (0..command.get_times())
                    .map(|_| {
                        commands
                            .iter()
                            .map(|cmd| self.get_sequence_cmd(cmd, args))
                            .collect::<ATResult<String>>()
                    })
                    .collect()
            }
            None => self.sequences.get_sequence_cmd(command, args),
        }
    }

    /// Generate sequence from given `key`. Returns string with generated
    /// sequence or error if `key` is invalid or `key` does not exists in sequences
    /// or combinations.
    ///
    /// ```
    /// # use shortcut_autotyper::error::ErrType;
    /// # use shortcut_autotyper::*;
    /// let seq = Sequences::new(&[("A", "seq a"), ("B", "b")]).unwrap();
    /// let comb = Combinations::new(seq, &[("X", "A B3")]).unwrap();
    /// assert_eq!(comb.get_sequence("X", &Vec::new()).unwrap(), String::from("seq abbb"));
    /// ```
    pub fn get_sequence(&self, key: &str, args: &Vec<String>) -> ATResult<String> {
        Self::decompose(key)?
            .iter()
            .map(|command| self.get_sequence_cmd(command, args))
            .collect()
    }

    /// Decompose string to list of [`Command`]s.
    fn decompose(combination: &str) -> ATResult<Vec<Command>> {
        combination
            .split_whitespace()
            .map(Command::from_str)
            .collect()
    }

    /// Returns list of all errors in [`Combinations`]. If there is no error,
    /// returns `Ok(())`.
    pub fn get_errors(&self) -> ATVecResult<()> {
        let mut errors = Vec::new();
        if let Err(e) = &mut Command::are_valid_names(self.combinations.keys()) {
            errors.append(e)
        }
        if let Err(e) = &mut self.sequences.get_errors() {
            errors.append(e)
        }
        self.combinations.values().for_each(|combination| {
            match Combinations::decompose(combination) {
                Ok(commands) => commands.iter().for_each(|command| match command.valid() {
                    Ok(_) => match self.sequences.get(command.get_name()) {
                        Some(_) => {}
                        None => errors.push(ErrAutoType::new(ErrType::UnknownSequence(
                            String::from(command.get_name()),
                        ))),
                    },
                    Err(e) => errors.push(e),
                }),
                Err(e) => errors.push(e),
            }
        });
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Check if [`Combinations`] are valid.
    pub fn is_valid(&self) -> bool {
        !self.combinations.iter().any(|(key, value)| {
            Command::valid_name(key).is_err()
                || match Self::decompose(value) {
                    Ok(combinations) => combinations
                        .iter()
                        .any(|command| self.sequences.get(command.get_name()).is_none()),
                    Err(_) => true,
                }
        })
    }

    /// Insert new combination to existing combinations if `key` is valid
    /// and in `value` are only existing [`Sequences`] or [`Combinations`].
    ///
    /// ```
    /// # use shortcut_autotyper::error::ErrType;
    /// # use shortcut_autotyper::*;
    ///    let seq = Sequences::new(&[("A", "seq a"), ("B", "b")]).unwrap();
    ///    let mut comb = Combinations::new(seq, &[]).unwrap();
    ///    assert_eq!(comb.insert("X", "A B3"), Ok(()));
    ///    assert_eq!(
    ///        comb.insert("X", "A B3"),
    ///        Err(ErrType::KeyIsInCombinations(String::from("X")).into())
    ///    );
    ///    assert_eq!(
    ///        comb.insert("A", "A B3"),
    ///        Err(ErrType::KeyIsInSequences(String::from("A")).into())
    ///    );
    ///    assert_eq!(
    ///        comb.insert("C", "A D3"),
    ///        Err(ErrType::SequenceNotExist(String::from("D")).into())
    ///    );
    /// ```
    pub fn insert(&mut self, key: &str, value: &str) -> ATResult<()> {
        Command::valid_name(key)?;
        if self.sequences.get(key).is_some() {
            return ErrType::KeyIsInSequences(String::from(key)).into();
        };
        if self.combinations.get(key).is_some() {
            return ErrType::KeyIsInCombinations(String::from(key)).into();
        };
        let commands = Self::decompose(value)?;
        if let Some(cmd) = commands
            .iter()
            .find(|cmd| self.sequences.get(cmd.get_name()).is_none())
        {
            return ErrType::SequenceNotExist(String::from(cmd.get_name())).into();
        };

        self.combinations
            .insert(String::from(key), String::from(value));

        Ok(())
    }

    /// Retrieves a sorted list of references to all
    /// available commands in the current context.
    ///
    /// This function returns a vector containing references
    /// to all commands found in the `sequences` and `combinations`
    /// data structures of the current context. The commands
    /// are sorted alphabetically in ascending order.
    ///
    /// # Return Value
    ///
    /// A `Vec<&String>` containing references to all available commands,
    /// sorted in ascending order.
    pub fn list_all_commands(&self) -> Vec<&String> {
        let mut commands = self
            .sequences
            .get_keys()
            .chain(self.combinations.keys())
            .collect::<Vec<&String>>();
        commands.sort();
        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_combination() -> Combinations {
        Combinations::new(
            Sequences::new(&[("A", "A1"), ("B", "B1"), ("AB", "AB1"), ("BA", "BA1")]).unwrap(),
            &[("X", "A2 B3..5")],
        )
        .unwrap()
    }

    #[test]
    fn get_sequence() -> ATResult<()> {
        let combinations = example_combination();
        for _ in 0..1000 {
            let seq = combinations.get_sequence("X1", &Vec::new())?;
            assert!(seq.len() >= "A1A1B1B1B1".len());
            assert!(seq.len() <= "A1A1B1B1B1B1B1".len());
            assert!(seq.starts_with("A1A1B1B1B1"));
        }
        for _ in 0..1000 {
            let seq = combinations.get_sequence("X2", &Vec::new())?;
            assert!(seq.len() >= "A1A1B1B1B1".len() * 2, "Sequence: {}\n", seq);
            assert!(
                seq.len() <= "A1A1B1B1B1B1B1".len() * 2,
                "Sequence: {}\n",
                seq
            );
            assert!(seq.starts_with("A1A1B1B1B1"), "Sequence: {}\n", seq);
        }
        for _ in 0..1000 {
            let seq = combinations.get_sequence("X3..5", &Vec::new())?;
            assert!(seq.len() >= "A1A1B1B1B1".len() * 3, "Sequence: {}\n", seq);
            assert!(
                seq.len() <= "A1A1B1B1B1B1B1".len() * 5,
                "Sequence: {}\n",
                seq
            );
            assert!(seq.starts_with("A1A1B1B1B1"), "Sequence: {}\n", seq);
        }
        Ok(())
    }

    #[test]
    fn decompose() -> ATResult<()> {
        use crate::command::Command as cmd;
        assert_eq!(
            Combinations::decompose("A B C D")?,
            vec![cmd::new("A"), cmd::new("B"), cmd::new("C"), cmd::new("D")]
        );
        assert_eq!(
            Combinations::decompose("  A    B     ")?,
            vec![cmd::new("A"), cmd::new("B")]
        );
        Ok(())
    }

    #[test]
    fn get_errors() {
        let get_sequence =
            || Sequences::new(&[("A", "A1"), ("B", "B1"), ("AB", "AB1"), ("BA", "BA1")]).unwrap();
        let get_combinations = |combs: &[(&str, &str)]| {
            let mut combinations = HashMap::new();
            combs.iter().for_each(|(key, value)| {
                combinations.insert(String::from(*key), String::from(*value));
            });
            combinations
        };

        let errors = Combinations::new(get_sequence(), &[("X", "A3 B3..5")])
            .unwrap()
            .get_errors();
        assert_eq!(errors, Ok(()));
        let errors = Combinations {
            sequences: get_sequence(),
            combinations: get_combinations(&[("X", "A3 B~3..5"), ("Y", "A C3")]),
        }
        .get_errors()
        .unwrap_err();
        assert!(
            errors.contains(&ErrAutoType::new(ErrType::InvalidKeyFormat(String::from(
                "B~"
            ))))
        );
        assert!(
            errors.contains(&ErrAutoType::new(ErrType::UnknownSequence(String::from(
                "C"
            ))))
        );
        for i in errors {
            println!("{}", i);
        }
    }

    #[test]
    fn is_valid() {
        let get_sequence =
            || Sequences::new(&[("A", "A1"), ("B", "B1"), ("AB", "AB1"), ("BA", "BA1")]).unwrap();
        let get_combinations = |combs: &[(&str, &str)]| {
            let mut combinations = HashMap::new();
            combs.iter().for_each(|(key, value)| {
                combinations.insert(String::from(*key), String::from(*value));
            });
            combinations
        };

        assert!(Combinations::new(get_sequence(), &[("X", "A3 B3..5")])
            .unwrap()
            .is_valid());
        assert!(!Combinations {
            sequences: get_sequence(),
            combinations: get_combinations(&[("X", "A3 B~3..5")]),
        }
        .is_valid());
        assert!(!Combinations {
            sequences: get_sequence(),
            combinations: get_combinations(&[("X", "A3 C3..5")]),
        }
        .is_valid());
    }

    #[test]
    fn insert() -> ATResult<()> {
        let mut comb = Combinations {
            sequences: Sequences::new(&[("A", "A1"), ("B", "B1"), ("AB", "AB1"), ("BA", "BA1")])
                .unwrap(),
            combinations: HashMap::new(),
        };
        comb.insert("X", "A5")?;
        assert!(comb.combinations.get("X").is_some());
        comb.insert("Y", "B4 AB1..3")?;
        assert!(comb.combinations.get("Y").is_some());

        assert_eq!(
            comb.insert("X", ""),
            ErrType::KeyIsInCombinations(String::from("X")).into()
        );
        assert_eq!(
            comb.insert("Y", ""),
            ErrType::KeyIsInCombinations(String::from("Y")).into()
        );

        assert_eq!(
            comb.insert("A", ""),
            ErrType::KeyIsInSequences(String::from("A")).into()
        );

        assert_eq!(
            comb.insert("AB", ""),
            ErrType::KeyIsInSequences(String::from("AB")).into()
        );

        Ok(())
    }

    #[test]
    fn de_serialization() {
        let comb = example_combination();
        let serialized = serde_json::to_string(&comb).unwrap();
        println!("{}", serialized);
        let deserialized = serde_json::from_str::<Combinations>(&serialized).unwrap();
        assert_eq!(comb, deserialized);
    }
}
