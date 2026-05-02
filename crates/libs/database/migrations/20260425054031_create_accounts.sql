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
