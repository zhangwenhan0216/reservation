use std::{path::Path, thread};

use sqlx::{migrate::Migrator, Connection, Executor};

pub struct TestDb {
  pub host: String,
  pub port: u16,
  pub username: String,
  pub password: String,
  pub database: String,
  pub migration_path: String,
}
impl TestDb {
  pub fn new(
    host: impl Into<String>,
    port: u16,
    username: impl Into<String>,
    password: impl Into<String>,
    migration_path: impl Into<String>,
  ) -> Self {
    let host = host.into();
    let username = username.into();
    let password = password.into();
    let migration_path = migration_path.into();
    let migration_path_cloned = migration_path.clone();

    let database = format!("test_{}", uuid::Uuid::new_v4());
    let database_cloned = database.clone();

    let test_db = Self {
      host,
      port,
      username,
      password,
      database,
      migration_path,
    };

    let server_url = test_db.server_url();
    let url = test_db.url();

    thread::spawn(move || {
      let rt = tokio::runtime::Runtime::new().unwrap();
      rt.block_on(async move {
        let mut conn = sqlx::PgConnection::connect(&server_url).await.unwrap();
        conn
          .execute(format!(r#"CREATE DATABASE "{}""#, database_cloned).as_str())
          .await
          .unwrap();

        let mut conn = sqlx::PgConnection::connect(&url).await.unwrap();
        let m = Migrator::new(Path::new(&migration_path_cloned))
          .await
          .unwrap();
        m.run(&mut conn).await.unwrap();
      });
    })
    .join()
    .expect("Failed to create test database");

    test_db
  }

  pub fn server_url(&self) -> String {
    if self.password.is_empty() {
      format!("postgres://{}@{}:{}", self.username, self.host, self.port)
    } else {
      format!(
        "postgres://{}:{}@{}:{}",
        self.username, self.password, self.host, self.port
      )
    }
  }
  pub fn url(&self) -> String {
    format!("{}/{}", self.server_url(), self.database)
  }

  pub async fn get_pool(&self) -> sqlx::Pool<sqlx::Postgres> {
    sqlx::Pool::<sqlx::Postgres>::connect(&self.url())
      .await
      .unwrap()
  }
}

impl Drop for TestDb {
  fn drop(&mut self) {
    let server_url = self.server_url();
    let db_name = self.database.clone();
    thread::spawn(move || {
      let rt = tokio::runtime::Runtime::new().unwrap();
      rt.block_on(async move {
        let mut conn = sqlx::PgConnection::connect(&server_url).await.unwrap();
        // Terminate all connections to the database
        conn
          .execute(
            format!(
              r#"SELECT pg_terminate_backend(pid)
               FROM pg_stat_activity
               WHERE pid <> pg_backend_pid()
                 AND datname = '{}';"#,
              db_name
            )
            .as_str(),
          )
          .await
          .unwrap();

        conn
          .execute(format!(r#"DROP DATABASE "{}""#, db_name).as_str())
          .await
          .unwrap();
      });
    })
    .join()
    .expect("Failed to drop test database");
  }
}

#[cfg(test)]
mod tests {
  use sqlx::Row;

  use super::*;

  #[tokio::test]
  async fn test_test_db_creation() {
    let test_db =
      TestDb::new("localhost", 5432, "postgres", "123456", "./migrations");
    let pool = test_db.get_pool().await;

    let id: i32 =
      sqlx::query("INSERT INTO test_table (name) VALUES ($1) RETURNING id")
        .bind("test_name")
        .fetch_one(&pool)
        .await
        .unwrap()
        .get("id");

    let (_res_id, name) = sqlx::query_as::<_, (i32, String)>(
      "SELECT id, name FROM test_table WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(name, "test_name");
  }
}
