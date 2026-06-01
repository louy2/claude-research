# Two ways to compare productivity across countries

A small, reproducible exercise on a distinction Paul Krugman draws between two
ways of comparing economic performance across countries:

> One method is to compare the growth in inflation-adjusted GDP per hour within
> countries. This is a standard way to make cross-country comparisons, but one
> that answers the wrong question. The other method is to compare the
> year-by-year value of output per worker-hour, adjusted for differences in
> national price levels to control for exchange rate instability, but not for
> changing price levels over time. This measure is, I would argue, much more
> meaningful for comparing trends in economic welfare across countries.
> — Paul Krugman

It computes both comparisons for **United States, Euro area, Netherlands,
Japan, Taiwan, China, Brazil, and Zambia** and renders an interactive HTML
report.

## The two methods → two Penn World Table concepts

The Penn World Table (PWT) is built precisely to keep these apart:

| | Krugman's method | PWT series | What it measures |
|---|---|---|---|
| **Method 1** | growth in inflation-adjusted GDP per hour *within* countries | `rgdpna` — real GDP at constant **national** prices | growth over time within one country (indexed to 1960 = 100) |
| **Method 2** | year-by-year output per worker-hour, PPP-adjusted, *not* deflated over time | `cgdpo` — output-side real GDP at **current PPPs** | the *level* of output across countries in each year |

Both are divided by total hours worked (`emp` × `avh`) to get **output per
hour**.

### The punchline
- **Method 1** indexes every country to its own 1960 level, so China and Taiwan
  look like the runaway winners and the US looks slow. It describes the
  *journey*, not the destination — Krugman's "wrong question" for welfare.
- **Method 2** keeps the levels, so the gaps are real: the Netherlands and euro
  core sit at the US frontier, Japan and Taiwan converged toward it, and China,
  Brazil and Zambia — fast growth notwithstanding — remain well below it.

## Data

Penn World Table **11.0** (Feenstra, Inklaar & Timmer), 1950–2023, 185
countries, CC BY 4.0 — <https://www.rug.nl/ggdc/productivity/pwt/>.
Downloaded automatically from DataverseNL.

- **Euro area** = fixed 11-member core (AUT, BEL, DEU, ESP, FIN, FRA, GRC, IRL,
  ITA, NLD, PRT) with continuous 1950–2023 coverage; aggregate = Σ GDP ÷ Σ hours.
- **Netherlands** is the European frontier proxy, shown alongside the aggregate.
- **Zambia** has average-hours data only from 2005, so its output-per-hour
  series is short.

## Run it

```bash
pip install -r requirements.txt
python src/run.py            # add --force to re-download PWT
```

Outputs:
- `report/krugman_comparison.html` — interactive report (open in a browser)
- `report/preview_*.png` — static chart previews
- `data/processed/*.parquet` — tidy panel and both computed measures

## Layout

```
src/config.py      entities, euro-area composition, paths, data-source URL
src/fetch.py       download PWT 11.0 (resolves Dataverse signed URL, retries)
src/transform.py   tidy panel + Method 1 / Method 2 tables → parquet
src/report.py      Plotly HTML report
src/run.py         fetch → transform → report
```

## Output files

| File | Contents |
|---|---|
| `data/processed/pwt_tidy.parquet` | per-entity, per-year panel (hours, both GDP concepts, both per-hour series) |
| `data/processed/method1_growth.parquet` | `rgdpna`-based output per hour, indexed to 1960 |
| `data/processed/method2_levels.parquet` | `cgdpo`-based output per hour, level + % of US |
| `data/processed/cagr_table.parquet` | compound annual growth rates by sub-period |
