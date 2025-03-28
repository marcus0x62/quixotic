use rand::{distr::Alphanumeric, Rng};

pub mod markov;
pub mod rcdom;

pub fn rand_link() -> String {
    let mut rng = rand::rng();

    let len = rng.random_range(4..16);
    (&mut rng)
        .sample_iter(Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
