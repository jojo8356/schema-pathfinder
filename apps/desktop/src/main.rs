use schema_pathfinder::pathfinder_core::{
    best_path, discover_postgres_foreign_keys, list_tables_from_edges, load_edges_from_fixture,
    parse_format, render_path, render_tables, ForeignKeyEdge, TableIdentifier,
};
use slint::ComponentHandle;
use std::cell::RefCell;
use std::env;
use std::path::Path;
use std::rc::Rc;

slint::include_modules!();

const DEFAULT_FIXTURE: &str = "fixtures/postgres/dressshot_seed_fk_edges.json";

fn main() -> Result<(), slint::PlatformError> {
    let window = AppWindow::new()?;
    let state = Rc::new(RefCell::new(DesktopState::default()));

    window.set_fixture_path(DEFAULT_FIXTURE.into());
    window.set_database_url("".into());
    load_initial_edges(&window, &state);
    bind_load_fixture(&window, &state);
    bind_load_database(&window, &state);
    bind_find_path(&window, &state);

    window.run()
}

#[derive(Default)]
struct DesktopState {
    edges: Vec<ForeignKeyEdge>,
    tables: Vec<TableIdentifier>,
}

fn load_initial_edges(window: &AppWindow, state: &Rc<RefCell<DesktopState>>) {
    if Path::new(DEFAULT_FIXTURE).exists() {
        load_fixture_path(window, state, DEFAULT_FIXTURE);
        return;
    }

    window.set_status("No fixture loaded".into());
    window.set_tables_text("NO_TABLES".into());
}

fn bind_load_fixture(window: &AppWindow, state: &Rc<RefCell<DesktopState>>) {
    let weak_window = window.as_weak();
    let callback_state = Rc::clone(state);

    window.on_load_fixture(move |path| {
        if let Some(window) = weak_window.upgrade() {
            load_fixture_path(&window, &callback_state, path.as_str());
        }
    });
}

fn bind_load_database(window: &AppWindow, state: &Rc<RefCell<DesktopState>>) {
    let weak_window = window.as_weak();
    let callback_state = Rc::clone(state);

    window.on_load_database(move |database_url| {
        if let Some(window) = weak_window.upgrade() {
            load_database_url(&window, &callback_state, database_url.as_str());
        }
    });
}

fn bind_find_path(window: &AppWindow, state: &Rc<RefCell<DesktopState>>) {
    let weak_window = window.as_weak();
    let callback_state = Rc::clone(state);

    window.on_find_path(move |source, target, format| {
        if let Some(window) = weak_window.upgrade() {
            render_selected_path(
                &window,
                &callback_state,
                source.as_str(),
                target.as_str(),
                format.as_str(),
            );
        }
    });
}

fn load_fixture_path(window: &AppWindow, state: &Rc<RefCell<DesktopState>>, path: &str) {
    match load_edges_from_fixture(path) {
        Ok(edges) => {
            replace_edges(window, state, edges, format!("Loaded fixture {}", path));
        }
        Err(error) => {
            window.set_status(format!("{}: {}", error.code, error.message).into());
        }
    }
}

fn load_database_url(
    window: &AppWindow,
    state: &Rc<RefCell<DesktopState>>,
    raw_database_url: &str,
) {
    let database_url = raw_database_url.trim();

    if database_url.is_empty() {
        load_database_url_from_env(window, state);
        return;
    }

    load_database_url_value(window, state, database_url, "Loaded Postgres URL metadata");
}

fn load_database_url_from_env(window: &AppWindow, state: &Rc<RefCell<DesktopState>>) {
    match env::var("DATABASE_URL") {
        Ok(database_url) => {
            load_database_url_value(window, state, &database_url, "Loaded DATABASE_URL metadata");
        }
        Err(error) => {
            window.set_status(format!("DB_CONFIG_MISSING: {}", error).into());
        }
    }
}

fn load_database_url_value(
    window: &AppWindow,
    state: &Rc<RefCell<DesktopState>>,
    database_url: &str,
    status: &str,
) {
    match discover_postgres_foreign_keys(database_url) {
        Ok(edges) => {
            replace_edges(window, state, edges, status.to_string());
        }
        Err(error) => {
            window.set_status(format!("{}: {}", error.code, error.message).into());
        }
    }
}

fn replace_edges(
    window: &AppWindow,
    state: &Rc<RefCell<DesktopState>>,
    edges: Vec<ForeignKeyEdge>,
    status: String,
) {
    let tables = list_tables_from_edges(&edges);
    let tables_text = render_tables(&tables);
    let source = first_table_name(&tables);
    let target = last_table_name(&tables);

    {
        let mut current = state.borrow_mut();
        current.edges = edges;
        current.tables = tables;
    }

    window.set_tables_text(tables_text.into());
    window.set_source_table(source.into());
    window.set_target_table(target.into());
    window.set_output_text("".into());
    window.set_status(status.into());
}

fn render_selected_path(
    window: &AppWindow,
    state: &Rc<RefCell<DesktopState>>,
    source_table: &str,
    target_table: &str,
    format_name: &str,
) {
    let current = state.borrow();

    if current.edges.is_empty() {
        window.set_status("NO_TABLES: load a fixture or DATABASE_URL first".into());
        return;
    }

    let format = match parse_format(format_name) {
        Ok(value) => value,
        Err(error) => {
            window.set_status(format!("{}: {}", error.code, error.message).into());
            return;
        }
    };

    let path = match best_path(&current.edges, source_table, target_table) {
        Ok(value) => value,
        Err(error) => {
            window.set_status(format!("{}: {}", error.code, error.message).into());
            window.set_output_text("".into());
            return;
        }
    };

    match render_path(&path, format) {
        Ok(rendered) => {
            window.set_output_text(rendered.into());
            window.set_status(
                format!("Path found: {} hop(s), score {}", path.length, path.score).into(),
            );
        }
        Err(error) => {
            window.set_status(format!("{}: {}", error.code, error.message).into());
        }
    }
}

fn first_table_name(tables: &[TableIdentifier]) -> String {
    if let Some(table) = tables.first() {
        return table_name(table);
    }

    String::new()
}

fn last_table_name(tables: &[TableIdentifier]) -> String {
    if let Some(table) = tables.last() {
        return table_name(table);
    }

    String::new()
}

fn table_name(table: &TableIdentifier) -> String {
    if table.schema == "public" {
        return table.table.clone();
    }

    format!("{}.{}", table.schema, table.table)
}
