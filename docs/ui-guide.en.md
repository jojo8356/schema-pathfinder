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
- `min-width` / `min-height`: minimum usable size (640 x 610), the smallest window in which the Source column, the fixed-height Path form and the result column fit side by side without being squeezed.
- Panels use `min-*`, `preferred-*`, and `*-stretch` instead of hard-coded heights and widths, with `min-width: 0px` so they can shrink.

The layout adapts to every window size **without ever reading the window width** (see the Slint
constraint below): adaptivity comes only from layout constraints.

- **Small screens**: the window never goes below 640 x 610. The Path panel puts one control per line with its label on the left (`min-width: 88px`), so no selector is squeezed below a readable width, even at the minimum. The Source column shrinks first (`min-width: 240px`).
- **Medium and large screens**: the Source column keeps a comfortable width (`preferred-width: 340px`, `max-width: 440px`) while Path and Result take all leftover width (`horizontal-stretch: 1`). The window/screen size is never read: that would close a Slint binding loop.

The Result panel absorbs the extra vertical space, and the Databases and Tables lists show a scrollbar only when their content overflows.

The shared controls (table selectors, format selector, max-links box) are defined once in `main.slint` (`TableSelector`, `FormatSelector`, `MaxLinksBox`), and there is only one layout: there are no two variants left to keep in step.

### A Slint constraint worth knowing

Slint sizes a resizable window from the *layout info* of its content. Any layout property that reads
the window geometry back therefore closes a loop on that size, which the compiler reports as:

```text
warning: The binding for the property 'compact' is part of a binding loop
(root.compact -> layoutinfo-h -> root.layoutinfo-h -> root.compact).
         This was allowed in previous version of Slint, but is deprecated and may cause panic at runtime
```

Four rules follow from it, locked in by `tests/desktop_ui.test.mjs`:

1. **Never read the window geometry** in `main.slint`: no `root.width`, no `root.height`, no
   `parent.width` / `self.width` - not even in the condition of an `if`. Since Slint 1.10, binding
   loops going through the condition of an `if` are detected: a plain breakpoint boolean is enough to
   close the loop.
2. **Responsiveness comes from layout constraints**: `min-width` / `preferred-width` / `max-width`,
   `horizontal-stretch` / `vertical-stretch`. The only way to
   read `root.width` without a loop would be to pin the window with `width` / `height` - in Slint
   those properties give a fixed, non-resizable size - which this project refuses.
3. **No ternaries**: conditions are written as `if` blocks. A `height: cond ? 48px : 56px;` buries a
   branch in the middle of a line, and that is how window-size branches creep back in. The test
   `expresses conditions with blocks instead of ternary expressions` checks that no `?` survives
   outside comments.

4. **No content layout uses `alignment: stretch`** (the default). With that alignment, Slint
   sizes `*-stretch: 0` children at their **minimum** and, when every factor is 0, shares the
   leftover **equally**: that is what inflated the drop-downs vertically in the first screenshots.
   Content layouts therefore declare `alignment: start` (every row keeps its preferred height), the
   Path panel has a fixed height (`min-height` = `max-height` = 360px), and only the Result panel
   (`vertical-stretch: 1`) or a dedicated spacer absorbs the extra space.

Both are re-checked on every build: the `tests` job in `.github/workflows/build-binaries.yml`
recompiles the desktop UI and fails as soon as the output contains "binding loop".

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
