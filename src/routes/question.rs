use std::collections::HashMap;
use warp::http::StatusCode;

use crate::store::Store;
use crate::types::pagination::extract_pagination;
use crate::types::question::{Question, QuestionId};
use crate::error;


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

pub async fn update_question(
    id: String,
    store: Store,
    question: Question
) -> Result<impl warp::Reply, warp::Rejection> {
    match store.questions.write().await.get_mut(&QuestionId(id)) {
        Some(q) => *q = question,
        None => return Err(error::Error::QuestionNotFound.into())
    }
    Ok(warp::reply::with_status(
        "Question modified",
        StatusCode::OK,
    ))
}

pub async fn delete_question(
    id: String,
    store: Store
) -> Result<impl warp::Reply, warp::Rejection> {
    match store.questions.write().await.remove_entry(&QuestionId(id)) {
        Some((id, _)) => return Ok(warp::reply::with_status(
            format!("Question with id {} removed", id.0),
            StatusCode::OK,
        )),
        None => return Err(error::Error::QuestionNotFound.into())
    }
}

pub async fn get_questions(
    params: HashMap<String,
    String>,store: Store
) -> Result<impl warp::Reply, warp::Rejection> {
    if !params.is_empty() {
        let pagination = extract_pagination(params)?;
        if pagination.start > pagination.end {
            return Err(error::Error::StartLargerThanEnd.into())
        }
        let res: Vec<Question> = store.questions.read().await.values().cloned().collect();
        if pagination.end > res.len() {
            return Err(error::Error::OutOfBounds.into())
        }
        let res = &res[pagination.start..pagination.end];
        Ok(warp::reply::json(&res))
    } else {
        let res: Vec<Question> = store.questions.read().await.values().cloned().collect();
        Ok(warp::reply::json(&res))
    }
}
