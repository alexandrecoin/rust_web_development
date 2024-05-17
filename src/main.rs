use tokio;
use warp::Filter;

#[derive(Debug)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>
}

impl Question {
    fn new(id: &str, title: &str, content: &str, tags: Vec<String>) -> Self {
        Question {
            id: QuestionId::from(id),
            title: String::from(title),
            content: String::from(content),
            tags: Some(tags)
        }
    }
}

#[derive(Debug)]
struct QuestionId(String);

impl QuestionId {
    fn from(id: &str) -> Self {
        QuestionId(String::from(id))
    }
}

#[tokio::main]
async fn main() {
    let hello = warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name));

    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
