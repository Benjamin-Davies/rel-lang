use chumsky::prelude::*;

use crate::{
    Domain,
    parser::{Error, Span, handle_errors},
    relation::Relation,
};

pub fn parse_relation(filename: &str, src: &str) -> Result<(String, Relation), Error> {
    match relation().parse(src).into_result() {
        Ok((name, r)) => Ok((name, r)),
        Err(errs) => {
            handle_errors(
                filename,
                src,
                errs.into_iter().map(|e| e.map_token(|c| c.to_string())),
            );
            Err(Error)
        }
    }
}

pub(super) fn header<'src>()
-> impl Parser<'src, &'src str, (String, (Domain, Domain)), extra::Err<Rich<'src, char, Span>>> {
    text::ident()
        .then_ignore(just(" ("))
        .then(u32_digits())
        .then_ignore(just(", "))
        .then(u32_digits())
        .then_ignore(just(")\n"))
        .map(|((name, x), y): ((&str, _), _)| (name.to_owned(), (..x, ..y)))
}

fn relation<'src>()
-> impl Parser<'src, &'src str, (String, Relation), extra::Err<Rich<'src, char, Span>>> {
    let line = u32_digits()
        .then_ignore(just(" : "))
        .then(u32_digits().separated_by(just(", ")).collect::<Vec<_>>())
        .then_ignore(just('\n'));

    header()
        .then(line.repeated().collect::<Vec<_>>())
        .map(|((name, domain), lines)| {
            let relation = Relation::sparse(
                domain,
                lines
                    .into_iter()
                    .flat_map(|(x, ys)| ys.into_iter().map(move |y| (x - 1, y - 1))),
            );
            (name, relation)
        })
}

fn u32_digits<'src>() -> impl Parser<'src, &'src str, u32, extra::Err<Rich<'src, char, Span>>> {
    text::digits(10)
        .to_slice()
        .try_map(|s: &str, span| s.parse().map_err(|e| Rich::custom(span, e)))
}

#[cfg(test)]
mod tests {
    use crate::{parser::relation::parse_relation, relation::Relation};

    #[test]
    fn test_parse_r1() {
        let (name, relation) =
            parse_relation("R1.ascii", include_str!("../../examples/R1.ascii")).unwrap();

        assert_eq!(name, "R1");
        let expected = Relation::sparse((..5, ..5), [(0, 1), (1, 2), (2, 3), (3, 4), (3, 1)]);
        assert_eq!(relation, expected);
    }

    #[test]
    fn test_parse_r2() {
        let (name, relation) =
            parse_relation("R2.ascii", include_str!("../../examples/R2.ascii")).unwrap();

        assert_eq!(name, "R2");
        let expected = Relation::sparse(
            (..5, ..5),
            [
                (1, 0),
                (1, 1),
                (1, 2),
                (1, 3),
                (1, 4),
                (4, 0),
                (4, 1),
                (4, 2),
                (4, 3),
                (4, 4),
            ],
        );
        assert_eq!(relation, expected);
    }

    #[test]
    fn test_parse_r3() {
        let (name, relation) =
            parse_relation("R3.ascii", include_str!("../../examples/R3.ascii")).unwrap();

        assert_eq!(name, "R3");
        let expected = Relation::sparse(
            (..5, ..5),
            [
                (0, 1),
                (0, 2),
                (0, 3),
                (0, 4),
                (1, 2),
                (1, 3),
                (1, 4),
                (2, 3),
                (2, 4),
                (3, 4),
            ],
        );
        assert_eq!(relation, expected);
    }
}
