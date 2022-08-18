// Using https://blog.logrocket.com/rust-and-tui-building-a-command-line-interface-in-rust/

mod todo;
use chrono::{Duration, Utc, Local, TimeZone};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use crossterm::event;
use todo::{Todo, ProgramData};
use tui::layout::{Layout, Direction, Constraint, Alignment};
use tui::style::{Color, Style, Modifier};
use tui::text::{Spans, Span};
use tui::widgets::{Paragraph, Block, Borders, BorderType, Tabs, ListState, List, ListItem};
use std::cmp::{max, min};
use std::path::Path;
use std::io;
use tui::{backend::CrosstermBackend, Terminal};
use std::sync::mpsc;
use std::thread::{self, current};
use crossterm::event::Event as CEvent;

enum Event<I> {
   Input(I),
   Tick
}

#[derive(Clone)]
enum Tab {
    Tasks,
    NewTask
}

impl From<&Tab> for usize {
    fn from(input: &Tab) -> usize {
        match input {
            Tab::Tasks => 0,
            Tab::NewTask => 1
        }
    }
}

fn task_view_widget(opt_task: Option<&Todo>) -> Paragraph {
    if let Some(task) = opt_task {
        Paragraph::new( vec![
            Spans::from(vec![Span::styled(
                task.get_name(),
                Style::default().add_modifier(Modifier::BOLD).add_modifier(Modifier::UNDERLINED).fg(Color::Blue)
            )]),
            Spans::from(vec![if task.is_done() 
                            {Span::styled("Completed", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))} 
                    else {Span::styled("Not Completed", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))}
            ]),
            Spans::from(vec![if let Some(due_date) = task.get_do_by() {
                Span::styled(due_date.format("Due on %b %d, %Y at %H:%M").to_string(), 
                Style::default().fg(if task.is_late() && !task.is_done() {Color::Red} 
                                    else {Color::Green}))
            } else {
                Span::styled("No due date set", Style::default().fg(Color::DarkGray))
            }]),
            Spans::from(vec![Span::raw(match task.get_time_taken() {
                Some(dur) => if dur.num_hours() > 0 {format!("Takes {}h {}m", dur.num_hours(), dur.num_minutes() - 60 * dur.num_hours())} else {format!("Takes {}m", dur.num_minutes())},
                None => String::from("No duration set")
            })]),
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![Span::raw(task.get_desc())]),
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![if let Some(do_date) = task.get_do_at() {
                Span::styled(do_date.format("Scheduled for %b %d, %Y at %H:%M").to_string(), 
                Style::default().fg(if do_date < Local::now() && !task.is_done() {Color::Red} 
                                    else {Color::Green}))
            } else {
                Span::styled("No date scheduled", Style::default().fg(Color::DarkGray))
            }]),
        ])
    } else {
        Paragraph::new(vec![Spans::from(vec![Span::styled("No Task Selected...", Style::default().fg(Color::DarkGray))])])
    }
    .block(Block::default()
        .borders(Borders::all())
        .border_type(BorderType::Rounded)
    )
}

fn task_list_widget(tasks: &Vec<Todo>) -> List {
    List::new::<Vec<ListItem>>(
        tasks.iter().zip(0..tasks.len()).map(|(task, idx)| 
            ListItem::new(
                Spans::from(vec![Span::styled(
                    task.get_name(), 
                    if task.is_done() {Style::default().fg(Color::Green)} 
                    else if !task.is_late() {Style::default().fg(Color::Yellow)} 
                    else {Style::default().fg(Color::Red)}
                )])
            )
        ).collect()
    )
    .highlight_style(Style::default().bg(Color::White))
    .block(
        Block::default()
        .borders(Borders::all())
        .border_type(BorderType::Rounded)
    )
}

fn main() {
    let mut program_data = ProgramData::get_data_or_blank(Path::new("/home/aprzn/.evertu"));
    let test_todo = Todo::new(String::from("Stuff"), String::from("Do stuff and things and more stuff")).do_by(Local.timestamp(1431648000, 0)).time_taken(Duration::minutes(90));
    let test_todo_2 = Todo::new(String::from("Things"), String::from("Do *even more* stuff and things and stuff")).do_by(Local.timestamp(2431648000, 0)).time_taken(Duration::minutes(430));
    program_data.add_task(test_todo);
    program_data.add_task(test_todo_2);
    // println!("{}", test_todo.to_json().unwrap());

    // Input mode
    enable_raw_mode().expect("Must be able to run in Raw Mode");
    // IPC
    let (tx, rx) = mpsc::channel();
    // Update every 200 ms *OR* when an input is received
    let tick_rate = Duration::milliseconds(200);
    thread::spawn(move || {
        let mut last_tick = Utc::now();
        loop {
            // Timeout is duration until next tick
            let timeout = tick_rate
                .checked_sub(&(Utc::now() - last_tick))
                .unwrap_or_else(|| Duration::seconds(0));
            // Wait for events within that duration and send them over the mpsc channel
            if event::poll(timeout.to_std().expect("Chrono duration incompatible with std duration")).expect("Event Poll Broken?") {
                if let CEvent::Key(key) = event::read().expect("Failed to read events") {
                    tx.send(Event::Input(key)).expect("Failed to send events");
                }
            }
            // If no inputs received during that time, send a tick event
            if (Utc::now() - last_tick) >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Utc::now();
                }
            }
        }
    });

    // Initialize terminal
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).expect("Failed to create terminal");
    terminal.clear().expect("Terminal clearing failed");

    let mut current_tab = Tab::Tasks;
    let mut task_list_state = ListState::default();
    task_list_state.select(None);
    // Rendering Loop
    loop {
        terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(2)
                    ].as_ref()
                ).split(size);
            
            // let test_task = Paragraph::new(usize::from(current_tab.clone()).to_string())
            //     .style(Style::default().fg(Color::Blue))
            //     .alignment(Alignment::Left)
            //     .block(Block::default()
            //                 .borders(Borders::all())
            //                 .style(Style::default().fg(Color::White))
            //                 .title("Expanded View")
            //                 .border_type(BorderType::Rounded)
            //     );
            
            let menu_titles = vec!["Task View", "New Task"];

            let menu = menu_titles.iter().map(|title| {
                Spans::from(vec![
                    Span::styled(
                        *title,
                        Style::default().fg(Color::Yellow)
                    )
                ])
            }).collect();
                
            let tabs = Tabs::new(menu)
                                    .select((&current_tab).into())
                                    .highlight_style(Style::default()
                                        .add_modifier(Modifier::BOLD)
                                        .add_modifier(Modifier::UNDERLINED)
                                    ).block(Block::default()
                                        .borders(Borders::all())
                                        .border_type(BorderType::Rounded)
                                    );

            rect.render_widget(tabs, chunks[0]);

            // Render a different main panel based on the current tab
            match current_tab {
                Tab::Tasks => {
                    let task_chunks = Layout::default()
                                    .direction(Direction::Horizontal)
                                    .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
                                    .split(chunks[1]);
                    rect.render_widget(task_view_widget(program_data.get_task_by_optional_index(task_list_state.selected())), task_chunks[1]);
                    rect.render_stateful_widget(task_list_widget(program_data.get_tasks()), task_chunks[0], &mut task_list_state);
                },
                Tab::NewTask => { }
            }
        }).unwrap();

        // Do stuff with user input
        match rx.recv().expect("Error recieving event") {
            Event::Input(event) => match event.code {
                event::KeyCode::Char('q') => {
                    disable_raw_mode().unwrap();
                    terminal.clear().unwrap();
                    terminal.show_cursor().unwrap();
                    break;
                },
                event::KeyCode::Char('h') => current_tab = Tab::Tasks,
                event::KeyCode::Char('N') => current_tab = Tab::NewTask,
                event::KeyCode::Up => {
                    if let Some(selected) = task_list_state.selected() {
                        if selected > 0 {task_list_state.select(Some(selected - 1));}
                    } else {
                        if program_data.get_tasks().len() > 0 { task_list_state.select(Some(0)); }
                    }
                },
                event::KeyCode::Down => {
                    if let Some(selected) = task_list_state.selected() {
                        if selected + 1 < program_data.get_tasks().len() {task_list_state.select(Some(selected + 1))};
                    } else {
                        if program_data.get_tasks().len() > 0 { task_list_state.select(Some(program_data.get_tasks().len() - 1)); }
                    }
                },
                event::KeyCode::Enter => if let Some(task) = program_data.get_task_by_optional_index_mut(task_list_state.selected()) {task.toggle_done()},
                _ => {}
            },
            Event::Tick => {}
        }
    }
}
