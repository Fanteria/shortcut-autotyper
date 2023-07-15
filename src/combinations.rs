use crate::{error::{ATResult, ErrAutoType, ErrType}, sequence::Sequences};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize)]
pub struct Combinations{
    combinations: HashMap<String, String>,
    sequences: Sequences,
}

impl Combinations {
    pub fn new(sequences: Sequences, combinations: &[(&str, &str)]) -> ATResult<Combinations> {
        let mut comb = Combinations { combinations: HashMap::new(), sequences };
        for (key, value) in combinations.iter() {
            comb.insert(key, value)?;
        }
        Ok(comb)
    }

    pub fn get_sequence(&self, key: &str) -> ATResult<String> {
        // TODO implement
        Ok(String::from(""))
    }

    fn is_valid_key(key: &str) -> bool {
        key.chars().position(|c| !c.is_alphabetic()).is_none()
    }

    fn decompose(combination: &str) -> ATResult<Vec<&str>> {
        let decomposed = Vec::new();
        // TODO implement

        Ok(decomposed)
    }

    fn is_valid_value(value: &str) -> bool {
        // TODO implement
        false
    }

    pub fn is_valid(&self) -> bool {
        self.combinations
            .iter()
            .find(|(key, value)| Self::is_valid_key(key) && Self::is_valid_value(value))
            .is_none()
    }

    pub fn insert(&mut self, key: &str, value: &str) -> ATResult<()> {
        // TODO implement
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // fn example_combination() -> Combinations {
    //     let mut seq = Sequences::default();
    //     seq.insert("A", "A1").unwrap();
    //     seq.insert("B", "B1").unwrap();
    //     seq.insert("AB", "AB1").unwrap();
    //     seq.insert("BA", "BA1").unwrap();
    //
    // }
}
