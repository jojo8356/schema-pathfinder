use schema_pathfinder::pathfinder_core::{
    best_path, list_tables_from_edges, load_edges_from_env_or_fixture, parse_format, render_path,
    PathfinderError,
};
use serde::Deserialize;
use serde_json::json;
use std::env;
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

#[derive(Debug)]
struct ApiConfig {
    bind: String,
    fixture: Option<String>,
    admin_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PathRequest {
    #[serde(rename = "sourceTable")]
    source_table: String,
    #[serde(rename = "targetTable")]
    target_table: String,
    format: String,
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

    ApiConfig {
        bind,
        fixture,
        admin_token,
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
    if request.method() == &Method::Get && request.url() == "/api/health" {
        return json_response(StatusCode(200), json!({ "status": "ok" }));
    }

    if is_authorized(request, config) == false {
        return json_response(StatusCode(403), error_json("FORBIDDEN", "Forbidden"));
    }

    if request.method() == &Method::Get && request.url() == "/admin/pathfinder/tables" {
        return list_tables_response(config);
    }

    if request.method() == &Method::Post && request.url() == "/admin/pathfinder/path" {
        return path_response(request, config);
    }

    json_response(StatusCode(404), error_json("NOT_FOUND", "Not found"))
}

fn list_tables_response(config: &ApiConfig) -> Response<std::io::Cursor<Vec<u8>>> {
    match load_edges_from_env_or_fixture(config.fixture.as_deref()) {
        Ok(edges) => json_response(StatusCode(200), json!({ "tables": list_tables_from_edges(&edges) })),
        Err(error) => json_response(StatusCode(500), error_json(error.code, &error.message)),
    }
}

fn path_response(request: &mut Request, config: &ApiConfig) -> Response<std::io::Cursor<Vec<u8>>> {
    let mut body = String::new();

    if let Err(error) = request.as_reader().read_to_string(&mut body) {
        return json_response(StatusCode(400), error_json("REQUEST_READ_FAILED", &error.to_string()));
    }

    let path_request: PathRequest = match serde_json::from_str(&body) {
        Ok(value) => value,
        Err(error) => {
            return json_response(StatusCode(400), error_json("REQUEST_JSON_INVALID", &error.to_string()));
        }
    };

    let format = match parse_format(&path_request.format) {
        Ok(value) => value,
        Err(error) => {
            return json_response(StatusCode(400), error_json(error.code, &error.message));
        }
    };

    let edges = match load_edges_from_env_or_fixture(config.fixture.as_deref()) {
        Ok(value) => value,
        Err(error) => {
            return json_response(StatusCode(500), error_json(error.code, &error.message));
        }
    };

    match best_path(&edges, &path_request.source_table, &path_request.target_table) {
        Ok(path) => {
            let rendered = match render_path(&path, format) {
                Ok(value) => value,
                Err(error) => {
                    return json_response(StatusCode(500), error_json(error.code, &error.message));
                }
            };

            json_response(StatusCode(200), json!({
                "paths": [path],
                "rendered": rendered
            }))
        }
        Err(error) => {
            if error.code == "NO_DECLARED_FK_PATH" {
                return json_response(StatusCode(200), json!({
                    "paths": [],
                    "noPathReason": "NO_DECLARED_FK_PATH"
                }));
            }

            json_response(StatusCode(400), error_json(error.code, &error.message))
        }
    }
}

fn is_authorized(request: &Request, config: &ApiConfig) -> bool {
    if let Some(token) = config.admin_token.as_ref() {
        if header_value(request, "x-schema-pathfinder-admin-token") == Some(token.as_str()) {
            return true;
        }
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

fn json_response(status: StatusCode, value: serde_json::Value) -> Response<std::io::Cursor<Vec<u8>>> {
    let header = Header::from_bytes("content-type", "application/json").expect("static header is valid");

    Response::from_string(value.to_string())
        .with_status_code(status)
        .with_header(header)
}
