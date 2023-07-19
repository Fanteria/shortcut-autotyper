use crate::{
    command::Command,
    error::{ATResult, ErrAutoType, ErrType},
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
            None => {
                return Err(ErrAutoType::new(ErrType::KeyIsInCombinations(
                    String::from(command.get_name()),
                )))
            }
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
            .map(|c| Command::from_str(c))
            .collect()
    }

    pub fn is_valid(&self) -> bool {
        self.combinations
            .iter()
            .find(|(key, value)| {
                Command::is_valid_name(key).is_err()
                    || match Self::decompose(value) {
                        Ok(combinations) => combinations
                            .iter()
                            .find(|command| self.sequences.get(command.get_name()).is_none())
                            .is_some(),
                        Err(_) => true,
                    }
            })
            .is_none()
    }

    pub fn insert(&mut self, key: &str, value: &str) -> ATResult<()> {
        Command::is_valid_name(key)?;
        if self.sequences.get(key).is_some() {
            return Err(ErrAutoType::new(ErrType::KeyIsInSequences(String::from(
                key,
            ))));
        };
        if self.combinations.get(key).is_some() {
            return Err(ErrAutoType::new(ErrType::KeyIsInCombinations(
                String::from(key),
            )));
        };
        let commands = Self::decompose(value)?;
        if let Some(cmd) = commands
            .iter()
            .find(|cmd| self.sequences.get(cmd.get_name()).is_none())
        {
            return Err(ErrAutoType::new(ErrType::SequenceNotExist(String::from(
                cmd.get_name(),
            ))));
        };

        self.combinations
            .insert(String::from(key), String::from(value));

        Ok(())
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
    fn new() -> ATResult<()> {
        // TODO implement
        Ok(())
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
            println!("{seq}");
            assert!(seq.len() >= "A1A1B1B1B1".len() * 2, "Sequence: {}\n", seq);
            assert!(seq.len() <= "A1A1B1B1B1B1B1".len() * 2, "Sequence: {}\n", seq);
            assert!(seq.starts_with("A1A1B1B1B1"), "Sequence: {}\n", seq);
        }
        for _ in 0..1000 {
            let seq = combinations.get_sequence("X3..5")?; 
            println!("{seq}");
            assert!(seq.len() >= "A1A1B1B1B1".len() * 3, "Sequence: {}\n", seq);
            assert!(seq.len() <= "A1A1B1B1B1B1B1".len() * 5, "Sequence: {}\n", seq);
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
            Err(ErrAutoType::new(ErrType::KeyIsInCombinations(
                String::from("X")
            )))
        );
        assert_eq!(
            comb.insert("Y", ""),
            Err(ErrAutoType::new(ErrType::KeyIsInCombinations(
                String::from("Y")
            )))
        );

        assert_eq!(
            comb.insert("A", ""),
            Err(ErrAutoType::new(ErrType::KeyIsInSequences(String::from(
                "A"
            ))))
        );

        assert_eq!(
            comb.insert("AB", ""),
            Err(ErrAutoType::new(ErrType::KeyIsInSequences(String::from(
                "AB"
            ))))
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
