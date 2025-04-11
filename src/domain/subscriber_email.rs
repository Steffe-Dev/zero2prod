use validator::validate_email;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for SubscriberEmail {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        SubscriberEmail::parse(value)
    }
}
impl std::fmt::Display for SubscriberEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // We just forward the Display implementation of
        // the wrapped String.
        self.0.fmt(f)
    }
}

impl SubscriberEmail {
    /// Returns an instance of `SubscriberEmail` if the input satisfies all
    /// our validation constraints on subscriber emails.
    /// It panics otherwise.
    pub fn parse(email: String) -> Result<Self, String> {
        if validate_email(&email) {
            Ok(Self(email))
        } else {
            Err(format!("{} is not a valid subscriber email.", email))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claims::assert_err;
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;
    use quickcheck::{Arbitrary, Gen};
    use rand::SeedableRng;
    use rand::prelude::StdRng;

    #[derive(Debug, Clone)]
    struct ValidateEmailFixture(pub String);

    impl Arbitrary for ValidateEmailFixture {
        fn arbitrary(g: &mut Gen) -> Self {
            let seed = u64::arbitrary(g);
            let mut rng = StdRng::seed_from_u64(seed);
            let email = SafeEmail().fake_with_rng(&mut rng);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidateEmailFixture) -> bool {
        SubscriberEmail::parse(valid_email.0).is_ok()
    }

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }
}
