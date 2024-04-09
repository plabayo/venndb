import random

# Define constants
POOLS = [
    "poolA",
    "poolB",
    "poolC",
    "poolD",
    "poolE",
    "poolF",
    "poolG",
    "poolH",
    "poolI",
    "poolJ",
]
COUNTRIES = ["US", "CA", "GB", "AU", "JP", "DE", "FR", "IT", "ES", "NL"]


# Define function to generate random IPv4 address
def random_ipv4():
    return ".".join(str(random.randint(0, 255)) for _ in range(4))


known_proxies = set()


# Define function to generate random row
def random_row():
    while "true":
        id = random.randint(1, 4294967295)
        if id not in known_proxies:
            known_proxies.add(id)
            break

    address = f"{random_ipv4()}:{random.randint(1000, 9999)}"
    username = f"user{random.randint(1, 100)}"
    password = f"pass{random.randint(1, 100)}"
    tcp = random.choice(["true", "false"])
    udp = random.choice(["true", "false"])
    http = random.choice(["true", "false"])
    socks5 = random.choice(["true", "false"])
    datacenter = random.choice(["true", "false"])
    residential = random.choice(["true", "false"])
    mobile = random.choice(["true", "false"])
    pool = random.choice(POOLS)
    if pool in ["poolA", "poolB"]:
        country = "US"
    if pool == "poolJ":
        country = "BE"
    elif pool == "poolB":
        country = random.choice(["US", "CA"])
    elif pool == "poolC":
        country = random.choice(["US", "CA", "GB"])
    elif pool == "poolD":
        country = random.choice(["US", "CA", "GB", "AU"])
    elif pool == "poolE":
        country = random.choice(COUNTRIES[:5])
    else:
        country = random.choice(COUNTRIES)
    return [
        id,
        address,
        username,
        password,
        tcp,
        udp,
        http,
        socks5,
        datacenter,
        residential,
        mobile,
        pool,
        country,
    ]


# Generate CSV rows
for i in range(100000):
    row = random_row()
    print(",".join(str(col) for col in row))
