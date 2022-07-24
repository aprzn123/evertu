use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};

// Represents a single todo item in a form that can be serialized by Serde
#[derive(Clone, Debug, Serialize, Deserialize)]
struct SerialTodo {
    name:       String,
    desc:       String,
    done:       bool,
    do_at:      Option<String>, // Dates serialized as DateTime.to_rfc3339
    do_by:      Option<String>,
    ignore_by:  Option<String>
}

// Represents a single todo item in a form that is more understandable to the software
#[derive(Clone, PartialEq, Debug)]
pub struct Todo {
    name:       String,
    desc:       String,
    done:       bool,
    do_at:      Option<DateTime<Local>>,
    do_by:      Option<DateTime<Local>>,
    ignore_by:  Option<DateTime<Local>>
}

impl From<Todo> for SerialTodo {
    fn from(todo: Todo) -> Self {
        Self {
            name:       todo.name,
            desc:       todo.desc,
            done:       todo.done,
            do_at:      todo.do_at.map(|datetime: DateTime<Local>| datetime.to_rfc3339()),
            do_by:      todo.do_by.map(|datetime: DateTime<Local>| datetime.to_rfc3339()),
            ignore_by:  todo.ignore_by.map(|datetime: DateTime<Local>| datetime.to_rfc3339()),
        }
    }
}

impl From<SerialTodo> for Todo {
    fn from(todo: SerialTodo) -> Self {
        Self {
            name:       todo.name,
            desc:       todo.desc,
            done:       todo.done,
            do_at:      todo.do_at.map(|datetime_str: String| DateTime::parse_from_rfc3339(datetime_str.as_str())
                                                                       .expect("Bad Date Formatting!")
                                                                       .with_timezone(&Local)),

            do_by:      todo.do_by.map(|datetime_str: String| DateTime::parse_from_rfc3339(datetime_str.as_str())
                                                                       .expect("Bad Date Formatting!")
                                                                       .with_timezone(&Local)),

            ignore_by:  todo.ignore_by.map(|datetime_str: String| DateTime::parse_from_rfc3339(datetime_str.as_str())
                                                                           .expect("Bad Date Formatting!")
                                                                           .with_timezone(&Local)),
        }
    }
}

impl Todo {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&SerialTodo::from(self.clone()))
    }
    pub fn from_json(json: String) -> Result<Self, serde_json::Error> {
        serde_json::from_str::<SerialTodo>(json.as_str()).map(|x|Todo::from(x))
    }
    pub fn new(name: String, desc: String) -> Self {
        Self {
            name: name,
            desc: desc,
            done: false,
            do_at: None,
            do_by: None,
            ignore_by: None
        }
    }

    pub fn do_at(&mut self, datetime: DateTime<Local>) {
        self.do_at = Some(datetime);
    }
    pub fn do_by(&mut self, datetime: DateTime<Local>) {
        self.do_by = Some(datetime);
    }
    pub fn ignore_by(&mut self, datetime: DateTime<Local>) {
        self.ignore_by = Some(datetime);
    }
}



#[derive(Debug, Serialize, Deserialize)]
struct SerialProgramData {
    tasks: Vec<SerialTodo>
}

#[derive(Debug)]
pub struct ProgramData {
    tasks: Vec<Todo>
}

impl From<ProgramData> for SerialProgramData {
    fn from(data: ProgramData) -> Self {
        Self {
            tasks: data.tasks.iter().map(|x| SerialTodo::from(x.clone())).collect()
        }
    }
}

impl From<SerialProgramData> for ProgramData {
    fn from(data: SerialProgramData) -> Self {
        Self {
            tasks: data.tasks.iter().map(|x| Todo::from(x.clone())).collect()
        }
    }
}