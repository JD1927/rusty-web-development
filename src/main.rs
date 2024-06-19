#![warn(clippy::all)]

use clap::Parser;
use dotenv::dotenv;
use sqlx::migrate;
use std::env;
use tracing::info;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::{http::Method, Filter};

use handle_errors::return_error;

mod profanity;
mod routes;
mod store;
mod types;

/// Q&A web service API
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Which errors we want to log (info, warn or error)
    #[clap(short, long, default_value = "info")]
    log_level: String,
    /// URL for the postgres database
    #[clap(long, default_value = "localhost")]
    database_host: String,
    /// PORT number for the database connection
    #[clap(long, default_value = "5432")]
    database_port: u16,
    /// Database name
    #[clap(long, default_value = "rustwebdev")]
    database_name: String,
    /// Web server port
    #[clap(long, default_value = "8080")]
    port: u16,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    if env::var("BAD_WORDS_API_KEY").is_err() {
        panic!("Bad words API key not set!");
    }

    if env::var("PASETO_KEY").is_err() {
        panic!("PASETO key not set!");
    }
    let port = env::var("PORT")
        .ok()
        .map(|val| val.parse::<u16>())
        .unwrap_or(Ok(8080))
        .map_err(handle_errors::Error::ParseInt)
        .unwrap();

    let args = Args::parse();

    let log_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| {
        format!(
            "handle_errors={},rusty_web_development={},warp={}",
            args.log_level, args.log_level, args.log_level
        )
    });
    // Connection
    // postgres://username:password@localhost:5432/rustwebdev
    let store = store::Store::new(&format!(
        "postgres://postgres:password@{}:{}/{}",
        args.database_host, args.database_port, args.database_name
    ))
    .await;

    let _ = migrate!()
        .run(&store.clone().connection)
        .await
        .map_err(handle_errors::Error::MigrationError);

    let store_filter = warp::any().map(move || store.clone());

    tracing_subscriber::fmt()
        // Use the filter we built above to determine which traces to record.
        .with_env_filter(log_filter)
        // Record an event when eac span closes.
        // This can be used to time our routes durations!
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::PUT, Method::DELETE, Method::POST, Method::GET]);

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

    let routes = get_questions
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
        .recover(return_error);

    info!("Q&A service build ID:[{}]", env!("RUSTY_WEB_DEV_VERSION"));
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}
