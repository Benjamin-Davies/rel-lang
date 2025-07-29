use chumsky::prelude::*;

use crate::{
    parser::{Error, Span, handle_errors, relation::header},
    relation::Relation,
};

pub fn parse_matrix(filename: &str, src: &str) -> Result<(String, Relation), Error> {
    match matrix().parse(src).into_result() {
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

fn matrix<'src>()
-> impl Parser<'src, &'src str, (String, Relation), extra::Err<Rich<'src, char, Span>>> {
    let c = select! { 'X' => true, ' ' => false };
    let line = just("|")
        .ignore_then(c.repeated().at_most(u32::MAX as usize).collect::<Vec<_>>())
        .then_ignore(just("|\n"));

    header()
        .then(
            line.repeated()
                .at_most(u32::MAX as usize)
                .collect::<Vec<_>>(),
        )
        .map(|((name, domain), lines)| {
            let relation = Relation::sparse(
                domain,
                lines.into_iter().enumerate().flat_map(|(i, cells)| {
                    cells
                        .into_iter()
                        .enumerate()
                        .filter(|&(_, cell)| cell)
                        .map(move |(j, _)| (i as u32, j as u32))
                }),
            );
            (name, relation)
        })
}

#[cfg(test)]
mod tests {
    use crate::{parser::matrix::parse_matrix, relation::Relation};

    #[test]
    fn test_parse_r1_matrix() {
        let (name, relation) =
            parse_matrix("R1.matrix", include_str!("../../examples/R1.matrix")).unwrap();

        assert_eq!(name, "R1");
        let expected = Relation::sparse((..5, ..5), [(0, 1), (1, 2), (2, 3), (3, 4), (3, 1)]);
        assert_eq!(relation, expected);
    }

    #[test]
    fn test_parse_r2_matrix() {
        let (name, relation) =
            parse_matrix("R2.matrix", include_str!("../../examples/R2.matrix")).unwrap();

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
    fn test_parse_r3_matrix() {
        let (name, relation) =
            parse_matrix("R3.matrix", include_str!("../../examples/R3.matrix")).unwrap();

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
