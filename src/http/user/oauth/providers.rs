use crate::http::common::*;

pub async fn list_providers(State(state): State<ApiContext>) -> Result<Json<Vec<String>>> {
    let providers = state
        .config
        .oauth
        .keys()
        .map(|k| k.to_string())
        .collect::<Vec<_>>();
    Ok(Json(providers))
}
