import json
import sys
import requests
import os
from dotenv import load_dotenv
import pandas as pd
import tqdm
import qdrant_client
import psycopg2

load_dotenv()

api_key = os.environ.get('API_KEY')
api_url = os.environ.get('API_URL')
qdrant_url = os.environ.get('PYTHON_QDRANT_URL')
qdrant_api_key = os.environ.get('QDRANT_API_KEY')
db_url = os.environ.get('DATABASE_URL')
url = api_url + '/index_document'

svd_client = qdrant_client.QdrantClient(url=qdrant_url, api_key=qdrant_api_key, timeout=100000000)

class IndexDocumentRequest:
    def __init__(self, doc_html, story_id, index):
        self.doc_html = doc_html
        self.story_id = story_id
        self.index = index

    def get_payload(self):
        json_payload = {
            'doc_html': self.doc_html,
            'story_id': self.story_id,
            'index': self.index
        }
        return json.dumps(json_payload)
    
    def send_post_request(self):
        stringified_json_payload = self.get_payload()
        headers = {"Content-Type": "application/json", "Authorization": api_key}
        req_result = requests.post(url, data=stringified_json_payload, headers=headers)

        if req_result.status_code != 200:
            req_error = req_result.text
            print(req_error)

def process_erred_story_ids(index):
    errored_story_ids_df = pd.read_csv(f"./erred_stories/errors_{index}.csv")

    for story_id in tqdm.tqdm(errored_story_ids_df['story_id']):
        pg_conn = psycopg2.connect(db_url, sslmode="require")
        pg_cursor = pg_conn.cursor()
        query = f"SELECT doc_html, story_id, index, qdrant_point_id FROM doc_embeddings WHERE story_id = {story_id}"
        pg_cursor.execute(query)
        pg_results = pg_cursor.fetchall()
        pg_cursor.close()
        pg_conn.close()

        for pg_result in pg_results:
            pg_doc_html = pg_result[0]
            pg_story_id = pg_result[1]
            pg_index = pg_result[2]
            pg_qdrant_point_id = pg_result[3]

            qdrant_success = False
            qdrant_point_exists = False
            while not qdrant_success:
                try:
                    qdrant_point_exists = len(svd_client.retrieve(collection_name="doc_embeddings", ids=[str(pg_qdrant_point_id)])) > 0
                    qdrant_success = True
                except:
                    qdrant_success = False

            if not qdrant_point_exists:
                new_request = IndexDocumentRequest(pg_doc_html, pg_story_id, pg_index)
                new_request.send_post_request()

if __name__ == "__main__":
    index = sys.argv[1]
    process_erred_story_ids(index)
