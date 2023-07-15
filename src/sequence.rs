use crate::error::{ATResult, ErrAutoType, ErrType};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Default, Debug, PartialEq)]
pub struct Sequences(HashMap<String, String>);

impl Sequences {
    pub fn new(sequences: &[(&str, &str)]) -> ATResult<Sequences> {
        let mut seq = Sequences(HashMap::new());
        for (key, value) in sequences.iter() {
            seq.insert(key, value)?;
        }
        Ok(seq)
    }

    pub fn get_sequence(&self, key: &str) -> ATResult<String> {
        let index = match key.chars().position(|c| c.is_digit(10)) {
            Some(i) => i,
            None => {
                return match self.0.get(key) {
                    Some(s) => Ok(s.clone()),
                    None => Err(ErrAutoType::new(ErrType::SequenceNotExist(String::from(
                        key,
                    )))),
                }
            }
        };

        let base = match self.0.get(&key[..index]) {
            Some(s) => s,
            None => {
                return Err(ErrAutoType::new(ErrType::SequenceNotExist(String::from(
                    &key[..index],
                ))))
            }
        };
        let args = &key[index..];
        if let Ok(times) = args.parse::<usize>() {
            return Ok(base.repeat(times));
        }
        let index = match args.find("..") {
            Some(i) => i,
            None => {
                return Err(ErrAutoType::new(ErrType::WrongSequenceArg(String::from(
                    args,
                ))))
            }
        };

        match (
            args[..index].parse::<usize>(),
            args[index + 2..].parse::<usize>(),
        ) {
            (Ok(start), Ok(end)) => Ok(base.repeat(rand::thread_rng().gen_range(start..=end))),
            _ => Err(ErrAutoType::new(ErrType::WrongSequenceArg(String::from(
                args,
            )))),
        }
    }

    fn is_valid_key(key: &str) -> bool {
        key.chars().position(|c| !c.is_alphabetic()).is_none()
    }

    pub fn is_valid(&self) -> bool {
        self.0
            .iter()
            .find(|(key, _)| Self::is_valid_key(key))
            .is_none()
    }

    pub fn insert(&mut self, key: &str, value: &str) -> ATResult<()> {
        if Self::is_valid_key(key) {
            match self.0.get(key) {
                Some(_) => Err(ErrAutoType::new(ErrType::KeyIsInSequences(String::from(
                    key,
                )))),
                None => {
                    self.0.insert(String::from(key), String::from(value));
                    Ok(())
                }
            }
        } else {
            Err(ErrAutoType::new(ErrType::InvalidSequenceKey(String::from(
                key,
            ))))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_sequences() -> Sequences {
        Sequences::new(&[("A", "A1"), ("B", "B1"), ("AB", "AB1"), ("BA", "BA1")]).unwrap()
    }

    #[test]
    fn test_new() -> ATResult<()> {
        Sequences::new(&[("A", "A1"), ("B", "B1")])?;
        Ok(())
    }

    #[test]
    fn test_insert() {
        let mut seq = Sequences::default();
        assert!(seq.insert("A", "").is_ok());
        assert!(seq.insert("asdf", "").is_ok());
        assert!(seq.insert("aSdF", "").is_ok());
        assert!(seq.insert("asdf", "").is_err());
        assert!(seq.insert("asdf1", "").is_err());
        assert!(seq.insert("as12df1", "").is_err());
        assert!(seq.insert("asdf.", "").is_err());
        assert!(seq.insert("asdf/", "").is_err());
    }

    #[test]
    fn test_basic_get_sequence() {
        let seq = example_sequences();
        assert_eq!(seq.get_sequence("A"), Ok(String::from("A1")));
        assert_eq!(seq.get_sequence("AB"), Ok(String::from("AB1")));
        assert_eq!(
            seq.get_sequence("X"),
            Err(ErrAutoType::new(ErrType::SequenceNotExist(String::from(
                "X"
            ))))
        );
        assert_eq!(
            seq.get_sequence("Y"),
            Err(ErrAutoType::new(ErrType::SequenceNotExist(String::from(
                "Y"
            ))))
        );
    }

    #[test]
    fn test_numbered_get_sequence() -> ATResult<()> {
        let seq = example_sequences();
        assert_eq!(&seq.get_sequence("B1")?, "B1");
        assert_eq!(&seq.get_sequence("BA1")?, "BA1");

        assert_eq!(&seq.get_sequence("A2")?, "A1A1");
        assert_eq!(&seq.get_sequence("B2")?, "B1B1");

        assert_eq!(
            seq.get_sequence("X2"),
            Err(ErrAutoType::new(ErrType::SequenceNotExist(String::from(
                "X"
            ))))
        );
        assert_eq!(
            seq.get_sequence("Y5"),
            Err(ErrAutoType::new(ErrType::SequenceNotExist(String::from(
                "Y"
            ))))
        );

        Ok(())
    }

    #[test]
    fn test_range_get_sequence() -> ATResult<()> {
        let seq = example_sequences();
        let repeat_check = |sequence: &str, output: &str, min, max| -> ATResult<()> {
            let mut generated;
            for i in 0..=100 {
                generated = seq.get_sequence(sequence)?;
                assert!(generated.len() % output.len() == 0);
                assert!(generated.len() / output.len() >= min);
                assert!(generated.len() / output.len() <= max);
            }
            Ok(())
        };

        repeat_check("B2..5", "B1", 2, 5)?;
        repeat_check("A2..4", "A1", 2, 4)?;
        repeat_check("AB5..10", "AB1", 5, 10)?;

        Ok(())
    }

    #[test]
    fn test_de_serialization() {
        let seq = example_sequences();
        let serialized = serde_json::to_string(&seq).unwrap();
        println!("{}", serialized);
        let deserialized = serde_json::from_str::<Sequences>(&serialized).unwrap();
        assert_eq!(seq, deserialized);
    }
}
