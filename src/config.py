"""Shared configuration for the Krugman productivity-comparison pipeline.

The analysis reproduces the two cross-country comparison methods that Paul
Krugman contrasts:

  Method 1 -- "growth in inflation-adjusted GDP per hour *within* countries".
      The standard productivity-growth comparison. Uses PWT `rgdpna`
      (real GDP at constant *national* prices), which is the series PWT
      designs for comparing growth over time within a single country.

  Method 2 -- "the year-by-year value of output per worker-hour, adjusted for
      differences in national price levels ... but not for changing price
      levels over time". Uses PWT `cgdpo` (output-side real GDP at *current*
      PPPs), the series PWT designs for comparing the *level* of output
      across countries in a given year.

Both are divided by total hours worked = `emp` (persons engaged, millions)
x `avh` (average annual hours per person engaged), giving output per hour.
"""
from pathlib import Path

# --- paths -----------------------------------------------------------------
ROOT = Path(__file__).resolve().parent.parent
RAW_DIR = ROOT / "data" / "raw"
PROC_DIR = ROOT / "data" / "processed"
REPORT_DIR = ROOT / "report"
RAW_XLSX = RAW_DIR / "pwt110.xlsx"

# --- data source -----------------------------------------------------------
# Penn World Table version 11.0 (Feenstra, Inklaar & Timmer), 1950-2023,
# 185 countries. Hosted on DataverseNL; the datafile endpoint 303-redirects
# to a time-limited signed object-store URL, which we follow per-request.
PWT_VERSION = "11.0"
PWT_DATAFILE_URL = "https://dataverse.nl/api/access/datafile/554105"
PWT_LANDING = "https://www.rug.nl/ggdc/productivity/pwt/"

# --- entities under study --------------------------------------------------
# Single-country targets (ISO3 code -> display label).
TARGETS = {
    "USA": "United States",
    "NLD": "Netherlands",      # Europe proxy (high-productivity frontier EU economy)
    "CHN": "China",
    "JPN": "Japan",
    "TWN": "Taiwan",
    "BRA": "Brazil",
    "ZMB": "Zambia",
}

# Euro-area aggregate: fixed composition of 11 members with continuous
# 1950-2023 PWT coverage (~90%+ of euro-area GDP). A fixed basket keeps the
# per-hour level/growth series internally consistent across all years.
EUROZONE_CORE = [
    "AUT", "BEL", "DEU", "ESP", "FIN",
    "FRA", "GRC", "IRL", "ITA", "NLD", "PRT",
]
EUROZONE_LABEL = "Euro area (11-member core)"

# Order entities for plotting / tables.
DISPLAY_ORDER = [
    "United States",
    EUROZONE_LABEL,
    "Netherlands",
    "Japan",
    "Taiwan",
    "China",
    "Brazil",
    "Zambia",
]

# Reference economy for relative-level comparisons (Method 2).
REFERENCE = "United States"

# Base year for indexing the within-country growth series (Method 1).
INDEX_BASE_YEAR = 1960

# Sub-periods for compound-annual-growth-rate tables (Method 1).
CAGR_PERIODS = [(1960, 1980), (1980, 2000), (2000, 2023), (1960, 2023)]
