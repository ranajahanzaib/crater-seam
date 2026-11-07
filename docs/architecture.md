# Architecture

One Rust workspace member, `craterseam`, keeps compilation and deployment simple. Modules isolate concerns:

- `model`: strongly typed profiles, raw/parsed/canonical records, issues, summaries and baselines.
- `io`: CSV/physical Parquet ingestion, lossless source cells, SHA-256, revision-pinned acquisition and provenance verification.
- `coordinates`: strict domain/direction validation and canonical conversion.
- `audit` / `rules`: deterministic findings, schema evidence and complete-dataset aggregates.
- `baseline`: evidence identities, new/resolved findings, count changes and threshold policy.
- `report`: stable JSON/CSV exports with explicit source/canonical separation.
- `server`: Axum API, bounded pagination, complete-data density bins, viewport-filtered deterministic markers, nearest-record lookup, static React build.
- `main`: Clap CLI, typed errors, status/exit behavior, tracing.

Raw records are shared by `Arc` between the four audited source assumptions. Parsed results and findings are immutable after startup. APIs cannot select a filesystem path or load a new dataset. Server access is loopback only; no permissive CORS. Query conventions are enumerated; pagination caps at 100, page offsets are saturating, map bins cap at 16,200 per layer, focused marker output caps at 3,000 per map. Source and importer are independent explicit parameters. Filtered totals remain complete, even when marker rendering is sampled.

Density bins are canonical, 2° equirectangular cells. Hover/click queries inspect actual records, not cell centroids. Selection uses a stable source-row key so duplicate IDs remain inspectable. Selection, pan and zoom are shared React state between two Canvas layers; SVG draws graticule, labels and selected target markers. Native controls provide a keyboard route through records independent of the pointer-driven map.

API routes:

| Route                                          | Behavior                                                                 |
| ---------------------------------------------- | ------------------------------------------------------------------------ |
| `/api/health`                                  | Engine status/version                                                    |
| `/api/dataset`                                 | Provenance, active profile, schema findings                              |
| `/api/summary`                                 | Source audit and separate simulated importer comparison                  |
| `/api/conventions`                             | Explicit supported assumptions                                           |
| `/api/map`                                     | Full filtered density plus focused point samples                         |
| `/api/issues`                                  | Searchable record pagination; includes clean rows when unfiltered        |
| `/api/records/:id`                             | First matching ID, or `row:N` for unambiguous source-row lookup          |
| `/api/nearest`                                 | Nearest actual record in a bounded degree tolerance on the selected side |
| `/api/report?format=json\|flagged\|normalized` | Whole source-profile exports                                             |

All data endpoints accept `source=signed_east|unsigned_east|signed_west|unsigned_west`. Simulation-capable endpoints additionally accept `importer` with the same choices (default unsigned east). Record/map filters: `search`, `rule`, `severity`, `disposition`, `affected`. Pagination: zero-based `page`, `limit`. Map viewport: normalized `x`, `y`, `zoom` (1–12). API extraction failures return JSON errors; unknown API routes return JSON 404.

The MVP intentionally uses linear scans and whole-dataset in-memory exports. Large reports allocate memory proportional to this input; this is a local workstation tool, not a hardened public multiuser service or an unbounded ingestion system. There is no scientific crater-position inference in the frontend.
