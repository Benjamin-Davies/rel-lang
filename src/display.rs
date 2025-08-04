use std::fmt;

use crate::relation::Relation;

pub struct DisplayRelation<'a> {
    name: &'a str,
    relation: &'a Relation,
}

pub struct DisplayMatrix<'a> {
    relation: &'a Relation,
}

impl Relation {
    pub fn display<'a>(&'a self, name: &'a str) -> DisplayRelation<'a> {
        DisplayRelation {
            name,
            relation: self,
        }
    }

    pub fn display_matrix<'a>(&'a self) -> DisplayMatrix<'a> {
        DisplayMatrix { relation: self }
    }
}

impl fmt::Display for DisplayRelation<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} ({}, {})",
            self.name,
            self.relation.domain().0.end,
            self.relation.domain().1.end
        )?;

        let mut last_x = None;
        // Entries are guaranteed to be in ascending order
        for (x, y) in self.relation.iter() {
            if Some(x) == last_x {
                write!(f, ", ")?;
            } else {
                write!(f, "\n{} : ", x + 1)?;
                last_x = Some(x);
            }
            write!(f, "{}", y + 1)?;
        }
        writeln!(f)?;

        Ok(())
    }
}

impl fmt::Display for DisplayMatrix<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "+")?;
        for _ in 0..self.relation.domain().1.end {
            write!(f, "-")?;
        }
        writeln!(f, "+")?;

        for x in 0..self.relation.domain().0.end {
            write!(f, "|")?;
            for y in 0..self.relation.domain().1.end {
                if self.relation.contains((x, y)) {
                    write!(f, "X")?;
                } else {
                    write!(f, " ")?;
                }
            }
            writeln!(f, "|")?;
        }

        write!(f, "+")?;
        for _ in 0..self.relation.domain().1.end {
            write!(f, "-")?;
        }
        writeln!(f, "+")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{matrix::parse_matrix, relation::parse_relation};

    #[test]
    fn test_display_r1() {
        let src = include_str!("../examples/R1.ascii");
        let (name, relation) = parse_relation("R1.ascii", src).unwrap();
        assert_eq!(relation.display(&name).to_string(), src);
    }

    #[test]
    fn test_display_r2() {
        let src = include_str!("../examples/R2.ascii");
        let (name, relation) = parse_relation("R2.ascii", src).unwrap();
        assert_eq!(relation.display(&name).to_string(), src);
    }

    #[test]
    fn test_display_r3() {
        let src = include_str!("../examples/R3.ascii");
        let (name, relation) = parse_relation("R3.ascii", src).unwrap();
        assert_eq!(relation.display(&name).to_string(), src);
    }

    #[test]
    fn test_display_matrix_r1() {
        let src = include_str!("../examples/R1.matrix");
        let relation = parse_matrix("R1.matrix", src).unwrap();
        assert_eq!(relation.display_matrix().to_string(), src);
    }

    #[test]
    fn test_display_matrix_r2() {
        let src = include_str!("../examples/R2.matrix");
        let relation = parse_matrix("R2.matrix", src).unwrap();
        assert_eq!(relation.display_matrix().to_string(), src);
    }

    #[test]
    fn test_display_matrix_r3() {
        let src = include_str!("../examples/R3.matrix");
        let relation = parse_matrix("R3.matrix", src).unwrap();
        assert_eq!(relation.display_matrix().to_string(), src);
    }
}
