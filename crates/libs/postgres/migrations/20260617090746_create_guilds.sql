CREATE TABLE IF NOT EXISTS public.guilds (
    id CHAR(26) PRIMARY KEY,

    name TEXT NOT NULL,
    owner_id CHAR(26) NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT guild_name_length CHECK (char_length(name) BETWEEN 1 AND 100)
);

CREATE TABLE IF NOT EXISTS public.guild_members (
    guild_id CHAR(26) NOT NULL REFERENCES public.guilds(id) ON DELETE CASCADE,
    user_id CHAR(26) NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,

    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (guild_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_guild_members_user_id
    ON guild_members (user_id);
