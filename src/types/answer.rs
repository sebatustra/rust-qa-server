use serde::{Deserialize, Serialize};

use crate::types::question::QuestionId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Answer {
  pub id: AnswerId,
  pub content: String,
  pub question_id: QuestionId
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash, Deserialize)]
pub struct AnswerId(pub String);
