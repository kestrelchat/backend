// Kestrel - a modern instant-messaging service written in Rust
// Copyright (C) 2026 Kestrel Chat
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

pub mod create_account;

use chrono::NaiveDate;
pub use create_account::create_account;

use crate::{connection::Database, error::DatabaseError, models::account::Account};

#[async_trait::async_trait]
pub trait AccountOps {
    async fn create_account(
        &self,
        db: &Database,
        email: &str,
        password_hash: &str,
        birthday: NaiveDate,
    ) -> Result<Account, DatabaseError>;
}
