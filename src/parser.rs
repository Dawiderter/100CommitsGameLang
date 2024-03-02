use pest::Parser;
use pest_derive::Parser;

#[derive(Debug, Parser)]
#[grammar = "grammar.pest"]
pub struct LangParser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_test() {
        let test_input = include_str!("./test.csv");

        let parsed = LangParser::parse(Rule::file, test_input).unwrap().next().unwrap();

        let mut field_sum = 0.0;
        let mut record_count = 0;

        for record in parsed.into_inner() {
            match record.as_rule() {
                Rule::record => {
                    record_count += 1;

                    for field in record.into_inner() {
                        field_sum += field.as_str().parse::<f64>().unwrap();
                    }
                },
                Rule::EOI => {},
                _ => unreachable!(),
            }
        }

        println!("Sum {}", field_sum);
        println!("Records {}", record_count);
    }
}
