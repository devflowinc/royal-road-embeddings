use regex::Regex;
use regex_split::RegexSplit;
use scraper::Html;
use std::cmp;

pub fn remove_large_chunks(cur_chunks: Vec<String>) -> Vec<String> {
    let max_chunk_len = 10000;
    let mut chunks = cur_chunks;
    let mut new_chunks: Vec<String> = vec![];
    for chunk in chunks.iter_mut() {
        if chunk.len() < max_chunk_len {
            new_chunks.push(chunk.to_string());
            continue;
        }

        let char_count = chunk.chars().count() as f32;

        let num_new_chunks = (char_count / max_chunk_len as f32).ceil() as usize;
        let chunk_size = (char_count / num_new_chunks as f32).ceil();
        let mut total_length = char_count;

        while total_length > 0.0 {
            let amt_to_take = cmp::min(chunk_size as usize, total_length as usize);
            let new_chunk = chunk.chars().take(amt_to_take).collect::<String>();
            new_chunks.push(new_chunk);
            chunk.drain(0..amt_to_take as usize);
            total_length -= amt_to_take as f32;
        }
    }

    new_chunks.retain(|x| !x.is_empty());
    new_chunks
}

pub fn chunk_document(document: String) -> Vec<String> {
    let document_without_newlines = document.replace('\n', " ");
    let dom = Html::parse_fragment(&document_without_newlines);

    // get the raw text from the HTML
    let clean_text = dom.root_element().text().collect::<String>();

    // split the text into sentences
    let split_sentence_regex = Regex::new(r"[.!?\n]+").expect("Invalid regex");
    let mut sentences: Vec<&str> = split_sentence_regex
        .split_inclusive_left(&clean_text)
        .collect();

    let mut groups: Vec<String> = vec![];
    let target_group_size = 30;

    if sentences.len() < target_group_size {
        groups.push(sentences.join(""));
        return remove_large_chunks(groups);
    }

    let mut remainder = (sentences.len() % target_group_size) as f32;
    let group_count = ((sentences.len() / target_group_size) as f32).floor();
    let remainder_per_group = (remainder / group_count).ceil();

    while remainder > 0.0 {
        let group_size =
            target_group_size + cmp::min(remainder as usize, remainder_per_group as usize) as usize;
        let group = sentences
            .iter()
            .take(group_size)
            .copied()
            .collect::<Vec<&str>>()
            .join("");
        groups.push(group);
        sentences.drain(0..group_size);
        remainder -= remainder_per_group as f32;
    }

    while !sentences.is_empty() {
        let group = sentences
            .iter()
            .take(target_group_size)
            .copied()
            .collect::<Vec<&str>>()
            .join("");
        groups.push(group);
        sentences.drain(0..target_group_size);
    }

    remove_large_chunks(groups)
}
