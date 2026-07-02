use crate::notes::validation::require_uuid;
use std::collections::HashSet;

pub(super) async fn new_note_id(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    reserved_ids: &mut HashSet<String>,
) -> Result<String, String> {
    for _ in 0..32 {
        let id: String = sqlx::query_scalar(
            "SELECT lower(hex(randomblob(4))) || '-' ||
                    lower(hex(randomblob(2))) || '-' ||
                    lower(hex(randomblob(2))) || '-' ||
                    lower(hex(randomblob(2))) || '-' ||
                    lower(hex(randomblob(6)))",
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| format!("generate notes id: {e}"))?;
        require_uuid(&id, "generated_id")?;
        if reserved_ids.contains(&id) {
            continue;
        }
        let exists: Option<i64> = sqlx::query_scalar(
            "SELECT 1
             WHERE EXISTS (SELECT 1 FROM notes_pages WHERE id = ?)
                OR EXISTS (SELECT 1 FROM notes_blocks WHERE id = ?)
                OR EXISTS (SELECT 1 FROM notes_comment_threads WHERE id = ?)
                OR EXISTS (SELECT 1 FROM notes_comments WHERE id = ?)",
        )
        .bind(&id)
        .bind(&id)
        .bind(&id)
        .bind(&id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| format!("check generated notes id: {e}"))?;
        if exists.is_none() {
            reserved_ids.insert(id.clone());
            return Ok(id);
        }
    }
    Err("could not generate a unique notes id".to_string())
}
