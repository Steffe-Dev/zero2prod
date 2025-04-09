use rand::{Rng, distr::Alphanumeric};

const TOKEN_LENGTH: usize = 25;

#[derive(Debug)]
pub struct SubcriptionToken(String);

impl AsRef<str> for SubcriptionToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for SubcriptionToken {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        SubcriptionToken::parse(value)
    }
}

impl SubcriptionToken {
    pub fn generate() -> Self {
        let mut rng = rand::rng();
        SubcriptionToken(
            std::iter::repeat_with(|| rng.sample(Alphanumeric))
                .map(char::from)
                .take(TOKEN_LENGTH)
                .collect(),
        )
    }

    /// Returns an instance of `SubscriberEmail` if the input satisfies all
    /// our validation constraints on subscriber emails.
    /// It panics otherwise.
    pub fn parse(token: String) -> Result<Self, String> {
        if Self::is_valid_token(&token) {
            Ok(Self(token))
        } else {
            Err(format!("{} is not a valid subscriber token.", token))
        }
    }

    fn is_valid_token(token: &str) -> bool {
        token.len() == TOKEN_LENGTH && token.chars().all(|c| c.is_alphanumeric())
    }
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};

    use super::*;

    #[test]
    fn token_generation_is_valid() {
        let token = SubcriptionToken::generate();
        assert!(SubcriptionToken::is_valid_token(token.as_ref()));
    }

    #[test]
    fn generated_tokens_are_different() {
        let token1 = SubcriptionToken::generate();
        let token2 = SubcriptionToken::generate();
        assert_ne!(token1.as_ref(), token2.as_ref());
    }

    #[test]
    fn generated_token_length_is_25() {
        let token = SubcriptionToken::generate();
        assert_eq!(token.as_ref().len(), 25);
    }

    #[test]
    fn valid_token_is_parsed_successfully() {
        let token_str = "VALIDTOKEN12345ABCDE67890".to_string();
        let token = SubcriptionToken::parse(token_str.clone());
        assert_ok!(&token);
        assert_eq!(token.unwrap().as_ref(), token_str);
    }

    #[test]
    fn invalid_token_is_parsed_with_an_error() {
        let token_str = "validtoken12345E67890".to_string();
        let token = SubcriptionToken::parse(token_str.clone());
        assert_err!(token);
    }
}
