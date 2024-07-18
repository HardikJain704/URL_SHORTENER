use sqlx::{PgPool, Error};

#[derive(sqlx::FromRow)]
pub struct StoredURL {
    pub id: String,
    pub target_url: String,
}

pub async fn get_link_by_id(pool: &PgPool, id: &str) -> Result<StoredURL, Error> {
    sqlx::query_as::<_, StoredURL>(
        r#"
        SELECT id, target_url FROM links WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn insert_link(pool: &PgPool, id: &str, target_url: &str) -> Result<(), Error> {
    sqlx::query(
        r#"
        INSERT INTO links(id, target_url) VALUES ($1, $2)
        "#,
    )
    .bind(id)
    .bind(target_url)
    .execute(pool)
    .await?;

    Ok(())
}
