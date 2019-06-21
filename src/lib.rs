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
mod tests {
    use super::*;

    #[test]
    fn test_s_stemmer() {
        let mut s = String::from("flies");
        assert_eq!(s_stem(&mut s), "fly");

        let mut s = String::from("blesses");
        assert_eq!(s_stem(&mut s), "bless");

        let mut s = String::from("suitcases");
        assert_eq!(s_stem(&mut s), "suitcas");

        let mut s = String::from("theres");
        assert_eq!(s_stem(&mut s), "ther");

        let mut s = String::from("suns");
        assert_eq!(s_stem(&mut s), "sun");
    }
}
