use serde::{Deserialize, Serialize};
use std::{collections::HashMap, num};
use warp::{
    filters::cors::CorsForbidden,
    http::{Method, StatusCode},
    reject::Reject,
    Filter, Rejection, Reply,
};

#[derive(Debug)]
enum Error {
    MissingParameters,
    InvalidRange,
    ParseInt(num::ParseIntError),
}

#[derive(Clone)]
struct Store {
    questions: HashMap<QuestionId, Question>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
struct QuestionId(String);

#[derive(Debug)]
struct Pagination {
    start: usize,
    end: usize,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::ParseInt(ref err) => {
                write!(f, "Cannot parse parameter: {}", err)
            }
            Error::MissingParameters => write!(f, "Missing parameter"),
            Error::InvalidRange => write!(f, "Invalid pagination range"),
        }
    }
}

impl Reject for Error {}

impl Store {
    fn new() -> Self {
        Store {
            questions: Self::init(),
        }
    }

    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../question.json");
        serde_json::from_str(file).expect("can't read questions.json")
    }
}

async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    if !params.is_empty() {
        let res: Vec<Question> = store.questions.values().cloned().collect();
        let store_length = res.len();
        let pagination = extract_pagination(params, store_length)?;
        let res = &res[pagination.start..pagination.end];

        Ok(warp::reply::json(&res))
    } else {
        let res: Vec<Question> = store.questions.values().cloned().collect();

        Ok(warp::reply::json(&res))
    }
}

fn extract_pagination(
    params: HashMap<String, String>,
    store_length: usize,
) -> Result<Pagination, Error> {
    if let (Some(start), Some(end)) = (params.get("start"), params.get("end")) {
        let start = start.parse::<usize>().map_err(Error::ParseInt)?;
        let end = end.parse::<usize>().map_err(Error::ParseInt)?;

        if start < end && start <= store_length && end <= store_length {
            return Ok(Pagination { start, end });
        } else {
            return Err(Error::InvalidRange);
        }
    }
    Err(Error::MissingParameters)
}

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    println!("{:?}", r);
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
    let store_filter = warp::any().map(move || store.clone());
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::PUT, Method::DELETE, Method::POST, Method::GET]);

    let get_items = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter)
        .and_then(get_questions)
        .recover(return_error);

    let routes = get_items.with(cors);

    println!("[WARP] - Running on http://localhost:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
