//! Render reproducible UI artifacts without touching the user's real practice history.
use cmdtyper::{
    app::{App, AppState},
    core::engine::TypingEngine,
    data::models::{Difficulty, RecordMode},
    ui,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Terminal, backend::TestBackend};
use std::{
    env, fs,
    path::Path,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
fn shot(app: &App, out: &Path, name: &str, width: u16, height: u16) {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| ui::render(frame, app)).unwrap();
    let buffer = terminal.backend().buffer();
    let cells:Vec<_>=(0..height).flat_map(|y|(0..width).map(move|x|(x,y))).map(|(x,y)|{let cell=&buffer[(x,y)];serde_json::json!({"x":x,"y":y,"s":cell.symbol(),"fg":format!("{:?}",cell.fg),"bg":format!("{:?}",cell.bg)})}).collect();
    fs::write(
        out.join(format!("{name}-{width}x{height}.json")),
        serde_json::to_vec(&serde_json::json!({"width":width,"height":height,"cells":cells}))
            .unwrap(),
    )
    .unwrap();
}
fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = root.join("target/ui-validation");
    fs::create_dir_all(&out).unwrap();
    let user = env::temp_dir().join(format!(
        "cmdtyper-render-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    unsafe {
        env::set_var("CMDTYPER_DATA_DIR", root.join("data"));
        env::set_var("CMDTYPER_USER_DIR", &user);
    }
    let mut app = App::new().unwrap();
    let now = Instant::now();
    let wall = chrono::Utc::now().timestamp_millis();
    for i in 0..15 {
        let mut engine = TypingEngine::new("ls -la /var/log");
        for (j, ch) in engine.target.clone().iter().enumerate() {
            engine.input_at(
                *ch,
                now + Duration::from_millis(i * 5000 + j as u64 * 180),
                wall + i as i64 * 5000 + j as i64 * 180,
            );
        }
        app.persist_record(engine.finish_at(
            "ls-la",
            Difficulty::Basic,
            RecordMode::Typing,
            now + Duration::from_millis(i * 5000 + 3000),
            wall + i as i64 * 5000 + 3000,
        ));
    }
    for (name, state) in [
        ("home", AppState::Home),
        ("calendar", AppState::Calendar),
        ("learn", AppState::LearnHub),
        ("scenarios", AppState::Scenarios),
    ] {
        app.state = state;
        for (w, h) in [(40, 10), (80, 24), (120, 36)] {
            shot(&app, &out, name, w, h);
        }
    }
    cmdtyper::flow::practice_flow::enter(&mut app, None, None);
    shot(&app, &out, "foundations", 120, 36);
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    shot(&app, &out, "foundation-teaching", 80, 24);
    for ch in app.typing_engine.target.clone() {
        app.handle_key(KeyEvent::new(
            if ch == '\n' {
                KeyCode::Enter
            } else {
                KeyCode::Char(ch)
            },
            KeyModifiers::NONE,
        ));
    }
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    shot(&app, &out, "three-exercises", 80, 24);
    app.state = AppState::Scenarios;
    app.scenario_state.selected_index = 0;
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    shot(&app, &out, "scenario-typing", 80, 24);
    let target = app.typing_engine.target.clone();
    for ch in target {
        app.handle_key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
    }
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    shot(&app, &out, "scenario-evidence", 80, 24);
    if let Some(command) = app
        .commands
        .iter()
        .find(|c| c.id == "fileops-mkdir-work-parents")
        .cloned()
    {
        app.typing_engine.reset(&command.command);
        app.typing_commands = vec![command];
        app.typing_index = 0;
        app.state = AppState::Typing;
        let first = app
            .typing_engine
            .target
            .iter()
            .take_while(|c| **c != '\n')
            .copied()
            .collect::<Vec<_>>();
        for ch in first {
            app.handle_key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
        }
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        shot(&app, &out, "workflow-output", 80, 24);
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        shot(&app, &out, "workflow", 80, 24);
        shot(&app, &out, "workflow", 40, 10);
    }
    fs::remove_dir_all(user).unwrap();
    println!("{}", out.display());
}
