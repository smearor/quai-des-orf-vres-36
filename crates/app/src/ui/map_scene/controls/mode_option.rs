//! `ModeOption` - one radio option in an orientation/mode group.

use crate::l10n::LabelKey;

/// One radio option in a mode group: the i18n label key plus the
/// mode the option applies.
#[derive(Debug, Clone, Copy)]
pub struct ModeOption<T> {
    /// The i18n key of the option's label.
    pub label: LabelKey,
    /// The mode the option sets.
    pub mode: T,
}
