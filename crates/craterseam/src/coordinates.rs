use crate::model::*;
/// Canonical wrapping only. Source-domain validation must precede this function.
pub fn canonical(longitude: f64) -> f64 {
    let x = if (-180.0..180.0).contains(&longitude) {
        longitude
    } else {
        (longitude + 180.0).rem_euclid(360.0) - 180.0
    };
    if x == 0.0 { 0.0 } else { x }
}
pub fn normalize(
    latitude: f64,
    longitude: f64,
    profile: &CoordinateProfile,
) -> Option<NormalizedCoordinate> {
    if !latitude.is_finite()
        || !(-90.0..=90.0).contains(&latitude)
        || !longitude.is_finite()
        || profile.direction == Direction::Unknown
        || profile.latitude_type == LatitudeType::Planetographic
    {
        return None;
    }
    let valid = match profile.domain {
        Domain::Signed => (-180.0..=180.0).contains(&longitude),
        Domain::Unsigned => (0.0..=360.0).contains(&longitude),
    };
    valid.then(|| NormalizedCoordinate {
        latitude: if latitude == 0.0 { 0.0 } else { latitude },
        longitude: canonical(if profile.direction == Direction::West {
            -longitude
        } else {
            longitude
        }),
    })
}
pub fn same_location(a: NormalizedCoordinate, b: NormalizedCoordinate) -> bool {
    (a.latitude - b.latitude).abs() < 1e-9
        && (a.latitude.abs() == 90.0 || canonical(a.longitude - b.longitude).abs() < 1e-9)
}
