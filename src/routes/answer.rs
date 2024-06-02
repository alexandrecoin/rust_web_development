use crate::{error, store, types};
use std::collections::HashMap;
use warp::http::StatusCode;
use warp::{Rejection, Reply};

pub(crate) async fn add_answer(
    store: store::Store,
    params: HashMap<String, String>,
) -> Result<impl Reply, Rejection> {
    let answer = types::answer::Answer {
        id: types::answer::AnswerId("1".to_string()),
        content: match params.get("content") {
            Some(c) => c.to_string(),
            None => return Err(warp::reject::custom(error::Error::MissingParameters)),
        },
        question_id: match params.get("questionId") {
            Some(id) => match store
                .questions
                .read()
                .await
                .get(&types::question::QuestionId(id.to_string()))
            {
                Some(q) => types::question::QuestionId(q.id.to_string()),
                None => return Err(warp::reject::custom(error::Error::QuestionNotFound)),
            },
            None => return Err(warp::reject::custom(error::Error::MissingParameters)),
        },
    };

    store
        .answers
        .write()
        .await
        .insert(answer.id.clone(), answer);

    Ok(warp::reply::with_status("Answer created", StatusCode::OK))
}
