use std::collections::HashMap;
use uuid::Uuid;
use warp::http::StatusCode;

use crate::error::Error;
use crate::store::Store;
use crate::types::{
    answer::{Answer, AnswerId},
    question::QuestionId,
};

pub async fn add_answer(
    question_id: String,
    store: Store,
    params: HashMap<String, String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    if question_id.is_empty() {
        return Err(warp::reject::custom(Error::InvalidQuestionId));
    };
    let answer_content = match params.get("content") {
        Some(content) => content.to_string(),
        None => return Err(warp::reject::custom(Error::Required("content".to_string()))),
    };

    match store
        .questions
        .read()
        .await
        .get(&QuestionId(question_id.clone()))
    {
        Some(_) => (),
        None => return Err(warp::reject::custom(Error::QuestionNotFound(question_id))),
    }

    let answer = Answer {
        id: AnswerId(Uuid::new_v4().to_string()),
        content: answer_content,
        question_id: QuestionId(question_id),
    };
    store
        .answers
        .write()
        .await
        .insert(answer.id.clone(), answer);
    Ok(warp::reply::with_status("Answer added", StatusCode::OK))
}

pub async fn get_answers_by_question_id(
    question_id: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    if question_id.is_empty() {
        return Err(warp::reject::custom(Error::InvalidQuestionId));
    };

    let answers: Vec<Answer> = store
        .answers
        .read()
        .await
        .values()
        .filter(|answer| answer.question_id.0.contains(&question_id))
        .cloned()
        .collect();

    Ok(warp::reply::json(&answers))
}
