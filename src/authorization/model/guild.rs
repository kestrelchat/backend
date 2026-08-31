define_permissions!(GuildPermission {
  ADMINISTRATOR    = 1 << 0,
  MANAGE_GUILD     = 1 << 1,
  MANAGE_ROLES     = 1 << 2,
  MANAGE_CHANNELS  = 1 << 3,
  KICK_MEMBERS     = 1 << 4,
  BAN_MEMBERS      = 1 << 5,
  CREATE_INVITE    = 1 << 10,
  CHANGE_NICKNAME  = 1 << 11,
  MANAGE_NICKNAMES = 1 << 12,
});
