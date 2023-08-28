-- Add up migration script here
CREATE TABLE doc_embeddings (
    id UUID NOT NULL UNIQUE PRIMARY KEY,
    doc_html TEXT NOT NULL,
    story_id BIGINT NOT NULL,
    doc_num BIGINT NOT NULL,
    qdrant_point_id UUID NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT unique_story_id_doc_num_pair UNIQUE (story_id, doc_num)
);
 
CREATE TRIGGER update_updated_at
BEFORE UPDATE ON doc_embeddings
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();
