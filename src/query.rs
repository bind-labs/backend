use validator::ValidationError;

struct Query {
    original_query: String,
    feeds: Vec<String>,
}

pub fn validate_query(query: &String) -> Result<(), ValidationError> {
    Ok(())
}
