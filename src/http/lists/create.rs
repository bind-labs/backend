use crate::http::auth::AuthUser;
use crate::http::common::*;
use crate::sql::{Icon, UserList};

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateListRequest {
    pub title: String,
    pub description: Option<String>,
    pub icon: Icon,
}

pub async fn create_list(
    user: AuthUser,
    State(state): State<ApiContext>,
    Json(body): Json<CreateListRequest>,
) -> Result<Json<UserList>> {
    body.validate()?;

    let query = sqlx::query_as!(
        UserList,
        r#"
        INSERT INTO user_list (owner, title, description, icon)
        VALUES ($1, $2, $3, $4)
        RETURNING 
            id,
            owner,
            title,
            description,
            icon as "icon:Icon",
            created_at,
            updated_at
        "#,
        user.id,
        body.title,
        body.description,
        body.icon as _
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(query))
}
