CREATE TABLE IF NOT EXISTS public.guild_roles (
    id CHAR(26) PRIMARY KEY,
    guild_id CHAR(26) NOT NULL REFERENCES public.guilds(id) ON DELETE CASCADE,
    permissions BIGINT NOT NULL DEFAULT 0,
    name TEXT NOT NULL DEFAULT 'new role',

    UNIQUE (id, guild_id)
);

CREATE TABLE IF NOT EXISTS public.member_roles (
    user_id CHAR(26) NOT NULL,
    guild_id CHAR(26) NOT NULL,
    role_id  CHAR(26) NOT NULL,

    PRIMARY KEY (user_id, guild_id, role_id),
    FOREIGN KEY (guild_id, user_id) REFERENCES public.guild_members(guild_id, user_id) ON DELETE CASCADE,
    FOREIGN KEY (role_id, guild_id) REFERENCES public.guild_roles(id, guild_id) ON DELETE CASCADE
);
