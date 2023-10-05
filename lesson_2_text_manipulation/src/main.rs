use slug::slugify;
use std::env;
use std::io;
use std::process;

fn main() {
    let mutation = env::args()
        .nth(1)
        .or_else(|| {
            println!(
                "Please specify one of {:?} as a script argument",
                [
                    "lowercase",
                    "uppercase",
                    "no-spaces",
                    "slugify",
                    "little-big"
                ]
            );
            process::exit(1);
        })
        .unwrap();
    let mut input = String::new();
    io::stdin().lines().for_each(|result| {
        result
            .map(|l| {
                input.push_str(&l);
                input.push('\n');
            })
            .expect("Could not read from stdin")
    });
    println!("{}", mutate(&mutation, input));
}

// :( I cannot write a function that dispatches closures based on HashMap key yet :)
fn mutate(arg: &str, content: String) -> String {
    match arg {
        "lowercase" => content.to_lowercase(),
        "upercase" => content.to_uppercase(),
        "no-spaces" => content.replace([' ', '\n'], ""),
        "slugify" => slugify(content),
        "little-big" => little_big(content),
        _ => content,
    }
}

fn little_big(content: String) -> String {
    content
        .split_whitespace()
        .zip([false, true].into_iter().cycle())
        .map(|(w, big)| {
            let mut word = if big {
                w.to_uppercase()
            } else {
                w.to_lowercase()
            };
            word.push(' ');
            word
        })
        .collect()
}
