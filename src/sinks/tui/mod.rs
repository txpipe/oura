use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
    thread::JoinHandle,
    time::Duration,
};

use crate::{framework::Event, utils::throttle::Throttle, pipelining::StageReceiver};

pub type Error = Box<dyn std::error::Error>;

use crossterm::{
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

fn reducer_loop(event_rx: StageReceiver, app: Arc<RwLock<ConsoleApp>>) -> Result<(), Error> {
    let mut throttle = Throttle::new(Duration::from_millis(THROTTLE_MIN_SPAN_MILLIS));

    loop {
        let evt = event_rx.recv()?;
        throttle.wait_turn();
        //println!("{:?}", evt);
        let mut app = app.write().expect("failed to acquire TUI app lock");
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
            let app = app.read().expect("failed to acquire TUI app lock");
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
                if let crossterm::event::KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }

        throttle.wait_turn();
    }
}

pub fn bootstrap(rx: StageReceiver) -> Result<JoinHandle<()>, Error> {
    let console = Arc::new(RwLock::new(ConsoleApp {
        tx_count: 0,
        tx_items: VecDeque::new(),
    }));

    let c1 = console.clone();
    let _handle1 =
        std::thread::spawn(move || reducer_loop(rx, c1).expect("TUI reducer loop failed"));

    let c2 = console;
    let handle2 = std::thread::spawn(move || tui_loop(c2).expect("TUI drawing loop failed"));

    Ok(handle2)
}
