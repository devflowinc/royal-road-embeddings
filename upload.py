import json
import requests
import os
from dotenv import load_dotenv
import pandas as pd
import redis

load_dotenv()

api_key = os.environ.get('API_KEY')
api_url = os.environ.get('API_URL')
redis_url = os.environ.get('REDIS_URL')
redis_password = os.environ.get('REDIS_PASSWORD')
url = api_url + '/index_document'

redis_client = redis.StrictRedis.from_url(redis_url, decode_responses=True, password=redis_password)

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
        redis_key = f'{self.story_id}-{self.index}'
        if redis_client.get(redis_key) == "done":
            print(f"Already indexed {redis_key}")
            return

        redis_client.set(redis_key, "done")

        stringified_json_payload = self.get_payload()
        headers = {"Content-Type": "application/json", "Authorization": api_key}
        req_result = requests.post(url, data=stringified_json_payload, headers=headers)

        if req_result.status_code != 200:
            req_error = req_result.text
            print(req_error)

def main():
    df = pd.read_pickle('cleaned_normalized_df_no_grouping.pkl')

    i = 0

    for index, row in df.iterrows():
        doc_html = row['content']
        story_id = int(row['FictionId'])
        index = int(row['Order'])
        index_doc_request = IndexDocumentRequest(doc_html, story_id, index)
        index_doc_request.send_post_request()
        i += 1

if __name__ == '__main__':
    main()
