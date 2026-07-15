use super::*;
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};

type AssignmentRow = (
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    String,
    Option<String>,
    i64,
    i64,
);

pub(crate) async fn assignments(
    pool: &SqlitePool,
    owner_kind: MusicAssignmentOwnerKind,
    owner_id: &str,
) -> MusicLibraryResult<Vec<MusicContextAssignment>> {
    validate_id(owner_id, "ownerId")?;
    let rows = sqlx::query_as::<_, AssignmentRow>(
        "SELECT owner_kind, owner_id, phase, behavior, playlist_id, soundscape_id,
                provenance_kind, provenance_id, updated_at, version
         FROM music_context_assignments
         WHERE owner_kind = ? AND owner_id = ?
         ORDER BY CASE phase WHEN 'focus' THEN 0 WHEN 'short-break' THEN 1 ELSE 2 END",
    )
    .bind(owner_kind.as_ref())
    .bind(owner_id)
    .fetch_all(pool)
    .await
    .map_err(|error| MusicLibraryError::database("load music context assignments", error))?;
    rows.into_iter().map(decode).collect()
}

pub(crate) async fn replace_assignments(
    pool: &SqlitePool,
    request: MusicContextAssignmentSet,
) -> MusicLibraryResult<Vec<MusicContextAssignment>> {
    validate_set(&request)?;
    let mut transaction = pool.begin().await.map_err(|error| {
        MusicLibraryError::database("begin music context assignment update", error)
    })?;
    let prior_versions = sqlx::query_as::<_, (String, i64)>(
        "SELECT phase, version FROM music_context_assignments
         WHERE owner_kind = ? AND owner_id = ?",
    )
    .bind(request.owner_kind.as_ref())
    .bind(&request.owner_id)
    .fetch_all(&mut *transaction)
    .await
    .map_err(|error| MusicLibraryError::database("load music context assignment versions", error))?
    .into_iter()
    .collect::<HashMap<_, _>>();
    sqlx::query("DELETE FROM music_context_assignments WHERE owner_kind = ? AND owner_id = ?")
        .bind(request.owner_kind.as_ref())
        .bind(&request.owner_id)
        .execute(&mut *transaction)
        .await
        .map_err(|error| MusicLibraryError::database("replace music context assignments", error))?;
    for assignment in request.assignments {
        let version = prior_versions
            .get(assignment.phase.as_ref())
            .copied()
            .unwrap_or(0)
            + 1;
        sqlx::query(
            "INSERT INTO music_context_assignments
                (owner_kind, owner_id, phase, behavior, playlist_id, soundscape_id,
                 provenance_kind, provenance_id, updated_at, version)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(request.owner_kind.as_ref())
        .bind(&request.owner_id)
        .bind(assignment.phase.as_ref())
        .bind(assignment.behavior.as_ref())
        .bind(assignment.playlist_id)
        .bind(assignment.soundscape_id)
        .bind(assignment.provenance_kind.as_ref())
        .bind(assignment.provenance_id)
        .bind(request.updated_at)
        .bind(version)
        .execute(&mut *transaction)
        .await
        .map_err(|error| MusicLibraryError::database("write music context assignment", error))?;
    }
    super::writes::commit(transaction, "commit music context assignment update").await?;
    assignments(pool, request.owner_kind, &request.owner_id).await
}

fn validate_set(request: &MusicContextAssignmentSet) -> MusicLibraryResult<()> {
    validate_id(&request.owner_id, "ownerId")?;
    if request.updated_at <= 0 {
        return Err(MusicLibraryError::validation(
            "updatedAt",
            "must be positive",
        ));
    }
    if request.assignments.len() > 3 {
        return Err(MusicLibraryError::validation(
            "assignments",
            "cannot contain more than three phases",
        ));
    }
    let mut phases = HashSet::new();
    for assignment in &request.assignments {
        if !phases.insert(assignment.phase) {
            return Err(MusicLibraryError::validation(
                "assignments",
                "cannot contain a phase more than once",
            ));
        }
        validate_optional_id(&assignment.playlist_id, "playlistId")?;
        validate_optional_id(&assignment.soundscape_id, "soundscapeId")?;
        validate_optional_id(&assignment.provenance_id, "provenanceId")?;
        let expected_provenance = match request.owner_kind {
            MusicAssignmentOwnerKind::EventSnapshot => MusicAssignmentProvenanceKind::CopiedProject,
            MusicAssignmentOwnerKind::WorkEnvironment => {
                MusicAssignmentProvenanceKind::WorkEnvironment
            }
            MusicAssignmentOwnerKind::ProjectDefault | MusicAssignmentOwnerKind::EventOverride => {
                MusicAssignmentProvenanceKind::Explicit
            }
        };
        if assignment.provenance_kind != expected_provenance {
            return Err(MusicLibraryError::validation(
                "provenanceKind",
                format!(
                    "{} assignments require {} provenance",
                    request.owner_kind.as_ref(),
                    expected_provenance.as_ref()
                ),
            ));
        }
    }
    Ok(())
}

fn validate_optional_id(value: &Option<String>, field: &str) -> MusicLibraryResult<()> {
    if let Some(value) = value {
        validate_id(value, field)?;
    }
    Ok(())
}

fn decode(row: AssignmentRow) -> MusicLibraryResult<MusicContextAssignment> {
    Ok(MusicContextAssignment {
        owner_kind: MusicAssignmentOwnerKind::try_from(row.0.as_str())
            .map_err(|message| MusicLibraryError::runtime("decode assignment owner", message))?,
        owner_id: row.1,
        phase: MusicActivityPhase::try_from(row.2.as_str())
            .map_err(|message| MusicLibraryError::runtime("decode assignment phase", message))?,
        behavior: MusicAssignmentBehavior::try_from(row.3.as_str())
            .map_err(|message| MusicLibraryError::runtime("decode assignment behavior", message))?,
        playlist_id: row.4,
        soundscape_id: row.5,
        provenance_kind: MusicAssignmentProvenanceKind::try_from(row.6.as_str()).map_err(
            |message| MusicLibraryError::runtime("decode assignment provenance", message),
        )?,
        provenance_id: row.7,
        updated_at: row.8,
        version: row.9,
    })
}
