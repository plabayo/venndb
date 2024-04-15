from dataclasses import dataclass

# Define input data

input_data = """
proxydb                       fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ naive_proxy_db_100         6.499 µs      │ 18.04 µs      │ 7.999 µs      │ 7.929 µs      │ 100     │ 100
│                             alloc:        │               │               │               │         │
│                               71          │ 71            │ 71            │ 70.29         │         │
│                               334 B       │ 334 B         │ 334 B         │ 330.6 B       │         │
│                             dealloc:      │               │               │               │         │
│                               71          │ 71            │ 71            │ 70.29         │         │
│                               334 B       │ 334 B         │ 334 B         │ 330.6 B       │         │
├─ naive_proxy_db_100_000     3.79 ms       │ 5.731 ms      │ 3.837 ms      │ 3.9 ms        │ 100     │ 100
│                             alloc:        │               │               │               │         │
│                               68046       │ 68047         │ 68046         │ 68046         │         │
│                               323.3 KB    │ 323.7 KB      │ 323.3 KB      │ 323.3 KB      │         │
│                             dealloc:      │               │               │               │         │
│                               68046       │ 68046         │ 68046         │ 68046         │         │
│                               328.4 KB    │ 328.4 KB      │ 328.4 KB      │ 328.4 KB      │         │
│                             grow:         │               │               │               │         │
│                               12          │ 12            │ 12            │ 12            │         │
│                               5.056 KB    │ 5.056 KB      │ 5.056 KB      │ 5.056 KB      │         │
├─ naive_proxy_db_12_500      404 µs        │ 478.7 µs      │ 407.7 µs      │ 412.7 µs      │ 100     │ 100
│                             alloc:        │               │               │               │         │
│                               8549        │ 8549          │ 8549          │ 8549          │         │
│                               40.73 KB    │ 40.73 KB      │ 40.73 KB      │ 40.73 KB      │         │
│                             dealloc:      │               │               │               │         │
│                               8549        │ 8549          │ 8549          │ 8549          │         │
│                               41.31 KB    │ 41.31 KB      │ 41.31 KB      │ 41.31 KB      │         │
│                             grow:         │               │               │               │         │
│                               6           │ 6             │ 6             │ 6             │         │
│                               576 B       │ 576 B         │ 576 B         │ 576 B         │         │
├─ sql_lite_proxy_db_100      32.58 µs      │ 302 µs        │ 37.37 µs      │ 42.32 µs      │ 100     │ 100
│                             alloc:        │               │               │               │         │
│                               97          │ 97            │ 97            │ 97            │         │
│                               4.043 KB    │ 4.043 KB      │ 4.043 KB      │ 4.043 KB      │         │
│                             dealloc:      │               │               │               │         │
│                               97          │ 97            │ 97            │ 97            │         │
│                               4.043 KB    │ 4.043 KB      │ 4.043 KB      │ 4.043 KB      │         │
├─ sql_lite_proxy_db_100_000  8.219 ms      │ 9.424 ms      │ 8.298 ms      │ 8.34 ms       │ 100     │ 100
│                             alloc:        │               │               │               │         │
│                               119         │ 119           │ 119           │ 119           │         │
│                               5.023 KB    │ 5.015 KB      │ 5.022 KB      │ 5.024 KB      │         │
│                             dealloc:      │               │               │               │         │
│                               119         │ 119           │ 119           │ 119           │         │
│                               5.023 KB    │ 5.015 KB      │ 5.022 KB      │ 5.024 KB      │         │
├─ sql_lite_proxy_db_12_500   1.061 ms      │ 1.727 ms      │ 1.073 ms      │ 1.094 ms      │ 100     │ 100
│                             alloc:        │               │               │               │         │
│                               119         │ 119           │ 119           │ 119           │         │
│                               5.025 KB    │ 5.027 KB      │ 5.023 KB      │ 5.023 KB      │         │
│                             dealloc:      │               │               │               │         │
│                               119         │ 119           │ 119           │ 119           │         │
│                               5.025 KB    │ 5.027 KB      │ 5.023 KB      │ 5.023 KB      │         │
├─ venn_proxy_db_100          894.9 ns      │ 2.738 µs      │ 915.9 ns      │ 946.2 ns      │ 100     │ 400
│                             alloc:        │               │               │               │         │
│                               6           │ 6             │ 6             │ 6             │         │
│                               46 B        │ 46 B          │ 46 B          │ 46 B          │         │
│                             dealloc:      │               │               │               │         │
│                               6           │ 6             │ 6             │ 6             │         │
│                               46 B        │ 46 B          │ 46 B          │ 46 B          │         │
├─ venn_proxy_db_100_000      124.2 µs      │ 156.3 µs      │ 129.2 µs      │ 130.4 µs      │ 100     │ 100
│                             alloc:        │               │               │               │         │
│                               6           │ 6             │ 6             │ 6             │         │
│                               25.02 KB    │ 25.02 KB      │ 25.02 KB      │ 25.02 KB      │         │
│                             dealloc:      │               │               │               │         │
│                               6           │ 6             │ 6             │ 6             │         │
│                               25.02 KB    │ 25.02 KB      │ 25.02 KB      │ 25.02 KB      │         │
╰─ venn_proxy_db_12_500       16.04 µs      │ 25.54 µs      │ 16.97 µs      │ 17.72 µs      │ 100     │ 100
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
