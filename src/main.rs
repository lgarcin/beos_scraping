use alert::alert;
use clipboard::{ClipboardContext, ClipboardProvider};
use regex::{Error, Regex};
use select::document::Document;
use select::node::Node;
use select::predicate::Predicate;
use select::predicate::{Class, Name, Text};
use ureq;

const EXO_SEPARATOR: &str = "-----";
const BASE_URL: &str = "https://beos.prepas.org/sujet.php?id=";

fn parse_node(node: &Node) -> String {
    let mut parsed_node = String::new();
    match node.name() {
        Some("ol") => {
            parsed_node += r"\begin{question}";
            parsed_node += "\n";
            for child in node.children() {
                parsed_node += &parse_node(&child);
            }
            parsed_node += r"\end{question}";
            parsed_node += "\n";
        }
        Some("ul") => {
            parsed_node += r"\begin{itemize}";
            parsed_node += "\n";
            for child in node.children() {
                parsed_node += &parse_node(&child);
            }
            parsed_node += r"\end{itemize}";
            parsed_node += "\n";
        }
        Some("li") => {
            parsed_node += r"\item";
            parsed_node += "\n";
            match node.as_text() {
                Some(text) => parsed_node += text,
                _ => {}
            };
            for child in node.children() {
                parsed_node += &parse_node(&child);
            }
            parsed_node += "\n";
        }
        Some("br") => {
            parsed_node += "\n";
        }
        Some("u") => {
            if node.text().contains("Exercice") {
                parsed_node += "\n";
                parsed_node += EXO_SEPARATOR;
                parsed_node += "\n";
            }
        }
        other => {
            if other == Some("div") {
                parsed_node += "\n"
            }
            match node.as_text() {
                Some(text) => {
                    if text.contains("Exercice") {
                        parsed_node += "\n";
                        parsed_node += EXO_SEPARATOR;
                        parsed_node += "\n";
                    } else {
                        parsed_node += text
                    }
                }
                _ => {}
            };
            for child in node
                .children()
                .take_while(|n| !n.text().starts_with("Indication"))
            {
                parsed_node += &parse_node(&child);
            }
        }
    }
    parsed_node
}

fn parse_document(document: &Document) -> String {
    let mut parsed_document = String::new();
    for node in document.find(Class("latex")) {
        parsed_document += &parse_node(&node);
    }

    if !parsed_document.contains(EXO_SEPARATOR) {
        parsed_document = EXO_SEPARATOR.to_owned() + &parsed_document;
    }
    parsed_document
}

fn preambule(document: &Document) -> String {
    let mut info = document
        .find(Name("div").child(Name("div")).child(Name("p")).child(Text))
        .into_selection()
        .iter()
        // .filter_map(|node| node.next())
        .take(3)
        .map(|node| node.text().trim().to_owned())
        .collect::<Vec<_>>();
    info.reverse();

    let mut preambule = String::new();
    preambule += r"\begin{exo}[comment=";
    preambule += &info.join(" ");
    preambule += "]\n";
    preambule
}

fn process_exo(exercice: &str) -> Result<String, Error> {
    let mut exo = exercice.to_owned();
    exo = Regex::new(r"(?s)(1\..*)")?
        .replace_all(&exo, "\\begin{question}\n$1\\end{question}\n$2")
        .into_owned();
    exo = Regex::new(r"(?s)(a\).*?)(\n\s*\d\.)")?
        .replace_all(&exo, "\n\\begin{question}\n$1\\end{question}\n$2")
        .into_owned();
    exo = Regex::new(r"(?s)(\s+)\d\.")?
        .replace_all(&exo, "$1\\item ")
        .into_owned();
    exo = Regex::new(r"(?s)(\n\s+)[a-z]\)")?
        .replace_all(&exo, "$1\\item ")
        .into_owned();
    Ok(exo)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let nb_exo = std::env::args().nth(1).expect("Donner le numéro de l'exo");

    let url: String = BASE_URL.to_owned() + &nb_exo;

    let resp = ureq::get(&url).call()?;
    let body = resp.into_reader();

    //let resp = reqwest::blocking::get(url)?;
    let document = Document::from_read(body)?;
    let preambule = preambule(&document);

    let response = parse_document(&document)
        .split(EXO_SEPARATOR)
        .skip(1)
        .map(process_exo)
        .flat_map(|x| x)
        .map(|exo| preambule.clone() + &exo + "\\end{exo}\n")
        .collect();

    alert("Beos scraping", "Copied into clipboard");
    let mut cx: ClipboardContext = ClipboardProvider::new()?;
    cx.set_contents(response)?;

    Ok(())
}

