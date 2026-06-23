SET lock_timeout = 0;

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
