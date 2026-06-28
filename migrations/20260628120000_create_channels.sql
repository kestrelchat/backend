CREATE TYPE channel_type AS ENUM ('GUILD_TEXT', 'DIRECT', 'GROUP');

CREATE TABLE IF NOT EXISTS public.channels (
    id CHAR(26) PRIMARY KEY,

    type channel_type NOT NULL,

    guild_id CHAR(26) REFERENCES public.guilds(id) ON DELETE CASCADE,
    category_id CHAR(26) REFERENCES public.channels(id) ON DELETE SET NULL,

    position INTEGER NOT NULL DEFAULT 0,

    identifier TEXT NOT NULL,
    display_name TEXT NOT NULL,

    emoji_id CHAR(26) DEFAULT NULL,

    topic TEXT DEFAULT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT channel_identifier_format
        CHECK (identifier ~ '^[a-z0-9_]+$'),
    CONSTRAINT channel_identifier_length
        CHECK (char_length(identifier) BETWEEN 1 AND 100),
    CONSTRAINT channel_display_name_length
        CHECK (char_length(display_name) BETWEEN 1 AND 100)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_channel_identifier_unique_per_guild
    ON channels (guild_id, identifier)
    WHERE category_id IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_channel_identifier_unique_per_category
    ON channels (category_id, identifier)
    WHERE category_id IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_channel_dm_identifier
    ON channels (identifier)
    WHERE type = 'DIRECT';

CREATE INDEX IF NOT EXISTS idx_channels_guild_id
    ON channels (guild_id);
