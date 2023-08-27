-- Add down migration script here
DROP TRIGGER IF EXISTS update_updated_at ON doc_embeddings;

DROP TABLE IF EXISTS doc_embeddings;
