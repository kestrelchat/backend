import os

import tomllib

deps = set()

project_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

for root, _, files in os.walk(project_root):
    if "Cargo.toml" in files:
        path = os.path.join(root, "Cargo.toml")
        try:
            with open(path, "rb") as f:
                data = tomllib.load(f)

            for section in ["dependencies", "dev-dependencies", "build-dependencies"]:
                if section in data:
                    for name, value in data[section].items():
                        if isinstance(value, dict) and "package" in value:
                            deps.add(value["package"])
                        else:
                            deps.add(name)

        except Exception as e:
            print(f"Failed to read {path}: {e}")

for dep in sorted(deps):
    print(dep)
