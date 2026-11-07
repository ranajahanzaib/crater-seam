use craterseam::{
    audit::audit,
    baseline::{baseline, compare},
    coordinates::*,
    io,
    model::*,
    report,
};
use proptest::prelude::*;
use std::path::Path;
fn fixture() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/coordinates.csv")
}
fn profile(domain: Domain, direction: Direction) -> CoordinateProfile {
    CoordinateProfile {
        domain,
        direction,
        ..Default::default()
    }
}
#[test]
fn boundaries() {
    let p = profile(Domain::Signed, Direction::East);
    for (x, y) in [
        (170., 170.),
        (-170., -170.),
        (180., -180.),
        (-180., -180.),
        (-0., 0.),
    ] {
        let c = normalize(0., x, &p).unwrap();
        assert!((c.longitude - y).abs() < 1e-12);
        assert!(!c.longitude.is_sign_negative() || y != 0.);
    }
    assert!(normalize(0., 180.0000000001, &p).is_none());
    assert!(normalize(0., -180.0000000001, &p).is_none());
}
#[test]
fn unsigned_and_west() {
    let p = profile(Domain::Unsigned, Direction::East);
    for (x, y) in [
        (350., -10.),
        (270., -90.),
        (360., 0.),
        (0., 0.),
        (359.9, -0.1),
    ] {
        assert!((normalize(0., x, &p).unwrap().longitude - y).abs() < 1e-10)
    }
    assert!(normalize(0., -0.0000001, &p).is_none());
    let p = profile(Domain::Unsigned, Direction::West);
    assert_eq!(normalize(0., 90., &p).unwrap().longitude, -90.);
    assert_eq!(normalize(0., 270., &p).unwrap().longitude, 90.);
}
#[test]
fn invalid_and_polar() {
    let p = CoordinateProfile::default();
    for lat in [-105.38, 95., f64::NAN, f64::INFINITY] {
        assert!(normalize(lat, 0., &p).is_none())
    }
    for lon in [f64::NAN, f64::INFINITY, 999.] {
        assert!(normalize(0., lon, &p).is_none())
    }
    assert!(normalize(90., 180., &p).is_some());
    assert!(same_location(
        normalize(90., 0., &p).unwrap(),
        normalize(90., 90., &p).unwrap()
    ));
    assert!(same_location(
        normalize(0., 180., &p).unwrap(),
        normalize(0., -180., &p).unwrap()
    ));
}
#[test]
fn fixtures_preserve_and_reconcile() {
    let p = CoordinateProfile::default();
    let input = io::load(&fixture(), &p).unwrap();
    let a = audit(&input, &p);
    assert_eq!(a.summary.total, 22);
    assert_eq!(
        a.summary.total,
        a.summary.accepted + a.summary.normalized + a.summary.review + a.summary.excluded
    );
    assert_eq!(a.summary.valid_coordinates + a.summary.excluded, 22);
    assert_eq!(a.summary.duplicate_records, 2);
    assert_eq!(a.records[9].raw.fields["latitude_deg"], "-105.38");
    assert!(a.records[9].normalized.is_none());
    assert_eq!(a.records[21].issues.len(), 2);
    assert_eq!(a.records[13].issues.len(), 1);
    assert_eq!(a.records[15].issues[0].rule_id, "MALFORMED_COORDINATE");
    assert_eq!(
        a.summary.affected_records,
        a.records.iter().filter(|r| !r.issues.is_empty()).count()
    );
    for r in &a.records {
        if let Some(n) = r.normalized {
            assert!((-90.0..=90.0).contains(&n.latitude));
            assert!((-180.0..180.0).contains(&n.longitude));
        }
    }
    let bytes = report::csv_bytes(&a, true).unwrap();
    let mut reader = csv::Reader::from_reader(bytes.as_slice());
    assert!(reader.headers().unwrap().iter().any(|x| x == "extra"));
    assert_eq!(reader.records().count(), a.summary.valid_coordinates);
    let bytes = report::csv_bytes(&a, false).unwrap();
    assert_eq!(
        csv::Reader::from_reader(bytes.as_slice()).records().count(),
        a.summary.affected_records
    );
}
#[test]
fn unknown_direction_and_unsupported_latitude() {
    let mut p = CoordinateProfile::default();
    let input = io::load(&fixture(), &p).unwrap();
    p.direction = Direction::Unknown;
    let a = audit(&input, &p);
    assert_eq!(a.summary.excluded, 22);
    assert!(
        a.schema_findings
            .iter()
            .any(|i| i.rule_id == "AMBIGUOUS_LONGITUDE_DIRECTION")
    );
    p.direction = Direction::East;
    p.latitude_type = LatitudeType::Planetographic;
    assert_eq!(audit(&input, &p).summary.excluded, 22);
}
#[test]
fn missing_schema_and_ambiguity() {
    let t = tempfile::NamedTempFile::with_suffix(".csv").unwrap();
    std::fs::write(t.path(), "crater_id,longitude_deg\na,12\n").unwrap();
    let p = CoordinateProfile::default();
    let a = audit(&io::load(t.path(), &p).unwrap(), &p);
    assert!(
        a.schema_findings
            .iter()
            .any(|f| f.rule_id == "MISSING_REQUIRED_COLUMN")
    );
    assert!(
        a.schema_findings
            .iter()
            .any(|f| f.rule_id == "SOURCE_DOMAIN_AMBIGUITY")
    );
    assert_eq!(a.records.len(), 1);
}
#[test]
fn regression_known_dirty_new_and_resolved() {
    let p = CoordinateProfile::default();
    let input = io::load(&fixture(), &p).unwrap();
    let a = audit(&input, &p);
    let b = baseline(&a);
    assert!(compare(&a, &b, 0).passed);
    let mut modified = a.clone();
    modified.records[0].issues.push(craterseam::audit::issue(
        "LATITUDE_OUT_OF_RANGE",
        Severity::Critical,
        serde_json::json!(100),
        "[-90,90]",
        "new",
    ));
    let c = compare(&modified, &b, 0);
    assert!(!c.passed);
    assert_eq!(c.new_critical, 1);
    assert!(compare(&modified, &b, 1).passed);
    let mut fixed = a.clone();
    fixed.records[9].issues.clear();
    assert!(!compare(&fixed, &b, 0).resolved_findings.is_empty());
    let mut p = a.clone();
    p.profile.direction = Direction::West;
    assert!(!compare(&p, &b, 0).passed);
}
#[test]
fn reorder_does_not_create_new_findings() {
    let p = CoordinateProfile::default();
    let mut input = io::load(&fixture(), &p).unwrap();
    let b = baseline(&audit(&input, &p));
    input.records.reverse();
    assert!(compare(&audit(&input, &p), &b, 0).passed);
}
proptest! {#[test]fn signed_idempotent(lat in -90.0f64..=90.0,lon in -180.0f64..=180.0){let p=CoordinateProfile::default();let n=normalize(lat,lon,&p).unwrap();let n2=normalize(n.latitude,n.longitude,&p).unwrap();prop_assert!((n.longitude-n2.longitude).abs()<1e-12);prop_assert!((-180.0..180.0).contains(&n.longitude));}#[test]fn unsigned_equivalence(lon in 0.0f64..=360.0){let p=profile(Domain::Unsigned,Direction::East);let n=normalize(0.,lon,&p).unwrap();let other = NormalizedCoordinate { latitude: 0., longitude: canonical(lon-360.) }; prop_assert!(same_location(n,other));}}
#[test]
fn physical_schema_conflict_and_negative_under_unsigned() {
    let p = profile(Domain::Unsigned, Direction::East);
    let mut input = io::load(&fixture(), &p).unwrap();
    input.provenance.schema[0].physical_type = "BYTE_ARRAY".into();
    input.provenance.schema[0].logical_type = "Some(String)".into();
    let a = audit(&input, &p);
    assert!(
        a.schema_findings
            .iter()
            .any(|f| f.rule_id == "SCHEMA_TYPE_MISMATCH")
    );
    let row = &a.records[2];
    assert!(
        row.issues
            .iter()
            .any(|i| i.rule_id == "LONGITUDE_DOMAIN_MISMATCH")
    );
    assert_eq!(row.raw.fields["longitude_deg"], "-180");
    assert!(a.records[5].normalized.is_some());
}
#[test]
fn blank_ids_do_not_duplicate_real_row_named_identifiers() {
    let t = tempfile::NamedTempFile::with_suffix(".csv").unwrap();
    std::fs::write(
        t.path(),
        "crater_id,latitude_deg,longitude_deg\n,0,0\nrow:1,0,1\nrow:1,0,2\n",
    )
    .unwrap();
    let p = CoordinateProfile::default();
    let a = audit(&io::load(t.path(), &p).unwrap(), &p);
    assert_eq!(a.summary.duplicate_records, 2);
    assert!(a.records[0].issues.is_empty());
}
#[test]
fn histogram_includes_finite_unsigned_values_and_outliers() {
    let p = profile(Domain::Unsigned, Direction::East);
    let a = audit(&io::load(&fixture(), &p).unwrap(), &p);
    let finite = a
        .records
        .iter()
        .filter(|r| r.parsed.longitude.is_some())
        .count();
    assert_eq!(a.summary.longitude_histogram.iter().sum::<usize>(), finite);
    assert!(a.summary.longitude_histogram[37] > 0);
}
