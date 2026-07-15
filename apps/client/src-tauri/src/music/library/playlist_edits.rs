use super::*;
use sqlx::SqlitePool;

pub(crate) async fn bulk_edit_memberships(
    pool: &SqlitePool,
    request: MusicBulkMembershipEdit,
) -> MusicLibraryResult<MusicBulkMembershipResult> {
    validate_bulk_membership_edit(&request)?;
    let mut transaction = pool
        .begin()
        .await
        .map_err(|error| MusicLibraryError::database("begin bulk playlist edit", error))?;
    let mut changed_count = 0_i64;
    for playlist_id in &request.add_playlist_ids {
        let mut next_position: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM music_playlist_memberships WHERE playlist_id = ?",
        )
        .bind(playlist_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| MusicLibraryError::database("load next playlist position", error))?;
        for (item_index, item_id) in request.item_ids.iter().enumerate() {
            let membership_id = format!("{}:{}:{}", request.action_id, item_index, playlist_id);
            let result = sqlx::query(
                "INSERT INTO music_playlist_memberships
                    (id, playlist_id, item_id, position, weight, enabled, focus_fit,
                     created_at, updated_at, version)
                 VALUES (?, ?, ?, ?, 'normal', 1, 'unknown', ?, ?, 1)
                 ON CONFLICT(playlist_id, item_id) DO NOTHING",
            )
            .bind(membership_id)
            .bind(playlist_id)
            .bind(item_id)
            .bind(next_position)
            .bind(request.updated_at)
            .bind(request.updated_at)
            .execute(&mut *transaction)
            .await
            .map_err(|error| MusicLibraryError::database("bulk add playlist membership", error))?;
            if result.rows_affected() > 0 {
                next_position += 1;
                changed_count += result.rows_affected() as i64;
                super::search::refresh_item(&mut transaction, item_id).await?;
            }
        }
    }
    for playlist_id in &request.remove_playlist_ids {
        for item_id in &request.item_ids {
            let result = sqlx::query(
                "DELETE FROM music_playlist_memberships WHERE playlist_id = ? AND item_id = ?",
            )
            .bind(playlist_id)
            .bind(item_id)
            .execute(&mut *transaction)
            .await
            .map_err(|error| {
                MusicLibraryError::database("bulk remove playlist membership", error)
            })?;
            if result.rows_affected() > 0 {
                changed_count += result.rows_affected() as i64;
                super::search::refresh_item(&mut transaction, item_id).await?;
            }
        }
    }
    if let Some(weight) = request.weight {
        for playlist_id in &request.weight_playlist_ids {
            for item_id in &request.item_ids {
                let result = sqlx::query(
                    "UPDATE music_playlist_memberships
                     SET weight = ?, updated_at = ?, version = version + 1
                     WHERE playlist_id = ? AND item_id = ?",
                )
                .bind(weight.as_ref())
                .bind(request.updated_at)
                .bind(playlist_id)
                .bind(item_id)
                .execute(&mut *transaction)
                .await
                .map_err(|error| {
                    MusicLibraryError::database("bulk update membership weight", error)
                })?;
                if result.rows_affected() > 0 {
                    changed_count += result.rows_affected() as i64;
                    super::search::refresh_item(&mut transaction, item_id).await?;
                }
            }
        }
    }
    super::writes::commit(transaction, "commit bulk playlist edit").await?;
    Ok(MusicBulkMembershipResult { changed_count })
}

pub(crate) async fn reorder_playlist(
    pool: &SqlitePool,
    request: MusicPlaylistReorder,
) -> MusicLibraryResult<MusicPlaylistReorderResult> {
    validate_playlist_reorder(&request)?;
    let mut transaction = pool
        .begin()
        .await
        .map_err(|error| MusicLibraryError::database("begin playlist reorder", error))?;
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT id, item_id FROM music_playlist_memberships
         WHERE playlist_id = ? ORDER BY position, id",
    )
    .bind(&request.playlist_id)
    .fetch_all(&mut *transaction)
    .await
    .map_err(|error| MusicLibraryError::database("load playlist order", error))?;
    let Some(source_index) = rows
        .iter()
        .position(|(_, item_id)| item_id == &request.item_id)
    else {
        return Err(MusicLibraryError::not_found(
            "playlist membership item",
            &request.item_id,
        ));
    };
    if request.target_index as usize >= rows.len() {
        return Err(MusicLibraryError::validation(
            "targetIndex",
            format!("must be less than {}", rows.len()),
        ));
    }
    let mut reordered = rows;
    let moved = reordered.remove(source_index);
    reordered.insert(request.target_index as usize, moved);
    for (position, (membership_id, _)) in reordered.iter().enumerate() {
        sqlx::query(
            "UPDATE music_playlist_memberships
             SET position = ?, updated_at = ?, version = version + 1 WHERE id = ?",
        )
        .bind(position as i64)
        .bind(request.updated_at)
        .bind(membership_id)
        .execute(&mut *transaction)
        .await
        .map_err(|error| MusicLibraryError::database("persist playlist order", error))?;
    }
    super::writes::commit(transaction, "commit playlist reorder").await?;
    Ok(MusicPlaylistReorderResult {
        item_ids: reordered.into_iter().map(|(_, item_id)| item_id).collect(),
    })
}

pub(crate) async fn bulk_set_review_state(
    pool: &SqlitePool,
    request: MusicBulkReviewWrite,
) -> MusicLibraryResult<MusicBulkMembershipResult> {
    validate_bulk_review_write(&request)?;
    let mut transaction = pool
        .begin()
        .await
        .map_err(|error| MusicLibraryError::database("begin bulk review update", error))?;
    for item in &request.items {
        let updated = sqlx::query(
            "UPDATE music_library_items
             SET review_state = ?, review_changed_at = ?, review_deferred_until = ?,
                 updated_at = ?, version = version + 1
             WHERE id = ? AND version = ?",
        )
        .bind(request.review_state.as_ref())
        .bind(request.updated_at)
        .bind(request.deferred_until)
        .bind(request.updated_at)
        .bind(&item.item_id)
        .bind(item.expected_version)
        .execute(&mut *transaction)
        .await
        .map_err(|error| MusicLibraryError::database("bulk update review state", error))?;
        if updated.rows_affected() == 0 {
            return Err(MusicLibraryError::stale(
                "music library item",
                &item.item_id,
            ));
        }
        super::search::refresh_item(&mut transaction, &item.item_id).await?;
    }
    super::writes::commit(transaction, "commit bulk review update").await?;
    Ok(MusicBulkMembershipResult {
        changed_count: request.items.len() as i64,
    })
}

pub(crate) async fn bulk_snooze(
    pool: &SqlitePool,
    request: MusicBulkSnoozeWrite,
) -> MusicLibraryResult<MusicBulkMembershipResult> {
    validate_bulk_snooze_write(&request)?;
    let mut transaction = pool
        .begin()
        .await
        .map_err(|error| MusicLibraryError::database("begin bulk snooze", error))?;
    for (index, item_id) in request.item_ids.iter().enumerate() {
        sqlx::query(
            "INSERT INTO music_snoozes
                (id, item_id, scope, playlist_id, starts_at, ends_at, reason, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(format!("{}:{}", request.action_id, index))
        .bind(item_id)
        .bind(request.scope.as_ref())
        .bind(&request.playlist_id)
        .bind(request.starts_at)
        .bind(request.ends_at)
        .bind(&request.reason)
        .bind(request.created_at)
        .execute(&mut *transaction)
        .await
        .map_err(|error| MusicLibraryError::database("bulk save snooze", error))?;
        super::search::refresh_item(&mut transaction, item_id).await?;
    }
    super::writes::commit(transaction, "commit bulk snooze").await?;
    Ok(MusicBulkMembershipResult {
        changed_count: request.item_ids.len() as i64,
    })
}
