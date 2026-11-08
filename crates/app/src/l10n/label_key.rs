//! `LabelKey` - the i18n key of a control's label.

/// The i18n message key a menu label resolves through `L10n`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LabelKey(pub &'static str);

impl LabelKey {
    /// The raw message key.
    pub fn as_str(&self) -> &'static str {
        self.0
    }
}
