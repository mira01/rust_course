use core::fmt::Display;
use std::convert::TryFrom;
use std::error::Error;

use csv as csv_crate;
use slug::slugify as slug_slugify;
use stanza::renderer::console::Console;
use stanza::renderer::Renderer;
use stanza::style::{MaxWidth, Styles};
use stanza::table::Table;
use term_size::dimensions as term_size_dimensions;

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
    Help,
}

impl Mutation {
    pub fn mutate(&self, text: String) -> StringResult {
        match &self {
            Mutation::Lowercase => lowercase(text),
            Mutation::Uppercase => uppercase(text),
            Mutation::NoSpaces => no_spaces(text),
            Mutation::Slugify => slugify(text),
            Mutation::LittleBig => little_big(text),
            Mutation::CamelCase => camel_case(text),
            Mutation::Csv => csv(text),
            Mutation::Help => help(),
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
            "help" => Ok(Self::Help),
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
            Mutation::Help => "help",
        };
        write!(fmt, "{}", variant)
    }
}

fn help() -> StringResult {
    let help = "This program can perform following operations:\n\
        lowercase uppercase no-spaces slugify little-big camel-case csv help\n\
        and can run in two modes. \n\
        \n\
        In first mode it expects name of the operation as a cli argument and text to operate on \n\
        coming from stdin. In this mode it can handle multiline strings.\n\
        \n\
        In second mode it runs in a loop reading lines where first word is operation name\n\
        and the rest is argument for the operation.\n\
        csv operation works differently: the text after method name is considered a path to read from\n\
        ";
    Ok(help.to_string())
}

fn lowercase(input: String) -> StringResult {
    Ok(input.to_lowercase())
}

fn uppercase(input: String) -> StringResult {
    Ok(input.to_uppercase())
}

fn no_spaces(input: String) -> StringResult {
    Ok(input.replace([' ', '\n'], ""))
}

fn slugify(input: String) -> StringResult {
    Ok(slug_slugify(input))
}

fn little_big(input: String) -> StringResult {
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

fn camel_case(input: String) -> StringResult {
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
fn csv(input: String) -> StringResult {
    let mut rdr = csv_crate::Reader::from_reader(input.as_bytes());
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
    if let Some((width, _)) = term_size_dimensions() {
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
