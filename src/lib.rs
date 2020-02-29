use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct NewTodo {
    pub title: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTodo {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Id {
    pub id: Uuid
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Todo {
    pub id: Uuid,
    pub title: String,
    pub completed: bool,
}

impl Todo {
    pub fn from_new(todo: &NewTodo) -> Self {
        Todo {
            id: Uuid::new_v4(),
            title: todo.title.clone(),
            completed: false,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn create_todo_from_new_todo() {
        let new = NewTodo { title: String::from("Learn Rust!") };
        let todo = Todo::from_new(&new);
        assert_eq!(todo.title, new.title);
        assert_eq!(todo.completed, false);
        assert_eq!(todo.id.to_string().len(), 36);
    }
}