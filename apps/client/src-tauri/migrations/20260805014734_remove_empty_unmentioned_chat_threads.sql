CREATE TEMP TABLE removed_empty_chat_reply_threads (
    id TEXT PRIMARY KEY NOT NULL,
    root_item_id TEXT NOT NULL UNIQUE
) STRICT;

INSERT INTO removed_empty_chat_reply_threads (id, root_item_id)
SELECT thread.id, thread.root_item_id
FROM chat_reply_threads thread
WHERE NOT EXISTS (
    SELECT 1
    FROM chat_conversation_items reply
    WHERE reply.reply_thread_id = thread.id
)
AND NOT EXISTS (
    SELECT 1
    FROM chat_work_assignments assignment
    WHERE assignment.reply_thread_id = thread.id
)
AND NOT EXISTS (
    SELECT 1
    FROM chat_organizational_drafts draft
    WHERE draft.reply_thread_id = thread.id
)
AND NOT EXISTS (
    SELECT 1
    FROM chat_scheduled_messages scheduled
    WHERE scheduled.reply_thread_id = thread.id
)
AND NOT EXISTS (
    SELECT 1
    FROM chat_communication_messages message
    JOIN chat_participant_mentions mention
      ON mention.message_revision_id = message.current_revision_id
    WHERE message.item_id = thread.root_item_id
);

UPDATE chat_organizational_command_receipts
SET result_data = json_set(result_data, '$.replyThreadId', NULL)
WHERE command_kind = 'post_message'
  AND state = 'completed'
  AND json_extract(result_data, '$.message.itemId') IN (
      SELECT root_item_id FROM removed_empty_chat_reply_threads
  )
  AND json_extract(result_data, '$.replyThreadId') IN (
      SELECT id FROM removed_empty_chat_reply_threads
  );

UPDATE chat_organizational_command_receipts
SET result_data = json_set(result_data, '$.Locator.reply_thread_id', NULL)
WHERE command_kind = 'post_message'
  AND state = 'completed'
  AND json_extract(result_data, '$.Locator.message_item_id') IN (
      SELECT root_item_id FROM removed_empty_chat_reply_threads
  )
  AND json_extract(result_data, '$.Locator.reply_thread_id') IN (
      SELECT id FROM removed_empty_chat_reply_threads
  );

DELETE FROM chat_reply_threads
WHERE id IN (SELECT id FROM removed_empty_chat_reply_threads);

DROP TABLE removed_empty_chat_reply_threads;
