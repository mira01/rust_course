use slug::slugify as slug_slugify;
use std::env;
use std::io;
use std::process;
use std::error::Error;
use std::convert::TryFrom;
use core::fmt::Display;

#[derive(Debug, Clone, Copy)]
enum Mutation {
    Lowercase,
    Uppercase,
    NoSpaces,
    Slugify,
    LittleBig,
    CamelCase,
}

type StringResult = Result<String, Box<dyn Error>>;

impl Mutation {
    fn mutate(&self) -> StringResult {
        match &self {
            Mutation::Lowercase => lowercase(),
            Mutation::Uppercase => uppercase(),
            Mutation::NoSpaces => no_spaces(),
            Mutation::Slugify => slugify(),
            Mutation::LittleBig => little_big(),
            Mutation::CamelCase => camel_case(),
        }
    }
}

impl TryFrom<&str> for Mutation {
    type Error = Box<dyn Error>;

    fn try_from(item: &str) -> Result<Self, Self::Error> {
        match item {
            "lowercase" => Ok(Self::Lowercase),
            "uppercase" => Ok(Self::Uppercase),
            "no-spaces" => Ok(Self::NoSpaces),
            "slugify"   => Ok(Self::Slugify),
            "little-big" => Ok(Self::LittleBig),
            "camel-case" => Ok(Self::CamelCase),
            m => Err(format!("Unknown method {}", m).into())
        }
    }
}

impl Display for Mutation {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let variant = match self {
            Mutation::Lowercase => "lowercase",
            Mutation::Uppercase => "uppercase",
            Mutation::NoSpaces => "no_spaces",
            Mutation::Slugify => "slugify",
            Mutation::LittleBig => "little_big",
            Mutation::CamelCase => "camel_case",
        };
        write!(fmt, "{}", variant)
    }
}

fn main() {
    match run() {
        Ok(output) => print!("{}", output),
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1);
        }
    };
}

fn run() -> StringResult {
    let mutation = get_mutation()?;
    eprintln!("Will apply {}:", mutation);
    mutation.mutate()
}

fn get_mutation() -> Result<Mutation, Box<dyn Error>> {
   let mutation = env::args().nth(1).ok_or("Cli argument not provided")?;
   Mutation::try_from(&mutation as &str)
}

fn get_stdin() -> StringResult {
    let mut input = String::new();
    let lines = io::stdin().lines();
    for line in lines {
        input.push_str(&line?);
        input.push('\n');
    }
    Ok(input)
}

fn lowercase() -> Result<String, Box<dyn Error>> {
    let input = get_stdin()?;
    Ok(input.to_lowercase())
}

fn uppercase() -> Result<String, Box<dyn Error>> {
    let input = get_stdin()?;
    Ok(input.to_uppercase())
}

fn no_spaces() -> Result<String, Box<dyn Error>> {
    let input = get_stdin()?;
    Ok(input.replace([' ', '\n'], ""))
}

fn slugify() -> StringResult {
    let input = get_stdin()?;
    Ok(slug_slugify(input))
}

fn little_big() -> StringResult {
    let input = get_stdin()?;
    Ok(input
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
        .to_string())
}

fn camel_case() -> StringResult {
    let input = get_stdin()?;
    Ok(input
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
        .collect())
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
