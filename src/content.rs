

#[derive(Debug, PartialEq, Eq)]
enum ContentItem {
    Value(String),
    Variable(usize),
}

impl ContentItem {
    pub fn generate_content(&self, args: &Vec<String>) -> String {
        match self {
            ContentItem::Value(v) => v.clone(),
            ContentItem::Variable(v) => match args.get(*v) {
                Some(v) => v.clone(),
                None => format!("<{v}>"),
            }
        }
    }
}

impl ToString for ContentItem {
    fn to_string(&self) -> String {
        match self {
            ContentItem::Value(v) => v.clone(),
            ContentItem::Variable(v) => format!("<{v}>"),
        }
    }
}

impl ToString for Content {
    fn to_string(&self) -> String {
        self.0.iter().map(|c| c.to_string()).collect()
    }
}

pub struct Content(Vec<ContentItem>);

impl From<&str> for Content {
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
                    cont.push(ContentItem::Value(last.clone()));
                    last.clear();
                    cont.push(ContentItem::Variable(variable.parse().unwrap()));
                    variable.clear();
                    read_variable = false;
                } else {
                    last.push('<');
                    last += &variable;
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
    }
}
