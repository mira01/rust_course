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
                    "little-big",
                    "camel-case"
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
pub fn mutate(arg: &str, content: String) -> String {
    match arg {
        "lowercase" => content.to_lowercase(),
        "uppercase" => content.to_uppercase(),
        "no-spaces" => content.replace([' ', '\n'], ""),
        "slugify" => slugify(content),
        "little-big" => little_big(content),
        "camel-case" => camel_case(content),
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
        .collect::<String>()
        .trim()
        .to_string()
}

// maybe could be simplified
fn camel_case(content: String) -> String {
    content
        .split_whitespace()
        .map(|w| {
            let mut word = String::new();
            if let Some(first) = w.chars().next() {
                word = first.to_uppercase().to_string();
                let rest: String = w
                    .chars()
                    .skip(1)
                    .map(|ch| ch.to_lowercase().to_string())
                    .collect();
                word.push_str(&rest);
            }
            word
        })
        .collect()
}

#[cfg(test)]
mod test {
    use crate::mutate;

    #[test]
    fn transformations() {
        assert_eq!(
            mutate("lowercase", "Lorem ipsum DOLOR sIT AmeT".to_string()),
            "lorem ipsum dolor sit amet"
        );
        assert_eq!(
            mutate("uppercase", "Lorem ipsum DOLOR sIT AmeT".to_string()),
            "LOREM IPSUM DOLOR SIT AMET"
        );
        assert_eq!(
            mutate("no-spaces", "Lorem ipsum DOLOR sIT AmeT".to_string()),
            "LoremipsumDOLORsITAmeT"
        );
        assert_eq!(
            mutate("slugify", "Lorem ipsum DOLOR sIT AmeT".to_string()),
            "lorem-ipsum-dolor-sit-amet"
        );
        assert_eq!(
            mutate("little-big", "Lorem ipsum DOLOR sIT AmeT".to_string()),
            "lorem IPSUM dolor SIT amet"
        );
        assert_eq!(
            mutate("camel-case", "Lorem ipsum DOLOR sIT AmeT".to_string()),
            "LoremIpsumDolorSitAmet"
        );
    }
}
