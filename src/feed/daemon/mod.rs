use sqlx::PgPool;

pub fn gather_out_of_date_feeds(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    Ok(())
}
