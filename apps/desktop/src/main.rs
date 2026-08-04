use schema_pathfinder::pathfinder_core::{
    best_path, discover_postgres_schema, load_schema_from_fixture, load_schema_from_sql_file,
    parse_format, render_path, render_tables, ForeignKeyEdge, SchemaMetadata, TableIdentifier,
};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::env;
use std::path::Path;
use std::rc::Rc;

slint::include_modules!();

const DEFAULT_FIXTURE: &str = "fixtures/postgres/dressshot_seed_fk_edges.json";

fn main() -> Result<(), slint::PlatformError> {
    let window = AppWindow::new()?;
    let state = Rc::new(RefCell::new(DesktopState::default()));

    window.set_fixture_path(DEFAULT_FIXTURE.into());
    window.set_sql_path("".into());
    window.set_database_url("".into());
    load_initial_edges(&window, &state);
    bind_load_source(&window, &state);
    bind_select_schema(&window, &state);
    bind_find_path(&window, &state);
    bind_toggle_fullscreen(&window);
    window.window().set_maximized(true);
    sync_initial_schema_selection(&window, &state);

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

fn sync_initial_schema_selection(window: &AppWindow, state: &Rc<RefCell<DesktopState>>) {
    let current = state.borrow();
    let schema = window.get_selected_schema();

    apply_schema_selection(window, &current.tables, schema.as_str());
}

fn bind_load_source(window: &AppWindow, state: &Rc<RefCell<DesktopState>>) {
    let weak_window = window.as_weak();
    let callback_state = Rc::clone(state);

    window.on_load_source(move |source_kind, fixture_path, sql_path, database_url| {
        if let Some(window) = weak_window.upgrade() {
            load_selected_source(
                &window,
                &callback_state,
                source_kind.as_str(),
                fixture_path.as_str(),
                sql_path.as_str(),
                database_url.as_str(),
            );
        }
    });
}

fn load_selected_source(
    window: &AppWindow,
    state: &Rc<RefCell<DesktopState>>,
    source_kind: &str,
    fixture_path: &str,
    sql_path: &str,
    database_url: &str,
) {
    if source_kind == "fixture" {
        load_fixture_path(window, state, fixture_path);
        return;
    }

    if source_kind == "sql" {
        load_sql_path(window, state, sql_path);
        return;
    }

    if source_kind == "postgres" {
        load_database_url(window, state, database_url);
        return;
    }

    window.set_status(format!("SOURCE_KIND_UNSUPPORTED: {}", source_kind).into());
}

fn bind_select_schema(window: &AppWindow, state: &Rc<RefCell<DesktopState>>) {
    let weak_window = window.as_weak();
    let callback_state = Rc::clone(state);

    window.on_select_schema(move |schema| {
        if let Some(window) = weak_window.upgrade() {
            let current = callback_state.borrow();
            apply_schema_selection(&window, &current.tables, schema.as_str());
        }
    });
}

fn bind_toggle_fullscreen(window: &AppWindow) {
    let weak_window = window.as_weak();

    window.on_toggle_fullscreen(move || {
        if let Some(window) = weak_window.upgrade() {
            if window.window().is_fullscreen() {
                window.window().set_fullscreen(false);
                window.window().set_maximized(true);
                return;
            }

            window.window().set_fullscreen(true);
        }
    });
}

fn bind_find_path(window: &AppWindow, state: &Rc<RefCell<DesktopState>>) {
    let weak_window = window.as_weak();
    let callback_state = Rc::clone(state);

    window.on_find_path(move |schema, source, target, format| {
        if let Some(window) = weak_window.upgrade() {
            render_selected_path(
                &window,
                &callback_state,
                schema.as_str(),
                source.as_str(),
                target.as_str(),
                format.as_str(),
            );
        }
    });
}

fn load_fixture_path(window: &AppWindow, state: &Rc<RefCell<DesktopState>>, path: &str) {
    match load_schema_from_fixture(path) {
        Ok(schema) => {
            replace_schema(window, state, schema, format!("Loaded fixture {}", path));
        }
        Err(error) => {
            window.set_status(format!("{}: {}", error.code, error.message).into());
        }
    }
}

fn load_sql_path(window: &AppWindow, state: &Rc<RefCell<DesktopState>>, raw_path: &str) {
    let path = raw_path.trim();

    if path.is_empty() {
        window.set_status("SQL_PATH_MISSING: choose a PostgreSQL .sql file".into());
        return;
    }

    match load_schema_from_sql_file(path) {
        Ok(schema) => {
            replace_schema(window, state, schema, format!("Loaded SQL schema {}", path));
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
    match discover_postgres_schema(database_url) {
        Ok(schema) => {
            replace_schema(window, state, schema, status.to_string());
        }
        Err(error) => {
            window.set_status(format!("{}: {}", error.code, error.message).into());
        }
    }
}

fn replace_schema(
    window: &AppWindow,
    state: &Rc<RefCell<DesktopState>>,
    schema: SchemaMetadata,
    status: String,
) {
    let tables_text = render_tables(&schema.tables);
    let schema_names = schema_options(&schema.tables);
    let selected_schema = first_schema_name(&schema_names);

    {
        let mut current = state.borrow_mut();
        current.edges = schema.edges;
        current.tables = schema.tables;
    }

    window.set_tables_text(tables_text.into());
    window.set_schema_options(string_model(schema_names));
    apply_schema_selection(window, &state.borrow().tables, &selected_schema);
    window.set_output_text("".into());
    window.set_status(status.into());
}

fn render_selected_path(
    window: &AppWindow,
    state: &Rc<RefCell<DesktopState>>,
    schema_name: &str,
    source_table: &str,
    target_table: &str,
    format_name: &str,
) {
    let current = state.borrow();

    if current.edges.is_empty() {
        window.set_status("NO_TABLES: load a fixture, SQL file, or DATABASE_URL first".into());
        return;
    }

    let format = match parse_format(format_name) {
        Ok(value) => value,
        Err(error) => {
            window.set_status(format!("{}: {}", error.code, error.message).into());
            return;
        }
    };

    let source_name = qualified_table_name(schema_name, source_table);
    let target_name = qualified_table_name(schema_name, target_table);

    let path = match best_path(&current.edges, &source_name, &target_name) {
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

fn apply_schema_selection(window: &AppWindow, tables: &[TableIdentifier], schema_name: &str) {
    let tables_for_schema = table_options_for_schema(tables, schema_name);
    let source = first_table_name(&tables_for_schema);
    let target = last_table_name(&tables_for_schema);
    let target_index = last_table_index(tables_for_schema.len());

    window.set_selected_schema(schema_name.into());
    window.set_selected_schema_index(schema_index(tables, schema_name));
    window.set_table_options(string_model(tables_for_schema));
    window.set_source_table(source.into());
    window.set_source_table_index(0);
    window.set_target_table(target.into());
    window.set_target_table_index(target_index);
}

fn schema_options(tables: &[TableIdentifier]) -> Vec<String> {
    let mut schemas = BTreeSet::new();

    for table in tables {
        schemas.insert(table.schema.clone());
    }

    let values: Vec<String> = schemas.into_iter().collect();

    if values.is_empty() {
        return vec!["public".to_string()];
    }

    values
}

fn first_schema_name(schemas: &[String]) -> String {
    if let Some(schema) = schemas.first() {
        return schema.clone();
    }

    "public".to_string()
}

fn schema_index(tables: &[TableIdentifier], schema_name: &str) -> i32 {
    let schemas = schema_options(tables);

    for index in 0..schemas.len() {
        if schemas[index] == schema_name {
            return index as i32;
        }
    }

    0
}

fn table_options_for_schema(tables: &[TableIdentifier], schema_name: &str) -> Vec<String> {
    let mut values = Vec::new();

    for table in tables {
        if table.schema == schema_name {
            values.push(table.table.clone());
        }
    }

    values.sort();

    values
}

fn first_table_name(tables: &[String]) -> String {
    if let Some(table) = tables.first() {
        return table.clone();
    }

    String::new()
}

fn last_table_name(tables: &[String]) -> String {
    if let Some(table) = tables.last() {
        return table.clone();
    }

    String::new()
}

fn last_table_index(table_count: usize) -> i32 {
    if table_count == 0 {
        return 0;
    }

    (table_count - 1) as i32
}

fn qualified_table_name(schema_name: &str, table_name: &str) -> String {
    if table_name.contains('.') {
        return table_name.to_string();
    }

    format!("{}.{}", schema_name, table_name)
}

fn string_model(values: Vec<String>) -> ModelRc<SharedString> {
    let shared_values: Vec<SharedString> = values.into_iter().map(SharedString::from).collect();

    ModelRc::new(VecModel::from(shared_values))
}
