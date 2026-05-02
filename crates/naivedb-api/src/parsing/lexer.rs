#[derive(Debug, PartialEq)]
pub enum LexError {
    Unsupported,
    IllegalState(String),
}
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: Vec<char>) -> Self {
        Self { input, pos: 0 }
    }
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            if token == Token::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        Ok(tokens)
    }
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        let _ = self.read_whitespaces0();
        let ch = match self.peek() {
            Some(ch) => ch,
            None => return Ok(Token::Eof),
        };
        let token = match ch {
            '*' => self.read_astarisk()?,
            '=' => {
                self.advance();
                Token::Eq
            }
            c if c.is_ascii_alphabetic() => self.read_ident_or_keyword()?,
            _ => Err(LexError::Unsupported)?,
        };
        Ok(token)
    }

    fn read_ident_or_keyword(&mut self) -> Result<Token, LexError> {
        let mut buf = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() {
                buf.push(c);
                let _ = self.advance();
            } else {
                break;
            }
        }
        let tkn = match buf.to_uppercase().as_str() {
            "SELECT" => Token::Select,
            "WHERE" => Token::Where,
            "FROM" => Token::From,
            _ => Token::Ident(buf),
        };
        Ok(tkn)
    }

    fn read_astarisk(&mut self) -> Result<Token, LexError> {
        match self.peek() {
            Some('*') => {
                let _ = self.advance();
                Ok(Token::Asterisk)
            }
            Some(_) => Err(LexError::IllegalState("token is not *".into())),
            None => Err(LexError::IllegalState("eof".into())),
        }
    }

    fn read_whitespaces0(&mut self) -> usize {
        let mut ws = 0;
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                let _ = self.advance();
                ws += 1;
            } else {
                break;
            }
        }
        ws
    }

    // primitive operations
    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }
    fn advance(&mut self) -> Option<char> {
        let c = self.peek();
        self.pos += 1;
        c
    }
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Select,
    From,
    Where,
    Asterisk,
    Ident(String),
    LitString(String),
    // ops
    Eq,
    Eof,
}

#[cfg(test)]
mod tests {
    use crate::parsing::lexer::{Lexer, Token};

    #[test]
    fn lexing_simple_select_statement_step_by_step() {
        let q = "SELECT * FROM t1";
        let mut lex = Lexer::new(q.chars().into_iter().collect());
        let tkn = lex.next_token().expect("OK");
        assert_eq!(tkn, Token::Select);
        let tkn = lex.next_token().expect("OK");
        assert_eq!(tkn, Token::Asterisk);
        let tkn = lex.next_token().expect("OK");
        assert_eq!(tkn, Token::From);
        let tkn = lex.next_token().expect("OK");
        assert_eq!(tkn, Token::Ident("t1".into()));
    }
    #[test]
    fn lexing_simple_select_statement() {
        let q = "SELECT * FROM t1";
        let mut lex = Lexer::new(q.chars().into_iter().collect());
        let result = lex.tokenize();
        assert_eq!(
            result,
            Ok(vec![
                Token::Select,
                Token::Asterisk,
                Token::From,
                Token::Ident("t1".into()),
                Token::Eof
            ])
        )
    }
}
