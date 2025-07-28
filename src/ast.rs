#[derive(Debug)]
pub struct Program<'src> {
    pub items: Vec<Item<'src>>,
}

#[derive(Debug)]
pub enum Item<'src> {
    Procedure {
        name: &'src str,
        params: Vec<&'src str>,
        decls: Vec<&'src str>,
        body: Vec<Stmt<'src>>,
    },
    Function {
        name: &'src str,
        params: Vec<&'src str>,
        value: Expr<'src>,
    },
}

#[derive(Debug)]
pub enum Stmt<'src> {
    Assign {
        lhs: &'src str,
        rhs: Expr<'src>,
    },
    While {
        cond: Expr<'src>,
        body: Vec<Stmt<'src>>,
    },
    Return {
        value: Expr<'src>,
    },
    If {
        cond: Expr<'src>,
        body: Vec<Stmt<'src>>,
        else_body: Option<Vec<Stmt<'src>>>,
    },
}

#[derive(Debug)]
pub enum Expr<'src> {
    Ident {
        ident: &'src str,
    },
    Call {
        func: &'src str,
        args: Vec<Expr<'src>>,
    },
    Negate {
        value: Box<Expr<'src>>,
    },
    BinExpr {
        left: Box<Expr<'src>>,
        op: BinOp,
        right: Box<Expr<'src>>,
    },
    Transpose {
        value: Box<Expr<'src>>,
    },
}

#[derive(Debug)]
pub enum BinOp {
    Union,
    Intersect,
    Compose,
    Sum,
}
