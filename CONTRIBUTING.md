# Contributing to CraterSeam

Use stable Rust and Node.js 22.12+; see the README for data acquisition and local startup. Keep `PLAN.md` unchanged. Run `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `pnpm --dir web test`, and `pnpm --dir web build` before submitting a change. Start the Rust server to run the Playwright and real-data integration checks.

Coordinate logic belongs in the Rust engine. Preserve raw input, source-row association, explicit conventions and deterministic serialization. Include a focused fixture and test for any coordinate correctness fix. Do not replace missing source evidence with inferred direction or fabricated coordinates.

Baseline updates require explicit review of the comparison and original evidence. Generate candidate baselines in an ignored artifact directory, then copy an accepted candidate into `tests/baselines` in a clearly explained change. Never automatically accept baseline changes in CI.

Do not commit acquired datasets, node_modules, target output, or large audit reports. Cite source and license terms when adding data or imagery. Report defects with the source file hash, profile, engine version, affected record IDs/source rows, and reproduction steps when available.
