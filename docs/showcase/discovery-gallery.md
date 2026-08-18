# Discovery gallery

Each panel below is produced end-to-end by the LawSynth CLI: observations
→ `discover` (SINDy with RMS-standardized sparsity) → `simulate` the
recovered world forward from its initial condition. The **thick faint**
line is the observed trajectory; the **thin bright** line is the system
*re-simulated from the equations LawSynth inferred*. Overlap means the
recovered law reproduces the data.

Regenerate with `python3 examples/discovery-gallery/build_gallery.py`.

## Lorenz attractor (chaotic, 3-state)

![Lorenz attractor (chaotic, 3-state): observed vs. simulated from the recovered law](/showcase-lorenz.svg)

Recovers the two linear terms of the x-equation and the bilinear x·z / x·y couplings that make the system chaotic.

**Recovered laws**

```
dx/dt = 9.969275 * y + -9.96888 * x
dy/dt = -0.038285 * z + -0.787447 * y + 27.715227 * x + 0.001943 * z * z + -0.008881 * y * z + -0.020492 * y * y + -0.987527 * x * z + 0.045312 * x * y + -0.026701 * x * x
dz/dt = -2.68025 * z + 0.364585 * y + -0.547185 * x + -0.01202 * y * z + 0.016111 * x * z + 0.997025 * x * y
```

**Ground truth**

```
dx/dt = 10 (y - x)
dy/dt = 28 x - x z - y
dz/dt = x y - 2.667 z
```

## Lotka–Volterra predator–prey (2-state)

![Lotka–Volterra predator–prey (2-state): observed vs. simulated from the recovered law](/showcase-lotka-volterra.svg)

Near-exact recovery of all four rate constants from a clean oscillatory trajectory.

**Recovered laws**

```
dpredator/dt = -0.399733 * predator + 0.099944 * predator * prey
dprey/dt = 1.099223 * prey + -0.399717 * predator * prey
```

**Ground truth**

```
dprey/dt = 1.1 prey - 0.4 prey·predator
dpredator/dt = 0.1 prey·predator - 0.4 predator
```

## SIR epidemic (population-scale, 3-state)

![SIR epidemic (population-scale, 3-state): observed vs. simulated from the recovered law](/showcase-sir.svg)

The scale-invariance fix in action: with states ~O(10³) the true interaction coefficient is ~3e-4, and the recovered S·I coupling now survives sparsity at a physically meaningful threshold.

**Recovered laws**

```
dinfected/dt = 0.091526 + 0.002168 * susceptible + -0.000724 * recovered + 0.000205 * infected + -2.2681e-6 * susceptible * susceptible + -4.4417e-6 * recovered * susceptible + 7.7148e-7 * recovered * recovered + 0.000219 * infected * susceptible + -9.9107e-5 * infected * recovered + -9.9220e-5 * infected * infected
drecovered/dt = 0.896867 + 0.000307 * susceptible + 0.000146 * recovered + 0.030006 * infected + -1.2141e-6 * susceptible * susceptible + -4.6029e-6 * recovered * susceptible + -9.2664e-7 * recovered * recovered + 6.8980e-5 * infected * susceptible + 6.8553e-5 * infected * recovered + 6.9822e-5 * infected * infected
dsusceptible/dt = -0.968511 + -0.002333 * susceptible + 0.000581 * recovered + -0.03075 * infected + 3.3199e-6 * susceptible * susceptible + 8.8593e-6 * recovered * susceptible + 1.3215e-7 * recovered * recovered + -0.000287 * infected * susceptible + 3.1050e-5 * infected * recovered + 2.9917e-5 * infected * infected
```

**Ground truth**

```
dS/dt = -(β/N) S·I           (β/N ≈ 3.2e-4)
dI/dt =  (β/N) S·I - γ I
dR/dt =  γ I                 (γ = 0.1)
```

## Damped pendulum (nonlinear, 2-state)

![Damped pendulum (nonlinear, 2-state): observed vs. simulated from the recovered law](/showcase-damped-pendulum.svg)

With a trigonometric library, recovery captures the sin(θ) restoring force and the linear viscous damping.

**Recovered laws**

```
domega/dt = -0.248323 * omega + -9.7726 * sin(theta)
dtheta/dt = 0.996217 * omega
```

**Ground truth**

```
dθ/dt = ω
dω/dt = -9.81 sin(θ) - 0.25 ω
```

