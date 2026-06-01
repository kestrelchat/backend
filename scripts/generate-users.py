import random
import string
import sys
from datetime import date, timedelta

import requests

BASE_URL = "http://localhost:5181"
REGISTER_ENDPOINT = f"{BASE_URL}/auth/register"

PASSWORD = "MyW0rd1sP@ssed!"


def random_string(n=10):
    return "".join(random.choices(string.ascii_lowercase + string.digits, k=n))


def random_email():
    return f"{random_string(8)}@localhost"


def random_username():
    return f"dev_{random_string(6)}"


def random_birthday(min_age=18, max_age=40):
    today = date.today()
    age = random.randint(min_age, max_age)
    birth_year = today.year - age

    start = date(birth_year, 1, 1)
    end = date(birth_year, 12, 28)
    delta = (end - start).days
    return (start + timedelta(days=random.randint(0, delta))).isoformat()


def create_user(i):
    username = random_username()

    payload = {
        "email": random_email(),
        "username": username,
        "password": PASSWORD,
        "birthday": random_birthday(),
    }

    r = requests.post(REGISTER_ENDPOINT, json=payload)

    if r.status_code == 200:
        data = r.json()
        print(
            f"[{i}] Created user {username} with id {data.get('id')} ({data.get('email')})"
        )
    else:
        print(f"[{i}] Failed ({r.status_code}): {r.text}")


def main():
    if len(sys.argv) < 2:
        print("Usage: python generateusers.py <count>")
        sys.exit(1)

    try:
        count = int(sys.argv[1])
    except ValueError:
        print("Count must be an integer")
        sys.exit(1)

    print(f"Creating {count} users...\n")

    for i in range(count):
        create_user(i)

    print(f"\nDone. Default password for all created users: {PASSWORD}")


if __name__ == "__main__":
    main()
