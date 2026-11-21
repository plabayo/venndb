"""Plot dense_query Divan logs. Repeated runs can be concatenated per engine.

Usage: python3 scripts/bench_dense_graph.py ORIGINAL BLOCKWISE FINAL OUTPUT.svg
Requires ReportLab.
"""

from math import ceil
import re
import sys
from collections import defaultdict
from statistics import mean

from reportlab.graphics import renderSVG
from reportlab.graphics.charts.barcharts import HorizontalBarChart
from reportlab.graphics.shapes import Drawing, Rect, String
from reportlab.lib.colors import HexColor


def read_medians(path):
    samples = defaultdict(list)
    group = filters = None
    with open(path) as log:
        for line in log:
            node = re.match(r"^[│ ]*[├╰]─ (\w+)\s+(.*)$", line)
            if not node:
                continue
            label, body = node.groups()
            if not label.isdigit():
                group = label
            elif body.startswith("│"):
                filters = int(label)
            elif group in ("independent", "clustered"):
                value, unit = body.split("│")[2].split()
                samples[group, filters, int(label)].append(
                    float(value) * {"ns": 1, "µs": 1000, "ms": 1000000}[unit]
                )
    return {key: mean(values) for key, values in samples.items()}


def plot(paths, output):
    engines = [read_medians(path) for path in paths]
    colors = [HexColor(code) for code in ("#64748b", "#c27a33", "#087d76")]
    fig = Drawing(740, 336)
    fig.add(Rect(0, 0, 740, 336, fillColor=HexColor("#ffffff"), strokeColor=None))
    fig.add(String(18, 310, "Dense query latency", fontName="Helvetica-Bold", fontSize=18))
    for x, title, color in zip((18, 160, 310), ("Original engine", "Blockwise", "Blockwise + small path"), colors):
        fig.add(Rect(x, 283, 11, 11, fillColor=color, strokeColor=None))
        fig.add(String(x + 17, 284, title, fontSize=11))
    for left, rows, unit, scale in [(0, 1000, "ns", 1), (370, 8388611, "µs", 1000)]:
        fig.add(String(left + 18, 249, f"{rows:,} rows · {unit} per query", fontName="Helvetica-Bold", fontSize=13))
        cases = [("independent", 2), ("independent", 4), ("independent", 8)]
        if rows > 1000:
            cases.append(("clustered", 8))
        chart = HorizontalBarChart()
        chart.x, chart.y, chart.width, chart.height = left + 103, 48, 230, 182
        chart.data = [[engine[g, n, rows] / scale for g, n in reversed(cases)] for engine in reversed(engines)]
        chart.categoryAxis.categoryNames = [f"{n} {g}" for g, n in reversed(cases)]
        chart.categoryAxis.labels.fontSize = 10
        chart.categoryAxis.labels.dx = -5
        chart.valueAxis.valueMin = 0
        chart.valueAxis.valueStep = 25 if rows == 1000 else 50
        peak = max(max(series) for series in chart.data)
        chart.valueAxis.valueMax = ceil(peak / chart.valueAxis.valueStep) * chart.valueAxis.valueStep
        chart.valueAxis.visibleGrid = True
        chart.valueAxis.gridStrokeColor = HexColor("#e2e8f0")
        chart.barSpacing = 1
        for i, color in enumerate(reversed(colors)):
            chart.bars[i].fillColor = color
            chart.bars[i].strokeColor = None
        fig.add(chart)
    fig.add(String(18, 12, "Generated query builder through first row access · mean of run medians · Apple M4 Max", fontSize=10))
    renderSVG.drawToFile(fig, output)


if __name__ == "__main__":
    plot(sys.argv[1:4], sys.argv[4])
