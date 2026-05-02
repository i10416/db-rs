use crate::{
    Query, SelectStatement,
    parsing::lexer::{Lexer, Token},
};

#[derive(Debug)]
pub enum ParseError {
    IllegalStage(String),
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn from_chars_unchecked(chars: Vec<char>) -> Self {
        let mut lex = Lexer::new(chars);
        let tokens = lex.tokenize().expect("must be valid tokens");
        Self { tokens, pos: 0 }
    }
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }
    pub fn parse(&mut self) -> Result<Query, ParseError> {
        self.parse_statement()
    }
    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        let token = self.peek().clone();
        if token == expected {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::IllegalStage(format!(
                "expected {expected:?}, got {token:?}"
            )))
        }
    }

    fn parse_statement(&mut self) -> Result<Query, ParseError> {
        match self.peek() {
            Token::Select => self.parse_select(),
            _ => todo!(),
        }
    }
    fn parse_select(&mut self) -> Result<Query, ParseError> {
        let _ = self.expect(Token::Select)?;
        let _ = self.expect(Token::Asterisk)?;
        let _ = self.expect(Token::From)?;
        let table = self.parse_ident()?;
        Ok(Query::Select(SelectStatement {
            from_table: table.to_string(),
        }))
    }
    fn parse_ident(&mut self) -> Result<String, ParseError> {
        let tkn = self.peek().clone();
        match tkn {
            Token::Ident(id) => {
                let _ = self.advance();
                Ok(id.to_string())
            }
            _ => Err(ParseError::IllegalStage("not an identifier".to_string())),
        }
    }
    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> &Token {
        self.pos += 1;
        self.tokens.get(self.pos - 1).unwrap_or(&Token::Eof)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Query,
        parsing::{lexer::Lexer, parser::Parser},
    };

    #[test]
    fn parse_select_statement() {
        let q = "SELECT * FROM t1 WHERE TRUE";
        let mut lex = Lexer::new(q.chars().into_iter().collect());
        let q = lex.tokenize().expect("OK");
        let mut p = Parser::new(q);
        let result = p.parse().expect("OK");
        assert_eq!(
            result,
            Query::Select(crate::SelectStatement {
                from_table: "t1".to_string()
            })
        )
    }
}
