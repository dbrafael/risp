// use std::{collections::VecDeque, fmt::{Display, Formatter}, rc::Rc};

// use crate::{ast::{FunContext, VarContext}, interp::SexprResult};

// pub enum SexprType<'a> {
//     Callable(&'a str),
//     //       name     args
//     List,
// }

// pub struct Sexpr<'a> {
//     ty: SexprType<'a>,
//     nodes: VecDeque<SexprMember<'a>>,
//     vars: VecDeque<Box<VarContext<'a>>>,
//     funs: VecDeque<Box<FunContext<'a>>>,
// }

// impl <'a>Sexpr<'a> {
//     pub fn new(
//         nodes: Vec<SexprMember<'a>>,
//         vars: VecDeque<Box<VarContext<'a>>>,
//         funs: VecDeque<Box<FunContext<'a>>>
//     ) -> Self {
//         assert!(nodes.len() > 0, "Tried to build empty sexpr");
//         let mut nodes: VecDeque<SexprMember<'a>> = nodes.into();
//         let first = nodes.pop_front().unwrap();
//         let ty = match first {
//             SexprMember::Ident(ident) => SexprType::Callable(ident),
//             _ => {
//                 nodes.push_front(first);
//                 SexprType::List
//             },
//         };
//         let mut vars = vars.clone();
//         let mut funs = funs.clone();
//         vars.push_front(Box::new(Vec::new()));
//         funs.push_front(Box::new(Vec::new()));
//         Self {
//             ty,
//             nodes,
//             vars,
//             funs,
//         }
//     }

//     fn resolve_var(&self, ident: &'a str) -> SexprResult<'a> {
//         for context in self.vars.iter() {
//             if let Some(found) = context
//                 .iter()
//                 .find(|(vname, _)| {
//                     *vname == ident
//                 }) {
//                     return found.1.clone();
//                 }
//         }
//         panic!("Variable with name {ident} not found");
//     }


//     fn update_var(&mut self, ident: &'a str, value: SexprResult<'a>) {
//         let last = self.vars.len() - 1;
//         assert!(last > 0);
//         let local_vars = self
//             .vars
//             .get(last)
//             .unwrap();
//         let exists = local_vars
//             .iter()
//             .enumerate()
//             .find_map(|(i, (vname, _))|if *vname == ident { Some(i) } else { None });
//         match exists {
//             Some(i) => {
//                 let exists = local_vars.get_mut(i).unwrap();
//                 exists.1 = value;
//             },
//             None => {
//                 local_vars.push((ident, value));
//             }
//         }
//     }

//     fn update_fun(&mut self, ident: &'a str, args: Sexpr<'a>, cb: Sexpr<'a>) {
//         for fun in self.funs.iter_mut() {
//             if fun.0 == ident {
//                 fun.1 = args;
//                 fun.2 = cb;
//                 return;
//             }
//         }
//         self.funs.push((ident, args, cb));
//     }

//     fn with_args(&mut self, args: VecDeque<(&'a str, SexprResult<'a>)>) -> &mut Self {
//         for arg in args {
//             self.update_var(arg.0, arg.1);
//         }
//         self
//     }

//     fn call_fun(&self, (fname, fargs, fsex): (&'a str, Sexpr<'a>, Sexpr<'a>), args: VecDeque<SexprResult<'a>>) -> SexprResult<'a> {
//         assert!(fargs.nodes.len() == args.len(), "Function {fname} called with the wrong number of arguments");
//         let args = args
//             .iter()
//             .enumerate()
//             .map(|(i, value)| (fargs.nodes.get(i).unwrap(), value));
//         let fsex = fsex.with_args(args);
//         fsex.resolve()
//     }

//     pub fn resolve(&self) -> SexprResult<'a> {
//         self.assign_vars();
//         match self.ty {
//             SexprType::List => SexprResult::List(self
//                 .nodes
//                 .iter()
//                 .map(|node| match node {
//                     SexprMember::Sexpr(s) => s.resolve(),
//                     SexprMember::String(i) => SexprResult::String(i),
//                     SexprMember::Number(i) => SexprResult::Number(i),
//                     _ => unreachable!(),
//                 })
//             ),
//             SexprType::Callable(fun) => {
//                 match fun {
//                     "let" => {
//                         assert!(self.nodes.len() == 2, "let call only takes 2 arguments");
//                         let ident = self
//                             .nodes
//                             .pop_front()
//                             .expect("Expected ident to assign value to")
//                             .expect_ident();
//                         let value = self
//                             .nodes
//                             .pop_front()
//                             .expect("Expected value to assign to ident")
//                             .computed();
// 		        self.update_var(ident, value);
//                         value
//                     },
//                     "fun" => {
//                         assert!(self.nodes.len() == 3, "fun call only takes 3 arguments");
//                         let ident = self
//                             .nodes
//                             .pop_front()
//                             .expect("Expected ident to assign function to")
//                             .expect_ident();
//                         let args = self
//                             .nodes
//                             .pop_front()
//                             .expect("Expected arg list to be ident s-expr")
//                             .expect_arglist();
//                         let cb = self
//                             .nodes
//                             .pop_front()
//                             .expect("Expected sexprt to be last argument")
//                             .expect_sexpr();
// 		        self.update_fun(ident, args.clone(), cb.clone());
//                         SexprResult::Unit
//                     },
//                     fname => {
//                         match self.funs.iter().find(|f| f.0 == fname) {
//                             Some(fun) => {

//                                 let args = self.nodes.iter().map(|n| n.computed()).collect();
//                                 self.call_fun(fun.clone(), args)
//                             },
//                             None => panic!("Function {fname} not found"),
//                         }
//                     }
//                 }
//             } 
//         }
//     }
// }

// impl Display for Sexpr<'_> {
//     fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
//         write!(fmt, "(* {})", self
//             .0
//             .iter()
//             .map(|m| m.to_string())
//             .collect::<Vec<String>>()
//             .join(" ")
//         )
//     }
// }

// impl <'a>IntoIterator for Sexpr<'a> {
//     type Item = SexprMember<'a>;
//     type IntoIter = std::vec::IntoIter<Self::Item>;
//     fn into_iter(self) -> Self::IntoIter {
//         self.0.into_iter()
//     }
// }

// #[derive(Clone)]
// pub enum SexprMember<'a> {
//     Sexpr(Sexpr<'a>),
//     Ident(&'a str),
//     String(Vec<&'a str>),
//     Number(i32),
// }

// impl <'a> SexprMember<'a> {
//     fn expect_ident(&self) -> &'a str{
//         match self {
//             SexprMember::Ident(ident) => ident,
//             _ => panic!("Expected ident")
//         }
//     }

//     fn expect_sexpr(&self) -> &Sexpr<'a>{
//         match self {
//             SexprMember::Sexpr(s) => s,
//             _ => panic!("Expected ident")
//         }
//     }

//     fn expect_arglist(&self) -> &Sexpr<'a> {
//         match self {
//             SexprMember::Sexpr(s) => {
//                 if s.nodes.iter().all(|n| match n {
//                     SexprMember::Ident(_) => true,
//                     _ => false,
//                 }) {
//                     return s;
//                 }
//                 panic!("Not all args are idents")
//             },
//             _ => panic!("Expected arg list")
//         }
//     }

//     fn computed(&self) -> SexprResult<'a> {
//         match self {
//             SexprMember::Sexpr(s) => s.resolve(),
//             SexprMember::String(i) => SexprResult::String(i.join("")),
//             SexprMember::Number(i) => SexprResult::Number(*i),
//             _ => unreachable!(),
//         }
//     }
// }

// impl Display for SexprMember<'_> {
//     fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
//         write!(fmt, "{}", match self {
//             SexprMember::Sexpr(s) => s.to_string(),
//             SexprMember::Ident(t) => t.to_string(),
//             SexprMember::Number(n) => format!("n'{}", n.to_string()),
//             SexprMember::String(s) => format!("s'{}", s.join("")),
//         })
//     }
// }
