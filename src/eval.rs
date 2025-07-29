use std::{
    collections::HashMap,
    fmt,
    ops::{self, ControlFlow},
};

use crate::{ast, relation::Relation};

#[derive(Debug)]
pub enum Error {
    ArityMismatch,
    DomainMismatch,
    UnknownLocal,
    UninitializedLocal,
    UnknownFunction { name: String },
    ProcedureDidNotReturn,
}

#[derive(Debug)]
pub struct Globals {
    functions: HashMap<String, Function>,
}

#[derive(Debug, Default)]
pub struct Locals {
    relations: HashMap<String, Option<Relation>>,
}

pub enum Function {
    BuiltIn(
        &'static str,
        Box<dyn Fn(Vec<Relation>) -> Result<Relation, Error>>,
    ),
    Custom(ast::Item),
}

impl Globals {
    fn builtins() -> Self {
        let mut globals = Self {
            functions: HashMap::new(),
        };
        globals.register_builtins();
        globals
    }

    fn register_builtins(&mut self) {
        self.register_builtin("TRUE", |[]| Ok(Relation::true_relation()));
        self.register_builtin("true", |[]| Ok(Relation::true_relation()));
        self.register_builtin("FALSE", |[]| Ok(Relation::false_relation()));
        self.register_builtin("false", |[]| Ok(Relation::false_relation()));
        self.register_builtin("L", |[r]| Ok(Relation::universal(r.domain())));
        self.register_builtin("O", |[r]| Ok(Relation::empty(r.domain())));
        self.register_builtin("I", |[r]| {
            let (x_domain, y_domain) = r.domain();
            if x_domain != y_domain {
                return Err(Error::DomainMismatch);
            }
            Ok(Relation::identity(x_domain))
        });

        self.register_builtin("eq", |[lhs, rhs]| {
            dbg!(&lhs, &rhs);
            if lhs.domain() != rhs.domain() {
                return Err(Error::DomainMismatch);
            }
            Ok(dbg!(Relation::from(lhs == rhs)))
        });
    }

    fn register_builtin<const N: usize>(
        &mut self,
        name: &'static str,
        f: impl Fn([Relation; N]) -> Result<Relation, Error> + 'static,
    ) {
        let function = Box::new(move |args: Vec<Relation>| {
            let args = args.try_into().map_err(|_| Error::ArityMismatch)?;
            f(args)
        });
        let inserted = self
            .functions
            .insert(name.to_owned(), Function::BuiltIn(name, function));
        assert!(
            inserted.is_none(),
            "function {name} already registered in globals"
        );
    }
}

impl Default for Globals {
    fn default() -> Self {
        Self::builtins()
    }
}

impl Extend<ast::Item> for Globals {
    fn extend<T: IntoIterator<Item = ast::Item>>(&mut self, iter: T) {
        self.functions.extend(
            iter.into_iter()
                .map(|item| (item.name().to_owned(), Function::Custom(item))),
        );
    }
}

impl Locals {
    pub fn assign(&mut self, name: &str, value: Relation) {
        self.relations.insert(name.to_owned(), Some(value));
    }
}

impl Function {
    fn call(&self, globals: &Globals, args: Vec<Relation>) -> Result<Relation, Error> {
        match self {
            Function::BuiltIn(_name, f) => f(args),
            Function::Custom(item) => match item {
                ast::Item::Procedure {
                    name: _,
                    params,
                    decls,
                    body,
                } => {
                    if params.len() != args.len() {
                        return Err(Error::ArityMismatch);
                    }
                    let mut locals = Locals::default();
                    for (param, arg) in params.iter().zip(args) {
                        locals.relations.insert((*param).to_owned(), Some(arg));
                    }
                    for decl in decls {
                        locals.relations.insert((*decl).to_owned(), None);
                    }

                    let res = eval_stmts(globals, &mut locals, body);
                    match res {
                        ControlFlow::Break(Ok(r)) => Ok(r),
                        ControlFlow::Break(Err(err)) => Err(err),
                        ControlFlow::Continue(()) => Err(Error::ProcedureDidNotReturn),
                    }
                }
                ast::Item::Function {
                    name: _,
                    params,
                    value,
                } => {
                    if params.len() != args.len() {
                        return Err(Error::ArityMismatch);
                    }
                    let mut locals = Locals::default();
                    for (param, arg) in params.iter().zip(args) {
                        locals.relations.insert((*param).to_owned(), Some(arg));
                    }

                    eval(globals, &locals, value)
                }
            },
        }
    }
}

fn eval_stmts(
    globals: &Globals,
    locals: &mut Locals,
    body: &[ast::Stmt],
) -> ops::ControlFlow<Result<Relation, Error>> {
    for stmt in body {
        eval_stmt(globals, locals, stmt)?;
    }
    ControlFlow::Continue(())
}

fn eval_stmt(
    globals: &Globals,
    locals: &mut Locals,
    stmt: &ast::Stmt,
) -> ops::ControlFlow<Result<Relation, Error>> {
    match stmt {
        ast::Stmt::Assign { lhs, rhs } => {
            if !locals.relations.contains_key(lhs) {
                return ops::ControlFlow::Break(Err(Error::UnknownLocal));
            }

            let value = match eval(globals, locals, rhs) {
                Ok(v) => v,
                Err(e) => return ops::ControlFlow::Break(Err(e)),
            };

            // We checked that lhs exists earlier, and variables never become undeclared
            let var = locals.relations.get_mut(lhs).unwrap();
            *var = Some(value);
            ops::ControlFlow::Continue(())
        }
        ast::Stmt::While { cond, body } => loop {
            dbg!(cond);
            let cond_value = match eval(globals, locals, cond) {
                Ok(v) => v,
                Err(e) => return ops::ControlFlow::Break(Err(e)),
            };
            dbg!(&cond_value);
            if cond_value.is_empty() {
                break ControlFlow::Continue(());
            }

            eval_stmts(globals, locals, body)?;
        },
        ast::Stmt::Return { value } => {
            let value = match eval(globals, locals, value) {
                Ok(v) => v,
                Err(e) => return ops::ControlFlow::Break(Err(e)),
            };
            ops::ControlFlow::Break(Ok(value))
        }
        ast::Stmt::If {
            cond,
            then_body,
            else_body,
        } => {
            let cond_value = match eval(globals, locals, cond) {
                Ok(v) => v,
                Err(e) => return ops::ControlFlow::Break(Err(e)),
            };
            if cond_value.is_empty() {
                eval_stmts(globals, locals, then_body)
            } else if let Some(else_body) = else_body {
                eval_stmts(globals, locals, else_body)
            } else {
                ControlFlow::Continue(())
            }
        }
    }
}

pub fn eval(globals: &Globals, locals: &Locals, expr: &ast::Expr) -> Result<Relation, Error> {
    match expr {
        ast::Expr::Ident { ident } => locals
            .relations
            .get(ident)
            .cloned()
            .ok_or(Error::UnknownLocal)?
            .ok_or(Error::UninitializedLocal),
        ast::Expr::Call { func, args } => {
            let func = globals
                .functions
                .get(func)
                .ok_or_else(|| Error::UnknownFunction {
                    name: func.to_owned(),
                })?;
            let args = args
                .into_iter()
                .map(|arg| eval(globals, locals, arg))
                .collect::<Result<_, _>>()?;
            func.call(globals, args)
        }
        ast::Expr::Negate { value } => eval(globals, locals, value).map(|r| -r),
        ast::Expr::BinExpr { left, op, right } => {
            let lhs = eval(globals, locals, left)?;
            let rhs = eval(globals, locals, right)?;
            match op {
                ast::BinOp::Union => {
                    if lhs.domain() != rhs.domain() {
                        return Err(Error::DomainMismatch);
                    }
                    Ok(lhs | rhs)
                }
                ast::BinOp::Intersect => {
                    if lhs.domain() != rhs.domain() {
                        return Err(Error::DomainMismatch);
                    }
                    Ok(lhs & rhs)
                }
                ast::BinOp::Compose => {
                    if lhs.domain().1 != rhs.domain().0 {
                        return Err(Error::DomainMismatch);
                    }
                    Ok(lhs * rhs)
                }
                ast::BinOp::Sum => todo!(),
            }
        }
        ast::Expr::Transpose { value } => eval(globals, locals, value).map(|r| r.converse()),
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Function::BuiltIn(name, _) => write!(f, "BuiltIn({name})"),
            Function::Custom(item) => write!(f, "Custom({item:?})"),
        }
    }
}
