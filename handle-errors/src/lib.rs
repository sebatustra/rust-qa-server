use warp::{
    filters::{
        body::BodyDeserializeError,
        cors::CorsForbidden
    },
    reject::Reject,
    Rejection,
    Reply,
    http::StatusCode
};
use tracing::{event, Level};

#[derive(Debug)]
pub enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    OutOfBounds,
    StartLargerThanEnd,
	DatabaseQueryError,
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
			Error::DatabaseQueryError => {
				write!(f, "Cannot update, invalid data.")
			}
        }
    }
}

pub async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
	if let Some(crate::Error::DatabaseQueryError) = r.find() {
		event!(Level::ERROR, "Database query error");
		Ok(warp::reply::with_status(
 		crate::Error::DatabaseQueryError.to_string(),
			StatusCode::UNPROCESSABLE_ENTITY,
		))
	} else if let Some(error) = r.find::<Error>() {
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
