"""Build tidy parquet tables and compute Krugman's two productivity measures.

Inputs : data/raw/pwt110.xlsx
Outputs: data/processed/pwt_tidy.parquet      (per-entity, per-year panel)
         data/processed/method1_growth.parquet (within-country growth, indexed)
         data/processed/method2_levels.parquet  (cross-country PPP levels)
         data/processed/cagr_table.parquet      (growth-rate summary)

Units
-----
emp    : persons engaged, millions
avh    : average annual hours per person engaged
rgdpna : real GDP at constant 2017 national prices, millions 2017US$
cgdpo  : output-side real GDP at current PPPs, millions 2017US$

  total hours (millions)        = emp * avh
  output per hour ($/hour)      = GDP (mil$) / total hours (mil hours)

so `rgdpna/(emp*avh)` and `cgdpo/(emp*avh)` are both in US$ per hour.
"""
from __future__ import annotations

import pandas as pd

from config import (
    CAGR_PERIODS,
    EUROZONE_CORE,
    EUROZONE_LABEL,
    INDEX_BASE_YEAR,
    PROC_DIR,
    RAW_XLSX,
    REFERENCE,
    TARGETS,
)

PWT_COLS = ["countrycode", "country", "year", "emp", "avh", "rgdpna", "cgdpo"]


def _load_raw() -> pd.DataFrame:
    df = pd.read_excel(RAW_XLSX, sheet_name="Data")[PWT_COLS]
    return df


def _entity_panel(df: pd.DataFrame) -> pd.DataFrame:
    """Long panel of (entity, year) with hours + both GDP concepts.

    Single countries pass through; the euro-area aggregate is the sum of GDP
    and the sum of hours over its fixed member set (so per-hour ratios are
    correctly hours-weighted, not naive averages of national ratios).
    """
    rows = []

    # --- single-country targets -------------------------------------------
    for code, label in TARGETS.items():
        sub = df[df.countrycode == code].copy()
        sub = sub.assign(entity=label)
        rows.append(sub[["entity", "year", "emp", "avh", "rgdpna", "cgdpo"]])

    # --- euro-area aggregate (fixed-composition) --------------------------
    ea = df[df.countrycode.isin(EUROZONE_CORE)].copy()
    ea = ea.dropna(subset=["emp", "avh", "rgdpna", "cgdpo"])
    # keep only years in which all members are present -> consistent basket
    full_years = (
        ea.groupby("year").countrycode.nunique()
        .pipe(lambda s: s[s == len(EUROZONE_CORE)].index)
    )
    ea = ea[ea.year.isin(full_years)].copy()
    ea["hours"] = ea.emp * ea.avh
    agg = (
        ea.groupby("year")
        .agg(rgdpna=("rgdpna", "sum"), cgdpo=("cgdpo", "sum"), hours=("hours", "sum"))
        .reset_index()
    )
    # store emp/avh as aggregate hours via emp=hours, avh=1 so emp*avh==hours
    agg = agg.assign(entity=EUROZONE_LABEL, emp=agg.hours, avh=1.0)
    rows.append(agg[["entity", "year", "emp", "avh", "rgdpna", "cgdpo"]])

    panel = pd.concat(rows, ignore_index=True)
    panel["hours"] = panel.emp * panel.avh
    # the two output-per-hour series
    panel["prod_na"] = panel.rgdpna / panel.hours      # Method 1 input (constant national prices)
    panel["prod_ppp"] = panel.cgdpo / panel.hours       # Method 2 (current PPPs)
    return panel.sort_values(["entity", "year"]).reset_index(drop=True)


def _method1_growth(panel: pd.DataFrame) -> pd.DataFrame:
    """Within-country growth: index `prod_na` to 100 at INDEX_BASE_YEAR.

    Each entity is normalised to its own base-year level, so the chart shows
    *growth trajectories*, not comparable cross-country levels -- exactly the
    'standard' comparison Krugman says answers the wrong welfare question.
    """
    out = []
    for ent, g in panel.groupby("entity"):
        g = g.dropna(subset=["prod_na"]).sort_values("year")
        if g.empty:
            continue
        base = g[g.year == INDEX_BASE_YEAR]
        base_val = base.prod_na.iloc[0] if len(base) else g.prod_na.iloc[0]
        base_year = base.year.iloc[0] if len(base) else g.year.iloc[0]
        gg = g.assign(
            prod_na_index=100.0 * g.prod_na / base_val,
            index_base_year=base_year,
        )
        out.append(gg[["entity", "year", "prod_na", "prod_na_index", "index_base_year"]])
    return pd.concat(out, ignore_index=True)


def _method2_levels(panel: pd.DataFrame) -> pd.DataFrame:
    """Cross-country levels: `prod_ppp` and its ratio to the US, year by year."""
    ref = (
        panel[panel.entity == REFERENCE][["year", "prod_ppp"]]
        .rename(columns={"prod_ppp": "ref_prod_ppp"})
    )
    m = panel.merge(ref, on="year", how="left")
    m["pct_of_us"] = 100.0 * m.prod_ppp / m.ref_prod_ppp
    return m.dropna(subset=["prod_ppp"])[
        ["entity", "year", "prod_ppp", "pct_of_us"]
    ].reset_index(drop=True)


def _cagr_table(m1: pd.DataFrame) -> pd.DataFrame:
    """Compound annual growth rate of `prod_na` over CAGR_PERIODS (Method 1)."""
    recs = []
    for ent, g in m1.groupby("entity"):
        g = g.set_index("year").sort_index()
        for y0, y1 in CAGR_PERIODS:
            if y0 in g.index and y1 in g.index:
                v0, v1 = g.loc[y0, "prod_na"], g.loc[y1, "prod_na"]
                cagr = (v1 / v0) ** (1.0 / (y1 - y0)) - 1.0
                recs.append({"entity": ent, "period": f"{y0}-{y1}",
                             "cagr_pct": round(100 * cagr, 2)})
            else:
                recs.append({"entity": ent, "period": f"{y0}-{y1}", "cagr_pct": None})
    return pd.DataFrame(recs)


def build() -> dict[str, pd.DataFrame]:
    PROC_DIR.mkdir(parents=True, exist_ok=True)
    panel = _entity_panel(_load_raw())
    m1 = _method1_growth(panel)
    m2 = _method2_levels(panel)
    cagr = _cagr_table(m1)

    panel.to_parquet(PROC_DIR / "pwt_tidy.parquet", index=False)
    m1.to_parquet(PROC_DIR / "method1_growth.parquet", index=False)
    m2.to_parquet(PROC_DIR / "method2_levels.parquet", index=False)
    cagr.to_parquet(PROC_DIR / "cagr_table.parquet", index=False)
    print(f"[transform] wrote 4 parquet tables to {PROC_DIR}")
    return {"panel": panel, "m1": m1, "m2": m2, "cagr": cagr}


if __name__ == "__main__":
    build()
