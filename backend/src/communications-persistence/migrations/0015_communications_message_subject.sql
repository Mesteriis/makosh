ALTER TABLE makosh_data.communications_evidence_summaries
  ADD COLUMN message_subject TEXT CHECK (
    message_subject IS NULL
    OR (
      octet_length(message_subject) BETWEEN 1 AND 998
      AND message_subject = btrim(message_subject)
      AND position(chr(10) IN message_subject) = 0
      AND position(chr(13) IN message_subject) = 0
    )
  );
