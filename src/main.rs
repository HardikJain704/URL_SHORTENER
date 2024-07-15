use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use anyhow::Context;
mod routes;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect("postgres://postgres:password@localhost:5432/postgres")
        .await
        .context("could not connect to database_url")?;

    sqlx::migrate!().run(&db).await?;

    let app = routes::create_routes(db);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3008));
    println!("listening on {}", addr);
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use reqwest::Client;
    use url::Url;

    #[tokio::test]
    async fn test_create_short_url() {
        let client = Client::new();
        let res = client.post("http://localhost:3008/")
            .body("https://www.google.com")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .await
            .expect("failed to get response")
            .text()
            .await
            .expect("failed to get payload");


        assert!(res.starts_with("http://localhost:3008"));

    }

    #[tokio::test]
    async fn test_get_original_url() {
        let client = Client::new();
        let res = client.get("http://localhost:3008/kjDxx-16GK")
        .send()
        .await
        .expect("Failed to send request");


       println!("{:?}" , res.url().host());

       assert_eq!(res.status(), 200);
   
      assert_eq!(Url::parse(res.url().as_str()), Url::parse("https://www.google.com"));

   }
   
    #[tokio::test]
    async fn test_non_existent_short_url() {
        let client = Client::new();
        let res = client.get("http://localhost:3008/short/nonexistent")
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(res.status(), 404);
    }
   
}
