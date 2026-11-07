use crate::model::*;
use std::collections::BTreeMap;
pub fn definitions() -> BTreeMap<String, RuleDefinition> {
    [
 ("LATITUDE_OUT_OF_RANGE",Severity::Critical,"Physically impossible latitude outside [-90,90]. Exclude and review; never clamp."),
 ("LONGITUDE_DOMAIN_MISMATCH",Severity::Critical,"Value violates the explicitly declared source-domain contract. A different valid profile is not evidence of a source defect."),
 ("MISSING_LATITUDE",Severity::Critical,"No latitude; record cannot be placed."),
 ("MISSING_LONGITUDE",Severity::Critical,"No longitude; record cannot be placed."),
 ("NON_FINITE_COORDINATE",Severity::Critical,"NaN or infinity cannot be placed."),
 ("MALFORMED_COORDINATE",Severity::Critical,"Source coordinate is not a parseable number."),
 ("AMBIGUOUS_LONGITUDE_DIRECTION",Severity::Critical,"Direction is unknown; numeric ranges cannot establish east or west."),
 ("CONVENTION_METADATA_CONFLICT",Severity::Warning,"Documented domain conflicts with observed values or explicitly selected scientific evidence."),
 ("SCHEMA_TYPE_MISMATCH",Severity::Warning,"Documented identifier type differs from physical input type."),
 ("MISSING_REQUIRED_COLUMN",Severity::Critical,"Configured coordinate column is absent from the physical schema."),
 ("DUPLICATE_RECORD_ID",Severity::Warning,"All occurrences of a repeated nonempty ID are retained and addressed by row."),
 ("NORMALIZATION_APPLIED",Severity::Informational,"A permitted coordinate was converted to canonical signed east. Seam equivalence does not imply motion."),
 ("SOURCE_DOMAIN_AMBIGUITY",Severity::Warning,"Observed numeric range fits multiple domains; use explicit metadata or user choice."),
 ("UNSUPPORTED_LATITUDE_CONVERSION",Severity::Critical,"Planetographic conversion requires an unavailable body model."),
 ("LATITUDE_TYPE_UNCONFIRMED",Severity::Warning,"Latitude is preserved without conversion; its type is not confirmed."),
 ].into_iter().map(|(id,severity,explanation)|(id.into(),RuleDefinition{severity,explanation:explanation.into()})).collect()
}
