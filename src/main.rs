use alert::alert;
use clipboard::{ClipboardContext, ClipboardProvider};
use select::document::Document;
use select::node::Node;
use select::predicate::Class;
use std::env;

fn parse(node: Node) -> String {
    let mut res = String::new();
    match node.name() {
        Some("ol") => {
            res += r"\begin{question}";
            res += "\n";
            for child in node.children() {
                res += &parse(child);
            }
            res += r"\end{question}";
            res += "\n";
        }
        Some("ul") => {
            res += r"\begin{itemize}";
            res += "\n";
            for child in node.children() {
                res += &parse(child);
            }
            res += r"\end{itemize}";
            res += "\n";
        }
        Some("li") => {
            res += r"\item";
            res += "\n";
            match node.as_text() {
                Some(text) => res += text,
                _ => {}
            };
            for child in node.children() {
                res += &parse(child);
            }
            res += "\n";
        }
        Some("br") => {
            res += "\n";
        }
        Some("u") => {}
        Some("div") => {
            res += "\n";
            match node.as_text() {
                Some(text) => res += text,
                _ => {}
            };
            for child in node.children() {
                res += &parse(child);
            }
            //res += "\n"
        }
        _ => {
            match node.as_text() {
                Some(text) => res += text,
                _ => {}
            };
            for child in node.children() {
                res += &parse(child);
            }
        }
    }
    res
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let nb_exo = &args[1];
    let url = "https://beos.prepas.org/?q=Epreuve%20Orale%20".to_owned() + nb_exo;

    let resp = reqwest::blocking::get(&url)?;
    let document = Document::from_read(resp)?;

    let mut info = document
        .find(Class("field-label"))
        .into_selection()
        .iter()
        .filter_map(|node| node.next())
        .take(3)
        .map(|node| node.text())
        .collect::<Vec<_>>();
    info.reverse();

    let mut exo = String::new();
    exo += r"\begin{exo}[comment=";
    exo += &info.join(" ");
    exo += "]\n";

    for node in document.find(Class("tex2jax")) {
        exo += &parse(node);
    }

    exo += r"\end{exo}";
    exo += "\n";
    alert("Beos scraping", "Copied into clipboard");
    let mut cx: ClipboardContext = ClipboardProvider::new()?;
    cx.set_contents(exo)?;

    Ok(())
}
