-- PostgreSQL full-text search for ai_record (DATABASE_SPEC / architecture design alignment).

ALTER TABLE ai_record ADD COLUMN IF NOT EXISTS search_document TSVECTOR;

CREATE INDEX IF NOT EXISTS idx_ai_record_search_document
  ON ai_record USING GIN (search_document);

UPDATE ai_record
SET search_document = to_tsvector(
  'simple',
  coalesce(canonical_text, '') || ' ' ||
  coalesce(object_text, '') || ' ' ||
  coalesce(subject, '') || ' ' ||
  coalesce(predicate, '')
)
WHERE search_document IS NULL
  AND status <> 'deleted';

CREATE OR REPLACE FUNCTION ai_record_search_document_trigger() RETURNS trigger AS $$
BEGIN
  NEW.search_document := to_tsvector(
    'simple',
    coalesce(NEW.canonical_text, '') || ' ' ||
    coalesce(NEW.object_text, '') || ' ' ||
    coalesce(NEW.subject, '') || ' ' ||
    coalesce(NEW.predicate, '')
  );
  RETURN NEW;
END
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_ai_record_search_document ON ai_record;
CREATE TRIGGER trg_ai_record_search_document
  BEFORE INSERT OR UPDATE OF canonical_text, object_text, subject, predicate, status
  ON ai_record
  FOR EACH ROW
  WHEN (NEW.status <> 'deleted')
  EXECUTE FUNCTION ai_record_search_document_trigger();
