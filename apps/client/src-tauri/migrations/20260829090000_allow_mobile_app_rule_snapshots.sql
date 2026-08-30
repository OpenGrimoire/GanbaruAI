CREATE TABLE doomscrolling_block_event_rule_snapshots_new (
    block_event_id TEXT PRIMARY KEY REFERENCES doomscrolling_block_events(id) ON DELETE CASCADE,
    rule_id TEXT,
    rule_kind TEXT CHECK (
        rule_kind IS NULL OR
        rule_kind IN (
            'domain',
            'url_pattern',
            'category',
            'custom_category',
            'usage_limit',
            'desktop_app',
            'mobile_app'
        )
    ),
    rule_label TEXT,
    environment_id TEXT,
    blocker_mode TEXT CHECK (
        blocker_mode IS NULL OR
        blocker_mode IN ('blacklist', 'whitelist', 'limit')
    )
);

INSERT INTO doomscrolling_block_event_rule_snapshots_new (
    block_event_id,
    rule_id,
    rule_kind,
    rule_label,
    environment_id,
    blocker_mode
)
SELECT
    block_event_id,
    rule_id,
    rule_kind,
    rule_label,
    environment_id,
    blocker_mode
FROM doomscrolling_block_event_rule_snapshots;

DROP TABLE doomscrolling_block_event_rule_snapshots;

ALTER TABLE doomscrolling_block_event_rule_snapshots_new
RENAME TO doomscrolling_block_event_rule_snapshots;
