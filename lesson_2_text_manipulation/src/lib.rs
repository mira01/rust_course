use core::fmt::Display;
use std::convert::TryFrom;
use std::error::Error;
use std::io::{self, stdin};

use csv as csv_crate;
use slug::slugify as slug_slugify;
use stanza::renderer::console::Console;
use stanza::renderer::Renderer;
use stanza::style::{MaxWidth, Styles};
use stanza::table::Table;
use term_size;

pub type StringResult = Result<String, Box<dyn Error>>;

const DEFAULT_WIDTH: usize = 80;

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
            "slugify" => Ok(Self::Slugify),
            "little-big" => Ok(Self::LittleBig),
            "camel-case" => Ok(Self::CamelCase),
            "csv" => Ok(Self::Csv),
            m => Err(format!("Unknown method {}", m).into()),
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

/// Get standard input as string or return error
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

/// Csv mutation
/// I am not happy how this function looks. It does manny things at once
fn csv() -> StringResult {
    let mut rdr = csv_crate::Reader::from_reader(stdin());
    let headers = rdr.headers()?;
    if headers.is_empty() {
        return Err("Empty headers".into());
    }
    let max_column_width = max_column_width(headers.len());
    let mut table = Table::with_styles(Styles::default().with(MaxWidth(max_column_width)));
    table.push_row(headers);
    for result in rdr.records() {
        let record = result?;
        table.push_row(record.iter());
    }
    if table.is_empty() {
        return Err("Cannot render table".into());
    }

    let renderer = Console::default();
    let mut output = renderer.render(&table).to_string();
    output.push('\n');
    Ok(output)
}

/// Get terminal width
fn term_width() -> usize {
    if let Some((width, _)) = term_size::dimensions() {
        width
    } else {
        DEFAULT_WIDTH
    }
}

/// Compute maximum column width based on size of terminal
fn max_column_width(count: usize) -> usize {
    let delimiters = count - 1;
    let borders = 2;
    (term_width() - borders - delimiters) / count
}
