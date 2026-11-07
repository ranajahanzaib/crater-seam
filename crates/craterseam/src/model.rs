use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    Signed,
    Unsigned,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    East,
    West,
    Unknown,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LatitudeType {
    Planetocentric,
    Unconfirmed,
    Planetographic,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Documentation {
    pub longitude_domain: Domain,
    pub id_type: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoordinateProfile {
    pub name: String,
    pub domain: Domain,
    pub direction: Direction,
    pub latitude_type: LatitudeType,
    pub id_column: String,
    pub latitude_column: String,
    pub longitude_column: String,
    pub diameter_column: Option<String>,
    pub depth_column: Option<String>,
    pub evidence: String,
    pub documentation: Option<Documentation>,
}
impl Default for CoordinateProfile {
    fn default() -> Self {
        serde_json::from_str(include_str!("../../../profiles/robbins.json"))
            .expect("bundled profile")
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawCraterRecord {
    pub id: String,
    pub source_row: usize,
    pub fields: BTreeMap<String, serde_json::Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedCraterRecord {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub diameter_km: Option<f64>,
    pub depth_km: Option<f64>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NormalizedCoordinate {
    pub latitude: f64,
    pub longitude: f64,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Informational,
    Warning,
    Critical,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    Accepted,
    Normalized,
    Review,
    Excluded,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditIssue {
    pub rule_id: String,
    pub severity: Severity,
    pub title: String,
    pub explanation: String,
    pub original_value: serde_json::Value,
    pub expected: String,
    pub suggested_action: String,
    pub displayable: bool,
    pub automatic_normalization_safe: bool,
    pub record_id: Option<String>,
    pub source_row: Option<usize>,
}
pub type SchemaFinding = AuditIssue;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordAudit {
    pub raw: std::sync::Arc<RawCraterRecord>,
    pub parsed: ParsedCraterRecord,
    pub normalized: Option<NormalizedCoordinate>,
    pub issues: Vec<AuditIssue>,
    pub disposition: Disposition,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaColumn {
    pub name: String,
    pub physical_type: String,
    pub logical_type: String,
    #[serde(default)]
    pub null_count: usize,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RangeStats {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub nulls: usize,
    pub malformed: usize,
    pub non_finite: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub dataset_name: String,
    pub source_url: Option<String>,
    pub acquired_at: Option<String>,
    pub source_revision: Option<String>,
    pub sha256: String,
    pub schema: Vec<SchemaColumn>,
    pub row_count: usize,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditSummary {
    pub total: usize,
    pub accepted: usize,
    pub normalized: usize,
    pub review: usize,
    pub excluded: usize,
    pub valid_coordinates: usize,
    pub affected_records: usize,
    pub duplicate_records: usize,
    pub rule_counts: BTreeMap<String, usize>,
    pub latitude: RangeStats,
    pub longitude: RangeStats,
    pub latitude_histogram: Vec<usize>,
    pub longitude_histogram: Vec<usize>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDefinition {
    pub severity: Severity,
    pub explanation: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetAudit {
    pub engine_version: String,
    pub provenance: Provenance,
    pub profile: CoordinateProfile,
    pub summary: AuditSummary,
    pub schema_findings: Vec<SchemaFinding>,
    pub rule_definitions: BTreeMap<String, RuleDefinition>,
    pub records: Vec<RecordAudit>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditBaseline {
    pub engine_version: String,
    pub profile: CoordinateProfile,
    pub schema: Vec<SchemaColumn>,
    pub summary: AuditSummary,
    pub significant_findings: BTreeMap<String, Severity>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditComparison {
    pub new_findings: Vec<String>,
    pub resolved_findings: Vec<String>,
    pub new_critical: usize,
    pub rule_count_changes: BTreeMap<String, i64>,
    pub contract_changes: Vec<String>,
    pub passed: bool,
}
