#![allow(unused)]
use crate::http::common::*;
use crate::sql::Feed;

pub async fn list_feeds(State(state): State<ApiContext>) -> Result<Json<Vec<Feed>>> {
    todo!()
}
