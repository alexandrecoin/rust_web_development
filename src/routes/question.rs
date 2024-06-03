use crate::{store, types};
use std::collections::HashMap;
use warp::http::StatusCode;
use warp::{Rejection, Reply};
use handle_errors::Error;

pub(crate) async fn get_questions(
    params: HashMap<String, String>,
    store: store::Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    if !params.is_empty() {
        let questions: Vec<types::question::Question> =
            store.questions.read().await.values().cloned().collect();
        let pagination = types::pagination::extract_pagination(params, questions.len())?;
        let questions = &questions[pagination.start..pagination.end];
        Ok(warp::reply::json(&questions))
    } else {
        let questions: Vec<types::question::Question> =
            store.questions.read().await.values().cloned().collect();
        Ok(warp::reply::json(&questions))
    }
}

pub(crate) async fn add_question(
    store: store::Store,
    question: types::question::Question,
) -> Result<impl Reply, Rejection> {
    store
        .questions
        .write()
        .await
        .insert(question.id.clone(), question);
    Ok(warp::reply::with_status("Question added", StatusCode::OK))
}

pub(crate) async fn update_question(
    id: String,
    store: store::Store,
    question: types::question::Question,
) -> Result<impl Reply, Rejection> {
    match store
        .questions
        .write()
        .await
        .get_mut(&types::question::QuestionId(id))
    {
        Some(q) => *q = question,
        None => return Err(warp::reject::custom(Error::QuestionNotFound)),
    }
    Ok(warp::reply::with_status("Question updated", StatusCode::OK))
}

pub(crate) async fn delete_question(
    id: String,
    store: store::Store,
) -> Result<impl Reply, Rejection> {
    return match store
        .questions
        .write()
        .await
        .remove(&types::question::QuestionId(id))
    {
        Some(_) => Ok(warp::reply::with_status("Question deleted", StatusCode::OK)),
        None => Err(warp::reject::custom(Error::QuestionNotFound)),
    };
}
