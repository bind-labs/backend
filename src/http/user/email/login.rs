use crate::auth::{jwt::BindJwtToken, password::verify_password};
use crate::http::common::*;
use crate::sql::User;

#[derive(Deserialize)]
pub struct UserLoginRequest {
    email: Option<String>,
    username: Option<String>,
    password: String,
}

#[derive(Serialize)]
pub struct UserLoginResponse {
    token: String,
}

pub async fn login(
    State(state): State<ApiContext>,
    Json(info): Json<UserLoginRequest>,
) -> Result<Json<UserLoginResponse>> {
    let user = if let Some(email) = info.email {
        User::get_by_email(&state.pool, &email).await.ok()
    } else if let Some(username) = info.username {
        User::get_by_username(&state.pool, &username).await.ok()
    } else {
        return Err(Error::BadRequest("Email or username required".to_string()));
    };

    if let Some(user) = user
        && let Some(ref password_hash) = user.password_hash
        && verify_password(&info.password, password_hash).expect("Failed to verify password")
    {
        Ok(Json(UserLoginResponse {
            token: BindJwtToken::user_to_token(&user, &state.jwt_secret)
                .expect("Failed to create token"),
        }))
    } else {
        Err(Error::LoginFailed)
    }
}
