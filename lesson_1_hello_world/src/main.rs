use std::env;

fn main() {
    if let Some(translated) = get_locale_greet() {
        println!("{}", translated);
    } else {
        println!("Hello, world!");
    }
}

fn get_locale_greet() -> Option<&'static str> {
    env::var("LANG").ok().and_then(|lang| {
        lang.split('_').next().and_then(|lang| match lang {
            "en" => Some("Hello, world!"),
            "cs" => Some("Ahoj světe!"),
            "es" => Some("¡Hola el mundo!"),
            _ => None,
        })
    })
}
