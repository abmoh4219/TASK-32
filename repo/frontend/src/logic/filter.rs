//! Pure-function filter state used by the knowledge points table.
//! Lives in the frontend crate so it can be unit-tested without WASM.

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KnowledgeFilterState {
    pub chapter: Option<String>,
    pub tags: Vec<String>,
    pub difficulty_min: Option<i64>,
    pub difficulty_max: Option<i64>,
    pub discrimination_min: Option<f64>,
    pub discrimination_max: Option<f64>,
}

impl KnowledgeFilterState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

    pub fn set_difficulty_range(&mut self, min: Option<i64>, max: Option<i64>) {
        self.difficulty_min = min;
        self.difficulty_max = max;
    }

    /// Apply a discrimination band preset (Poor/Acceptable/Good/Excellent),
    /// matching the thresholds documented in CLAUDE.md Open Questions:
    ///   Poor <0.1, Acceptable 0.1–0.3, Good 0.3–0.5, Excellent >0.5
    pub fn apply_discrimination_band(&mut self, band: DiscriminationBandPreset) {
        let (min, max) = match band {
            DiscriminationBandPreset::Poor => (-1.0, 0.1),
            DiscriminationBandPreset::Acceptable => (0.1, 0.3),
            DiscriminationBandPreset::Good => (0.3, 0.5),
            DiscriminationBandPreset::Excellent => (0.5, 1.0001),
        };
        self.discrimination_min = Some(min);
        self.discrimination_max = Some(max);
    }

    pub fn is_empty(&self) -> bool {
        self.chapter.is_none()
            && self.tags.is_empty()
            && self.difficulty_min.is_none()
            && self.difficulty_max.is_none()
            && self.discrimination_min.is_none()
            && self.discrimination_max.is_none()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscriminationBandPreset {
    Poor,
    Acceptable,
    Good,
    Excellent,
}
