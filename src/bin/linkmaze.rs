// MIT License
//
// Copyright (c) 2024 Marcus Butler
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
use actix_web::{
    get, http::header::ContentType, web, App, HttpResponse, HttpServer, Responder, Result,
};
use bytes::{BufMut, BytesMut};
use clap::Parser;
use rand::Rng;
use std::process::exit;

use quixotic::markov::{train, MarkovIterator};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value_t = String::from("/quixotic"))]
    linkpath: String,
    #[arg(short, long, default_value_t = 0.20)]
    percent: f32,
    #[arg(short, long)]
    train: String,
    #[arg(long, default_value_t = 3005)]
    listen_port: u16,
    #[arg(long, default_value_t = String::from("0.0.0.0"))]
    listen_addr: String,
    #[arg(long, default_value_t = 250)]
    min_tokens: u32,
    #[arg(long, default_value_t = 12500)]
    max_tokens: u32,
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    if args.min_tokens > args.max_tokens {
        eprintln!(
            "Error: min_tokens ({}) must be less than max_tokens ({})",
            args.min_tokens, args.max_tokens
        );
        exit(1);
    }

    let markov = train(args.train)?;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(args.linkpath.clone()))
            .app_data(web::Data::new(markov.clone()))
            .app_data(web::Data::new((args.min_tokens, args.max_tokens)))
            .service(maze)
    })
    .bind((args.listen_addr, args.listen_port))?
    .run()
    .await
}

#[get("/{uri}")]
async fn maze(
    path: web::Path<String>,
    linkpath: web::Data<String>,
    markov: web::Data<MarkovIterator<String>>,
    limits: web::Data<(u32, u32)>,
) -> impl Responder {
    let uri = path.into_inner();
    let (min_tokens, max_tokens) = *limits.into_inner();

    let mut rng = rand::rng();
    let n_tokens = rng.random_range(min_tokens..max_tokens);

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

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(res)
}
