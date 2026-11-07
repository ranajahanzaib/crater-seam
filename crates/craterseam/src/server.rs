use crate::{
    audit::audit,
    coordinates::*,
    io::{Input, Result},
    model::*,
    report,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};
#[derive(Clone)]
pub struct AppState {
    pub audits: Arc<Vec<DatasetAudit>>,
}
#[derive(Default, Deserialize, Clone)]
pub struct Params {
    pub source: Option<String>,
    pub importer: Option<String>,
    pub search: Option<String>,
    pub rule: Option<String>,
    pub severity: Option<String>,
    pub disposition: Option<String>,
    pub affected: Option<bool>,
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub format: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub side: Option<String>,
    pub tolerance: Option<f64>,
    pub zoom: Option<f64>,
    pub x: Option<f64>,
    pub y: Option<f64>,
}
fn index(key: Option<&str>) -> Option<usize> {
    match key.unwrap_or("signed_east") {
        "signed_east" => Some(0),
        "unsigned_east" => Some(1),
        "signed_west" => Some(2),
        "unsigned_west" => Some(3),
        _ => None,
    }
}
fn profiles(p: &CoordinateProfile) -> Vec<CoordinateProfile> {
    [
        (Domain::Signed, Direction::East),
        (Domain::Unsigned, Direction::East),
        (Domain::Signed, Direction::West),
        (Domain::Unsigned, Direction::West),
    ]
    .into_iter()
    .map(|(domain, direction)| {
        let mut p = p.clone();
        p.domain = domain;
        p.direction = direction;
        p.name = format!("{domain:?} {direction:?} (explicit selection)");
        p
    })
    .collect()
}
fn get_audit<'a>(
    s: &'a AppState,
    p: &Params,
) -> std::result::Result<&'a DatasetAudit, &'static str> {
    index(p.source.as_deref())
        .map(|i| &s.audits[i])
        .ok_or("Unknown source convention")
}
fn error(s: &str) -> Response {
    (StatusCode::BAD_REQUEST, Json(json!({"error":s}))).into_response()
}
fn importer(p: &Params, a: &DatasetAudit) -> std::result::Result<CoordinateProfile, &'static str> {
    index(p.importer.as_deref().or(Some("unsigned_east")))
        .map(|i| profiles(&a.profile)[i].clone())
        .ok_or("Unknown importer convention")
}
fn simulated(r: &RecordAudit, p: &CoordinateProfile) -> Option<NormalizedCoordinate> {
    r.parsed
        .latitude
        .zip(r.parsed.longitude)
        .and_then(|(a, b)| normalize(a, b, p))
}
fn affected(r: &RecordAudit, p: &CoordinateProfile) -> bool {
    match (r.normalized, simulated(r, p)) {
        (Some(a), Some(b)) => !same_location(a, b),
        (None, None) => false,
        _ => true,
    }
}
fn matches(r: &RecordAudit, p: &Params, imp: &CoordinateProfile) -> bool {
    p.search
        .as_ref()
        .is_none_or(|s| r.raw.id.to_lowercase().contains(&s.to_lowercase()))
        && p.rule
            .as_ref()
            .is_none_or(|rule| rule.is_empty() || r.issues.iter().any(|i| &i.rule_id == rule))
        && p.severity.as_ref().is_none_or(|sev| {
            sev.is_empty()
                || r.issues
                    .iter()
                    .any(|i| format!("{:?}", i.severity).to_lowercase() == *sev)
        })
        && p.disposition
            .as_ref()
            .is_none_or(|d| d.is_empty() || format!("{:?}", r.disposition).to_lowercase() == *d)
        && (!p.affected.unwrap_or(false) || affected(r, imp))
}
async fn json_rejections(response: Response) -> Response {
    if matches!(response.status().as_u16(), 400 | 422)
        && !response
            .headers()
            .get(header::CONTENT_TYPE)
            .is_some_and(|v| v.as_bytes().starts_with(b"application/json"))
    {
        let bytes = axum::body::to_bytes(response.into_body(), 8192)
            .await
            .unwrap_or_default();
        return error(&String::from_utf8_lossy(&bytes));
    }
    response
}
async fn unknown_api() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({"error":"Unknown API endpoint"})),
    )
        .into_response()
}
async fn health() -> Json<Value> {
    Json(json!({"status":"ok","engine_version":VERSION}))
}
async fn conventions() -> Json<Value> {
    Json(
        json!({"conventions":["signed_east","unsigned_east","signed_west","unsigned_west"],"canonical":"positive east, [-180,180)","direction":"explicit; never inferred from ranges"}),
    )
}
async fn dataset(State(s): State<AppState>, Query(p): Query<Params>) -> Response {
    let a = match get_audit(&s, &p) {
        Ok(a) => a,
        Err(e) => return error(e),
    };
    Json(json!({"provenance":a.provenance,"profile":a.profile,"schema_findings":a.schema_findings,"engine_version":VERSION})).into_response()
}
fn comparison(a: &DatasetAudit, imp: &CoordinateProfile) -> Value {
    let (mut included, mut excluded, mut changed, mut moved, mut restored, mut review) =
        (0, 0, 0, 0, 0, 0);
    for r in &a.records {
        let n = simulated(r, imp);
        if let Some(n) = n {
            included += 1;
            if r.parsed
                .longitude
                .is_some_and(|v| (v - n.longitude).abs() > 1e-12)
            {
                changed += 1;
            }
        } else {
            excluded += 1;
        }
        match (r.normalized, n) {
            (Some(x), Some(y)) if !same_location(x, y) => moved += 1,
            (Some(_), None) => restored += 1,
            _ => {}
        }
        if r.disposition == Disposition::Review || r.disposition == Disposition::Excluded {
            review += 1;
        }
    }
    json!({"included":included,"excluded":excluded,"normalized":changed,"located_differently":moved,"incorrectly_excluded":restored,"requiring_review":review,"affected":a.records.iter().filter(|r|affected(r,imp)).count(),"label":"Simulated consequence"})
}
async fn summary(State(s): State<AppState>, Query(p): Query<Params>) -> Response {
    let a = match get_audit(&s, &p) {
        Ok(a) => a,
        Err(e) => return error(e),
    };
    let imp = match importer(&p, a) {
        Ok(p) => p,
        Err(e) => return error(e),
    };
    Json(json!({"summary":a.summary,"simulation":comparison(a,&imp)})).into_response()
}
async fn issues(State(s): State<AppState>, Query(p): Query<Params>) -> Response {
    let a = match get_audit(&s, &p) {
        Ok(a) => a,
        Err(e) => return error(e),
    };
    let imp = match importer(&p, a) {
        Ok(p) => p,
        Err(e) => return error(e),
    };
    let limit = p.limit.unwrap_or(25).clamp(1, 100);
    let page = p.page.unwrap_or(0).min(1_000_000);
    let skip = page.saturating_mul(limit);
    let mut total = 0;
    let mut rows = Vec::new();
    for r in &a.records {
        if matches(r, &p, &imp) {
            if total >= skip && rows.len() < limit {
                rows.push(json!({"id":r.raw.id,"source_row":r.raw.source_row,"parsed":r.parsed,"normalized":r.normalized,"issues":r.issues,"disposition":r.disposition,"simulation":simulated(r,&imp),"affected":affected(r,&imp)}));
            }
            total += 1;
        }
    }
    Json(json!({"total":total,"page":page,"limit":limit,"rows":rows})).into_response()
}
fn record_json(r: &RecordAudit, a: &DatasetAudit, imp: &CoordinateProfile) -> Value {
    let n = simulated(r, imp);
    let why = match (r.normalized, n) {
        (Some(x), Some(y)) if !same_location(x, y) => {
            "Simulated consequence: the explicit longitude-direction assumption places this point at a different longitude."
        }
        (Some(_), None) => {
            "Simulated consequence: the strict importer rejects this source value. The validated coordinate is placeable and has not necessarily moved."
        }
        (None, _) => {
            "Excluded by the selected source profile; see observed findings. No validated map position is fabricated."
        }
        _ => {
            if r.issues
                .iter()
                .any(|i| i.rule_id == "NORMALIZATION_APPLIED")
            {
                "A valid coordinate was converted to canonical representation. Seam wrapping alone does not move its geographic location."
            } else {
                "Source and canonical coordinates are numerically identical. This point has not moved."
            }
        }
    };
    json!({"record":r,"profile":a.profile,"simulation":n,"why":why,"calculation":r.parsed.longitude.map(|v|format!("east = {} × {v}; canonical = (east + 180) rem_euclid 360 - 180",if a.profile.direction==Direction::West{-1}else{1}))})
}
async fn record(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Query(p): Query<Params>,
) -> Response {
    let a = match get_audit(&s, &p) {
        Ok(a) => a,
        Err(e) => return error(e),
    };
    let imp = match importer(&p, a) {
        Ok(p) => p,
        Err(e) => return error(e),
    };
    let r = if let Some(row) = id
        .strip_prefix("row:")
        .and_then(|v| v.parse::<usize>().ok())
    {
        row.checked_sub(1).and_then(|i| a.records.get(i))
    } else {
        a.records.iter().find(|r| r.raw.id == id)
    };
    match r {
        Some(r) => Json(record_json(r, a, &imp)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error":"Record not found"})),
        )
            .into_response(),
    }
}
async fn nearest(State(s): State<AppState>, Query(p): Query<Params>) -> Response {
    let a = match get_audit(&s, &p) {
        Ok(a) => a,
        Err(e) => return error(e),
    };
    let imp = match importer(&p, a) {
        Ok(p) => p,
        Err(e) => return error(e),
    };
    let (Some(lat), Some(lon)) = (p.lat, p.lon) else {
        return error("lat and lon are required");
    };
    if !lat.is_finite()
        || !lon.is_finite()
        || !(-90.0..=90.0).contains(&lat)
        || !(-180.0..=180.0).contains(&lon)
    {
        return error("Invalid map coordinate");
    }
    let mut best = None;
    let mut distance = p.tolerance.unwrap_or(4.0).clamp(0.001, 15.0).powi(2);
    for r in &a.records {
        if !matches(r, &p, &imp) {
            continue;
        }
        let n = if p.side.as_deref() == Some("naive") {
            simulated(r, &imp)
        } else {
            r.normalized
        };
        if let Some(n) = n {
            let d = (n.latitude - lat).powi(2) + canonical(n.longitude - lon).powi(2);
            if d < distance {
                distance = d;
                best = Some(r);
            }
        }
    }
    Json(best.map(|r| record_json(r, a, &imp))).into_response()
}
async fn map(State(s): State<AppState>, Query(p): Query<Params>) -> Response {
    let a = match get_audit(&s, &p) {
        Ok(a) => a,
        Err(e) => return error(e),
    };
    let imp = match importer(&p, a) {
        Ok(p) => p,
        Err(e) => return error(e),
    };
    let mut naive = vec![0usize; 180 * 90];
    let mut validated = vec![0usize; 180 * 90];
    let mut omitted = vec![0usize; 180 * 90];
    let mut count = 0;
    let zoom = p.zoom.unwrap_or(1.0).clamp(1.0, 12.0);
    let x = p.x.unwrap_or(0.0).clamp(0.0, 1.0 - 1.0 / zoom);
    let y = p.y.unwrap_or(0.0).clamp(0.0, 1.0 - 1.0 / zoom);
    if !zoom.is_finite() || !x.is_finite() || !y.is_finite() {
        return error("Viewport must be finite");
    }
    let visible = |c: NormalizedCoordinate| {
        let cx = (c.longitude + 180.0) / 360.0;
        let cy = (90.0 - c.latitude) / 180.0;
        cx >= x && cx <= x + 1.0 / zoom && cy >= y && cy <= y + 1.0 / zoom
    };
    let mut naive_points = Vec::new();
    let mut validated_points = Vec::new();
    let bin = |n: NormalizedCoordinate| {
        let x = (((n.longitude + 180.0) / 2.0).floor() as usize).min(179);
        let y = (((90.0 - n.latitude) / 2.0).floor() as usize).min(89);
        y * 180 + x
    };
    for r in &a.records {
        if !matches(r, &p, &imp) {
            continue;
        }
        count += 1;
        let n = simulated(r, &imp);
        if let Some(n) = n {
            naive[bin(n)] += 1;
            if zoom > 2.0 && visible(n) {
                naive_points.push(json!([
                    n.longitude,
                    n.latitude,
                    r.raw.source_row,
                    affected(r, &imp)
                ]));
            }
        }
        if let Some(v) = r.normalized {
            validated[bin(v)] += 1;
            if zoom > 2.0 && visible(v) {
                validated_points.push(json!([
                    v.longitude,
                    v.latitude,
                    r.raw.source_row,
                    affected(r, &imp)
                ]));
            }
            if affected(r, &imp) {
                omitted[bin(v)] += 1;
            }
        }
    }
    let compact = |bins: Vec<usize>| {
        bins.into_iter()
            .enumerate()
            .filter(|(_, v)| *v > 0)
            .map(|(i, n)| json!([i % 180, i / 180, n]))
            .collect::<Vec<_>>()
    };
    let sample = |v: Vec<Value>| {
        let step = v.len().div_ceil(3000).max(1);
        v.into_iter().step_by(step).take(3000).collect::<Vec<_>>()
    };
    let naive_count: usize = naive.iter().sum();
    let validated_count: usize = validated.iter().sum();
    Json(json!({"naive_points":sample(naive_points),"validated_points":sample(validated_points),"naive_count":naive_count,"validated_count":validated_count,"columns":180,"rows":90,"bin_degrees":2,"filtered_records":count,"naive":compact(naive),"validated":compact(validated),"affected":compact(omitted),"representation":"Complete filtered data aggregated into 2 degree cells; cells are not individual crater locations"})).into_response()
}
async fn report_endpoint(State(s): State<AppState>, Query(p): Query<Params>) -> Response {
    let a = match get_audit(&s, &p) {
        Ok(a) => a,
        Err(e) => return error(e),
    };
    let (format, name, bytes) = match p.format.as_deref().unwrap_or("json") {
        "json" => (
            "application/json",
            "audit.json",
            serde_json::to_vec(a).map_err(|e| e.to_string()),
        ),
        "flagged" => (
            "text/csv",
            "flagged.csv",
            report::csv_bytes(a, false).map_err(|e| e.to_string()),
        ),
        "normalized" => (
            "text/csv",
            "normalized.csv",
            report::csv_bytes(a, true).map_err(|e| e.to_string()),
        ),
        _ => return error("format must be json, flagged or normalized"),
    };
    match bytes {
        Ok(b) => (
            [
                (header::CONTENT_TYPE, format),
                (
                    header::CONTENT_DISPOSITION,
                    &format!("attachment; filename={name}"),
                ),
            ],
            b,
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":e}))).into_response(),
    }
}
pub fn router(input: Input, p: &CoordinateProfile, web: &std::path::Path) -> Router {
    let audits = profiles(p).iter().map(|p| audit(&input, p)).collect();
    let state = AppState {
        audits: Arc::new(audits),
    };
    Router::new()
        .route("/api/health", get(health))
        .route("/api/{*rest}", get(unknown_api))
        .route("/api/conventions", get(conventions))
        .route("/api/dataset", get(dataset))
        .route("/api/summary", get(summary))
        .route("/api/map", get(map))
        .route("/api/issues", get(issues))
        .route("/api/nearest", get(nearest))
        .route("/api/records/{id}", get(record))
        .route("/api/report", get(report_endpoint))
        .layer(axum::middleware::map_response(json_rejections))
        .with_state(state)
        .fallback_service(
            ServeDir::new(web).not_found_service(ServeFile::new(web.join("index.html"))),
        )
}
pub async fn serve(
    input: Input,
    p: CoordinateProfile,
    port: u16,
    web: std::path::PathBuf,
) -> Result<()> {
    let router = router(input, &p, &web);
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)).await?;
    tracing::info!(port, "CraterSeam ready at http://127.0.0.1:{port}");
    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
