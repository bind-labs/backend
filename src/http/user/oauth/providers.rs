use crate::http::common::*;

/// List available OAuth providers
#[utoipa::path(
    get,
    path = "/providers",
    tag = "user:oauth",
    responses(
        (status = 200, description = "List of available OAuth providers", body = Vec<String>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_providers(State(state): State<ApiContext>) -> Result<Json<Vec<String>>> {
    let providers = state
        .oauth_clients
        .keys()
        .map(|k| k.to_string())
        .collect::<Vec<_>>();
    Ok(Json(providers))
}
