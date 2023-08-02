use sqlx::postgres::{PgPoolOptions, PgPool, PgRow};
use sqlx::Row;
use handle_errors::Error;

use crate::types::{
    answer::{Answer, AnswerId},
    question::{Question, QuestionId}
};

#[derive(Debug, Clone)]
pub struct Store {
    pub connection: PgPool,
}

impl Store {
    pub async fn new(db_url: &str) -> Self {
		let db_pool = match PgPoolOptions::new()
			.max_connections(5)
			.connect(db_url).await {
				Ok(pool) => pool,
				Err(e) => panic!("Couldn't establick DB connection: {}", e),
			};
		Store {
			connection: db_pool,
		}
    }

	pub async fn get_questions(
		&self,
		limit: Option<i32>,
		offset: i32,
	) -> Result<Vec<Question>, sqlx::Error> {
		match sqlx::query("SELECT * from questions LIMIT $1 OFFSET $2")
			.bind(limit)
			.bind(offset)
			.map(|row: PgRow| Question {
				id: QuestionId(row.get("id")),
				title: row.get("title"),
				content: row.get("content"),
				tags: row.get("tags"),
				})
			.fetch_all(&self.connection)
			.await {
				Ok(questions) => Ok(questions),
				Err(e) => Err(e)
			}
	}
}
