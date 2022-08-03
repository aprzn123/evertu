use chrono::{DateTime, Local, Utc, TimeZone, serde::ts_seconds_option};
use serde::{Serialize, Deserialize, Serializer, Deserializer};

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

// Represents a single todo item
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Todo {
    name:       String,
    desc:       String,
    done:       bool,
    #[serde(serialize_with = "serialize_local_dt", deserialize_with = "deserialize_local_dt")]
    do_at:      Option<DateTime<Local>>,
    #[serde(serialize_with = "serialize_local_dt", deserialize_with = "deserialize_local_dt")]
    do_by:      Option<DateTime<Local>>,
    #[serde(serialize_with = "serialize_local_dt", deserialize_with = "deserialize_local_dt")]
    ignore_by:  Option<DateTime<Local>>
}

impl Todo {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
    pub fn from_json(json: String) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json.as_str())
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

    pub fn get_name(&self) -> String { self.name.clone() }
    pub fn get_desc(&self) -> String { self.desc.clone() }
    pub fn is_done(&self) -> bool { self.done }
    pub fn get_do_at(&self) -> Option<DateTime<Local>> { self.do_at.clone() }
    pub fn get_do_by(&self) -> Option<DateTime<Local>> { self.do_by.clone() }
    pub fn get_ignore_by(&self) -> Option<DateTime<Local>> { self.ignore_by.clone() }
}



#[derive(Debug, Serialize, Deserialize)]
pub struct ProgramData {
    tasks: Vec<Todo>
}
