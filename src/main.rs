mod todo;
use todo::Todo;
use std::path::{Path, PathBuf};

fn main() {
    let test_todo = Todo::new(String::from("Stuff"), String::from("Do stuff and things and more stuff"));
    println!("{}", test_todo.to_json().unwrap());
}
