use core::fmt;
use std::fmt::{Display, Formatter};

const SYMBOLS: [char; 3] = ['(', ')', '"'];

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    LParen,
    RParen,
    Quote,
}

impl<'a> TryInto<Symbol> for &'a str {
    type Error = ();
    fn try_into(self) -> Result<Symbol, Self::Error> {
        Ok(match self {
            "(" => Symbol::LParen,
            ")" => Symbol::RParen,
            "\"" => Symbol::Quote,
            _ => return Err(()),
        })
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::LParen => "LPAR",
                Self::RParen => "RPAR",
                Self::Quote => "QUOT",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub enum TokenType {
    Ident,
    Symbol(Symbol),
}

#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub ty: TokenType,
    pub data: &'a str,
}

impl<'a> Token<'a> {
    pub fn reclaim(self) -> &'a str {
        self.data
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.ty {
                TokenType::Ident => format!("{}", self.data),
                TokenType::Symbol(_) => format!("SYM {}", self.data),
            }
        )
    }
}

impl<'a> Into<Token<'a>> for &'a str {
    fn into(self) -> Token<'a> {
        let ty = match TryInto::<Symbol>::try_into(self) {
            Ok(sym) => TokenType::Symbol(sym),
            Err(_) => TokenType::Ident,
        };
        Token { ty, data: self }
    }
}

pub fn tokenize<'a>(input: &'a str) -> Vec<Token<'a>> {
    input
        .split_whitespace()
        .flat_map(|toks| {
            // A whitespace split will yield something like "(add" or "10))("
            toks.split_inclusive(&SYMBOLS).flat_map(|some_toks| {
                // split inclusive leaves some idents with a ) after "foo)" so we have to split them
                if some_toks.len() > 1 {
                    let len = some_toks.len() - 1;
                    let tail = &some_toks[len..];
                    let rest = &some_toks[..len];
                    if SYMBOLS.contains(&tail.chars().last().unwrap()) {
                        return vec![rest.into(), tail.into()];
                    }
                }
                vec![some_toks.into()]
            })
        })
        .collect()
}
