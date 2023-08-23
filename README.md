# royal-road-embeddings

## Docker-compose file

Run docker-compose file
```
sudo docker compose up
```

## Embedding server
Setup python embeddings server locally
```
python -m venv venv
. venv/bin/activate
pip install -r requirements.txt
```
Run embedding server
```
python python_src/embeddings.py
```

## Main server

```
cp .env.dist .env # Set .env file
cargo run
```

## Envirnoment variables
```
API_KEY="key" # The key needed for most routes
EMBEDDING_SERVER_CALL="http://localhost:5000/encode" # The route to call a post to
```
