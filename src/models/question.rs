enum QuestionType {
    String,
    SingleChoice(Vec<String>),
}

struct Question {
    statement: String,
    answer: Option<String>,
    question_type: QuestionType,
}

impl Question {
    pub fn new(statement: String, question_type: QuestionType) -> Question {
        Question {
            statement,
            question_type,
            answer: None,
        }
    }

    pub fn get_statement(&self) -> &str {
        &self.statement
    }

    fn validate_answer(&self, answer: &String) -> Result<(), String> {
        match &self.question_type {
            QuestionType::SingleChoice(answers) if !answers.contains(&answer) => Err(format!(
                "Invalid answer. Possible answers are: {}",
                answers.join(", ")
            )),
            _ => Ok(()),
        }
    }

    pub fn set_answer(&mut self, answer: String) -> Result<(), String> {
        self.validate_answer(&answer)?;
        self.answer = Some(answer);
        Ok(())
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
    use crate::models::question::{Question, QuestionType};

    #[test]
    fn test_create_a_question_with_a_statement_and_retrieve_it() {
        let question = Question::new("What is your name?".to_string(), QuestionType::String);
        assert_eq!(question.get_statement(), "What is your name?");
    }

    #[test]
    fn test_create_a_string_question_and_answer_it() {
        let mut question = Question::new("What is your name?".to_string(), QuestionType::String);
        question.set_answer("John".to_string()).unwrap();
        assert_eq!(question.get_answer().unwrap(), "John");
    }

    #[test]
    fn test_create_a_string_question_and_get_answer_without_answering_shall_return_error() {
        let question = Question::new("What is your name?".to_string(), QuestionType::String);
        assert_eq!(question.get_answer().unwrap_err(), "No answer provided");
    }

    #[test]
    fn test_create_a_single_choice_question_and_retrieve_possible_answers_shall_get_them_all() {
        let question = Question::new(
            "What is your favorite color?".to_string(),
            QuestionType::SingleChoice(vec!["Red".to_string(), "Blue".to_string()]),
        );
        assert_eq!(question.get_statement(), "What is your favorite color?");
        match question.question_type {
            QuestionType::SingleChoice(answers) => assert_eq!(answers, vec!["Red", "Blue"]),
            _ => panic!("Invalid question type"),
        }
    }

    #[test]
    fn test_create_a_single_choice_question_and_answer_it_should_save_answer() {
        let mut question = Question::new(
            "What is your favorite color?".to_string(),
            QuestionType::SingleChoice(vec!["Red".to_string(), "Blue".to_string()]),
        );
        question.set_answer("Blue".to_string()).unwrap();
        assert_eq!(question.get_answer().unwrap(), "Blue");
    }

    #[test]
    fn test_create_a_single_choice_question_and_answer_it_with_a_bad_answer_should_return_error() {
        let mut question = Question::new(
            "What is your favorite color?".to_string(),
            QuestionType::SingleChoice(vec!["Red".to_string(), "Blue".to_string()]),
        );
        question.set_answer("Green".to_string()).unwrap_err();
    }
}
