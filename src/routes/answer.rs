use tracing::instrument;
use warp::http::StatusCode;

use crate::profanity::check_profanity;
use crate::store::Store;
use crate::types::answer::NewAnswer;

#[instrument]
pub async fn add_answer(
    store: Store,
    new_answer: NewAnswer,
) -> Result<impl warp::Reply, warp::Rejection> {
    let content = match check_profanity(new_answer.content).await {
        Ok(res) => res,
        Err(e) => return Err(warp::reject::custom(e)),
    };

    let new_answer = NewAnswer {
        content,
        question_id: new_answer.question_id,
    };

    match store.add_answer(new_answer).await {
        Ok(_) => Ok(warp::reply::with_status("Answer added!", StatusCode::OK)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

#[instrument]
pub async fn get_answers_by_question_id(
    question_id: i32,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let res = match store.get_answers_by_question_id(question_id).await {
        Ok(res) => res,
        Err(e) => return Err(warp::reject::custom(e)),
    };

    Ok(warp::reply::json(&res))
}
