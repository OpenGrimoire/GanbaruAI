WITH ordered_output AS (
    SELECT
        event.thread_id,
        json_extract(event.payload_data, '$.payload.itemId') AS item_id,
        json_extract(event.payload_data, '$.payload.delta') AS output_delta
    FROM chat_events AS event
    WHERE event.event_type = 'content_delta'
      AND json_extract(event.payload_data, '$.payload.streamKind') IN (
          'command_output',
          'file_change_output'
      )
    ORDER BY event.thread_id, item_id, event.sequence
), restored_output AS (
    SELECT thread_id, item_id, group_concat(output_delta, '') AS detail
    FROM ordered_output
    GROUP BY thread_id, item_id
)
UPDATE chat_activities
SET detail = (
    SELECT restored_output.detail
    FROM restored_output
    WHERE restored_output.thread_id = chat_activities.thread_id
      AND restored_output.item_id = chat_activities.id
)
WHERE COALESCE(detail, '') = ''
  AND item_kind IN ('command_execution', 'file_change')
  AND EXISTS (
      SELECT 1
      FROM restored_output
      WHERE restored_output.thread_id = chat_activities.thread_id
        AND restored_output.item_id = chat_activities.id
  );
