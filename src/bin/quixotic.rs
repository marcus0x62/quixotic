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
use std::{
    fs::{copy, create_dir, exists, read_to_string, File},
    io::{BufWriter, Error, Write},
    path::Path,
};

use clap::Parser;
use html5ever::driver::ParseOpts;
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{parse_document, serialize};
use rand::Rng;
use walkdir::WalkDir;

use quixotic::{
    markov::{train, MarkovIterator},
    rcdom::{RcDom, SerializableHandle},
};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value_t = false)]
    embed_linkmaze: bool,
    #[arg(long, default_value_t = 0.40)]
    scramble_images: f32,
    #[arg(long)]
    linkmaze_path: Option<String>,
    #[arg(short, long)]
    input: String,
    #[arg(short, long)]
    output: String,
    #[arg(short, long, default_value_t = 0.20)]
    percent: f32,
    #[arg(short, long)]
    train: Option<String>,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let mut res = train(args.train.unwrap_or(args.input.clone()))?;
    let mut images = vec![];

    for entry in WalkDir::new(&args.input) {
        let path = match entry {
            Ok(path) => path,
            Err(e) => panic!("unable to read input file: {e:?}"),
        };

        let Ok(strip_input) = path.path().strip_prefix(&args.input) else {
            panic!("file {:?} does not begin with {}", path.path(), args.input);
        };

        let output_file = Path::new(&args.output).join(strip_input);
        if path.file_type().is_dir() && !exists(&output_file)? {
            create_dir(&output_file)?;
            continue;
        } else if !path.file_type().is_file() {
            continue;
        }

        // Build a list of images to use in random substitution
        match path.path().extension().unwrap().to_str() {
            Some("png") | Some("gif") | Some("svg") | Some("jpg") | Some("jpeg") | Some("webp")
            | Some("avif") => images.push(path.path().to_owned()),
            _ => {}
        }

        let output_buf = match path.path().extension().unwrap().to_str() {
            Some("html") => {
                let contents = read_to_string(path.path())?;
                transform_html(
                    contents,
                    &mut res,
                    1.0 - args.percent,
                    args.embed_linkmaze,
                    args.linkmaze_path.clone(),
                )
            }
            Some("txt") => {
                let mut output_lines = vec![];
                let contents = read_to_string(path.path())?;
                for line in contents.lines() {
                    let mut output_line = vec![];
                    for word in line.split(" ") {
                        if rand::random::<f32>() < (1.0 - args.percent) {
                            output_line.push(String::from(word));
                        } else {
                            let Some(tok) = res.next() else {
                                panic!("could not generate token!");
                            };
                            output_line.push(tok);
                        }
                    }

                    output_lines.push(output_line.join(" "));
                }

                output_lines.join("\n")
            }
            Some("png") | Some("gif") | Some("svg") | Some("jpg") | Some("jpeg") | Some("webp")
            | Some("avif")
                if args.scramble_images > 0.00 =>
            {
                let mut rng = rand::thread_rng();
                if rng.gen::<f32>() > args.scramble_images {
                    copy(path.path(), &output_file)?;
                } else {
                    copy(images[rng.gen_range(0..images.len())].clone(), &output_file)?;
                }
                continue;
            }
            _ => {
                copy(path.path(), &output_file)?;
                continue;
            }
        };

        let mut file = File::create(output_file)?;
        file.write_all(output_buf.as_bytes())?;
    }

    Ok(())
}

fn transform_html(
    contents: String,
    markov: &mut MarkovIterator<String>,
    percent: f32,
    embed_linkmaze: bool,
    linkmaze_path: Option<String>,
) -> String {
    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let dom = parse_document(RcDom::default(), opts).one(contents);

    let mut buf = BufWriter::new(Vec::new());
    let document =
        SerializableHandle::new(dom.document, markov, percent, embed_linkmaze, linkmaze_path);
    serialize(&mut buf, &document, Default::default()).expect("serialization failure");
    let bytes = buf.into_inner().unwrap();
    String::from_utf8(bytes).unwrap()
}
