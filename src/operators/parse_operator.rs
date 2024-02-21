use regex::Regex;
use regex_split::RegexSplit;
use scraper::Html;
use std::cmp;

pub fn count_tokens(text: &str) -> usize {
    let split_word_regex = Regex::new(r"(\s+)|[^\w\s]").expect("Invalid regex");
    split_word_regex.split(text).count()
}

pub fn split_large_groups_on_chars(cur_chunks: Vec<String>) -> Vec<String> {
    let max_chars_in_chunk = 30000;

    let mut chunks = cur_chunks;
    let mut new_chunks: Vec<String> = vec![];
    for chunk in chunks.iter_mut() {
        if chunk.len() < max_chars_in_chunk {
            new_chunks.push(chunk.to_string());
            continue;
        }

        let char_count = chunk.chars().count() as f32;

        let num_new_chunks = (char_count / max_chars_in_chunk as f32).ceil() as usize;
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

pub fn group_sentences(sentences: Vec<&str>) -> Vec<String> {
    let max_token_size = 5000;
    let mut sentences = sentences;
    let mut groups: Vec<String> = vec![];

    while sentences.len() > 0 {
        let mut sentences_to_remove = 0;
        let mut group = String::new();

        for sentence in sentences.iter() {
            sentences_to_remove += 1;
            let token_count = count_tokens(sentence);

            if count_tokens(group.as_str()) + token_count > max_token_size {
                break;
            }

            group.push_str(sentence);
        }

        let group = sentences.drain(0..sentences_to_remove).collect::<String>();
        groups.push(group);
    }

    groups
}

pub fn chunk_document(document: String) -> Vec<String> {
    let document_without_newlines = document.replace('\n', " ");
    let dom = Html::parse_fragment(&document_without_newlines);

    // get the raw text from the HTML
    let clean_text = dom.root_element().text().collect::<String>();

    // split the text into sentences
    let split_sentence_regex = Regex::new(r"[.!?\n]+").expect("Invalid regex");
    let sentences: Vec<&str> = split_sentence_regex
        .split_inclusive_left(&clean_text)
        .collect();

    let sentence_groups = group_sentences(sentences);

    split_large_groups_on_chars(sentence_groups)
}
