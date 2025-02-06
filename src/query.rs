use nom::error::ErrorKind;
use nom::sequence::{separated_pair, terminated};
use nom::Parser;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, space1},
    multi::separated_list1,
    sequence::{delimited, preceded},
    IResult,
};
use validator::ValidationError;

#[derive(PartialEq, Debug, Clone)]
pub enum SearchExpr {
    Phrase(String),
    Word(String),
    BinaryOp(BinaryOperator, Box<SearchExpr>, Box<SearchExpr>),
    Not(Box<SearchExpr>),
    Field(String, Box<SearchExpr>),
    IsRead,
    IsUnread,
    Group(Vec<SearchExpr>),
}
#[derive(PartialEq, Debug, Clone)]
pub enum BinaryOperator {
    And,
    Or,
}

#[derive(Debug)]
pub struct Query {
    pub exprs: Vec<SearchExpr>,
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
        parse_group,
        parse_binary_op,
        parse_not_op,
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
    Ok((input, Query { exprs }))
}

pub fn validate_query(query: &str) -> Result<(), ValidationError> {
    match parse_query(query) {
        Ok(_) => Ok(()),
        Err(_) => Err(ValidationError::new("Invalid query")),
    }
}

fn expr_to_sql(expr: &SearchExpr, params: &mut Vec<String>) -> (String, Vec<String>) {
    match expr {
        SearchExpr::Word(word) => {
            params.push(format!("%{}%", word)); // Add parameter
            ("content ILIKE ?".to_string(), params.clone()) // Use ? as a placeholder
        }
        SearchExpr::Phrase(phrase) => {
            params.push(phrase.clone()); // Add parameter
            ("content = ?".to_string(), params.clone()) // Use ? as a placeholder
        }
        SearchExpr::BinaryOp(op, left, right) => {
            let (left_sql, left_params) = expr_to_sql(left, params);
            let (right_sql, right_params) = expr_to_sql(right, &mut left_params.clone());
            let operator = match op {
                BinaryOperator::And => "AND",
                BinaryOperator::Or => "OR",
            };
            (
                format!("({} {} {})", left_sql, operator, right_sql),
                right_params,
            )
        }
        SearchExpr::Not(expr) => {
            let (expr_sql, expr_params) = expr_to_sql(expr, params);
            (format!("NOT ({})", expr_sql), expr_params)
        }
        SearchExpr::Field(field, expr) => match expr.as_ref() {
            SearchExpr::Word(word) | SearchExpr::Phrase(word) => {
                params.push(word.clone());
                (format!("{} = ?", field), params.clone())
            }
            SearchExpr::Group(exprs) => {
                let mut group_params = params.clone();
                let conditions: Vec<String> = exprs
                    .iter()
                    .map(|expr| {
                        let (sql, new_params) = expr_to_sql(
                            &SearchExpr::Field(field.clone(), Box::new(expr.clone())),
                            &mut group_params,
                        );
                        group_params = new_params;
                        sql
                    })
                    .collect();
                (format!("({})", conditions.join(" AND ")), group_params)
            }
            SearchExpr::BinaryOp(op, expr_a, expr_b) if op == &BinaryOperator::Or => {
                let (expr_a_sql, expr_a_params) =
                    expr_to_sql(&SearchExpr::Field(field.clone(), expr_a.clone()), params);
                let (expr_b_sql, expr_b_params) = expr_to_sql(
                    &SearchExpr::Field(field.clone(), expr_b.clone()),
                    &mut expr_a_params.clone(),
                );
                (format!("{} OR {}", expr_a_sql, expr_b_sql), expr_b_params)
            }
            _ => unreachable!("Fields can only have groups, words or phrases"),
        },
        SearchExpr::IsRead => {
            unimplemented!()
        }
        SearchExpr::IsUnread => {
            unimplemented!()
        }
        SearchExpr::Group(exprs) => {
            let mut group_params = params.clone();
            let conditions: Vec<String> = exprs
                .iter()
                .map(|expr| {
                    let (sql, new_params) = expr_to_sql(expr, &mut group_params);
                    group_params = new_params;
                    sql
                })
                .collect();
            (format!("({})", conditions.join(" AND ")), group_params)
        }
    }
}

impl Query {
    pub fn to_sql(&self) -> (String, Vec<String>) {
        let mut params = Vec::new();
        let conditions: Vec<String> = self
            .exprs
            .iter()
            .map(|expr| {
                let (sql, new_params) = expr_to_sql(expr, &mut params);
                params = new_params;
                sql
            })
            .collect();

        (conditions.join(" AND "), params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_word() {
        let parsed = parse_word("hello world").unwrap();
        assert_eq!(parsed.1, SearchExpr::Word("hello".to_string()));
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
            parsed.1.exprs,
            vec![
                SearchExpr::Word("hello".to_string()),
                SearchExpr::Word("world".to_string())
            ]
        );

        let parsed = parse_query("hello world feed:123").unwrap();
        assert_eq!(
            parsed.1.exprs,
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
            parsed.1.exprs,
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
            parsed.1.exprs,
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
            parsed.1.exprs,
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

    #[test]
    fn to_sql_query() {
        let query = "\"hello world\" NOT title:hello".to_string();
        let (_, parsed) = parse_query(&query).unwrap();
        let (sql_query, values) = parsed.to_sql();
        assert_eq!(sql_query, "content = ? AND NOT (title = ?)");
        assert_eq!(values, vec!["hello world", "hello"]);

        let query = "hello world feed:123 OR feed:2456 NOT title:hello".to_string();
        let (_, parsed) = parse_query(&query).unwrap();
        let (sql_query, values) = parsed.to_sql();
        assert_eq!(
            sql_query,
            "content ILIKE ? AND content ILIKE ? AND (feed = ? OR feed = ?) AND NOT (title = ?)"
        );
        assert_eq!(values, vec!["%hello%", "%world%", "123", "2456", "hello"]);

        let query = "title:(hello world) type:(video OR podcasts)".to_string();
        let (_, parsed) = parse_query(&query).unwrap();
        let (sql_query, values) = parsed.to_sql();
        assert_eq!(
            sql_query,
            "(title = ? AND title = ?) AND (type = ? OR type = ?)"
        );
        assert_eq!(values, vec!["hello", "world", "video", "podcasts"]);
    }
}
