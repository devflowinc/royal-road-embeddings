-- Add up migration script here
CREATE TABLE doc_group_embeddings (
    id UUID NOT NULL UNIQUE PRIMARY KEY,
    story_id BIGINT NOT NULL,
    doc_group_size INTEGER NOT NULL,
    index INTEGER NOT NULL,
    qdrant_point_id UUID NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT unique_story_id_group_size_index UNIQUE (story_id, doc_group_size, index)
);
 
CREATE TRIGGER update_updated_at
BEFORE UPDATE ON doc_group_embeddings
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();
