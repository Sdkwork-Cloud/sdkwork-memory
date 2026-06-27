DROP TRIGGER IF EXISTS trg_ai_record_search_document ON ai_record;
DROP FUNCTION IF EXISTS ai_record_search_document_trigger();
DROP INDEX IF EXISTS idx_ai_record_search_document;
ALTER TABLE ai_record DROP COLUMN IF EXISTS search_document;
