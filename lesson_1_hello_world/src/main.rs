use std::env;

fn main() {
    let locale_greet = env::var("LANG")
        .ok()
        .and_then(|lang| {
            lang.split('_')
                .next()
                .and_then(|lang|
                          match lang {
                              "en" => Some("Hello, world!"),
                              "cs" => Some("Ahoj světe!"),
                              "es" => Some("¡Hola el mundo!"),
                              _ => None,
                          }
                         )}
                );
    if let Some(translated) = locale_greet {
        println!("{}", translated);
    } else {
        println!("Hello, world!");
    }

}
