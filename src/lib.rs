// Copyright 2019 The Polystem authors.
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

//! # Polystem
//!
//! A collection of common stemming algorithms.

use std::str;

pub trait Stemmer {
    fn stem(word: &str) -> String;
}

pub struct S;

impl Stemmer for S {
    /// A simple stemmer that strips `ies`, `es` and `s` from terms. Dervied
    /// from the s-stemmer in the [Atire](http://atire.org) search engine.
    ///
    /// # Examples
    ///
    /// ```
    /// use polystem::Stemmer;
    ///
    /// let term = "flies";
    /// let stem = polystem::S::stem(&term);
    ///
    /// assert_eq!("fly", stem);
    /// ```
    fn stem(word: &str) -> String {
        let mut stem = word.to_ascii_lowercase();

        if stem.ends_with("ies") {
            stem.truncate(stem.len() - 3);
            stem.push('y');
        } else if stem.ends_with("es") {
            stem.truncate(stem.len() - 2);
        } else if stem.ends_with("s") {
            stem.truncate(stem.len() - 1);
        }

        stem
    }
}

pub struct Porter {
    buf: Vec<u8>,
    k: usize,
    j: usize,
}

impl Porter {
    // Construct new `Porter`.
    //
    // The end index `k` starts counting from 1. The index `j` is a general
    // index used during the stemming process.
    fn new(word: &str) -> Porter {
        Porter {
            buf: word.to_ascii_lowercase().into_bytes(),
            k: word.len(),
            j: 0,
        }
    }

    // Check if byte at index `i` is a consonant or not.
    #[inline]
    fn is_consonant(&self, i: usize) -> bool {
        match self.buf[i] {
            b'a' | b'e' | b'i' | b'o' | b'u' => false,
            b'y' => {
                if 0 == i {
                    true
                } else {
                    !self.is_consonant(i - 1)
                }
            }
            _ => true,
        }
    }

    // Determines if the current stem contains a vowel between indexes
    // `[0, j)`.
    #[inline]
    fn has_vowel(&self) -> bool {
        for i in 0..self.j {
            if !self.is_consonant(i) {
                return true;
            }
        }

        false
    }

    // Returns the number of consonant sequences within `[0, j)`.
    fn count(&self) -> usize {
        let mut i = 0;
        let mut n = 0;
        let j = self.j;
        loop {
            if i >= j {
                return n;
            }
            if !self.is_consonant(i) {
                break;
            }
            i += 1;
        }
        i += 1;
        loop {
            loop {
                if i >= j {
                    return n;
                }
                if self.is_consonant(i) {
                    break;
                }
                i += 1;
            }

            i += 1;
            n += 1;
            loop {
                if i >= j {
                    return n;
                }
                if !self.is_consonant(i) {
                    break;
                }
                i += 1;
            }

            i += 1;
        }
    }

    // Return `true` if `[j, j-1]` contains a double consonant, `false`
    // otherwise.
    #[inline]
    fn double_consonant(&self, index: usize) -> bool {
        if index < 1 || index > self.k - 1 {
            return false;
        }

        if self.buf[index] != self.buf[index - 1] {
            return false;
        }

        self.is_consonant(index)
    }

    // Consonant - vowel - consonant. Returns `true` when the sequence `index -
    // 2`, `index - 1`, `index` is of the form consonant, vowel, consonant. The
    // second consonant in the sequence can not be an 'w', 'x' or 'y'.
    #[inline]
    fn cvc(&self, index: usize) -> bool {
        if index < 2 || index > self.k - 1 {
            return false;
        }

        if !self.is_consonant(index)
            || self.is_consonant(index - 1)
            || !self.is_consonant(index - 2)
        {
            return false;
        }

        match self.buf[index] {
            b'w' | b'x' | b'y' => false,
            _ => true,
        }
    }

    // Return `true` if the current buffer `self.buf` ends with the string `s`
    // and update the index `self.j`. Return false otherwise.
    fn ends_with(&mut self, s: &str) -> bool {
        let end_bytes = s.as_bytes();
        let len = end_bytes.len();

        if len > self.k {
            return false;
        }

        let a: &[u8] = &self.buf[self.k - len..self.k];
        if a != end_bytes {
            return false;
        }

        self.j = self.k - len;
        true
    }

    // Replace characters in `[j, k)` to the string `s` and update `k`.
    #[inline]
    fn replace(&mut self, s: &str) {
        let s_buf = s.as_bytes();
        let len = s.len();

        for i in 0..(len) {
            self.buf[self.j + i] = s_buf[i];
        }
        self.k = self.j + len;
    }

    #[inline]
    fn r(&mut self, s: &str) {
        if self.count() > 0 {
            self.replace(s);
        }
    }

    // Strip plurals and 'ed' or 'ing'.
    //
    // The following are examples of the operations performed:
    //
    // ```notrust
    // caresses  ->  caress
    // ponies    ->  poni
    // ties      ->  ti
    // caress    ->  caress
    // cats      ->  cat
    //
    // feed      ->  feed
    // agreed    ->  agree
    // disabled  ->  disable
    //
    // matting   ->  mat
    // mating    ->  mate
    // meeting   ->  meet
    // milling   ->  mill
    // messing   ->  mess
    //
    // meetings  ->  meet
    // ```
    #[inline]
    fn step1ab(&mut self) {
        if b's' == self.buf[self.k - 1] {
            if self.ends_with("sses") {
                self.k -= 2;
            } else if self.ends_with("ies") {
                self.replace("i");
            } else if b's' != self.buf[self.k - 2] {
                self.k -= 1;
            }
        }

        if self.ends_with("eed") {
            if self.count() > 0 {
                self.k -= 1;
            }
        } else if (self.ends_with("ed") || self.ends_with("ing"))
            && self.has_vowel()
        {
            self.k = self.j;
            if self.ends_with("at") {
                self.replace("ate");
            } else if self.ends_with("bl") {
                self.replace("ble");
            } else if self.ends_with("iz") {
                self.replace("ize");
            } else if self.double_consonant(self.k - 1) {
                self.k -= 1;
                match self.buf[self.k - 1] {
                    b'l' | b's' | b'z' => self.k += 1,
                    _ => (),
                }
            } else if 1 == self.count() && self.cvc(self.k - 1) {
                self.replace("e");
            }
        }
    }

    // Replace a terminal 'y' with an 'i' when there is another vowel in the
    // stem.
    #[inline]
    fn step1c(&mut self) {
        if self.ends_with("y") && self.has_vowel() {
            self.buf[self.k - 1] = b'i';
        }
    }

    // Convert double suffices into their single counterparts. For example the
    // suffix 'ization' becomes 'ize' (also 'ation' becomes 'ate', etc). Note
    // the string before the suffix must have a consonant-vowel-consonant
    // greater than `0`, hence the use of `self.r`.
    #[inline]
    fn step2(&mut self) {
        match self.buf[self.k - 2] {
            b'a' => {
                if self.ends_with("ational") {
                    self.r("ate");
                    return;
                }
                if self.ends_with("tional") {
                    self.r("tion");
                    return;
                }
            }
            b'c' => {
                if self.ends_with("enci") {
                    self.r("ence");
                    return;
                }
                if self.ends_with("anci") {
                    self.r("ance");
                    return;
                }
            }
            b'e' => {
                if self.ends_with("izer") {
                    self.r("ize");
                    return;
                }
            }
            b'l' => {
                if self.ends_with("bli") {
                    self.r("ble");
                    return;
                }
                if self.ends_with("alli") {
                    self.r("al");
                    return;
                }
                if self.ends_with("entli") {
                    self.r("ent");
                    return;
                }
                if self.ends_with("eli") {
                    self.r("e");
                    return;
                }
                if self.ends_with("ousli") {
                    self.r("ous");
                    return;
                }
            }
            b'o' => {
                if self.ends_with("ization") {
                    self.r("ize");
                    return;
                }
                if self.ends_with("ation") {
                    self.r("ate");
                    return;
                }
                if self.ends_with("ator") {
                    self.r("ate");
                    return;
                }
            }
            b's' => {
                if self.ends_with("alism") {
                    self.r("al");
                    return;
                }
                if self.ends_with("iveness") {
                    self.r("ive");
                    return;
                }
                if self.ends_with("fulness") {
                    self.r("ful");
                    return;
                }
                if self.ends_with("ousness") {
                    self.r("ous");
                    return;
                }
            }
            b't' => {
                if self.ends_with("aliti") {
                    self.r("al");
                    return;
                }
                if self.ends_with("iviti") {
                    self.r("ive");
                    return;
                }
                if self.ends_with("biliti") {
                    self.r("ble");
                    return;
                }
            }
            b'g' => {
                if self.ends_with("logi") {
                    self.r("log");
                    return;
                }
            }
            _ => (),
        }
    }

    // Convert double suffices into single form. Similar to `step2`, convert
    // 'ic', 'full', 'ness', etc.
    #[inline]
    fn step3(&mut self) {
        match self.buf[self.k - 1] {
            b'e' => {
                if self.ends_with("icate") {
                    self.r("ic");
                    return;
                }
                if self.ends_with("ative") {
                    self.r("");
                    return;
                }
                if self.ends_with("alize") {
                    self.r("al");
                    return;
                }
            }
            b'i' => {
                if self.ends_with("iciti") {
                    self.r("ic");
                    return;
                }
            }
            b'l' => {
                if self.ends_with("ical") {
                    self.r("ic");
                    return;
                }
                if self.ends_with("ful") {
                    self.r("");
                    return;
                }
            }
            b's' => {
                if self.ends_with("ness") {
                    self.r("");
                    return;
                }
            }
            _ => (),
        }
    }

    // Remove 'ant', 'ence', etc when in the context '<c>vcvc<v>', where 'c' is
    // a consonant and 'v' is a vowel, and '<.>' indicates arbitrary presence.
    #[inline]
    fn step4(&mut self) {
        match self.buf[self.k - 2] {
            b'a' => {
                if self.ends_with("al") {
                    // fall through
                } else {
                    return;
                }
            }
            b'c' => {
                if self.ends_with("ance") {
                    // fall through
                } else if self.ends_with("ence") {
                    // fall through
                } else {
                    return;
                }
            }
            b'e' => {
                if self.ends_with("er") {
                    // fall through
                } else {
                    return;
                }
            }
            b'i' => {
                if self.ends_with("ic") {
                    // fall through
                } else {
                    return;
                }
            }
            b'l' => {
                if self.ends_with("able") {
                    // fall through
                } else if self.ends_with("ible") {
                    // fall through
                } else {
                    return;
                }
            }
            b'n' => {
                if self.ends_with("ant") {
                    // fall through
                } else if self.ends_with("ement") {
                    // fall through
                } else if self.ends_with("ment") {
                    // fall through
                } else if self.ends_with("ent") {
                    // fall through
                } else {
                    return;
                }
            }
            b'o' => {
                if self.ends_with("ion")
                    && (b's' == self.buf[self.j - 1]
                        || b't' == self.buf[self.j - 1])
                {
                    // fall through
                } else if self.ends_with("ou") {
                    // fall through
                } else {
                    return;
                }
            }
            b's' => {
                if self.ends_with("ism") {
                    // fall through
                } else {
                    return;
                }
            }
            b't' => {
                if self.ends_with("ate") {
                    // fall through
                } else if self.ends_with("iti") {
                    // fall through
                } else {
                    return;
                }
            }
            b'u' => {
                if self.ends_with("ous") {
                    // fall through
                } else {
                    return;
                }
            }
            b'v' => {
                if self.ends_with("ive") {
                    // fall through
                } else {
                    return;
                }
            }
            b'z' => {
                if self.ends_with("ize") {
                    // fall through
                } else {
                    return;
                }
            }
            _ => return,
        }

        if self.count() > 1 {
            self.k = self.j;
        }
    }

    // Remove terminal 'e' if `count() > 1`; and map 'll' to 'l' if
    // `count() > 1`.
    #[inline]
    fn step5(&mut self) {
        self.j = self.k;
        if b'e' == self.buf[self.k - 1] {
            let c = self.count();
            if c > 1 || c == 1 && !self.cvc(self.k - 2) {
                self.k -= 1;
            }
        }

        if b'l' == self.buf[self.k - 1]
            && self.double_consonant(self.k - 1)
            && self.count() > 1
        {
            self.k -= 1;
        }
    }

    // Return the resulting stem as a `String`.
    fn _stem(&self) -> String {
        unsafe { str::from_utf8_unchecked(&self.buf[..self.k]).to_owned() }
    }
}

impl Stemmer for Porter {
    /// Porter stemming algorithm.
    ///
    /// This version was derived from the C version published at
    /// [tartarus.org/martin/PorterStemmer][tartarus]
    ///
    /// >Porter, 1980, An algorithm for suffix stripping, Program, Vol. 14,
    /// >No. 3, pp 130-137
    ///
    /// [tartarus]: https://tartarus.org/martin/PorterStemmer/
    fn stem(word: &str) -> String {
        if word.len() > 2 {
            let mut porter = Porter::new(word);
            porter.step1ab();
            porter.step1c();
            porter.step2();
            porter.step3();
            porter.step4();
            porter.step5();

            return porter._stem();
        }

        String::from(word)
    }
}

#[cfg(test)]
mod fixture_test;

#[cfg(test)]
mod tests {
    use super::*;
    use fixture_test::*;

    #[test]
    fn test_s_stem() {
        for (i, _) in S_WORDS.iter().enumerate() {
            let word = S_WORDS[i];
            let expected = S_STEMS[i];

            assert_eq!(S::stem(&word), expected);
        }
    }

    #[test]
    fn test_is_consonant() {
        let p = Porter::new("y");
        assert_eq!(p.is_consonant(0), true);

        let p = Porter::new("ey");
        assert_eq!(p.is_consonant(1), true);

        let p = Porter::new("ly");
        assert_eq!(p.is_consonant(1), false);

        let p = Porter::new("aeiou");
        for (i, _) in p.buf.iter().enumerate() {
            assert_eq!(p.is_consonant(i), false);
        }
        let p = Porter::new("bcdfghjklmnpqrstvwxz");
        for (i, _) in p.buf.iter().enumerate() {
            assert_eq!(p.is_consonant(i), true);
        }
    }

    #[test]
    fn test_has_vowel() {
        let mut p = Porter::new("follow");
        p.j = 2;
        assert_eq!(p.has_vowel(), true);

        let p = Porter::new("fllw");
        assert_eq!(p.has_vowel(), false);
    }

    #[test]
    fn test_count() {
        let p = Porter::new("be");
        assert_eq!(p.count(), 0);

        let mut p = Porter::new("beb");
        p.j = 3;
        assert_eq!(p.count(), 1);

        let mut p = Porter::new("bebebe");
        p.j = 6;
        assert_eq!(p.count(), 2);

        let mut p = Porter::new("bebebebe");
        p.j = 8;
        assert_eq!(p.count(), 3);
    }

    #[test]
    fn test_double_consonant() {
        let p = Porter::new("be");
        assert_eq!(p.double_consonant(0), false);

        let p = Porter::new("bbee");
        assert_eq!(p.double_consonant(1), true);

        let p = Porter::new("bbee");
        assert_eq!(p.double_consonant(2), false);

        let p = Porter::new("bbee");
        assert_eq!(p.double_consonant(3), false);

        let p = Porter::new("bbee");
        assert_eq!(p.double_consonant(4), false);
    }

    #[test]
    fn test_cvc() {
        let p = Porter::new("bab");
        assert_eq!(p.cvc(0), false);

        let p = Porter::new("bab");
        assert_eq!(p.cvc(1), false);

        let p = Porter::new("bab");
        assert_eq!(p.cvc(2), true);

        let p = Porter::new("bab");
        assert_eq!(p.cvc(3), false);

        let p = Porter::new("cave");
        assert_eq!(p.cvc(2), true);

        let p = Porter::new("lov");
        assert_eq!(p.cvc(2), true);

        let p = Porter::new("hop");
        assert_eq!(p.cvc(2), true);

        let p = Porter::new("crim");
        assert_eq!(p.cvc(3), true);

        let p = Porter::new("snow");
        assert_eq!(p.cvc(3), false);

        let p = Porter::new("box");
        assert_eq!(p.cvc(2), false);

        let p = Porter::new("tray");
        assert_eq!(p.cvc(3), false);
    }

    #[test]
    fn test_ends_with() {
        let mut p = Porter::new("session");
        assert_eq!(p.ends_with("ion"), true);

        let mut p = Porter::new("session");
        assert_eq!(p.ends_with("ions"), false);

        let mut p = Porter::new("s");
        assert_eq!(p.ends_with("s"), true);
    }

    #[test]
    fn test_replace() {
        let mut p = Porter::new("session");
        p.j = 4;
        p.replace("bar");
        assert_eq!(p.buf, b"sessbar");
    }

    #[test]
    fn test_porter_stem() {
        for (i, _) in PORTER_WORDS.iter().enumerate() {
            let word = PORTER_WORDS[i];
            let expected = PORTER_STEMS[i];

            assert_eq!(Porter::stem(&word), expected);
        }
    }
}
