INSERT INTO chat_messages (
    id,
    thread_id,
    turn_id,
    sequence_anchor,
    role,
    normalized_markdown,
    streaming_state,
    provider_item_id,
    content_metadata_schema_version,
    content_metadata_data,
    created_at,
    updated_at
)
SELECT
    id,
    thread_id,
    turn_id,
    sequence_anchor,
    'assistant',
    COALESCE(detail, ''),
    CASE status
        WHEN 'completed' THEN 'complete'
        WHEN 'interrupted' THEN 'interrupted'
        WHEN 'failed' THEN 'failed'
        WHEN 'pending' THEN 'pending'
        ELSE 'streaming'
    END,
    provider_item_id,
    safe_metadata_schema_version,
    safe_metadata_data,
    created_at,
    updated_at
FROM chat_activities
WHERE item_kind = 'assistant_message'
ON CONFLICT(id) DO UPDATE SET
    normalized_markdown = CASE
        WHEN excluded.normalized_markdown = '' THEN chat_messages.normalized_markdown
        ELSE excluded.normalized_markdown
    END,
    streaming_state = excluded.streaming_state,
    content_metadata_schema_version = excluded.content_metadata_schema_version,
    content_metadata_data = excluded.content_metadata_data,
    updated_at = excluded.updated_at;

DELETE FROM chat_activities
WHERE item_kind IN ('assistant_message', 'user_message');

WITH diff_candidates AS (
    SELECT
        t.id AS turn_id,
        -1 AS sequence,
        json_extract(file.value, '$.relativePath') AS relative_path,
        file.value AS file_data
    FROM chat_turns t
    JOIN json_each(COALESCE(t.changed_file_summary_data, '[]')) file
    UNION ALL
    SELECT
        e.turn_id,
        e.sequence,
        json_extract(file.value, '$.relativePath') AS relative_path,
        file.value AS file_data
    FROM chat_events e
    JOIN json_each(json_extract(e.payload_data, '$.payload.files')) file
    WHERE e.event_type = 'diff_updated'
      AND e.turn_id IS NOT NULL
      AND e.invalidated_at IS NULL
), ranked_diff_candidates AS (
    SELECT
        turn_id,
        relative_path,
        file_data,
        row_number() OVER (
            PARTITION BY turn_id, relative_path
            ORDER BY sequence DESC
        ) AS rank
    FROM diff_candidates
    WHERE relative_path IS NOT NULL
)
UPDATE chat_turns
SET changed_file_summary_schema_version = 1,
    changed_file_summary_data = COALESCE((
        SELECT json_group_array(json(file_data))
        FROM ranked_diff_candidates
        WHERE ranked_diff_candidates.turn_id = chat_turns.id
          AND rank = 1
    ), '[]')
WHERE EXISTS (
    SELECT 1
    FROM ranked_diff_candidates
    WHERE ranked_diff_candidates.turn_id = chat_turns.id
      AND rank = 1
);
