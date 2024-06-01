use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use tokio;
use tokio::sync::RwLock;
use warp::body::BodyDeserializeError;
use warp::reject::Reject;
use warp::{
    cors::CorsForbidden,
    http::{Method, StatusCode},
    Filter, Rejection, Reply,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, Hash, PartialEq)]
struct QuestionId(String);

impl std::fmt::Display for QuestionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "QuestionId is")
    }
}

struct Answer {
    id: AnswerId,
    content: String,
    question_id: QuestionId,
}

#[derive(Clone, Eq, Hash, PartialEq)]
struct AnswerId(String);

#[derive(Clone)]
struct Store {
    questions: Arc<RwLock<HashMap<QuestionId, Question>>>,
    answers: Arc<RwLock<HashMap<AnswerId, Answer>>>,
}

impl Store {
    fn new() -> Self {
        Store {
            questions: Arc::new(RwLock::new(Self::init())),
            answers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn add_question(self, question: Question) -> Self {
        self.questions
            .write()
            .await
            .insert(question.id.clone(), question);
        self
    }

    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../questions.json");
        serde_json::from_str(file).expect("Can't read questions.json")
    }
}

async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    if !params.is_empty() {
        let questions: Vec<Question> = store.questions.read().await.values().cloned().collect();
        let pagination = extract_pagination(params, questions.len())?;
        let questions = &questions[pagination.start..pagination.end];
        Ok(warp::reply::json(&questions))
    } else {
        let questions: Vec<Question> = store.questions.read().await.values().cloned().collect();
        Ok(warp::reply::json(&questions))
    }
}

async fn add_question(store: Store, question: Question) -> Result<impl Reply, Rejection> {
    store
        .questions
        .write()
        .await
        .insert(question.id.clone(), question);
    Ok(warp::reply::with_status("Question added", StatusCode::OK))
}

async fn update_question(
    id: String,
    store: Store,
    question: Question,
) -> Result<impl Reply, Rejection> {
    match store.questions.write().await.get_mut(&QuestionId(id)) {
        Some(q) => *q = question,
        None => return Err(warp::reject::custom(Error::QuestionNotFound)),
    }
    Ok(warp::reply::with_status("Question updated", StatusCode::OK))
}

async fn delete_question(id: String, store: Store) -> Result<impl Reply, Rejection> {
    return match store.questions.write().await.remove(&QuestionId(id)) {
        Some(_) => Ok(warp::reply::with_status("Question deleted", StatusCode::OK)),
        None => Err(warp::reject::custom(Error::QuestionNotFound)),
    };
}

async fn add_answer(
    store: Store,
    params: HashMap<String, String>,
) -> Result<impl Reply, Rejection> {
    let answer = Answer {
        id: AnswerId("1".to_string()),
        content: match params.get("content") {
            Some(c) => c.to_string(),
            None => return Err(warp::reject::custom(Error::MissingParameters)),
        },
        question_id: match params.get("questionId") {
            Some(id) => match store.questions.read().await.get(&QuestionId(id.to_string())) {
                Some(q) => QuestionId(q.id.to_string()),
                None => return Err(warp::reject::custom(Error::QuestionNotFound))
            },
            None => return Err(warp::reject::custom(Error::MissingParameters)),
        },
    };

    store
        .answers
        .write()
        .await
        .insert(answer.id.clone(), answer);

    Ok(warp::reply::with_status("Answer created", StatusCode::OK))
}



async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    println!("{:?}", r);
    if let Some(error) = r.find::<Error>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::RANGE_NOT_SATISFIABLE,
        ))
    } else if let Some(error) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}

#[derive(Debug)]
enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    NonProcessable,
    OutOfBounds,
    QuestionNotFound,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::ParseError(ref err) => {
                write!(f, "Cannot parse parameter: {}", err)
            }
            Error::MissingParameters => write!(f, "Missing one or more parameters"),
            Error::NonProcessable => {
                write!(f, "Start parameter cannot be greater than end parameter")
            }
            Error::OutOfBounds => {
                write!(f, "Not enough questions, please lower your end parameter")
            }
            Error::QuestionNotFound => {
                write!(f, "Question not found")
            }
        }
    }
}

impl Reject for Error {}

#[derive(Debug)]
struct Pagination {
    start: usize,
    end: usize,
}

fn extract_pagination(
    params: HashMap<String, String>,
    max_possible_length: usize,
) -> Result<Pagination, Error> {
    if params.contains_key("start") && params.contains_key("end") {
        let start = params
            .get("start")
            .unwrap()
            .parse::<usize>()
            .map_err(Error::ParseError)?;
        let end = params
            .get("end")
            .unwrap()
            .parse::<usize>()
            .map_err(Error::ParseError)?;

        if start > end {
            return Err(Error::NonProcessable);
        };

        if end > max_possible_length {
            return Err(Error::OutOfBounds);
        };

        return Ok(Pagination { start, end });
    }
    Err(Error::MissingParameters)
}

#[tokio::main]
async fn main() {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::GET, Method::POST, Method::PUT, Method::DELETE]);

    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());

    let get_items = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter.clone())
        .and_then(get_questions);

    let add_item = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(add_question);

    let update_item = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(update_question);

    let delete_item = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(delete_question);

    let add_answer = warp::post()
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::form())
        .and_then(add_answer);

    let routes = get_items
        .or(add_item)
        .or(update_item)
        .or(delete_item)
        .or(add_answer)
        .with(cors)
        .recover(return_error);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
