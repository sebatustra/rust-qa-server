use std::collections::HashMap;
use tracing::{instrument, info, event, Level};
use warp::http::StatusCode;
use handle_errors::Error;

use crate::store::Store;
use crate::types::pagination::extract_pagination;
use crate::types::pagination::Pagination;
use crate::types::question::{Question, NewQuestion};

/// Route handler responsible of adding a new question to the store.
/// Takes as parameters the store and the question struct
pub async fn add_question(
    store: Store,
    new_question: NewQuestion
) -> Result<impl warp::Reply, warp::Rejection> {
    if let Err(_e) = store.add_question(new_question).await {
		return Err(warp::reject::custom(Error::DatabaseQueryError));
	}

	Ok(warp::reply::with_status("question added", StatusCode::OK))
}

/// Route handler responsible of updating a question, based on its id.
/// Takes as parameters the question id, the store, and the (new) question struct.
pub async fn update_question(
    id: i32,
    store: Store,
    question: Question
) -> Result<impl warp::Reply, warp::Rejection> {
    let res = match store.update_question(question, id).await {
		Ok(res) => res,
		Err(_e) => return
			Err(warp::reject::custom(Error::DatabaseQueryError)),
	};

	Ok(warp::reply::json(&res))
}

/// Route handler responsible of deleting a question, based on its id.
/// Takes as parameters the question id and the store.
pub async fn delete_question(
    id: i32,
    store: Store
) -> Result<impl warp::Reply, warp::Rejection> {
    if let Err(_e) = store.delete_question(id).await {
		return Err(warp::reject::custom(Error::DatabaseQueryError))
	}
	Ok(warp::reply::with_status(
		format!("Question {} deleted", id),
		StatusCode::OK)
	)
}

/// Route handler responsible of returing questions, based on the query params.
/// Takes as parameters the params (HashMap) and the store.
#[instrument]
pub async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
	event!(target: "practical_rust_book", Level::INFO, "querying questions");
 	let mut pagination = Pagination::default();

    if !params.is_empty() {
		event!(Level::INFO, pagination = true);
		pagination = extract_pagination(params)?;
	}
	info!(pagination = false);
	let res: Vec<Question> = match store
		.get_questions(pagination.limit, pagination.offset)
		.await {
			Ok(res) => res,
			Err(_e) => {
				return Err(warp::reject::custom(
					Error::DatabaseQueryError
				))
			},
	};
	Ok(warp::reply::json(&res))
}
