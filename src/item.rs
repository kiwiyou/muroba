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
