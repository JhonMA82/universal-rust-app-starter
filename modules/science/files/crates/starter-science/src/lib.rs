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
/// A scalar reference with named units, explicit provenance and error bounds.
/// Convert both values to the documented unit before comparing.
pub struct ReferenceCase<I> {
    pub name: &'static str,
    pub input: I,
    pub unit: &'static str,
    pub expected: f64,
    pub tolerance: Tolerance,
    pub provenance: &'static str,
}
impl<I> ReferenceCase<I> {
    pub fn verify(&self, actual: f64) -> bool {
        self.tolerance.agrees(actual, self.expected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unit_conversion_matches_known_volume() {
        let side = quantities::Length::new::<length::centimeter>(100.0);
        let v = (side * side * side).get::<volume::cubic_meter>();
        let case = ReferenceCase {
            name: "one meter cube",
            input: side,
            unit: "m³",
            expected: 1.0,
            tolerance: Tolerance::new(1e-12, 1e-12).unwrap(),
            provenance: "100 cm = 1 m; volume = side³",
        };
        assert!(case.verify(v));
        assert!(!case.verify(f64::NAN));
    }
}
