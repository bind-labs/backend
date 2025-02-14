use axum::extract::State;
use axum::Json;

use crate::http::common::{ApiContext, Result};

pub async fn list_providers(State(state): State<ApiContext>) -> Result<Json<Vec<String>>> {
    let providers = state
        .config
        .oauth
        .keys()
        .map(|k| k.to_string())
        .collect::<Vec<_>>();
    Ok(Json(providers))
}
