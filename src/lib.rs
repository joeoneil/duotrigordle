#![allow(unused)]
#![feature(iter_collect_into)]

use std::{fmt::Display, fs, rc::Rc};

use rand::seq::SliceRandom;

pub fn init_word_list() -> Rc<[&'static [char]]> {
    fs::read_to_string("./src/words.txt")
        .unwrap()
        .lines()
        .filter(|s| s.len() == 5)
        .map(|s| &*String::from(s).chars().collect::<Vec<_>>().leak())
        .collect()
}

pub fn to_char_slice(s: &str) -> &'static [char] {
    &*String::from(s).chars().collect::<Vec<_>>().leak()
}

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub enum Color {
    Gray,
    Yellow,
    Green,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Guess<'a> {
    guess: &'a [char],
    data: [Color; 5],
}

impl<'a> Display for Guess<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const GREEN: &str = "\x1b[30;42m";
        const YELLOW: &str = "\x1b[30;43m";
        const GRAY: &str = "\x1b[30;47m";
        const RESET: &str = "\x1b[0m";

        for i in 0..5 {
            write!(
                f,
                "{}{}",
                match self.data[i] {
                    Color::Green => GREEN,
                    Color::Yellow => YELLOW,
                    Color::Gray => GRAY,
                },
                self.guess[i]
            )?;
        }

        write!(f, "{}", RESET)
    }
}

impl<'a> Guess<'a> {
    pub fn new(answer: &'a [char], guess: &'a [char]) -> Self {
        let mut data = [Color::Gray; 5];
        let mut a_2 = [' '; 5];
        let mut g_2 = [' '; 5];
        for i in 0..5 {
            g_2[i] = guess[i];
            a_2[i] = answer[i];
            if guess[i] == answer[i] {
                data[i] = Color::Green;
            }
        }

        for i in 0..5 {
            if data[i] == Color::Green {
                continue;
            }
            for loc in guess.indices(&answer[i]) {
                if data[loc] == Color::Gray {
                    data[loc] = Color::Yellow;
                    break;
                }
            }
        }

        Self { guess, data }
    }

    fn plausible(&self, answer: &'a [char]) -> bool {
        let mut a_2 = [' '; 5];
        for i in 0..5 {
            a_2[i] = answer[i];
        }
        for i in 0..5 {
            match self.data[i] {
                Color::Green => {
                    if answer[i] != self.guess[i] {
                        return false;
                    }
                }
                Color::Yellow => {
                    if answer[i] == self.guess[i] {
                        return false;
                    }
                    if let None = a_2.index_of(&self.guess[i]) {
                        return false;
                    }
                    let c = self.guess[i];
                    // total occur - gray occur <= total occur answer
                    let cnt = {
                        let mut cnt = 0;
                        for j in 0..5 {
                            if self.data[j] == Color::Gray && self.guess[j] == c {
                                cnt += 1;
                            }
                        }
                        cnt
                    };
                    if (self.guess.indices(&self.guess[i]).len() as isize) - cnt
                        > (answer.indices(&self.guess[i]).len() as isize)
                    {
                        return false;
                    }
                }
                Color::Gray => {
                    // if letter appears i in answer, then appears j in guess, there are j - i gray
                    // in guess
                    let c = self.guess[i];
                    let cnt = {
                        let mut cnt = 0;
                        for j in 0..5 {
                            if self.data[j] == Color::Gray && self.guess[j] == c {
                                cnt += 1;
                            }
                        }
                        cnt
                    };
                    if (self.guess.indices(&self.guess[i]).len() as isize)
                        - (answer.indices(&self.guess[i]).len() as isize)
                        != cnt
                    {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn filter(&self, words: Vec<&'a [char]>) -> Vec<&'a [char]> {
        words.into_iter().filter(|w| self.plausible(w)).collect()
    }
}

#[derive(Clone, Debug)]
pub struct Wordle<'a> {
    answer: &'a [char],
    guess: Vec<Guess<'a>>,
}

impl<'a> Wordle<'a> {
    pub fn new(answer: &'a [char]) -> Self {
        Wordle {
            answer,
            guess: vec![],
        }
    }

    pub fn filter(&self, mut words: Vec<&'a [char]>) -> Vec<&'a [char]> {
        for g in &self.guess {
            words = g.filter(words);
        }
        words
    }

    pub fn filter_last(&self, mut words: Vec<&'a [char]>) -> Vec<&'a [char]> {
        if let Some(g) = &self.guess.last() {
            g.filter(words)
        } else {
            words
        }
    }

    pub fn guess(&mut self, guess: &'a [char]) -> &Guess<'a> {
        self.guess.push(Guess::new(self.answer, guess));
        self.guess.last().unwrap()
    }

    fn clear(&mut self) {
        self.guess.clear();
    }

    pub fn solved(&self) -> bool {
        for g in &self.guess {
            if g.guess == self.answer {
                return true;
            }
        }
        false
    }
}

#[derive(Clone, Debug)]
pub struct Duotrigordle<'a> {
    wordles: Vec<Wordle<'a>>,
    words: Vec<&'a [char]>,
    lists: Vec<Vec<&'a [char]>>,
    solved: [bool; 32],
}

impl<'a> Duotrigordle<'a> {
    pub fn new(words: Vec<&'a [char]>) -> Self {
        let mut lists = vec![];
        for _ in 0..32 {
            lists.push(words.clone());
        }
        Self {
            wordles: words
                .choose_multiple(&mut rand::thread_rng(), 32)
                .map(|w| Wordle::new(w))
                .collect(),
            words,
            lists,
            solved: [false; 32],
        }
    }

    pub fn new_single_fixed(fixed: &'a [char], words: Vec<&'a [char]>) -> Self {
        let mut wordles = vec![Wordle::new(fixed)];
        let mut lists = vec![];
        for _ in 0..32 {
            lists.push(words.clone());
        }
        Self {
            wordles: words
                .clone()
                .into_iter()
                .filter(|e| *e != fixed)
                .collect::<Vec<_>>()
                .choose_multiple(&mut rand::thread_rng(), 31)
                .map(|w| Wordle::new(w))
                .collect_into(&mut wordles)
                .to_owned(),
            words,
            lists,
            solved: [false; 32],
        }
    }

    pub fn solveable(&mut self) -> (bool, Vec<&'a [char]>) {
        let answers = (0..32)
            .filter_map(|i| self.solveable_from(i))
            .collect::<Vec<_>>();
        (answers.len() > 0, answers)
    }

    pub fn answers(&self) -> Vec<&'a [char]> {
        self.wordles.iter().map(|w| w.answer).collect()
    }

    pub fn solveable_from(&mut self, index: usize) -> Option<&'a [char]> {
        self.reset();
        let mut g = self.wordles[index].answer;
        // while there is more than 1 unsolved wordle
        while self.solved.iter().filter(|b| !**b).count() > 1 {
            // guess word on all unsolved wordles, updating answer lists and
            // solved wordles list.
            self.guess(g);
            let wc = self
                .lists
                .iter()
                .zip(self.wordles.iter())
                .enumerate()
                .filter(|(i, _)| !self.solved[*i])
                .map(|(i, (l, w))| (l.len(), w))
                .collect::<Vec<_>>();
            if *wc.iter().map(|(l, w)| l).min().unwrap() != 1 {
                return None;
            }
            g = wc
                .iter()
                .find(|(l, w)| *l == 1)
                .map(|(_, w)| w.answer)
                .unwrap();
        }

        Some(self.wordles[index].answer)
    }

    pub fn guess(&mut self, word: &'a [char]) -> &Vec<Vec<&'a [char]>> {
        for i in 0..32 {
            if self.wordles[i].answer == word {
                self.solved[i] = true;
                let g = self.wordles[i].guess(word);
                self.lists[i] = g.filter(self.lists[i].clone());
            }
        }
        let guesses = self
            .wordles
            .iter_mut()
            .map(|e| e.guess(word))
            .collect::<Vec<_>>()
            .clone();
        for i in 0..32 {
            if self.solved[i] {
                continue;
            }
            self.lists[i] = guesses[i].filter(self.lists[i].clone());
        }
        &self.lists
    }

    pub fn guesses(&self, index: usize) -> &Vec<Guess<'a>> {
        &self.wordles[index].guess
    }

    pub fn solved(&self) -> &[bool; 32] {
        &self.solved
    }

    pub fn reset(&mut self) {
        for w in &mut self.wordles {
            w.clear();
        }
        self.solved = [false; 32];
        self.lists = vec![];
        for _ in 0..32 {
            self.lists.push(self.words.clone());
        }
    }
}

pub trait IndexOf<T> {
    fn index_of(&self, elem: &T) -> Option<usize>;

    fn indices(&self, elem: &T) -> Vec<usize>;
}

impl<T> IndexOf<T> for [T]
where
    T: PartialEq,
{
    fn index_of(&self, elem: &T) -> Option<usize> {
        for (index, e) in self.iter().enumerate() {
            if elem == e {
                return Some(index);
            }
        }
        None
    }

    fn indices(&self, elem: &T) -> Vec<usize> {
        let mut v = vec![];
        for (index, e) in self.iter().enumerate() {
            if elem == e {
                v.push(index);
            }
        }
        v
    }
}

impl<T> IndexOf<T> for Vec<T>
where
    T: PartialEq,
{
    fn index_of(&self, elem: &T) -> Option<usize> {
        for (index, e) in self.iter().enumerate() {
            if elem == e {
                return Some(index);
            }
        }
        None
    }

    fn indices(&self, elem: &T) -> Vec<usize> {
        let mut v = vec![];
        for (index, e) in self.iter().enumerate() {
            if elem == e {
                v.push(index)
            }
        }
        v
    }
}
