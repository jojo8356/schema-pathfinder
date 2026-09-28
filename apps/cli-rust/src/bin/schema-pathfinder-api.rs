use schema_pathfinder::pathfinder_core::{
    discover_postgres_databases, discover_postgres_schema, discover_postgres_schemas,
    load_schema_from_env_or_fixture, load_schema_from_fixture, load_schema_from_sql, parse_format,
    pathfinder_error, ranked_paths_by_complexity, render_paths, PathfinderError, SchemaMetadata,
    DEFAULT_MAX_LINKS,
};
use serde::Deserialize;
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

#[derive(Debug)]
struct ApiConfig {
    bind: String,
    fixture: Option<String>,
    admin_token: Option<String>,
    web_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct SourceRequest {
    #[serde(rename = "sourceKind")]
    source_kind: String,
    #[serde(rename = "sourceValue")]
    source_value: String,
}

#[derive(Debug, Deserialize)]
struct PathRequest {
    #[serde(rename = "sourceTable")]
    source_table: String,
    #[serde(rename = "targetTable")]
    target_table: String,
    format: String,
    #[serde(rename = "sourceKind")]
    source_kind: String,
    #[serde(rename = "sourceValue")]
    source_value: String,
    #[serde(rename = "maxLinks")]
    max_links: Option<usize>,
}

fn main() {
    let config = read_config();
    let result = run_server(config);

    if let Err(error) = result {
        eprintln!("{}: {}", error.code, error.message);
        std::process::exit(1);
    }
}

fn read_config() -> ApiConfig {
    let bind = env::var("SCHEMA_PATHFINDER_BIND").unwrap_or_else(|_| "127.0.0.1:8787".to_string());
    let fixture = env::var("SCHEMA_PATHFINDER_FIXTURE").ok();
    let admin_token = env::var("SCHEMA_PATHFINDER_ADMIN_TOKEN").ok();
    let web_dir = env::var("SCHEMA_PATHFINDER_WEB_DIR")
        .ok()
        .map(PathBuf::from);

    ApiConfig {
        bind,
        fixture,
        admin_token,
        web_dir,
    }
}

fn run_server(config: ApiConfig) -> Result<(), PathfinderError> {
    let server = Server::http(&config.bind).map_err(|error| PathfinderError {
        code: "API_BIND_FAILED",
        message: error.to_string(),
    })?;

    println!("schema-pathfinder-api listening on http://{}", config.bind);

    for request in server.incoming_requests() {
        handle_request(request, &config);
    }

    Ok(())
}

fn handle_request(mut request: Request, config: &ApiConfig) {
    let response = route_request(&mut request, config);
    let _ = request.respond(response);
}

fn route_request(request: &mut Request, config: &ApiConfig) -> Response<std::io::Cursor<Vec<u8>>> {
    if request.method() == &Method::Options {
        return empty_response(StatusCode(204));
    }

    if request.method() == &Method::Get && request.url() == "/api/health" {
        return json_response(StatusCode(200), json!({ "status": "ok" }));
    }

    if request.url().starts_with("/api/") || request.url().starts_with("/admin/") {
        return route_api_request(request, config);
    }

    static_response(request, config)
}

fn route_api_request(
    request: &mut Request,
    config: &ApiConfig,
) -> Response<std::io::Cursor<Vec<u8>>> {
    if is_authorized(request, config) == false {
        return json_response(StatusCode(403), error_json("FORBIDDEN", "Forbidden"));
    }

    if request.method() == &Method::Get && is_tables_route(request.url()) {
        return list_tables_response(config);
    }

    if request.method() == &Method::Post && is_tables_route(request.url()) {
        return source_tables_response(request, config);
    }

    if request.method() == &Method::Post && is_databases_route(request.url()) {
        return source_databases_response(request, config);
    }

    if request.method() == &Method::Post && is_schemas_route(request.url()) {
        return source_schemas_response(request, config);
    }

    if request.method() == &Method::Post && is_path_route(request.url()) {
        return path_response(request, config);
    }

    json_response(StatusCode(404), error_json("NOT_FOUND", "Not found"))
}

fn is_tables_route(url: &str) -> bool {
    url == "/api/tables" || url == "/admin/pathfinder/tables"
}

fn is_databases_route(url: &str) -> bool {
    url == "/api/databases" || url == "/admin/pathfinder/databases"
}

fn is_schemas_route(url: &str) -> bool {
    url == "/api/schemas" || url == "/admin/pathfinder/schemas"
}

fn is_path_route(url: &str) -> bool {
    url == "/api/path" || url == "/admin/pathfinder/path"
}

fn list_tables_response(config: &ApiConfig) -> Response<std::io::Cursor<Vec<u8>>> {
    match load_schema(config) {
        Ok(schema) => json_response(StatusCode(200), json!({ "tables": schema.tables })),
        Err(error) => json_response(StatusCode(500), error_json(error.code, &error.message)),
    }
}

fn source_tables_response(
    request: &mut Request,
    config: &ApiConfig,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let source_request: SourceRequest = match parse_json_body(request) {
        Ok(value) => value,
        Err(error) => {
            return json_response(StatusCode(400), error_json(error.code, &error.message));
        }
    };

    match load_schema_for_source(
        config,
        &source_request.source_kind,
        &source_request.source_value,
    ) {
        Ok(schema) => json_response(StatusCode(200), json!({ "tables": schema.tables })),
        Err(error) => json_response(StatusCode(500), error_json(error.code, &error.message)),
    }
}

fn source_databases_response(
    request: &mut Request,
    config: &ApiConfig,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let source_request: SourceRequest = match parse_json_body(request) {
        Ok(value) => value,
        Err(error) => {
            return json_response(StatusCode(400), error_json(error.code, &error.message));
        }
    };

    match load_postgres_url_for_source(
        config,
        &source_request.source_kind,
        &source_request.source_value,
    ) {
        Ok(database_url) => match discover_postgres_databases(&database_url) {
            Ok(databases) => json_response(StatusCode(200), json!({ "databases": databases })),
            Err(error) => json_response(StatusCode(500), error_json(error.code, &error.message)),
        },
        Err(error) => json_response(StatusCode(400), error_json(error.code, &error.message)),
    }
}

fn source_schemas_response(
    request: &mut Request,
    config: &ApiConfig,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let source_request: SourceRequest = match parse_json_body(request) {
        Ok(value) => value,
        Err(error) => {
            return json_response(StatusCode(400), error_json(error.code, &error.message));
        }
    };

    match load_postgres_url_for_source(
        config,
        &source_request.source_kind,
        &source_request.source_value,
    ) {
        Ok(database_url) => match discover_postgres_schemas(&database_url) {
            Ok(schemas) => json_response(StatusCode(200), json!({ "schemas": schemas })),
            Err(error) => json_response(StatusCode(500), error_json(error.code, &error.message)),
        },
        Err(error) => json_response(StatusCode(400), error_json(error.code, &error.message)),
    }
}

fn path_response(request: &mut Request, config: &ApiConfig) -> Response<std::io::Cursor<Vec<u8>>> {
    let path_request: PathRequest = match parse_json_body(request) {
        Ok(value) => value,
        Err(error) => {
            return json_response(StatusCode(400), error_json(error.code, &error.message));
        }
    };

    let format = match parse_format(&path_request.format) {
        Ok(value) => value,
        Err(error) => {
            return json_response(StatusCode(400), error_json(error.code, &error.message));
        }
    };

    let schema = match load_schema_for_source(
        config,
        &path_request.source_kind,
        &path_request.source_value,
    ) {
        Ok(value) => value,
        Err(error) => {
            return json_response(StatusCode(500), error_json(error.code, &error.message));
        }
    };

    let max_links = path_request.max_links.unwrap_or(DEFAULT_MAX_LINKS);

    match ranked_paths_by_complexity(
        &schema.edges,
        &path_request.source_table,
        &path_request.target_table,
        max_links,
    ) {
        Ok(paths) => {
            let rendered = match render_paths(&paths, format) {
                Ok(value) => value,
                Err(error) => {
                    return json_response(StatusCode(500), error_json(error.code, &error.message));
                }
            };

            json_response(
                StatusCode(200),
                json!({
                    "paths": paths,
                    "rendered": rendered
                }),
            )
        }
        Err(error) => {
            if error.code == "NO_DECLARED_FK_PATH" {
                return json_response(
                    StatusCode(200),
                    json!({
                        "paths": [],
                        "noPathReason": "NO_DECLARED_FK_PATH"
                    }),
                );
            }

            json_response(StatusCode(400), error_json(error.code, &error.message))
        }
    }
}

fn parse_json_body<T: for<'de> Deserialize<'de>>(
    request: &mut Request,
) -> Result<T, PathfinderError> {
    let mut body = String::new();

    request
        .as_reader()
        .read_to_string(&mut body)
        .map_err(|error| PathfinderError {
            code: "REQUEST_READ_FAILED",
            message: error.to_string(),
        })?;

    serde_json::from_str(&body).map_err(|error| PathfinderError {
        code: "REQUEST_JSON_INVALID",
        message: error.to_string(),
    })
}

fn load_schema(config: &ApiConfig) -> Result<SchemaMetadata, PathfinderError> {
    load_schema_from_env_or_fixture(config.fixture.as_deref())
}

fn load_postgres_url_for_source(
    _config: &ApiConfig,
    source_kind: &str,
    source_value: &str,
) -> Result<String, PathfinderError> {
    let trimmed_value = source_value.trim();

    if source_kind == "postgres" {
        if trimmed_value.is_empty() {
            return env::var("DATABASE_URL")
                .map_err(|error| pathfinder_error("DB_CONFIG_MISSING", error));
        }

        return Ok(trimmed_value.to_string());
    }

    if source_kind == "fixture" || source_kind == "sql" {
        return Err(PathfinderError {
            code: "SOURCE_KIND_UNSUPPORTED",
            message: format!(
                "{} source does not provide PostgreSQL server metadata",
                source_kind
            ),
        });
    }

    Err(PathfinderError {
        code: "SOURCE_KIND_UNSUPPORTED",
        message: format!("Unsupported source kind: {}", source_kind),
    })
}

fn load_schema_for_source(
    config: &ApiConfig,
    source_kind: &str,
    source_value: &str,
) -> Result<SchemaMetadata, PathfinderError> {
    let trimmed_value = source_value.trim();

    if source_kind == "fixture" {
        if trimmed_value.is_empty() {
            return load_schema(config);
        }

        return load_schema_from_fixture(trimmed_value);
    }

    if source_kind == "sql" {
        if trimmed_value.is_empty() {
            return Err(PathfinderError {
                code: "SQL_INPUT_MISSING",
                message: "SQL source is missing".to_string(),
            });
        }

        return load_schema_from_sql(trimmed_value);
    }

    if source_kind == "postgres" {
        if trimmed_value.is_empty() {
            return load_schema(config);
        }

        return discover_postgres_schema(trimmed_value);
    }

    Err(PathfinderError {
        code: "SOURCE_KIND_UNSUPPORTED",
        message: format!("Unsupported source kind: {}", source_kind),
    })
}

fn static_response(request: &Request, config: &ApiConfig) -> Response<std::io::Cursor<Vec<u8>>> {
    let Some(web_dir) = config.web_dir.as_ref() else {
        return json_response(StatusCode(404), error_json("NOT_FOUND", "Not found"));
    };

    let path = static_file_path(web_dir, request.url());

    match fs::read(&path) {
        Ok(bytes) => bytes_response(StatusCode(200), bytes, content_type_for_path(&path)),
        Err(_) => match fs::read(web_dir.join("index.html")) {
            Ok(bytes) => bytes_response(StatusCode(200), bytes, "text/html; charset=utf-8"),
            Err(error) => json_response(
                StatusCode(404),
                error_json("STATIC_NOT_FOUND", &error.to_string()),
            ),
        },
    }
}

fn static_file_path(web_dir: &Path, url: &str) -> PathBuf {
    let route = url.split('?').next().unwrap_or("/");
    let relative = route.trim_start_matches('/');
    let mut path = PathBuf::from(web_dir);

    if relative.is_empty() {
        path.push("index.html");
        return path;
    }

    for component in Path::new(relative).components() {
        if let Component::Normal(value) = component {
            path.push(value);
        }
    }

    path
}

fn content_type_for_path(path: &Path) -> &'static str {
    match path.extension().and_then(|value| value.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("json") => "application/json",
        Some("png") => "image/png",
        _ => "application/octet-stream",
    }
}

fn is_authorized(request: &Request, config: &ApiConfig) -> bool {
    if let Some(token) = config.admin_token.as_ref() {
        if header_value(request, "x-schema-pathfinder-admin-token") == Some(token.as_str()) {
            return true;
        }

        return false;
    }

    true
}

fn header_value<'a>(request: &'a Request, name: &str) -> Option<&'a str> {
    for header in request.headers() {
        if header.field.to_string().eq_ignore_ascii_case(name) {
            return Some(header.value.as_str());
        }
    }

    None
}

fn error_json(code: &str, title: &str) -> serde_json::Value {
    json!({
        "error": {
            "code": code,
            "title": title
        }
    })
}

fn empty_response(status: StatusCode) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_data(Vec::new())
        .with_status_code(status)
        .with_header(cors_header())
}

fn json_response(
    status: StatusCode,
    value: serde_json::Value,
) -> Response<std::io::Cursor<Vec<u8>>> {
    bytes_response(status, value.to_string().into_bytes(), "application/json")
}

fn bytes_response(
    status: StatusCode,
    bytes: Vec<u8>,
    content_type: &str,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let header = Header::from_bytes("content-type", content_type).expect("static header is valid");

    Response::from_data(bytes)
        .with_status_code(status)
        .with_header(header)
        .with_header(cors_header())
}

fn cors_header() -> Header {
    Header::from_bytes("access-control-allow-origin", "*").expect("static header is valid")
}
