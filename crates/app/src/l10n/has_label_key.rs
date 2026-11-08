//! `HasLabelKey` - an enum whose variants carry menu-label i18n
//! keys.

use crate::l10n::LabelKey;

/// An enum whose variants name their i18n menu label - the
/// orientation and mode radios resolve it through `L10n`.
pub trait HasLabelKey {
    /// The i18n key of the variant's menu label.
    fn label_key(&self) -> LabelKey;
}
