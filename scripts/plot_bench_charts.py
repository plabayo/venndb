from dataclasses import dataclass

# Define input data

input_data = """
proxydb                       fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ naive_proxy_db_100         6.874 µs      │ 19.16 µs      │ 7.478 µs      │ 8.183 µs      │ 100     │ 100
│                             alloc:        │               │               │               │         │
│                               73          │ 73            │ 73            │ 72.27         │         │
│                               377 B       │ 377 B         │ 377 B         │ 373.2 B       │         │
│                             dealloc:      │               │               │               │         │
│                               73          │ 73            │ 73            │ 72.27         │         │
│                               377 B       │ 377 B         │ 377 B         │ 373.2 B       │         │
├─ naive_proxy_db_100_000     3.769 ms      │ 5.285 ms      │ 3.882 ms      │ 4.003 ms      │ 100     │ 100
│                             alloc:        │               │               │               │         │
│                               68635       │ 68635         │ 68635         │ 68635         │         │
│                               324 KB      │ 324 KB        │ 324 KB        │ 324 KB        │         │
│                             dealloc:      │               │               │               │         │
│                               68635       │ 68635         │ 68635         │ 68635         │         │
│                               333.2 KB    │ 333.2 KB      │ 333.2 KB      │ 333.2 KB      │         │
│                             grow:         │               │               │               │         │
│                               13          │ 13            │ 13            │ 13            │         │
│                               9.152 KB    │ 9.152 KB      │ 9.152 KB      │ 9.152 KB      │         │
├─ naive_proxy_db_12_500      402.2 µs      │ 434.3 µs      │ 407.6 µs      │ 409.9 µs      │ 100     │ 100
│                             alloc:        │               │               │               │         │
│                               8515        │ 8515          │ 8515          │ 8515          │         │
│                               40.22 KB    │ 40.22 KB      │ 40.22 KB      │ 40.22 KB      │         │
│                             dealloc:      │               │               │               │         │
│                               8515        │ 8515          │ 8515          │ 8515          │         │
│                               41.44 KB    │ 41.44 KB      │ 41.44 KB      │ 41.44 KB      │         │
│                             grow:         │               │               │               │         │
│                               8           │ 8             │ 8             │ 8             │         │
│                               1.216 KB    │ 1.216 KB      │ 1.216 KB      │ 1.216 KB      │         │
├─ sql_lite_proxy_db_100      34.16 µs      │ 78.04 µs      │ 36.33 µs      │ 37.44 µs      │ 100     │ 100
│                             alloc:        │               │               │               │         │
│                               108         │ 108           │ 108           │ 108           │         │
│                               4.529 KB    │ 4.529 KB      │ 4.529 KB      │ 4.529 KB      │         │
│                             dealloc:      │               │               │               │         │
│                               108         │ 108           │ 108           │ 108           │         │
│                               4.529 KB    │ 4.529 KB      │ 4.529 KB      │ 4.529 KB      │         │
├─ sql_lite_proxy_db_100_000  8.334 ms      │ 10.07 ms      │ 8.628 ms      │ 8.802 ms      │ 100     │ 100
│                             alloc:        │               │               │               │         │
│                               119         │ 119           │ 119           │ 119           │         │
│                               5.023 KB    │ 5.025 KB      │ 5.022 KB      │ 5.024 KB      │         │
│                             dealloc:      │               │               │               │         │
│                               119         │ 119           │ 119           │ 119           │         │
│                               5.023 KB    │ 5.025 KB      │ 5.022 KB      │ 5.024 KB      │         │
├─ sql_lite_proxy_db_12_500   1.099 ms      │ 1.519 ms      │ 1.182 ms      │ 1.209 ms      │ 100     │ 100
│                             alloc:        │               │               │               │         │
│                               119         │ 119           │ 119           │ 119           │         │
│                               5.021 KB    │ 5.025 KB      │ 5.026 KB      │ 5.023 KB      │         │
│                             dealloc:      │               │               │               │         │
│                               119         │ 119           │ 119           │ 119           │         │
│                               5.021 KB    │ 5.025 KB      │ 5.026 KB      │ 5.023 KB      │         │
├─ venn_proxy_db_100          916 ns        │ 8.499 µs      │ 994 ns        │ 1.227 µs      │ 100     │ 400
│                             alloc:        │               │               │               │         │
│                               6           │ 6             │ 6             │ 6             │         │
│                               46 B        │ 46 B          │ 46 B          │ 46 B          │         │
│                             dealloc:      │               │               │               │         │
│                               6           │ 6             │ 6             │ 6             │         │
│                               46 B        │ 46 B          │ 46 B          │ 46 B          │         │
├─ venn_proxy_db_100_000      128.3 µs      │ 152.1 µs      │ 136.5 µs      │ 137.5 µs      │ 100     │ 100
│                             alloc:        │               │               │               │         │
│                               6           │ 6             │ 6             │ 6             │         │
│                               25.02 KB    │ 25.02 KB      │ 25.02 KB      │ 25.02 KB      │         │
│                             dealloc:      │               │               │               │         │
│                               6           │ 6             │ 6             │ 6             │         │
│                               25.02 KB    │ 25.02 KB      │ 25.02 KB      │ 25.02 KB      │         │
╰─ venn_proxy_db_12_500       16.79 µs      │ 23.16 µs      │ 17.54 µs      │ 17.73 µs      │ 100     │ 100
                              alloc:        │               │               │               │         │
                                6           │ 6             │ 6             │ 6             │         │
                                3.15 KB     │ 3.15 KB       │ 3.15 KB       │ 3.15 KB       │         │
                              dealloc:      │               │               │               │         │
                                6           │ 6             │ 6             │ 6             │         │
                                3.15 KB     │ 3.15 KB       │ 3.15 KB       │ 3.15 KB       │         │
"""  # noqa:E501


#####################################################
#### PARSE CODE
#####################################################


@dataclass
# Units are in microseconds
class Performance:
    fastest: float
    slowest: float
    median: float
    mean: float


@dataclass
# Units are in KB
class Allocation:
    fastest: float
    slowest: float
    median: float
    mean: float


results_performance = {}
results_allocation = {}

input_lines = input_data.splitlines()
current_key = None
while True:
    try:
        line = input_lines.pop(0)
    except IndexError:
        break

    line = line.strip()
    if not line or line.startswith("proxydb"):
        continue

    if line.startswith("├") or line.startswith("╰"):
        parts = [
            part
            for part in line.split(" ")
            if part != "" and "│" not in part and "─" not in part
        ]

        current_key = parts[0]

        def get_value(parts, index):
            (value, unit) = parts[index : index + 2]  # noqa:E203
            value = float(value)
            if unit == "ms":
                value *= 1000
            elif unit == "ns":
                value /= 1000
            elif unit != "µs":
                raise ValueError(f"Unexpected unit: {unit}")
            return value

        fastest = get_value(parts, 1)
        slowest = get_value(parts, 3)
        median = get_value(parts, 5)
        mean = get_value(parts, 7)

        results_performance[current_key] = Performance(
            fastest,
            slowest,
            median,
            mean,
        )

        input_lines.pop(0)  # alloc
        input_lines.pop(0)  # count
        line = input_lines.pop(0)

        parts = [
            part
            for part in line.split(" ")
            if part != "" and "│" not in part and "─" not in part
        ]

        def get_value(parts, index):
            (value, unit) = parts[index : index + 2]  # noqa:E203
            value = float(value)
            if unit == "B":
                value /= 1000
            elif unit != "KB":
                raise ValueError(f"Unexpected unit: {unit}")
            return value

        fastest = get_value(parts, 0)
        slowest = get_value(parts, 2)
        median = get_value(parts, 4)
        mean = get_value(parts, 6)

        results_allocation[current_key] = Allocation(
            fastest,
            slowest,
            median,
            mean,
        )



#####################################################
#### PRINTER CODE
#####################################################


def print_performance_chart(results_performance):
    # Find the maximum value for each performance metric
    max_fastest = max(result.fastest for result in results_performance.values())
    max_slowest = max(result.slowest for result in results_performance.values())
    max_median = max(result.median for result in results_performance.values())

    # Print the chart header
    print(f"{'Performance Results'.center(80)}")
    print(f"{'(Units in microseconds)'.center(80)}")
    print(f"{'-' * 80}")

    # Print the chart rows
    for name, result in results_performance.items():
        fastest_bar = "#" * int(result.fastest / max_fastest * 70)
        slowest_bar = "#" * int(result.slowest / max_slowest * 70)
        median_bar = "#" * int(result.median / max_median * 70)

        print(
            f"{name.ljust(40)} |{fastest_bar.ljust(70)}| {result.fastest:.2f} µs (Fastest)"
        )
        print(
            f"{''.ljust(40)} |{median_bar.ljust(70)}| {result.median:.2f} µs (Median)"
        )
        print(
            f"{''.ljust(40)} |{slowest_bar.ljust(70)}| {result.slowest:.2f} µs (Slowest)"
        )
        print(f"{'-' * 80}")


def print_performance_table(results_performance):
    print("| Proxy DB | Fastest (µs) | Median (µs) | Slowest (µs) |")
    print("| --- | --- | --- | --- |")
    for name, result in results_performance.items():
        print(
            f"| {name.ljust(30)} | {result.fastest:.2f} | {result.median:.2f} | {result.slowest:.2f} |"
        )


def print_allocation_table(results_allocation):
    print("| Proxy DB | Fastest (KB) | Median (KB) | Slowest (KB) |")
    print("| --- | --- | --- | --- |")
    for name, result in results_allocation.items():
        print(
            f"| {name.ljust(30)} | {result.fastest:.2f} | {result.median:.2f} | {result.slowest:.2f} |"
        )


#####################################################
#### PRINT PERFORMANCE  OUTPUT
#####################################################


print(
    """
Performance for Database with `100` records:
"""
)

print_performance_table(
    {key: value for (key, value) in results_performance.items() if key.endswith("_100")}
)


print(
    """
Performance for Database with `12_500` records:
"""
)

print_performance_table(
    {
        key: value
        for (key, value) in results_performance.items()
        if key.endswith("_12_500")
    }
)


print(
    """
Performance for Database with `100_000` records:
"""
)

print_performance_table(
    {
        key: value
        for (key, value) in results_performance.items()
        if key.endswith("_100_000")
    }
)

print(
    """
"""
)


#####################################################
#### PRINT Allocation  OUTPUT
#####################################################


print(
    """
Allocations for Database with `100` records:
"""
)

print_allocation_table(
    {key: value for (key, value) in results_allocation.items() if key.endswith("_100")}
)


print(
    """
Allocations for Database with `12_500` records:
"""
)

print_allocation_table(
    {
        key: value
        for (key, value) in results_allocation.items()
        if key.endswith("_12_500")
    }
)


print(
    """
Allocations for Database with `100_000` records:
"""
)

print_allocation_table(
    {
        key: value
        for (key, value) in results_allocation.items()
        if key.endswith("_100_000")
    }
)

print(
    """
"""
)
