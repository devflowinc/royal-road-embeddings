import os
from dotenv import load_dotenv
import qdrant_client
from qdrant_client.models import Filter, FieldCondition, Range, MatchValue
import psycopg2
import csv
import pandas as pd
import tqdm
import sys

load_dotenv()
qdrant_url = os.environ.get('PYTHON_QDRANT_URL')
qdrant_api_key = os.environ.get('QDRANT_API_KEY')
db_url = os.environ.get('DATABASE_URL')

svd_client = qdrant_client.QdrantClient(url=qdrant_url, api_key=qdrant_api_key, timeout=100000000)

def get_errors(index):
    output_file_path = f"./erred_stories/errors_{index}.csv"

    pg_conn = psycopg2.connect(db_url)
    pg_cursor = pg_conn.cursor()

    unique_story_id_chunks = pd.read_csv(f"./distinct_story_id_chunks/chunk_{index}.csv")

    with open(output_file_path, 'w') as output_file:
        csv_writer = csv.writer(output_file)
        csv_writer.writerow(['story_id'])

        for story_id in tqdm.tqdm(unique_story_id_chunks['story_id']):
            query = f"SELECT COUNT(*) FROM doc_embeddings WHERE story_id = {story_id}"
            pg_cursor.execute(query)
            pg_count = pg_cursor.fetchone()[0]

            qdrant_points = ([])
            qdrant_success = False
            while not qdrant_success:
                try:
                    qdrant_points = svd_client.scroll(
                        collection_name="doc_embeddings",
                        scroll_filter=Filter(
                            must=[
                                FieldCondition(
                                    key='story_id',
                                    match=MatchValue(value=str(story_id))
                                )
                            ]
                        ),
                        limit = pg_count
                    )
                    qdrant_success = True
                except:
                    qdrant_success = False

            num_qdrant_points = len(qdrant_points[0])

            if num_qdrant_points < pg_count:
                csv_writer.writerow([story_id])
            
    pg_cursor.close()
    pg_conn.close()

if __name__ == "__main__":
    index = sys.argv[1]
    get_errors(index)

            
