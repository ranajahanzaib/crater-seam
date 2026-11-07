import assert from "node:assert/strict";
const base = process.env.CRATERSEAM_URL ?? "http://127.0.0.1:8787";
const get = async (path) => {
  const r = await fetch(base + path);
  assert.equal(r.status, 200, path);
  return r.json();
};
const { summary: s, simulation } = await get("/api/summary");
assert.equal(s.total, 384278);
assert.equal(s.valid_coordinates, 384277);
assert.equal(s.excluded, 1);
assert.equal(s.duplicate_records, 0);
assert.equal(s.latitude.min, -105.383);
assert.equal(simulation.incorrectly_excluded, 173937);
assert.equal(simulation.included, 210340);
assert.equal(s.accepted + s.normalized + s.review + s.excluded, s.total);
const metadata = await get("/api/dataset");
assert.equal(
  metadata.provenance.sha256,
  "0b7365cdd8547915c2d79b0261e14b6e66efaa3d6ec12be25aabb352d40acd74",
);
assert.equal(
  metadata.provenance.schema.find((s) => s.name === "crater_id").physical_type,
  "BYTE_ARRAY",
);
const { record } = await get("/api/records/12-1-09489");
assert.equal(record.raw.source_row, 284838);
assert.equal(record.normalized, null);
const map = await get("/api/map");
assert.equal(
  map.validated.reduce((sum, b) => sum + b[2], 0),
  s.valid_coordinates,
);
assert.equal(
  map.naive.reduce((sum, b) => sum + b[2], 0),
  simulation.included,
);
for (const [format, rows] of [
  ["flagged", s.affected_records],
  ["normalized", s.valid_coordinates],
]) {
  const response = await fetch(`${base}/api/report?format=${format}`);
  assert.equal(response.status, 200);
  const csv = await response.text();
  assert.equal(csv.trimEnd().split("\n").length - 1, rows, format);
}
const report = await get("/api/report");
assert.equal(report.records.length, s.total);
assert.equal(
  report.records.filter((r) => r.normalized !== null).length,
  s.valid_coordinates,
);
assert.ok(report.rule_definitions.LATITUDE_OUT_OF_RANGE);
assert.equal(report.provenance.sha256, metadata.provenance.sha256);
const west = await get("/api/summary?importer=signed_west");
assert.ok(west.simulation.located_differently > 0);
assert.equal(west.simulation.incorrectly_excluded, 0);
console.log(
  JSON.stringify(
    {
      status: "passed",
      records: s.total,
      valid: s.valid_coordinates,
      invalid: s.excluded,
      simulated_omissions: simulation.incorrectly_excluded,
      exports: ["json", "flagged", "normalized"],
    },
    null,
    2,
  ),
);
