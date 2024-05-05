use std::num;
use warp::{
    filters::body::BodyDeserializeError, filters::cors::CorsForbidden, http::StatusCode,
    reject::Reject, reply::Reply, Rejection,
};

#[derive(Debug)]
pub enum Error {
    MissingParameters(String),
    InvalidRange,
    ParseInt(num::ParseIntError),
    QuestionNotFound(String),
    InvalidQuestionId,
    Required(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::ParseInt(ref err) => {
                write!(f, "Cannot parse parameter: {}", err)
            }
            Error::MissingParameters(ref message) => {
                write!(f, "Missing parameters: {}", message)
            }
            Error::InvalidRange => write!(f, "Invalid pagination range"),
            Error::QuestionNotFound(ref id) => {
                write!(f, "Question with id '{}' not found!", id)
            }
            Error::InvalidQuestionId => write!(f, "Invalid question ID!"),
            Error::Required(ref field) => {
                write!(
                    f,
                    "Field '{}' is required. Cannot be empty or undefined",
                    field
                )
            }
        }
    }
}

impl Reject for Error {}

pub async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = r.find::<Error>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::BAD_REQUEST,
        ))
    } else if let Some(error) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}
