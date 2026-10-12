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
- `min-width` / `min-height`: minimum usable size (420 x 380), small enough for a half-screen tile or a short window.
- Panels use `min-*`, `preferred-*`, and `*-stretch` instead of hard-coded heights and widths, with `min-width: 0px` so they can shrink.

The layout is responsive and follows two thresholds declared on the window:

```slint
property <bool> compact: root.width < 900px;
property <bool> wide: root.width >= 1500px;
```

- **Above 900 px**: the Source column stays on the left and the Path and Result panels take the remaining width. The Schema / From table / To table selectors are laid out as columns, and Format / Max links / Find path fit on a single row.
- **Below 900 px** (`compact`): the interface switches to a single scrolling column, and every control of the Path panel moves to its own line with its label on the left. No selector is squeezed below a readable width, even at 420 px.
- **Above 1500 px** (`wide`): the workspace stops stretching. It is centred and capped at 1800 px (`content-max-width`), the Source column widens (up to 460 px) and the typography goes from 13 px to 15 px. The monospace result lines stay readable instead of running across the whole screen.
- Margins follow the same rule (`gutter`: 12 px compact, 16 px normal, 28 px wide).

The Result panel absorbs the extra vertical space, and the Databases and Tables lists show a scrollbar only when their content overflows.

The controls shared by both layouts (table selectors, format selector, max-links box) are defined once in `main.slint` (`TableSelector`, `FormatSelector`, `MaxLinksBox`) so the wide and the narrow layouts cannot drift apart.

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
