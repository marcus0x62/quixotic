// Copyright 2024 Marcus Butler <marcusb@marcusb.org>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the “Software”), to
// deal in the Software without restriction, including without limitation the
// rights to use, copy, modify, merge, publish, distribute, sublicense, and/or
// sell copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use std::{cmp::PartialEq, collections::HashMap, fmt::Display, fs::read_to_string, hash::Hash, io};

use rand::Rng;
use walkdir::WalkDir;

use crate::rcdom::tokenize_html;

pub struct MarkovIterator<T> {
    current_token: Option<T>,
    chain: HashMap<T, Vec<T>>,
}

impl<T: Clone + Default + Display + Eq + Hash + PartialEq> MarkovIterator<T> {
    pub fn new(tokens: impl Iterator<Item = T>) -> Result<MarkovIterator<T>, io::Error> {
        let mut chain = HashMap::<T, Vec<T>>::new();

        let mut last = T::default();
        for (i, token) in tokens.enumerate() {
            if i == 0 {
                last = token;
                continue;
            }

            if let Some(links) = chain.get_mut(&last) {
                links.push(token.clone());
            } else {
                chain.insert(last, vec![token.clone()]);
            }

            last = token;
        }

        Ok(Self {
            current_token: None,
            chain,
        })
    }

    fn random_token(&self) -> Option<T> {
        let tokens = self.chain.keys().count();

        let mut rng = rand::rng();
        let idx = rng.random_range(0..tokens);

        self.chain.keys().nth(idx).cloned()
    }
}

impl<T: Clone + std::fmt::Debug + Default + Display + Eq + Hash> Iterator for MarkovIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        loop {
            while self.current_token.is_none() {
                self.current_token = self.random_token();
            }

            let token = self.current_token.clone()?;

            let mut rng = rand::rng();

            let Some(links) = self.chain.get(&token) else {
                self.current_token = None;
                continue;
            };

            if links.is_empty() {
                self.current_token = None;
                continue;
            }

            let next_token = links[rng.random_range(0..links.len())].clone();

            self.current_token = Some(next_token);
            return Some(token);
        }
    }
}

pub fn train(input: String) -> Result<MarkovIterator<String>, std::io::Error> {
    let mut tokens = vec![];
    for entry in WalkDir::new(input) {
        let path = match entry {
            Ok(path) => path,
            Err(e) => panic!("unable to read training file: {e:?}"),
        };

        if !path.file_type().is_file() {
            continue;
        }

        match path.path().extension().unwrap_or_default().to_str() {
            Some("html") => {
                let Ok(contents) = read_to_string(path.path()) else {
                    continue;
                };

                tokens.extend(tokenize_html(contents));
            }
            Some("txt") => {
                let Ok(contents) = read_to_string(path.path()) else {
                    continue;
                };

                for line in contents.lines() {
                    let words = line.split(' ');

                    for word in words {
                        let word = word
                            .chars()
                            .filter(|x| match x {
                                ',' | '.' | '!' | '?' | ':' | ';' => true,
                                '\n' | '\r' | '"' | '\'' => false,
                                _ => true,
                            })
                            .collect::<String>();
                        tokens.push(word);
                    }
                }
            }
            _ => continue,
        }
    }

    MarkovIterator::new(tokens.into_iter())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn read_file() -> Result<(), std::io::Error> {
        let contents = fs::read_to_string("test.txt")?;

        let mut tokens = vec![];

        for line in contents.lines() {
            let words = line.split(' ');

            for word in words {
                let word = word.trim();

                let word: String = word
                    .chars()
                    .filter_map(|x| match x {
                        ',' | '.' | '!' | '?' | ':' | ';' => Some(x),
                        '\n' | '\r' | '"' | '\'' => None,
                        _ => Some(x),
                    })
                    .collect();

                tokens.push(word);
            }
        }

        let mut res = MarkovIterator::new(tokens.into_iter())?;

        for _ in 0..1_000_000 {
            let tok = res.next();
            assert!(!tok.is_none());
        }

        Ok(())
    }
}
