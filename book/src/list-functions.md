# Predefined functions

## Utility

```nbt
fn unit_of<T>(x: T) -> T
fn value_of<T>(x: T) -> Scalar
fn is_nan<T>(x: T) -> Bool
fn is_infinite<T>(x: T) -> Bool
```

## Math

### Basics

```nbt
fn abs<T>(x: T) -> T
fn round<T>(x: T) -> T
fn floor<T>(x: T) -> T
fn ceil<T>(x: T) -> T
fn mod<T>(a: T, b: T) -> T
fn sqrt<D>(x: D^2) -> D
fn sqr<D>(x: D) -> D^2
```

### Exponential and logarithm

```nbt
fn exp(x: Scalar) -> Scalar
fn ln(x: Scalar) -> Scalar
fn log(x: Scalar) -> Scalar
fn log10(x: Scalar) -> Scalar
fn log2(x: Scalar) -> Scalar
```

### Trigonometry

Basic:

```nbt
fn cos(x: Scalar) -> Scalar
fn sin(x: Scalar) -> Scalar
fn tan(x: Scalar) -> Scalar
fn asin(x: Scalar) -> Scalar
fn acos(x: Scalar) -> Scalar
fn atan(x: Scalar) -> Scalar
fn atan2<T>(y: T, x: T) -> Scalar
```

Hyperbolic:

```nbt
fn sinh(x: Scalar) -> Scalar
fn cosh(x: Scalar) -> Scalar
fn tanh(x: Scalar) -> Scalar
fn asinh(x: Scalar) -> Scalar
fn acosh(x: Scalar) -> Scalar
fn atanh(x: Scalar) -> Scalar
```

Extra:

```nbt
fn cot(x: Scalar) -> Scalar
fn acot(x: Scalar) -> Scalar
fn coth(x: Scalar) -> Scalar
fn acoth(x: Scalar) -> Scalar
fn secant(x: Scalar) -> Scalar
fn arcsecant(x: Scalar) -> Scalar
fn cosecant(x: Scalar) -> Scalar
fn csc(x: Scalar) -> Scalar
fn acsc(x: Scalar) -> Scalar
fn sech(x: Scalar) -> Scalar
fn asech(x: Scalar) -> Scalar
fn csch(x: Scalar) -> Scalar
fn acsch(x: Scalar) -> Scalar
```

### Others

```nbt
fn gamma(x: Scalar) -> Scalar
```

### Statistics

```nbt
fn mean<D>(xs: D…) -> D
fn maximum<D>(xs: D…) -> D
fn minimum<D>(xs: D…) -> D
```

### Geometry

```nbt
fn hypot2<T>(x: T, y: T) -> T
fn hypot3<T>(x: T, y: T, z: T) -> T
fn circle_area<L>(radius: L) -> L^2
fn circle_circumference<L>(radius: L) -> L
fn sphere_area<L>(radius: L) -> L^2
fn sphere_volume<L>(radius: L) -> L^3
```

### Random sampling

```nbt
fn random() -> Scalar
fn rand_uniform<T>(a: T, b: T) -> T
fn rand_int<T>(a: T, b: T) -> T
fn rand_normal<T>(μ: T, σ: T) -> T
fn rand_bernoulli(p: Scalar) -> Scalar
fn rand_binomial(n: Scalar, p: Scalar) -> Scalar
fn rand_geometric(p: Scalar) -> Scalar
fn rand_poisson<T>(λ: T) -> T
fn rand_exponential<T>(λ: T) -> 1/T
fn rand_lognormal(μ: Scalar, σ: Scalar) -> Scalar
fn rand_pareto<T>(α: Scalar, min: T) -> T
```

### Algebra (experimental)

```nbt
fn quadratic_equation<A2, B2>(a: A2, b: B2, c: B2²/A2) -> String
```

## Date and time

See [this page](date-and-time.md) for details.

## Physics

### Temperature conversion

```nbt
fn from_celsius(t_celsius: Scalar) -> Temperature
fn celsius(t_kelvin: Temperature) -> Scalar
fn from_fahrenheit(t_fahrenheit: Scalar) -> Temperature
fn fahrenheit(t_kelvin: Temperature) -> Scalar
```

## Chemistry

```nbt
# Get properties of a chemical element by its symbol or name (case-insensitive).
fn element(pattern: String) -> ChemicalElement
```

## Strings

```nbt
fn str_length(s: String) -> Scalar
fn str_slice(s: String, start: Scalar, end: Scalar) -> String
fn str_append(a: String, b: String) -> String
fn str_contains(haystack: String, needle: String) -> Bool
fn str_replace(s: String, pattern: String, replacement: String) -> String
fn str_repeat(a: String, n: Scalar) -> String
fn chr(n: Scalar) -> String
fn hex(n: Scalar) -> String
```
