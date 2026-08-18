ALTER TABLE chat_teammate_policy_revisions
ADD COLUMN safety_mode TEXT NOT NULL DEFAULT 'ask_for_approval' CHECK (
    safety_mode IN ('ask_for_approval', 'approve_for_me', 'full_access', 'custom')
);
