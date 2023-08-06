use crate::{
    command::Command,
    error::{ATResult, ATVecResult, ErrType},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Keys, HashMap},
    str::FromStr,
};

#[derive(Deserialize, Serialize, Default, Debug, PartialEq)]
/// Structure for collection of defined sequences.
pub struct Sequences(HashMap<String, String>);

impl Sequences {
    /// Create new instance of [`Sequences`] if identification name is valid
    /// [`Command::valid_name()`] and unique.
    /// [`Sequences`] are list of tuples, where first value is name
    /// and second required sequence
    ///
    /// ```
    /// # use shortcut_autotyper::error::ErrType;
    /// # use shortcut_autotyper::*;
    /// let seq = Sequences::new(&[("A", "seq a"), ("B", "seq b")]);
    /// assert!(seq.is_ok());
    /// let seq = Sequences::new(&[("~", "invalid seq")]);
    /// assert_eq!(seq, Err(ErrType::InvalidKeyFormat(String::from("~")).into()));
    /// ```
    pub fn new(sequences: &[(&str, &str)]) -> ATResult<Sequences> {
        let mut seq = Sequences(HashMap::new());
        for (key, value) in sequences.iter() {
            seq.insert(key, value)?;
        }
        Ok(seq)
    }

    /// Generate sequence from given `key`. Returns string with generated
    /// sequence or error if `key` is invalid or sequence `key` does not exists.
    ///
    /// ```
    /// # use shortcut_autotyper::error::ErrType;
    /// # use shortcut_autotyper::*;
    /// let seq = Sequences::new(&[("A", "seq a,")]).unwrap();
    /// assert_eq!(seq.get_sequence("A3").unwrap(), String::from("seq a,seq a,seq a,"));
    /// ```
    pub fn get_sequence(&self, key: &str) -> ATResult<String> {
        let command = Command::from_str(key)?;
        match self.0.get(command.get_name()) {
            Some(s) => Ok(s.repeat(command.get_times())),
            None => ErrType::SequenceNotExist(String::from(command.get_name())).into(),
        }
    }

    /// Generate sequence from given [`Command`]. Returns string with generated
    /// sequence or error if sequence does not constraint value with command name.
    pub fn get_sequence_cmd(&self, command: &Command) -> ATResult<String> {
        match self.0.get(command.get_name()) {
            Some(s) => Ok(s.repeat(command.get_times())),
            None => ErrType::SequenceNotExist(String::from(command.get_name())).into(),
        }
    }

    /// Returns a reference to the value corresponding to the key.
    /// If value does not exists. Then returns [`None`].
    pub fn get(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }

    /// Find all keys invalid and returns errors caused by them
    /// as [`ATVecResult`]. If there are no invalid keys returns `Ok(())`.
    pub fn get_errors(&self) -> ATVecResult<()> {
        Command::are_valid_names(self.0.keys())
    }

    /// Check if all keys in sequence are valid.
    pub fn is_valid(&self) -> bool {
        !self
            .0
            .iter()
            .any(|(key, _)| Command::valid_name(key).is_err())
    }

    /// Insert `key` and `value` to sequences. If `key` is invalid,
    /// or is in sequences, error will be returned.
    pub fn insert(&mut self, key: &str, value: &str) -> ATResult<()> {
        Command::valid_name(key)?;
        match self.0.get(key) {
            Some(_) => ErrType::KeyIsInSequences(String::from(key)).into(),
            None => {
                self.0.insert(String::from(key), String::from(value));
                Ok(())
            }
        }
    }

    pub fn get_keys(&self) -> Keys<'_, String, String> {
        self.0.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_sequences() -> Sequences {
        Sequences::new(&[("A", "A1"), ("B", "B1"), ("AB", "AB1"), ("BA", "BA1")]).unwrap()
    }

    #[test]
    fn new() -> ATResult<()> {
        Sequences::new(&[("A", "A1"), ("B", "B1")])?;
        Ok(())
    }

    #[test]
    fn insert() {
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
    fn basic_get_sequence() {
        let seq = example_sequences();
        assert_eq!(seq.get_sequence("A"), Ok(String::from("A1")));
        assert_eq!(seq.get_sequence("AB"), Ok(String::from("AB1")));
        assert_eq!(
            seq.get_sequence("X"),
            ErrType::SequenceNotExist(String::from("X")).into()
        );
        assert_eq!(
            seq.get_sequence("Y"),
            ErrType::SequenceNotExist(String::from("Y")).into()
        );
    }

    #[test]
    fn numbered_get_sequence() -> ATResult<()> {
        let seq = example_sequences();
        assert_eq!(&seq.get_sequence("B1")?, "B1");
        assert_eq!(&seq.get_sequence("BA1")?, "BA1");

        assert_eq!(&seq.get_sequence("A2")?, "A1A1");
        assert_eq!(&seq.get_sequence("B2")?, "B1B1");

        assert_eq!(
            seq.get_sequence("X2"),
            ErrType::SequenceNotExist(String::from("X")).into()
        );
        assert_eq!(
            seq.get_sequence("Y5"),
            ErrType::SequenceNotExist(String::from("Y")).into()
        );

        Ok(())
    }

    #[test]
    fn range_get_sequence() -> ATResult<()> {
        let seq = example_sequences();
        let repeat_check = |sequence: &str, output: &str, min, max| -> ATResult<()> {
            let mut generated;
            for _ in 0..=100 {
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
    fn get_errors() {
        assert_eq!(example_sequences().get_errors(), Ok(()));

        let mut seq = HashMap::new();
        seq.insert(String::from(""), String::from(""));
        seq.insert(String::from("1"), String::from(""));
        seq.insert(String::from("A4"), String::from(""));
        seq.insert(String::from("/A"), String::from(""));
        seq.insert(String::from("B A"), String::from(""));
        let seq = Sequences(seq);
        let errors = seq.get_errors().unwrap_err();

        assert_eq!(errors.len(), 5);
        assert!(errors.contains(&ErrType::KeyCannotBeEmpty.into()));
        assert!(errors.contains(&ErrType::InvalidKeyFormat(String::from("1")).into()));
        assert!(errors.contains(&ErrType::InvalidKeyFormat(String::from("A4")).into()));
        assert!(errors.contains(&ErrType::InvalidKeyFormat(String::from("/A")).into()));
        assert!(errors.contains(&ErrType::InvalidKeyFormat(String::from("B A")).into()));
    }

    #[test]
    fn is_valid() {
        assert!(example_sequences().is_valid());

        let mut seq = HashMap::new();
        seq.insert(String::from("1"), String::from(""));
        assert!(!Sequences(seq).is_valid());

        let mut seq = HashMap::new();
        seq.insert(String::from("A4"), String::from(""));
        assert!(!Sequences(seq).is_valid());

        let mut seq = HashMap::new();
        seq.insert(String::from("/A"), String::from(""));
        assert!(!Sequences(seq).is_valid());

        let mut seq = HashMap::new();
        seq.insert(String::from("B A"), String::from(""));
        assert!(!Sequences(seq).is_valid());
    }

    #[test]
    fn de_serialization() {
        let seq = example_sequences();
        let serialized = serde_json::to_string(&seq).unwrap();
        println!("{}", serialized);
        let deserialized = serde_json::from_str::<Sequences>(&serialized).unwrap();
        assert_eq!(seq, deserialized);
    }
}
