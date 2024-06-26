use std::collections::HashMap;
use tracing::{event, instrument, Level};
use warp::http::StatusCode;

use crate::profanity::check_profanity;
use crate::store::Store;
use crate::types::account::Session;
use crate::types::pagination::{extract_pagination, Pagination};
use crate::types::question::{NewQuestion, Question};

#[instrument]
pub async fn add_question(
    session: Session,
    store: Store,
    new_question: NewQuestion,
) -> Result<impl warp::Reply, warp::Rejection> {
    let account_id = session.account_id;
    // Uses tokio::join! to wrap the async function that returns future, without awaiting it
    // tokio::spawn (parallelism) and tokio::join! (concurrent)
    let title = check_profanity(new_question.title);
    let content = check_profanity(new_question.content);
    // Run both on parallel, returning a tuple that contains the result for both title and content
    let (title, content) = tokio::join!(title, content);

    // Check if title has an error
    match title.is_err() {
        true => return Err(warp::reject::custom(title.unwrap_err())),
        false => (),
    }
    // Check if content has an error
    match content.is_err() {
        true => return Err(warp::reject::custom(content.unwrap_err())),
        false => (),
    }

    let question = NewQuestion {
        title: title.unwrap(),
        content: content.unwrap(),
        tags: new_question.tags,
    };

    match store.add_question(question, account_id).await {
        Ok(_) => {
            Ok(warp::reply::with_status("Question added", StatusCode::OK))
        }
        Err(e) => Err(warp::reject::custom(e)),
    }
}

#[instrument]
pub async fn update_question(
    question_id: i32,
    session: Session,
    store: Store,
    question: Question,
) -> Result<impl warp::Reply, warp::Rejection> {
    let account_id = session.account_id;

    if store.is_question_owner(question_id, &account_id).await? {
        // Uses tokio::join! to wrap the async function that returns future, without awaiting it
        let title = check_profanity(question.title);
        let content = check_profanity(question.content);
        // Run both concurrently, returning a tuple that contains the result for both title and content
        let (title, content) = tokio::join!(title, content);

        if title.is_ok() && content.is_ok() {
            let question = Question {
                id: question.id,
                title: title.unwrap(),
                content: content.unwrap(),
                tags: question.tags,
            };
            match store
                .update_question(question, question_id, account_id)
                .await
            {
                Ok(res) => Ok(warp::reply::json(&res)),
                Err(e) => Err(warp::reject::custom(e)),
            }
        } else {
            Err(warp::reject::custom(
                title.expect_err("Expected API call to have failed here"),
            ))
        }
    } else {
        Err(warp::reject::custom(handle_errors::Error::Unauthorized))
    }
}

#[instrument]
pub async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    event!(target: "rusty-web-development", Level::INFO, "querying questions");
    let mut pagination = Pagination::default();
    if !params.is_empty() {
        event!(Level::INFO, pagination = true);
        pagination = extract_pagination(params)?;
    }
    event!(Level::INFO, pagination = false);
    let res: Vec<Question> = match store
        .get_questions(pagination.limit, pagination.offset)
        .await
    {
        Ok(res) => res,
        Err(e) => {
            return Err(warp::reject::custom(e));
        }
    };
    Ok(warp::reply::json(&res))
}

#[instrument]
pub async fn get_question_by_id(
    question_id: i32,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let res = match store.get_question_by_id(question_id).await {
        Ok(res) => res,
        Err(e) => return Err(warp::reject::custom(e)),
    };

    Ok(warp::reply::json(&res))
}

#[instrument]
pub async fn delete_question(
    question_id: i32,
    session: Session,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let account_id = session.account_id;
    if store.is_question_owner(question_id, &account_id).await? {
        match store.delete_question(question_id, account_id).await {
            Ok(_) => Ok(warp::reply::with_status(
                format!("Question {} deleted", question_id),
                StatusCode::OK,
            )),
            Err(e) => Err(warp::reject::custom(e)),
        }
    } else {
        Err(warp::reject::custom(handle_errors::Error::Unauthorized))
    }
}
