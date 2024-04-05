struct Question {
    statement: String,
}

impl Question {
    pub fn new(statement: String) -> Question {
        Question { statement }
    }

    pub fn get_statement(&self) -> &str {
        &self.statement
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
}
