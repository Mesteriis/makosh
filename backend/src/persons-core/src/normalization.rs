use crate::PersonsTransitionErrorV1;

pub fn normalize_email_v1(value: &str) -> Result<String, PersonsTransitionErrorV1> {
    let normalized = value.trim().to_lowercase();
    if normalized.is_empty()
        || normalized.len() > 320
        || normalized.chars().any(char::is_control)
        || normalized.matches('@').count() != 1
    {
        return Err(PersonsTransitionErrorV1::InvalidEmail);
    }
    let (local, domain) = normalized
        .split_once('@')
        .ok_or(PersonsTransitionErrorV1::InvalidEmail)?;
    if local.is_empty()
        || domain.is_empty()
        || domain.starts_with('.')
        || domain.ends_with('.')
        || !domain.contains('.')
        || local.chars().any(char::is_whitespace)
        || domain.chars().any(char::is_whitespace)
    {
        return Err(PersonsTransitionErrorV1::InvalidEmail);
    }
    Ok(normalized)
}

pub fn normalize_phone_v1(value: &str) -> Result<String, PersonsTransitionErrorV1> {
    let trimmed = value.trim();
    if !trimmed.starts_with('+') {
        return Err(PersonsTransitionErrorV1::InvalidPhone);
    }
    let digits: String = trimmed
        .chars()
        .skip(1)
        .filter(|value| value.is_ascii_digit())
        .collect();
    if digits.len() < 7
        || digits.len() > 15
        || digits.starts_with('0')
        || trimmed
            .chars()
            .any(|value| !(value.is_ascii_digit() || matches!(value, '+' | ' ' | '-' | '(' | ')')))
    {
        return Err(PersonsTransitionErrorV1::InvalidPhone);
    }
    Ok(format!("+{digits}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization_is_provider_neutral_and_deterministic() {
        assert_eq!(
            normalize_email_v1(" Ada@Example.TEST ").expect("email"),
            "ada@example.test"
        );
        assert_eq!(
            normalize_phone_v1("+34 (910) 000-000").expect("phone"),
            "+34910000000"
        );
        assert!(normalize_email_v1("ada").is_err());
        assert!(normalize_phone_v1("910000000").is_err());
    }
}
