-- Drop page_queue table
DROP INDEX IF EXISTS idx_page_queue_job_id;
DROP INDEX IF EXISTS idx_page_queue_status;
DROP INDEX IF EXISTS idx_page_queue_job_status;
DROP TABLE IF EXISTS page_queue;
