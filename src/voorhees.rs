use std::str::from_utf8;

#[derive(Debug, PartialEq)]
pub enum Value {
    Null,
    Boolean(bool),
    // Number(f32)
    Array(Vec<Value>),
}

#[derive(Debug, PartialEq)]
pub struct ParseError(String);

struct Lexer<'a> {
    s: &'a [u8],
    pos: usize
}

impl<'a> Lexer<'a> {
    fn new(s: &[u8]) -> Lexer {
        Lexer{s: s, pos: 0}
    }

    fn eof(&self) -> bool {
        self.pos >= self.s.len()
    }

    fn advance(&mut self) {
        if !self.eof() {
            self.pos += 1;
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        if self.eof() {
            None
        } else {
            Some(self.s[self.pos])
        }
    }

    fn take_byte(&mut self) -> Option<u8> {
        if self.eof() {
            None
        } else {
            let res = self.s[self.pos];
            self.advance();
            Some(res)
        }
    }

    fn take<T>(&mut self, pred: T) -> Option<u8> where T : FnOnce(u8) -> bool {
        if let Some(ch) = self.peek_byte() {
            if pred(ch) {
                return Some(ch)
            }
        }
        return None;
    }

    fn take_while<T>(&mut self, pred: T) -> &'a [u8] where T : Fn(u8) -> bool {
        let start_pos = self.pos;
        while let Some(ch) = self.peek_byte() {
            if pred(ch) {
                self.advance();
            } else {
                break;
            }
        }

        &self.s[start_pos..self.pos]
    }

    fn skip_whitespace(&mut self) {
        self.take_while(|ch| ch == ' ' as u8 || ch == '\t' as u8  || ch == '\r' as u8  || ch == '\n' as u8);
    }

    fn is_identifier_start(b: u8) -> bool {
        (b >= 'a' as u8 && b <= 'z' as u8) || (b >= 'A' as u8 && b <= 'Z' as u8) || b == '_' as u8
    }

    fn is_identifier_char(b: u8) -> bool {
        Self::is_identifier_start(b) || (b >= '0' as u8 && b <= '9' as u8)
    }

    fn token(&mut self) -> &'a [u8] {
        self.skip_whitespace();

        if self.eof() {
            return &[];
        }

        let next_char = |lexer: &mut Self| {
            let start_pos = lexer.pos;
            lexer.advance();
            &lexer.s[start_pos..lexer.pos]
        };

        let byte = self.peek_byte().unwrap();

        let result = match byte as char {
            '[' => next_char(self),
            ']' => next_char(self),
            ',' => next_char(self),
            ':' => next_char(self),
            '{' => next_char(self),
            '}' => next_char(self),
            _ if Self::is_identifier_start(byte) => {
                self.take_while(Self::is_identifier_char)
            },
            _ => {
                next_char(self)
            }
        };

        self.skip_whitespace();
        
        result
    }

    fn rest(&self) -> &'a [u8] {
        &self.s[self.pos..self.s.len()]
    }
}

pub fn parse(s: &str) -> Result<Value, ParseError> {
    let mut lexer = Lexer::new(s.as_bytes());
    let v = parse_(&mut lexer)?;

    lexer.skip_whitespace();

    if !lexer.eof() {
        Err(ParseError(
            "Extra goop at the end of the file: ".to_owned() + from_utf8(lexer.rest()).unwrap(),
        ))
    } else {
        Ok(v)
    }
}

const NULL_TOKEN: &'static [u8] = b"null";
const TRUE_TOKEN: &'static [u8] = b"true";
const FALSE_TOKEN: &'static [u8] = b"false";
const OPEN_BRACKET_TOKEN: &'static [u8] = b"[";
const CLOSE_BRACKET_TOKEN: &'static [u8] = b"]";
const OPEN_BRACE_TOKEN: &'static [u8] = b"{";
const CLOSE_BRACE_TOKEN: &'static [u8] = b"}";
const COLON_TOKEN: &'static [u8] = b":";
const COMMA_TOKEN: &'static [u8] = b",";

fn parse_(lexer: &mut Lexer) -> Result<Value, ParseError> {
    let token = lexer.token();
    println!("token '{}'", from_utf8(token).unwrap());

    if token.len() == 0 {
        Err(ParseError("Unexpected end of document".to_owned()))
    } else if token == NULL_TOKEN {
        Ok(Value::Null)
    } else if token == TRUE_TOKEN {
        Ok(Value::Boolean(true))
    } else if token == FALSE_TOKEN {
        Ok(Value::Boolean(false))
    } else if token == OPEN_BRACKET_TOKEN {
        let mut arr = Vec::new();
        loop {
            let val = parse_(lexer)?;
            arr.push(val);

            let next = lexer.token();
            if next == CLOSE_BRACKET_TOKEN{
                break;
            } else if next == COMMA_TOKEN {
                continue;
            } else if next.len() == 0 {
                return Err(ParseError("Unexpected end of document".to_owned()));
            } else {
                return Err(ParseError("Expected ',' or ']' but got '".to_owned() + from_utf8(next).unwrap() + "'"));
            }
        }

        Ok(Value::Array(arr))
    } else {
        Err(ParseError("Unknown token '".to_owned() + from_utf8(token).unwrap() + "'"))
    }
}

#[test]
fn prims() {
    assert_eq!(Ok(Value::Null), parse("null"));
    assert_eq!(Ok(Value::Boolean(true)), parse("true"));
    assert_eq!(Ok(Value::Boolean(false)), parse("false"));
}

#[test]
fn simple_array() {
    let expected = vec![Value::Boolean(true), Value::Boolean(false), Value::Null];
    assert_eq!(Ok(Value::Array(expected)), parse("[true,false,null]"));
}

#[test]
fn nested_array() {
    let expected = Value::Array(vec![
        Value::Boolean(true),
        Value::Array(vec![Value::Boolean(false), Value::Null]),
    ]);

    assert_eq!(Ok(expected), parse("[true,[false,null]]"));
}

#[test]
fn whitespace() {
    let expected = Value::Array(vec![Value::Boolean(true), Value::Boolean(false)]);

    assert_eq!(Ok(expected), parse(" [ true , false ] "));
}
