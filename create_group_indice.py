#!/usr/bin/env python3

import requests
import os
from dotenv import load_dotenv
import psycopg2
from tqdm import tqdm
import sys
import pandas as pd

load_dotenv()
api_key = os.environ.get('API_KEY')
api_url = os.environ.get('API_URL')
db_url = os.environ.get('DATABASE_URL')

def create_document_group(group_size):
    request_body = {
        "doc_group_size": group_size
    }

    headers = {"Content-Type": "application/json", "Authorization": api_key}
    response = requests.post(api_url + "/document_group", json=request_body, headers=headers)
    return response.status_code

def index_document_group(story_id, group_size):
    request_body = {
        "story_id": story_id,
        "doc_group_size": group_size
    }

    headers = {"Content-Type": "application/json", "Authorization": api_key}
    response = requests.put(api_url + "/document_group", json=request_body, headers=headers)
    return response.status_code

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python foo.py <offset>")
        sys.exit(1)

    batch_size = 10000

    offset = int(sys.argv[1]) * batch_size

    query = f"SELECT DISTINCT story_id FROM doc_embeddings OFFSET {offset}"

    conn = psycopg2.connect(db_url)

    cursor = conn.cursor()

    cursor.execute(query)

    results = cursor.fetchmany(batch_size)
    
    for result in tqdm(results, desc="Processing results"):
        story_id = result
        index_document_group(story_id, 50)

    cursor.close()
    conn.close()
