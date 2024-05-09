use std::{
    collections::{HashMap, VecDeque},
    process::exit,
};

use crate::ast::{Ident, Member, SExpr, AST};

#[derive(Debug, Clone)]
pub struct Fun<'a> {
    args: Vec<Ident<'a>>,
    expr: SExpr<'a>,
}

impl<'a> Fun<'a> {
    fn reg_args(&self, args: Vec<Member<'a>>, context: &mut ProgContext<'a>) -> Result<(), String> {
        if self.args.len() != args.len() {
            return Err("Arg number missmatch".to_string());
        };
        for (i, member) in args.into_iter().enumerate() {
            context.reg_var(self.args.get(i).unwrap(), member);
        }
        Ok(())
    }

    pub fn call(self, args: Vec<Member<'a>>, context: &mut ProgContext<'a>) -> OpResult<'a> {
        context.next_level();
        self.reg_args(args, context)?;
        let res = interpret_sexpr(self.expr, context);
        context.pop_level();
        res
    }

    fn get_and_call(
        ident: Ident<'a>,
        args: Vec<Member<'a>>,
        context: &mut ProgContext<'a>,
    ) -> OpResult<'a> {
        let fun = context
            .get_fun(ident)
            .ok_or(format!("Function {ident} not found"))?;
        let args = args.into_iter().map(|m| m.resolve(context)).collect();
        fun.call(args, context)
    }
}

type Bindings<'a, T> = HashMap<Ident<'a>, T>;

#[derive(Debug, Default)]
pub struct Scope<'a> {
    variables: Bindings<'a, Member<'a>>,
    functions: Bindings<'a, Fun<'a>>,
}

pub struct ProgContext<'a> {
    stack: Vec<Scope<'a>>,
}

impl<'a> ProgContext<'a> {
    pub fn default() -> Self {
        Self {
            stack: vec![Scope::default()],
        }
    }

    pub fn next_level(&mut self) {
        self.stack.push(Scope::default());
    }

    pub fn pop_level(&mut self) {
        self.stack.pop();
    }
    pub fn reg_fun(&mut self, ident: Ident<'a>, fun: Fun<'a>) {
        self.stack.last_mut().unwrap().functions.insert(ident, fun);
    }
    pub fn reg_var(&mut self, ident: Ident<'a>, value: Member<'a>) {
        self.stack
            .last_mut()
            .unwrap()
            .variables
            .insert(ident, value);
    }

    pub fn get_fun(&self, ident: Ident<'a>) -> Option<Fun<'a>> {
        let layer = self.stack.len();
        for i in (0..layer).rev() {
            let scope = self.stack.get(i).unwrap();
            match scope.functions.get(&ident).cloned() {
                Some(fun) => return Some(fun),
                None => continue,
            }
        }
        None
    }

    pub fn get_var(&self, ident: Ident<'a>) -> Member<'a> {
        let layer = self.stack.len();
        for i in (0..layer).rev() {
            let scope = self.stack.get(i).unwrap();
            match scope.variables.get(&ident) {
                Some(var) => return var.clone(),
                None => continue,
            }
        }
        panic!("Variable {ident} not found");
    }
}

pub fn interpret<'a>(tree: AST<'a>) {
    let mut context = ProgContext::default();
    for line in tree.prog {
        println!("Interpreting {line:?}");
        match interpret_sexpr(line, &mut context) {
            Ok(ans) => println!("{ans:?}"),
            Err(e) => {
                println!("{e}");
                exit(1);
            }
        };
    }
}

type OpResult<'a> = Result<Member<'a>, String>;

fn define_fun<'a>(mut p_args: VecDeque<Member<'a>>, context: &mut ProgContext<'a>) -> OpResult<'a> {
    let ident = p_args
        .pop_front()
        .ok_or("def needs 3 arguments (def {name} ({args...}) ({function}))".to_string())?
        .as_ident()
        .ok_or("def needs argument 1 to be ident".to_string())?;
    let args = p_args
        .pop_front()
        .ok_or("def needs 3 arguments (def {name} ({args...}) ({function}))".to_string())?
        .as_ident_list()
        .ok_or("def needs argument 2 to be ident list".to_string())?;
    let expr = p_args
        .pop_front()
        .ok_or("def needs 3 arguments (def {name} ({args...}) ({function}))".to_string())?
        .as_sexpr()
        .ok_or("def needs argument 3 to be SExpr".to_string())?;
    assert!(
        p_args.len() == 0,
        "def needs 3 arguments (def {{name}} ({{args...}}) ({{function}}))"
    );
    let fun = Fun { args, expr };
    context.reg_fun(&ident, fun);
    Ok(Member::Unit)
}

pub fn interpret_sexpr<'a>(
    mut sexpr: SExpr<'a>,
    mut context: &mut ProgContext<'a>,
) -> OpResult<'a> {
    let first = sexpr.members.pop_front().unwrap().as_ident();
    if first.is_some() {
        let fun_name = first.unwrap();
        // is a function call
        // context level isnt raised for let and def, they update the parent context
        // function.call raises its context
        let result = match fun_name {
            "def" => define_fun(sexpr.members, &mut context),
            "let" => {
                let ident = sexpr
                    .members
                    .pop_front()
                    .ok_or(format!("let requires 2 arguments"))?
                    .as_ident()
                    .ok_or(format!("let requires argument 1 to be ident"))?;
                let value = sexpr
                    .members
                    .pop_front()
                    .ok_or(format!("let requires 2 arguments"))?;
                if sexpr.members.len() != 0 {
                    return Err(format!("let requires 2 arguments"));
                }
                context.reg_var(ident, value.clone());
                Ok(value)
            }
            "+" => fun_std_sum(sexpr.members.into(), context),
            "*" => fun_std_mul(sexpr.members.into(), context),
            _ => Fun::get_and_call(fun_name, sexpr.members.into(), context),
        };
        return result;
    } else {
        // is a list
        context.next_level();
        context.pop_level();
        todo!();
    }
}

fn fun_std_sum<'a>(args: Vec<Member<'a>>, context: &mut ProgContext<'a>) -> OpResult<'a> {
    context.next_level();
    let mut args: Vec<Member<'a>> = args.into_iter().map(|arg| arg.resolve(context)).collect();
    println!("std::sum {args:?}");
    let mut res = args.pop().ok_or(format!("Sum called with no arguments"))?;
    for rhs in args {
        res = match res {
            Member::Ident(_) => panic!("Attempted to add to unresolved ident"),
            Member::String(s) => Member::String(s + &rhs.into_string_value(context)),
            Member::NumI(n) => Member::NumI(n + rhs.into_inum_value(context)),
            Member::Unit => res,
            Member::SExpr(_) => unreachable!(),
        };
    }
    context.pop_level();
    Ok(res)
}

#[allow(unused_variables)]
fn fun_std_mul<'a>(args: Vec<Member<'a>>, context: &mut ProgContext<'a>) -> OpResult<'a> {
    todo!()
}
