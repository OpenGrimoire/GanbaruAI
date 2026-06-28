const DEFAULT_STATUSES: &[(&str, &str, &str, i64, i64, i64)] = &[
    ("backlog", "Backlog", "not_started", 30, 0, 0),
    ("todo", "To do", "not_started", 31, 10, 0),
    ("in-progress", "In progress", "active", 19, 20, 0),
    ("in-review", "In review", "active", 23, 30, 0),
    ("blocked", "Blocked", "blocked", 2, 40, 0),
    ("done", "Done", "done", 13, 50, 1),
];

const DEFAULT_PRIORITIES: &[(&str, &str, i64, i64)] = &[
    ("low", "Low", 30, 0),
    ("normal", "Normal", 19, 10),
    ("high", "High", 7, 20),
    ("urgent", "Urgent", 2, 30),
];

const BLANK_TEMPLATE_SECTIONS: &[(&str, &str, i64)] = &[("general", "General", 0)];
const SOFTWARE_TEMPLATE_SECTIONS: &[(&str, &str, i64)] = &[
    ("planning", "Planning", 0),
    ("design", "Design", 10),
    ("implementation", "Implementation", 20),
    ("testing", "Testing", 30),
    ("release", "Release", 40),
];
const COURSE_TEMPLATE_SECTIONS: &[(&str, &str, i64)] = &[
    ("general", "General", 0),
    ("assignments", "Assignments", 10),
    ("exams", "Exams", 20),
    ("reading", "Reading", 30),
];
const ROUTINE_TEMPLATE_SECTIONS: &[(&str, &str, i64)] = &[
    ("morning", "Morning", 0),
    ("afternoon", "Afternoon", 10),
    ("evening", "Evening", 20),
];
const READING_TEMPLATE_SECTIONS: &[(&str, &str, i64)] = &[
    ("to-read", "To read", 0),
    ("reading", "Reading", 10),
    ("finished", "Finished", 20),
];
const CHORES_TEMPLATE_SECTIONS: &[(&str, &str, i64)] = &[
    ("general", "General", 0),
    ("daily", "Daily", 10),
    ("weekly", "Weekly", 20),
    ("monthly", "Monthly", 30),
];

fn project_template_sections(template_id: &str) -> &'static [(&'static str, &'static str, i64)] {
    match template_id {
        "software" => SOFTWARE_TEMPLATE_SECTIONS,
        "course" => COURSE_TEMPLATE_SECTIONS,
        "routine" => ROUTINE_TEMPLATE_SECTIONS,
        "reading" => READING_TEMPLATE_SECTIONS,
        "chores" => CHORES_TEMPLATE_SECTIONS,
        _ => BLANK_TEMPLATE_SECTIONS,
    }
}

pub(super) async fn insert_template_sections(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    project_id: &str,
    template_id: &str,
) -> Result<(), String> {
    for (slug, name, sort_order) in project_template_sections(template_id.trim()) {
        sqlx::query(
            "INSERT INTO project_sections (id, project_id, name, sort_order)
             VALUES (?, ?, ?, ?)",
        )
        .bind(format!("section-{project_id}-{slug}"))
        .bind(project_id)
        .bind(name)
        .bind(sort_order)
        .execute(&mut **tx)
        .await
        .map_err(|e| format!("create project template section: {e}"))?;
    }
    Ok(())
}

pub(super) async fn insert_default_statuses(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    project_id: &str,
) -> Result<(), String> {
    for (slug, name, category, color, sort_order, terminal) in DEFAULT_STATUSES {
        sqlx::query(
            "INSERT INTO project_statuses (id, project_id, name, category, color, sort_order, terminal)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(format!("status-{project_id}-{slug}"))
        .bind(project_id)
        .bind(name)
        .bind(category)
        .bind(color)
        .bind(sort_order)
        .bind(terminal)
        .execute(&mut **tx)
        .await
        .map_err(|e| format!("create default project status: {e}"))?;
    }
    Ok(())
}

pub(super) async fn insert_default_priorities(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    project_id: &str,
) -> Result<(), String> {
    for (id, name, color, sort_order) in DEFAULT_PRIORITIES {
        sqlx::query(
            "INSERT INTO project_priorities (id, project_id, name, color, sort_order)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(project_id)
        .bind(name)
        .bind(color)
        .bind(sort_order)
        .execute(&mut **tx)
        .await
        .map_err(|e| format!("create default project priority: {e}"))?;
    }
    Ok(())
}
