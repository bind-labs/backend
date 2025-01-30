use nom::sequence::separated_pair;
use nom::Parser;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, space0},
    combinator::{map, opt},
    multi::separated_list1,
    sequence::{delimited, preceded},
    IResult,
};
use tracing_subscriber::field::delimited;
use validator::ValidationError;

#[derive(PartialEq, Debug)]
enum SearchExpr {
    Phrase(String),
    Word(String),
    Exact(String),
    BinaryOp(BinaryOperator, Box<SearchExpr>, Box<SearchExpr>),
    Not(Box<SearchExpr>),
    Field(String, Box<SearchExpr>),
    FeedId(String),
    TagId(String),
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
struct Query {
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
        alt((parse_word, parse_phrase)),
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

fn parse_feed_id(input: &str) -> IResult<&str, SearchExpr> {
    let (input, feed_id) =
        preceded(tag("feed:"), take_while1(|c: char| c.is_numeric())).parse(input)?;
    Ok((input, SearchExpr::FeedId(feed_id.to_string())))
}

fn parse_tag_id(input: &str) -> IResult<&str, SearchExpr> {
    let (input, tag_id) =
        preceded(tag("tag:"), take_while1(|c: char| c.is_numeric())).parse(input)?;
    Ok((input, SearchExpr::TagId(tag_id.to_string())))
}



fn parse_query(input: &str) -> IResult<&str, Query> {
    let (input, exprs) = separated_list1(
        char(' '),
        alt((
            parse_is,
            parse_feed_id,
            parse_tag_id,
            parse_field,
            parse_word,
            parse_phrase,
        )),
    ).parse(input)?;
    Ok((input, Query { feeds: exprs }))
}


mod tests {
    use super::*;

    #[test]
    fn test_parse_word() {
        let parsed = parse_word("hello world").expect("This should not fail");
        assert_eq!(parsed.1, SearchExpr::Word("hello".to_string()));
        assert_eq!(parsed.0, " world");
    }
    #[test]
    fn test_parse_phrase() {
        let parsed = parse_phrase("\"hello world\"").expect("This should not fail");
        assert_eq!(parsed.1, SearchExpr::Phrase("hello world".to_string()));
        assert_eq!(parsed.0, "");
    }

    #[test]
    fn test_parse_field() {
        let parsed = parse_field("title:hello world").expect("This should not fail");
        assert_eq!(
            parsed.1,
            SearchExpr::Field(
                "title".to_string(),
                Box::new(SearchExpr::Word("hello".to_string()))
            )
        );
        assert_eq!(parsed.0, " world");
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
    fn test_parse_feed_id() {
        let parsed = parse_feed_id("feed:123").expect("This should not fail");
        assert_eq!(parsed.1, SearchExpr::FeedId("123".to_string()));
        assert_eq!(parsed.0, "");
    }

    #[test]
    fn test_parse_tag_id() {
        let parsed = parse_tag_id("tag:123").expect("This should not fail");
        assert_eq!(parsed.1, SearchExpr::TagId("123".to_string()));
        assert_eq!(parsed.0, "");
    }

    #[test]
    fn test_parsing_query() {
        let parsed = parse_query("\"hello world\" link:yellow testing feed:123 is:unread").expect("This should not fail");
        assert_eq!(parsed.1.feeds.len(), 5);
    }
}

pub fn validate_query(query: &String) -> Result<(), ValidationError> {
    Ok(())
}
