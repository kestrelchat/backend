CREATE TABLE IF NOT EXISTS public.group_members (
    channel_id CHAR(26) NOT NULL REFERENCES public.channels(id) ON DELETE CASCADE,
    user_id CHAR(26) NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,

    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (channel_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_group_members_user_id
    ON group_members (user_id);
