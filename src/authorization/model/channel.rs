define_permissions!(ChannelPermission {
  VIEW_CHANNEL         = 1 << 0,
  SEND_MESSAGE         = 1 << 1,
  READ_MESSAGE_HISTORY = 1 << 2,
  MANAGE_CHANNEL       = 1 << 3,
});
