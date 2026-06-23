SET lock_timeout = 0;

CREATE INDEX idx_sessions_user_id on sessions(user_id);
