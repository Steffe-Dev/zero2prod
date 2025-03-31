use unicode_segmentation::UnicodeSegmentation;

pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}

// We don't want to make this public. Then parsing could be bypassed,
// which makes the new-type a bit useless
pub struct SubscriberName(String);

// Ideomatic way to expose an interface to a reference to the data inside
// our new-type
impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl SubscriberName {
    /// Returns an instance of `SubscriberName` if the input satisfies all
    /// our validation constraints on subscriber names.
    /// It panics otherwise.
    pub fn parse(name: String) -> Result<SubscriberName, String> {
        let is_empty_or_whitespace = name.trim().is_empty();

        // A grapheme is defined by the Unicode standard as a "user-perceived"
        // character: `å` is a single grapheme, but it is composed of two characters
        // (`a` and `̊`).
        //
        // `graphemes` returns an iterator over the graphemes in the input `s`.
        // `true` specifies that we want to use the extended grapheme definition set,
        // the recommended one.
        let is_too_long = name.graphemes(true).count() > 256;

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_char = name.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_char {
            return Err(format!("{} is not a valid subscriber name", name));
        }
        Ok(Self(name))
    }
}
