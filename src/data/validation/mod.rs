pub mod email;
pub mod password;
pub mod username;

pub enum ValidationError {
  Email(email::ValidationError),
  Password(password::ValidationError),
  Username(username::ValidationError),
}
