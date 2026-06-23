# Kestrel - a modern instant-messaging service written in Rust
# Copyright (C) 2026 Kestrel Chat
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License as published
# by the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
# GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License
# along with this program. If not, see <https://www.gnu.org/licenses/>.

import sys
from datetime import datetime, timezone
from pathlib import Path

migrations = Path("./migrations")
migrations.mkdir(exist_ok=True)

if len(sys.argv) < 2:
    print("usage: py new_migration.py {<migration_name>}")
    sys.exit(1)

name = "_".join(sys.argv[1:]).lower()
timestamp = datetime.now(timezone.utc).strftime("%Y%m%d%H%M%S")
file = migrations / f"{timestamp}_{name}.sql"
file.touch(exist_ok=True)

print(f"created migration: {file.name}")
