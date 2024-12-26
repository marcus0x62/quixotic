use rand::{distributions::Alphanumeric, Rng};

pub mod markov;
pub mod rcdom;

pub fn rand_link() -> String {
    let mut rng = rand::thread_rng();

    let len = rng.gen_range(4..16);
    (&mut rng)
        .sample_iter(Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
