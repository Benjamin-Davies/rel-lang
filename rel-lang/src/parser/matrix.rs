use chumsky::prelude::*;

use crate::{
    parser::{Error, Span, handle_errors},
    relation::Relation,
};

pub fn parse_matrix(filename: &str, src: &str) -> Result<Relation, Error> {
    match matrix().parse(src).into_result() {
        Ok(r) => Ok(r),
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

fn matrix<'src>() -> impl Parser<'src, &'src str, Relation, extra::Err<Rich<'src, char, Span>>> {
    let c = select! { 'X' => true, ' ' => false };
    let header = just("+")
        .ignore_then(just("-").repeated().at_most(u32::MAX as usize).count())
        .then_ignore(just("+\n"));
    let line = just("|")
        .ignore_then(c.repeated().at_most(u32::MAX as usize).collect::<Vec<_>>())
        .then_ignore(just("|\n"));

    header
        .clone()
        .then(
            line.repeated()
                .at_most(u32::MAX as usize)
                .collect::<Vec<_>>(),
        )
        .then_ignore(header)
        .map(|(width, lines)| {
            let relation = Relation::sparse(
                (..width as u32, ..lines.len() as u32),
                lines.into_iter().enumerate().flat_map(|(i, cells)| {
                    cells
                        .into_iter()
                        .enumerate()
                        .filter(|&(_, cell)| cell)
                        .map(move |(j, _)| (i as u32, j as u32))
                }),
            );
            relation
        })
}

#[cfg(test)]
mod tests {
    use crate::{parser::matrix::parse_matrix, relation::Relation};

    #[test]
    fn test_parse_r1_matrix() {
        let relation = parse_matrix("R1.matrix", include_str!("../../examples/R1.matrix")).unwrap();

        let expected = Relation::sparse((..5, ..5), [(0, 1), (1, 2), (2, 3), (3, 4), (3, 1)]);
        assert_eq!(relation, expected);
    }

    #[test]
    fn test_parse_r2_matrix() {
        let relation = parse_matrix("R2.matrix", include_str!("../../examples/R2.matrix")).unwrap();

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
    fn test_parse_r3_matrix() {
        let relation = parse_matrix("R3.matrix", include_str!("../../examples/R3.matrix")).unwrap();

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
