use crate::types::question::QuestionId;
pub struct Answer {
    pub(crate) id: AnswerId,
    pub(crate) content: String,
    pub(crate) question_id: QuestionId,
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct AnswerId(pub String);
