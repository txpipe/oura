use std::{
    collections::VecDeque,
    sync::{mpsc::Receiver, Arc, RwLock},
    thread::JoinHandle,
    time::Duration,
};

use crate::ports::Event;

use crate::utils::throttle::Throttle;

pub type Error = Box<dyn std::error::Error>;

use crossterm::{
    event::EnableMouseCapture,
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use std::io;
use tui::text::Span;
use tui::Terminal;
use tui::{
    backend::CrosstermBackend,
    widgets::{List, ListItem},
};

const THROTTLE_MIN_SPAN_MILLIS: u64 = 500;

struct ConsoleApp {
    tx_count: u64,
    tx_items: VecDeque<Event>,
}

fn reducer_loop(event_rx: Receiver<Event>, app: Arc<RwLock<ConsoleApp>>) -> Result<(), Error> {
    let mut throttle = Throttle::new(Duration::from_millis(THROTTLE_MIN_SPAN_MILLIS));

    loop {
        let evt = event_rx.recv()?;
        throttle.wait_turn();
        //println!("{:?}", evt);
        let mut app = app.write().unwrap();
        app.tx_count += 1;
        app.tx_items.push_back(evt);
        if app.tx_items.len() > 10 {
            app.tx_items.pop_front();
        }
    }
}

fn tui_loop(app: Arc<RwLock<ConsoleApp>>) -> Result<(), Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut throttle = Throttle::new(Duration::from_millis(250));

    loop {
        terminal.draw(|f| {
            let app = app.read().unwrap();
            let size = f.size();

            let list_items = app
                .tx_items
                .iter()
                .map(|evt| {
                    let content = Span::from(format!("{:?} => {:?}", evt.context, evt.data));
                    ListItem::new(content)
                })
                .collect::<Vec<_>>();

            let list = List::new(list_items);

            f.render_widget(list, size);
        })?;

        if crossterm::event::poll(Duration::from_millis(50))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                match key.code {
                    crossterm::event::KeyCode::Char('q') => return Ok(()),
                    _ => {}
                }
            }
        }

        throttle.wait_turn();
    }
}

pub fn bootstrap(rx: Receiver<Event>) -> Result<JoinHandle<()>, Error> {
    let console = Arc::new(RwLock::new(ConsoleApp {
        tx_count: 0,
        tx_items: VecDeque::new(),
    }));

    let c1 = console.clone();
    let handle1 = std::thread::spawn(move || reducer_loop(rx, c1).unwrap());

    let c2 = console.clone();
    let handle2 = std::thread::spawn(move || tui_loop(c2).unwrap());

    Ok(handle2)
}
