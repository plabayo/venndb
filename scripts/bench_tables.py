"""Turn the output of `just bench-full` into the tables of the README.

Usage: just bench-full | python3 scripts/bench_tables.py
"""

import re
import sys
from collections import OrderedDict

DATABASES = ["VennProxyDB", "RoaringProxyDB", "SqliteProxyDB", "NaiveProxyDB"]
TITLES = OrderedDict(
    [
        ("build", "create the database"),
        ("get_by_id", "look up by key"),
        ("pick/bools", "`tcp + socks5 + datacenter`"),
        ("pick/country", "`tcp + country`"),
        ("pick/pool_country", "`tcp + !mobile + pool + country`"),
        ("pick/city", "`tcp + datacenter + country + city`"),
        ("pick/asn", "`udp + !residential + asn`"),
        ("pick/countries", "`mobile + pool + 3 countries`"),
        ("pick/cities", "`tcp + 5 cities`"),
        ("pick_if/bools", "`tcp + socks5 + datacenter`, not blocked (closure)"),
        ("pick_column/bools", "`tcp + socks5 + datacenter`, not blocked (column)"),
    ]
)

# bench -> rows -> database -> [median time, peak memory]
results = OrderedDict()
path = []
last = None
section = None
for line in sys.stdin:
    node = re.match(r"^([│ ]*)(?:├─|╰─) (\S+)\s*(.*)$", line.rstrip())
    if node:
        depth = len(node.group(1)) // 3
        path = path[:depth] + [node.group(2)]
        columns = [column.strip() for column in node.group(3).split("│")]
        last = section = None
        if len(columns) >= 3 and columns[2]:
            database = path[-2] if path[-2] in DATABASES else DATABASES[0]
            bench = "/".join(p for p in path[:-1] if p not in DATABASES)
            last = [columns[2], "-"]
            results.setdefault(bench, OrderedDict()).setdefault(path[-1], {})[database] = last
        continue
    if last is None:
        continue
    name = re.search(r"(max alloc|alloc|dealloc|grow|shrink):", line)
    size = re.search(r"([0-9.]+ (?:B|KB|MB|GB))\s", line)
    if name:
        section = name.group(1)
    elif section == "max alloc" and size:
        last[1] = size.group(1)


NAMES = [d.removesuffix("ProxyDB").lower() for d in DATABASES]
SIZES = list(results.get("build", {}))


def row(label, databases, column):
    cells = [databases.get(d, ["-", "-"])[column] for d in DATABASES]
    print(f"| {label} | " + " | ".join(cells) + " |")


for size in SIZES:
    print(f"\nDatabase with `{int(size):_}` records:\n")
    print("| | " + " | ".join(NAMES) + " |")
    print("| - |" + " - |" * len(DATABASES))
    for bench, title in TITLES.items():
        row(title, results.get(bench, {}).get(size, {}), 0)

print("\nPeak memory to create the database, rows not included:\n")
print("| records | " + " | ".join(NAMES[:2]) + " |")
print("| - | - | - |")
for size in SIZES:
    databases = results["build"][size]
    cells = [databases.get(d, ["-", "-"])[1] for d in DATABASES[:2]]
    print(f"| `{int(size):_}` | " + " | ".join(cells) + " |")
