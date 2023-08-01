use std::collections::HashMap;
use tracing::{instrument, info};
use warp::http::StatusCode;
use handle_errors::Error;

use crate::store::Store;
use crate::types::pagination::extract_pagination;
use crate::types::question::{Question, QuestionId};

/// Route handler responsible of adding a new question to the store.
/// Takes as parameters the store and the question struct
pub async fn add_question(
    store: Store,
    question: Question
) -> Result<impl warp::Reply, warp::Rejection> {
    store.questions.write().await.insert(question.id.clone(), question);
    Ok(warp::reply::with_status(
        "Question added",
        StatusCode::OK
    ))
}

/// Route handler responsible of updating a question, based on its id.
/// Takes as parameters the question id, the store, and the (new) question struct.
pub async fn update_question(
    id: String,
    store: Store,
    question: Question
) -> Result<impl warp::Reply, warp::Rejection> {
    match store.questions.write().await.get_mut(&QuestionId(id)) {
        Some(q) => *q = question,
        None => return Err(Error::QuestionNotFound.into())
    }
    Ok(warp::reply::with_status(
        "Question modified",
        StatusCode::OK,
    ))
}

/// Route handler responsible of deleting a question, based on its id.
/// Takes as parameters the question id and the store.
pub async fn delete_question(
    id: String,
    store: Store
) -> Result<impl warp::Reply, warp::Rejection> {
    match store.questions.write().await.remove_entry(&QuestionId(id)) {
        Some((id, _)) => Ok(warp::reply::with_status(
            format!("Question with id {} removed", id.0),
            StatusCode::OK,
        )),
        None => Err(Error::QuestionNotFound.into())
    }
}

/// Route handler responsible of returing questions, based on the query params.
/// Takes as parameters the params (HashMap) and the store.
#[instrument]
pub async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
	info!("Querying questions");
    if !params.is_empty() {
        let pagination = extract_pagination(params)?;
		info!(pagination = true);
        if pagination.start > pagination.end {
            return Err(Error::StartLargerThanEnd.into())
        }
        let res: Vec<Question> = store.questions.read().await.values().cloned().collect();
        if pagination.end > res.len() {
            return Err(Error::OutOfBounds.into())
        }
        let res = &res[pagination.start..pagination.end];
        Ok(warp::reply::json(&res))
    } else {
		info!(pagination = false);
        let res: Vec<Question> = store.questions.read().await.values().cloned().collect();
        Ok(warp::reply::json(&res))
    }
}
