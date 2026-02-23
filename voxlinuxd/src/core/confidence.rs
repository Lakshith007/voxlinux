#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    Low,
    Medium,
    High,
}

impl Confidence {
    pub fn min(self, other: Confidence) -> Confidence {
        use Confidence::*;
        match (self, other) {
            (Low, _) | (_, Low) => Low,
            (Medium, _) | (_, Medium) => Medium,
            (High, High) => High,
        }
    }
}
