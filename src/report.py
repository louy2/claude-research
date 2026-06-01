"""Render the self-contained HTML report with interactive Plotly charts."""
from __future__ import annotations

from datetime import date

import pandas as pd
import plotly.graph_objects as go
import plotly.io as pio

from config import (
    DISPLAY_ORDER,
    EUROZONE_CORE,
    INDEX_BASE_YEAR,
    PROC_DIR,
    PWT_LANDING,
    PWT_VERSION,
    REFERENCE,
    REPORT_DIR,
)

# colour-blind-friendly palette, stable per entity
PALETTE = {
    "United States": "#1f77b4",
    "Euro area (11-member core)": "#9467bd",
    "Netherlands": "#17becf",
    "Japan": "#d62728",
    "Taiwan": "#ff7f0e",
    "China": "#2ca02c",
    "Brazil": "#8c564b",
    "Zambia": "#7f7f7f",
}


def _order(entities) -> list[str]:
    present = set(entities)
    return [e for e in DISPLAY_ORDER if e in present]


def _fig_to_div(fig: go.Figure) -> str:
    return pio.to_html(fig, include_plotlyjs=False, full_html=False,
                       config={"displaylogo": False})


def _line(df, x, y, title, ytitle, log=False, hovfmt=".0f"):
    fig = go.Figure()
    for ent in _order(df.entity.unique()):
        g = df[df.entity == ent].sort_values(x)
        fig.add_trace(go.Scatter(
            x=g[x], y=g[y], name=ent, mode="lines",
            line=dict(color=PALETTE.get(ent), width=2.4),
            hovertemplate=f"%{{x}} · {ent} · %{{y:{hovfmt}}}<extra></extra>",
        ))
    fig.update_layout(
        title=title, template="plotly_white", height=520,
        hovermode="x unified", legend=dict(orientation="h", y=-0.18),
        margin=dict(l=60, r=30, t=60, b=80),
    )
    fig.update_xaxes(title="Year")
    fig.update_yaxes(title=ytitle, type="log" if log else "linear")
    return fig


def _cagr_html(cagr: pd.DataFrame) -> str:
    piv = cagr.pivot(index="entity", columns="period", values="cagr_pct")
    piv = piv.reindex(_order(piv.index))
    cols = list(piv.columns)
    head = "".join(f"<th>{c}</th>" for c in cols)
    body = ""
    for ent, row in piv.iterrows():
        cells = "".join(
            f"<td>{'' if pd.isna(row[c]) else f'{row[c]:.2f}%'}</td>" for c in cols
        )
        body += f"<tr><th class='rowh'>{ent}</th>{cells}</tr>"
    return (f"<table class='tbl'><thead><tr><th>Entity</th>{head}</tr></thead>"
            f"<tbody>{body}</tbody></table>")


def build() -> str:
    REPORT_DIR.mkdir(parents=True, exist_ok=True)
    m1 = pd.read_parquet(PROC_DIR / "method1_growth.parquet")
    m2 = pd.read_parquet(PROC_DIR / "method2_levels.parquet")
    cagr = pd.read_parquet(PROC_DIR / "cagr_table.parquet")

    fig1 = _line(m1, "year", "prod_na_index",
                 f"Method 1 — Real output per hour, indexed to {INDEX_BASE_YEAR}=100 "
                 "(constant national prices)",
                 f"Index ({INDEX_BASE_YEAR}=100, log scale)", log=True, hovfmt=".0f")
    fig2 = _line(m2, "year", "prod_ppp",
                 "Method 2 — Output per hour, level (current PPPs)",
                 "US$ per hour (current PPPs, log scale)", log=True, hovfmt=".1f")
    fig3 = _line(m2, "year", "pct_of_us",
                 "Method 2 — Output per hour relative to the United States",
                 "% of US output per hour", log=False, hovfmt=".1f")

    latest = int(m2.year.max())
    div1, div2, div3 = _fig_to_div(fig1), _fig_to_div(fig2), _fig_to_div(fig3)
    cagr_tbl = _cagr_html(cagr)
    ez_members = ", ".join(EUROZONE_CORE)

    html = f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Two ways to compare productivity — a Krugman exercise</title>
<script src="https://cdn.plot.ly/plotly-2.32.0.min.js" charset="utf-8"></script>
<style>
  :root {{ --ink:#1a1a1a; --mut:#5b5b5b; --line:#e4e4e7; --accent:#1f77b4; }}
  body {{ font:16px/1.6 -apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,Helvetica,Arial,sans-serif;
          color:var(--ink); max-width:960px; margin:0 auto; padding:32px 20px 80px; }}
  h1 {{ font-size:30px; line-height:1.25; margin:0 0 6px; }}
  h2 {{ font-size:22px; margin:44px 0 10px; padding-top:14px; border-top:1px solid var(--line); }}
  h3 {{ font-size:17px; margin:26px 0 6px; }}
  .sub {{ color:var(--mut); margin:0 0 24px; }}
  blockquote {{ margin:18px 0; padding:12px 20px; border-left:4px solid var(--accent);
                background:#f6f8fb; color:#33373b; font-style:italic; }}
  blockquote cite {{ display:block; font-style:normal; color:var(--mut); margin-top:8px; font-size:14px; }}
  .chart {{ margin:14px 0 8px; }}
  .note {{ font-size:14px; color:var(--mut); margin:4px 0 0; }}
  table.tbl {{ border-collapse:collapse; width:100%; font-size:14.5px; margin:12px 0; }}
  table.tbl th, table.tbl td {{ border:1px solid var(--line); padding:7px 10px; text-align:right; }}
  table.tbl thead th {{ background:#f3f4f6; text-align:right; }}
  table.tbl th.rowh {{ text-align:left; background:#fafafa; }}
  table.tbl td {{ font-variant-numeric:tabular-nums; }}
  .takeaway {{ background:#fffdf2; border:1px solid #f0e6b8; border-radius:8px; padding:14px 18px; margin:16px 0; }}
  footer {{ margin-top:50px; padding-top:16px; border-top:1px solid var(--line); color:var(--mut); font-size:13px; }}
  a {{ color:var(--accent); }}
  code {{ background:#f3f4f6; padding:1px 5px; border-radius:4px; font-size:13.5px; }}
</style></head><body>

<h1>Two ways to compare productivity across countries</h1>
<p class="sub">An exercise on Paul Krugman's distinction, built on Penn World Table {PWT_VERSION}
(1950–{latest}). Generated {date.today().isoformat()}.</p>

<blockquote>
  One method is to compare the growth in inflation-adjusted GDP per hour within countries.
  This is a standard way to make cross-country comparisons, but one that answers the wrong
  question. The other method is to compare the year-by-year value of output per worker-hour,
  adjusted for differences in national price levels to control for exchange rate instability,
  but not for changing price levels over time. This measure is, I would argue, much more
  meaningful for comparing trends in economic welfare across countries.
  <cite>— Paul Krugman</cite>
</blockquote>

<p>The two methods map directly onto two distinct GDP concepts in the Penn World Table,
which is built precisely to keep them apart:</p>
<ul>
  <li><strong>Method 1 (within-country growth):</strong> <code>rgdpna</code> — real GDP at
      constant <em>national</em> prices. PWT's recommended series for tracking growth
      <em>over time within one country</em>. We divide by total hours
      (<code>emp × avh</code>) and index each entity to its own {INDEX_BASE_YEAR} level.</li>
  <li><strong>Method 2 (cross-country welfare levels):</strong> <code>cgdpo</code> —
      output-side real GDP at <em>current</em> PPPs. PPP adjustment removes exchange-rate
      noise and national-price-level differences; "current" means each year is valued at
      that year's prices, <em>not</em> deflated over time. PWT's recommended series for
      comparing the <em>level</em> of output across countries in a given year.</li>
</ul>

<h2>Method 1 — Growth in real output per hour, within countries</h2>
<p>Each line is normalised to <strong>100 in {INDEX_BASE_YEAR}</strong>, so the chart shows
how fast each economy's own labour productivity grew — the "standard" comparison.</p>
<div class="chart">{div1}</div>
<p class="note">Source: PWT {PWT_VERSION}, <code>rgdpna/(emp·avh)</code>. Log scale: a constant
slope is a constant growth rate. Zambia's hours series begins in 2005, so its line is short.</p>

<h3>Compound annual growth rate of output per hour (%)</h3>
{cagr_tbl}

<div class="takeaway"><strong>Why Krugman calls this "the wrong question."</strong>
Read alone, this chart says Taiwan, China and Japan are runaway success stories and that
the US "grows slowly." But indexing every country to 100 erases <em>where each one started</em>.
A country can post the fastest growth rate on earth and still produce far less per hour than
the US in <em>every single year</em>. Growth rates describe the journey, not the destination —
so they are a poor guide to <em>relative economic welfare</em>.</div>

<h2>Method 2 — Output per hour, levels at current PPPs</h2>
<p>Now the same labour-productivity concept, but in comparable <strong>US$ per hour</strong>
for each year (current PPPs). No indexing: the gaps between the lines are real.</p>
<div class="chart">{div2}</div>
<p class="note">Source: PWT {PWT_VERSION}, <code>cgdpo/(emp·avh)</code>, current PPPs, log scale.</p>

<h3>Relative to the United States</h3>
<p>The clearest welfare-convergence picture: each economy's output per hour as a percentage
of the US level, year by year.</p>
<div class="chart">{div3}</div>
<p class="note">100% = US output per hour in the same year.</p>

<div class="takeaway"><strong>What Method 2 reveals.</strong> The Netherlands and the euro-area
core sit close to — at times above — the US frontier. Japan and Taiwan <em>converged</em> from
far below toward the rich-country range. China and Brazil grew fast yet remain a large multiple
below the frontier in level terms, and China's gap, while shrinking, is still wide. Zambia
barely registers. These are statements about <em>welfare levels</em> that Method 1's
growth-rate chart simply cannot make.</div>

<h2>Notes &amp; caveats</h2>
<ul>
  <li><strong>Euro area</strong> is a fixed 11-member basket ({ez_members}) with continuous
      1950–{latest} coverage — the aggregate is Σ GDP ÷ Σ hours, i.e. correctly hours-weighted.</li>
  <li><strong>Netherlands</strong> is used as the European frontier proxy alongside the
      euro-area aggregate, per the request.</li>
  <li><strong>Zambia</strong> has average-hours data only from 2005 onward in PWT; earlier
      output-per-hour values cannot be computed.</li>
  <li><strong>Output per hour</strong> uses persons engaged × average annual hours; it is a
      labour-productivity measure, not GDP per capita.</li>
  <li>Method 2 uses <em>current</em> PPPs by design (Krugman's "not adjusted for changing
      price levels over time"). PWT also offers chained-PPP series (<code>rgdpo</code>) for
      joint cross-country-and-over-time comparison; that is a third concept, not used here.</li>
</ul>

<footer>
  Data: Penn World Table {PWT_VERSION} (Feenstra, Inklaar &amp; Timmer), CC&nbsp;BY&nbsp;4.0 —
  <a href="{PWT_LANDING}">{PWT_LANDING}</a>. Fully reproducible:
  <code>python src/run.py</code>. Charts: Plotly.
</footer>
</body></html>"""

    out = REPORT_DIR / "krugman_comparison.html"
    out.write_text(html, encoding="utf-8")
    print(f"[report] wrote {out}")
    return str(out)


if __name__ == "__main__":
    build()
