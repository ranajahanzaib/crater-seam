# Rules and counts

The canonical registry is `crates/craterseam/src/rules.rs`; JSON reports include it as `rule_definitions`. Every emitted issue includes its stable ID, severity, title, explanation, original evidence, expected contract, suggested action, displayability, automatic-normalization safety, and record ID/source row or dataset scope.

Critical findings prevent safe placement or violate the explicit source contract: invalid latitude, missing/malformed/nonfinite coordinates, source-domain mismatch, missing coordinate columns, unknown direction, unsupported latitude conversion. Warnings include duplicate IDs, metadata/schema conflicts, domain ambiguity and unconfirmed latitude type. Safe canonical conversion is informational.

Parsing precedes range validation. A malformed or absent cell is not also labeled out of range. Distinct latitude and longitude defects can coexist on a row. `affected_records` counts rows with at least one observed record finding, not finding instances. `rule_counts` includes both record and dataset issues; multiple issues can belong to one row, and dataset findings do not manufacture affected records.

Disposition is exclusive: excluded if unplaceable, otherwise review for duplicate ID, otherwise normalized if numerically converted, otherwise accepted. The UI manual-review metric also includes excluded records, explicitly identified as an overlapping review workload. Placeable totals include duplicate records requiring review. Optional depth nulls do not affect coordinate validity.

Simulation results are a separate comparison over the same rows: included, excluded, normalized, geographically different, incorrectly excluded, and total affected. A row can be affected by an interpretation without having an observed source defect. No simulation issues are injected into the source-data audit.

Source histograms count every parsed finite coordinate, including domain outliers. Latitude has 36 interior 5° buckets plus two outside-±90° buckets. Longitude has 36 interior signed 10° buckets plus two outside-±180° buckets. An unsigned value greater than 180° is represented in the upper overflow bucket; that bucket alone does not imply invalidity under an unsigned source profile.
