import { codeFence, frontMatter, markdownDocument } from "./content_markdown.js";
import type { DocumentationPageSource } from "./site.js";

/**
 * A catalog of well-known named dynamical systems, each rendered as a single
 * reference page under `/systems/<slug>`. Every entry carries the *textbook*
 * governing equations (verbatim, not fitted approximations), typical parameter
 * values, a short accurate description, and the LawSynth workflow that recovers
 * the same law from a trajectory. The equations are the primary SEO target
 * (queries like "lorenz system equations", "SIR model differential equations").
 *
 * Scientific accuracy is a hard constraint: only systems whose standard form is
 * unambiguous are included. If a system's canonical equations vary across
 * sources it is omitted rather than guessed.
 */

export type SystemCategory =
  | "chaos"
  | "mechanics"
  | "biology"
  | "chemistry"
  | "ecology"
  | "epidemiology"
  | "maps";

export interface SystemEquation {
  /** Left-hand side, e.g. `dx/dt` or `x_{n+1}`. */
  readonly target: string;
  /** Right-hand side in the standard textbook form. */
  readonly expression: string;
}

export interface DynamicalSystem {
  readonly id: string;
  readonly title: string;
  /** Path segment under `/systems/`, e.g. `lorenz`. */
  readonly slug: string;
  readonly category: SystemCategory;
  /** Discrete-time map (true) vs continuous-time flow (false/undefined). */
  readonly discrete?: boolean;
  /** One-line summary used as the meta description; should read as "<name> equations …". */
  readonly summary: string;
  /** Longer, accurate description paragraphs. */
  readonly description: readonly string[];
  /** State column names for the discovery command. */
  readonly stateColumns: readonly string[];
  readonly equations: readonly SystemEquation[];
  /** Auxiliary definitions (e.g. a piecewise nonlinearity), rendered with the equations. */
  readonly auxiliary?: readonly string[];
  /** Typical parameter values as human-readable strings. */
  readonly parameters: readonly string[];
  readonly tags: readonly string[];
  readonly references?: readonly string[];
  readonly order: number;
}

const CATEGORY_LABELS: Readonly<Record<SystemCategory, string>> = Object.freeze({
  chaos: "Chaotic flows",
  mechanics: "Mechanics & oscillators",
  biology: "Biology & neuroscience",
  chemistry: "Chemical kinetics",
  ecology: "Population & ecology",
  epidemiology: "Epidemiology (compartmental)",
  maps: "Discrete maps",
});

/** Ordering of category groups on the index page. */
const CATEGORY_ORDER: readonly SystemCategory[] = Object.freeze([
  "chaos",
  "mechanics",
  "biology",
  "chemistry",
  "ecology",
  "epidemiology",
  "maps",
]);

const RAW_SYSTEMS: readonly Omit<DynamicalSystem, "order">[] = [
  // ---- Chaotic flows -------------------------------------------------------
  {
    id: "lorenz",
    title: "Lorenz system",
    slug: "lorenz",
    category: "chaos",
    summary:
      "Lorenz system equations (σ=10, ρ=28, β=8/3): the three-variable convection model and the textbook example of deterministic chaos.",
    description: [
      "The Lorenz system is a simplified model of atmospheric convection introduced by Edward Lorenz in 1963. For the classic parameters it produces the butterfly-shaped strange attractor and exhibits sensitive dependence on initial conditions.",
    ],
    stateColumns: ["x", "y", "z"],
    equations: [
      { target: "dx/dt", expression: "σ (y − x)" },
      { target: "dy/dt", expression: "x (ρ − z) − y" },
      { target: "dz/dt", expression: "x y − β z" },
    ],
    parameters: ["σ = 10", "ρ = 28", "β = 8/3 ≈ 2.667"],
    tags: ["attractor", "convection", "butterfly-effect"],
    references: ["Lorenz, E. N. (1963). Deterministic Nonperiodic Flow. J. Atmos. Sci."],
  },
  {
    id: "rossler",
    title: "Rössler attractor",
    slug: "rossler",
    category: "chaos",
    summary:
      "Rössler attractor equations (a=b=0.2, c=5.7): a single-scroll chaotic flow with only one quadratic nonlinearity.",
    description: [
      "The Rössler system was designed by Otto Rössler in 1976 as a minimal continuous system exhibiting chaos with a single folded band. Only the z-equation is nonlinear, which makes it a common teaching example alongside Lorenz.",
    ],
    stateColumns: ["x", "y", "z"],
    equations: [
      { target: "dx/dt", expression: "−y − z" },
      { target: "dy/dt", expression: "x + a y" },
      { target: "dz/dt", expression: "b + z (x − c)" },
    ],
    parameters: ["a = 0.2", "b = 0.2", "c = 5.7"],
    tags: ["attractor", "single-scroll"],
    references: ["Rössler, O. E. (1976). An equation for continuous chaos. Phys. Lett. A."],
  },
  {
    id: "chen",
    title: "Chen system",
    slug: "chen",
    category: "chaos",
    summary:
      "Chen system equations (a=35, b=3, c=28): a chaotic attractor dual to Lorenz, introduced via feedback control.",
    description: [
      "The Chen system, introduced by Guanrong Chen and Tetsushi Ueta in 1999, is topologically distinct from the Lorenz attractor although it shares a similar polynomial structure. It is a standard benchmark in chaos synchronization and control.",
    ],
    stateColumns: ["x", "y", "z"],
    equations: [
      { target: "dx/dt", expression: "a (y − x)" },
      { target: "dy/dt", expression: "(c − a) x − x z + c y" },
      { target: "dz/dt", expression: "x y − b z" },
    ],
    parameters: ["a = 35", "b = 3", "c = 28"],
    tags: ["attractor", "control"],
    references: ["Chen, G. & Ueta, T. (1999). Yet another chaotic attractor. Int. J. Bifurcation Chaos."],
  },
  {
    id: "chua",
    title: "Chua's circuit",
    slug: "chua",
    category: "chaos",
    summary:
      "Chua's circuit equations (dimensionless): the double-scroll attractor from a nonlinear electronic circuit with a piecewise-linear resistor.",
    description: [
      "Chua's circuit is the simplest electronic circuit that exhibits chaos, producing the classic double-scroll attractor. The nonlinearity comes from the Chua diode, modelled here by the piecewise-linear function f(x).",
    ],
    stateColumns: ["x", "y", "z"],
    equations: [
      { target: "dx/dt", expression: "α (y − x − f(x))" },
      { target: "dy/dt", expression: "x − y + z" },
      { target: "dz/dt", expression: "−β y" },
    ],
    auxiliary: ["f(x) = m₁ x + ½ (m₀ − m₁) (|x + 1| − |x − 1|)"],
    parameters: ["α = 15.6", "β = 28", "m₀ = −8/7 ≈ −1.143", "m₁ = −5/7 ≈ −0.714"],
    tags: ["double-scroll", "circuit", "piecewise-linear"],
    references: ["Matsumoto, T. (1984). A chaotic attractor from Chua's circuit. IEEE Trans. Circuits Syst."],
  },
  {
    id: "thomas",
    title: "Thomas' cyclically symmetric attractor",
    slug: "thomas",
    category: "chaos",
    summary:
      "Thomas' cyclically symmetric attractor equations (b=0.208186): a labyrinth chaos flow symmetric under cyclic permutation of x, y, z.",
    description: [
      "René Thomas' system is cyclically symmetric: each equation is the same up to a rotation of the variables. As the dissipation parameter b decreases the flow moves from a stable point through limit cycles into 'labyrinth chaos'.",
    ],
    stateColumns: ["x", "y", "z"],
    equations: [
      { target: "dx/dt", expression: "sin(y) − b x" },
      { target: "dy/dt", expression: "sin(z) − b y" },
      { target: "dz/dt", expression: "sin(x) − b z" },
    ],
    parameters: ["b = 0.208186 (chaotic)"],
    tags: ["labyrinth-chaos", "symmetric"],
    references: ["Thomas, R. (1999). Deterministic chaos seen in terms of feedback circuits. Int. J. Bifurcation Chaos."],
  },
  {
    id: "rikitake",
    title: "Rikitake dynamo",
    slug: "rikitake",
    category: "chaos",
    summary:
      "Rikitake two-disk dynamo equations (μ=2, a=5): a model of geomagnetic field reversals with chaotic polarity switching.",
    description: [
      "The Rikitake system models a pair of coupled self-exciting dynamos and reproduces the irregular reversals of the Earth's magnetic field. It is a canonical low-dimensional model of geodynamo behaviour.",
    ],
    stateColumns: ["x", "y", "z"],
    equations: [
      { target: "dx/dt", expression: "−μ x + z y" },
      { target: "dy/dt", expression: "−μ y + (z − a) x" },
      { target: "dz/dt", expression: "1 − x y" },
    ],
    parameters: ["μ = 2", "a = 5"],
    tags: ["dynamo", "geomagnetic", "reversals"],
    references: ["Rikitake, T. (1958). Oscillations of a system of disk dynamos. Proc. Camb. Phil. Soc."],
  },
  {
    id: "halvorsen",
    title: "Halvorsen attractor",
    slug: "halvorsen",
    category: "chaos",
    summary:
      "Halvorsen attractor equations (a=1.4): a cyclically symmetric chaotic flow with quadratic cross-terms.",
    description: [
      "The Halvorsen attractor is a cyclically symmetric system whose three equations differ only by a cyclic permutation of the variables. It produces a distinctive multi-armed spiral attractor.",
    ],
    stateColumns: ["x", "y", "z"],
    equations: [
      { target: "dx/dt", expression: "−a x − 4 y − 4 z − y²" },
      { target: "dy/dt", expression: "−a y − 4 z − 4 x − z²" },
      { target: "dz/dt", expression: "−a z − 4 x − 4 y − x²" },
    ],
    parameters: ["a = 1.4"],
    tags: ["attractor", "symmetric"],
  },
  {
    id: "lorenz-84",
    title: "Lorenz–84 atmospheric model",
    slug: "lorenz-84",
    category: "chaos",
    summary:
      "Lorenz-84 equations (a=0.25, b=4, F=8, G=1): a low-order model of the global atmospheric circulation.",
    description: [
      "Lorenz's 1984 model is a three-variable caricature of the large-scale mid-latitude atmospheric circulation, where x is a westerly wind current and y, z are a cosine/sine pair of eddies. It is widely used to study atmospheric predictability.",
    ],
    stateColumns: ["x", "y", "z"],
    equations: [
      { target: "dx/dt", expression: "−a x − y² − z² + a F" },
      { target: "dy/dt", expression: "−y + x y − b x z + G" },
      { target: "dz/dt", expression: "−z + b x y + x z" },
    ],
    parameters: ["a = 0.25", "b = 4", "F = 8", "G = 1"],
    tags: ["atmosphere", "circulation", "predictability"],
    references: ["Lorenz, E. N. (1984). Irregularity: a fundamental property of the atmosphere. Tellus A."],
  },
  {
    id: "shimizu-morioka",
    title: "Shimizu–Morioka system",
    slug: "shimizu-morioka",
    category: "chaos",
    summary:
      "Shimizu–Morioka equations (λ=0.75, α=0.45): a symmetric system exhibiting a Lorenz-like attractor.",
    description: [
      "The Shimizu–Morioka system arises as a normal-form reduction near the onset of Lorenz-like chaos and is invariant under the symmetry (x, y, z) → (−x, −y, z).",
    ],
    stateColumns: ["x", "y", "z"],
    equations: [
      { target: "dx/dt", expression: "y" },
      { target: "dy/dt", expression: "x − λ y − x z" },
      { target: "dz/dt", expression: "−α z + x²" },
    ],
    parameters: ["λ = 0.75", "α = 0.45"],
    tags: ["attractor", "normal-form"],
    references: ["Shimizu, T. & Morioka, N. (1980). On the bifurcation of a symmetric limit cycle. Phys. Lett. A."],
  },
  {
    id: "genesio-tesi",
    title: "Genesio–Tesi system",
    slug: "genesio-tesi",
    category: "chaos",
    summary:
      "Genesio–Tesi jerk equations (a=1.2, b=2.92, c=6): a scalar third-order chaotic system with a single quadratic term.",
    description: [
      "The Genesio–Tesi system is a 'jerk' system — a single third-order ODE written as three first-order equations — with one quadratic nonlinearity. It is a standard testbed for chaos prediction and control methods.",
    ],
    stateColumns: ["x", "y", "z"],
    equations: [
      { target: "dx/dt", expression: "y" },
      { target: "dy/dt", expression: "z" },
      { target: "dz/dt", expression: "−c x − b y − a z + x²" },
    ],
    parameters: ["a = 1.2", "b = 2.92", "c = 6"],
    tags: ["jerk", "control"],
    references: ["Genesio, R. & Tesi, A. (1992). Harmonic balance methods for chaotic systems. Automatica."],
  },
  {
    id: "sprott-a",
    title: "Sprott A system",
    slug: "sprott-a",
    category: "chaos",
    summary:
      "Sprott A equations: one of the algebraically simplest chaotic flows (conservative, Nosé–Hoover form).",
    description: [
      "Sprott A is one of the 19 minimally-complex chaotic systems catalogued by J. C. Sprott in 1994 through an exhaustive computer search. It is conservative and equivalent to the Nosé–Hoover oscillator.",
    ],
    stateColumns: ["x", "y", "z"],
    equations: [
      { target: "dx/dt", expression: "y" },
      { target: "dy/dt", expression: "−x + y z" },
      { target: "dz/dt", expression: "1 − y²" },
    ],
    parameters: ["no free parameters"],
    tags: ["sprott", "minimal", "conservative"],
    references: ["Sprott, J. C. (1994). Some simple chaotic flows. Phys. Rev. E 50, R647."],
  },
  {
    id: "sprott-b",
    title: "Sprott B system",
    slug: "sprott-b",
    category: "chaos",
    summary: "Sprott B equations: a minimal dissipative chaotic flow with two quadratic terms.",
    description: [
      "Sprott B is another entry from Sprott's 1994 catalog of algebraically simple chaotic flows, using only two quadratic nonlinearities and no adjustable parameters.",
    ],
    stateColumns: ["x", "y", "z"],
    equations: [
      { target: "dx/dt", expression: "y z" },
      { target: "dy/dt", expression: "x − y" },
      { target: "dz/dt", expression: "1 − x y" },
    ],
    parameters: ["no free parameters"],
    tags: ["sprott", "minimal"],
    references: ["Sprott, J. C. (1994). Some simple chaotic flows. Phys. Rev. E 50, R647."],
  },
  {
    id: "sprott-c",
    title: "Sprott C system",
    slug: "sprott-c",
    category: "chaos",
    summary: "Sprott C equations: a minimal chaotic flow closely related to Sprott B.",
    description: [
      "Sprott C differs from Sprott B only in its third equation, replacing the x·y term with x², and is one of Sprott's simple chaotic flows.",
    ],
    stateColumns: ["x", "y", "z"],
    equations: [
      { target: "dx/dt", expression: "y z" },
      { target: "dy/dt", expression: "x − y" },
      { target: "dz/dt", expression: "1 − x²" },
    ],
    parameters: ["no free parameters"],
    tags: ["sprott", "minimal"],
    references: ["Sprott, J. C. (1994). Some simple chaotic flows. Phys. Rev. E 50, R647."],
  },
  {
    id: "sprott-d",
    title: "Sprott D system",
    slug: "sprott-d",
    category: "chaos",
    summary: "Sprott D equations: a minimal chaotic flow with a single cubic-order (3y²) term.",
    description: [
      "Sprott D is a dissipative member of Sprott's 1994 family, notable for the 3y² term in its third equation.",
    ],
    stateColumns: ["x", "y", "z"],
    equations: [
      { target: "dx/dt", expression: "−y" },
      { target: "dy/dt", expression: "x + z" },
      { target: "dz/dt", expression: "x z + 3 y²" },
    ],
    parameters: ["no free parameters"],
    tags: ["sprott", "minimal"],
    references: ["Sprott, J. C. (1994). Some simple chaotic flows. Phys. Rev. E 50, R647."],
  },
  {
    id: "sprott-e",
    title: "Sprott E system",
    slug: "sprott-e",
    category: "chaos",
    summary: "Sprott E equations: a minimal chaotic flow with quadratic and linear terms.",
    description: [
      "Sprott E is one of Sprott's algebraically simple chaotic flows, combining a y·z coupling with an x² term.",
    ],
    stateColumns: ["x", "y", "z"],
    equations: [
      { target: "dx/dt", expression: "y z" },
      { target: "dy/dt", expression: "x² − y" },
      { target: "dz/dt", expression: "1 − 4 x" },
    ],
    parameters: ["no free parameters"],
    tags: ["sprott", "minimal"],
    references: ["Sprott, J. C. (1994). Some simple chaotic flows. Phys. Rev. E 50, R647."],
  },
  {
    id: "sprott-f",
    title: "Sprott F system",
    slug: "sprott-f",
    category: "chaos",
    summary: "Sprott F equations (a=0.5): a minimal chaotic flow with one quadratic nonlinearity.",
    description: [
      "Sprott F uses a single adjustable coefficient (a = 0.5) and one quadratic term, and belongs to Sprott's 1994 catalog of simple chaotic flows.",
    ],
    stateColumns: ["x", "y", "z"],
    equations: [
      { target: "dx/dt", expression: "y + z" },
      { target: "dy/dt", expression: "−x + a y" },
      { target: "dz/dt", expression: "x² − z" },
    ],
    parameters: ["a = 0.5"],
    tags: ["sprott", "minimal"],
    references: ["Sprott, J. C. (1994). Some simple chaotic flows. Phys. Rev. E 50, R647."],
  },
  {
    id: "sprott-g",
    title: "Sprott G system",
    slug: "sprott-g",
    category: "chaos",
    summary: "Sprott G equations (a=0.4): a minimal chaotic flow with a single x·z nonlinearity.",
    description: [
      "Sprott G uses one adjustable coefficient (a = 0.4) and a single bilinear term, and is one of Sprott's simple chaotic flows.",
    ],
    stateColumns: ["x", "y", "z"],
    equations: [
      { target: "dx/dt", expression: "a x + z" },
      { target: "dy/dt", expression: "x z − y" },
      { target: "dz/dt", expression: "−x + y" },
    ],
    parameters: ["a = 0.4"],
    tags: ["sprott", "minimal"],
    references: ["Sprott, J. C. (1994). Some simple chaotic flows. Phys. Rev. E 50, R647."],
  },

  // ---- Mechanics & oscillators --------------------------------------------
  {
    id: "duffing",
    title: "Duffing oscillator",
    slug: "duffing",
    category: "mechanics",
    summary:
      "Duffing oscillator equations (δ=0.3, α=−1, β=1, γ=0.5, ω=1.2): a forced, damped oscillator with a cubic stiffness term.",
    description: [
      "The Duffing equation models a damped oscillator with a nonlinear (cubic) restoring force and periodic forcing. With a double-well potential (α < 0, β > 0) and suitable forcing it becomes chaotic. Written as a first-order system in position x and velocity v, the forcing makes it non-autonomous.",
    ],
    stateColumns: ["x", "v"],
    equations: [
      { target: "dx/dt", expression: "v" },
      { target: "dv/dt", expression: "−δ v − α x − β x³ + γ cos(ω t)" },
    ],
    parameters: ["δ = 0.3", "α = −1", "β = 1", "γ = 0.5", "ω = 1.2"],
    tags: ["forced", "cubic", "double-well"],
    references: ["Duffing, G. (1918). Erzwungene Schwingungen bei veränderlicher Eigenfrequenz."],
  },
  {
    id: "van-der-pol",
    title: "Van der Pol oscillator",
    slug: "van-der-pol",
    category: "mechanics",
    summary:
      "Van der Pol oscillator equations (μ=1): a self-sustaining nonlinear oscillator with amplitude-dependent damping and a limit cycle.",
    description: [
      "The Van der Pol oscillator has nonlinear damping that pumps energy in at small amplitude and dissipates it at large amplitude, producing a stable limit cycle. For large μ it exhibits relaxation oscillations.",
    ],
    stateColumns: ["x", "v"],
    equations: [
      { target: "dx/dt", expression: "v" },
      { target: "dv/dt", expression: "μ (1 − x²) v − x" },
    ],
    parameters: ["μ = 1 (limit cycle)", "μ ≫ 1 (relaxation oscillations)"],
    tags: ["limit-cycle", "self-sustaining", "relaxation"],
    references: ["van der Pol, B. (1926). On relaxation-oscillations. Phil. Mag."],
  },
  {
    id: "damped-driven-pendulum",
    title: "Damped driven pendulum",
    slug: "damped-driven-pendulum",
    category: "mechanics",
    summary:
      "Damped driven pendulum equations (b=0.5, A=1.15, Ω=2/3): a periodically forced pendulum that routes to chaos via period doubling.",
    description: [
      "A rigid pendulum with linear damping and sinusoidal drive is the classic low-dimensional route-to-chaos demonstration (Baker & Gollub). The equation is written in dimensionless form with angle θ and angular velocity ω; the forcing makes it non-autonomous.",
    ],
    stateColumns: ["theta", "omega"],
    equations: [
      { target: "dθ/dt", expression: "ω" },
      { target: "dω/dt", expression: "−sin(θ) − b ω + A cos(Ω t)" },
    ],
    parameters: ["b = 0.5", "A = 1.15", "Ω = 2/3"],
    tags: ["forced", "period-doubling", "pendulum"],
    references: ["Baker, G. L. & Gollub, J. P. (1996). Chaotic Dynamics: An Introduction."],
  },
  {
    id: "simple-pendulum",
    title: "Simple pendulum",
    slug: "simple-pendulum",
    category: "mechanics",
    summary:
      "Simple pendulum equations: the undamped nonlinear pendulum dθ/dt = ω, dω/dt = −(g/L) sin θ.",
    description: [
      "The undamped simple pendulum is the archetypal conservative nonlinear oscillator. For small angles sin θ ≈ θ recovers simple harmonic motion; for large angles the full sine term makes the period amplitude-dependent.",
    ],
    stateColumns: ["theta", "omega"],
    equations: [
      { target: "dθ/dt", expression: "ω" },
      { target: "dω/dt", expression: "−(g / L) sin(θ)" },
    ],
    parameters: ["g / L = 1 (nondimensional)"],
    tags: ["conservative", "nonlinear", "pendulum"],
  },
  {
    id: "damped-harmonic-oscillator",
    title: "Damped harmonic oscillator",
    slug: "damped-harmonic-oscillator",
    category: "mechanics",
    summary:
      "Damped harmonic oscillator equations (ω₀=2, ζ=0.15): a mass–spring–damper whose damping ratio ζ sets the physical regime.",
    description: [
      "The linear damped harmonic oscillator is the simplest system where a single recovered parameter, the damping ratio ζ, distinguishes under-, critical-, and over-damped behaviour. Written in first-order form with position x and velocity v.",
    ],
    stateColumns: ["x", "v"],
    equations: [
      { target: "dx/dt", expression: "v" },
      { target: "dv/dt", expression: "−ω₀² x − 2 ζ ω₀ v" },
    ],
    parameters: ["ω₀ = 2", "ζ = 0.15 (under-damped)"],
    tags: ["linear", "damping", "spring"],
  },

  // ---- Biology & neuroscience ---------------------------------------------
  {
    id: "fitzhugh-nagumo",
    title: "FitzHugh–Nagumo model",
    slug: "fitzhugh-nagumo",
    category: "biology",
    summary:
      "FitzHugh–Nagumo equations (a=0.7, b=0.8, ε=0.08, I=0.5): a two-variable model of excitable neuron dynamics and spiking.",
    description: [
      "The FitzHugh–Nagumo model is a reduction of the Hodgkin–Huxley equations to two variables: a fast excitation variable v (membrane potential) and a slow recovery variable w. It captures the essential threshold, spiking, and refractory behaviour of excitable cells.",
    ],
    stateColumns: ["v", "w"],
    equations: [
      { target: "dv/dt", expression: "v − v³/3 − w + I" },
      { target: "dw/dt", expression: "ε (v + a − b w)" },
    ],
    parameters: ["a = 0.7", "b = 0.8", "ε = 0.08", "I = 0.5 (applied current)"],
    tags: ["neuron", "excitable", "spiking"],
    references: ["FitzHugh, R. (1961). Impulses and physiological states in models of nerve membrane. Biophys. J."],
  },

  // ---- Chemical kinetics --------------------------------------------------
  {
    id: "brusselator",
    title: "Brusselator",
    slug: "brusselator",
    category: "chemistry",
    summary:
      "Brusselator equations (A=1, B=3): an autocatalytic reaction model with a limit cycle above the Hopf threshold B > 1 + A².",
    description: [
      "The Brusselator is a theoretical model of an autocatalytic chemical reaction proposed by Prigogine and Lefever. Above the Hopf bifurcation (B > 1 + A²) the concentrations settle onto a stable limit cycle — a prototype of chemical oscillations.",
    ],
    stateColumns: ["x", "y"],
    equations: [
      { target: "dx/dt", expression: "A − (B + 1) x + x² y" },
      { target: "dy/dt", expression: "B x − x² y" },
    ],
    parameters: ["A = 1", "B = 3 (oscillatory, since B > 1 + A²)"],
    tags: ["autocatalytic", "limit-cycle", "oscillation"],
    references: ["Prigogine, I. & Lefever, R. (1968). Symmetry breaking instabilities in dissipative systems. J. Chem. Phys."],
  },
  {
    id: "selkov",
    title: "Sel'kov glycolysis model",
    slug: "selkov",
    category: "chemistry",
    summary:
      "Sel'kov glycolytic oscillator equations (a=0.06, b=0.6): a two-variable model of oscillations in glycolysis.",
    description: [
      "The Sel'kov model is a minimal model of the oscillations observed in glycolysis, with x the concentration of ADP and y the concentration of F6P. It is a standard textbook example (Strogatz) of a chemical limit cycle born in a Hopf bifurcation.",
    ],
    stateColumns: ["x", "y"],
    equations: [
      { target: "dx/dt", expression: "−x + a y + x² y" },
      { target: "dy/dt", expression: "b − a y − x² y" },
    ],
    parameters: ["a = 0.06", "b = 0.6"],
    tags: ["glycolysis", "limit-cycle", "oscillation"],
    references: ["Sel'kov, E. E. (1968). Self-oscillations in glycolysis. Eur. J. Biochem."],
  },

  // ---- Population & ecology -----------------------------------------------
  {
    id: "lotka-volterra",
    title: "Lotka–Volterra predator–prey model",
    slug: "lotka-volterra",
    category: "ecology",
    summary:
      "Lotka–Volterra equations (α=1.1, β=0.4, δ=0.1, γ=0.4): the classic predator–prey model with out-of-phase population cycles.",
    description: [
      "The Lotka–Volterra equations describe two interacting species — prey x and predator y — with mass-action coupling. The populations oscillate out of phase around a neutrally stable center; the model is the origin of predator–prey theory.",
    ],
    stateColumns: ["x", "y"],
    equations: [
      { target: "dx/dt", expression: "α x − β x y" },
      { target: "dy/dt", expression: "δ x y − γ y" },
    ],
    parameters: ["α = 1.1", "β = 0.4", "δ = 0.1", "γ = 0.4"],
    tags: ["predator-prey", "oscillation", "mass-action"],
    references: ["Lotka, A. J. (1925). Elements of Physical Biology; Volterra, V. (1926). Nature."],
  },
  {
    id: "competitive-lotka-volterra",
    title: "Competitive Lotka–Volterra model",
    slug: "competitive-lotka-volterra",
    category: "ecology",
    summary:
      "Competitive Lotka–Volterra equations: two species with logistic self-limitation and interspecific competition coefficients.",
    description: [
      "The competitive Lotka–Volterra model extends logistic growth to two species that compete for shared resources. The interaction coefficients α₁₂ and α₂₁ measure how strongly each species suppresses the other; outcomes range from coexistence to competitive exclusion.",
    ],
    stateColumns: ["N1", "N2"],
    equations: [
      { target: "dN₁/dt", expression: "r₁ N₁ (1 − (N₁ + α₁₂ N₂) / K₁)" },
      { target: "dN₂/dt", expression: "r₂ N₂ (1 − (N₂ + α₂₁ N₁) / K₂)" },
    ],
    parameters: ["r₁ = r₂ = 1", "K₁ = K₂ = 1", "α₁₂ = α₂₁ = 0.5 (coexistence)"],
    tags: ["competition", "logistic", "coexistence"],
  },
  {
    id: "rosenzweig-macarthur",
    title: "Rosenzweig–MacArthur model",
    slug: "rosenzweig-macarthur",
    category: "ecology",
    summary:
      "Rosenzweig–MacArthur equations: predator–prey with logistic prey growth and a Holling type II functional response.",
    description: [
      "The Rosenzweig–MacArthur model adds two realistic ingredients to Lotka–Volterra: logistic (density-limited) prey growth and a saturating Holling type II predation rate. It famously predicts the 'paradox of enrichment', where increasing carrying capacity K destabilises the system.",
    ],
    stateColumns: ["N", "P"],
    equations: [
      { target: "dN/dt", expression: "r N (1 − N / K) − a N P / (1 + a h N)" },
      { target: "dP/dt", expression: "e a N P / (1 + a h N) − m P" },
    ],
    parameters: ["r = 1", "K = 10", "a = 1", "h = 0.5", "e = 0.5", "m = 0.3"],
    tags: ["predator-prey", "holling", "enrichment"],
    references: ["Rosenzweig, M. L. & MacArthur, R. H. (1963). Graphical representation of predator–prey interactions. Am. Nat."],
  },
  {
    id: "logistic-growth",
    title: "Logistic growth model",
    slug: "logistic-growth",
    category: "ecology",
    summary:
      "Logistic growth equation (r=0.5, K=100): dx/dt = r x (1 − x/K), the S-curve of resource-limited population growth.",
    description: [
      "The continuous logistic equation models population growth that saturates at a carrying capacity K. Growth is nearly exponential when x ≪ K and slows to zero as x approaches K, producing the characteristic sigmoid (S-shaped) trajectory.",
    ],
    stateColumns: ["x"],
    equations: [{ target: "dx/dt", expression: "r x (1 − x / K)" }],
    parameters: ["r = 0.5 (growth rate)", "K = 100 (carrying capacity)"],
    tags: ["logistic", "carrying-capacity", "sigmoid"],
    references: ["Verhulst, P. F. (1838). Notice sur la loi que la population suit dans son accroissement."],
  },

  // ---- Epidemiology (compartmental) ---------------------------------------
  {
    id: "sir",
    title: "SIR epidemic model",
    slug: "sir",
    category: "epidemiology",
    summary:
      "SIR model differential equations (β=0.3, γ=0.1, R₀=3): the susceptible–infected–recovered compartmental epidemic model.",
    description: [
      "The SIR model divides a population of size N into susceptible, infected, and recovered compartments with mass-action transmission. The basic reproduction number R₀ = β/γ determines whether an outbreak grows or dies out.",
    ],
    stateColumns: ["S", "I", "R"],
    equations: [
      { target: "dS/dt", expression: "−β S I / N" },
      { target: "dI/dt", expression: "β S I / N − γ I" },
      { target: "dR/dt", expression: "γ I" },
    ],
    parameters: ["β = 0.3 (transmission)", "γ = 0.1 (recovery)", "R₀ = β/γ = 3"],
    tags: ["compartmental", "outbreak", "reproduction-number"],
    references: ["Kermack, W. O. & McKendrick, A. G. (1927). A contribution to the mathematical theory of epidemics. Proc. R. Soc. A."],
  },
  {
    id: "seir",
    title: "SEIR epidemic model",
    slug: "seir",
    category: "epidemiology",
    summary:
      "SEIR model differential equations (β=0.6, σ=0.2, γ=0.1): SIR extended with an exposed (latent) compartment.",
    description: [
      "The SEIR model adds an Exposed compartment for individuals who are infected but not yet infectious, with 1/σ the mean latent period. It is the standard framework for diseases with a meaningful incubation period.",
    ],
    stateColumns: ["S", "E", "I", "R"],
    equations: [
      { target: "dS/dt", expression: "−β S I / N" },
      { target: "dE/dt", expression: "β S I / N − σ E" },
      { target: "dI/dt", expression: "σ E − γ I" },
      { target: "dR/dt", expression: "γ I" },
    ],
    parameters: ["β = 0.6 (transmission)", "σ = 0.2 (1/latent period)", "γ = 0.1 (recovery)"],
    tags: ["compartmental", "latent", "incubation"],
  },
  {
    id: "sis",
    title: "SIS epidemic model",
    slug: "sis",
    category: "epidemiology",
    summary:
      "SIS model differential equations (β=0.4, γ=0.1): susceptible–infected–susceptible dynamics for diseases without lasting immunity.",
    description: [
      "The SIS model applies to infections that confer no lasting immunity, so recovered individuals return directly to the susceptible pool. It admits an endemic steady state when R₀ = β/γ > 1.",
    ],
    stateColumns: ["S", "I"],
    equations: [
      { target: "dS/dt", expression: "−β S I / N + γ I" },
      { target: "dI/dt", expression: "β S I / N − γ I" },
    ],
    parameters: ["β = 0.4 (transmission)", "γ = 0.1 (recovery)", "R₀ = β/γ = 4"],
    tags: ["compartmental", "endemic", "no-immunity"],
  },

  // ---- Discrete maps ------------------------------------------------------
  {
    id: "logistic-map",
    title: "Logistic map",
    slug: "logistic-map",
    category: "maps",
    discrete: true,
    summary:
      "Logistic map equation x_{n+1} = r x_n (1 − x_n): the canonical route to chaos via period-doubling bifurcations.",
    description: [
      "The logistic map is the simplest nonlinear difference equation and the textbook example of the period-doubling route to chaos. As r increases past ≈3.57 the orbit becomes chaotic, with the Feigenbaum constant governing the bifurcation cascade.",
    ],
    stateColumns: ["x"],
    equations: [{ target: "x_{n+1}", expression: "r x_n (1 − x_n)" }],
    parameters: ["r = 3.9 (chaotic)", "0 ≤ r ≤ 4"],
    tags: ["period-doubling", "feigenbaum", "difference-equation"],
    references: ["May, R. M. (1976). Simple mathematical models with very complicated dynamics. Nature."],
  },
  {
    id: "henon-map",
    title: "Hénon map",
    slug: "henon-map",
    category: "maps",
    discrete: true,
    summary:
      "Hénon map equations (a=1.4, b=0.3): a two-dimensional quadratic map with a strange attractor.",
    description: [
      "The Hénon map is a two-dimensional invertible map that produces a self-similar strange attractor. It was introduced as a simplified model of the Poincaré section of the Lorenz system.",
    ],
    stateColumns: ["x", "y"],
    equations: [
      { target: "x_{n+1}", expression: "1 − a x_n² + y_n" },
      { target: "y_{n+1}", expression: "b x_n" },
    ],
    parameters: ["a = 1.4", "b = 0.3"],
    tags: ["strange-attractor", "quadratic", "invertible"],
    references: ["Hénon, M. (1976). A two-dimensional mapping with a strange attractor. Commun. Math. Phys."],
  },
  {
    id: "tinkerbell-map",
    title: "Tinkerbell map",
    slug: "tinkerbell-map",
    category: "maps",
    discrete: true,
    summary:
      "Tinkerbell map equations (a=0.9, b=−0.6013, c=2.0, d=0.5): a two-dimensional quadratic map with a bird-shaped attractor.",
    description: [
      "The Tinkerbell map is a discrete-time dynamical system whose attractor resembles a bird in flight. It has richer nonlinearity than the Hénon map, with quadratic terms in both coordinates.",
    ],
    stateColumns: ["x", "y"],
    equations: [
      { target: "x_{n+1}", expression: "x_n² − y_n² + a x_n + b y_n" },
      { target: "y_{n+1}", expression: "2 x_n y_n + c x_n + d y_n" },
    ],
    parameters: ["a = 0.9", "b = −0.6013", "c = 2.0", "d = 0.5"],
    tags: ["strange-attractor", "quadratic"],
  },
  {
    id: "standard-map",
    title: "Chirikov standard map",
    slug: "standard-map",
    category: "maps",
    discrete: true,
    summary:
      "Chirikov standard map equations (K=0.9716): an area-preserving map and the prototype of Hamiltonian chaos.",
    description: [
      "The Chirikov standard map is an area-preserving (symplectic) map describing a periodically kicked rotor. As the kick strength K increases the last invariant KAM tori break up (near K ≈ 0.9716), giving global chaos — the canonical model of Hamiltonian chaos. Angles are taken modulo 2π.",
    ],
    stateColumns: ["theta", "p"],
    equations: [
      { target: "p_{n+1}", expression: "p_n + K sin(θ_n)" },
      { target: "θ_{n+1}", expression: "θ_n + p_{n+1}   (mod 2π)" },
    ],
    parameters: ["K = 0.9716 (critical)", "K ≳ 1 (global chaos)"],
    tags: ["area-preserving", "hamiltonian", "kicked-rotor"],
    references: ["Chirikov, B. V. (1979). A universal instability of many-dimensional oscillator systems. Phys. Rep."],
  },
  {
    id: "gingerbreadman-map",
    title: "Gingerbreadman map",
    slug: "gingerbreadman-map",
    category: "maps",
    discrete: true,
    summary:
      "Gingerbreadman map equations x_{n+1} = 1 − y_n + |x_n|, y_{n+1} = x_n: a piecewise-linear chaotic map.",
    description: [
      "The Gingerbreadman map is a piecewise-linear, area-preserving chaotic map whose attractor resembles a gingerbread man. Its chaos arises purely from the absolute-value nonlinearity, with no smooth curvature.",
    ],
    stateColumns: ["x", "y"],
    equations: [
      { target: "x_{n+1}", expression: "1 − y_n + |x_n|" },
      { target: "y_{n+1}", expression: "x_n" },
    ],
    parameters: ["no free parameters"],
    tags: ["piecewise-linear", "area-preserving"],
    references: ["Devaney, R. L. (1984). A piecewise linear model for the zones of instability of an area-preserving map. Physica D."],
  },
];

/** The catalog, frozen and assigned a stable navigation order. */
export const SYSTEM_ENTRIES: readonly DynamicalSystem[] = Object.freeze(
  RAW_SYSTEMS.map((system, index) => Object.freeze({ ...system, order: index + 2 })),
);

/** `lawsynth discover` command that recovers a continuous system from a trajectory CSV. */
function discoverCommand(system: DynamicalSystem): string {
  const timeColumn = system.discrete ? "n" : "t";
  return `lawsynth discover ${system.slug}.csv --time ${timeColumn} --state ${system.stateColumns.join(",")} --output ${system.slug}.lsworld`;
}

/** The equation block (LHS = RHS lines) plus any auxiliary definitions. */
function equationLines(system: DynamicalSystem): string {
  const laws = system.equations.map((equation) => `${equation.target} = ${equation.expression}`);
  const auxiliary = system.auxiliary ?? [];
  return [...laws, ...(auxiliary.length > 0 ? ["", ...auxiliary] : [])].join("\n");
}

function recoverSection(system: DynamicalSystem): readonly string[] {
  if (system.discrete) {
    return [
      "## Recover this map from data with LawSynth",
      `The ${system.title} is a discrete-time map, so identify its update rule by regressing the next iterate on a sparse library of terms in the current state — the discrete analogue of the sequentially-thresholded least-squares method LawSynth uses for flows. Store the orbit as a CSV with an iteration index and one column per state variable:`,
      codeFence("bash", `$ ${discoverCommand(system)}`),
      "Then inspect the recovered update rule:",
      codeFence("bash", `$ lawsynth explain ${system.slug}.lsworld`),
    ];
  }
  return [
    "## Recover this system from data with LawSynth",
    `Given a trajectory CSV with a time column and one column per state variable (${system.stateColumns.join(", ")}), LawSynth recovers the governing equations above with a single deterministic command:`,
    codeFence("bash", `$ ${discoverCommand(system)}`),
    "Then explain, forecast, and report on the recovered world:",
    codeFence(
      "bash",
      [
        `$ lawsynth explain ${system.slug}.lsworld`,
        `$ lawsynth forecast ${system.slug}.lsworld --horizon 40 --step 0.05 --output ${system.slug}-forecast.csv`,
        `$ lawsynth report ${system.slug}.lsworld --output ${system.slug}.html`,
      ].join("\n"),
    ),
  ];
}

/** The systems index page plus one reference page per named system, ready for `compileSite`. */
export function systemsPages(): readonly DocumentationPageSource[] {
  const grouped = CATEGORY_ORDER.map((category) => {
    const entries = SYSTEM_ENTRIES.filter((system) => system.category === category);
    if (entries.length === 0) return "";
    return [
      `## ${CATEGORY_LABELS[category]}`,
      ...entries.map((system) => `- **[${system.title}](/systems/${system.slug})** — ${system.summary}`),
    ].join("\n");
  }).filter(Boolean);

  const index: DocumentationPageSource = {
    path: "/systems",
    section: "systems",
    article: true,
    publishedAt: "2026-09-14",
    updatedAt: "2026-09-14",
    source: markdownDocument(
      frontMatter({
        title: "Dynamical systems catalog",
        description:
          "Reference equations, typical parameters, and LawSynth discovery recipes for classic named dynamical systems — Lorenz, Rössler, Van der Pol, SIR, Hénon, and more.",
        order: 1,
        tags: ["systems", "equations", "catalog"],
      }),
      "# Dynamical systems catalog",
      "A reference library of well-known named dynamical systems. Each page gives the exact textbook governing equations, typical parameter values, a short description, and the LawSynth workflow that recovers the same law from time-series data.",
      ...grouped,
    ),
  };

  const pages = SYSTEM_ENTRIES.map((system): DocumentationPageSource => ({
    path: `/systems/${system.slug}`,
    section: "systems",
    article: true,
    publishedAt: "2026-09-14",
    updatedAt: "2026-09-14",
    source: markdownDocument(
      frontMatter({
        title: system.title,
        description: system.summary,
        order: system.order,
        tags: ["systems", system.category, ...system.tags],
      }),
      `# ${system.title}`,
      ...system.description,
      system.discrete ? "## Governing equations (discrete map)" : "## Governing equations",
      "The standard textbook form:",
      codeFence("text", equationLines(system)),
      `**Typical parameters:** ${system.parameters.join(", ")}.`,
      ...recoverSection(system),
      ...(system.references && system.references.length > 0
        ? ["## References", ...system.references.map((reference) => `- ${reference}`)]
        : []),
    ),
  }));

  return Object.freeze([index, ...pages]);
}
