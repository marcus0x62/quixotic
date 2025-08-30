use bytes::{BufMut, BytesMut};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use quixotic::markov;
use rand::Rng;

pub fn ntokens_benchmark(c: &mut Criterion) {
    let markov = markov::train("/home/marcusb/code/marcusb.org/public".into()).unwrap();
    c.bench_function("markov 128k n_tokens", |b| {
        b.iter(|| {
            let tok = markov.n_tokens(128000);
            assert_eq!(tok.len(), 128000);
        })
    });
}

pub fn linkmaze_benchmark(c: &mut Criterion) {
    let markov = markov::train("/home/marcusb/code/marcusb.org/public".into()).unwrap();
    c.bench_function("linkmaze text generation n_tokens=128k", |b| {
        b.iter(|| {
            let uri = "/quixotic";
            let linkpath = "/quixotic";

            let mut rng = rand::rng();
            let n_tokens = 128000;

            let mut res = BytesMut::with_capacity(n_tokens as usize * 12);
            res.put(&b"<!doctype html><html lang=en><head><title>"[..]);
            res.put(uri.as_bytes());
            res.put(&b"</title></head><body><p>"[..]);

            let tokens = markov.n_tokens(n_tokens);

            for token in tokens {
                let r = rng.random::<u8>();
                res.put(&b" "[..]);
                res.put(token.as_bytes());
                if r < 5 {
                    res.put(&b".</p><p>"[..]);
                } else if r < 10 {
                    let rand_link = quixotic::rand_link(&mut rng);
                    res.put(&b" <a href=/"[..]);
                    res.put(linkpath.as_bytes());
                    res.put(&b"/"[..]);
                    res.put(rand_link.as_bytes());
                    res.put(&b".html>"[..]);
                    res.put(rand_link.as_bytes());
                    res.put(&b"</a>"[..]);
                }
            }
        })
    });
}

criterion_group!(benches, ntokens_benchmark, linkmaze_benchmark);
criterion_main!(benches);
