pub struct Lexer<'s> {
    character_stream: &'s [char],
}

impl<'s> Lexer<'s> {
    pub fn new(character_stream: &'s [char]) -> Self {
        Self { character_stream }
    }

    fn trim_whitespace(&mut self) {
        while self.character_stream.len() > 0 && self.character_stream[0].is_whitespace() {
            self.character_stream = &self.character_stream[1..];
        }
    }

    fn truncate(&mut self, n: usize) -> &'s [char] {
        let token = &self.character_stream[0..n];
        self.character_stream = &self.character_stream[n..];
        token
    }

    fn truncate_while<F: FnMut(&char) -> bool>(&mut self, mut predicate: F) -> &'s [char] {
        let mut n = 0;
        while self.character_stream.len() > n && predicate(&self.character_stream[n]) {
            n += 1;
        }
        self.truncate(n)
    }

    pub fn next_token(&mut self) -> Option<String> {
        self.trim_whitespace();
        if self.character_stream.len() == 0 {
            return None;
        }

        if self.character_stream[0].is_alphabetic() {
            return Some(
                self.truncate_while(|x| x.is_alphanumeric())
                    .iter()
                    .collect(),
            );
        }

        if self.character_stream[0].is_numeric() {
            return Some(self.truncate_while(|x| x.is_numeric()).iter().collect());
        }

        return Some(self.truncate(1).iter().collect());
    }
}

impl<'s> Iterator for Lexer<'s> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[cfg(test)]
mod tests {
    use super::Lexer;
    use crate::file_types::xml::read_xml_file;
    const FILE_PATH: &'static str = "files/tokenize.html";

    #[test]
    fn lexeme() {
        let content = read_xml_file(FILE_PATH).unwrap();

        let char_slice = content.chars().collect::<Vec<_>>();
        let mut lexer = Lexer::new(&char_slice);

        // LSERT
        assert_eq!(lexer.next(), Some(String::from("LSERT")));

        // Hello, 20
        assert_eq!(lexer.next(), Some(String::from("Hello")));
        assert_eq!(lexer.next(), Some(String::from(",")));
        assert_eq!(lexer.next(), Some(String::from("20")));

        // 20Hello
        assert_eq!(lexer.next(), Some(String::from("20")));
        assert_eq!(lexer.next(), Some(String::from("Hello")));

        // 20, 10
        assert_eq!(lexer.next(), Some(String::from("20")));
        assert_eq!(lexer.next(), Some(String::from(",")));
        assert_eq!(lexer.next(), Some(String::from("10")));

        // [20, 10, 30]
        assert_eq!(lexer.next(), Some(String::from("[")));
        assert_eq!(lexer.next(), Some(String::from("20")));
        assert_eq!(lexer.next(), Some(String::from(",")));
        assert_eq!(lexer.next(), Some(String::from("10")));
        assert_eq!(lexer.next(), Some(String::from(",")));
        assert_eq!(lexer.next(), Some(String::from("30")));
        assert_eq!(lexer.next(), Some(String::from("]")));

        // 2002, 20.10.20
        assert_eq!(lexer.next(), Some(String::from("2002")));
        assert_eq!(lexer.next(), Some(String::from(",")));
        assert_eq!(lexer.next(), Some(String::from("20")));
        assert_eq!(lexer.next(), Some(String::from(".")));
        assert_eq!(lexer.next(), Some(String::from("10")));
        assert_eq!(lexer.next(), Some(String::from(".")));
        assert_eq!(lexer.next(), Some(String::from("20")));

        // 10.2532
        assert_eq!(lexer.next(), Some(String::from("10")));
        assert_eq!(lexer.next(), Some(String::from(".")));
        assert_eq!(lexer.next(), Some(String::from("2532")));
    }
}
