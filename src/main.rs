use serde::{
    Serialize,
    Deserialize
};
use std::collections::HashMap;
use warp::{
    Filter,
    Rejection,
    Reply,
    reject::Reject,
    http::StatusCode,
    http::Method,
    filters::{cors::CorsForbidden, body::BodyDeserializeError,},
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct Store {
    questions: Arc<RwLock<HashMap<QuestionId, Question>>>,
    answers: Arc<RwLock<HashMap<AnswerId, Answer>>>,
}

impl Store {
    fn new() -> Self {
        Store {
            questions: Arc::new(RwLock::new(Self::init())),
            answers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../questions.json");
        serde_json::from_str(file).expect("can't read questions.json")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash, Deserialize)]
struct QuestionId(String);

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash, Deserialize)]
struct AnswerId(String);

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Answer {
    id: AnswerId,
    content: String,
    question_id: QuestionId
}

#[derive(Debug)]
struct Pagination {
    start: usize,
    end: usize,
}

#[derive(Debug)]
enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    OutOfBounds,
    StartLargerThanEnd,
    QuestionNotFound
}

impl Reject for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ParseError(err) => {
                write!(f, "cannot parse parameter: {}", err)
            },
            Error::MissingParameters => write!(f, "Missing parameter."),
            Error::OutOfBounds => write!(f, "Index out of bounds."),
            Error::StartLargerThanEnd => write!(f, "Start larger than end."),
            Error::QuestionNotFound => write!(f, "Question not found"),
        }
    }
}

fn extract_pagination (
    params: HashMap<String, String>,
) -> Result<Pagination, Error> {
    if params.contains_key("start") && params.contains_key("end") {
        return Ok( Pagination {
            start: params
                .get("start")
                .unwrap()
                .parse::<usize>()
                .map_err(Error::ParseError)?,
            end: params
                .get("end")
                .unwrap()
                .parse::<usize>()
                .map_err(Error::ParseError)?
        })
    }
    Err(Error::MissingParameters)
}

async fn add_answer(
    store: Store,
    params: HashMap<String, String>
) -> Result<impl Reply, Rejection> {
    let answer = Answer {
        id: AnswerId(store.answers.read().await.len().to_string()),
        content: params.get("content").unwrap().to_string(),
        question_id: QuestionId(params.get("question_id").unwrap().to_string()),
    };

    store.answers.write().await.insert(answer.id.clone(), answer);

    Ok(warp::reply::with_status("Answer added", StatusCode::OK))
}

async fn add_question(
    store: Store,
    question: Question
) -> Result<impl Reply, Rejection> {
    store.questions.write().await.insert(question.id.clone(), question);
    Ok(warp::reply::with_status(
        "Question added",
        StatusCode::OK
    ))
}

async fn update_question(
    id: String,
    store: Store,
    question: Question
) -> Result<impl Reply, Rejection> {
    match store.questions.write().await.get_mut(&QuestionId(id)) {
        Some(q) => *q = question,
        None => return Err(Error::QuestionNotFound.into())
    }
    Ok(warp::reply::with_status(
        "Question modified",
        StatusCode::OK,
    ))
}

async fn delete_question(
    id: String,
    store: Store
) -> Result<impl Reply, Rejection> {
    match store.questions.write().await.remove_entry(&QuestionId(id)) {
        Some((id, _)) => return Ok(warp::reply::with_status(
            format!("Question with id {} removed", id.0),
            StatusCode::OK,
        )),
        None => return Err(Error::QuestionNotFound.into())
    }
}

async fn get_questions(
    params: HashMap<String,
    String>,store: Store
) -> Result<impl Reply, Rejection> {
    if !params.is_empty() {
        let pagination = extract_pagination(params)?;
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
        let res: Vec<Question> = store.questions.read().await.values().cloned().collect();
        Ok(warp::reply::json(&res))
    }
}

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = r.find::<Error>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::RANGE_NOT_SATISFIABLE,
        ))
    } else if let Some(error) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
        error.to_string(),
        StatusCode::FORBIDDEN,
        ))
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        Ok(warp::reply::with_status(
            format!("{}, Please try again later!", error.to_string()),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else {
        Ok(warp::reply::with_status(
        "Route not found".to_string(),
        StatusCode::NOT_FOUND,
        ))
    }
}

#[tokio::main]
async fn main() {

    let store = Store::new();
    let store_filter = warp::any()
        .map(move || store.clone());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(
            &[Method::PUT, Method::DELETE, Method::GET, Method::POST]
        );

    let get_questions =
        warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter.clone())
        .and_then(get_questions);

    let add_questions =
        warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(add_question);

    let update_question =
        warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(update_question);

    let delete_question =
        warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(delete_question);

    let add_answer =
        warp::post()
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::form())
        .and_then(add_answer);

    let routes =
        get_questions
        .or(add_questions)
        .or(update_question)
        .or(add_answer)
        .or(delete_question)
        .with(cors)
        .recover(return_error);


    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
