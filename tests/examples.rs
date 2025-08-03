use rel_lang::repl::Repl;

macro_rules! test {
    ($name:ident, $input:literal, $expected:literal) => {
        #[test]
        fn $name() {
            let input: &str = $input;
            let expected: &str = $expected;

            let mut repl = Repl::new();
            let mut output = Vec::new();
            for line in input.lines() {
                repl.process_input(line, &mut output).unwrap();
            }

            pretty_assertions::assert_eq!(
                String::from_utf8(output).unwrap().trim(),
                expected.trim(),
            );
        }
    };
}

test!(
    test_example_rtc1,
    r#"
.load prog examples/Examples.prog
.load rel examples/R1.ascii
RTC1(R1)
"#,
    r#"
Program loaded successfully from 'examples/Examples.prog'
Relation 'R1' loaded successfully from 'examples/R1.ascii'
<expr> (5, 5)
1 : 1, 2, 3, 4, 5
2 : 2, 3, 4, 5
3 : 2, 3, 4, 5
4 : 2, 3, 4, 5
5 : 5
"#
);

test!(
    test_example_tc1,
    r#"
.load prog examples/Examples.prog
.load rel examples/R1.ascii
TC1(R1)
"#,
    r#"
Program loaded successfully from 'examples/Examples.prog'
Relation 'R1' loaded successfully from 'examples/R1.ascii'
<expr> (5, 5)
1 : 2, 3, 4, 5
2 : 2, 3, 4, 5
3 : 2, 3, 4, 5
4 : 2, 3, 4, 5
"#
);
test!(
    test_example_rtc2,
    r#"
.load prog examples/Examples.prog
.load rel examples/R1.ascii
RTC2(R1)
"#,
    r#"
Program loaded successfully from 'examples/Examples.prog'
Relation 'R1' loaded successfully from 'examples/R1.ascii'
<expr> (5, 5)
1 : 1, 2, 3, 4, 5
2 : 2, 3, 4, 5
3 : 2, 3, 4, 5
4 : 2, 3, 4, 5
5 : 5
"#
);

test!(
    test_example_rtc3,
    r#"
.load prog examples/Examples.prog
.load rel examples/R1.ascii
RTC3(R1)
"#,
    r#"
Program loaded successfully from 'examples/Examples.prog'
Relation 'R1' loaded successfully from 'examples/R1.ascii'
<expr> (5, 5)
1 : 1, 2, 3, 4, 5
2 : 2, 3, 4, 5
3 : 2, 3, 4, 5
4 : 2, 3, 4, 5
5 : 5
"#
);

test!(
    test_example_rtc4,
    r#"
.load prog examples/Examples.prog
.load rel examples/R1.ascii
RTC4(R1)
"#,
    r#"
Program loaded successfully from 'examples/Examples.prog'
Relation 'R1' loaded successfully from 'examples/R1.ascii'
<expr> (5, 5)
1 : 1, 2, 3, 4, 5
2 : 2, 3, 4, 5
3 : 2, 3, 4, 5
4 : 2, 3, 4, 5
5 : 5
"#
);

test!(
    test_example_rtc5,
    r#"
.load prog examples/Examples.prog
.load rel examples/R1.ascii
RTC5(R1)
"#,
    r#"
Program loaded successfully from 'examples/Examples.prog'
Relation 'R1' loaded successfully from 'examples/R1.ascii'
<expr> (5, 5)
1 : 1, 2, 3, 4, 5
2 : 2, 3, 4, 5
3 : 2, 3, 4, 5
4 : 2, 3, 4, 5
5 : 5
"#
);
