# Conventions, Coordination, and Effective Demand

## Weaving Schelling and Heath into Keynes' *General Theory*, with Calibrations from the United States, Canada, China, and Japan

*A research synthesis*

---

## 0. Abstract

Keynes' *General Theory of Employment, Interest and Money* (1936) is usually read as a book about aggregate demand. This paper re-reads it as a book about **cooperation under uncertainty**: an economy is a vast, decentralized coordination game, and involuntary unemployment is what a failed equilibrium of that game looks like from the outside. Two later bodies of work supply the micro-machinery Keynes lacked. **Thomas Schelling** shows *how* dispersed agents coordinate without communication — through focal points, conventions, credible commitment, and tipping dynamics — and therefore how they can coordinate on bad outcomes. **Joseph Heath** shows *why* cooperative equilibria hold at all — because human practical reason is norm-governed, not merely instrumental, and because institutions (firms, welfare states, markets themselves) are devices for capturing the benefits of cooperation that uncoordinated exchange leaves on the table. Read through Schelling and Heath, the *General Theory*'s central objects — the state of long-term expectation, the convention of the stock market, liquidity preference, animal spirits, the socialisation of investment — become tractable game-theoretic and normative claims. The paper then assembles the wider literature standing at this intersection, and calibrates the resulting model against four societies whose coordination regimes differ sharply: the United States, Canada, China, and Japan.

---

## 1. The *General Theory* as a theory of coordination

### 1.1 What Keynes actually claimed

Strip the *General Theory* to its load-bearing propositions and each one is a claim about coordination among strangers:

1. **Effective demand (Ch. 3).** Output and employment are set by what entrepreneurs *expect* to sell, not by what households would like to buy at full employment. Production takes time; entrepreneurs must commit resources today against a guess about everyone else's spending tomorrow. Say's Law fails because there is no mechanism — no auctioneer, no shared plan — that reconciles these guesses in advance.

2. **The two-tier theory of expectation (Chs. 5 and 12).** Short-term expectations govern the utilization of existing capacity and are disciplined by quick feedback. Long-term expectations govern investment and are *not* disciplined by feedback, because "our existing knowledge does not provide a sufficient basis for a calculated mathematical expectation" (GT, Ch. 12). Where calculation gives out, something else must fill the gap.

3. **Convention (Ch. 12).** What fills the gap is a convention: "The essence of this convention … lies in assuming that the existing state of affairs will continue indefinitely, except in so far as we have specific reasons to expect a change." Keynes is explicit that the convention is not a belief anyone holds to be *true*; it is a practice everyone follows because everyone else follows it. Valuations established this way are stable so long as the convention holds — and "precarious" because nothing anchors the convention except itself.

4. **The beauty contest (Ch. 12).** Professional investment is likened to a newspaper competition in which entrants pick not the faces they find prettiest, nor even the faces average opinion finds prettiest, but "what average opinion expects the average opinion to be" — and some, Keynes notes, practice the fourth and fifth degrees. Asset prices are thus the fixed point of an iterated guessing game about other minds, not an estimate of any fundamental.

5. **Animal spirits (Ch. 12).** Because long-term expectation cannot rest on calculation, enterprise depends on "a spontaneous urge to action rather than inaction." Formal decision theory cannot even get the entrepreneur out of bed; if "animal spirits are dimmed and the spontaneous optimism falters, enterprise will fade and die."

6. **Liquidity preference (Chs. 13–17).** Money is the asset you hold when you decline to bet on the conventions governing all other assets. The rate of interest is "a highly conventional … phenomenon" (Ch. 15): it sits where opinion expects it to sit. Hoarding is the individually rational exit from a coordination game whose other players you no longer trust — and if enough players exit at once, the game collapses for everyone. Chapter 17's "essential properties" of money (negligible elasticities of production and substitution) explain why this exit option, unlike a demand for shoes or ships, employs no one.

7. **Unemployment equilibrium (Chs. 18–19).** The economy can come to rest at many levels of activity. Flexible wages do not rescue it, because wage cuts operate on expectations and distribution, not just on a single market's price. There are multiple resting points, and no invisible hand guaranteeing convergence to the good one.

8. **The socialisation of investment (Ch. 24).** If private conventions cannot hold long-term expectation steady, the state must — "a somewhat comprehensive socialisation of investment will prove the only means of securing an approximation to full employment," pursued through "all manner of compromises" between public authority and private initiative.

### 1.2 The missing microfoundation

Keynes asserted all of this in 1936 with no formal apparatus for strategic interaction — von Neumann and Morgenstern were eight years away, Nash sixteen. He could describe conventional expectation with rare literary skill, but he could not say *what kind of object* a convention is, *why* it is stable, *why* it tends to collapse discontinuously rather than eroding smoothly, or *what* policy is doing when it "manages confidence." The standard postwar formalizations (IS-LM, the neoclassical synthesis) kept his accounting identities and set these questions aside.

Game theory after Schelling, and the philosophy of norms after Heath, offer answers. That is the weave attempted here: Schelling supplies the **positive mechanics** of conventions (Section 2), Heath supplies the **normative statics** — why rules bind and what cooperation is for (Section 3) — and Section 4 argues that much of the *General Theory* can be reconstructed by running Schelling's mechanics on Heath's foundations at the scale of a monetary economy.

---

## 2. Schelling: the mechanics of tacit coordination

*(Core texts: The Strategy of Conflict, 1960; Micromotives and Macrobehavior, 1978.)*

### 2.0 The buried connection: Schelling began as a Keynesian

Before anything else, a historical fact that makes this paper's weave a *reunion* rather than an arranged marriage. Schelling was trained as a Keynesian macroeconomist: his first publications were "Capital Growth and Equilibrium" (*AER*, 1947) and "Income Determination: A Graphic Solution" (*REStat*, 1948), and his first book was *National Income Behavior: An Introduction to Algebraic Analysis* (McGraw-Hill, 1951) — a treatise on Keynesian income determination and multiplier models. Benjamin Wilson's "Keynes Goes Nuclear" (*Modern Intellectual History*, 2021) documents the deeper continuity: Schelling's "The Reciprocal Fear of Surprise Attack" (1958) transplanted the crossed-curve stability analysis and interacting-expectations logic of Keynesian macrodynamics into deterrence theory — bargainers reach agreement "when their expectations jointly converge on an outcome," a self-reinforcing equilibrium framework carried over from macroeconomics. The convergent-expectations equilibrium is a formal thread running through the multiplier, the focal point, the bank run, and mutual deterrence alike. Reading Schelling back into the *General Theory* is therefore not an anachronism; it is completing a circuit that Schelling himself opened from the other end.

### 2.1 Focal points and the possibility of tacit coordination

Schelling's central discovery in *The Strategy of Conflict* is that coordination games with multiple equilibria — which classical game theory declared indeterminate — are routinely solved in practice by **focal points**: solutions salient by precedent, analogy, symmetry, or cultural common knowledge. Two strangers told to meet in New York with no ability to communicate converge on Grand Central at noon. The equilibrium selected is not the "best" one; it is the *conspicuous* one.

This is very close to the object Keynes needed. The Chapter 12 convention — project the present forward — has the structure of a focal point in the space of possible forecasting rules: it requires no information anyone lacks, it is symmetric, and each investor's adherence is justified by everyone else's. The beauty contest is a focal-point search made explicit: at the fixed point of "what average opinion expects average opinion to be" sits whatever valuation is *salient*, which is normally yesterday's price. Keynes' remark that a conventional valuation is established by "the mass psychology of a large number of ignorant individuals" reads, after Schelling, as a mechanism rather than a sneer: when nobody can calculate, salience becomes the default equilibrium-selection device. (A caution flagged in Section 6: Davis and the post-Keynesian literature argue that Lewis-style equilibrium accounts of convention presuppose a stability of common knowledge that Keynesian uncertainty denies — the fit is productive, not seamless.)

Three Schelling corollaries matter for macroeconomics:

- **Conventions are self-enforcing but not self-justifying.** Each participant's conformity is a best response to others' conformity; nothing about the convention need track fundamentals. Bad conventions (overvaluation, panic pricing, a deflationary anchor) can be as stable as good ones — nothing in the selection mechanism favors the good ones.
- **Convention change tends to be discontinuous.** Because the convention's main support is mutual expectation, contrary evidence often fails to erode it gradually; it holds, and then it *breaks*, when some event makes it common knowledge that common knowledge has been lost. This gives Keynes' "violent" revisions of valuation a mechanism.
- **Focal points can be supplied.** A third party loud enough to be heard by everyone can *create* salience — which is what a central bank's announced target, a peg, a deposit guarantee, or "whatever it takes" actually does. Confidence policy is focal-point engineering (developed in Section 4.4).

### 2.2 Commitment: the strategy of constraining yourself

Schelling's second great theme is that in interdependent decision, **the power to bind oneself is a strategic asset**: the side that visibly forecloses its own options moves the equilibrium. Burn the bridge behind you and your threat to stand becomes credible.

Much of macroeconomic policy is applied commitment theory. A central bank's problem is that its future self will be tempted to accommodate; institutional independence, mandates, and rules are Schelling-style bridge-burning that make the announced focal point credible. Conversely, Keynes' Chapter 24 case for the socialisation of investment is a commitment argument: private investors *cannot* commit to sustained investment (each is free to flee to liquidity at any moment, and knows the others are too), whereas a state, "calculating the marginal efficiency of capital-goods on long views and on the basis of the general social advantage," can. The state's comparative advantage is not information; it is the ability to be *un-nimble* — to be the player who cannot run, and around whom the others can therefore coordinate.

### 2.3 Micromotives, tipping, and the aggregation problem

*Micromotives and Macrobehavior* is Schelling's study of how individually innocuous choices aggregate into collectively unchosen outcomes — the checkerboard segregation model, critical-mass dynamics, the dying seminar whose attendance depends on attendance ("nearly everybody, if asked, alleges that he'd have continued attending pretty regularly if enough others had cared enough to attend regularly"). Its master concept is the **tipping point**: systems whose participants condition on each other's behavior have thresholds, and cross-threshold dynamics are self-accelerating. Two details of the book confirm how macroeconomic its imagination remained. Chapter 2, "The Inescapable Mathematics of Musical Chairs," is a meditation on closed-system accounting identities — every purchase is a sale; one child is always chairless however alert the children — the arithmetic constraints within which fallacy-of-composition results like the paradox of thrift arise. (The identity alone does not deliver the paradox; it takes the behavioral coupling of Chapter 3's models to do that. Schelling keeps the two rigorously separate, which is itself a lesson macroeconomic rhetoric often needs.) And Chapter 3's taxonomy of **self-fulfilling prophecies** takes its canonical cases straight from Keynes' decade: "when people believed that a bank was on the verge of insolvency they hurried to withdraw their deposits, provoking the insolvency they feared," and the expected shortage that causes the hoarding that causes the shortage. Schelling even adds two variants macro theory still underuses: the *self-displacing* prophecy (everyone aims slightly above the average, so the average escalates without limit — a fine model of a bubble's drift phase) and the *self-negating* prophecy (everyone expects the crowd and stays home).

This is the formal home of the *General Theory*'s most distinctive claims:

- **The paradox of thrift is a Schelling diagram.** Each household's saving decision conditions on expected income; each firm's hiring conditions on expected demand. Individually prudent retrenchment, aggregated, produces the collective outcome nobody chose — lower income and often lower total saving. Keynes' fallacy-of-composition arguments are micromotives/macrobehavior arguments.
- **The marginal propensity to consume is a coupling coefficient.** The multiplier measures how tightly one agent's spending is wired to another's income — that is, how fast disturbances propagate through the interaction structure. High coupling means both strong policy leverage and strong crash dynamics.
- **Animal spirits have critical mass.** Investment booms and collapses behave like Schelling's attendance model: above a threshold of participating optimists, optimism is self-confirming and recruits; below it, pessimism is. The "state of confidence" is then not only a mood spreading person-to-person (Shiller's epidemiology captures that channel); it is also, and more structurally, the region of a threshold model the economy currently occupies — which is why confidence can change faster than any contagion process would allow.

### 2.4 What Schelling cannot supply

Schelling's framework is deliberately austere about motivation: agents best-respond to expectations. But a pure best-response account of convention has a well-known instability — if conformity is only ever instrumentally rational, then every convention is hostage to every fluctuation in expectation, and the miracle is not that conventions break but that they ever hold. Keynes saw this ("a practical theory of the future … subject to sudden and violent changes"), and Schelling can describe the breaking beautifully. What neither explains is the observed *stickiness*: why do workers not accept money-wage cuts, why do firms maintain price and employment relationships through slumps, why does anyone honor obligations when panic would pay? For that we need an account of agents who follow rules *as rules*. That is Heath.

---

## 3. Heath: why cooperation holds, and what it is for

*(Core texts: Communicative Action and Rational Choice, 2001; The Efficient Society, 2001; "The Benefits of Cooperation," 2006; Following the Rules, 2008; Filthy Lucre, 2009 / Economics Without Illusions, 2010; "Three Normative Models of the Welfare State," 2011; Morality, Competition, and the Firm, 2014; The Machinery of Government, 2020; Cooperation and Social Justice, 2022.)*

### 3.1 Norm-following as the solution to the instability problem

Heath's *Following the Rules* argues that purely instrumental rationality is self-undermining in social settings: it cannot generate the assurance that cooperation requires (the folk-theorem multiplicity means "rational" agents don't know *which* equilibrium they are in), and it licenses defection wherever monitoring fails. Actual human practical reason, Heath argues, is **deontically constrained**: people treat social norms as directly reason-giving — they follow rules *because they are the rules*, not because each act of conformity passes a cost-benefit test. Norm-conformity is a disposition with its own motivational force, culturally transmitted and sanction-stabilized, and it is this non-instrumental residue that makes large-scale cooperation among strangers possible.

Grafted onto Keynes, this yields the missing account of conventional *stability*:

- Keynes' Chapter 12 convention holds not merely because each investor calculates that others will hold it (Schelling's mechanism, which is real but fragile) but because market participants inhabit a **normatively thick practice** — accounting standards, fiduciary duties, professional valuation customs, the entire etiquette of "sound finance" — that makes projecting the present forward the *proper* thing to do, not just the expedient thing.
- Keynes' famous observation that workers resist money-wage cuts while accepting real-wage erosion through prices (Ch. 2) — long treated as an embarrassment ("money illusion") — becomes, through Heath, intelligible as rational at the level of the practice: relative money wages are the *fairness convention* of the labor market, a norm that stabilizes cooperation between firms and workers. Firms decline to cut wages in slumps for the same reason. Wage stickiness is not friction; it is the normative infrastructure of ongoing cooperation doing its job. (Truman Bewley's interview evidence and Akerlof's fair-wage models fit naturally here.)
- Conversely, Heath's framework predicts *where* conventions will be fragile: in arenas deliberately stripped of normative thickness — anonymous, exit-friendly, purely instrumental arenas. Keynes named the arena: the more the stock exchange approximates a frictionless casino of strangers, "the risk of the predominance of speculation does … increase" (Ch. 12, with the famous comparison of Wall Street to a casino and his musing about making purchases "permanent and indissoluble, like marriage"). Keynes' proposed transfer taxes are, in Heath's terms, an attempt to re-thicken a practice that liquidity had thinned.

### 3.2 The benefits of cooperation: a taxonomy for macroeconomics

In "The Benefits of Cooperation" (*Philosophy & Public Affairs*, 2006) Heath catalogs the five distinct mechanisms by which cooperation makes everyone better off — **economies of scale, gains from trade, risk pooling, self-binding, and information transmission** — and insists that different institutions are specialized vehicles for different mechanisms (corporations for scale, markets and property for trade, insurance and the welfare state for risk pooling, constitutions and families for self-binding, science and media for information), each with its characteristic free-rider pathology (shirking, moral hazard, adverse selection, lying). His pointed diagnosis is that contract theory and welfare economics suffer a **"catallactic bias"**: a tacit privileging of gains-from-trade — the invisible-hand theorem — as *the* mechanism of cooperative benefit, with the idealized conditions of the welfare theorems in effect assuming the other four mechanisms away. In "Three Normative Models of the Welfare State" (2011) he completes the argument: of the redistributive, communitarian, and public-economic reconstructions of the welfare state, the *efficiency* one — the state as the institution whose unique asset, compulsory universal membership, lets it resolve collective-action problems and pool risks that markets cannot (moral hazard, adverse selection) — best explains what welfare states actually do. The welfare state is not charity or vote-buying; it is "a huge insurance company with an army" (Krugman's line, which Heath adopts as epigraph), i.e., the capture of efficiency gains that private insurance cannot reach.

Keynes can be read as a major casualty of the catallactic bias Heath diagnoses: the postwar synthesis rendered the *General Theory* as a story about one mechanism (gains from trade, obstructed by sticky prices), when much of the book concerns the other mechanisms failing — risk pooling (uninsurable uncertainty), self-binding (no private commitment to sustained investment), and information transmission (the beauty contest degrading price signals into mirrors). (The fourth mapping — demand complementarity as an analogue of economies of scale — is looser, and is flagged as such.)

This taxonomy reorganizes Keynes:

- **Uninsurable uncertainty is a missing risk-pooling market.** Keynes' fundamental uncertainty ("we simply do not know") means the private economy cannot pool the risk that matters most — the risk that the *aggregate* future disappoints. Precautionary saving and liquidity preference are each agent's makeshift self-insurance against a risk that is, in the aggregate, only worsened by everyone self-insuring at once (the paradox of thrift, again). The Keynesian state — automatic stabilizers, deposit insurance, unemployment insurance, lender of last resort — is Heath's risk-pooling logic applied to *macroeconomic* risk: the one insurer whose pool is the whole game.
- **Effective demand is a network of gains from trade that can fail to be realized.** Involuntary unemployment is Heath's "suboptimal equilibrium" at economy scale: mutually beneficial trades (idle workers, unmet wants) that no bilateral action can consummate, because each trade is profitable only if the others occur. This is the market-failure vocabulary applied to the macro whole — and it is why Heath's "market failures approach" to ethics has a natural macro extension: the duty of price/wage forbearance in a panic, or of governments to stabilize, is the duty not to exploit a coordination failure.
- **Self-binding is Chapter 24.** Heath's fourth benefit of cooperation — institutions as commitment devices — is Schelling's commitment theory in normative dress, and it is what "a somewhat comprehensive socialisation of investment" provides: a standing, rule-bound commitment that aggregate investment will not be permitted to collapse, which stabilizes private long-term expectation by being *non-optional*.

### 3.3 Efficiency, not warmth: the Heathian reading of Keynesian policy

Heath's signature polemical move — in *The Efficient Society* and *Filthy Lucre* — is to defend cooperative institutions on **efficiency** grounds, refusing both the right's equation of markets with efficiency and the left's suspicion of it. Canada's single-payer health care is defended not as compassion but as the cheaper, Pareto-superior arrangement; likewise deposit insurance, public pensions, and financial regulation.

This is also a natural reading of Keynes, who was no socialist: the *General Theory* ends by insisting that its purpose is to *save* the market order's "efficiency and freedom" from its one great remediable defect, the failure to coordinate investment at full employment. Keynes and Heath share the same architecture: markets are magnificent cooperative technology for allocating; they systematically fail at *assuring* — at holding expectations steady enough for the allocating to proceed — and the state's macro role is to supply the assurance, not to replace the allocation. Both are, in this sense, theorists of the **mixed economy as the efficient solution to the assurance problem**, against both laissez-faire and planning.

---

## 4. The weave: a Schelling–Heath–Keynes model of the macroeconomy

### 4.1 The economy as a monetary stag hunt

Assemble the pieces. Model the economy as a many-player **assurance game (stag hunt)** played repeatedly under fundamental uncertainty:

- Each agent (firm, household, investor) chooses between **Commit** — invest, hire, spend, hold illiquid claims on the future — and **Withdraw** — hoard liquidity, delay, self-insure.
- Payoffs are interdependent through demand: Commit pays well iff enough others Commit (the stag); Withdraw pays a modest certain return regardless (the hare).
- **Money is what makes the hare available.** In a barter economy, "withdrawing" would itself be a demand for something producible. Keynes' Chapter 17 point — money is the asset with negligible elasticity of production, which private effort cannot create more of when demanded — is, on this reading, what turns the macroeconomy from a game with one equilibrium into a stag hunt with (at least) two: a high-commitment, high-employment equilibrium and a low-commitment, unemployment equilibrium. *In this model, involuntary unemployment is the hare equilibrium of a monetary economy.*
- **Fundamental uncertainty strips away the standard selection arguments.** Even with calculable risk, stag hunts resist confident selection — the risk-dominance literature shows that "rational" players may well pick the hare. Keynesian uncertainty removes even the materials those refinements work with: equilibrium selection must run on Schelling salience and Heath norms instead, and the economy sits wherever **convention** currently points. (This makes the model's Keynesian pessimism *stronger*, not weaker, than the risk-based version: there is no presumption that decentralized reasoning finds the stag.)

The three authors then divide the labor as follows:

| Layer | Question | Keynes' term | Supplied by |
|---|---|---|---|
| Payoff structure | Why are there multiple equilibria at all? | Monetary economy; liquidity (Ch. 17) | Keynes |
| Equilibrium selection | Which equilibrium do we land on? | Convention, beauty contest, state of confidence (Ch. 12) | Schelling: focal points, salience |
| Dynamics | How do we move between them? | Violent revision, animal spirits, multiplier | Schelling: tipping, critical mass, coupling |
| Stability | Why doesn't selection dissolve into constant flux? | "The essence of this convention"; sticky wages (Ch. 2) | Heath: deontic constraint, normative thickness |
| Purpose & policy | What is the cooperative surplus and who underwrites it? | Full employment; socialisation of investment (Ch. 24) | Heath: benefits of cooperation, risk pooling; Schelling: commitment |

### 4.2 Confidence, formally

"The state of confidence" — Keynes admits businessmen watch it obsessively while economists cannot define it — becomes definable: **confidence is the subjective probability each agent assigns to sufficient aggregate Commitment, given the current convention and its normative backing.** It has three distinct supports, which the model treats as separate parameters:

1. **σ (salience/authority):** the clarity and authority of the reigning focal point — is there one obvious thing to expect? (Schelling)
2. **ν (normative thickness):** the degree to which commitment is norm-governed rather than continuously recalculated — how much contrary news does it take before agents *permit themselves* to defect? (Heath)
3. **λ (liquidity pull):** the attractiveness and availability of the exit option — the certain payoff of Withdraw. (Keynes)

A slump is any event that lowers σ (the focal point is discredited), erodes ν (defection becomes normal), or raises λ (safety pays better) past the tipping threshold. The three parameters also define the **policy space**, and — as Section 5 argues — they are the parameters along which the US, Canada, China, and Japan most visibly differ.

### 4.3 Crisis as convention collapse

The model's crisis sequence, composited from 1929, 1990 (Japan), 2008 (US), and 2021 (China property):

1. A thin-ν arena (speculative asset market: the most anonymous, exit-friendly, purely instrumental arena in the economy) runs the beauty contest to an extreme valuation.
2. An event makes the loss of common knowledge itself common knowledge — Schelling's discontinuity. The convention does not adjust; it *vacates*. ("The practice of calmness and immobility, of certainty and security, suddenly breaks down," as Keynes put it in the 1937 QJE restatement.)
3. With σ destroyed, agents fall back on the only focal point that needs no coordination: **liquidity** (λ dominates). Flight to money is the one move you can make without guessing what others think.
4. The tipping dynamics of Section 2.3 propagate the withdrawal through demand coupling (multiplier), and the economy settles at the hare equilibrium.
5. Recovery has no reliable spontaneous mechanism, because no private agent can unilaterally re-supply σ: re-committing alone is simply losing the stag hunt by yourself. Someone must (a) be *loud* enough to create a new focal point, (b) be *bound* enough for the focal point to be credible, and (c) *pool the risk* of the transition. (a) is Schelling salience, (b) is Schelling commitment, (c) is Heath risk-pooling — and in most societies the only agent plausibly possessing all three at economy scale is the state. This turns Chapter 24 from an assertion into an argument (not a derivation: the premises — that no private coalition can assemble the three — are empirical, and Section 5's China case shows the state can also *fail* the credibility condition).

### 4.4 Policy as coordination engineering

The weave reclassifies the Keynesian toolkit by mechanism rather than instrument:

- **Focal-point supply (σ):** inflation targets, forward guidance, exchange-rate pegs, "whatever it takes," five-year plans, announced public investment pipelines. These work *when believed* by giving the beauty contest a new answer — policy as the loud third party of Section 2.1.
- **Commitment manufacture (σ-credibility):** central-bank independence, fiscal rules with escape clauses, treaty obligations, automatic stabilizers (commitment mechanized — no discretion to fail to respond).
- **Norm maintenance (ν):** financial regulation as practice-thickening (Keynes' transfer tax; margin rules; the post-2008 macroprudential turn); wage-bargaining architecture; the ethics of forbearance in crisis (Heath's market-failures approach applied to panics: don't exploit the fire sale).
- **Exit-option management (λ):** the interest rate as the price of the hare; deposit insurance and lender-of-last-resort as *detoxifying* the exit (making safety available without demand collapse); in the limit, negative rates and cash restrictions as raising the hare's cost.
- **Direct commitment of demand (bypassing selection):** public investment — the state simply *plays Commit* at scale, raising the payoff to everyone else's Commit until the private tipping threshold is crossed. This is why fiscal policy can work when announcements fail: it asks much less of belief — though not nothing, since its effect still depends on the private sector expecting the commitment to persist rather than to be reversed or taxed back.

### 4.5 What the weave predicts (and the postwar synthesis didn't)

1. **Discontinuity:** confidence crises are typically cliff events rather than smooth deteriorations (convention collapse, not parameter drift).
2. **Hysteresis of conventions:** after a collapse, the *new* convention (e.g., "deflation is normal," "property only rises") is itself sticky — slumps and bubbles both self-perpetuate. Recovery of output does not automatically recover ν or σ.
3. **Institution-dependence:** identical shocks produce different macro dynamics in societies with different σ, ν, λ — there is no institution-free macroeconomics. (This is the license for Section 5.)
4. **Limits of monetarism:** where ν and σ are damaged, interest-rate policy mostly re-prices an exit everyone has already taken (Japan, 1995–2012; the liquidity trap as coordination failure, not a technical floor).
5. **The casino conjecture:** the more purely instrumental (thin-ν, high-liquidity) an asset market's culture, the more violent its conventional revisions — testable across financial systems (Section 5).

A caveat on the word "predicts": the crisis cases used throughout (1929, Japan 1990, US 2008, China 2021) also informed the model's construction. Section 5 is therefore calibration and illustration — a demonstration that the framework organizes the cases coherently — not out-of-sample confirmation. The genuinely testable content is the comparative structure (same shock, different regimes, different dynamics) and the research agenda in Section 7.

---

## 5. Calibration: four societies, four coordination regimes

*(This section states the calibrated model; the empirical anchors and sources are gathered in Appendix A.)*

The model of Section 4 says macro dynamics are governed by (σ, ν, λ): who supplies focal points and with what authority; how thick the norms are that hold commitment steady; and how seductive the exit into liquidity is. The four societies picked out here make a useful comparative set because they sit in four different regions of that parameter space — with the honest proviso that four cases selected with the model in mind constitute an illustration and a plausibility probe, not an experiment.

**A note on measurement before calibrating.** The survey instruments do not agree with each other, and the disagreements are informative. On World Values Survey generalized trust ("most people can be trusted," Wave 7), the ordering is China 63.5% > Canada 46.7% > US 37.0% > Japan 33.7% — which contradicts both Fukuyama's classic classification (US and Japan high-trust, China low-trust/familistic) and the Edelman Trust Barometer (2025: China 77, Canada ≈52, US 47, Japan 37 — the *lowest* in Edelman's entire sample). The resolution is that these instruments measure different objects, and none of them measures ν directly. WVS-type questions capture *stated generalized trust*, which in China's case is dominated by in-group trust within a narrow radius (the Delhey–Welzel radius-of-trust point, and the standard reconciliation with Fukuyama's familism), and which in authoritarian survey contexts is plausibly inflated. Edelman captures sentiment toward *named institutions*. ν in this model is neither: it is the **thickness of the practices that actually govern commitment** — observable in behavior (does the bank get bailed out? does the firm do layoffs? does the wage round bind?), not in survey answers. Japan is the clearest exhibit: it scores *low* on stated trust (WVS 33.7%, Edelman last, OECD trust-in-government ~26%) while exhibiting the tightest norms Gelfand's instrument measured among large advanced economies (8.6 on the 2011 *Science* scale, vs. China 7.9 and the US 5.1) and organizational commitment practices among the thickest in the capitalist world. High ν does not mean people *say* they trust; it means defection from the practice is not among the live options. The calibration below therefore leans on institutional behavior, using survey data only as corroborating texture. (Full figures and caveats in Appendix A.)

### 5.1 United States: high salience-volatility, thin norms, glamorous exit

The US is the closest real-world approximation to the *General Theory*'s Chapter 12 economy — which is unsurprising, since Keynes calibrated Chapter 12 on Wall Street ("of the fluid markets of the world, New York is the most extreme"). The empirical profile is consistent across instruments: the paradigm Liberal Market Economy in Hall & Soskice's classification (coordination via competitive markets and formal contract); the most individualist society on Hofstede's classic scale (91/100, though WVS-based re-estimates shrink the Anglo–East Asian gap); culturally loose (Gelfand 5.1, vs. Japan's 8.6); stock market capitalization at 216% of GDP in 2024 — the highest of any major economy and roughly 3.4× China's ratio — so the beauty contest is the economy's principal capital-allocation mechanism to a degree true nowhere else; employment-at-will as the default labor doctrine; and a low household saving rate (4.6% in 2024), i.e., income commits to current demand readily *when confidence permits*. Conventions form fast, propagate fast (a deep, homogeneous media-finance culture is a salience machine — Shiller's *Narrative Economics* is essentially a study of American σ), and break fast: the fragmented, crisis-prone banking history that Bordo, Redish & Rockoff contrast with Canada's (crises in 1907, the 1930s, 2008) is thin-ν finance doing what thin-ν finance does. Meanwhile trust in the federal government has fallen from 73% (1958) to 17–22% (2024–25) — the *public* σ-supplier operates on badly eroded normative capital, which is why so much of the confidence-management burden has migrated to one of the few institutions still broadly credible to markets: the Federal Reserve, whose evolution from secrecy (pre-1994) to forward guidance, dot plots, and open-ended crisis facilities is the Schelling program adopted as official doctrine. **US regime: weak ν, strong but unstable σ (privately generated, publicly patched by the Fed), high λ glamour — maximum multiplier on animal spirits in both directions.**

### 5.2 Canada: the Heathian control case

Canada shares the US's market economy, culture, and continent — it is classified as an LME in the same varieties-of-capitalism family, with a nearly-as-individualist culture (Hofstede 80) — yet exhibits systematically damped Keynesian dynamics, which is why Heath could write *The Efficient Society* about it. The difference is institutional, and it is observable. Banking: an oligopoly of large, nationally-branched, diversified banks under a single federal regulator — a structure Bordo, Redish & Rockoff trace to the 1867 founding design — that has come through 1907, the 1930s, and 2008 without systemic failure. In 2008–09 Canada had **no bank failures and no bank bailouts** (extraordinary liquidity support, yes; equity rescues, no), a shorter and shallower recession than the US, and the World Economic Forum ranked its banking system the world's soundest for six consecutive years (2007–2013). That is thick-ν finance by design, trading dynamism for assurance. Risk pooling: universal single-payer health insurance (Canada Health Act) and constitutionally entrenched federal equalization lower the precautionary component of λ — the household need not self-insure against catastrophe by hoarding. And the survey data cooperate for once: Canada's generalized trust (46.7%, WVS Wave 7) runs ten points above the US, and trust in the national government (49%, OECD 2024) sits well above the OECD average of 39% and far above US levels. The result is textbook: Canada imported the 2008 shock through trade, not through domestic convention collapse — a recession without a panic. **Canada regime: moderate σ, high ν, managed λ — the demonstration that Anglo-liberal economies can buy convention stability with cooperative institutions, at some cost in exuberance.** Canada functions in this comparison almost as a control case for the US — "almost," because countries are not randomized and Canada differs from the US in more than its institutions. (Two further caveats keep the case honest: Edelman's 2025 Canadian report is titled around "grievance taking hold" — the normative capital is being drawn down, not guaranteed — and Canada's household debt-to-income ratio is now among the OECD's highest, a λ-vulnerability the 2008 story does not cover.)

### 5.3 Japan: thick norms, broken focal point

Japan is the model's deepest lesson. Its postwar coordination regime — main-bank monitoring, keiretsu cross-shareholding, lifetime employment, shuntō synchronized wage rounds — may be the thickest-ν capitalism yet constructed: Aoki formalized it as a distinct, internally consistent equilibrium held together by institutional complementarities (the J-firm's horizontal information structure only pays given long-term employment, which only pays given main-bank contingent governance, and so on). This machinery produced miracle-era growth — assurance so strong that firms could commit to decades-long investment — and then, after the bubble collapse (Nikkei peak 38,916 on 29 December 1989; Tokyo commercial land down roughly 80% peak-to-trough over the following decade and a half), the same machinery *held the economy at the bad equilibrium with equal firmness*. Thick ν cuts both ways: norms against layoffs and against letting relationship-borrowers fail prevented an American-style violent purge, but also entrenched the new convention — "prices do not rise; the safe thing is to pay down debt." Koo's balance-sheet recession thesis documents from flow-of-funds data what the model calls the corporate sector collectively playing Withdraw: firms switched from profit maximization to debt minimization *even at zero interest rates*. And zero rates could not fix it — the Bank of Japan hit zero in February 1999 and stayed at or below it for a quarter century — the pattern Section 4.5(4) describes: λ-pricing does little when the convention itself says Commit doesn't pay. (Note the λ nuance the data force: Japanese households are no longer big *savers* — the flow saving rate had fallen to ~1.5% by 2023 as the population aged — but their accumulated wealth stayed parked in the ultimate Withdraw asset, currency and deposits; Japan's λ is a stock phenomenon and a corporate-sector phenomenon, not a household-flow one.) Re-anchoring required an unusually forceful *external* focal-point shock — Abenomics' explicit 2% target and "regime change" rhetoric began the work; the 2022–24 global inflation shock did much of the rest; and the revival of shuntō as a live convention is the visible signature of the new equilibrium: a 5.24% settlement in 2024 (first above 5% since 1991), ~5.4% in 2025. Note also that the thick-ν bundle itself has partially unwound — non-regular workers are now 36.8% of employees (2024), cross-held shares fell below 10% of holdings by 2017, the classic main-bank monitoring role is gone — so the Japan that escaped the deflation convention is also a Japan with structurally thinner ν than the one that got locked into it, the complementarity-unraveling dynamic Aoki's framework describes. **Japan regime: very high (but eroding) ν, σ monopolized by consensus institutions — devastating when the consensus convention is deflationary — and a deeply entrenched stock-λ. Thick norms make conventions durable, including the ones you desperately need to break.**

### 5.4 China: the state as monopoly focal-point supplier

China inverts the US case: σ is supplied predominantly by the state, through instruments few other economies possess at this scale — five-year plans, growth targets, window guidance (the PBOC's *structural* lending facilities alone exceeded RMB 7 trillion by end-2024, on top of informal guidance to a banking system dominated by state-controlled institutions), and above all the implicit guarantee as the master convention: "the government will not let X fall." The VoC literature concedes China fits neither of its boxes and reaches for new labels ("state-permeated capitalism," Nölke et al.); in this model's terms the label is simple: *monopoly σ*. Beneath it, generalized ν beyond particularist networks is historically thin — Fukuyama's "familistic" classification and Greif's collectivist (Maghribi-type) equilibrium of multilateral, in-group reputation enforcement both describe guanxi-based commerce, and both are consistent with China's *high* WVS trust score once that score is read as narrow-radius trust (see the measurement note above). And λ is enormous: gross national saving of 42.5–43% of GDP (2023), the highest of any major economy, with urban household saving around 30–35% of disposable income — driven, as Chamon & Prasad and the subsequent IMF literature document, in large part by the privatized burden of education, health, housing, and pension uncertainty. This is a Heathian risk-pooling gap functioning as a permanent structural drag on Commit: Keynes' precautionary motive institutionalized at civilization scale. The regime can achieve coordination feats unavailable elsewhere — the 2008–10 stimulus was arguably the largest deliberate re-coordination of a national stag hunt on record — but it concentrates conventional risk on a single focal point, and the property episode shows what happens when the monopolist withdraws it. Housing (roughly 70% of household wealth) was a state-salience beauty contest: "housing only rises, because the government cannot afford otherwise." The Three Red Lines (August 2020) were a deliberate σ-withdrawal; Evergrande — over $300 billion in liabilities — defaulted in December 2021 and was ordered into liquidation in January 2024, with thirty-plus developers following; pre-sale buyers discovered the guarantee was gone (the 2022 mortgage boycotts on unfinished projects are a strikingly clean case of conditional cooperation running in reverse); and the aftermath — household retrenchment, deflationary drift, piecemeal confidence measures underperforming from 2022 through 2025 even as the Red Lines themselves were quietly retired — reads as a Keynesian convention collapse *without* the market-panic phase but *with* the full withdrawal-equilibrium aftermath, resistant to interest-rate medicine for the same reason Japan's was. When the dominant supplier of salience deliberately breaks a convention it created, no private mechanism of comparable reach exists to form a replacement. **China regime: monopoly σ (high power, single point of failure), particularist ν, extreme precautionary λ rooted in the risk-pooling gap. The model's clearest prescription follows: China's cheapest path to re-coordination is Heathian, not monetary — pool household risk (health, pension, hukou reform, on which the July 2024 State Council action plan is a start) to lower λ structurally, because the σ monopoly cannot credibly re-inflate the convention it itself shot.**

### 5.5 The comparative table

| Parameter | United States | Canada | Japan | China |
|---|---|---|---|---|
| Focal-point supply (σ) | Market-generated, volatile; Fed as patcher | Mixed, stable | Consensus institutions; slow to change | State monopoly; powerful, single point of failure |
| Normative thickness (ν) | Thin, contractual | High, institutional | Very high, relational-organizational | Particularist (guanxi) + state guarantee |
| Exit pull (λ) | High-glamour financial liquidity | Managed (risk pooled) | Entrenched stock-λ (deposit-heavy wealth; corporate debt-minimization) | Extreme precautionary saving (insurance gap) |
| Characteristic failure | Speculative convention collapse (1929, 2008) | Imported recessions | Convention lock-in at bad equilibrium (1990s–2010s) | Focal-point rupture by the supplier (2021–) |
| Characteristic strength | Fast re-coordination, deep re-allocation | Crisis avoidance | Long-horizon commitment | Mobilization at will |
| Keynes chapter it illustrates | Ch. 12 (beauty contest) | Ch. 24 (the compromise working) | Ch. 17 + liquidity trap (Ch. 15) | Ch. 12 + Ch. 24 fused in one actor |
| Effective policy margin | σ-repair (Fed credibility) + ν-thickening (macroprudential) | Maintenance | σ-rupture of the bad convention (Abenomics) | λ-reduction via risk pooling (welfare state) |

The cross-country pattern: **σ, ν, and λ are partial substitutes in supplying assurance, and each society leans on the one its history made cheap** — the US on privately generated salience, Canada on institutional norms, Japan on organizational norms, China on state salience. Each society's characteristic macro-pathology is the failure mode of its own favorite instrument. There is, therefore, no universal Keynesianism: the *General Theory*'s policy chapters are a menu whose correct item depends on which assurance technology is locally broken.

---

## 6. The wider intersection: a guide to the literature

*(Annotated; full citations in Section 8. Grouped by the role each body of work plays in the weave. A note on novelty: a systematic search finds **almost no existing scholarship whose central project is connecting Schelling to Keynes, and none connecting Heath to Keynes or to macroeconomics at all**. The exceptions prove instructive: Wilson (2021) establishes the *historical* Schelling–Keynes link (Schelling's game theory grew out of his Keynesian training — see Section 2.0) but does not build the theoretical weave forward into macroeconomics; Lanteri & Carabelli (2011) argue that Keynes' original beauty contest is closer to a Schelling-style focal-point problem than to the dominance-solvable guessing game the experimental literature made of it; and the higher-order-beliefs finance literature (Morris–Shin; Allen–Morris–Shin) formalizes the beauty contest without the normative layer. The three-way weave attempted in Sections 3–4 — Schelling mechanics, Heath foundations, Keynesian payoff structure — appears to be genuinely open territory, surrounded by rich adjacent literatures, mapped below.)*

### 6.1 The coordination reading of Keynes (the hinge)

- **Benjamin Wilson, "Keynes Goes Nuclear: Thomas Schelling and the Macroeconomic Origins of Strategic Stability" (*Modern Intellectual History*, 2021)** — the historical demonstration that Schelling's strategic theory descends from his Keynesian macroeconomics (see Section 2.0). The intellectual charter for this paper.
- **Axel Leijonhufvud, *On Keynesian Economics and the Economics of Keynes* (1968)** — the founding modern reinterpretation of Keynes as a theorist of inter-agent coordination failure rather than sticky wages; set the agenda every later formalization works within.
- **John Latsis, Guillemette de Larquier & Franck Bessis, "Are Conventions Solutions to Uncertainty? Contrasting Visions of Social Coordination" (*JPKE*, 2010)** — the keystone map of this whole terrain: explicitly contrasts the three developed accounts of convention — game-theoretic (Lewis–Sugden–Young), Keynesian/post-Keynesian, and French *économie des conventions* — and assesses their (surprisingly thin) overlap. The present paper is an attempt to *increase* that overlap. See also Latsis, "Is There Redemption for Conventions?" (*CJE*, 2005).
- **Jean-Pierre Dupuy, "Convention et Common Knowledge" (*Revue économique*, 1989)** — the most direct confrontation of Lewis's common-knowledge account of convention with Keynes' specular (beauty-contest) logic.
- **Jörg Bibow, Paul Lewis & Jochen Runde, "Uncertainty, Conventional Behavior, and Economic Sociology" (*AJES*, 2005)** — a rare explicit three-way bridge between Keynes' stock-market conventions, the French mimetic school, and Cambridge social ontology.
- **Shaun Hargreaves Heap, *The New Keynesian Macroeconomics: Time, Belief and Social Interdependence* (1992)** — an early, underappreciated synthesis reconstructing Keynesian macro around belief interdependence, conventions, and norms with explicitly game-theoretic tools; his *Game Theory: A Critical Introduction* (with Varoufakis) treats the beauty contest in the same spirit.
- **Alessandro Lanteri & Anna Carabelli, "Beauty Contested" (*EJHET*, 2011)** — argues the experimental p-beauty-contest tradition mis-translates Keynes: his original game is not dominance-solvable but a Schelling-style coordination problem under uncertainty. The closest existing text to this paper's Schelling–Keynes hinge.

### 6.2 The game theory of convention (the Schelling line)

- **David Lewis, *Convention* (1969)** — took Schelling's focal points and built the first rigorous theory of convention as a self-perpetuating solution to recurring coordination problems, defining common knowledge en route. The formal definition of what Keynes' Chapter 12 "convention" *is* — though Davis (1997) and Latsis et al. (2010) argue the fit is imperfect, since Keynes' convention operates where the common-knowledge basis Lewis requires is unavailable.
- **Robert Sugden, *The Economics of Rights, Co-operation and Welfare* (1986; 2nd ed. 2004)** — conventions emerge spontaneously by evolutionary dynamics and acquire *normative* force; with **Mehta, Starmer & Sugden's** experimental work on primary vs. secondary salience (*AER*, 1994) and his team-reasoning papers, Sugden is the closest thing to a one-man Schelling–Heath bridge, though he never wrote directly on Keynes.
- **Michael Bacharach, *Beyond Individual Choice* (2006, ed. Gold & Sugden)** — team reasoning: agents who frame the situation as "what should *we* do?" transform stag hunts into single-agent problems. A rival (or complement) to Heath's deontic account of how assurance problems actually get solved — and a rationality-based alternative to animal spirits as the selector of the good equilibrium.
- **Brian Skyrms, *The Stag Hunt and the Evolution of Social Structure* (2004)** — makes the stag hunt (Section 4.1's chassis) the master model of the social contract; evolutionary and signaling dynamics of the assurance problem.
- **H. Peyton Young, "The Evolution of Conventions" (*Econometrica*, 1993) and *Individual Strategy and Social Structure* (1998)** — stochastic evolution of conventions: which conventions are stochastically stable, and how societies *switch* conventions in punctuated shifts — the formal dynamics of the conventional change Keynes could only describe as "sudden and violent."
- **Ken Binmore, *Game Theory and the Social Contract* (1994/1998); *Natural Justice* (2005)** — fairness norms as equilibrium-selection devices in the game of life; a naturalistic rival to Heath on the same explanandum.
- **Cristina Bicchieri, *The Grammar of Society* (2006)** — social norms as conditional preferences keyed to empirical and normative expectations; supplies the measurement theory for ν, and the account of how norms unravel (relevant to convention collapse).
- **Cyril Hédoin** (e.g., "Community-Based Reasoning in Games," *Games*, 2016) — contemporary bridge figure connecting Lewisian convention, rule-following, and institutional economics; salience as a function of community membership (which is why σ is society-specific, as Section 5 requires).

### 6.3 Coordination-failure macroeconomics (the Keynes-formalized line)

- **John Bryant, "A Simple Rational Expectations Keynes-type Model" (*QJE*, 1983)** and **Russell Cooper & Andrew John, "Coordinating Coordination Failures in Keynesian Models" (*QJE*, 1988)** — the canonical demonstrations that strategic complementarities generate multiple Pareto-ranked macro equilibria: Keynesian unemployment as coordination on the bad equilibrium of an economy-scale stag hunt. Section 4.1 is this literature with Schelling selection and Heath stability added. Cooper's *Coordination Games* (1999) consolidates the program.
- **Peter Diamond, "Aggregate Demand Management in Search Equilibrium" (*JPE*, 1982)** — the coconut model: trading externalities generate multiple Pareto-ranked steady states; optimism about others' trading is self-fulfilling.
- **David Cass & Karl Shell, "Do Sunspots Matter?" (*JPE*, 1983)** — extrinsic uncertainty can coordinate behavior in equilibrium; a sunspot is a Schelling focal point admitted into rational-expectations theory, i.e., a convention formalized.
- **Peter Howitt & R. Preston McAfee, "Animal Spirits" (*AER*, 1992)** — self-fulfilling waves of optimism and pessimism switch a search economy between high- and low-activity equilibria under fully rational expectations: Keynes' animal spirits recast as an equilibrium-selection variable.
- **Roger Farmer, *The Macroeconomics of Self-Fulfilling Prophecies* (1993); *Expectations, Employment and Prices* (2010); *Prosperity for All* (2016)** — beliefs as an independent fundamental; the "belief function" supplies the missing equilibrium-selection equation, and a continuum of unemployment steady states replaces the natural rate. The closest thing modern macro has to a formal Chapter 12.
- **Stephen Morris & Hyun Song Shin, "Social Value of Public Information" (*AER*, 2002)** and **Franklin Allen, Morris & Shin, "Beauty Contests and Iterated Expectations in Asset Markets" (*RFS*, 2006)** — the rationalist vindication of the beauty contest: with higher-order beliefs, the law of iterated expectations fails for average opinion, prices overweight public signals *because* they are focal, and public information is powerful for Schelling reasons, not informational ones. Their global-games program (Morris–Shin, *AER*, 1998) turns multiple-equilibrium crisis stories into a theory of *when* conventions break.
- **Van Huyck, Battalio & Beil, "Tacit Coordination Games, Strategic Uncertainty, and Coordination Failure" (*AER*, 1990)** — the canonical experiments: real groups playing stag-hunt-like games reliably converge on inefficient, secure equilibria as group size grows. The laboratory counterpart of involuntary unemployment — and of why large anonymous economies (low ν) coordinate worse than small thick ones.
- **Rosemarie Nagel, "Unraveling in Guessing Games" (*AER*, 1995)**, **Bosch-Domènech, Montalvo, Nagel & Satorra (*AER*, 2002)**, **Camerer, *Behavioral Game Theory* (2003)**, and **Mauersberger & Nagel's** *Handbook of Computational Economics* chapter (2018) — Keynes' guessing game made operational; level-k reasoning as measured, finite-depth iteration of "average opinion about average opinion," run in labs and in actual newspapers; the 2018 chapter argues formally that New Keynesian expectations models have beauty-contest structure.
- **George Akerlof & Robert Shiller, *Animal Spirits* (2009)** — the behavioral-macro restatement: confidence, fairness, corruption, money illusion, and *stories* as macro forces. Shiller's *Narrative Economics* (2019) is σ studied as epidemiology. **Marchionatti (*Kyklos*, 1999)** disciplines the appropriation: in Keynes, animal spirits are non-rational but *reasonable* conduct under uncertainty, not a bias.
- **Roman Frydman & Michael Goldberg, *Imperfect Knowledge Economics* (2007)**; **John Kay & Mervyn King, *Radical Uncertainty* (2020)** — the modern restatements of Keynesian uncertainty against probabilistic reduction; Kay and King's "reference narratives" as devices for coping and coordinating are Schelling σ under another name.

### 6.4 Keynes scholarship on convention and uncertainty (the philology)

- **Anna Carabelli, *On Keynes's Method* (1988)**; **Rod O'Donnell, *Keynes: Philosophy, Economics and Politics* (1989)**; **John B. Davis, *Keynes's Philosophical Development* (1994)** and **"J. M. Keynes on History and Convention" (1997)**; **Sheila Dow** (with Alexander Dow, "Animal Spirits and Rationality," 1985); **Victoria Chick, *Macroeconomics after Keynes* (1983)**; **Tony Lawson, "Keynes and Conventions" (*Review of Social Economy*, 1993)** — establish from the *Treatise on Probability* through the 1937 QJE restatement that convention, weight of argument, and non-numerical probability are the *philosophical core* of Keynes, not local color. Davis is especially important for this paper: he argues the later Keynes held an irreducibly *social*, interdependent conception of judgment — and warns where Lewis-style equilibrium readings of Keynesian convention overreach.
- **Jochen Runde & Sohei Mizuhara (eds.), *The Philosophy of Keynes' Economics: Probability, Uncertainty and Convention* (2003)** — the central edited volume on this intersection's philosophical side, with a dedicated section on convention.
- **David Dequech** — "Financial Conventions in Keynes's Theory: The Stock Exchange" (*JPKE*, 2011), "Conventions in Keynes's Theory of Goods Markets" (*JPKE*, 2022), and related papers: the most systematic contemporary taxonomist of Keynesian conventions, distinguishing conformity mechanisms and relating conventions to institutions and to the game-theoretic literature. The nearest existing analogue of this paper's ν parameter.
- **The French *économie des conventions* school** — founded in the 1989 *Revue économique* special issue (Dupuy, Eymard-Duvernay, Favereau, Orléan, Salais, Thévenot), with Keynes' convention explicitly as its point of departure. **Olivier Favereau** ("La Théorie Générale: de l'économie conventionnelle à l'économie des conventions," 1988; "Keynes After the Economics of Conventions," 2013) calls Keynes "the first and greatest conventionalist economist" and reads both 1929 and 2008 as failures of twin conventions. **André Orléan** ("Mimetic Contagion and Speculative Bubbles," 1989; *The Empire of Value*, 2014) generalizes the beauty contest into a full theory of economic value: liquidity requires a "convention of valuation" produced by mimetic polarization, not discovery of fundamentals. **Boltanski & Thévenot, *On Justification* (1991/2006)** supply the school's sociological wing — plural "orders of worth" as competing conventions of evaluation, useful for Section 5's claim that different societies justify commitment differently.
- **Jens Beckert, *Imagined Futures: Fictional Expectations and Capitalist Dynamics* (2016)** — "fictional expectations" as the sociology of Chapter 12: under fundamental uncertainty, coordination runs on shared imaginaries of the future, which are politically produced and contested.

### 6.5 Cooperation, norms, and the moral economy (the Heath line)

- **Samuel Bowles & Herbert Gintis, *A Cooperative Species* (2011)**; **Ernst Fehr and colleagues** (Fehr & Gächter, *AER*, 2000; Fischbacher, Gächter & Fehr, *Economics Letters*, 2001) — the empirical human is a *conditional* cooperator with a taste for norm enforcement: people cooperate if they expect others to. Aggregate cooperation therefore has beauty-contest structure itself — which is the experimental basis for ν, and why Withdraw cascades once norm violation becomes visible (conditional cooperation running in reverse is a bank run).
- **Elinor Ostrom, *Governing the Commons* (1990)** — self-organized assurance without the state; the boundary conditions (small scale, monitoring, graduated sanctions) that explain why *macro* assurance, which violates all of them, needs the state.
- **George Akerlof** (fair-wage/effort models; *Identity Economics* with Kranton, 2010) and **Truman Bewley, *Why Wages Don't Fall During a Recession* (1999)** — the Heathian reading of wage stickiness (Section 3.1) documented in the field.
- **Albert Hirschman, *Exit, Voice, and Loyalty* (1970)** — λ is exit; ν is loyalty; the trade-off between them is Hirschman's, and his *The Passions and the Interests* (1977) is the deep history of Heath's efficiency defense of cooperative capitalism.

### 6.6 Comparative institutional calibration (the Section 5 toolkit)

- **Peter Hall & David Soskice, *Varieties of Capitalism* (2001)** — LME/CME as (explicitly game-theoretic) national solutions to coordination problems; the US/Japan contrast is their paradigm case. For China, which the binary cannot hold, see **Witt & Redding's** business-systems mapping (*Socio-Economic Review*, 2013) and **Nölke, ten Brink, May & Claar, *State-Permeated Capitalism in Large Emerging Economies* (2019)**.
- **Masahiko Aoki, *Toward a Comparative Institutional Analysis* (2001)** — institutions as self-sustaining systems of shared beliefs, i.e., equilibrium summaries of societal games; institutional complementarities generate multiple viable national configurations. The formal bridge between Schelling conventions and national systems, built by the leading theorist of the Japanese firm.
- **Avner Greif, "Cultural Beliefs and the Organization of Society" (*JPE*, 1994) and *Institutions and the Path to the Modern Economy* (2006)** — collectivist vs. individualist equilibria (Maghribi/Genoese) as durable cultural conventions of enforcement; the historical demonstration that shared expectations determine *which economy you get*, and the foundation for the China/US ν contrast.
- **Francis Fukuyama, *Trust* (1995)**; **Joseph Henrich, *The WEIRDest People in the World* (2020)**; **Michele Gelfand, *Rule Makers, Rule Breakers* (2018)** — the trust-radius, kinship-psychology, and tightness-looseness measurements underlying the four-country parameterization.
- **Richard Koo, *The Holy Grail of Macroeconomics* (2008)** — balance-sheet recession: Japan's private sector minimizing debt rather than maximizing profit is the Withdraw equilibrium documented from flow-of-funds data; Koo is also the standard lens now applied to post-2021 China.

---

## 7. Coda: what the weave changes

Three conclusions, in ascending order of ambition.

**For reading Keynes.** The *General Theory* rewards being read as an early and remarkably complete description of a monetary stag hunt: an economy whose multiple equilibria are created by liquidity (Ch. 17), selected by convention and salience (Ch. 12), propagated by coupling (the multiplier), stabilized by norms (Ch. 2's wage conventions), and governable by the one agent plausibly capable of commitment and risk pooling at scale (Ch. 24). Much of the machinery Keynes lacked in 1936 now exists: Schelling built the selection and dynamics, Heath built the stability and the normative accounting, and the coordination-failure literature built the formal chassis. What has not been done — and what this paper sketches — is assembling them in one frame. The historical warrant is stronger than generally noticed: Schelling's game theory was Keynesian macrodynamics exported to strategy (Wilson 2021); bringing it back to macroeconomics is repatriation, not appropriation.

**For macroeconomic policy.** "Confidence" decomposes into three separately manipulable objects — salience (σ), normative thickness (ν), and exit pull (λ) — and the four-country calibration shows that societies differ not in whether they solve the assurance problem but in *which instrument they have historically leaned on*, and that each society's characteristic crisis is the failure mode of its own favorite instrument. Policy advice that ignores this is exporting a solution to a problem the importing society doesn't have: telling 1990s Japan to cut rates (its problem was ν-lock-in, not λ-pricing), or telling 2020s China to cut rates (its problem is a σ-rupture plus a λ rooted in the risk-pooling gap), repeats the same category error. The general prescription is Tinbergen for coordination: match the instrument to the broken parameter.

**For political philosophy.** Heath's program and Keynes' converge on a single claim from opposite directions: on this reading, the market order's deepest vulnerability is not injustice but *assurance failure*, and the institutions that remedy it — the welfare state as risk pool, the central bank as focal point, financial regulation as norm-thickener, public investment as standing commitment — are justified on efficiency grounds internal to the market's own logic, needing no appeal beyond it. That is why the weave holds: Schelling explains how strangers coordinate, Keynes explains why a monetary economy of strangers can coordinate on disaster, and Heath explains why the repair of that failure is not a departure from the cooperative order but its completion.

A research agenda follows naturally: formalize the (σ, ν, λ) stag hunt and derive the comparative statics of Section 5.5; operationalize ν behaviorally (Bicchieri-style norm measurement inside financial and wage-setting practices, extending Dequech's taxonomy); test the "casino conjecture" (conventional-revision violence as a function of a market's normative thinness) across the four financial systems; and run the beauty-contest experiments cross-culturally with Gelfand tightness as a treatment variable — the lab analogue of the calibration attempted here.

---

## 8. References

*(Bibliographic details verified by web search during preparation except where marked ◊, which indicates a standard citation not independently re-verified.)*

**Primary texts**

- Keynes, J. M. (1936). *The General Theory of Employment, Interest and Money*. London: Macmillan.
- Keynes, J. M. (1937). "The General Theory of Employment." *Quarterly Journal of Economics* 51(2): 209–223.
- Schelling, T. C. (1947). "Capital Growth and Equilibrium." *American Economic Review* 37(5).
- Schelling, T. C. (1951). *National Income Behavior: An Introduction to Algebraic Analysis*. New York: McGraw-Hill.
- Schelling, T. C. (1960). *The Strategy of Conflict*. Cambridge, MA: Harvard University Press.
- Schelling, T. C. (1966). *Arms and Influence*. New Haven: Yale University Press.
- Schelling, T. C. (1971). "Dynamic Models of Segregation." *Journal of Mathematical Sociology* 1(2): 143–186.
- Schelling, T. C. (1978). *Micromotives and Macrobehavior*. New York: W. W. Norton.
- Schelling, T. C. (2006). "An Astonishing Sixty Years: The Legacy of Hiroshima" (Nobel lecture, 2005). *American Economic Review* 96(4).
- Heath, J. (2001). *Communicative Action and Rational Choice*. Cambridge, MA: MIT Press.
- Heath, J. (2001). *The Efficient Society: Why Canada Is as Close to Utopia as It Gets*. Toronto: Viking/Penguin Canada.
- Heath, J. (2006). "The Benefits of Cooperation." *Philosophy & Public Affairs* 34(4): 313–351.
- Heath, J. (2008). *Following the Rules: Practical Reasoning and Deontic Constraint*. New York: Oxford University Press.
- Heath, J. (2009). *Filthy Lucre: Economics for People Who Hate Capitalism*. Toronto: HarperCollins Canada. (US ed.: *Economics Without Illusions*, Broadway Books, 2010.)
- Heath, J. (2011). "Three Normative Models of the Welfare State." *Public Reason* 3(2): 13–43.
- Heath, J. (2014). *Morality, Competition, and the Firm: The Market Failures Approach to Business Ethics*. New York: Oxford University Press.
- Heath, J. (2020). *The Machinery of Government: Public Administration and the Liberal State*. New York: Oxford University Press.
- Heath, J. (2022). *Cooperation and Social Justice*. Toronto: University of Toronto Press.

**The hinge: Keynes, conventions, and game theory**

- Wilson, B. (2021). "Keynes Goes Nuclear: Thomas Schelling and the Macroeconomic Origins of Strategic Stability." *Modern Intellectual History* 18(1).
- Leijonhufvud, A. (1968). *On Keynesian Economics and the Economics of Keynes*. Oxford: Oxford University Press.
- Latsis, J. (2005). "Is There Redemption for Conventions?" *Cambridge Journal of Economics* 29(5).
- Latsis, J., G. de Larquier & F. Bessis (2010). "Are Conventions Solutions to Uncertainty? Contrasting Visions of Social Coordination." *Journal of Post Keynesian Economics* 32(4): 535–558.
- Dupuy, J.-P. (1989). "Convention et Common Knowledge." *Revue économique* 40(2).
- Bibow, J., P. Lewis & J. Runde (2005). "Uncertainty, Conventional Behavior, and Economic Sociology." *American Journal of Economics and Sociology* 64(2): 507–532.
- Hargreaves Heap, S. (1992). *The New Keynesian Macroeconomics: Time, Belief and Social Interdependence*. Aldershot: Edward Elgar.
- Lanteri, A. & A. Carabelli (2011). "Beauty Contested: How Much of Keynes' Remains in Behavioural Economics' Beauty Contests?" *European Journal of the History of Economic Thought* 18(2).

**Game theory of convention and cooperation**

- Lewis, D. K. (1969). *Convention: A Philosophical Study*. Cambridge, MA: Harvard University Press.
- Sugden, R. (1986). *The Economics of Rights, Co-operation and Welfare*. Oxford: Basil Blackwell. (2nd ed. Palgrave Macmillan, 2004.)
- Sugden, R. (1989). "Spontaneous Order." *Journal of Economic Perspectives* 3(4). ◊
- Sugden, R. (1993). "Thinking as a Team: Towards an Explanation of Nonselfish Behavior." *Social Philosophy and Policy* 10(1). ◊
- Mehta, J., C. Starmer & R. Sugden (1994). "The Nature of Salience: An Experimental Investigation of Pure Coordination Games." *American Economic Review* 84(3). ◊
- Bacharach, M. (2006). *Beyond Individual Choice: Teams and Frames in Game Theory* (N. Gold & R. Sugden, eds.). Princeton: Princeton University Press. ◊
- Skyrms, B. (2004). *The Stag Hunt and the Evolution of Social Structure*. Cambridge: Cambridge University Press. ◊
- Young, H. P. (1993). "The Evolution of Conventions." *Econometrica* 61(1).
- Young, H. P. (1998). *Individual Strategy and Social Structure: An Evolutionary Theory of Institutions*. Princeton: Princeton University Press.
- Binmore, K. (1994/1998). *Game Theory and the Social Contract*, 2 vols. Cambridge, MA: MIT Press. ◊
- Binmore, K. (2005). *Natural Justice*. Oxford: Oxford University Press. ◊
- Bicchieri, C. (2006). *The Grammar of Society: The Nature and Dynamics of Social Norms*. Cambridge: Cambridge University Press. ◊
- Hédoin, C. (2016). "Community-Based Reasoning in Games: Salience, Rule-Following, and Counterfactuals." *Games* 7(4): 36.

**Coordination-failure macroeconomics and the beauty contest**

- Bryant, J. (1983). "A Simple Rational Expectations Keynes-type Model." *Quarterly Journal of Economics* 98(3): 525–528.
- Diamond, P. (1982). "Aggregate Demand Management in Search Equilibrium." *Journal of Political Economy* 90(5). ◊
- Cass, D. & K. Shell (1983). "Do Sunspots Matter?" *Journal of Political Economy* 91(2). ◊
- Cooper, R. & A. John (1988). "Coordinating Coordination Failures in Keynesian Models." *Quarterly Journal of Economics* 103(3): 441–463.
- Cooper, R. (1999). *Coordination Games: Complementarities and Macroeconomics*. Cambridge: Cambridge University Press. ◊
- Van Huyck, J., R. Battalio & R. Beil (1990). "Tacit Coordination Games, Strategic Uncertainty, and Coordination Failure." *American Economic Review* 80(1). ◊
- Howitt, P. & R. P. McAfee (1992). "Animal Spirits." *American Economic Review* 82(3): 493–507.
- Farmer, R. E. A. (1993). *The Macroeconomics of Self-Fulfilling Prophecies*. Cambridge, MA: MIT Press.
- Farmer, R. E. A. (2010). *Expectations, Employment and Prices*. Oxford: Oxford University Press.
- Farmer, R. E. A. (2016). *Prosperity for All: How to Prevent Financial Crises*. Oxford: Oxford University Press.
- Morris, S. & H. S. Shin (1998). "Unique Equilibrium in a Model of Self-Fulfilling Currency Attacks." *American Economic Review* 88(3). ◊
- Morris, S. & H. S. Shin (2002). "Social Value of Public Information." *American Economic Review* 92(5). ◊
- Allen, F., S. Morris & H. S. Shin (2006). "Beauty Contests and Iterated Expectations in Asset Markets." *Review of Financial Studies* 19(3): 719–752.
- Nagel, R. (1995). "Unraveling in Guessing Games: An Experimental Study." *American Economic Review* 85(5): 1313–1326.
- Bosch-Domènech, A., J. G. Montalvo, R. Nagel & A. Satorra (2002). "One, Two, (Three), Infinity, …: Newspaper and Lab Beauty-Contest Experiments." *American Economic Review* 92(5). ◊
- Camerer, C. F. (2003). *Behavioral Game Theory: Experiments in Strategic Interaction*. Princeton: Princeton University Press. ◊
- Mauersberger, F. & R. Nagel (2018). "Levels of Reasoning in Keynesian Beauty Contests: A Generative Framework." In *Handbook of Computational Economics*, Vol. 4. Amsterdam: Elsevier.
- Akerlof, G. A. & R. J. Shiller (2009). *Animal Spirits: How Human Psychology Drives the Economy, and Why It Matters for Global Capitalism*. Princeton: Princeton University Press.
- Shiller, R. J. (2019). *Narrative Economics: How Stories Go Viral and Drive Major Economic Events*. Princeton: Princeton University Press.
- Marchionatti, R. (1999). "On Keynes' Animal Spirits." *Kyklos* 52(3).
- Frydman, R. & M. D. Goldberg (2007). *Imperfect Knowledge Economics: Exchange Rates and Risk*. Princeton: Princeton University Press. ◊
- Kay, J. & M. King (2020). *Radical Uncertainty: Decision-Making Beyond the Numbers*. New York: W. W. Norton.

**Keynes scholarship and the economics of conventions**

- Carabelli, A. (1988). *On Keynes's Method*. London: Macmillan. ◊
- O'Donnell, R. (1989). *Keynes: Philosophy, Economics and Politics*. London: Macmillan. ◊
- Davis, J. B. (1994). *Keynes's Philosophical Development*. Cambridge: Cambridge University Press. ◊
- Davis, J. B. (1997). "J. M. Keynes on History and Convention." In G. C. Harcourt & P. A. Riach (eds.), *A 'Second Edition' of The General Theory*, Vol. 2. London: Routledge.
- Lawson, T. (1993). "Keynes and Conventions." *Review of Social Economy* 51(2).
- Dow, A. & S. C. Dow (1985). "Animal Spirits and Rationality." In T. Lawson & H. Pesaran (eds.), *Keynes' Economics: Methodological Issues*. London: Croom Helm. ◊
- Chick, V. (1983). *Macroeconomics after Keynes: A Reconsideration of the General Theory*. Oxford: Philip Allan. ◊
- Runde, J. & S. Mizuhara, eds. (2003). *The Philosophy of Keynes' Economics: Probability, Uncertainty and Convention*. London: Routledge.
- Dequech, D. (2011). "Financial Conventions in Keynes's Theory: The Stock Exchange." *Journal of Post Keynesian Economics* 33(3): 469–489.
- Dequech, D. (2022). "Conventions in Keynes's Theory of Goods Markets: Investment and Production Decisions." *Journal of Post Keynesian Economics* 45(1).
- Dupuy, J.-P., F. Eymard-Duvernay, O. Favereau, A. Orléan, R. Salais & L. Thévenot, eds. (1989). "L'économie des conventions" (special issue). *Revue économique* 40(2).
- Favereau, O. (1988). "La Théorie Générale: de l'économie conventionnelle à l'économie des conventions." *Cahiers d'économie politique* 14–15: 197–220.
- Favereau, O. (2013). "Keynes After the Economics of Conventions." *Evolutionary and Institutional Economics Review* 10(2): 179–195.
- Orléan, A. (1989). "Mimetic Contagion and Speculative Bubbles." *Theory and Decision* 27.
- Orléan, A. (2014). *The Empire of Value: A New Foundation for Economics* (trans. M. B. DeBevoise). Cambridge, MA: MIT Press. (French original 2011.)
- Boltanski, L. & L. Thévenot (2006). *On Justification: Economies of Worth* (trans. C. Porter). Princeton: Princeton University Press. (French original 1991.)
- Beckert, J. (2016). *Imagined Futures: Fictional Expectations and Capitalist Dynamics*. Cambridge, MA: Harvard University Press. ◊

**Cooperation, norms, and the moral economy**

- Ostrom, E. (1990). *Governing the Commons: The Evolution of Institutions for Collective Action*. Cambridge: Cambridge University Press. ◊
- Bowles, S. & H. Gintis (2011). *A Cooperative Species: Human Reciprocity and Its Evolution*. Princeton: Princeton University Press. ◊
- Fehr, E. & S. Gächter (2000). "Cooperation and Punishment in Public Goods Experiments." *American Economic Review* 90(4). ◊
- Fischbacher, U., S. Gächter & E. Fehr (2001). "Are People Conditionally Cooperative? Evidence from a Public Goods Experiment." *Economics Letters* 71(3). ◊
- Akerlof, G. A. & R. E. Kranton (2010). *Identity Economics*. Princeton: Princeton University Press. ◊
- Bewley, T. (1999). *Why Wages Don't Fall During a Recession*. Cambridge, MA: Harvard University Press. ◊
- Hirschman, A. O. (1970). *Exit, Voice, and Loyalty*. Cambridge, MA: Harvard University Press. ◊
- Hirschman, A. O. (1977). *The Passions and the Interests*. Princeton: Princeton University Press. ◊

**Comparative institutions and country evidence**

- Hall, P. A. & D. Soskice, eds. (2001). *Varieties of Capitalism: The Institutional Foundations of Comparative Advantage*. Oxford: Oxford University Press.
- Aoki, M. (2001). *Toward a Comparative Institutional Analysis*. Cambridge, MA: MIT Press.
- Greif, A. (1994). "Cultural Beliefs and the Organization of Society: A Historical and Theoretical Reflection on Collectivist and Individualist Societies." *Journal of Political Economy* 102(5): 912–950.
- Greif, A. (2006). *Institutions and the Path to the Modern Economy: Lessons from Medieval Trade*. Cambridge: Cambridge University Press. ◊
- Fukuyama, F. (1995). *Trust: The Social Virtues and the Creation of Prosperity*. New York: Free Press.
- Henrich, J. (2020). *The WEIRDest People in the World*. New York: Farrar, Straus and Giroux.
- Gelfand, M. J., et al. (2011). "Differences Between Tight and Loose Cultures: A 33-Nation Study." *Science* 332: 1100–1104.
- Harrington, J. R. & M. J. Gelfand (2014). "Tightness–looseness across the 50 united states." *PNAS* 111(22). ◊
- Witt, M. A. & G. Redding (2013). "Asian Business Systems: Institutional Comparison, Clusters and Implications for Varieties of Capitalism and Business Systems Theory." *Socio-Economic Review* 11(2). ◊
- Nölke, A., T. ten Brink, C. May & S. Claar (2019). *State-Permeated Capitalism in Large Emerging Economies*. London: Routledge.
- Bordo, M., A. Redish & H. Rockoff (2011). "Why Didn't Canada Have a Banking Crisis in 2008 (or in 1930, or 1907, or …)?" NBER Working Paper 17312.
- Koo, R. C. (2008). *The Holy Grail of Macroeconomics: Lessons from Japan's Great Recession*. Singapore: John Wiley & Sons (Asia).
- Chamon, M. & E. Prasad (2010). "Why Are Saving Rates of Urban Households in China Rising?" *American Economic Journal: Macroeconomics* 2(1).
- Horioka, C. Y. (2024). "Household Saving in Japan: The Past, Present, and Future." NBER Working Paper 33181.

---

## Appendix A. Empirical anchors for the four-country calibration

All figures as compiled July 2026; sources and caveats noted. Saving-rate definitions differ across statistical systems — compare within-country dynamics, not levels.

### A.1 Trust and culture

| Measure | US | Canada | Japan | China |
|---|---|---|---|---|
| Generalized trust, "most people can be trusted" (Integrated Values Surveys, Wave 7, 2022) | 37.0% | 46.7% | 33.7% | 63.5% |
| Trust in national government (OECD Trust Survey 2024; US from Pew 2024–25) | 17–22% (Pew) | 49% | ~26% | — (not surveyed) |
| Edelman Trust Index 2025 (composite) | 47 | ~52.5 | 37 (lowest surveyed) | 77 (online urban sample; skews up) |
| Cultural tightness (Gelfand et al. 2011, *Science*) | 5.1 (loose) | not in study (qualitatively loose) | 8.6 (tight) | 7.9 (tight) |
| Hofstede individualism (classic scores) | 91 | 80 | 46 | 20 |

Caveats: (1) Pew's 2025 25-country survey puts US interpersonal trust at 55% — a large instrument effect vs. WVS; do not average across instruments. (2) WVS-type trust in authoritarian survey contexts is plausibly inflated and dominated by narrow-radius (in-group) trust — the reconciliation of China's 63.5% with Fukuyama's "low-trust/familistic" classification. (3) WVS-based re-estimates suggest Hofstede's classic scores overstate Anglo individualism and understate East Asian individualism; the US–China gap is real but smaller than 91-vs-20. (4) Japan's three-way tension — Fukuyama high (organizational) trust, WVS mid-30s, Edelman lowest — is discussed in Section 5's measurement note.

### A.2 Financial structure and saving

| Measure | US | Canada | Japan | China |
|---|---|---|---|---|
| Stock market capitalization / GDP (2024) | 216.3% | 150.4% | 156.7% | 62.7% |
| Household saving rate (latest, national definitions) | 4.6% (2024, BEA) | 5.0% (2024, StatCan) | ~1.5% (2023, Cabinet Office) | ~30–35% of disposable income (urban, survey-based) |
| Gross national saving / GDP | — | — | — | 42.5–43% (2023, World Bank) — highest of any major economy |

Notes: China's low market-cap ratio reflects bank-dominated, state-directed intermediation (PBOC structural lending facilities alone > RMB 7tn, end-2024), not low λ. Japan's low household *flow* saving (down from 11.8% in the 2020 pandemic spike; aging-driven decline per Horioka) coexists with a wealth *stock* still concentrated in currency and deposits — the paper's "stock-λ."

### A.3 Institutional and episode facts used in Section 5

- **US**: paradigm LME (Hall & Soskice); employment-at-will default doctrine; federal-government trust fell from 73% (1958) to 17% (2025, Pew); banking crises 1907, 1930s, 2008 (Bordo–Redish–Rockoff contrast with Canada).
- **Canada**: LME with concentrated, federally regulated branch banking (Big Six, OSFI); **no bank failures and no equity bailouts in 2008–09** (extraordinary liquidity support did occur, incl. ~C$114bn in facilities); WEF world's-soundest-banking ranking 2007–2013; Canada Health Act single-payer; constitutionally entrenched equalization (s.36(2), Constitution Act 1982); household debt-to-income now among OECD's highest (flagged; figure not verified).
- **Japan**: CME, group-coordinated variant. Bubble and aftermath: Nikkei closed at 38,915.87 on 1989-12-29, not regained until February 2024 (34 years); Tokyo commercial land ≈ −80% peak-to-trough; ZIRP from February 1999; QE 2001–06, QQE 2013, negative rates 2016–24. Institutional erosion: non-regular employment 36.8% of employees (2024, Statistics Bureau); cross-held shares <10% of holdings by 2017 (TSE/RIETI); classic main-bank monitoring defunct after early-2000s megabank consolidation. Revival: shuntō settlements 5.24% (2024, first >5% since 1991), ~5.4% (2025). Koo (2008): corporate debt-minimization at zero rates, from flow-of-funds data.
- **China**: state-permeated capitalism (Nölke et al.); five-year plans (14th FYP 2021–25); Three Red Lines announced August 2020 (liability/asset <70%, net gearing <100%, cash/short-term debt ≥1; debt-growth caps 15/10/5/0%); Evergrande breached all three, defaulted offshore December 2021 with >$300bn liabilities, Hong Kong liquidation order 2024-01-29; 30+ developer defaults; 2022 mortgage boycotts on unfinished pre-sold projects; Red Lines quietly retired/softened 2023–25; housing ≈ 70% of household wealth (approximate). Precautionary-saving mechanism: Chamon & Prasad (2010) — urban saving 18% (1995) → ~29% (2009) of disposable income, driven by privatized education/health/housing burdens and pension uncertainty; confirmed by IMF WP/18/277 and the December 2025 IMF working paper on reducing household savings. Hukou reform: State Council five-year action plan, July 2024.

### A.4 Verification notes

Compiled with three parallel research passes (literature; Schelling/Heath corpus; comparative institutions), each instructed to verify bibliographic and quantitative claims against primary or near-primary sources and to flag anything unverifiable. Known open items: the US figure on the OECD trust instrument specifically; Canada's current household debt-to-income ratio; the current state share of Chinese banking assets; Edelman's 2025 China government-trust component. Two textual cautions from the verification pass are honored in the body: Schelling's Chapter 2 is "The Inescapable Mathematics of Musical Chairs" (the phrase "economics of musical chairs" does not occur in the book), and Heath's five cooperative-benefit mechanisms appear in "The Benefits of Cooperation" (2006) as listed in Section 3.2, verified against the paper's text.
