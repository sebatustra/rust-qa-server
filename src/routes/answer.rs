use std::collections::HashMap;
use warp::http::StatusCode;

use crate::store::Store;
use crate::types::{
    answer::{Answer, AnswerId},
    question::QuestionId
};

pub async fn add_answer(
    store: Store,
    params: HashMap<String, String>
) -> Result<impl warp::Reply, warp::Rejection> {
    let answer = Answer {
        id: AnswerId(store.answers.read().await.len() as i32),
        content: params.get("content").unwrap().to_string(),
        question_id: QuestionId(params.get("question_id").unwrap().parse::<i32>().unwrap()),
    };

    store.answers.write().await.insert(answer.id.clone(), answer);

    Ok(warp::reply::with_status("Answer added", StatusCode::OK))
}
