-- remove errors
SET lock_timeout = 0;

-- accounts

CREATE TABLE IF NOT EXISTS public.accounts (
    id CHAR(26) PRIMARY KEY,

    email TEXT NOT NULL UNIQUE,
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,

    password TEXT NOT NULL,

    birthday DATE NOT NULL,

    totp_secret TEXT DEFAULT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()

    CHECK (email = lower(email))
);

-- users

CREATE TABLE IF NOT EXISTS public.users (
    id CHAR(26) PRIMARY KEY REFERENCES public.accounts(id),

    username TEXT NOT NULL,
    discrim VARCHAR(4) NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT username_length CHECK (char_length(username) BETWEEN 2 AND 32),
    CHECK (username = lower(username)),
    CONSTRAINT discrim_format CHECK (discrim ~ '^[a-z0-9]{4}$'),

    CONSTRAINT user_unique_tag UNIQUE (username, discrim)
);

-- sessions

CREATE TABLE IF NOT EXISTS public.sessions (
    id CHAR(26) PRIMARY KEY,
    user_id VARCHAR(26) NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    refresh_token TEXT NOT NULL,

    ip_address TEXT,
    country TEXT,
    region TEXT,
    city TEXT,

    user_agent TEXT,
    operating_system TEXT,
    platform TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '30 days'),

    last_used_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sessions_user_id on sessions(user_id);

-- relationships

CREATE TYPE relationship_type AS ENUM (
    'friend',
    'incoming_request',
    'outgoing_request',
    'block'
);

CREATE TABLE IF NOT EXISTS relationships (
    user_id CHAR(26) NOT NULL,
    target_id CHAR(26) NOT NULL,

    type relationship_type NOT NULL,

    nickname TEXT DEFAULT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (user_id, target_id, type),

    CONSTRAINT fk_relationships_user
        FOREIGN KEY (user_id)
        REFERENCES public.users(id)
        ON DELETE CASCADE,

    CONSTRAINT fk_relationships_target
        FOREIGN KEY (target_id)
        REFERENCES public.users(id)
        ON DELETE CASCADE,

    CONSTRAINT no_self_relation
        CHECK (user_id <> target_id),

    CONSTRAINT nickname_only_for_friends
        CHECK (
            type = 'friend' OR nickname IS NULL
        ),

    CONSTRAINT nickname_length
        CHECK (
            nickname IS NULL
            OR char_length(nickname) BETWEEN 1 AND 32
        )
);

CREATE INDEX IF NOT EXISTS idx_user_relationships_user_id
    ON relationships (user_id);

CREATE INDEX IF NOT EXISTS idx_user_relationships_target_id
    ON relationships (target_id);

CREATE INDEX IF NOT EXISTS idx_user_relationships_user_pair
    ON relationships (user_id, target_id);
