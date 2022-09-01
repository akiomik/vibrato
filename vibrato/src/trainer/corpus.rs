use std::io::{BufRead, BufReader, Read};

use regex::Regex;

use crate::errors::{Result, VibratoError};

/// Representation of a pair of a surface and features.
#[allow(unused)]
pub struct Word {
    surface: String,

    // Since a vector of strings consumes massive memory, a single string is stored and divided as
    // needed.
    features: String,
}

impl Word {
    /// Returns a surface string.
    #[allow(unused)]
    pub fn surface(&self) -> &str {
        &self.surface
    }

    /// Returns a concatenated feature string.
    #[allow(unused)]
    pub fn features(&self) -> &str {
        &self.features
    }

    /// Returns a vector of feature strings.
    #[allow(unused)]
    pub fn features_vec(&self) -> Vec<&str> {
        self.features.split(',').collect()
    }
}

/// Representation of a sentence.
#[allow(unused)]
pub struct Sentence {
    tokens: Vec<Word>,
}

impl Sentence {
    /// Returns a slice of tokens.
    #[allow(unused)]
    pub fn tokens(&self) -> &[Word] {
        &self.tokens
    }
}

/// Representation of a corpus.
#[allow(unused)]
pub struct Corpus {
    sentences: Vec<Sentence>,
}

impl Corpus {
    /// Loads a corpus from the given sink.
    ///
    /// # Arguments
    ///
    /// * `rdr` - A reader of the corpus.
    ///
    /// # Errors
    ///
    /// [`VibratoError`] is returned when an input format is invalid.
    #[allow(unused)]
    pub fn from_reader<R>(rdr: R) -> Result<Self>
    where
        R: Read,
    {
        let buf = BufReader::new(rdr);

        let mut sentences = vec![];
        let mut tokens = vec![];
        for line in buf.lines() {
            let line = line?;
            let mut spl = line.split('\t');
            let surface = spl.next();
            let features = spl.next();
            let rest = spl.next();
            match (surface, features, rest) {
                (Some(surface), Some(features), None) => {
                    tokens.push(Word {
                        surface: surface.to_string(),
                        features: features.to_string(),
                    });
                }
                (Some("EOS"), None, None) => {
                    sentences.push(Sentence { tokens });
                    tokens = vec![];
                }
                _ => {
                    return Err(VibratoError::invalid_format(
                        "rdr",
                        "Each line must be a pair of a surface and features or `EOS`",
                    ))
                }
            }
        }

        Ok(Self { sentences })
    }

    /// Returns a slice of sentences.
    #[allow(unused)]
    pub fn sentences(&self) -> &[Sentence] {
        &self.sentences
    }
}

/// Representation of a dictionary.
#[allow(unused)]
pub struct Dictionary {
    words: Vec<Word>,
}

impl Dictionary {
    /// Loads a dictionary from the given sink.
    ///
    /// # Arguments
    ///
    /// * `rdr` - A reader of the dictionary.
    ///
    /// # Errors
    ///
    /// [`VibratoError`] is returned when an input format is invalid.
    #[allow(unused)]
    pub fn from_reader<R>(rdr: R) -> Result<Self>
    where
        R: Read,
    {
        let buf = BufReader::new(rdr);

        let mut words = vec![];
        let surf_feature_pattern = Regex::new(r"^([^,]*),[^,]+,[^,]+,[^,]+,(.*)$").unwrap();
        for (i, line) in buf.lines().enumerate() {
            let line = line?;
            if let Some(m) = surf_feature_pattern.captures(&line) {
                let surface = m.get(1).unwrap().as_str().to_string();
                let features = m.get(2).unwrap().as_str().to_string();
                words.push(Word { surface, features });
            } else {
                return Err(VibratoError::invalid_format(
                    "rdr",
                    "Invalid dictionary format",
                ));
            }
        }

        Ok(Self { words })
    }

    /// Returns a slice of words.
    #[allow(unused)]
    pub fn words(&self) -> &[Word] {
        &self.words
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_corpus() {
        let corpus_data = "\
トスカーナ\t名詞,トスカーナ
地方\t名詞,チホー
に\t助詞,ニ
行く\t動詞,イク
EOS
火星\t名詞,カセー
猫\t名詞,ネコ
EOS
";

        let corpus = Corpus::from_reader(corpus_data.as_bytes()).unwrap();

        assert_eq!(2, corpus.sentences().len());

        let sentence1 = &corpus.sentences()[0];
        assert_eq!(4, sentence1.tokens().len());

        assert_eq!("トスカーナ", sentence1.tokens()[0].surface());
        assert_eq!("名詞,トスカーナ", sentence1.tokens()[0].features());
        assert_eq!("地方", sentence1.tokens()[1].surface());
        assert_eq!("名詞,チホー", sentence1.tokens()[1].features());
        assert_eq!("に", sentence1.tokens()[2].surface());
        assert_eq!("助詞,ニ", sentence1.tokens()[2].features());
        assert_eq!("行く", sentence1.tokens()[3].surface());
        assert_eq!("動詞,イク", sentence1.tokens()[3].features());

        let sentence2 = &corpus.sentences()[1];
        assert_eq!(2, sentence2.tokens().len());

        assert_eq!("火星", sentence2.tokens()[0].surface());
        assert_eq!("名詞,カセー", sentence2.tokens()[0].features());
        assert_eq!("猫", sentence2.tokens()[1].surface());
        assert_eq!("名詞,ネコ", sentence2.tokens()[1].features());
    }

    #[test]
    fn test_features_vec() {
        let corpus_data = "\
トスカーナ\t名詞,トスカーナ
EOS
";

        let corpus = Corpus::from_reader(corpus_data.as_bytes()).unwrap();

        assert_eq!(
            &["名詞", "トスカーナ"],
            corpus.sentences()[0].tokens()[0].features_vec().as_slice()
        );
    }

    #[test]
    fn test_load_dictionary() {
        let dictionary_data = "\
トスカーナ,1,2,3,名詞,トスカーナ
地方,4,5,6,名詞,チホー
に,7,8,9,助詞,ニ
";

        let dict = Dictionary::from_reader(dictionary_data.as_bytes()).unwrap();

        assert_eq!(3, dict.words().len());

        assert_eq!("トスカーナ", dict.words()[0].surface());
        assert_eq!("名詞,トスカーナ", dict.words()[0].features());
        assert_eq!("地方", dict.words()[1].surface());
        assert_eq!("名詞,チホー", dict.words()[1].features());
        assert_eq!("に", dict.words()[2].surface());
        assert_eq!("助詞,ニ", dict.words()[2].features());
    }
}
