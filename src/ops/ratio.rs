use crate::OpError;

pub const MIN_RATIO: f32 = 0.05;
pub const MAX_RATIO: f32 = 0.95;

/// Keeps every area at least `MIN_RATIO` of its parent; rejects non-finite input.
pub(crate) fn clamp_ratio(ratio: f32) -> Result<f32, OpError> {
    if ratio.is_finite() {
        Ok(ratio.clamp(MIN_RATIO, MAX_RATIO))
    } else {
        Err(OpError::BadRatio)
    }
}
