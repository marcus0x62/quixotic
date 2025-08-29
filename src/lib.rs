use rand::{distr::Alphanumeric, Rng};

pub mod markov;
pub mod rcdom;

pub fn rand_link(mut rng: impl Rng) -> String {
    let len = rng.random_range(4..16);
    (&mut rng)
        .sample_iter(Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
