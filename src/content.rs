/// Represents a content item that can either be a fixed
/// string value or a variable placeholder.
#[derive(Debug, PartialEq, Eq)]
enum ContentItem {
    Value(String),
    Variable(usize),
}

impl ContentItem {
    /// Generates the content based on the given arguments.
    ///
    /// If `self` is a `[ContentItem::Value]`, it returns a clone 
    /// of the contained string.
    /// If `self` is a `[ContentItem::Variable]`, it looks up 
    /// the corresponding argument in `args` and returns its value. 
    /// If the variable index is out of bounds, it returns 
    /// a formatted placeholder string.
    pub fn generate_content(&self, args: &Vec<String>) -> String {
        match self {
            ContentItem::Value(v) => v.clone(),
            ContentItem::Variable(v) => match args.get(*v) {
                Some(v) => v.clone(),
                None => format!("<{v}>"),
            },
        }
    }
}

impl ToString for ContentItem {
    /// Converts the `ContentItem` into a string representation.
    ///
    /// If `self` is a `Value`, it returns a clone of the contained string.
    /// If `self` is a `Variable`, it returns a formatted placeholder string.
    fn to_string(&self) -> String {
        match self {
            ContentItem::Value(v) => v.clone(),
            ContentItem::Variable(v) => format!("<{v}>"),
        }
    }
}

impl ToString for Content {
    /// Converts the `Content` into a single string by concatenating 
    /// its constituent items.
    fn to_string(&self) -> String {
        self.0.iter().map(|c| c.to_string()).collect()
    }
}

/// Represents a sequence of `ContentItem`s.
pub struct Content(Vec<ContentItem>);

impl From<&str> for Content {
    /// Parses a string and constructs a `Content` object.
    ///
    /// The input string is processed character by character, identifying fixed
    /// values and variable placeholders, and constructing the `Content` accordingly.
    fn from(value: &str) -> Self {
        let mut cont = Vec::new();
        let mut last = String::new();
        let mut variable = String::new();
        let mut read_variable = false;
        value.chars().for_each(|c| {
            if read_variable {
                if c.is_numeric() {
                    variable.push(c);
                } else if c == '>' {
                    if !variable.is_empty() {
                        cont.push(ContentItem::Value(last.clone()));
                        last.clear();
                        cont.push(ContentItem::Variable(variable.parse().unwrap()));
                        variable.clear();
                    } else {
                        last += "<>";
                    }
                    read_variable = false;
                } else {
                    last.push('<');
                    last += &variable;
                    last.push(c);
                    variable.clear();
                    read_variable = false;
                }
            } else {
                if c == '<' {
                    read_variable = true;
                } else {
                    last.push(c);
                }
            };
        });
        cont.push(ContentItem::Value(last));
        Content(cont)
    }
}

impl Content {
    /// Generates the content for this `Content` object based 
    /// on the provided arguments.
    ///
    /// It processes each `ContentItem` in the sequence and generates 
    /// the final content by replacing variable placeholders with their 
    /// corresponding values from `args`.
    ///
    /// # Examples
    ///
    /// ```
    /// use shortcut_autotyper::Content;
    /// let vec = vec![String::from("shortcut-autotyper"), String::from("X")];
    /// let content = Content::from("A <1> B");
    /// assert_eq!(&content.generate_content(&vec), "A X B");
    /// ```
    pub fn generate_content(&self, args: &Vec<String>) -> String {
        self.0.iter().map(|c| c.generate_content(args)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_paring() {
        let content = Content::from("A <1> B");
        assert_eq!(content.0[0], ContentItem::Value(String::from("A ")));
        assert_eq!(content.0[1], ContentItem::Variable(1));
        assert_eq!(content.0[2], ContentItem::Value(String::from(" B")));

        let content = Content::from("A <1> B <3><2>");
        assert_eq!(content.0[0], ContentItem::Value(String::from("A ")));
        assert_eq!(content.0[1], ContentItem::Variable(1));
        assert_eq!(content.0[2], ContentItem::Value(String::from(" B ")));
        assert_eq!(content.0[3], ContentItem::Variable(3));
        assert_eq!(content.0[4], ContentItem::Value(String::from("")));
        assert_eq!(content.0[5], ContentItem::Variable(2));

        let content = Content::from("A <1> B <C><2>");
        assert_eq!(content.0[0], ContentItem::Value(String::from("A ")));
        assert_eq!(content.0[1], ContentItem::Variable(1));
        assert_eq!(content.0[2], ContentItem::Value(String::from(" B <C>")));
        assert_eq!(content.0[3], ContentItem::Variable(2));

        let content = Content::from("A > <> B <C><2>");
        assert_eq!(
            content.0[0],
            ContentItem::Value(String::from("A > <> B <C>"))
        );
    }

    #[test]
    fn print_with_variables() {
        let vec = vec![
            String::from("filename"),
            String::from("X"),
            String::from("YY"),
            String::from("ZZZ"),
        ];

        let content = Content::from("A <1> B");
        assert_eq!(&content.generate_content(&vec), "A X B");

        let content = Content::from("A <8> B <2>");
        assert_eq!(&content.generate_content(&vec), "A <8> B YY");
    }
}
