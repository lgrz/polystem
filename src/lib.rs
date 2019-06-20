// s-stripping stemmer as per the ATIRE search engine (atire.org).
pub fn stemmer_s(term: &str) -> String {
    let s =
        if term.ends_with("ies") {
            let i = term.rfind("ies").unwrap();
            String::from(&term[0..i]) + "y"
        } else if term.ends_with("es") {
            let i = term.rfind("es").unwrap();
            String::from(&term[0..i])
        } else if term.ends_with("s") {
            let i = term.rfind("s").unwrap();
            String::from(&term[0..i])
        } else {
            String::from(term)
        };

    s
}

#[cfg(test)]
mod tests {
    use super::stemmer_s;
    #[test]
    fn test_s_stemmer() {
        assert_eq!(stemmer_s("flies"), "fly");
        assert_eq!(stemmer_s("blesses"), "bless");
        assert_ne!(stemmer_s("suitcases"), "suitcase");
        assert_ne!(stemmer_s("theres"), "there");
        assert_eq!(stemmer_s("foos"), "foo");
        assert_eq!(stemmer_s("foo"), "foo");
    }
}
