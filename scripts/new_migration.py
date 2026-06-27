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
