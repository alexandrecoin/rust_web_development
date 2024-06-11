use handle_errors::return_error;
use log::log;
use uuid::Uuid;
use warp::{http::Method, Filter};

mod routes;
mod store;
mod types;

#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    log::error!("This is an error");
    log::warn!("This is a warning");
    log::info!("This is an info");

    let log = warp::log::custom(|info| {
        log::info!("{} {} {} {:?} {:?}", info.method(), info.path(), info.status(), info.elapsed(), info.request_headers());
    });

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::GET, Method::POST, Method::PUT, Method::DELETE]);

    let store = store::Store::new();
    let store_filter = warp::any().map(move || store.clone());

    let id_filter = warp::any().map(|| Uuid::new_v4().to_string());

    let get_items = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter.clone())
        .and(id_filter)
        .and_then(routes::question::get_questions);

    let add_item = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::question::add_question);

    let update_item = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::question::update_question);

    let delete_item = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::question::delete_question);

    let add_answer = warp::post()
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::form())
        .and_then(routes::answer::add_answer);

    let routes = get_items
        .or(add_item)
        .or(update_item)
        .or(delete_item)
        .or(add_answer)
        .with(cors)
        .with(log)
        .recover(return_error);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
