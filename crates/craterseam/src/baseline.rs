use crate::model::*;
use std::collections::{BTreeMap, BTreeSet};
pub fn baseline(a: &DatasetAudit) -> AuditBaseline {
    let mut significant_findings = BTreeMap::new();
    for f in &a.schema_findings {
        if f.severity != Severity::Informational {
            significant_findings.insert(
                format!("dataset:{}:{}", f.rule_id, f.original_value),
                f.severity,
            );
        }
    }
    // ID + raw evidence avoids treating a row reorder as a new coordinate defect.
    for r in &a.records {
        for f in &r.issues {
            if f.severity != Severity::Informational {
                let key = format!(
                    "record:{}:{}:{}:{}",
                    r.raw.id, f.rule_id, f.original_value, f.explanation
                );
                significant_findings.insert(key, f.severity);
            }
        }
    }
    AuditBaseline {
        engine_version: a.engine_version.clone(),
        profile: a.profile.clone(),
        schema: a.provenance.schema.clone(),
        summary: a.summary.clone(),
        significant_findings,
    }
}
pub fn compare(a: &DatasetAudit, b: &AuditBaseline, max_new_critical: usize) -> AuditComparison {
    let current = baseline(a);
    let new_findings: Vec<_> = current
        .significant_findings
        .keys()
        .filter(|k| !b.significant_findings.contains_key(*k))
        .cloned()
        .collect();
    let resolved_findings = b
        .significant_findings
        .keys()
        .filter(|k| !current.significant_findings.contains_key(*k))
        .cloned()
        .collect();
    let new_critical = new_findings
        .iter()
        .filter(|k| current.significant_findings[*k] == Severity::Critical)
        .count();
    let keys: BTreeSet<_> = current
        .summary
        .rule_counts
        .keys()
        .chain(b.summary.rule_counts.keys())
        .cloned()
        .collect();
    let rule_count_changes = keys
        .into_iter()
        .filter_map(|k| {
            let delta = *current.summary.rule_counts.get(&k).unwrap_or(&0) as i64
                - *b.summary.rule_counts.get(&k).unwrap_or(&0) as i64;
            (delta != 0).then_some((k, delta))
        })
        .collect();
    let mut contract_changes = Vec::new();
    if b.engine_version != a.engine_version {
        contract_changes
            .push("Audit engine version changed; explicitly review and accept a baseline.".into());
    }
    if b.profile != a.profile {
        contract_changes.push("Coordinate profile changed; baseline is not comparable.".into());
    }
    for col in &b.schema {
        if !a.provenance.schema.iter().any(|c| {
            c.name == col.name
                && c.physical_type == col.physical_type
                && c.logical_type == col.logical_type
        }) {
            contract_changes.push(format!("Schema column removed or changed: {}", col.name));
        }
    }
    // Count thresholds catch additional occurrences even if evidence identifiers coincide.
    let increased_critical = a.summary.rule_counts.iter().any(|(rule, n)| {
        let critical = a
            .schema_findings
            .iter()
            .chain(a.records.iter().flat_map(|r| r.issues.iter()))
            .any(|i| &i.rule_id == rule && i.severity == Severity::Critical);
        critical && *n > b.summary.rule_counts.get(rule).copied().unwrap_or(0) + max_new_critical
    });
    let passed =
        new_critical <= max_new_critical && !increased_critical && contract_changes.is_empty();
    AuditComparison {
        new_findings,
        resolved_findings,
        new_critical,
        rule_count_changes,
        contract_changes,
        passed,
    }
}
