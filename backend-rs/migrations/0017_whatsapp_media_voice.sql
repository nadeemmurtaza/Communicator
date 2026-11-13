-- WhatsApp Media & Voice Support: Add media columns to messages table

ALTER TABLE messages ADD COLUMN media_type TEXT;
ALTER TABLE messages ADD COLUMN media_url TEXT;
ALTER TABLE messages ADD COLUMN media_filename TEXT;
ALTER TABLE messages ADD COLUMN media_mimetype TEXT;
ALTER TABLE messages ADD COLUMN voice_duration INTEGER;
ALTER TABLE messages ADD COLUMN transcription_text TEXT;

CREATE INDEX IF NOT EXISTS idx_messages_media_type ON messages(media_type);
