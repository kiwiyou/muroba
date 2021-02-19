/// Represents a prompt string.
///
/// A prompt is a text which describes what you ask for.
/// It is shown before the console interaction starts.
pub struct Prompt(pub String);

/// Marks the beginning of the user input.
///
/// After a [`BeginInput`] the user input is shown, which includes
/// the text currently being typed and the item chosen by the user.
///
/// Clean-up tasks such as resetting color should be done with [`EndInput`].
/// Therefore, stylers implementing [`BeginInput`] **must** implement [`EndInput`] as well.
pub struct BeginInput;

/// Marks the end of the user input.
///
/// It is responsible for restoring the console state after [`BeginInput`] is displayed.
pub struct EndInput;

/// Represents possible values of [`ConfirmQuery`].
///
/// It contains a default choice.
pub struct ConfirmChoice(pub Option<bool>);

/// Represents an element in a list, which is used by select-like queries.
pub struct ListItem {
    /// The item to be shown.
    pub item: String,
    /// `true` if the selection cursor is on the item.
    pub is_cursor: bool,
    /// `true` if the item is selected.
    pub is_selected: bool,
}
