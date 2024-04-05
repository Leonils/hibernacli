struct Question {
    statement: String,
    answer: Option<String>,
}

impl Question {
    pub fn new(statement: String) -> Question {
        Question {
            statement,
            answer: None,
        }
    }

    pub fn get_statement(&self) -> &str {
        &self.statement
    }

    pub fn answer(&mut self, answer: String) {
        self.answer = Some(answer);
    }

    pub fn get_answer(&self) -> Result<String, String> {
        match &self.answer {
            Some(answer) => Ok(answer.to_string()),
            None => Err("No answer provided".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::models::question::Question;

    #[test]
    fn test_create_a_question_with_a_statement_and_retrieve_it() {
        let question = Question::new("What is your name?".to_string());
        assert_eq!(question.get_statement(), "What is your name?");
    }

    #[test]
    fn test_create_a_string_question_and_answer_it() {
        let mut question = Question::new("What is your name?".to_string());
        question.answer("John".to_string());
        assert_eq!(question.get_answer().unwrap(), "John");
    }
}
