use warp::Filter;
use std::path::PathBuf;

pub fn web_routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let static_files = warp::fs::dir(PathBuf::from("frontend/dist"));

    let index = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file("frontend/dist/index.html"));

    let spa_fallback = warp::get()
        .and(warp::path::tail())
        .and(warp::any())
        .map(|_| {
            warp::reply::with_header(
                warp::reply::html(std::fs::read_to_string("frontend/dist/index.html").unwrap_or_default()),
                "content-type",
                "text/html",
            )
        });

    static_files
        .or(index)
        .or(spa_fallback)
}