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
