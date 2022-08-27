use std::{fs, io};
use std::io::Write;
use std::cmp::Ordering;
use std::path::Path;
use chrono::{DateTime, Local, Utc, TimeZone, serde::ts_seconds_option, Duration};
use serde::{Serialize, Deserialize, Serializer, Deserializer};

#[derive(Debug, Serialize, Deserialize, Clone)]
enum SortingMethod {
    None
}

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
}

impl Todo {
    pub fn new(name: String, desc: String) -> Self {
        Self {
            name,
            desc,
            done: false,
            time_taken: None,
            do_at: None,
            do_by: None,
        }
    }

    pub fn do_at(self, datetime: DateTime<Local>) -> Self {
        Self {
            do_at: Some(datetime),
            ..self
        }
    }
    pub fn do_by(self, datetime: DateTime<Local>) -> Self {
        Self {
            do_by: Some(datetime),
            ..self
        }
    }
    pub fn time_taken(self, duration: Duration) -> Self {
        Self {
            time_taken: Some(duration),
            ..self
        }
    }

    pub fn toggle_done(self) -> Self {
        Self {
            done: !self.done,
            ..self
        }
    }

    pub fn get_name(&self) -> String { self.name.clone() }
    pub fn get_desc(&self) -> String { self.desc.clone() }
    pub fn is_done(&self) -> bool { self.done }
    pub fn get_time_taken(&self) -> Option<Duration> { self.time_taken.clone() }
    pub fn get_do_at(&self) -> Option<DateTime<Local>> { self.do_at.clone() }
    pub fn get_do_by(&self) -> Option<DateTime<Local>> { self.do_by.clone() }
    pub fn is_late(&self) -> bool {if let Some(date) = self.get_do_by() {Local::now() > date} else {false}}
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ProgramData {
    tasks: Vec<Todo>,
    select_idx: Option<usize>,
    show_done: bool,
    visible: Vec<usize>,
    sorting_method: SortingMethod,
}

impl ProgramData {
    pub fn load_from_file(filename: &Path) -> Result<Self, io::Error> {
        let json = fs::read_to_string(filename)?;
        Ok(serde_json::from_str(json.as_str())?)
    }
    pub fn new_blank() -> Self {
        Self { tasks: vec![], select_idx: None, show_done: false, visible: vec![], sorting_method: SortingMethod::None }
    }
    pub fn get_data_or_blank(fp: &Path) -> ProgramData {
        if fp.is_file() {ProgramData::load_from_file(fp).expect("File cannot be read!")}
        else {ProgramData::new_blank()}
    }
    pub fn save_to_file(&self, fp: &Path) -> Result<(), io::Error> {
        let json = serde_json::to_string(self)?;
        let mut file = fs::File::create(fp)?;
        write!(&mut file, "{}", json.as_str())?;
        Ok(())
    }
    fn update_visible(&mut self) {
        fn order_tasks(sorting_method: SortingMethod, t1: &Todo, t2: &Todo) -> Ordering {
            Ordering::Less
        }
        let mut visible_unsorted_enum: Vec<(usize, &Todo)> 
            = self.tasks.iter()
                        .enumerate()
                        .filter(|(_idx, task)| self.is_visible(task)).collect();
        visible_unsorted_enum.sort_by(|(_i1, t1), (_i2, t2)| order_tasks(self.sorting_method.clone(), t1, t2));
        self.visible = visible_unsorted_enum.iter().map(|(idx, _)| *idx).collect();
    }

    pub fn get_visible_tasks(&self) -> Vec<&Todo> {self.visible.iter().map(|i| &self.tasks[*i]).collect()}
    pub fn get_task_refs(&self) -> Vec<&Todo> {self.tasks.iter().collect()}
    pub fn get_tasks(&self) -> &Vec<Todo> {&self.tasks}
    pub fn get_visible_idx(&self) -> Option<usize> {self.select_idx}
    pub fn add_task(&mut self, task: Todo) {self.tasks.push(task); self.update_visible();}
    /*pub fn get_task_by_optional_index(&self, index: Option<usize>) -> Option<&Todo> {
        match index {
            Some(n) => if self.tasks.len() > n {
                Some(&self.tasks[n])
            } else {
                None
            }
            None => None
        }
    }*/
    pub fn get_current_task(&self) -> Option<&Todo> {
        self.select_idx.map(|idx| &self.tasks[idx])
    }
    pub fn show_done(&self) -> bool {self.show_done}

    fn is_visible(&self, task: &Todo) -> bool {
        if !self.show_done && task.is_done() {return false};
        true
    }

    fn next_visible_task_idx(&self, idx: usize) -> usize {
        let mut out = idx + 1;
        while !self.is_visible(&self.tasks[out]) && out < self.tasks.len() - 1 {
            out += 1;
        }
        if !self.is_visible(&self.tasks[out]) {return idx}
        out
    }

    fn prev_visible_task_idx(&self, idx: usize) -> usize {
        let mut out = idx - 1;
        while !self.is_visible(&self.tasks[out]) && out > 0 {
            out -= 1;
        }
        if !self.is_visible(&self.tasks[out]) {return idx}
        out
    }

    pub fn next_task(&mut self) {
        //TODO: COMPLETELY REWRITE
        if let Some(idx) = self.select_idx {
            if self.tasks.len() > idx + 1 {
                self.select_idx = Some(self.next_visible_task_idx(idx));
            }
        } else {
            if self.tasks.len() > 0 {
                self.select_idx = Some(0);
            }
        }
    }

    pub fn prev_task(&mut self) {
        //TODO: COMPLETELY REWRITE
        if let Some(idx) = self.select_idx {
            if idx > 0 {
                self.select_idx = Some(self.prev_visible_task_idx(idx));
            }
        } else {
            if self.tasks.len() > 0 {
                self.select_idx = Some(0);
            }
        }
    }

    pub fn toggle_done(&mut self) {
        if let Some(idx) = self.select_idx {
            self.tasks[idx] = self.tasks[idx].clone().toggle_done();
            if !self.show_done() {self.next_task()}
        }
        self.update_visible();
    }
    pub fn toggle_show_done(&mut self) {self.show_done = !self.show_done; self.update_visible();}

    pub fn delete_current(&mut self) {
        if let Some(idx) = self.select_idx {
            self.tasks.remove(idx);
            if idx >= self.tasks.len() {
                self.select_idx = Some(idx - 1);
            }
        }
        self.update_visible();
    }
}
