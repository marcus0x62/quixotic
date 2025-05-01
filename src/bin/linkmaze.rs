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
use actix_web::{get, http::header::ContentType, web, App, HttpResponse, HttpServer, Result};
use clap::Parser;
use rand::Rng;
use std::sync::{Arc, Mutex};

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
    #[arg(long)]
    large_file_path: Option<String>,
    #[arg(long, default_value_t = 0.20)]
    large_file_percent: f32,
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    HttpServer::new(move || {
        let markov = train(args.train.clone()).unwrap();
        App::new()
            .app_data(web::Data::new(args.linkpath.clone()))
            .app_data(web::Data::new(Arc::new(Mutex::new(markov))))
            .app_data(web::Data::new(args.large_file_path.clone()))
            .app_data(web::Data::new(args.large_file_percent.clone()))
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
    markov: web::Data<Arc<Mutex<MarkovIterator<String>>>>,
    large_file_path: web::Data<Option<String>>,
    large_file_percent: web::Data<f32>,
) -> HttpResponse {
    let uri = path.into_inner();
    let mut res = format!("<!doctype html><html lang=en><head><title>{uri}</title></head><body>");

    let mut rng = rand::rng();

    let mut tokens = vec![];
    match markov.lock() {
        Ok(mut markov) => {
            for _ in 0..rng.random_range(250..12500) {
                tokens.push(markov.next().unwrap());
            }
        }
        Err(e) => {
            eprintln!("error unlocking markov chain: {e:?}");
            return HttpResponse::Ok().body("internal server error");
        }
    }

    let inject_large_file = large_file_path.is_some() && rand::random::<f32>() < large_file_percent.abs();
    let mut large_file_injected = false;

    let mut in_p = false;
    for (i, token) in tokens.iter().enumerate() {
        if i == 0 || rand::random::<f32>() < 0.05 && !in_p {
            res.push_str("<p>");
            in_p = true;
        }
        res.push_str(&format!(" {token}"));
        if i != tokens.len() && in_p && rand::random::<f32>() < 0.05 {
            res.push_str("</p><p>");
        }

        if rand::random::<f32>() < 0.02 {
            let rand_link = quixotic::rand_link();
            // at max once per page, with a 0.3 probability, make the link point to a file that
            // decompresses to large output on the client
            let large_file = large_file_path.as_ref().clone().unwrap();
            if inject_large_file && !large_file_injected && rand::random::<f32>() < 0.3 {
                res.push_str(&format!(
                    "<a href={}>{rand_link}</a>",
                    large_file
                ));
                eprintln!("injected large file path {large_file} into {uri}");
                large_file_injected = true;
            }
            else {
                res.push_str(&format!(
                    "<a href=/{}/{rand_link}.html>{rand_link}</a>",
                    *linkpath
                ));
            }
        }

        if i == tokens.len() - 1 {
            res.push_str("</p>");
        }
    }

    res.push_str("</body></html>");
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(res)
}
