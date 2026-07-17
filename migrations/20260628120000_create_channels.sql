CREATE TYPE channel_type AS ENUM ('GUILD_TEXT', 'DIRECT', 'GROUP');

CREATE TABLE IF NOT EXISTS public.channels (
    id CHAR(26) PRIMARY KEY,
    type channel_type NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS public.guild_channels (
    channel_id CHAR(26) PRIMARY KEY REFERENCES public.channels(id) ON DELETE CASCADE,
    guild_id CHAR(26) NOT NULL REFERENCES public.guilds(id) ON DELETE CASCADE,
    category_id CHAR(26) REFERENCES public.guild_channels(channel_id) ON DELETE SET NULL,

    position INTEGER NOT NULL DEFAULT 0,
    identifier TEXT NOT NULL,
    display_name TEXT NOT NULL,
    emoji_id CHAR(26),
    topic TEXT,

    CONSTRAINT guild_channels_identifier_format CHECK (identifier ~ '^[a-z0-9_]+$'),
    CONSTRAINT guild_channels_identifier_length CHECK (char_length(identifier) BETWEEN 1 AND 100),
    CONSTRAINT guild_channels_display_name_length CHECK (char_length(display_name) BETWEEN 1 AND 100)
);

CREATE TABLE IF NOT EXISTS public.direct_channels (
    channel_id CHAR(26) PRIMARY KEY REFERENCES public.channels(id) ON DELETE CASCADE,
    user_a CHAR(26) NOT NULL REFERENCES public.users(id),
    user_b CHAR(26) NOT NULL REFERENCES public.users(id),

    CONSTRAINT direct_channels_user_order CHECK (user_a < user_b),
    CONSTRAINT direct_channels_unique_pair UNIQUE (user_a, user_b)
);

CREATE TABLE IF NOT EXISTS public.group_channels (
    channel_id CHAR(26) PRIMARY KEY REFERENCES public.channels(id) ON DELETE CASCADE,
    owner_id CHAR(26) NOT NULL REFERENCES public.users(id),
    display_name TEXT NOT NULL,
    CONSTRAINT group_channels_display_name_length CHECK (char_length(display_name) BETWEEN 1 AND 100)
);

CREATE INDEX IF NOT EXISTS idx_guild_channels_guild_id ON public.guild_channels(guild_id);
CREATE INDEX IF NOT EXISTS idx_guild_channels_category_id ON public.guild_channels(category_id);
CREATE INDEX IF NOT EXISTS idx_guild_channels_guild_category_position ON public.guild_channels (guild_id, category_id, position DESC);

CREATE INDEX IF NOT EXISTS idx_direct_channels_user_a ON public.direct_channels(user_a);
CREATE INDEX IF NOT EXISTS idx_direct_channels_user_b ON public.direct_channels(user_b);

CREATE INDEX IF NOT EXISTS idx_group_channels_owner_id ON public.group_channels(owner_id);
