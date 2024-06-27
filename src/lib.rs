#![warn(clippy::all)]

use sqlx::migrate;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::{http::Method, reply::Reply, Filter};

use handle_errors::{return_error, Error};

pub mod config;
mod profanity;
mod routes;
mod store;
mod types;

async fn build_routes(
    store: store::Store,
) -> impl Filter<Extract = impl Reply> + Clone {
    let store_filter = warp::any().map(move || store.clone());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[
            Method::PUT,
            Method::DELETE,
            Method::POST,
            Method::GET,
        ]);

    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter.clone())
        .and_then(routes::question::get_questions)
        .with(warp::trace(|info| {
            tracing::info_span!(
                "get_questions request",
                method = %info.method(),
                path = %info.path(),
                id = %uuid::Uuid::new_v4(),
            )
        }));

    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::question::add_question);

    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::question::update_question);

    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and_then(routes::question::delete_question);

    let get_question_by_id = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::question::get_question_by_id);

    let add_answer = warp::post()
        .and(warp::path("questions"))
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and(warp::body::form())
        .and_then(routes::answer::add_answer);

    let get_answers_by_question_id = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::answer::get_answers_by_question_id);

    let registration = warp::post()
        .and(warp::path("registration"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::authentication::register);

    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::authentication::login);

    get_questions
        .or(add_question)
        .or(add_answer)
        .or(update_question)
        .or(delete_question)
        .or(get_question_by_id)
        .or(get_answers_by_question_id)
        .or(registration)
        .or(login)
        .with(cors)
        .with(warp::trace::request())
        .recover(return_error)
}

pub async fn setup_store(
    config: &config::Config,
) -> Result<store::Store, Error> {
    // Connection
    // postgres://username:password@localhost:5432/rustwebdev
    let store = store::Store::new(&format!(
        "postgres://{}:{}@{}:{}/{}",
        config.db_user,
        config.db_password,
        config.db_host,
        config.db_port,
        config.db_name
    ))
    .await;

    let _ = migrate!()
        .run(&store.clone().connection)
        .await
        .map_err(handle_errors::Error::MigrationError);

    let log_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| {
        format!(
            "handle_errors={},rusty_web_development={},warp={}",
            config.log_level, config.log_level, config.log_level
        )
    });

    tracing_subscriber::fmt()
        // Use the filter we built above to determine which traces to record.
        .with_env_filter(log_filter)
        // Record an event when eac span closes.
        // This can be used to time our routes durations!
        .with_span_events(FmtSpan::CLOSE)
        .init();

    Ok(store)
}

pub async fn run(config: config::Config, store: store::Store) {
    let routes = build_routes(store).await;
    warp::serve(routes).run(([127, 0, 0, 1], config.port)).await;
}
