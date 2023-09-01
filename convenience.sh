#!/bin/bash

echo "Resetting the Qdrant database..."
sudo docker compose stop qdrant-database
sudo docker compose rm -f qdrant-database
sudo docker volume rm royal-road-embeddings_qdrant_data
sudo docker compose up -d qdrant-database
sqlx database reset
