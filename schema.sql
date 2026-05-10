CREATE TABLE IF NOT EXISTS public.accounts (
    id CHAR(26) PRIMARY KEY,

    email TEXT NOT NULL UNIQUE,
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,

    password TEXT NOT NULL,

    birthday DATE NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()

    CHECK (email = lower(email))
);

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
