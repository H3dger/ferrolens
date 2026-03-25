# ferrolens

`ferrolens` is a terminal-first review lens for bioinformatics result tables.

It is designed for researchers working over SSH on Linux servers who need to inspect wide TSV/CSV tables, annotated VCF-derived outputs, candidate-region catalogs, and other result files without leaving the terminal.

## Status

`ferrolens` is currently best described as an **early public release / 0.1.x tool**: already useful for real-world review workflows, but still actively improving in ergonomics, compatibility, and large-file strategy.

## What it is good at

- Reviewing TSV/CSV/TXT result tables in a table-first TUI
- Inspecting VCF and `vcf.gz` files through a normalized review interface
- Navigating wide bioinformatics tables with horizontal panning and compact detail view
- Searching, filtering, sorting, hiding columns, and exporting current visible rows
- Working comfortably in SSH-only environments

## Supported input formats

- `csv`
- `tsv`
- `txt` with delimiter detection
- `vcf`
- `vcf.gz`

## Supported themes

- `default`
- `catppuccin` — Mocha semantic mapping with restrained full-palette accents in bars and headers

## Installation

### From crates.io

```bash
cargo install ferrolens
```

### From source

```bash
git clone https://github.com/H3dger/ferrolens.git ferrolens
cd ferrolens
cargo run -- --help
```

## Quick start

Run on a real table:

```bash
ferrolens /path/to/results.tsv
```

Run with Catppuccin Mocha styling:

```bash
ferrolens --theme catppuccin /path/to/results.tsv
```

If you are still running from source before packaging:

```bash
cargo run -- /path/to/results.tsv
cargo run -- --theme catppuccin /path/to/results.tsv
```

## Example workflows

### Review a candidate-region TSV

```bash
ferrolens /path/to/ct_candidate_catalog.refined.tsv
```

Then:

- move with `j/k`
- move focused column with `h/l`
- pan viewport with `[` / `]`
- search with `/`
- filter with `f`
- sort focused column with `S`
- export visible rows with `e`

### Review an annotated VCF-derived file

```bash
ferrolens --theme catppuccin /path/to/example.vcf.gz
```

Use the detail pane to inspect the fuller field content while the table view stays compact and truncated.

## Interactive controls

- `q` quit
- `j` / `k` or `Up` / `Down` move the selected row, or scroll the detail pane when detail focus is active
- `h` / `l` or `Left` / `Right` move table column focus when the table pane is focused
- `[` / `]` horizontally scroll the visible table viewport without changing focused column
- `H` hide the currently focused visible column
- `PageUp` / `PageDown` jump through rows in larger steps, or scroll the detail pane faster when detail focus is active
- `Tab` switch focus between table and detail
- `r` reset search/filter/sort/hidden-column state back to the full baseline view
- `e` export the current visible rows to a new TSV and show the output path in the status bar
- `/` open search input, `Enter` apply, `Esc` cancel
- `f` open filter input prefilled from the focused column type with lightweight categorical value hints
- `S` sort by the currently focused column, toggling ascending/descending
- `s` open manual sort input, `Enter` apply, `Esc` cancel

## Filter and sort syntax

- Search: free text across the current dataset, e.g. `tp53`
- Filter: expressions such as `AF < 0.01`, `FILTER == PASS`, `gene in [TP53, EGFR]`
- Sort: column name for ascending, `AF desc` for descending, or `-AF`

## Screenshots / demo assets

Current screenshot:

![ferrolens screenshot](./screenshot.png)

Planned additions before broader community promotion:

- short GIF of search/filter/sort/export workflow

## Current capabilities

- Parse CLI input paths and theme selection
- Load delimited text and VCF-based datasets into a shared `Dataset` model
- Preserve VCF review context including parsed INFO priorities and raw INFO payload
- Maintain app state for selection, filtering, sorting, hidden columns, and focused columns
- Run a real ratatui event loop with Normal, Search, Filter, and Sort input modes
- Render a strongly table-first layout with capped visible columns, truncated table cells, and fuller detail content
- Use current-column-aware sorting and filtering affordances
- Export current visible rows without mutating the source file

## Read-only guarantee

`ferrolens` does not modify source input files. Export always writes to a separate output path.

## Current limitations

- No BAM pileup / IGV-like visualization yet
- No mouse support or complex modal workflow yet
- Current large-file strategy is still memory-first; the tool is strongest today on small-to-medium result tables rather than massive cohort-scale tables
- CSV/TSV compatibility is already practical, but more real-world edge cases should still be tested before calling the tool mature

## Release positioning

Recommended current public positioning:

- **Version line:** `0.1.x`
- **Label:** early public release / community preview

That framing matches the current reality: genuinely useful, but still actively refining workflow polish, compatibility breadth, and scaling strategy.

## Release page checklist

When publishing on GitHub Releases, the minimum release page should include:

- the version number and short summary from `docs/releases/0.1.0.md`
- the screenshot above
- installation notes for source builds now, plus binary downloads once packaged
- a short known-limitations section pointing to the current large-file and compatibility caveats
