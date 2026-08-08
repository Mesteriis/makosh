ALTER TABLE makosh_data.communications_evidence_summaries
  ADD COLUMN body_media_type TEXT CHECK (
    body_media_type IS NULL OR body_media_type IN ('text/plain', 'text/html')
  );
