use serde::{Deserialize, Serialize};

use crate::types::question::QuestionId;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnswerId(pub i32);

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Answer {
    pub id: AnswerId,
    pub content: String,
    pub question_id: QuestionId,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct NewAnswer {
    pub content: String,
    pub question_id: QuestionId,
}
