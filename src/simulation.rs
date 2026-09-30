use std::ops::{Add, Mul};

const GRAVITY: f32 = 9.81;

pub struct Simulation<const N: usize> {
    pub height_map: Box<[f32; N]>,
    momentum: Box<[f32; N]>,
    dx: f32,
}

fn add_array<const N: usize, T: Add<Output = T> + Copy + Default>(
    a: &[T; N],
    b: &[T; N],
) -> Box<[T; N]> {
    let mut out = Box::new([T::default(); N]);
    for i in 0..N {
        out[i] = a[i] + b[i]
    }
    out
}

fn scale_array_by_scalar<const N: usize, T: Mul<Output = T> + Copy + Default>(
    a: &[T; N],
    x: T,
) -> Box<[T; N]> {
    let mut out = Box::new([T::default(); N]);
    for i in 0..N {
        out[i] = a[i] * x;
    }
    out
}

impl<const N: usize> Simulation<N> {
    pub fn new(initial_height: f32, dx: f32) -> Self {
        Self {
            height_map: Box::new([initial_height; N]),
            momentum: Box::new([0.0; N]),
            dx,
        }
    }

    fn left_middle(values: &[f32]) -> Vec<f32> {
        let mut out = vec![];
        for idx in 2..(values.len() - 2) {
            // WENO5
            let v_0 = 1.0 / 3.0 * values[idx - 2] - 7.0 / 6.0 * values[idx - 1]
                + 11.0 / 6.0 * values[idx];
            let v_1 = -1.0 / 6.0 * values[idx - 1]
                + 5.0 / 6.0 * values[idx]
                + 1.0 / 3.0 * values[idx + 1];
            let v_2 =
                1.0 / 3.0 * values[idx - 2] + 5.0 / 6.0 * values[idx - 1] - 1.0 / 6.0 * values[idx];

            let comb = 0.1 * v_0 + 0.6 * v_1 + 0.3 * v_2;
            out.push(comb);
        }
        out
    }

    fn flux(height_map: f32, momentum: f32) -> (f32, f32) {
        let unsafe_velocity = momentum / height_map;
        let velocity = if unsafe_velocity.is_normal() {
            unsafe_velocity
        } else {
            0.0
        };

        let forces = momentum * velocity + 0.5 * GRAVITY * height_map.powi(2);

        (momentum, forces)
    }

    fn jacobian_flux(height_map: f32, momentum: f32) -> [[f32; 2]; 2] {
        let velocity = momentum / height_map;
        [
            [0.0, 1.0],
            [-velocity.powi(2) + GRAVITY * height_map, 2.0 * velocity],
        ]
    }

    fn ghost(values: &[f32]) -> Vec<f32> {
        let mut out = vec![];
        // Looping BC
        out.push(values[0]);
        out.push(values[0]);
        out.extend_from_slice(values);
        out.push(values[values.len() - 1]);
        out.push(values[values.len() - 1]);
        out.push(values[values.len() - 1]);

        out
    }

    fn boundary_flux(
        state: ((f32, f32), (f32, f32)),
        flux: ((f32, f32), (f32, f32)),
        jacobians: ([[f32; 2]; 2], [[f32; 2]; 2]),
    ) -> (f32, f32) {
        // Lax-Friedrichs

        let alpha = [
            [
                jacobians.0[0][0].max(jacobians.1[0][0]),
                jacobians.0[0][1].max(jacobians.1[0][1]),
            ],
            [
                jacobians.0[1][0].max(jacobians.1[1][0]),
                jacobians.0[1][1].max(jacobians.1[1][1]),
            ],
        ];

        let height_diff = state.1.0 - state.0.0;
        let momentum_diff = state.1.1 - state.0.1;
        let height_flux = (flux.0.0 + flux.1.0) / 2.0
            + alpha[0][0] * height_diff / 2.0
            + alpha[0][1] * momentum_diff / 2.0;
        let momentum_flux = (flux.0.1 + flux.1.1) / 2.0
            + alpha[1][0] * height_diff / 2.0
            + alpha[1][1] * momentum_diff / 2.0;

        (height_flux, momentum_flux)
    }

    fn get_time_derivative(height_map: &[f32], momentum: &[f32]) -> (Box<[f32; N]>, Box<[f32; N]>) {
        let height_map_ghost = Self::ghost(&height_map);
        let momentum_ghost = Self::ghost(&momentum);

        let height_map_middle = Self::left_middle(&height_map_ghost);
        let momentum_middle = Self::left_middle(&momentum_ghost);

        // To get left values, use 0..N
        // To get right values use 1..(N + 1)

        let zipped = height_map_middle.iter().zip(momentum_middle.iter());

        let raw_fluxes = zipped
            .clone()
            .map(|(&height_map, &momentum)| Self::flux(height_map, momentum))
            .collect::<Vec<(f32, f32)>>();

        let jacobians = zipped
            .map(|(&height_map, &momentum)| Self::jacobian_flux(height_map, momentum))
            .collect::<Vec<_>>();

        let mut height_fluxes = Box::new([0.0; N]);
        let mut momentum_fluxes = Box::new([0.0; N]);
        for i in 0..N {
            let new = Self::boundary_flux(
                (
                    (height_map_middle[i], momentum_middle[i]),
                    (height_map_middle[i + 1], momentum_middle[i + 1]),
                ),
                (raw_fluxes[i], raw_fluxes[i + 1]),
                (jacobians[i], jacobians[i + 1]),
            );

            height_fluxes[i] = new.0;
            momentum_fluxes[i] = new.1;
        }
        (height_fluxes, momentum_fluxes)
    }

    pub(crate) fn update(&mut self, dt: f32) {
        let (height_change_1, momentum_change_1) =
            Self::get_time_derivative(&*self.height_map, &*self.momentum);
        let height_1 = add_array(
            &*self.height_map,
            &*scale_array_by_scalar(&*height_change_1, dt),
        );
        let momentum_1 = add_array(
            &*self.momentum,
            &*scale_array_by_scalar(&*momentum_change_1, dt),
        );

        let (height_change_2, momentum_change_2) =
            Self::get_time_derivative(&*height_1, &*momentum_1);
        let height_2 = add_array(
            &*scale_array_by_scalar(&*self.height_map, 3.0 / 4.0),
            &*add_array(
                &*scale_array_by_scalar(&*height_1, 1.0 / 4.0),
                &*scale_array_by_scalar(&*height_change_2, dt / 4.0),
            ),
        );
        let momentum_2 = add_array(
            &*scale_array_by_scalar(&*self.momentum, 3.0 / 4.0),
            &*add_array(
                &*scale_array_by_scalar(&*momentum_1, 1.0 / 4.0),
                &*scale_array_by_scalar(&*momentum_change_2, dt / 4.0),
            ),
        );

        let (height_change_3, momentum_change_3) =
            Self::get_time_derivative(&*height_2, &*momentum_2);
        self.height_map = add_array(
            &*scale_array_by_scalar(&*self.height_map, 1.0 / 3.0),
            &*add_array(
                &*scale_array_by_scalar(&*height_2, 2.0 / 3.0),
                &*scale_array_by_scalar(&*height_change_3, 2.0 * dt / 3.0),
            ),
        );
        self.momentum = add_array(
            &*scale_array_by_scalar(&*self.momentum, 1.0 / 3.0),
            &*add_array(
                &*scale_array_by_scalar(&*momentum_2, 2.0 / 3.0),
                &*scale_array_by_scalar(&*momentum_change_3, 2.0 * dt / 3.0),
            ),
        );
    }

    pub(crate) fn height(&self) -> &[f32] {
        &self.height_map[..]
    }
}
