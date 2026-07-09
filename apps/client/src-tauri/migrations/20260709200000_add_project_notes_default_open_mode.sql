ALTER TABLE projects
ADD COLUMN notes_default_open_mode TEXT CHECK (
    notes_default_open_mode IS NULL
    OR notes_default_open_mode IN ('center', 'side', 'full')
);
