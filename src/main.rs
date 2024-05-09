mod ast;
mod interp;
mod lexer;
mod sexpr;

use std::{
    fs,
    io::{BufReader, Read},
};

use interp::interpret;

use crate::ast::AST;

type Result<T> = std::result::Result<T, ()>;

fn main() -> Result<()> {
    parse_file("./test.risp")?;
    Ok(())
}

fn parse_file(path: &str) -> Result<()> {
    let file = fs::File::open(path).map_err(|e| println!("{e}"))?;
    let mut reader = BufReader::new(file);
    let mut text = String::new();
    reader
        .read_to_string(&mut text)
        .map_err(|e| println!("{e}"))?;
    let tokens = lexer::tokenize(&text);
    let tree: AST<'_> = tokens.try_into().map_err(|e| println!("{e}"))?;
    interpret(tree);
    Ok(())
}
