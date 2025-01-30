use crate::http::common::*;
use crate::{feed::parse::ParsedFeed, sql::FeedType};

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateFeedRequest {
    #[validate(url)]
    link: String,
    feed_type: FeedType,
}

pub async fn create_feed(
    State(state): State<ApiContext>,
    Json(body): Json<CreateFeedRequest>,
) -> Result<()> {
    body.validate()?;

    let feed_string = state
        .reqwest_client
        .get(&body.link)
        .send()
        .await?
        .text()
        .await?;

    let parsed_feed = match body.feed_type {
        FeedType::Atom => {
            ParsedFeed::try_from(atom_syndication::Feed::read_from(feed_string.as_bytes())?)?
        }
        FeedType::Rss => ParsedFeed::try_from(rss::Channel::read_from(feed_string.as_bytes())?)?,
        _ => unimplemented!(),
    };

    // store queries in the database

    todo!("Parse the feed string and save it to the database");
    Ok(())
}
