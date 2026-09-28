# UI Guide

## Desktop UI

The desktop application is designed to be self-contained. It can load:

- a JSON fixture;
- a PostgreSQL SQL file;
- a PostgreSQL URL;
- the `DATABASE_URL` environment variable.

The user selects the source type, enters a single value, loads the structure, then picks the schema and tables from selectors. The result can be rendered as `equation`, `sql`, `text`, `mermaid`, or `json`.

## Web UI

The web UI uses React and calls the Rust API. It is useful when you want a browser interface or a deployed integration. Unlike the desktop, it requires a server, because the browser cannot open a direct PostgreSQL connection.

## Interface principles

- A single source field, with a dynamic label depending on the selected type.
- Selectors for schemas and tables to avoid typos.
- Equation rendering with a line break at each `->` transition.
- A freely resizable desktop window, with no fixed size.
- No footer or unnecessary decorative text in the main surface.

## Resizing the desktop window

The Slint window no longer sets `width` and `height`: in Slint, setting both of these properties on a `Window` locks the window to a fixed size and the window manager refuses any resize. Only the layout constraints are declared:

- `preferred-width` / `preferred-height`: opening size (1280 x 820).
- `min-width` / `min-height`: minimum usable size (560 x 460).
- Panels use `min-*`, `preferred-*`, and `*-stretch` instead of hard-coded heights and widths.

The layout is responsive:

- above 900 px wide, the Source column stays on the left and the Path and Result panels take the remaining width;
- below 900 px, the interface switches to a single scrolling column so that no control is clipped;
- the Result panel absorbs the extra vertical space, and the Databases and Tables lists show a scrollbar only when their content overflows.

## Recommended flow

1. Choose the source type.
2. Load the structure.
3. Select the schema.
4. Choose the source table and the target table.
5. Choose the output format.
6. Set "Max links": the maximum number of links per path (5 by default, from 1 to 8).
7. Run the path search.

The search displays **all** the paths found, from the simplest to the most
complex: the 1-link paths first, then the 2-link ones, and so on, each numbered
(`Path 1`, `Path 2`, ...). This lets you choose a path that avoids a table not
yet populated at data-entry time. The "Max links" slider limits the maximum
length of the listed paths.

## Current limitations

- The web mode depends on the Rust API.
- The desktop mode does not launch an embedded backend.
- The search is based on declared foreign keys, not on automatic inference from column names.
