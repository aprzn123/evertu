// Using https://blog.logrocket.com/rust-and-tui-building-a-command-line-interface-in-rust/



mod todo;
use chrono::{Duration, DateTime, Utc, Local, TimeZone};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use crossterm::event;
use todo::Todo;
use tui::layout::{Layout, Direction, Constraint, Alignment};
use tui::style::{Color, Style, Modifier};
use tui::text::{Spans, Span};
use tui::widgets::{Paragraph, Block, Borders, BorderType, Tabs};
use std::path::{Path, PathBuf};
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

fn task_view_widget(task: &Todo) -> Paragraph {
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
            Style::default().fg(if due_date < Local::now() && !task.is_done() {Color::Red} 
                                else {Color::Green}))
        } else {
            Span::styled("No due date set", Style::default().fg(Color::DarkGray))
        }]),
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
}

fn main() {
    let test_todo = Todo::new(String::from("Stuff"), String::from("Do stuff and things and more stuff")).do_by(Local.timestamp(1431648000, 0));
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
    let mut focused_task_index: Option<i32> = None;
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

            match current_tab {
                Tab::Tasks => rect.render_widget(task_view_widget(&test_todo), chunks[1]),
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
                _ => {}
            },
            Event::Tick => {}
        }
    }
}
