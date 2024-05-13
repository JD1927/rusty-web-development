use handle_errors::Error;
use std::collections::HashMap;
use uuid::Uuid;
use warp::http::StatusCode;

use crate::store::Store;
use crate::types::pagination::extract_pagination;
use crate::types::question::{Question, QuestionId};

pub async fn add_question(
    store: Store,
    question: Question,
) -> Result<impl warp::Reply, warp::Rejection> {
    if question.title.is_empty() {
        return Err(warp::reject::custom(Error::Required("title".to_string())));
    }
    if question.content.is_empty() {
        return Err(warp::reject::custom(Error::Required("content".to_string())));
    }
    let question = Question {
        id: Some(QuestionId(Uuid::new_v4().to_string())),
        ..question
    };
    store
        .questions
        .write()
        .await
        .insert(question.id.clone().unwrap(), question);

    Ok(warp::reply::with_status("Question added", StatusCode::OK))
}

pub async fn update_question(
    id: String,
    store: Store,
    question: Question,
) -> Result<impl warp::Reply, warp::Rejection> {
    let question_id = id.clone();
    match store.questions.write().await.get_mut(&QuestionId(id)) {
        Some(q) => *q = question,
        None => return Err(warp::reject::custom(Error::QuestionNotFound(question_id))),
    }
    Ok(warp::reply::with_status("Question updated", StatusCode::OK))
}

pub async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
    req_id: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    log::info!("{} Start querying questions", req_id);
    if !params.is_empty() {
        let res: Vec<Question> = store.questions.read().await.values().cloned().collect();
        let store_length = res.len();
        let pagination = extract_pagination(params, store_length)?;
        log::info!("{} Pagination set {:?}", req_id, &pagination);
        let res = &res[pagination.start..pagination.end];

        Ok(warp::reply::json(&res))
    } else {
        log::info!("{} No pagination used", req_id);
        let res: Vec<Question> = store.questions.read().await.values().cloned().collect();

        Ok(warp::reply::json(&res))
    }
}

pub async fn get_question_by_id(
    id: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let question_id = id.clone();
    match store.questions.read().await.get(&QuestionId(id)) {
        Some(question) => Ok(warp::reply::json(&question)),
        None => Err(warp::reject::custom(Error::QuestionNotFound(question_id))),
    }
}

pub async fn delete_question(
    id: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let question_id = id.clone();
    match store.questions.write().await.remove(&QuestionId(id)) {
        Some(_) => Ok(warp::reply::with_status(
            "Question deleted!",
            StatusCode::OK,
        )),
        None => Err(warp::reject::custom(Error::QuestionNotFound(question_id))),
    }
}
