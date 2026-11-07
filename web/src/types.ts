export type Convention =
  "signed_east" | "unsigned_east" | "signed_west" | "unsigned_west";
export type Coordinate = { latitude: number; longitude: number };
export type Issue = {
  rule_id: string;
  severity: string;
  title: string;
  explanation: string;
  original_value: unknown;
  expected: string;
  suggested_action: string;
  displayable: boolean;
  automatic_normalization_safe: boolean;
};
export type Row = {
  id: string;
  source_row: number;
  parsed: { latitude: number | null; longitude: number | null };
  normalized: Coordinate | null;
  issues: Issue[];
  disposition: string;
  simulation: Coordinate | null;
  affected: boolean;
};
export type Detail = {
  record: {
    raw: { id: string; source_row: number; fields: Record<string, unknown> };
    parsed: Record<string, unknown>;
    normalized: Coordinate | null;
    issues: Issue[];
    disposition: string;
  };
  simulation: Coordinate | null;
  why: string;
  calculation: string | null;
  profile: unknown;
};
export type Range = {
  min: number | null;
  max: number | null;
  nulls: number;
  malformed: number;
  non_finite: number;
};
export type Summary = {
  total: number;
  accepted: number;
  normalized: number;
  review: number;
  excluded: number;
  valid_coordinates: number;
  affected_records: number;
  duplicate_records: number;
  rule_counts: Record<string, number>;
  latitude: Range;
  longitude: Range;
  latitude_histogram: number[];
  longitude_histogram: number[];
};
export type Simulation = {
  included: number;
  excluded: number;
  normalized: number;
  located_differently: number;
  incorrectly_excluded: number;
  requiring_review: number;
  affected: number;
};
export type Metadata = {
  provenance: {
    dataset_name: string;
    source_url: string | null;
    acquired_at: string | null;
    source_revision: string | null;
    sha256: string;
    schema: {
      name: string;
      physical_type: string;
      logical_type: string;
      null_count: number;
    }[];
    row_count: number;
  };
  profile: { name: string; evidence: string; latitude_type: string };
  schema_findings: Issue[];
  engine_version: string;
};
export type Bin = [number, number, number];
export type Point = [number, number, number, boolean];
export type MapData = {
  naive_points: Point[];
  validated_points: Point[];
  naive_count: number;
  validated_count: number;
  naive: Bin[];
  validated: Bin[];
  affected: Bin[];
  filtered_records: number;
  representation: string;
};
export type View = { zoom: number; x: number; y: number };
