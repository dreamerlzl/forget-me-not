//use std::fmt::Display;
//
//use serde::{Deserialize, Serialize};

pub type TaskContext = String;

pub fn default_context() -> TaskContext {
    "default".to_owned()
}

//#[derive(Debug, Deserialize, PartialEq, Clone, Serialize)]
//pub struct TaskContext(pub String);

//impl TaskContext {
//    pub fn new(context: String) -> Self {
//        Self(context)
//    }
//}
//
//impl Default for TaskContext {
//    fn default() -> Self {
//        TaskContext("default".to_owned())
//    }
//}
//
//impl Display for TaskContext {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        self.0.fmt(f)
//    }
//}
