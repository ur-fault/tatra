use std::io::Read;

fn main() {
    let file = std::env::args().nth(1).unwrap_or("-".to_string());

    let text = if file == "-" {
        let mut text = String::new();
        std::io::stdin().read_to_string(&mut text).unwrap();
        text
    } else {
        std::fs::read_to_string(file).unwrap()
    };

    let tokens = tokenize(&text);
    let styles = stylize(tokens);

    for style in styles {
        match style {
            Style::Text(text) => print!("{}", text),
            Style::Attribute(attr) => print!("{}", yansi::Style::from(attr).prefix()),
            Style::Foreground(color) => print!("{}", yansi::Style::new().fg(color).prefix()),
            Style::Background(color) => print!("{}", yansi::Style::new().bg(color).prefix()),
            Style::Reset => print!("{}", yansi::Style::new().resetting().suffix()),
        }
    }
}

#[derive(Debug)]
enum Style<'a> {
    Text(&'a str),
    Attribute(yansi::Attribute),
    Foreground(yansi::Color),
    Background(yansi::Color),
    Reset,
}

fn stylize(tokens: Vec<Token<'_>>) -> Vec<Style<'_>> {
    let mut styles = Vec::new();
    for token in tokens {
        if token.attr {
            if "/" == &token.slice[0..1] {
                let style = match &token.slice[1..2] {
                    "f" => Style::Foreground(parse_color(&token.slice[1..]).unwrap()),
                    "b" => Style::Background(parse_color(&token.slice[1..]).unwrap()),
                    "r" => Style::Reset,
                    "a" => Style::Attribute(parse_attr(&token.slice[1..]).unwrap()),
                    _ => {
                        eprintln!("unknown attribute: '{}'", token.slice);
                        continue;
                    }
                };

                styles.push(style);
            } else {
                for attr in token.slice.split(',') {
                    if attr == "reset" {
                        styles.push(Style::Reset);
                    } else if let Some(attr) = parse_attr(attr) {
                        styles.push(Style::Attribute(attr));
                    } else if let Some(color) = parse_color(attr) {
                        styles.push(Style::Foreground(color));
                    } else if attr.starts_with("on") {
                        if let Some(color) = parse_color(&attr[2..]) {
                            styles.push(Style::Background(color));
                        }
                    }
                }
            }
        } else {
            styles.push(Style::Text(token.slice));
        }
    }
    styles
}

fn parse_color(color: &str) -> Option<yansi::Color> {
    match color {
        "black" => Some(yansi::Color::Black),
        "red" => Some(yansi::Color::Red),
        "green" => Some(yansi::Color::Green),
        "yellow" => Some(yansi::Color::Yellow),
        "blue" => Some(yansi::Color::Blue),
        "magenta" => Some(yansi::Color::Magenta),
        "cyan" => Some(yansi::Color::Cyan),
        "white" => Some(yansi::Color::White),
        _ => None,
    }
}

fn parse_attr(attr: &str) -> Option<yansi::Attribute> {
    match attr {
        "bold" => Some(yansi::Attribute::Bold),
        "dim" => Some(yansi::Attribute::Dim),
        "italic" => Some(yansi::Attribute::Italic),
        "underline" => Some(yansi::Attribute::Underline),
        "blink" => Some(yansi::Attribute::Blink),
        "rapid-blink" => Some(yansi::Attribute::RapidBlink),
        "conceal" => Some(yansi::Attribute::Conceal),
        "strike" => Some(yansi::Attribute::Strike),
        "invert" => Some(yansi::Attribute::Invert),
        _ => None,
    }
}

#[derive(Debug)]
struct Token<'a> {
    attr: bool,
    slice: &'a str,
}

fn tokenize(text: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut in_escape = false;

    #[derive(Debug)]
    struct Parsing {
        attr: bool,
        start: usize,
        end: usize,
    }

    let mut parsing = Parsing {
        attr: false,
        start: 0,
        end: 0,
    };

    let mut chars = text.char_indices().peekable();

    while let Some((index, ch)) = chars.next() {
        if ch == '\\' && in_escape {
            in_escape = false;
            parsing.end = index + ch.len_utf8();
            let slice = &text[parsing.start + 1..parsing.end];
            parsing.attr = !parsing.attr;
            if slice.is_empty() {
                continue;
            }
            tokens.push(Token {
                attr: parsing.attr,
                slice,
            });

            parsing.start = index + ch.len_utf8();
        } else if ch == '\\' {
            in_escape = true && parsing.attr;
            let start = if parsing.attr && parsing.start + 1 <= index {
                parsing.start + 1
            } else {
                parsing.start
            };
            let slice = &text[start..index];

            if !slice.is_empty() {
                tokens.push(Token {
                    attr: parsing.attr,
                    slice,
                });
            }

            parsing.start = if parsing.attr {
                index + ch.len_utf8()
            } else {
                index
            };
            parsing.attr = !parsing.attr;
        } else {
            in_escape = false;
        }
    }

    let start = if parsing.attr {
        parsing.start + 1
    } else {
        parsing.start
    };
    let slice = &text[start..];
    tokens.push(Token {
        attr: parsing.attr,
        slice,
    });

    tokens
}
