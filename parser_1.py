#!/usr/bin/env python3
import json
import re
import sys
from bs4 import BeautifulSoup

def remove_html_tags(input_html):
    input_html = input_html.replace('\n', ' ')

    soup = BeautifulSoup(input_html, 'html.parser')

    tables = soup.find_all('table')

    for table in tables:
        table.decompose()

    clean_text = soup.get_text()

    return clean_text

def split_into_groups(sentences: list):
    groups = []
    min_group_size = 10
    remainder = len(sentences) % min_group_size

    while remainder > 0:
        groups.append(sentences[:min_group_size+1])
        sentences = sentences[min_group_size+1:]
        remainder -= 1
    
    while len(sentences) > 0:
        groups.append(sentences[:min_group_size])
        sentences = sentences[min_group_size:]

    return groups


def split_into_chunks(document: str):
    clean_sentences = re.split('[?!.]', document)
    groups = split_into_groups(clean_sentences)

    chunks = []

    while document:
        for group in groups:
            chunk = ''
            for sentence in group:
                sentence_length = len(sentence) + 1
                chunk += document[:sentence_length]
                document = document[sentence_length:]
            chunks.append(chunk)

    return chunks

def chunk_document(document: str):
    clean_text = remove_html_tags(document)
    chunks = split_into_chunks(clean_text)

    return chunks

if __name__ == "__main__":
    line = sys.argv[1]
    print(json.dumps({ "chunks": chunk_document(line) }, ensure_ascii=False))
