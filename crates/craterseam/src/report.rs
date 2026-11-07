use crate::{io::Result, model::*};
use std::{collections::BTreeSet, path::Path};
pub fn csv_bytes(a: &DatasetAudit, normalized: bool) -> Result<Vec<u8>> {
    let mut w = csv::Writer::from_writer(Vec::new());
    if normalized {
        let fields: BTreeSet<_> = a.provenance.schema.iter().map(|s| s.name.clone()).collect();
        let mut headers: Vec<_> = fields.iter().cloned().collect();
        headers.extend(
            [
                "craterseam_source_row",
                "craterseam_canonical_latitude",
                "craterseam_canonical_longitude",
                "craterseam_input_profile",
                "craterseam_output_convention",
                "craterseam_excluded_count",
            ]
            .map(str::to_string),
        );
        w.write_record(headers)?;
        for r in &a.records {
            if let Some(n) = r.normalized {
                let mut row: Vec<_> = fields
                    .iter()
                    .map(|k| r.raw.fields.get(k).map_or(String::new(), value_text))
                    .collect();
                row.extend([r.raw.source_row.to_string(),n.latitude.to_string(),n.longitude.to_string(),serde_json::to_string(&a.profile)?,"positive east; longitude [-180,180); latitude preserved; see profile latitude_type".into(),a.summary.excluded.to_string()]);
                w.write_record(row)?;
            }
        }
    } else {
        w.write_record([
            "record_id",
            "source_row",
            "raw_latitude",
            "raw_longitude",
            "canonical_latitude",
            "canonical_longitude",
            "issue_ids",
            "severity",
            "disposition",
        ])?;
        for r in &a.records {
            if !r.issues.is_empty() {
                w.write_record([
                    r.raw.id.clone(),
                    r.raw.source_row.to_string(),
                    r.raw
                        .fields
                        .get(&a.profile.latitude_column)
                        .map_or(String::new(), value_text),
                    r.raw
                        .fields
                        .get(&a.profile.longitude_column)
                        .map_or(String::new(), value_text),
                    r.normalized
                        .map_or(String::new(), |n| n.latitude.to_string()),
                    r.normalized
                        .map_or(String::new(), |n| n.longitude.to_string()),
                    r.issues
                        .iter()
                        .map(|i| i.rule_id.as_str())
                        .collect::<Vec<_>>()
                        .join("|"),
                    format!(
                        "{:?}",
                        r.issues
                            .iter()
                            .map(|i| i.severity)
                            .max()
                            .unwrap_or(Severity::Informational)
                    ),
                    format!("{:?}", r.disposition),
                ])?;
            }
        }
    }
    w.flush()?;
    Ok(w.into_inner().map_err(|e| e.into_error())?)
}
fn value_text(v: &serde_json::Value) -> String {
    if v.is_null() {
        String::new()
    } else {
        v.as_str().map_or_else(|| v.to_string(), str::to_owned)
    }
}
pub fn write(a: &DatasetAudit, dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    let file = std::fs::File::create(dir.join("audit.json"))?;
    serde_json::to_writer(std::io::BufWriter::new(file), a)?;
    std::fs::write(dir.join("flagged.csv"), csv_bytes(a, false)?)?;
    std::fs::write(dir.join("normalized.csv"), csv_bytes(a, true)?)?;
    std::fs::write(
        dir.join("baseline.json"),
        serde_json::to_vec_pretty(&crate::baseline::baseline(a))?,
    )?;
    std::fs::write(
        dir.join("summary.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"provenance":a.provenance,"profile":a.profile,"summary":a.summary,"schema_findings":a.schema_findings}),
        )?,
    )?;
    Ok(())
}
