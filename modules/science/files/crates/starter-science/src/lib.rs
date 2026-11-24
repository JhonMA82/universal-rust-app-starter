#![forbid(unsafe_code)]
pub use uom::si::{f64 as quantities, length, volume};
pub struct Tolerance {
    absolute: f64,
    relative: f64,
}
impl Tolerance {
    pub fn new(absolute: f64, relative: f64) -> Option<Self> {
        (absolute.is_finite() && absolute >= 0.0 && (0.0..=1.0).contains(&relative))
            .then_some(Self { absolute, relative })
    }
    pub fn agrees(&self, actual: f64, expected: f64) -> bool {
        actual.is_finite()
            && expected.is_finite()
            && (actual - expected).abs()
                <= self
                    .absolute
                    .max(self.relative * actual.abs().max(expected.abs()))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unit_conversion_matches_known_volume() {
        let side = quantities::Length::new::<length::centimeter>(100.0);
        let v = (side * side * side).get::<volume::cubic_meter>();
        let tolerance = Tolerance::new(1e-12, 1e-12).unwrap();
        assert!(tolerance.agrees(v, 1.0));
        assert!(!tolerance.agrees(v, f64::NAN));
    }
}
