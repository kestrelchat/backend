use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ChannelsConfig {
  #[serde(default = "default_group_member_limit")]
  pub group_member_limit: i64,
}

fn default_group_member_limit() -> i64 {
  600
}

impl Default for ChannelsConfig {
  fn default() -> Self {
    Self {
      group_member_limit: default_group_member_limit(),
    }
  }
}
