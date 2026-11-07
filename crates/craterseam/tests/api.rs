use axum::{
    body::{Body, to_bytes},
    http::Request,
};
use craterseam::{io, model::*, server};
use serde_json::Value;
use tower::ServiceExt;
fn app() -> axum::Router {
    let p = CoordinateProfile::default();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/coordinates.csv");
    server::router(
        io::load(&path, &p).unwrap(),
        &p,
        std::path::Path::new("/nonexistent"),
    )
}
async fn get(url: &str) -> (u16, Value) {
    let r = app()
        .oneshot(Request::builder().uri(url).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = r.status().as_u16();
    let b = to_bytes(r.into_body(), 10_000_000).await.unwrap();
    (status, serde_json::from_slice(&b).unwrap())
}
#[tokio::test]
async fn bounded_filters_duplicates_and_invalid_records() {
    let (status, v) = get("/api/issues?limit=1000000").await;
    assert_eq!(status, 200);
    assert_eq!(v["limit"], 100);
    assert_eq!(v["total"], 22);
    let (_, v) = get("/api/issues?rule=LATITUDE_OUT_OF_RANGE").await;
    assert_eq!(v["total"], 3);
    let (_, v) = get("/api/issues?search=duplicate").await;
    assert_eq!(v["total"], 2);
    let (_, v) = get("/api/records/row:10").await;
    assert!(v["record"]["normalized"].is_null());
    assert_eq!(v["record"]["raw"]["fields"]["latitude_deg"], "-105.38");
    let (_, v) = get("/api/records/row:17").await;
    assert_eq!(v["record"]["raw"]["source_row"], 17);
    assert_eq!(get("/api/records/absent").await.0, 404);
    assert_eq!(get("/api/summary?source=invalid").await.0, 400);
}
#[tokio::test]
async fn map_bins_reconcile_and_profiles_change_simulation() {
    let (_, v) = get("/api/map").await;
    let total: usize = v["validated"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b[2].as_u64().unwrap() as usize)
        .sum();
    let (_, s) = get("/api/summary").await;
    assert_eq!(
        total,
        s["summary"]["valid_coordinates"].as_u64().unwrap() as usize
    );
    assert!(s["simulation"]["incorrectly_excluded"].as_u64().unwrap() > 0);
    let (_, s) = get("/api/summary?importer=signed_west").await;
    assert_eq!(s["simulation"]["incorrectly_excluded"], 0);
    assert!(s["simulation"]["located_differently"].as_u64().unwrap() > 0);
    let (_, v) = get("/api/map?zoom=4&x=0.3&y=0.3").await;
    assert!(v["validated_points"].as_array().unwrap().len() <= 3000);
}
#[tokio::test]
async fn malformed_query_returns_json() {
    let (status, v) = get("/api/issues?page=oops").await;
    assert_eq!(status, 400);
    assert!(v["error"].as_str().unwrap().contains("page"));
    assert_eq!(get("/api/not-a-route").await.0, 404);
}
