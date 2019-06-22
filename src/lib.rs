//! # Polystem
//!
//! A collection of common stemming algorithms.

/// Strips `ies`, `es` and `s` from terms. Based on [Atire](http://atire.org).
///
/// # Examples
///
/// ```
/// let mut term = String::from("bars");
/// let stem = polystem::s_stem(&mut term);
///
/// assert_eq!("bar", stem);
/// ```
pub fn s_stem(term: &mut String) -> &mut String {
    if term.ends_with("ies") {
        term.truncate(term.len() - 3);
        term.push('y');
    } else if term.ends_with("es") {
        term.truncate(term.len() - 2);
    } else if term.ends_with("s") {
        term.truncate(term.len() - 1);
    }

    term
}

#[cfg(test)]
mod fixture_test;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s_stem() {
        for (i, _) in fixture_test::WORDS.iter().enumerate() {
            let word = fixture_test::WORDS[i];
            let mut word = String::from(word);
            let expected = fixture_test::S_STEMMED[i];
            assert_eq!(s_stem(&mut word), expected);
        }
    }


    }
}
