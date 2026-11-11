//! Physics operators ported from `operators_one_operator_per_file/physics/physics/*`.
//! One function per operator, named after the folder.  The arguments arrive in this order: the row boxes
//! first, then the optional constants (g, c, h ...) in the order of the Python signature, already filled
//! with their defaults (see `extra` in `generated.rs`).  A result that is not a finite number is turned
//! into an error by `Operator::run`, which is how Python's ZeroDivisionError / math domain errors show up.

use crate::value::Value;
use std::f64::consts::PI;

type R = Result<Value, String>;

fn n(x: f64) -> R {
    Ok(Value::Number(x))
}

/// a / b, but an error (like Python's ZeroDivisionError) when b is 0.
fn d(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("division by zero".into())
    } else {
        Ok(a / b)
    }
}

fn need(cond: bool, msg: &str) -> Result<(), String> {
    if cond {
        Ok(())
    } else {
        Err(msg.into())
    }
}

pub fn bernoulli_equation(a: &[f64]) -> R {
    // P1, v1, h1, v2, h2, rho, g
    n(a[0] + 0.5 * a[5] * (a[1].powi(2) - a[3].powi(2)) + a[5] * a[6] * (a[2] - a[4]))
}
pub fn bohr_energy_level(a: &[f64]) -> R {
    // n, Z
    n(-13.605693122994 * a[1].powi(2) / a[0].powi(2))
}
pub fn brewster_angle(a: &[f64]) -> R {
    n(d(a[1], a[0])?.atan())
}
pub fn buoyant_force(a: &[f64]) -> R {
    // rho, V, g
    n(a[0] * a[2] * a[1])
}
pub fn capacitance(a: &[f64]) -> R {
    n(a[0] * a[1])
}
pub fn centripetal_acceleration(a: &[f64]) -> R {
    n(a[0].powi(2) / a[1])
}
pub fn compton_shift(a: &[f64]) -> R {
    // theta, m, h, c
    n(a[2] / (a[1] * a[3]) * (1.0 - a[0].cos()))
}
pub fn continuity_equation(a: &[f64]) -> R {
    n(a[0] * a[1] / a[2])
}
pub fn coulomb_s_law(a: &[f64]) -> R {
    // q1, q2, r, k
    n(a[3] * a[0] * a[1] / a[2].powi(2))
}
pub fn critical_angle(a: &[f64]) -> R {
    need(a[0] > a[1], "total internal reflection needs n1 > n2")?;
    n((a[1] / a[0]).asin())
}
pub fn cyclotron_radius(a: &[f64]) -> R {
    n(a[0] * a[1] / (a[2].abs() * a[3]))
}
pub fn de_broglie_wavelength(a: &[f64]) -> R {
    // m, v, h
    n(a[2] / (a[0] * a[1]))
}
pub fn density(a: &[f64]) -> R {
    n(a[0] / a[1])
}
pub fn diffraction_grating(a: &[f64]) -> R {
    let s = a[0] * a[1] / a[2];
    need(s.abs() <= 1.0, "this order does not exist")?;
    n(s.asin())
}
pub fn displacement(a: &[f64]) -> R {
    n(a[0] * a[2] + 0.5 * a[1] * a[2].powi(2))
}
pub fn double_slit_fringe_spacing(a: &[f64]) -> R {
    n(a[0] * a[1] / a[2])
}
pub fn electric_power(a: &[f64]) -> R {
    n(a[0] * a[1])
}
pub fn energy_momentum_relation(a: &[f64]) -> R {
    // m, p, c
    n(((a[1] * a[2]).powi(2) + (a[0] * a[2].powi(2)).powi(2)).sqrt())
}
pub fn faradays_law(a: &[f64]) -> R {
    n(-a[0] * a[1] / a[2])
}
pub fn final_velocity(a: &[f64]) -> R {
    n(a[0] + a[1] * a[2])
}
pub fn frequency(a: &[f64]) -> R {
    n(1.0 / a[0])
}
pub fn gravitation(a: &[f64]) -> R {
    // m1, m2, r, G
    n(a[3] * a[0] * a[1] / a[2].powi(2))
}
pub fn gravitational_time_dilation(a: &[f64]) -> R {
    // dt, M, r, G, c
    let (dt, m, r, g, c) = (a[0], a[1], a[2], a[3], a[4]);
    need(r > 2.0 * g * m / c.powi(2), "r must be outside the Schwarzschild radius")?;
    n(dt * (1.0 - 2.0 * g * m / (r * c.powi(2))).sqrt())
}
pub fn heat(a: &[f64]) -> R {
    n(a[0] * a[1] * a[2])
}
pub fn heisenberg_uncertainty(a: &[f64]) -> R {
    // delta_x, hbar
    n(a[1] / (2.0 * a[0]))
}
pub fn hooke_s_law(a: &[f64]) -> R {
    n(-a[0] * a[1])
}
pub fn hydraulic_press(a: &[f64]) -> R {
    n(a[0] * a[2] / a[1])
}
pub fn hydrostatic_pressure(a: &[f64]) -> R {
    // rho, h, g
    n(a[0] * a[2] * a[1])
}
pub fn ideal_gas_law(a: &[f64]) -> R {
    // n, T, V, R
    n(a[0] * a[3] * a[1] / a[2])
}
pub fn impulse(a: &[f64]) -> R {
    n(a[0] * a[1])
}
pub fn inductor_energy(a: &[f64]) -> R {
    n(0.5 * a[0] * a[1].powi(2))
}
pub fn length_contraction(a: &[f64]) -> R {
    // L0, v, c
    need(a[1].abs() < a[2], "speed must be less than the speed of light")?;
    n(a[0] * (1.0 - a[1].powi(2) / a[2].powi(2)).sqrt())
}
pub fn lens_magnification(a: &[f64]) -> R {
    n(-a[0] / a[1])
}
pub fn lensmakers_equation(a: &[f64]) -> R {
    n(1.0 / ((a[0] - 1.0) * (d(1.0, a[1])? - d(1.0, a[2])?)))
}
pub fn magnetic_field_solenoid(a: &[f64]) -> R {
    // n, I, mu0
    n(a[2] * a[0] * a[1])
}
pub fn magnetic_field_straight_wire(a: &[f64]) -> R {
    // I, r, mu0
    n(a[2] * a[0] / (2.0 * PI * a[1]))
}
pub fn magnetic_flux(a: &[f64]) -> R {
    n(a[0] * a[1] * a[2].cos())
}
pub fn magnetic_force_on_charge(a: &[f64]) -> R {
    n(a[0] * a[1] * a[2] * a[3].sin())
}
pub fn magnetic_force_on_wire(a: &[f64]) -> R {
    n(a[0] * a[1] * a[2] * a[3].sin())
}
pub fn malus_law(a: &[f64]) -> R {
    n(a[0] * a[1].cos().powi(2))
}
pub fn mass_energy_equivalence(a: &[f64]) -> R {
    // m, c
    n(a[0] * a[1].powi(2))
}
pub fn momentum(a: &[f64]) -> R {
    n(a[0] * a[1])
}
pub fn motional_emf(a: &[f64]) -> R {
    n(a[0] * a[1] * a[2])
}
pub fn particle_in_a_box(a: &[f64]) -> R {
    // n, m, L, h
    n(a[0].powi(2) * a[3].powi(2) / (8.0 * a[1] * a[2].powi(2)))
}
pub fn pendulum_period(a: &[f64]) -> R {
    // L, g
    n(2.0 * PI * (a[0] / a[1]).sqrt())
}
pub fn photoelectric_effect(a: &[f64]) -> R {
    // f, work_function, h
    n((a[2] * a[0] - a[1]).max(0.0))
}
pub fn photon_energy(a: &[f64]) -> R {
    // f, h
    n(a[1] * a[0])
}
pub fn photon_momentum(a: &[f64]) -> R {
    // wavelength, h
    n(a[1] / a[0])
}
pub fn poiseuille_flow(a: &[f64]) -> R {
    n(PI * a[0].powi(4) * a[1] / (8.0 * a[2] * a[3]))
}
pub fn power_physics(a: &[f64]) -> R {
    n(a[0] / a[1])
}
pub fn pressure(a: &[f64]) -> R {
    n(a[0] / a[1])
}
pub fn quantum_harmonic_oscillator(a: &[f64]) -> R {
    // n, omega, hbar
    n(a[2] * a[1] * (a[0] + 0.5))
}
pub fn refractive_index(a: &[f64]) -> R {
    // v, c
    n(a[1] / a[0])
}
pub fn relativistic_doppler(a: &[f64]) -> R {
    // f0, v, c
    need(a[1].abs() < a[2], "speed must be less than the speed of light")?;
    let beta = a[1] / a[2];
    n(a[0] * ((1.0 - beta) / (1.0 + beta)).sqrt())
}
pub fn gravitational_time_dilation_from_radius(a: &[f64]) -> R {
    // dt, rs, r
    need(a[2] > a[1], "r must be outside the Schwarzschild radius")?;
    n(a[0] * (1.0 - a[1] / a[2]).sqrt())
}
pub fn relativistic_beta(a: &[f64]) -> R {
    // v, c
    need(a[0].abs() < a[1], "speed must be less than the speed of light")?;
    n(a[0] / a[1])
}
pub fn relativistic_doppler_frequency(a: &[f64]) -> R {
    // f0, beta
    n(a[0] * ((1.0 - a[1]) / (1.0 + a[1])).sqrt())
}
pub fn relativistic_kinetic_energy_from_gamma(a: &[f64]) -> R {
    // gamma, m, c
    n((a[0] - 1.0) * a[1] * a[2].powi(2))
}
pub fn relativistic_momentum_from_gamma(a: &[f64]) -> R {
    // gamma, m, v
    n(a[1] * a[2] * a[0])
}
pub fn length_contraction_from_gamma(a: &[f64]) -> R {
    // L0, gamma
    n(a[0] / a[1])
}
pub fn time_dilation_from_gamma(a: &[f64]) -> R {
    // dt0, gamma
    n(a[0] * a[1])
}
pub fn relativistic_kinetic_energy(a: &[f64]) -> R {
    // m, v, c
    need(a[1].abs() < a[2], "speed must be less than the speed of light")?;
    n((1.0 / (1.0 - a[1].powi(2) / a[2].powi(2)).sqrt() - 1.0) * a[0] * a[2].powi(2))
}
pub fn relativistic_momentum(a: &[f64]) -> R {
    // m, v, c
    need(a[1].abs() < a[2], "speed must be less than the speed of light")?;
    n(a[0] * a[1] / (1.0 - a[1].powi(2) / a[2].powi(2)).sqrt())
}
pub fn relativistic_velocity_addition(a: &[f64]) -> R {
    // u, v, c
    n((a[0] + a[1]) / (1.0 + a[0] * a[1] / a[2].powi(2)))
}
pub fn reynolds_number(a: &[f64]) -> R {
    n(a[0] * a[1] * a[2] / a[3])
}
pub fn rydberg_formula(a: &[f64]) -> R {
    // n1, n2, Z, R
    need(a[1] > a[0], "n2 must be larger than n1")?;
    n(1.0 / (a[3] * a[2].powi(2) * (d(1.0, a[0].powi(2))? - d(1.0, a[1].powi(2))?)))
}
pub fn schwarzschild_radius(a: &[f64]) -> R {
    // M, G, c
    n(2.0 * a[1] * a[0] / a[2].powi(2))
}
pub fn self_induced_emf(a: &[f64]) -> R {
    n(-a[0] * a[1] / a[2])
}
pub fn snell_s_law(a: &[f64]) -> R {
    n((a[0] * a[1].sin() / a[2]).asin())
}
pub fn stokes_drag(a: &[f64]) -> R {
    n(6.0 * PI * a[0] * a[1] * a[2])
}
pub fn thin_lens_equation(a: &[f64]) -> R {
    n(1.0 / (d(1.0, a[0])? - d(1.0, a[1])?))
}
pub fn torque(a: &[f64]) -> R {
    n(a[0] * a[1] * a[2].sin())
}
pub fn torricelli_speed(a: &[f64]) -> R {
    // h, g
    n((2.0 * a[1] * a[0]).sqrt())
}
pub fn velocity_squared(a: &[f64]) -> R {
    n((a[0] * a[0] + 2.0 * a[1] * a[2]).sqrt())
}
pub fn volume_flow_rate(a: &[f64]) -> R {
    n(a[0] * a[1])
}
pub fn wave_speed(a: &[f64]) -> R {
    n(a[0] * a[1])
}
pub fn weight(a: &[f64]) -> R {
    // m, g
    n(a[0] * a[1])
}
pub fn work(a: &[f64]) -> R {
    n(a[0] * a[1])
}

// ---------- motion for a walking or flying robot (the same operations in the same order as the Python files) ----------

pub fn speed_after_distance(a: &[f64]) -> R {
    // u, a, s
    let v2 = a[0] * a[0] + 2.0 * a[1] * a[2];
    need(v2 >= 0.0, "the motion stops before that distance")?;
    n(v2.sqrt())
}

pub fn time_to_distance(a: &[f64]) -> R {
    // u, a, s
    let (u, acc, s) = (a[0], a[1], a[2]);
    if acc == 0.0 {
        if u == 0.0 {
            need(s == 0.0, "not moving")?;
            return n(0.0);
        }
        let t = s / u;
        need(t >= 0.0, "that distance is not reached")?;
        return n(t + 0.0);
    }
    let disc = u * u + 2.0 * acc * s;
    need(disc >= 0.0, "that distance is not reached")?;
    let root = disc.sqrt();
    let mut best: Option<f64> = None;
    for t in [(-u + root) / acc, (-u - root) / acc] {
        if t >= 0.0 && best.map_or(true, |b| t < b) {
            best = Some(t);
        }
    }
    let t = best.ok_or_else(|| "that distance is not reached".to_string())?;
    n(t + 0.0)
}

fn opposes(u: f64, a: f64) -> Result<(), String> {
    need(!(u != 0.0 && a == 0.0), "no acceleration, it does not stop")?;
    need(u * a <= 0.0, "it is speeding up, it does not stop")
}

pub fn stopping_distance(a: &[f64]) -> R {
    // u, a
    opposes(a[0], a[1])?;
    if a[0] == 0.0 {
        return n(0.0);
    }
    n(-a[0] * a[0] / (2.0 * a[1]))
}

pub fn braking_time(a: &[f64]) -> R {
    // u, a
    opposes(a[0], a[1])?;
    if a[0] == 0.0 {
        return n(0.0);
    }
    n(-a[0] / a[1])
}

fn launch(v: f64, angle: f64, g: f64) -> Result<(), String> {
    need(v >= 0.0, "speed cannot be negative")?;
    need(angle.abs() <= std::f64::consts::FRAC_PI_2, "the angle is past straight up")?;
    need(g > 0.0, "gravity must pull downward")
}

pub fn projectile_range(a: &[f64]) -> R {
    // v, angle, g
    launch(a[0], a[1], a[2])?;
    n(a[0] * a[0] * (2.0 * a[1]).sin() / a[2])
}

pub fn projectile_time_of_flight(a: &[f64]) -> R {
    // v, angle, g
    launch(a[0], a[1], a[2])?;
    n(2.0 * a[0] * a[1].sin() / a[2])
}

pub fn hover_thrust(a: &[f64]) -> R {
    // m, g
    need(a[0] > 0.0, "mass must be positive")?;
    need(a[1] > 0.0, "gravity must pull downward")?;
    n(a[0] * a[1])
}

// ---------- walking and flying mechanics, batch 4 (the same operations in the same order as the Python files) ----------

fn mass_slope_gravity(m: f64, angle: f64, g: f64) -> Result<(), String> {
    need(m > 0.0, "mass must be positive")?;
    need((0.0..=std::f64::consts::FRAC_PI_2).contains(&angle), "the slope is past vertical")?;
    need(g > 0.0, "gravity must pull downward")
}

pub fn slope_force(a: &[f64]) -> R {
    // m, angle, g
    mass_slope_gravity(a[0], a[1], a[2])?;
    n(a[0] * a[2] * a[1].sin())
}

pub fn max_friction_on_slope(a: &[f64]) -> R {
    // mu, m, angle, g
    need(a[0] >= 0.0, "friction cannot be negative")?;
    mass_slope_gravity(a[1], a[2], a[3])?;
    n(a[0] * a[1] * a[3] * a[2].cos())
}

pub fn steepest_climb_angle(a: &[f64]) -> R {
    need(a[0] >= 0.0, "friction cannot be negative")?;
    n(a[0].atan())
}

pub fn cylinder_moment_of_inertia(a: &[f64]) -> R {
    // mass, radius
    need(a[0] > 0.0, "mass must be positive")?;
    need(a[1] >= 0.0, "radius cannot be negative")?;
    n(0.5 * a[0] * a[1] * a[1])
}

pub fn angular_acceleration(a: &[f64]) -> R {
    // torque, inertia
    need(a[1] > 0.0, "the moment of inertia must be positive")?;
    n(a[0] / a[1])
}

pub fn tipping_margin(a: &[f64]) -> R {
    // half_width, height, angle
    need(a[0] >= 0.0 && a[1] >= 0.0, "a length cannot be negative")?;
    need((0.0..=std::f64::consts::FRAC_PI_2).contains(&a[2]), "the slope is past vertical")?;
    n(a[0] * a[2].cos() - a[1] * a[2].sin())
}

pub fn impact_force(a: &[f64]) -> R {
    // mass, speed, distance
    need(a[0] > 0.0, "mass must be positive")?;
    need(a[2] > 0.0, "the stopping distance must be positive")?;
    n(a[0] * a[1] * a[1] / (2.0 * a[2]))
}

// ---------- one step of "where am I" (batch 5; the same operations in the same order as the Python files) ----------

pub fn exponential_smoothing(a: &[f64]) -> R {
    // previous, sample, alpha
    need((0.0..=1.0).contains(&a[2]), "the weight must be from 0 to 1")?;
    n(a[2] * a[1] + (1.0 - a[2]) * a[0])
}

pub fn kalman_predict_state(a: &[f64]) -> R {
    // x, u, t
    need(a[2] >= 0.0, "time cannot be negative")?;
    n(a[0] + a[1] * a[2])
}

pub fn kalman_predict_variance(a: &[f64]) -> R {
    // p, q
    need(a[0] >= 0.0, "a variance cannot be negative")?;
    need(a[1] >= 0.0, "process noise cannot be negative")?;
    n(a[0] + a[1])
}

pub fn kalman_update_state(a: &[f64]) -> R {
    // x, p, z, r
    need(a[1] >= 0.0, "a variance cannot be negative")?;
    need(a[3] > 0.0, "measurement noise must be positive")?;
    let k = a[1] / (a[1] + a[3]);
    n(a[0] + k * (a[2] - a[0]))
}

pub fn kalman_update_variance(a: &[f64]) -> R {
    // p, r
    need(a[0] >= 0.0, "a variance cannot be negative")?;
    need(a[1] > 0.0, "measurement noise must be positive")?;
    let k = a[0] / (a[0] + a[1]);
    n((1.0 - k) * a[0])
}

fn latitudes(lat1: f64, lat2: f64) -> Result<(), String> {
    let half_pi = std::f64::consts::FRAC_PI_2;
    need(lat1.abs() <= half_pi && lat2.abs() <= half_pi, "latitude is past the pole")
}

pub fn haversine_distance(a: &[f64]) -> R {
    // lat1, lon1, lat2, lon2
    latitudes(a[0], a[2])?;
    let s1 = ((a[2] - a[0]) / 2.0).sin();
    let s2 = ((a[3] - a[1]) / 2.0).sin();
    let h = (s1 * s1 + a[0].cos() * a[2].cos() * s2 * s2).max(0.0).min(1.0);
    n(6371000.0 * 2.0 * h.sqrt().atan2((1.0 - h).sqrt()))
}

pub fn bearing_between(a: &[f64]) -> R {
    // lat1, lon1, lat2, lon2
    latitudes(a[0], a[2])?;
    let dlon = a[3] - a[1];
    let east = dlon.sin() * a[2].cos();
    let north = a[0].cos() * a[2].sin() - a[0].sin() * a[2].cos() * dlon.cos();
    need(!(east == 0.0 && north == 0.0), "the two points are the same, there is no direction")?;
    n(north.atan2(east))
}

// ---------- the filter and the control step (batch 6; the same operations in the same order as the Python files) ----------

fn time_step(dt: f64) -> Result<(), String> {
    need(dt > 0.0, "time step must be positive")
}

pub fn complementary_filter(a: &[f64]) -> R {
    // previous, gyro, dt, measured, alpha
    time_step(a[2])?;
    need((0.0..=1.0).contains(&a[4]), "the weight must be from 0 to 1")?;
    n(a[4] * (a[0] + a[1] * a[2]) + (1.0 - a[4]) * a[3])
}

pub fn control_error(a: &[f64]) -> R {
    n(a[0] - a[1])
}

pub fn pid_proportional(a: &[f64]) -> R {
    n(a[0] * a[1])
}

pub fn pid_integral_state(a: &[f64]) -> R {
    // integral, error, dt
    time_step(a[2])?;
    n(a[0] + a[1] * a[2])
}

pub fn pid_integral_term(a: &[f64]) -> R {
    // ki, integral, error, dt
    time_step(a[3])?;
    n(a[0] * (a[1] + a[2] * a[3]))
}

pub fn pid_derivative(a: &[f64]) -> R {
    // kd, error, previous_error, dt
    time_step(a[3])?;
    n(a[0] * (a[1] - a[2]) / a[3])
}

pub fn pid_sum(a: &[f64]) -> R {
    n(a[0] + a[1] + a[2])
}

// ---------- saturation, the low-pass step, the turn radius and the GPS clock (batches 7 and 8; the same operations in the same order as the Python files) ----------

pub fn saturation(a: &[f64]) -> R {
    // value, low, high
    need(a[1] <= a[2], "the low limit is above the high limit")?;
    n(a[0].max(a[1]).min(a[2]))
}

pub fn low_pass_step(a: &[f64]) -> R {
    // previous, sample, dt, tau
    need(a[2] > 0.0, "time step must be positive")?;
    need(a[3] > 0.0, "the time constant must be positive")?;
    n(a[0] + a[2] / (a[2] + a[3]) * (a[1] - a[0]))
}

pub fn turn_radius(a: &[f64]) -> R {
    // speed, bank, g
    need(a[0] >= 0.0, "speed cannot be negative")?;
    need(a[1] != 0.0, "no bank, the turn radius is infinite")?;
    need(a[1].abs() < std::f64::consts::FRAC_PI_2, "the bank is past horizontal")?;
    need(a[2] > 0.0, "gravity must pull downward")?;
    n(a[0] * a[0] / (a[2] * a[1].tan()))
}

pub fn gps_kinematic_clock(a: &[f64]) -> R {
    need(a[0] >= 1.0, "a Lorentz factor is at least 1")?;
    n((1.0 / a[0] - 1.0) * 86400.0 * 1000000.0)
}

pub fn gps_gravitational_clock(a: &[f64]) -> R {
    // ground_radius, satellite_radius
    need(a[0] > 0.0 && a[1] > 0.0, "a radius must be positive")?;
    need(a[1] > a[0], "the satellite must be above the ground")?;
    let (gm, c) = (3.986004418e14, 299792458.0_f64);
    n(gm * (1.0 / a[0] - 1.0 / a[1]) / (c * c) * 86400.0 * 1000000.0)
}

pub fn gps_clock_correction(a: &[f64]) -> R {
    n(a[0] + a[1])
}

// ---------- Schwarzschild geometry outside a spherical mass (general relativity, job 1; the same operations as the Python files) ----------

fn outside_horizon(r: f64, rs: f64) -> Result<(), String> {
    need(r > 0.0, "a radius must be positive")?;
    need(rs >= 0.0, "the horizon radius cannot be negative")?;
    need(r > rs, "r is not outside the horizon")
}

pub fn schwarzschild_g_tt(a: &[f64]) -> R {
    // r, Rs
    outside_horizon(a[0], a[1])?;
    n(-(1.0 - a[1] / a[0]))
}

pub fn schwarzschild_g_rr(a: &[f64]) -> R {
    // r, Rs
    outside_horizon(a[0], a[1])?;
    n(1.0 / (1.0 - a[1] / a[0]))
}

pub fn photon_sphere_radius(a: &[f64]) -> R {
    need(a[0] > 0.0, "the horizon radius must be positive")?;
    n(1.5 * a[0])
}

pub fn isco_radius(a: &[f64]) -> R {
    need(a[0] > 0.0, "the horizon radius must be positive")?;
    n(3.0 * a[0])
}

// ---------- circular orbits around a non-spinning mass (general relativity, job 2; the same operations as the Python files) ----------

fn photon_sphere_check(r: f64, rs: f64) -> Result<(), String> {
    need(r > 0.0, "a radius must be positive")?;
    need(rs > 0.0, "the horizon radius must be positive")?;
    need(r > 1.5 * rs, "no circular orbit for a massive particle on or inside the photon sphere")
}

pub fn schwarzschild_orbit_angular_speed(a: &[f64]) -> R {
    // r, Rs
    photon_sphere_check(a[0], a[1])?;
    n(299792458.0 * (a[1] / (2.0 * a[0] * a[0] * a[0])).sqrt())
}

pub fn schwarzschild_orbit_period(a: &[f64]) -> R {
    photon_sphere_check(a[0], a[1])?;
    n(2.0 * std::f64::consts::PI / (299792458.0 * (a[1] / (2.0 * a[0] * a[0] * a[0])).sqrt()))
}

pub fn schwarzschild_coordinate_speed(a: &[f64]) -> R {
    photon_sphere_check(a[0], a[1])?;
    n(299792458.0 * (a[1] / (2.0 * a[0])).sqrt())
}

pub fn schwarzschild_local_orbit_speed(a: &[f64]) -> R {
    need(a[0] > 0.0, "a radius must be positive")?;
    need(a[1] > 0.0, "the horizon radius must be positive")?;
    need(a[0] >= 3.0 * a[1], "no stable circular orbit inside the ISCO")?;
    n(299792458.0 * ((a[1] / (2.0 * a[0])) / (1.0 - a[1] / a[0])).sqrt())
}

// ---------- light deflection (general relativity, job 3; the same operations as the Python file) ----------

pub fn light_deflection_angle(a: &[f64]) -> R {
    // b, Rs
    need(a[0] > 0.0, "a radius must be positive")?;
    need(a[1] > 0.0, "the horizon radius must be positive")?;
    need(a[0] > 1.5 * 3.0_f64.sqrt() * a[1], "the ray falls in; the small-angle formula does not apply")?;
    n(2.0 * a[1] / a[0])
}

// ---------- perihelion advance (general relativity, job 4; the same operations as the Python file) ----------

pub fn perihelion_advance(a: &[f64]) -> R {
    // a, eccentricity, period_days, Rs
    need(a[0] > 0.0, "a radius must be positive")?;
    need(a[2] > 0.0, "the period must be positive")?;
    need(a[3] > 0.0, "the horizon radius must be positive")?;
    need(a[1] >= 0.0 && a[1] < 1.0, "the eccentricity must be from 0 up to but not including 1")?;
    let pi = std::f64::consts::PI;
    n(3.0 * pi * a[3] / (a[0] * (1.0 - a[1] * a[1])) * (36525.0 / a[2]) * (648000.0 / pi))
}

// ---------- the frequency ratio between two static clocks (general relativity, job 5; the same operations as the Python file) ----------

pub fn schwarzschild_frequency_ratio(a: &[f64]) -> R {
    // r_source, r_receiver, Rs
    need(a[0] > 0.0 && a[1] > 0.0, "a radius must be positive")?;
    need(a[2] >= 0.0, "the horizon radius cannot be negative")?;
    need(a[0] > a[2] && a[1] > a[2], "r is not outside the horizon")?;
    n(((1.0 - a[2] / a[0]) / (1.0 - a[2] / a[1])).sqrt())
}

// ---------- one step of a radial fall (general relativity, job 6; the same operations as the Python file) ----------

pub fn schwarzschild_radial_step(a: &[f64]) -> R {
    // r, v, dt, Rs
    need(a[0] > 0.0, "a radius must be positive")?;
    need(a[3] >= 0.0, "the horizon radius cannot be negative")?;
    need(a[0] > a[3], "r is not outside the horizon")?;
    need(a[2] > 0.0, "time step must be positive")?;
    let c = 299792458.0_f64;
    let acc = -(a[3] * c * c) / (2.0 * a[0] * a[0]);
    n(a[0] + a[1] * a[2] + 0.5 * acc * a[2] * a[2])
}

// ---------- the Kerr geometry of a spinning mass (general relativity, job 7; the same operations as the Python files) ----------

fn kerr_check(rs: f64, a: f64) -> Result<(), String> {
    need(rs > 0.0, "the horizon radius must be positive")?;
    need(a.abs() <= rs / 2.0, "the spin is past the extreme Kerr limit")
}

pub fn kerr_outer_horizon(a: &[f64]) -> R {
    // Rs, a
    kerr_check(a[0], a[1])?;
    let h = a[0] / 2.0;
    n(h + (h * h - a[1] * a[1]).sqrt())
}

pub fn kerr_inner_horizon(a: &[f64]) -> R {
    // Rs, a
    kerr_check(a[0], a[1])?;
    let h = a[0] / 2.0;
    n(h - (h * h - a[1] * a[1]).sqrt())
}

pub fn kerr_frame_dragging_g_tphi(a: &[f64]) -> R {
    // r, Rs, a
    need(a[0] > 0.0, "a radius must be positive")?;
    kerr_check(a[1], a[2])?;
    n(-a[1] * a[2] / a[0] + 0.0)
}

// ---------- the Lense-Thirring rate (general relativity, job 8; the same operations as the Python file) ----------

pub fn lense_thirring_rate(a: &[f64]) -> R {
    // J, r, G, c
    need(a[1] > 0.0, "a radius must be positive")?;
    n(2.0 * a[2] * a[0] / (a[3] * a[3] * (a[1] * a[1] * a[1])))
}

// ---------- gravitational waves from a circular binary (general relativity, job 9; the same operations as the Python files) ----------

pub fn gravitational_wave_frequency(a: &[f64]) -> R {
    need(a[0] >= 0.0, "a frequency cannot be negative")?;
    n(2.0 * a[0])
}

pub fn chirp_mass(a: &[f64]) -> R {
    // m1, m2
    need(a[0] > 0.0 && a[1] > 0.0, "mass must be positive")?;
    n((a[0] * a[1]).powf(3.0 / 5.0) / (a[0] + a[1]).powf(1.0 / 5.0))
}

pub fn gravitational_wave_strain(a: &[f64]) -> R {
    // chirp_mass, f_gw, distance, G, c
    need(a[0] > 0.0, "mass must be positive")?;
    need(a[2] > 0.0, "a distance must be positive")?;
    need(a[1] >= 0.0, "a frequency cannot be negative")?;
    n(4.0 * (a[3] * a[0]).powf(5.0 / 3.0) * (std::f64::consts::PI * a[1]).powf(2.0 / 3.0) / (a[4].powf(4.0) * a[2]))
}

// ---------- one thin shell of the Tolman-Oppenheimer-Volkoff equation (general relativity, job 10; the same operations as the Python file) ----------

pub fn tov_pressure_step(a: &[f64]) -> R {
    // r, dr, M, rho, P, G, c
    let (r, dr, m, rho, p, g, c) = (a[0], a[1], a[2], a[3], a[4], a[5], a[6]);
    need(r > 0.0, "a radius must be positive")?;
    need(dr > 0.0, "the shell thickness must be positive")?;
    need(m > 0.0, "mass must be positive")?;
    need(rho > 0.0, "density must be positive")?;
    need(p >= 0.0, "pressure cannot be negative")?;
    need(2.0 * g * m / (r * c.powi(2)) < 1.0, "r is not outside the horizon")?;
    n(-(g * m * rho / r.powi(2)) * (1.0 + p / (rho * c.powi(2))) * (1.0 + 4.0 * std::f64::consts::PI * r.powf(3.0) * p / (m * c.powi(2))) / (1.0 - 2.0 * g * m / (r * c.powi(2))) * dr)
}

// ---------- quantum mechanics, job 1 (the same operations as the Python files) ----------

pub fn infinite_well_transition(a: &[f64]) -> R {
    // n2, n1, m, L, h
    need(a[0] >= 1.0 && a[0].fract() == 0.0 && a[1] >= 1.0 && a[1].fract() == 0.0, "a level must be a positive whole number")?;
    need(a[0] > a[1], "the upper level must be above the lower level")?;
    need(a[2] > 0.0, "mass must be positive")?;
    need(a[3] > 0.0, "a length must be positive")?;
    n((a[0].powi(2) - a[1].powi(2)) * a[4].powi(2) / (8.0 * a[2] * a[3].powi(2)))
}

pub fn infinite_well_probability_density(a: &[f64]) -> R {
    // n, L, x
    need(a[0] >= 1.0 && a[0].fract() == 0.0, "a level must be a positive whole number")?;
    need(a[1] > 0.0, "a length must be positive")?;
    need(a[2] >= 0.0 && a[2] <= a[1], "the position is outside the box")?;
    n(2.0 / a[1] * (a[0] * PI * a[2] / a[1]).sin().powi(2))
}

pub fn bohr_radius(a: &[f64]) -> R {
    // Z, a0
    need(a[0] >= 1.0 && a[0].fract() == 0.0, "the atomic number must be a positive whole number")?;
    n(a[1] / a[0])
}

pub fn electron_compton_wavelength(a: &[f64]) -> R {
    // m, h, c
    n(a[1] / (a[0] * a[2]))
}

// ---------- quantum mechanics, job 2 (the same operations as the Python file) ----------

pub fn rectangular_barrier_transmission(a: &[f64]) -> R {
    // E, V, L, m, hbar
    need(a[0] > 0.0, "the energy must be positive")?;
    need(a[0] < a[1], "the energy is not below the barrier")?;
    need(a[2] > 0.0, "a length must be positive")?;
    need(a[3] > 0.0, "mass must be positive")?;
    need(a[4] > 0.0, "hbar must be positive")?;
    let kappa = (2.0 * a[3] * (a[1] - a[0])).sqrt() / a[4];
    let s = (kappa * a[2]).sinh();
    n(1.0 / (1.0 + a[1].powi(2) * s.powi(2) / (4.0 * a[0] * (a[1] - a[0]))))
}

// ---------- quantum mechanics, job 3 (the same operations as the Python files) ----------

pub fn oscillator_level_gap(a: &[f64]) -> R {
    // omega, hbar
    need(a[0] > 0.0, "the angular frequency must be positive")?;
    need(a[1] > 0.0, "hbar must be positive")?;
    n(a[1] * a[0])
}

pub fn oscillator_ground_density(a: &[f64]) -> R {
    // m, omega, x, hbar
    need(a[0] > 0.0, "mass must be positive")?;
    need(a[1] > 0.0, "the angular frequency must be positive")?;
    need(a[3] > 0.0, "hbar must be positive")?;
    n((a[0] * a[1] / (PI * a[3])).sqrt() * (-a[0] * a[1] * a[2].powi(2) / a[3]).exp())
}

pub fn oscillator_ground_variance(a: &[f64]) -> R {
    // m, omega, hbar
    need(a[0] > 0.0, "mass must be positive")?;
    need(a[1] > 0.0, "the angular frequency must be positive")?;
    need(a[2] > 0.0, "hbar must be positive")?;
    n(a[2] / (2.0 * a[0] * a[1]))
}

// ---------- quantum mechanics, job 4 (the same operations as the Python files) ----------

pub fn spin_z_up_probability(a: &[f64]) -> R {
    // prepared
    need(a[0] == 1.0 || a[0] == -1.0, "the prepared spin must be 1 or -1")?;
    n(if a[0] == 1.0 { 1.0 } else { 0.0 })
}

pub fn spin_up_at_angle(a: &[f64]) -> R {
    // theta
    n((1.0 + a[0].cos()) / 2.0)
}

// ---------- quantum mechanics, job 5 (the same operations as the Python file) ----------

pub fn hydrogen_mean_radius(a: &[f64]) -> R {
    // n, l, Z, a0
    need(a[0] >= 1.0 && a[0].fract() == 0.0, "a level must be a positive whole number")?;
    need(a[2] >= 1.0 && a[2].fract() == 0.0, "the atomic number must be a positive whole number")?;
    need(a[1] >= 0.0 && a[1] <= a[0] - 1.0 && a[1].fract() == 0.0, "the angular momentum must be a whole number from 0 up to n - 1")?;
    n(a[3] / (2.0 * a[2]) * (3.0 * a[0].powi(2) - a[1] * (a[1] + 1.0)))
}

// ---------- quantum mechanics, job 6 (the same operations as the Python file) ----------

pub fn fermi_energy(a: &[f64]) -> R {
    // n, m, hbar
    need(a[0] > 0.0, "the density must be positive")?;
    need(a[1] > 0.0, "mass must be positive")?;
    need(a[2] > 0.0, "hbar must be positive")?;
    n(a[2].powi(2) / (2.0 * a[1]) * (3.0 * PI.powi(2) * a[0]).powf(2.0 / 3.0))
}
