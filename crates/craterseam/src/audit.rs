use crate::{coordinates::*, io::Input, model::*};
use serde_json::{Value, json};
use std::collections::BTreeMap;
pub fn issue(
    rule: &str,
    severity: Severity,
    value: Value,
    expected: &str,
    explanation: &str,
) -> AuditIssue {
    AuditIssue {
        rule_id: rule.into(),
        severity,
        title: rule.replace('_', " "),
        explanation: explanation.into(),
        original_value: value,
        expected: expected.into(),
        suggested_action: match rule {
            "NORMALIZATION_APPLIED" => {
                "Use the explicit canonical columns; retain the source values."
            }
            "LATITUDE_OUT_OF_RANGE" => {
                "Exclude from maps and review the derived source record; do not clamp."
            }
            "CONVENTION_METADATA_CONFLICT" | "SCHEMA_TYPE_MISMATCH" => {
                "Review the physical file against the documentation and scientific reference."
            }
            _ => "Review source values and the explicit profile before interpreting this record.",
        }
        .into(),
        displayable: false,
        automatic_normalization_safe: rule == "NORMALIZATION_APPLIED",
        record_id: None,
        source_row: None,
    }
}
fn parse(value: Option<&Value>, axis: &str, issues: &mut Vec<AuditIssue>) -> Option<f64> {
    let Some(v) =
        value.filter(|v| !v.is_null() && !v.as_str().is_some_and(|s| s.trim().is_empty()))
    else {
        issues.push(issue(
            &format!("MISSING_{}", axis.to_uppercase()),
            Severity::Critical,
            Value::Null,
            "finite coordinate",
            &format!("{axis} is absent; the record cannot be geographically placed."),
        ));
        return None;
    };
    let parsed = if let Some(n) = v.as_f64() {
        Some(n)
    } else {
        v.as_str().and_then(|s| s.trim().parse::<f64>().ok())
    };
    match parsed {
        None => {
            issues.push(issue(
                "MALFORMED_COORDINATE",
                Severity::Critical,
                v.clone(),
                "numeric coordinate",
                &format!("{axis} cannot be parsed as a number."),
            ));
            None
        }
        Some(n) if !n.is_finite() => {
            issues.push(issue(
                "NON_FINITE_COORDINATE",
                Severity::Critical,
                v.clone(),
                "finite coordinate",
                &format!("{axis} is NaN or infinity."),
            ));
            None
        }
        other => other,
    }
}
fn optional(v: Option<&Value>) -> Option<f64> {
    v.and_then(|x| x.as_f64().or_else(|| x.as_str()?.parse().ok()))
        .filter(|v| v.is_finite())
}
fn stats(s: &mut RangeStats, v: Option<&Value>) {
    if v.is_none_or(|v| v.is_null() || v.as_str().is_some_and(|v| v.trim().is_empty())) {
        s.nulls += 1;
        return;
    }
    match optional(v) {
        Some(n) => {
            s.min = Some(s.min.map_or(n, |x| x.min(n)));
            s.max = Some(s.max.map_or(n, |x| x.max(n)));
        }
        None => {
            if v.and_then(Value::as_str)
                .and_then(|s| s.parse::<f64>().ok())
                .is_some_and(|n| !n.is_finite())
            {
                s.non_finite += 1;
            } else {
                s.malformed += 1;
            }
        }
    }
}
pub fn audit(input: &Input, p: &CoordinateProfile) -> DatasetAudit {
    let mut summary = AuditSummary {
        total: input.records.len(),
        latitude_histogram: vec![0; 38],
        longitude_histogram: vec![0; 38],
        ..Default::default()
    };
    let mut findings = Vec::new();
    let mut counts = BTreeMap::new();
    let has_id = |r: &RawCraterRecord| {
        r.fields
            .get(&p.id_column)
            .is_some_and(|v| !v.is_null() && !v.as_str().is_some_and(|s| s.trim().is_empty()))
    };
    for r in &input.records {
        if has_id(r) {
            *counts.entry(r.id.clone()).or_insert(0usize) += 1;
        }
    }
    for col in [&p.latitude_column, &p.longitude_column] {
        if !input.provenance.schema.iter().any(|s| &s.name == col) {
            findings.push(issue(
                "MISSING_REQUIRED_COLUMN",
                Severity::Critical,
                json!(col),
                "required coordinate column",
                "The physical input schema lacks a configured coordinate field.",
            ));
        }
    }
    if let Some(doc) = &p.documentation
        && let Some(col) = input
            .provenance
            .schema
            .iter()
            .find(|s| s.name == p.id_column)
        && col.physical_type != "CSV_TEXT"
        && col.physical_type != doc.id_type
    {
        findings.push(issue("SCHEMA_TYPE_MISMATCH",Severity::Warning,json!(col),&doc.id_type,"The documented identifier type differs from the physical Parquet type. IDs are preserved without integer coercion."));
    }
    if p.direction == Direction::Unknown {
        findings.push(issue("AMBIGUOUS_LONGITUDE_DIRECTION",Severity::Critical,json!("unknown"),"explicit east or west direction","Numeric ranges do not establish longitude direction; an explicit source choice is required."));
    }
    if p.latitude_type == LatitudeType::Planetographic {
        findings.push(issue(
            "UNSUPPORTED_LATITUDE_CONVERSION",
            Severity::Critical,
            json!("planetographic"),
            "planetocentric or unconfirmed latitude",
            "A body model is required for conversion; this engine will not fabricate one.",
        ));
    }
    if p.latitude_type == LatitudeType::Unconfirmed {
        findings.push(issue("LATITUDE_TYPE_UNCONFIRMED",Severity::Warning,json!("unconfirmed"),"confirmed latitude metadata","Latitude is preserved as supplied. Map positions assume the supplied latitude is suitable for the lunar texture; no planetographic conversion is performed."));
    }
    let mut records = Vec::with_capacity(input.records.len());
    for raw in &input.records {
        let mut issues = Vec::new();
        let lat = parse(raw.fields.get(&p.latitude_column), "latitude", &mut issues);
        let lon = parse(
            raw.fields.get(&p.longitude_column),
            "longitude",
            &mut issues,
        );
        stats(&mut summary.latitude, raw.fields.get(&p.latitude_column));
        stats(&mut summary.longitude, raw.fields.get(&p.longitude_column));
        if let Some(v) = lat {
            let bin = if v < -90.0 {
                0
            } else if v > 90.0 {
                37
            } else {
                1 + (((v + 90.0) / 5.0).floor() as usize).min(35)
            };
            summary.latitude_histogram[bin] += 1;
            if !(-90.0..=90.0).contains(&v) {
                issues.push(issue("LATITUDE_OUT_OF_RANGE",Severity::Critical,json!(v),"[-90, 90] degrees","This latitude is physically impossible. It is excluded without correction; this finding concerns the derived file, not proof of an error in the original Robbins catalog."));
            }
        }
        if let Some(v) = lon {
            let bin = if v < -180.0 {
                0
            } else if v > 180.0 {
                37
            } else {
                1 + (((v + 180.0) / 10.0).floor() as usize).min(35)
            };
            summary.longitude_histogram[bin] += 1;
            let valid = match p.domain {
                Domain::Signed => (-180.0..=180.0).contains(&v),
                Domain::Unsigned => (0.0..=360.0).contains(&v),
            };
            if !valid {
                issues.push(issue("LONGITUDE_DOMAIN_MISMATCH",Severity::Critical,json!(v),match p.domain{Domain::Signed=>"[-180, 180]",Domain::Unsigned=>"[0, 360]"},"Value violates the selected source-domain contract. Another explicit source profile may permit this value; it is not automatically a physically invalid longitude."));
            }
        }
        if has_id(raw) && counts.get(&raw.id).copied().unwrap_or(0) > 1 {
            issues.push(issue(
                "DUPLICATE_RECORD_ID",
                Severity::Warning,
                json!(raw.id),
                "unique identifier",
                "All duplicate occurrences are retained and addressed by source row.",
            ));
            summary.duplicate_records += 1;
        }
        let normalized = lat.zip(lon).and_then(|(a, b)| normalize(a, b, p));
        let changed =
            normalized.is_some_and(|n| lon.is_some_and(|v| (v - n.longitude).abs() > 1e-12));
        if changed {
            issues.push(issue("NORMALIZATION_APPLIED",Severity::Informational,json!(lon),"[-180, 180), positive east","A valid source longitude was converted using the explicit direction and domain. Equivalent seam representations are the same geographic location."));
        }
        let disposition = if normalized.is_none() {
            Disposition::Excluded
        } else if has_id(raw) && counts.get(&raw.id).copied().unwrap_or(0) > 1 {
            Disposition::Review
        } else if changed {
            Disposition::Normalized
        } else {
            Disposition::Accepted
        };
        match disposition {
            Disposition::Accepted => summary.accepted += 1,
            Disposition::Normalized => summary.normalized += 1,
            Disposition::Review => summary.review += 1,
            Disposition::Excluded => summary.excluded += 1,
        };
        if normalized.is_some() {
            summary.valid_coordinates += 1;
        }
        if !issues.is_empty() {
            summary.affected_records += 1;
        }
        for i in &mut issues {
            i.record_id = Some(raw.id.clone());
            i.source_row = Some(raw.source_row);
            i.displayable = normalized.is_some();
            *summary.rule_counts.entry(i.rule_id.clone()).or_default() += 1;
        }
        records.push(RecordAudit {
            raw: raw.clone(),
            parsed: ParsedCraterRecord {
                latitude: lat,
                longitude: lon,
                diameter_km: p
                    .diameter_column
                    .as_ref()
                    .and_then(|c| optional(raw.fields.get(c))),
                depth_km: p
                    .depth_column
                    .as_ref()
                    .and_then(|c| optional(raw.fields.get(c))),
            },
            normalized,
            issues,
            disposition,
        });
    }
    if let Some(doc) = &p.documentation {
        let mismatch = match doc.longitude_domain {
            Domain::Unsigned => summary.longitude.min.is_some_and(|v| v < 0.0),
            Domain::Signed => summary.longitude.max.is_some_and(|v| v > 180.0),
        };
        if mismatch || doc.longitude_domain != p.domain {
            findings.push(issue("CONVENTION_METADATA_CONFLICT",Severity::Warning,json!({"observed":summary.longitude,"documented":doc.longitude_domain,"selected":p.domain}),"documentation consistent with observed values and scientific source","The published coordinate-domain claim conflicts with the observed range or the explicitly selected scientific profile. Ranges alone do not prove east/west direction."));
        }
    }
    if summary.longitude.min.is_some_and(|v| v >= 0.0)
        && summary.longitude.max.is_some_and(|v| v <= 180.0)
    {
        findings.push(issue("SOURCE_DOMAIN_AMBIGUITY",Severity::Warning,json!(summary.longitude),"explicit profile with evidence","The observed range fits both signed and unsigned domains. Selection must use metadata or a user choice."));
    }
    for f in &findings {
        *summary.rule_counts.entry(f.rule_id.clone()).or_default() += 1;
    }
    DatasetAudit {
        engine_version: VERSION.into(),
        provenance: input.provenance.clone(),
        profile: p.clone(),
        summary,
        schema_findings: findings,
        rule_definitions: crate::rules::definitions(),
        records,
    }
}
