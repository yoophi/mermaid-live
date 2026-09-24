use thiserror::Error;

/// Guards against a client asking for an image far larger than any terminal.
const MAX_DIMENSION_PX: u32 = 8192;
const MIN_SCALE_PERCENT: u32 = 25;
const MAX_SCALE_PERCENT: u32 = 400;

/// The box a chart must fit into, and the pixel density to render it at.
///
/// Sizes are device pixels and already include `scale`, so the logical box is
/// `width_px / scale`. Scale is held as a percentage so two requests for the
/// same image compare and cache identically.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RasterSpec {
    width_px: u32,
    height_px: u32,
    scale_percent: u32,
}

impl RasterSpec {
    pub fn new(width_px: u32, height_px: u32, scale: f64) -> Result<Self, RasterError> {
        if width_px == 0
            || height_px == 0
            || width_px > MAX_DIMENSION_PX
            || height_px > MAX_DIMENSION_PX
        {
            return Err(RasterError::InvalidSize);
        }

        if !scale.is_finite() || scale <= 0.0 {
            return Err(RasterError::InvalidScale);
        }

        let scale_percent = (scale * 100.0).round() as u32;
        if !(MIN_SCALE_PERCENT..=MAX_SCALE_PERCENT).contains(&scale_percent) {
            return Err(RasterError::InvalidScale);
        }

        Ok(Self {
            width_px,
            height_px,
            scale_percent,
        })
    }

    pub fn width_px(&self) -> u32 {
        self.width_px
    }

    pub fn height_px(&self) -> u32 {
        self.height_px
    }

    pub fn scale(&self) -> f64 {
        f64::from(self.scale_percent) / 100.0
    }

    /// Identity of a rendered image: the same chart in the same box at the same
    /// density is the same picture.
    pub fn cache_key(&self, fingerprint: &str) -> String {
        format!(
            "{fingerprint}-{}x{}@{}",
            self.width_px, self.height_px, self.scale_percent
        )
    }
}

#[derive(Debug, Error)]
pub enum RasterError {
    #[error("requested image size is out of range")]
    InvalidSize,
    #[error("requested image scale is out of range")]
    InvalidScale,
    #[error("rendering did not finish in time")]
    Timeout,
    #[error("{0}")]
    Renderer(String),
}

#[cfg(test)]
mod tests {
    use super::{RasterSpec, MAX_DIMENSION_PX};

    #[test]
    fn keeps_scale_as_a_comparable_percentage() {
        let spec = RasterSpec::new(800, 600, 2.0).expect("valid spec");

        assert_eq!(spec.scale(), 2.0);
        assert_eq!(spec.cache_key("abc"), "abc-800x600@200");
    }

    #[test]
    fn same_request_produces_the_same_cache_key() {
        let one = RasterSpec::new(800, 600, 1.5).expect("valid spec");
        let two = RasterSpec::new(800, 600, 1.5).expect("valid spec");

        assert_eq!(one.cache_key("abc"), two.cache_key("abc"));
    }

    #[test]
    fn a_different_box_is_a_different_image() {
        let small = RasterSpec::new(400, 600, 1.0).expect("valid spec");
        let large = RasterSpec::new(800, 600, 1.0).expect("valid spec");

        assert_ne!(small.cache_key("abc"), large.cache_key("abc"));
    }

    #[test]
    fn rejects_sizes_no_terminal_would_ask_for() {
        assert!(RasterSpec::new(0, 600, 1.0).is_err());
        assert!(RasterSpec::new(800, 0, 1.0).is_err());
        assert!(RasterSpec::new(MAX_DIMENSION_PX + 1, 600, 1.0).is_err());
    }

    #[test]
    fn rejects_unusable_scales() {
        assert!(RasterSpec::new(800, 600, 0.0).is_err());
        assert!(RasterSpec::new(800, 600, -1.0).is_err());
        assert!(RasterSpec::new(800, 600, f64::NAN).is_err());
        assert!(RasterSpec::new(800, 600, 1000.0).is_err());
    }
}
