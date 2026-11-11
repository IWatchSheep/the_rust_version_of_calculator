//! Operators that need MORE THAN ONE ROW (Python `operator_row_math/multirow_specs.py`).
//!
//! Choosing one of these in the Operators list adds the rows it needs, puts the named operator into each row, writes the join symbol of rows 2, 3 ... and,
//! when `=` is pressed, calculates every row and combines the results with the joins:  total = row 1;  total = join(total, row k)  for k = 2, 3 ...
//! `links` make a box copy another box, so the same u / point only has to be typed once.

/// What goes into a box of a row: typed text, or an operator with boxes of its own.
#[derive(Debug)]
pub enum Fill {
    Text(&'static str),
    Node(&'static NodeSpec),
    /// The box reads the answer of an EARLIER row (counting from 0): `=` fills it in from that row's answer; it is never typed.
    Row(usize),
}

/// An operator with some of its boxes pre-filled.
#[derive(Debug)]
pub struct NodeSpec {
    pub op: &'static str,
    pub fill: &'static [(usize, Fill)],
}

#[derive(Debug)]
pub struct RowSpec {
    pub title: &'static str,
    pub node: NodeSpec,
}

/// A link: the box `to` copies the box `from` (each given as (row, path of the box inside the row)).
pub type Link = ((usize, &'static [usize]), (usize, &'static [usize]));

#[derive(Debug)]
pub struct Spec {
    pub title: &'static str,
    pub rows: &'static [RowSpec],
    /// The join symbol between row 1 and 2, between the total and row 3 ...
    pub joins: &'static [&'static str],
    pub links: &'static [Link],
}

/// The multi-row operator laid out in the rows right now.
#[derive(Clone, Copy, Debug)]
pub struct Template {
    pub key: &'static str,
    pub spec: &'static Spec,
}

const PLAIN: &[(usize, Fill)] = &[];

static BRACKETS: Spec = Spec {
    title: "Brackets: work out the inside first, then use the result",
    rows: &[
        RowSpec { title: "inside the brackets  ( □ + □ )", node: NodeSpec { op: "math/arithmetic/addition", fill: PLAIN } },
        RowSpec { title: "outside the brackets  × □", node: NodeSpec { op: "_number", fill: PLAIN } },
    ],
    joins: &["×"],
    links: &[],
};

static STOKES: Spec = Spec {
    title: "Stokes' / Green's theorem (flat case): the line integral round the edge equals the double integral of (∂Q/∂x − ∂P/∂y) over the inside",
    rows: &[
        RowSpec { title: "∮ F · dr round the boundary", node: NodeSpec { op: "math/calculus/line_integral", fill: PLAIN } },
        RowSpec { title: "∬ (∂Q/∂x − ∂P/∂y) dA over the region", node: NodeSpec { op: "math/calculus/double_integral", fill: PLAIN } },
    ],
    joins: &["="],
    links: &[],
};

static SECOND_X: NodeSpec = NodeSpec { op: "math/calculus/second_partial_derivative", fill: &[(2, Fill::Text("0"))] };

static HEAT: Spec = Spec {
    title: "Heat equation  ∂u/∂t = α · ∂²u/∂x²  (u uses x for position and y for time; point = [x, t])",
    rows: &[
        RowSpec { title: "∂u/∂t", node: NodeSpec { op: "math/calculus/partial_derivative", fill: &[(2, Fill::Text("1"))] } },
        RowSpec { title: "α · ∂²u/∂x²", node: NodeSpec { op: "math/arithmetic/multiplication", fill: &[(1, Fill::Node(&SECOND_X))] } },
    ],
    joins: &["="],
    links: &[((1, &[1, 0]), (0, &[0])), ((1, &[1, 1]), (0, &[1]))],
};

static SQUARE: NodeSpec = NodeSpec { op: "math/algebra/square", fill: PLAIN };

static WAVE: Spec = Spec {
    title: "Wave equation  ∂²u/∂t² = c² · ∂²u/∂x²  (u uses x for position and y for time; point = [x, t])",
    rows: &[
        RowSpec { title: "∂²u/∂t²", node: NodeSpec { op: "math/calculus/second_partial_derivative", fill: &[(2, Fill::Text("1"))] } },
        RowSpec {
            title: "c² · ∂²u/∂x²",
            node: NodeSpec { op: "math/arithmetic/multiplication", fill: &[(0, Fill::Node(&SQUARE)), (1, Fill::Node(&SECOND_X))] },
        },
    ],
    joins: &["="],
    links: &[((1, &[1, 0]), (0, &[0])), ((1, &[1, 1]), (0, &[1]))],
};

static LAPLACE: Spec = Spec {
    title: "Laplace's equation  ∂²u/∂x² + ∂²u/∂y² = 0  (point = [x, y])",
    rows: &[
        RowSpec { title: "∂²u/∂x²", node: NodeSpec { op: "math/calculus/second_partial_derivative", fill: &[(2, Fill::Text("0"))] } },
        RowSpec { title: "∂²u/∂y²", node: NodeSpec { op: "math/calculus/second_partial_derivative", fill: &[(2, Fill::Text("1"))] } },
        RowSpec { title: "equals 0", node: NodeSpec { op: "_number", fill: &[(0, Fill::Text("0"))] } },
    ],
    joins: &["+", "="],
    links: &[((1, &[0]), (0, &[0])), ((1, &[1]), (0, &[1]))],
};

static EXTERIOR: Spec = Spec {
    title: "Exterior derivative. Row 1 is the product rule: d of f dx equals df wedged with dx. Row 2 says applying d twice gives the zero form.",
    rows: &[
        RowSpec { title: "Product rule: d(f dx) = df ∧ dx", node: NodeSpec { op: "math/differential_geometry_and_tensors/exterior_derivative", fill: &[] } },
        RowSpec { title: "Nilpotent: d of d is zero", node: NodeSpec { op: "math/differential_geometry_and_tensors/exterior_derivative_nilpotent", fill: &[] } },
    ],
    joins: &["="],
    links: &[],
};

static DIRECT_SUM: Spec = Spec {
    title: "Direct sum of two vector spaces. Row 1 is the sum A ⊕ B. Row 2 says its dimension is the sum of the two dimensions.",
    rows: &[
        RowSpec { title: "The sum: pairs (a, b) with component-wise operations", node: NodeSpec { op: "math/abstract_algebra/direct_sum", fill: &[] } },
        RowSpec { title: "Dimension: dim(A ⊕ B) = dim A + dim B", node: NodeSpec { op: "math/abstract_algebra/direct_sum_dimension", fill: &[] } },
    ],
    joins: &["="],
    links: &[],
};

static NAVIER_STOKES: Spec = Spec {
    title: "Navier–Stokes equations for an incompressible fluid. u = (u, v, w) is the velocity of the fluid, ρ its density, μ its viscosity, p the pressure and f an outside force on each volume of fluid, such as gravity. Row 1 says: density × the fluid's acceleration = the push from pressure differences (−∇p) + the friction between layers of fluid (μ∇²u) + the outside force (f). The acceleration has two parts: ∂u/∂t is how the velocity changes in time at one point, and (u·∇)u is the change from being carried along by the flow. Rows 2, 3 and 4 are Row 1 written out for the x, y and z directions. Row 5, continuity, says ∂u/∂x + ∂v/∂y + ∂w/∂z = 0: the flow neither squeezes nor creates fluid. Each row is one equation, drawn on the same rows and columns as its diagram. Reference only: nothing is calculated.",
    rows: &[
        RowSpec { title: "Vector form: density × acceleration = pressure push + friction + outside force", node: NodeSpec { op: "physics/physics/navier_stokes_equation", fill: &[] } },
        RowSpec { title: "x direction: the same balance for the velocity u along x", node: NodeSpec { op: "physics/physics/navier_stokes_x_momentum", fill: &[] } },
        RowSpec { title: "y direction: the same balance for the velocity v along y", node: NodeSpec { op: "physics/physics/navier_stokes_y_momentum", fill: &[] } },
        RowSpec { title: "z direction: the same balance for the velocity w along z", node: NodeSpec { op: "physics/physics/navier_stokes_z_momentum", fill: &[] } },
        RowSpec { title: "Continuity: the fluid is incompressible, so none is created or lost", node: NodeSpec { op: "physics/physics/navier_stokes_continuity", fill: &[] } },
    ],
    joins: &["=", "=", "=", "="],
    links: &[],
};

static KKT: Spec = Spec {
    title: "KKT conditions (Karush–Kuhn–Tucker): the conditions a best point must meet when the problem has inequality constraints g(x) ≤ 0. Row 1, stationarity: the slope of the objective and the slopes of the constraints cancel. Row 2, primal feasibility: the point is allowed, every constraint is satisfied. Row 3, dual feasibility: each multiplier λ is not negative. Row 4, complementary slackness: a constraint that is slack has multiplier zero. Reference only: nothing is calculated.",
    rows: &[
        RowSpec { title: "Stationarity: the slope of the objective and the slopes of the constraints cancel.", node: NodeSpec { op: "math/optimization_and_numerical_methods/kkt_conditions", fill: &[] } },
        RowSpec { title: "The point is allowed: every constraint is satisfied.", node: NodeSpec { op: "math/optimization_and_numerical_methods/kkt_primal_feasibility", fill: &[] } },
        RowSpec { title: "The multiplier pushes the right way: it is not negative.", node: NodeSpec { op: "math/optimization_and_numerical_methods/kkt_dual_feasibility", fill: &[] } },
        RowSpec { title: "A constraint that is slack has multiplier zero.", node: NodeSpec { op: "math/optimization_and_numerical_methods/kkt_complementary_slackness", fill: &[] } },
    ],
    joins: &["=", "=", "="],
    links: &[],
};

static FIELD_EXTENSION: Spec = Spec {
    title: "Field extension: K is a field that contains F as a subfield. Row 1 says K sits over F. Row 2 says the degree [K:F] is the dimension of K as a vector space over F. Reference only: nothing is calculated.",
    rows: &[
        RowSpec { title: "K contains F.", node: NodeSpec { op: "math/abstract_algebra/field_extension", fill: &[] } },
        RowSpec { title: "The degree is the dimension of K as a vector space over F.", node: NodeSpec { op: "math/abstract_algebra/field_extension_degree", fill: &[] } },
    ],
    joins: &["="],
    links: &[],
};

static HERON: Spec = Spec {
    title: "Heron's formula for the area of a triangle with sides a, b and c. Row 1 finds the semi-perimeter s, half the distance round the triangle. Row 2 reads that s (its four s boxes hold the answer of Row 1) and the same three sides, and gives the area A = √(s(s−a)(s−b)(s−c)). The rows are joined by →: the answer is the last row's answer, the area.",
    rows: &[
        RowSpec { title: "Half the distance around the triangle.", node: NodeSpec { op: "geometry/geometry/heron_semiperimeter", fill: &[] } },
        RowSpec { title: "The area from that semi-perimeter. The s boxes read Row 1; the sides are the ones typed in Row 1.", node: NodeSpec { op: "geometry/geometry/heron_area", fill: &[(0, Fill::Row(0))] } },
    ],
    joins: &["→"],
    links: &[((1, &[1]), (0, &[0])), ((1, &[2]), (0, &[1])), ((1, &[3]), (0, &[2]))],
};

static DOPPLER: Spec = Spec {
    title: "Relativistic Doppler effect. Row 1 turns the speed v into β, the fraction of the speed of light (positive when the source moves away). Row 2 reads that β and gives the frequency f = f₀ √((1−β)/(1+β)) that the observer hears. The rows are joined by →: the answer is the last row's answer, the frequency.",
    rows: &[
        RowSpec { title: "The speed as a fraction of the speed of light.", node: NodeSpec { op: "physics/physics/relativistic_beta", fill: &[] } },
        RowSpec { title: "The frequency the observer hears. The β boxes read Row 1; f₀ is typed here.", node: NodeSpec { op: "physics/physics/relativistic_doppler_frequency", fill: &[(1, Fill::Row(0))] } },
    ],
    joins: &["→"],
    links: &[],
};

static KINETIC_ENERGY: Spec = Spec {
    title: "Relativistic kinetic energy. Row 1 finds the Lorentz factor γ from the speed v (it declines when v reaches the speed of light). Row 2 reads that γ and gives K = (γ − 1) m c², the energy above the rest energy. The rows are joined by →: the answer is the last row's answer, the kinetic energy.",
    rows: &[
        RowSpec { title: "The Lorentz factor γ: how much time and mass stretch at this speed.", node: NodeSpec { op: "physics/physics/lorentz_factor", fill: &[] } },
        RowSpec { title: "The kinetic energy from that γ. The γ box reads Row 1; the rest mass m is typed here.", node: NodeSpec { op: "physics/physics/relativistic_kinetic_energy_from_gamma", fill: &[(0, Fill::Row(0))] } },
    ],
    joins: &["→"],
    links: &[],
};

static MOMENTUM: Spec = Spec {
    title: "Relativistic momentum. Row 1 finds the Lorentz factor γ from the speed v (it declines when v reaches the speed of light). Row 2 reads that γ and gives p = γ m v. The rows are joined by →: the answer is the last row's answer, the momentum.",
    rows: &[
        RowSpec { title: "The Lorentz factor γ: how much time and mass stretch at this speed.", node: NodeSpec { op: "physics/physics/lorentz_factor", fill: &[] } },
        RowSpec { title: "The momentum from that γ. The γ box reads Row 1; m is typed here and v is the speed typed in Row 1.", node: NodeSpec { op: "physics/physics/relativistic_momentum_from_gamma", fill: &[(0, Fill::Row(0))] } },
    ],
    joins: &["→"],
    links: &[((1, &[2]), (0, &[0]))],
};

static LENGTH_CONTRACTION: Spec = Spec {
    title: "Length contraction. Row 1 finds the Lorentz factor γ from the speed v (it declines when v reaches the speed of light). Row 2 reads that γ and gives the moving length L = L₀ / γ: an object of rest length L₀ is shorter along its motion. The rows are joined by →: the answer is the last row's answer, the moving length.",
    rows: &[
        RowSpec { title: "The Lorentz factor γ: how much time and mass stretch at this speed.", node: NodeSpec { op: "physics/physics/lorentz_factor", fill: &[] } },
        RowSpec { title: "The moving length from that γ. The γ box reads Row 1; the rest length L₀ is typed here.", node: NodeSpec { op: "physics/physics/length_contraction_from_gamma", fill: &[(1, Fill::Row(0))] } },
    ],
    joins: &["→"],
    links: &[],
};

static TIME_DILATION: Spec = Spec {
    title: "Time dilation. Row 1 finds the Lorentz factor γ from the speed v (it declines when v reaches the speed of light). Row 2 reads that γ and gives Δt = Δt₀ γ: the time the stay-at-home observer measures for a moving clock's proper time Δt₀. The rows are joined by →: the answer is the last row's answer, the elapsed time.",
    rows: &[
        RowSpec { title: "The Lorentz factor γ: how much time and mass stretch at this speed.", node: NodeSpec { op: "physics/physics/lorentz_factor", fill: &[] } },
        RowSpec { title: "The elapsed time from that γ. The γ box reads Row 1; the proper time Δt₀ is typed here.", node: NodeSpec { op: "physics/physics/time_dilation_from_gamma", fill: &[(1, Fill::Row(0))] } },
    ],
    joins: &["→"],
    links: &[],
};

static STANDARD_DEVIATION: Spec = Spec {
    title: "Standard deviation of a whole population. Row 1 is the variance of the list of numbers you type there: the average squared distance from the mean. Row 2 reads that variance and takes its square root, σ = √(σ²), the usual spread of the numbers in the same units as the numbers. The rows are joined by →: the answer is the last row's answer, the standard deviation.",
    rows: &[
        RowSpec { title: "The variance of the numbers typed here: their average squared distance from the mean.", node: NodeSpec { op: "math/statistics_and_probability/variance", fill: &[] } },
        RowSpec { title: "The standard deviation: the square root of that variance. The variance box reads Row 1.", node: NodeSpec { op: "math/statistics_and_probability/standard_deviation_from_variance", fill: &[(0, Fill::Row(0))] } },
    ],
    joins: &["→"],
    links: &[],
};

static SAMPLE_STANDARD_DEVIATION: Spec = Spec {
    title: "Standard deviation of a sample (dividing by n − 1). Row 1 is the sample variance of the list of numbers you type there; it needs at least two numbers. Row 2 reads that sample variance and takes its square root, s = √(s²). The rows are joined by →: the answer is the last row's answer, the sample standard deviation.",
    rows: &[
        RowSpec { title: "The sample variance of the numbers typed here (it needs at least two numbers).", node: NodeSpec { op: "math/statistics_and_probability/sample_variance", fill: &[] } },
        RowSpec { title: "The sample standard deviation: the square root of that variance. The sample variance box reads Row 1.", node: NodeSpec { op: "math/statistics_and_probability/sample_standard_deviation_from_variance", fill: &[(0, Fill::Row(0))] } },
    ],
    joins: &["→"],
    links: &[],
};

static GRAVITATIONAL_TIME_DILATION: Spec = Spec {
    title: "Gravitational time dilation. Row 1 finds the horizon radius rₛ = 2GM/c² of the mass M you type there (G and c are the usual constants). Row 2 reads that radius and gives the time Δt₀ on a clock at distance r from the mass, Δt₀ = Δt √(1 − rₛ/r), for the far-away time Δt; r must be outside the horizon. The rows are joined by →: the answer is the last row's answer, the clock time.",
    rows: &[
        RowSpec { title: "The horizon radius rₛ = 2GM/c² of the mass typed here.", node: NodeSpec { op: "physics/physics/schwarzschild_radius", fill: &[] } },
        RowSpec { title: "The clock time from that radius. The rₛ box reads Row 1; the far-away time Δt and the distance r are typed here; r must be outside the horizon.", node: NodeSpec { op: "physics/physics/gravitational_time_dilation_from_radius", fill: &[(1, Fill::Row(0))] } },
    ],
    joins: &["→"],
    links: &[],
};

static RANGE: Spec = Spec {
    title: "Range of a list of numbers: the largest minus the smallest. Row 1 finds the largest number in the list you type there. Row 2 finds the smallest number in the same list (it is copied from Row 1). Row 3 reads both answers and subtracts: range = max − min. The rows are joined by →: the answer is the last row's answer, the range.",
    rows: &[
        RowSpec { title: "The largest number in the list typed here.", node: NodeSpec { op: "math/statistics_and_probability/list_maximum", fill: &[] } },
        RowSpec { title: "The smallest number in the same list: copied from Row 1.", node: NodeSpec { op: "math/statistics_and_probability/list_minimum", fill: &[] } },
        RowSpec { title: "Largest minus smallest. The first box reads Row 1, the second box reads Row 2.", node: NodeSpec { op: "math/statistics_and_probability/range_from_ends", fill: &[(0, Fill::Row(0)), (1, Fill::Row(1))] } },
    ],
    joins: &["→", "→"],
    links: &[((1, &[0]), (0, &[0]))],
};

static INTERQUARTILE_RANGE: Spec = Spec {
    title: "Interquartile range of a list of numbers: how wide the middle half of the numbers is. Row 1 finds the third quartile Q3 (the 75th percentile) of the list you type there. Row 2 finds the first quartile Q1 (the 25th percentile) of the same list (copied from Row 1). Row 3 reads both answers and subtracts: IQR = Q3 − Q1. The rows are joined by →: the answer is the last row's answer, the interquartile range.",
    rows: &[
        RowSpec { title: "The upper quartile Q3, the 75th percentile of the list typed here.", node: NodeSpec { op: "math/statistics_and_probability/third_quartile", fill: &[] } },
        RowSpec { title: "The lower quartile Q1, the 25th percentile of the same list: copied from Row 1.", node: NodeSpec { op: "math/statistics_and_probability/first_quartile", fill: &[] } },
        RowSpec { title: "Q3 minus Q1. The first box reads Row 1, the second box reads Row 2.", node: NodeSpec { op: "math/statistics_and_probability/interquartile_range_from_quartiles", fill: &[(0, Fill::Row(0)), (1, Fill::Row(1))] } },
    ],
    joins: &["→", "→"],
    links: &[((1, &[0]), (0, &[0]))],
};

static VARIANCE_OF_RANDOM_VARIABLE: Spec = Spec {
    title: "Variance of a random variable: how spread out its values are, Var = E[X²] − (E[X])². Row 1 is the expected value E[X] of the values and probabilities you type there. Row 2 is the expected square E[X²]: the same values and probabilities (copied from Row 1), each value squared. Row 3 reads E[X²] from Row 2 and E[X] from Row 1 and gives the variance. The rows are joined by →: the answer is the last row's answer, the variance.",
    rows: &[
        RowSpec { title: "The expected value E[X] of the values and probabilities typed here.", node: NodeSpec { op: "math/statistics_and_probability/expected_value", fill: &[] } },
        RowSpec { title: "The expected square E[X²]: the same values and probabilities, copied from Row 1, each value squared.", node: NodeSpec { op: "math/statistics_and_probability/expected_square", fill: &[] } },
        RowSpec { title: "The variance E[X²] − (E[X])². The first box reads Row 2, the second box reads Row 1.", node: NodeSpec { op: "math/statistics_and_probability/variance_from_expectations", fill: &[(0, Fill::Row(1)), (1, Fill::Row(0))] } },
    ],
    joins: &["→", "→"],
    links: &[((1, &[0]), (0, &[0])), ((1, &[1]), (0, &[1]))],
};

static KALMAN: Spec = Spec {
    title: "Kalman filter, one step in one dimension. Row 1 predicts the state: where the filter expects to be after coasting at speed u for time t. Row 2 predicts the variance: how uncertain that expectation is, the old variance p plus the process noise q. Row 3 reads both predictions and the measurement z with its noise r, and gives the updated state x⁺ = x + k (z − x) with the gain k = p / (p + r). The rows are joined by →: the answer is the last row's answer, the updated state. The updated variance is its own operator, because → keeps only one number.",
    rows: &[
        RowSpec { title: "Where the filter expects to be.", node: NodeSpec { op: "physics/physics/kalman_predict_state", fill: &[] } },
        RowSpec { title: "How uncertain that expectation is.", node: NodeSpec { op: "physics/physics/kalman_predict_variance", fill: &[] } },
        RowSpec { title: "The estimate after the measurement. The predicted state reads Row 1 and the predicted variance reads Row 2; the measurement and its noise are typed here.", node: NodeSpec { op: "physics/physics/kalman_update_state", fill: &[(0, Fill::Row(0)), (1, Fill::Row(1))] } },
    ],
    joins: &["→", "→"],
    links: &[],
};

static PID: Spec = Spec {
    title: "PID controller, one step: the control output from the error. Row 1 is the proportional push P = Kp e from the current error. Row 2 is the integral push Ki (I + e t) from the accumulated error; its error box copies Row 1's error. Row 3 is the derivative push Kd (e − e₀) / t from how fast the error is changing; its error box copies Row 1's error and its time step copies Row 2's. Row 4 adds the three pushes: P from Row 1, I from Row 2, D from Row 3. The rows are joined by →: the answer is the last row's answer, the control output. The robot keeps the next integral with its own operator, pid_integral_state.",
    rows: &[
        RowSpec { title: "The push from the current error.", node: NodeSpec { op: "physics/physics/pid_proportional", fill: &[] } },
        RowSpec { title: "The push from the accumulated error. The error box copies Row 1.", node: NodeSpec { op: "physics/physics/pid_integral_term", fill: &[] } },
        RowSpec { title: "The push from how fast the error is changing. The error box copies Row 1 and the time step copies Row 2.", node: NodeSpec { op: "physics/physics/pid_derivative", fill: &[] } },
        RowSpec { title: "The control output. P reads Row 1, I reads Row 2 and D reads Row 3.", node: NodeSpec { op: "physics/physics/pid_sum", fill: &[(0, Fill::Row(0)), (1, Fill::Row(1)), (2, Fill::Row(2))] } },
    ],
    joins: &["→", "→", "→"],
    links: &[((1, &[2]), (0, &[1])), ((2, &[1]), (0, &[1])), ((2, &[3]), (1, &[3]))],
};

static GPS_CLOCK: Spec = Spec {
    title: "GPS satellite clock correction. Row 1 is the Lorentz factor γ at the satellite's speed (about 3874 m/s). Row 2 reads that γ and gives the special-relativity part: a moving clock runs slow, about −7 microseconds per day. Row 3 gives the gravitational part from the two radii typed there: a clock higher in the Earth's gravity runs fast, about +45 microseconds per day. Row 4 adds the two parts: the kinematic part from Row 2 and the gravitational part from Row 3, about +38 microseconds per day. The rows are joined by →: the answer is the last row's answer, the correction to add.",
    rows: &[
        RowSpec { title: "The Lorentz factor at the satellite's speed.", node: NodeSpec { op: "physics/physics/lorentz_factor", fill: &[] } },
        RowSpec { title: "Special relativity, about −7 microseconds per day. The γ box reads Row 1.", node: NodeSpec { op: "physics/physics/gps_kinematic_clock", fill: &[(0, Fill::Row(0))] } },
        RowSpec { title: "Gravity, about +45 microseconds per day. The two radii are typed here.", node: NodeSpec { op: "physics/physics/gps_gravitational_clock", fill: &[] } },
        RowSpec { title: "The correction to add, about +38 microseconds per day. The kinematic part reads Row 2 and the gravitational part reads Row 3.", node: NodeSpec { op: "physics/physics/gps_clock_correction", fill: &[(0, Fill::Row(1)), (1, Fill::Row(2))] } },
    ],
    joins: &["→", "→", "→"],
    links: &[],
};

/// The multi-row operators, by operator key.
pub fn get(key: &str) -> Option<&'static Spec> {
    Some(match key {
        "math/differential_geometry_and_tensors/stokes_theorem" => &STOKES,
        "math/transforms_and_differential_equations/heat_equation" => &HEAT,
        "math/transforms_and_differential_equations/wave_equation" => &WAVE,
        "math/transforms_and_differential_equations/laplace_s_equation" => &LAPLACE,
        "math/differential_geometry_and_tensors/exterior_derivative" => &EXTERIOR,
        "math/abstract_algebra/direct_sum" => &DIRECT_SUM,
        "physics/physics/navier_stokes_equation" => &NAVIER_STOKES,
        "math/statistics_and_probability/range" => &RANGE,
        "math/statistics_and_probability/interquartile_range" => &INTERQUARTILE_RANGE,
        "math/statistics_and_probability/variance_of_random_variable" => &VARIANCE_OF_RANDOM_VARIABLE,
        "math/statistics_and_probability/standard_deviation" => &STANDARD_DEVIATION,
        "math/statistics_and_probability/sample_standard_deviation" => &SAMPLE_STANDARD_DEVIATION,
        "physics/physics/gravitational_time_dilation" => &GRAVITATIONAL_TIME_DILATION,
        "physics/physics/kalman_update_state" => &KALMAN,
        "physics/physics/pid_sum" => &PID,
        "physics/physics/gps_clock_correction" => &GPS_CLOCK,
        "physics/physics/relativistic_kinetic_energy" => &KINETIC_ENERGY,
        "physics/physics/relativistic_momentum" => &MOMENTUM,
        "physics/physics/length_contraction" => &LENGTH_CONTRACTION,
        "physics/physics/time_dilation" => &TIME_DILATION,
        "geometry/geometry/heron_s_formula" => &HERON,
        "physics/physics/relativistic_doppler" => &DOPPLER,
        "math/optimization_and_numerical_methods/kkt_conditions" => &KKT,
        "math/abstract_algebra/field_extension" => &FIELD_EXTENSION,
        "math/constants_and_relations/open_bracket"
        | "math/constants_and_relations/close_bracket"
        | "math/constants_and_relations/square_brackets"
        | "math/constants_and_relations/curly_braces" => &BRACKETS,
        _ => return None,
    })
}
