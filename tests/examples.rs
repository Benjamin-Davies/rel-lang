use rel_lang::{
    eval::{Globals, Locals, eval},
    load_file,
    parser::parse_expr,
    relation::Relation,
};

#[test]
fn test_example_rtc1() {
    let mut globals = Globals::default();
    load_file("examples/Examples.prog", &mut globals).unwrap();

    let mut locals = Locals::default();
    let input = Relation::sparse((..5, ..5), [(0, 1), (1, 2), (2, 3), (3, 4), (3, 1)]);
    locals.assign("R", input);

    let expr = parse_expr("RTC1(R)").unwrap();
    let result = eval(&globals, &locals, &expr).unwrap();

    let expected = Relation::sparse(
        (..5, ..5),
        [
            // Row 0
            (0, 0),
            (0, 1),
            (0, 2),
            (0, 3),
            (0, 4),
            // Row 1
            (1, 1),
            (1, 2),
            (1, 3),
            (1, 4),
            // Row 2
            (2, 1),
            (2, 2),
            (2, 3),
            (2, 4),
            // Row 3
            (3, 1),
            (3, 2),
            (3, 3),
            (3, 4),
            // Row 4
            (4, 4),
        ],
    );
    assert_eq!(result, expected);
}

#[test]
fn test_example_tc1() {
    let mut globals = Globals::default();
    load_file("examples/Examples.prog", &mut globals).unwrap();

    let mut locals = Locals::default();
    let input = Relation::sparse((..5, ..5), [(0, 1), (1, 2), (2, 3), (3, 4), (3, 1)]);
    locals.assign("R", input);

    let expr = parse_expr("TC1(R)").unwrap();
    let result = eval(&globals, &locals, &expr).unwrap();

    let expected = Relation::sparse(
        (..5, ..5),
        [
            // Row 0
            (0, 1),
            (0, 2),
            (0, 3),
            (0, 4),
            // Row 1
            (1, 1),
            (1, 2),
            (1, 3),
            (1, 4),
            // Row 2
            (2, 1),
            (2, 2),
            (2, 3),
            (2, 4),
            // Row 3
            (3, 1),
            (3, 2),
            (3, 3),
            (3, 4),
            // Row 4
        ],
    );
    assert_eq!(result, expected);
}
