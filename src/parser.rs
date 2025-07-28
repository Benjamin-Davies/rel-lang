use ariadne::{Color, Label, Report, ReportKind, sources};
use chumsky::{input::ValueInput, prelude::*};

use crate::{
    Span,
    ast::{self, BinOp, Expr},
    lexer::{Token, lexer},
};

pub fn parse<'src>(filename: &str, src: &'src str) -> ast::Program<'src> {
    let (tokens, errs) = lexer().parse(src).into_output_errors();

    let parse_errs = if let Some(tokens) = &tokens {
        let (ast, parse_errs) = program()
            .parse(
                tokens
                    .as_slice()
                    .map((src.len()..src.len()).into(), |(t, s)| (t, s)),
            )
            .into_output_errors();

        if let Some(ast) = ast.filter(|_| errs.is_empty() && parse_errs.is_empty()) {
            return ast;
        }

        parse_errs
    } else {
        Vec::new()
    };

    let filename = filename.to_owned();
    let src = src.to_owned();
    errs.into_iter()
        .map(|e| e.map_token(|c| c.to_string()))
        .chain(
            parse_errs
                .into_iter()
                .map(|e| e.map_token(|tok| tok.to_string())),
        )
        .for_each(|e| {
            Report::build(ReportKind::Error, (filename.clone(), e.span().into_range()))
                .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
                .with_message(e.to_string())
                .with_label(
                    Label::new((filename.clone(), e.span().into_range()))
                        .with_message(e.reason().to_string())
                        .with_color(Color::Red),
                )
                .with_labels(e.contexts().map(|(label, span)| {
                    Label::new((filename.clone(), span.into_range()))
                        .with_message(format!("while parsing this {label}"))
                        .with_color(Color::Yellow)
                }))
                .finish()
                .print(sources([(filename.clone(), src.clone())]))
                .unwrap()
        });
    panic!("Parsing failed with errors");
}

fn program<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, ast::Program<'src>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    let ident = select! { Token::Ident(ident) => ident };

    let params = ident
        .separated_by(just(Token::Ctrl(',')))
        .collect()
        .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')));

    let decls = just(Token::Decl).ignore_then(ident.separated_by(just(Token::Ctrl(','))).collect());

    let body = just(Token::Beg)
        .ignore_then(stmt().repeated().collect())
        .then_ignore(just(Token::End));

    let procedure = ident
        .then(params.clone())
        .then(decls)
        .then(body)
        .then_ignore(just(Token::Ctrl('.')))
        .map(|(((name, params), decls), body)| ast::Item::Procedure {
            name,
            params,
            decls,
            body,
        });

    let function = ident
        .then(params)
        .then_ignore(just(Token::Op('=')))
        .then(expr())
        .then_ignore(just(Token::Ctrl('.')))
        .map(|((name, params), value)| ast::Item::Function {
            name,
            params,
            value,
        });

    let item = procedure.or(function);

    item.repeated()
        .collect()
        .map(|items| ast::Program { items })
}

fn stmt<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, ast::Stmt<'src>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    recursive(|stmt| {
        let ident = select! { Token::Ident(ident) => ident };

        let assign = ident
            .then_ignore(just(Token::Op('=')))
            .then(expr())
            .map(|(lhs, rhs)| ast::Stmt::Assign { lhs, rhs });

        let while_stmt = just::<_, I, _>(Token::While)
            .ignore_then(expr())
            .then_ignore(just(Token::Do))
            .then(stmt.clone().repeated().collect())
            .then_ignore(just(Token::Od))
            .map(|(cond, body)| ast::Stmt::While { cond, body });

        let return_stmt = just(Token::Return)
            .ignore_then(expr())
            .map(|value| ast::Stmt::Return { value });

        let if_stmt = just(Token::If)
            .ignore_then(expr())
            .then_ignore(just(Token::Then))
            .then(stmt.clone().repeated().collect())
            .then(
                just(Token::Else)
                    .ignore_then(stmt.repeated().collect())
                    .or_not(),
            )
            .then_ignore(just(Token::Fi))
            .map(|((cond, body), else_body)| ast::Stmt::If {
                cond,
                body,
                else_body,
            });

        assign.or(while_stmt).or(return_stmt).or(if_stmt)
    })
}

fn expr<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expr<'src>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    recursive(|expr| {
        let ident = select! { Token::Ident(ident) => ident };
        let ident_expr = ident.map(|ident| Expr::Ident { ident });

        let call = ident
            .then(
                expr.clone()
                    .separated_by(just(Token::Ctrl(',')))
                    .collect()
                    .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')'))),
            )
            .map(|(func, args)| Expr::Call { func, args });

        let parens = expr
            .clone()
            .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')));

        let inner_expr = parens
            .or(call)
            .or(ident_expr)
            .then(just(Token::Op('^')).or_not())
            .map(|(value, transpose)| match transpose {
                Some(_) => Expr::Transpose {
                    value: Box::new(value),
                },
                None => value,
            });

        let negate = just(Token::Op('-'))
            .ignore_then(expr)
            .map(|value| Expr::Negate {
                value: Box::new(value),
            });

        let term = inner_expr.or(negate);

        let product = term
            .separated_by(just(Token::Op('*')))
            .at_least(1)
            .collect()
            .map(|terms: Vec<_>| {
                let mut terms = terms.into_iter();
                let first_term = terms.next().expect("At least one term in product");
                terms.into_iter().fold(first_term, |l, r| Expr::BinExpr {
                    left: Box::new(l),
                    op: BinOp::Compose,
                    right: Box::new(r),
                })
            });

        let outer_bin_op = select! {
            Token::Op('|') => BinOp::Union,
            Token::Op('&') => BinOp::Intersect,
            Token::Op('+') => BinOp::Sum,
        };
        let bin_expr = product
            .clone()
            .then(outer_bin_op)
            .then(product.clone())
            .map(|((left, op), right)| Expr::BinExpr {
                left: Box::new(left),
                op,
                right: Box::new(right),
            });

        bin_expr.or(product)
    })
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;

    #[test]
    fn test_parse_examples() {
        let ast = parse("Examples.prog", include_str!("../examples/Examples.prog"));

        assert_eq!(ast.items.len(), 19);
    }
}
