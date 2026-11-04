//! `LabelMode` - how station labels orient on the tabletop.

/// Label orientation on the tabletop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LabelMode {
    /// Each label faces the nearest table edge - readable for the
    /// player on that side (tabletop default).
    #[default]
    Radial,
    /// Every label upright in screen space.
    Screen,
}
