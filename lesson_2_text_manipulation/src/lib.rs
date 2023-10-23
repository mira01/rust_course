use std::io::{self, stdin};
use std::error::Error;
use std::convert::TryFrom;
use core::fmt::Display;

use slug::slugify as slug_slugify;
use csv as csv_crate;
use stanza::renderer::console::Console;
use stanza::renderer::Renderer;
use stanza::style::{Styles, MaxWidth};
use stanza::table::Table;

pub type StringResult = Result<String, Box<dyn Error>>;

#[derive(Debug, Clone, Copy)]
pub enum Mutation {
    Lowercase,
    Uppercase,
    NoSpaces,
    Slugify,
    LittleBig,
    CamelCase,
    Csv,
}

impl Mutation {
    pub fn mutate(&self) -> StringResult {
        match &self {
            Mutation::Lowercase => lowercase(),
            Mutation::Uppercase => uppercase(),
            Mutation::NoSpaces => no_spaces(),
            Mutation::Slugify => slugify(),
            Mutation::LittleBig => little_big(),
            Mutation::CamelCase => camel_case(),
            Mutation::Csv => csv(),
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
            "csv"        => Ok(Self::Csv),
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
            Mutation::Csv => "csv",
        };
        write!(fmt, "{}", variant)
    }
}

pub fn get_stdin() -> StringResult {
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

fn csv() -> StringResult {
    let mut rdr = csv_crate::Reader::from_reader(stdin());
    let mut table = Table::with_styles(Styles::default().with(MaxWidth(80)));
    for result in rdr.records() {
        let record = result?;
        table.push_row(record.iter());
    }
    if table.is_empty() {
        return Err("Cannot render table".into());
    }
    let renderer = Console::default();
    let mut output = renderer.render(&table).to_string();
    output.push_str("\n");
    Ok(output)
}
