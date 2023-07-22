use crate::{
    command::Command,
    error::{ATResult, ATVecResult, ErrAutoType, ErrType},
    sequence::Sequences,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Combinations {
    combinations: HashMap<String, String>,
    sequences: Sequences,
}

impl Combinations {
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

    pub fn get_sequence(&self, key: &str) -> ATResult<String> {
        let command = Command::from_str(key)?;
        let combinations = match self.combinations.get(command.get_name()) {
            Some(c) => Self::decompose(c)?,
            None => return ErrType::KeyIsInCombinations(String::from(command.get_name())).into(),
        };
        let mut seq = String::new();
        for c in combinations.iter() {
            seq.push_str(
                &self
                    .sequences
                    .get_sequence(c.get_name())?
                    .repeat(c.get_times()),
            )
        }
        Ok(seq.repeat(command.get_times()))
    }

    fn decompose(combination: &str) -> ATResult<Vec<Command>> {
        combination
            .split_whitespace()
            .map(Command::from_str)
            .collect()
    }

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
                Ok(commands) => commands
                    .iter()
                    .for_each(|command| match command.is_valid() {
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

    pub fn is_valid(&self) -> bool {
        !self.combinations.iter().any(|(key, value)| {
            Command::is_valid_name(key).is_err()
                || match Self::decompose(value) {
                    Ok(combinations) => combinations
                        .iter()
                        .any(|command| self.sequences.get(command.get_name()).is_none()),
                    Err(_) => true,
                }
        })
    }

    pub fn insert(&mut self, key: &str, value: &str) -> ATResult<()> {
        Command::is_valid_name(key)?;
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

    pub fn deserialize(data: &str) -> ATResult<Combinations> {
        let deserialized = serde_json::from_str::<Combinations>(data).unwrap();
        deserialized.sequences.is_valid();
        deserialized.is_valid();
        // TODO implement
        Ok(deserialized)
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
            let seq = combinations.get_sequence("X1")?;
            assert!(seq.len() >= "A1A1B1B1B1".len());
            assert!(seq.len() <= "A1A1B1B1B1B1B1".len());
            assert!(seq.starts_with("A1A1B1B1B1"));
        }
        for _ in 0..1000 {
            let seq = combinations.get_sequence("X2")?;
            assert!(seq.len() >= "A1A1B1B1B1".len() * 2, "Sequence: {}\n", seq);
            assert!(
                seq.len() <= "A1A1B1B1B1B1B1".len() * 2,
                "Sequence: {}\n",
                seq
            );
            assert!(seq.starts_with("A1A1B1B1B1"), "Sequence: {}\n", seq);
        }
        for _ in 0..1000 {
            let seq = combinations.get_sequence("X3..5")?;
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
