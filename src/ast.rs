use std::collections::VecDeque;

use crate::{
    interp::{interpret_sexpr, ProgContext},
    lexer::{Symbol, Token, TokenType},
};

pub type Ident<'a> = &'a str;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Member<'a> {
    Ident(Ident<'a>),
    SExpr(SExpr<'a>),
    String(String),
    NumI(i32),
    Unit,
}

impl<'a> Member<'a> {
    pub fn resolve(self, context: &mut ProgContext<'a>) -> Member<'a> {
        let mut value = self;
        loop {
            match value {
                Member::Ident(i) => {
                    value = context.get_var(i);
                }
                Member::SExpr(s) => {
                    value = interpret_sexpr(s, context).unwrap();
                }
                _ => return value,
            }
        }
    }

    pub fn into_string_value(self, context: &mut ProgContext<'a>) -> String {
        match self {
            Self::Ident(s) => panic!("Ident {s} not resolved (into_string)"),
            Self::SExpr(s) => interpret_sexpr(s, context)
                .unwrap()
                .into_string_value(context),
            Self::String(s) => s.to_string(),
            Self::NumI(n) => n.to_string(),
            Self::Unit => "()".to_string(),
        }
    }

    pub fn into_inum_value(self, context: &mut ProgContext<'a>) -> i32 {
        match self {
            Self::Ident(s) => panic!("Ident {s} not resolved (into_string)"),
            Self::SExpr(s) => interpret_sexpr(s, context)
                .unwrap()
                .into_inum_value(context),
            Self::String(s) => match s.parse::<i32>() {
                Ok(n) => n,
                Err(_) => panic!("Cannot convert string {s} into i32"),
            },
            Self::NumI(n) => n,
            Self::Unit => 1,
        }
    }

    pub fn as_ident(self) -> Option<Ident<'a>> {
        match self {
            Self::Ident(i) => Some(i),
            _ => None,
        }
    }

    pub fn as_ident_list(self) -> Option<Vec<Ident<'a>>> {
        match self {
            Self::SExpr(s) => {
                if s.members.iter().all(|m| match m {
                    Self::Ident(_) => true,
                    _ => false,
                }) {
                    return Some(
                        s.members
                            .into_iter()
                            .map(|m| match m {
                                Self::Ident(i) => i,
                                _ => unreachable!(),
                            })
                            .collect(),
                    );
                }
            }
            _ => (),
        }
        None
    }

    pub fn as_sexpr(self) -> Option<SExpr<'a>> {
        match self {
            Self::SExpr(s) => Some(s),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SExpr<'a> {
    pub members: VecDeque<Member<'a>>,
}

impl<'a> SExpr<'a> {
    pub fn new(members: Vec<Member<'a>>) -> Self {
        Self {
            members: members.into(),
        }
    }
}

#[derive(Debug)]
pub struct AST<'a> {
    pub prog: Vec<SExpr<'a>>,
}

impl<'a> TryInto<AST<'a>> for Vec<Token<'a>> {
    type Error = &'static str;
    fn try_into(mut self) -> Result<AST<'a>, Self::Error> {
        let mut prog = Vec::new();
        self.reverse();
        while self.len() > 0 {
            let first = self.pop().ok_or("Empty input list")?;
            match first.ty {
                TokenType::Symbol(Symbol::LParen) => {
                    prog.push(take_until_match_parens(&mut self)?);
                }
                _ => return Err("Error parsing program into AST"),
            }
        }
        Ok(AST { prog })
    }
}

fn read_string<'a>(tokens: &mut Vec<Token<'a>>) -> Result<String, &'static str> {
    let mut st = String::new();
    loop {
        match tokens.pop() {
            Some(tok) => match tok.ty {
                TokenType::Symbol(Symbol::Quote) => return Ok(st),
                _ => st.push_str(tok.reclaim()),
            },
            None => return Err("Unterminated string literal"),
        }
    }
}

fn take_until_match_parens<'a>(tokens: &mut Vec<Token<'a>>) -> Result<SExpr<'a>, &'static str> {
    let mut base = SExpr::new(vec![]);
    loop {
        match tokens.pop() {
            Some(token) => match token.ty {
                TokenType::Symbol(sym) => match sym {
                    Symbol::RParen => return Ok(base),
                    Symbol::LParen => base
                        .members
                        .push_back(Member::SExpr(take_until_match_parens(tokens)?)),
                    Symbol::Quote => base.members.push_back(Member::String(read_string(tokens)?)),
                },
                _ => {
                    let reclaimed = token.reclaim();
                    if let Ok(num) = reclaimed.parse::<i32>() {
                        base.members.push_back(Member::NumI(num));
                    } else {
                        base.members.push_back(Member::Ident(reclaimed));
                    }
                }
            },
            None => return Ok(base),
        }
    }
}
