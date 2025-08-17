#[derive(Debug)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug)]
pub enum Item {
    Procedure {
        name: String,
        params: Vec<String>,
        decls: Vec<String>,
        body: Vec<Stmt>,
    },
    Function {
        name: String,
        params: Vec<String>,
        value: Expr,
    },
}

#[derive(Debug)]
pub enum Stmt {
    Assign {
        lhs: String,
        rhs: Expr,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    Return {
        value: Expr,
    },
    If {
        cond: Expr,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
    },
}

#[derive(Debug)]
pub enum Expr {
    Ident {
        ident: String,
    },
    Call {
        func: String,
        args: Vec<Expr>,
    },
    Negate {
        value: Box<Expr>,
    },
    BinExpr {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    Transpose {
        value: Box<Expr>,
    },
}

#[derive(Debug)]
pub enum BinOp {
    Union,
    Intersect,
    Compose,
    Sum,
}

impl Item {
    pub fn name(&self) -> &str {
        match self {
            Item::Procedure { name, .. } => name,
            Item::Function { name, .. } => name,
        }
    }
}
