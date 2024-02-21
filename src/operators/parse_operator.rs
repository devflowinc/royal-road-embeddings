use std::cmp;

use regex::Regex;
use regex_split::RegexSplit;
use scraper::Html;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ParseCallReturn {
    chunks: Vec<String>,
}

pub fn chunk_document(document: String) -> Vec<String> {
    let document_without_newlines = document.replace("\n", " ");
    let dom = Html::parse_fragment(&document_without_newlines);

    // get the raw text from the HTML
    let clean_text = dom.root_element().text().collect::<String>();

    // split the text into sentences
    let split_sentence_regex = Regex::new(r"[.!?]+").expect("Invalid regex");
    let mut sentences: Vec<&str> = split_sentence_regex
        .split_inclusive_left(&clean_text)
        .collect();

    let mut groups: Vec<String> = vec![];
    let min_group_size = 10;

    if sentences.len() < min_group_size {
        groups.push(sentences.join(""));
        groups.retain(|x| x != "");
        return groups;
    }

    let mut remainder = (sentences.len() % min_group_size) as f32;
    let group_count = ((sentences.len() / min_group_size) as f32).floor();
    let remainder_per_group = (remainder / group_count).ceil();

    while remainder > 0.0 {
        let group_size =
            min_group_size + cmp::min(remainder as usize, remainder_per_group as usize) as usize;
        let group = sentences
            .iter()
            .take(group_size)
            .map(|x| *x)
            .collect::<Vec<&str>>()
            .join(" ");
        groups.push(group);
        sentences.drain(0..group_size);
        remainder -= remainder_per_group;
    }

    while sentences.len() > 0 {
        let group = sentences
            .iter()
            .take(min_group_size)
            .map(|x| *x)
            .collect::<Vec<&str>>()
            .join("");
        groups.push(group);
        sentences.drain(0..min_group_size);
    }

    groups.retain(|x| x != "");
    groups
}
