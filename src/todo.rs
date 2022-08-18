use std::{fs, io};
use std::path::Path;
use chrono::{DateTime, Local, Utc, TimeZone, serde::ts_seconds_option, Duration};
use serde::{Serialize, Deserialize, Serializer, Deserializer, de::Visitor};

pub fn serialize_local_dt<S>
    (dt: &Option<DateTime<Local>>, serializer: S) 
    -> Result<S::Ok, S::Error> 
    where S: Serializer 
{
    ts_seconds_option::serialize(
        &dt.map(|loc| 
            Utc.from_local_datetime(&loc.naive_local()).single().unwrap()
        ), 
        serializer
    )
}

pub fn deserialize_local_dt<'de, D>(deserializer: D) 
    -> Result<Option<DateTime<Local>>, D::Error> 
    where D: Deserializer<'de>
{
    ts_seconds_option::deserialize(deserializer).map(
        |opt| opt.map(
            |utc| Local.from_utc_datetime(&utc.naive_utc())
        )
    )
}

pub fn serialize_duration<S>
    (dur_opt: &Option<Duration>, serializer: S) 
    -> Result<S::Ok, S::Error> 
    where S: Serializer
{
    match dur_opt {
        Some(dur) => serializer.serialize_some(&dur.num_milliseconds()),
        None => serializer.serialize_none()
    }
}

pub fn deserialize_duration<'de, D>(deserializer: D) 
    -> Result<Option<Duration>, D::Error> 
    where D: Deserializer<'de> 
{
    let ms: Option<i64> = Deserialize::deserialize(deserializer)?;
    Ok(ms.map(|t| Duration::milliseconds(t)))
}

// Represents a single todo item
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Todo {
    name:       String,
    desc:       String,
    done:       bool,
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
    time_taken: Option<Duration>,
    #[serde(serialize_with = "serialize_local_dt", deserialize_with = "deserialize_local_dt")]
    do_at:      Option<DateTime<Local>>,
    #[serde(serialize_with = "serialize_local_dt", deserialize_with = "deserialize_local_dt")]
    do_by:      Option<DateTime<Local>>,
    #[serde(serialize_with = "serialize_local_dt", deserialize_with = "deserialize_local_dt")]
    ignore_by:  Option<DateTime<Local>>
}

impl Todo {
    pub fn new(name: String, desc: String) -> Self {
        Self {
            name: name,
            desc: desc,
            done: false,
            time_taken: None,
            do_at: None,
            do_by: None,
            ignore_by: None
        }
    }

    pub fn do_at(&self, datetime: DateTime<Local>) -> Self {
        Self {
            name: self.name.clone(),
            desc: self.desc.clone(),
            done: self.done,
            time_taken: None,
            do_at: Some(datetime),
            do_by: self.do_by,
            ignore_by: self.ignore_by
        }
    }
    pub fn do_by(&self, datetime: DateTime<Local>) -> Self {
        Self {
            name: self.name.clone(),
            desc: self.desc.clone(),
            done: self.done,
            time_taken: None,
            do_at: self.do_at,
            do_by: Some(datetime),
            ignore_by: self.ignore_by
        }
    }
    pub fn ignore_by(&mut self, datetime: DateTime<Local>) -> Self {
        Self {
            name: self.name.clone(),
            desc: self.desc.clone(),
            done: self.done,
            time_taken: None,
            do_at: self.do_at,
            do_by: self.do_by,
            ignore_by: Some(datetime)
        }
    }

    pub fn time_taken(&mut self, duration: Duration) -> Self {
        Self {
            name: self.name.clone(),
            desc: self.desc.clone(),
            done: self.done,
            time_taken: Some(duration),
            do_at: self.do_at,
            do_by: self.do_by,
            ignore_by: self.ignore_by
        }
    }

    pub fn toggle_done(&mut self) {
        self.done = !self.done;
    }

    pub fn get_name(&self) -> String { self.name.clone() }
    pub fn get_desc(&self) -> String { self.desc.clone() }
    pub fn is_done(&self) -> bool { self.done }
    pub fn get_time_taken(&self) -> Option<Duration> { self.time_taken.clone() }
    pub fn get_do_at(&self) -> Option<DateTime<Local>> { self.do_at.clone() }
    pub fn get_do_by(&self) -> Option<DateTime<Local>> { self.do_by.clone() }
    pub fn get_ignore_by(&self) -> Option<DateTime<Local>> { self.ignore_by.clone() }
    pub fn is_late(&self) -> bool {if let Some(date) = self.get_do_by() {Local::now() > date} else {false}}
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ProgramData {
    tasks: Vec<Todo>
}

impl ProgramData {
    pub fn load_from_file(filename: &Path) -> Result<Self, io::Error> {
        let json = fs::read_to_string(filename)?;
        Ok(ProgramData { tasks: serde_json::from_str(json.as_str())? })
    }
    pub fn new_blank() -> Self {
        Self { tasks: vec![] }
    }

    pub fn get_data_or_blank(fp: &Path) -> ProgramData {
        if fp.is_file() {ProgramData::load_from_file(fp).expect("File cannot be read!")}
        else {ProgramData::new_blank()}
    }

    pub fn get_tasks(&self) -> &Vec<Todo> {&self.tasks}
    pub fn get_tasks_mut(&mut self) -> &mut Vec<Todo> {&mut self.tasks}
    pub fn add_task(&mut self, task: Todo) {self.tasks.push(task)}
    pub fn get_task_by_optional_index(&self, index: Option<usize>) -> Option<&Todo> {
        match index {
            Some(n) => if self.tasks.len() > n {
                Some(&self.tasks[n])
            } else {
                None
            }
            None => None
        }
    }
    pub fn get_task_by_optional_index_mut(&mut self, index: Option<usize>) -> Option<&mut Todo> {
        match index {
            Some(n) => if self.tasks.len() > n {
                Some(&mut self.tasks[n])
            } else {
                None
            }
            None => None
        }
    }
}