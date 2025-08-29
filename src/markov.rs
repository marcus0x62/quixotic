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
use std::{
    cmp::PartialEq, collections::HashMap, fmt::Display, fs::read_to_string, hash::Hash, sync::Arc,
};

use rand::Rng;
use walkdir::WalkDir;

use crate::rcdom::tokenize_html;

#[derive(Clone)]
pub struct MarkovIterator<T> {
    tokens: Vec<Arc<T>>,
    current_token: Option<Arc<T>>,
    chain: HashMap<Arc<T>, Vec<Arc<T>>>,
}

impl<T: Clone + Eq + Hash + PartialEq> MarkovIterator<T> {
    pub fn new(tokens: impl Iterator<Item = T>) -> MarkovIterator<T> {
        let mut markov = Self {
            chain: HashMap::<Arc<T>, Vec<Arc<T>>>::new(),
            current_token: None,
            tokens: tokens.map(|x| Arc::new(x)).collect(),
        };

        let mut last = markov.tokens[0].clone();
        for i in 0..markov.tokens.len() {
            if i == 0 {
                continue;
            }

            if let Some(links) = markov.chain.get_mut(&last) {
                links.push(markov.tokens[i].clone());
            } else {
                markov.chain.insert(last, vec![markov.tokens[i].clone()]);
            }

            last = markov.tokens[i].clone();
        }

        markov
    }

    fn random_token(&self) -> Arc<T> {
        let tokens = self.chain.keys().count();

        let mut rng = rand::rng();
        let idx = rng.random_range(0..tokens);

        loop {
            let Some(tok) = self.chain.keys().nth(idx).cloned() else {
                continue;
            };
            return tok;
        }
    }

    pub fn n_tokens(&self, n: u32) -> Vec<Arc<T>> {
        let mut tokens = vec![];
        let mut rng = rand::rng();
        let mut current_token = self.random_token();
        for _ in 0..n {
            let Some(links) = self.chain.get(&current_token) else {
                current_token = self.random_token();
                continue;
            };

            if links.is_empty() {
                current_token = self.random_token();
                continue;
            }

            let next_token = links[rng.random_range(0..links.len())].clone();

            tokens.push(current_token);
            current_token = next_token;
        }

        tokens
    }
}

impl<T: Clone + std::fmt::Debug + Display + Eq + Hash> Iterator for MarkovIterator<T> {
    type Item = Arc<T>;

    fn next(&mut self) -> Option<Arc<T>> {
        let mut rng = rand::rng();

        loop {
            if self.current_token.is_none() {
                self.current_token = Some(self.random_token());
            }

            let Some(token) = self.current_token.clone() else {
                self.current_token = None;
                continue;
            };

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

    Ok(MarkovIterator::new(tokens.into_iter()))
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

        let mut res = MarkovIterator::new(tokens.into_iter());

        for _ in 0..1_000_000 {
            let tok = res.next();
            assert!(!tok.is_none());
        }

        Ok(())
    }
}
