use nom::bytes::tag_no_case;
use nom::error::ErrorKind;
use nom::sequence::{separated_pair, terminated};
use nom::Parser;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, space1},
    combinator::{map, opt},
    multi::separated_list1,
    sequence::{delimited, preceded},
    IResult,
};
use validator::ValidationError;

use crate::feed::parsed_feed;

#[derive(PartialEq, Debug)]
enum SearchExpr {
    Phrase(String),
    Word(String),
    BinaryOp(BinaryOperator, Box<SearchExpr>, Box<SearchExpr>),
    Not(Box<SearchExpr>),
    Field(String, Box<SearchExpr>),
    IsRead,
    IsUnread,
    Group(Vec<SearchExpr>),
}
#[derive(PartialEq, Debug)]
enum BinaryOperator {
    And,
    Or,
}

#[derive(Debug)]
pub struct Query {
    feeds: Vec<SearchExpr>,
}

fn parse_word(input: &str) -> IResult<&str, SearchExpr> {
    let (input, word) = take_while1(|c: char| c.is_alphanumeric())(input)?;
    Ok((input, SearchExpr::Word(word.to_string())))
}

fn parse_phrase(input: &str) -> IResult<&str, SearchExpr> {
    let (input, phrase) =
        delimited(char('"'), take_while1(|c: char| c != '"'), char('"')).parse(input)?;
    Ok((input, SearchExpr::Phrase(phrase.to_string())))
}

fn parse_field(input: &str) -> IResult<&str, SearchExpr> {
    let (input, field) = separated_pair(
        take_while1(|c: char| c.is_alphanumeric()),
        char(':'),
        alt((parse_group, parse_word, parse_phrase)),
    )
    .parse(input)?;

    Ok((
        input,
        SearchExpr::Field(field.0.to_string(), Box::new(field.1)),
    ))
}

fn parse_is(input: &str) -> IResult<&str, SearchExpr> {
    let (input, (_, value)) =
        separated_pair(tag("is"), char(':'), alt((tag("read"), tag("unread")))).parse(input)?;

    match value {
        "read" => Ok((input, SearchExpr::IsRead)),
        "unread" => Ok((input, SearchExpr::IsUnread)),
        _ => unreachable!(),
    }
}

fn parse_group(input: &str) -> IResult<&str, SearchExpr> {
    let (input, exprs) = delimited(
        char('('),
        separated_list1(
            space1,
            alt((parse_group, parse_binary_op, parse_word, parse_phrase)),
        ),
        char(')'),
    )
    .parse(input)?;
    Ok((input, SearchExpr::Group(exprs)))
}

fn parse_binary_operator(input: &str) -> IResult<&str, BinaryOperator> {
    let (input, operator) = alt((tag("AND"), tag("OR"))).parse(input)?;
    let operator = match operator {
        "AND" => BinaryOperator::And,
        "OR" => BinaryOperator::Or,
        _ => unreachable!(),
    };

    Ok((input, operator))
}

fn parse_binary_op(input: &str) -> IResult<&str, SearchExpr> {
    let (input, (left, (operator, right))) = separated_pair(
        alt((parse_group, parse_is, parse_field, parse_word, parse_phrase)),
        space1,
        separated_pair(
            parse_binary_operator,
            space1,
            alt((parse_group, parse_is, parse_field, parse_word, parse_phrase)),
        ),
    )
    .parse(input)?;

    Ok((
        input,
        SearchExpr::BinaryOp(operator, Box::new(left), Box::new(right)),
    ))
}

fn parse_not_op(input: &str) -> IResult<&str, SearchExpr> {
    let (input, value) = preceded(
        terminated(tag("NOT"), space1),
        alt((parse_group, parse_field, parse_word, parse_phrase)),
    )
    .parse(input)?;

    Ok((input, SearchExpr::Not(Box::new(value))))
}

fn parse_expr(input: &str) -> IResult<&str, SearchExpr> {
    alt((
        parse_binary_op,
        parse_not_op,
        parse_group,
        parse_is,
        parse_field,
        parse_phrase,
        parse_word,
    ))
    .parse(input)
}

pub fn parse_query(input: &str) -> IResult<&str, Query> {
    let (input, exprs) = separated_list1(space1, parse_expr).parse(input)?;
    // Ensure that the entire input is consumed
    if !input.trim().is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            ErrorKind::Complete,
        )));
    }
    Ok((input, Query { feeds: exprs }))
}
mod tests {
    use super::*;

    #[test]
    fn test_parse_word() {
        let parsed = parse_word("hello world").unwrap();

        assert_eq!(parsed.0, " world");
    }
    #[test]
    fn test_parse_phrase() {
        let parsed = parse_phrase("\"hello world\"").unwrap();
        assert_eq!(parsed.1, SearchExpr::Phrase("hello world".to_string()));
        assert_eq!(parsed.0, "");
    }
    #[test]
    fn test_parse_is() {
        let parsed = parse_is("is:read").expect("This should not fail");
        assert_eq!(parsed.1, SearchExpr::IsRead);
        assert_eq!(parsed.0, "");

        let parsed = parse_is("is:unread").expect("This should not fail");
        assert_eq!(parsed.1, SearchExpr::IsUnread);
        assert_eq!(parsed.0, "");
    }

    #[test]
    fn test_parse_field() {
        let parsed = parse_field("title:hello").expect("This should not fail");
        assert_eq!(
            parsed.1,
            SearchExpr::Field(
                "title".to_string(),
                Box::new(SearchExpr::Word("hello".to_string()))
            )
        );

        let parsed = parse_field("title:(hello world)").expect("This should not fail");
        assert_eq!(
            parsed.1,
            SearchExpr::Field(
                "title".to_string(),
                Box::new(SearchExpr::Group(vec![
                    SearchExpr::Word("hello".to_string()),
                    SearchExpr::Word("world".to_string())
                ]))
            )
        );
    }

    #[test]
    fn test_parse_group() {
        let parsed = parse_group("(hello world)").expect("This should not fail");
        assert_eq!(
            parsed.1,
            SearchExpr::Group(vec![
                SearchExpr::Word("hello".to_string()),
                SearchExpr::Word("world".to_string())
            ])
        );
        assert_eq!(parsed.0, "");

        // handle
        let parsed = parse_group("(hello world feed:123)");
        assert!(parsed.is_err());

        // handle group of groups
        let parsed = parse_group("(hello (world))").expect("This should not fail");
        assert_eq!(
            parsed.1,
            SearchExpr::Group(vec![
                SearchExpr::Word("hello".to_string()),
                SearchExpr::Group(vec![SearchExpr::Word("world".to_string())])
            ])
        );
    }

    #[test]
    fn test_parse_not_op() {
        let parsed = parse_not_op("NOT hello").unwrap();
        assert_eq!(
            parsed.1,
            SearchExpr::Not(Box::new(SearchExpr::Word("hello".to_string())))
        );
        assert_eq!(parsed.0, "");

        let parsed = parse_not_op("NOT (hello world)").unwrap();
        assert_eq!(
            parsed.1,
            SearchExpr::Not(Box::new(SearchExpr::Group(vec![
                SearchExpr::Word("hello".to_string()),
                SearchExpr::Word("world".to_string())
            ])))
        );

        let parsed = parse_not_op("NOT title:hello").unwrap();
        assert_eq!(
            parsed.1,
            SearchExpr::Not(Box::new(SearchExpr::Field(
                "title".to_string(),
                Box::new(SearchExpr::Word("hello".to_string()))
            )))
        );
    }

    #[test]
    fn test_binary_op() {
        let parsed = parse_binary_op("hello AND world").unwrap();
        assert_eq!(
            parsed.1,
            SearchExpr::BinaryOp(
                BinaryOperator::And,
                Box::new(SearchExpr::Word("hello".to_string())),
                Box::new(SearchExpr::Word("world".to_string()))
            )
        );
        assert_eq!(parsed.0, "");

        let parsed = parse_binary_op("(hello OR world) OR (hello AND world)").unwrap();
        assert_eq!(
            parsed.1,
            SearchExpr::BinaryOp(
                BinaryOperator::Or,
                Box::new(SearchExpr::Group(vec![SearchExpr::BinaryOp(
                    BinaryOperator::Or,
                    Box::new(SearchExpr::Word("hello".to_string())),
                    Box::new(SearchExpr::Word("world".to_string()))
                )])),
                Box::new(SearchExpr::Group(vec![SearchExpr::BinaryOp(
                    BinaryOperator::And,
                    Box::new(SearchExpr::Word("hello".to_string())),
                    Box::new(SearchExpr::Word("world".to_string()))
                )]))
            )
        );
    }

    #[test]
    fn test_parsing_query() {
        let parsed = parse_query("hello world").unwrap();
        assert_eq!(
            parsed.1.feeds,
            vec![
                SearchExpr::Word("hello".to_string()),
                SearchExpr::Word("world".to_string())
            ]
        );

        let parsed = parse_query("hello world feed:123").unwrap();
        assert_eq!(
            parsed.1.feeds,
            vec![
                SearchExpr::Word("hello".to_string()),
                SearchExpr::Word("world".to_string()),
                SearchExpr::Field(
                    "feed".to_string(),
                    Box::new(SearchExpr::Word("123".to_string()))
                )
            ]
        );

        let parsed = parse_query("hello world feed:123 is:read").unwrap();
        assert_eq!(
            parsed.1.feeds,
            vec![
                SearchExpr::Word("hello".to_string()),
                SearchExpr::Word("world".to_string()),
                SearchExpr::Field(
                    "feed".to_string(),
                    Box::new(SearchExpr::Word("123".to_string()))
                ),
                SearchExpr::IsRead
            ]
        );

        let parsed = parse_query("hello world feed:123 is:read OR is:unread").unwrap();
        assert_eq!(
            parsed.1.feeds,
            vec![
                SearchExpr::Word("hello".to_string()),
                SearchExpr::Word("world".to_string()),
                SearchExpr::Field(
                    "feed".to_string(),
                    Box::new(SearchExpr::Word("123".to_string()))
                ),
                SearchExpr::BinaryOp(
                    BinaryOperator::Or,
                    Box::new(SearchExpr::IsRead),
                    Box::new(SearchExpr::IsUnread)
                )
            ]
        );

        let parsed =
            parse_query("\"hello world\" feed:123 OR feed:2456 is:read NOT title:hello").unwrap();
        assert_eq!(
            parsed.1.feeds,
            vec![
                SearchExpr::Phrase("hello world".to_string()),
                SearchExpr::BinaryOp(
                    BinaryOperator::Or,
                    Box::new(SearchExpr::Field(
                        "feed".to_string(),
                        Box::new(SearchExpr::Word("123".to_string()))
                    )),
                    Box::new(SearchExpr::Field(
                        "feed".to_string(),
                        Box::new(SearchExpr::Word("2456".to_string()))
                    ))
                ),
                SearchExpr::IsRead,
                SearchExpr::Not(Box::new(SearchExpr::Field(
                    "title".to_string(),
                    Box::new(SearchExpr::Word("hello".to_string()))
                )))
            ]
        );
    }
}

pub fn validate_query(query: &String) -> Result<(), ValidationError> {
    match parse_query(query) {
        Ok(_) => Ok(()),
        Err(_) => Err(ValidationError::new("Invalid query")),
    }
}
