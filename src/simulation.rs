const GRAVITY: f32 = 9.81;

pub struct Simulation<const N: usize> {
    pub height_map: Box<[f32; N]>,
    momentum: Box<[f32; N]>,
    dx: f32,
}

impl<const N: usize> Simulation<N> {
    pub fn new(initial_height: f32, dx: f32) -> Self {
        Self {
            height_map: Box::new([initial_height; N]),
            momentum: Box::new([0.0; N]),
            dx,
        }
    }

    fn minmod2(a: f32, b: f32) -> f32 {
        if a * b <= 0.0 {
            0.0
        } else {
            a.signum() * a.abs().min(b.abs())
        }
    }

    fn difference(grid: &[f32], ghosts: (f32, f32)) -> Box<[f32; N]> {
        let mut out = Box::new([0.0; N]);
        for i in 0..N {
            let left = if i == 0 {
                grid[0] - ghosts.0
            } else {
                grid[i] - grid[i - 1]
            };

            let right = if i == N - 1 {
                ghosts.1 - grid[N - 1]
            } else {
                grid[i + 1] - grid[i]
            };

            out[i] = Self::minmod2(left, right)
        }
        out
    }

    fn looping_bc(grid: &[f32]) -> (f32, f32) {
        (grid[N - 1], grid[0])
    }

    fn potential(height_map: f32, momentum: f32) -> (f32, f32) {
        let unsafe_velocity = momentum / height_map;
        let velocity = if unsafe_velocity.is_normal() {
            unsafe_velocity
        } else {
            0.0
        };

        let forces = momentum * velocity + 0.5 * GRAVITY * height_map.powi(2);

        (momentum, forces)
    }

    pub(crate) fn update(&mut self, dt: f32) {
        let ghost_height = Self::looping_bc(self.height_map.as_slice());
        let ghost_mom = Self::looping_bc(self.momentum.as_slice());

        let (potential_height, potential_mom) = std::iter::once((&ghost_height.0, &ghost_mom.0))
            .chain(self.height_map.iter().zip(self.momentum.iter()))
            .chain(std::iter::once((&ghost_height.1, &ghost_mom.1)))
            .map(|(&height, &momentum)| Self::potential(height, momentum))
            .collect::<(Vec<f32>, Vec<f32>)>();

        let pred_height = Self::difference(
            &potential_height[1..(N + 1)],
            (potential_height[0], potential_height[N + 1]),
        )
        .iter()
        .zip(self.height_map.iter())
        .map(|(delta, height)| height - dt / self.dx * delta)
        .collect::<Vec<f32>>();
        let pred_mom = Self::difference(
            &potential_mom[1..(N + 1)],
            (potential_mom[0], potential_mom[N + 1]),
        )
        .iter()
        .zip(self.momentum.iter())
        .map(|(delta, momentum)| momentum - dt / self.dx * delta)
        .collect::<Vec<f32>>();

        let ghost_pred_height = Self::looping_bc(&pred_height[..]);
        let ghost_pred_mom = Self::looping_bc(&pred_mom[..]);

        let (potential_pred_height, potential_pred_mom) =
            std::iter::once((&ghost_pred_height.0, &ghost_pred_mom.0))
                .chain(pred_height.iter().zip(pred_mom.iter()))
                .chain(std::iter::once((&ghost_pred_height.1, &ghost_pred_mom.1)))
                .map(|(&height, &momentum)| Self::potential(height, momentum))
                .collect::<(Vec<f32>, Vec<f32>)>();

        let mut new_height = Box::new([0.0; N]);
        let mut new_momentum = Box::new([0.0; N]);

        for (idx, (delta, height)) in Self::difference(
            &potential_pred_height[1..(N + 1)],
            (potential_pred_height[0], potential_pred_height[N + 1]),
        )
        .iter()
        .zip(pred_height)
        .enumerate()
        {
            new_height[idx] = (self.height_map[idx] + height) / 2.0 - dt / (2.0 * self.dx) * delta
        }

        for (idx, (delta, momentum)) in Self::difference(
            &potential_pred_mom[1..(N + 1)],
            (potential_pred_mom[0], potential_pred_mom[N + 1]),
        )
        .iter()
        .zip(pred_mom)
        .enumerate()
        {
            new_momentum[idx] = (self.momentum[idx] + momentum) / 2.0 - dt / (2.0 * self.dx) * delta
        }

        self.height_map = new_height;
        self.momentum = new_momentum;
    }

    pub(crate) fn height(&self) -> &[f32] {
        &self.height_map[..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TOLERANCE: f32 = 1e-9;

    #[test]
    fn test_diff() {
        let grid = Box::new([0.0, 1.0, 1.0, 0.0]);
        let ghosts = (1.0, 1.0);
        let expected = Box::new([0.0, 0.0, 0.0, 0.0]);
        let result: Box<[f32; 4]> = Simulation::difference(grid.as_slice(), ghosts);
        dbg!(&result, &expected);
        assert!(
            expected
                .iter()
                .zip(result.iter())
                .all(|(a, b)| (a - b).abs() < TOLERANCE)
        );
    }

    #[test]
    fn test_looping_bc() {
        let grid = Box::new([0.0, 1.0, 1.0, 1.0]);
        let expected = (1.0, 0.0);
        let result = Simulation::<4>::looping_bc(&grid[..]);
        assert!((result.0 - expected.0).abs() < TOLERANCE);
        assert!((result.1 - expected.1).abs() < TOLERANCE);
    }

    #[test]
    fn debug() {
        let mut sim = Simulation::<2>::new(2.0, 1.0);
        dbg!(&sim.height_map);
        dbg!(&sim.momentum);
        sim.update(1.0 / 60.0);
        dbg!(&sim.height_map);
        dbg!(&sim.momentum);
    }

    #[test]
    fn test_minmod() {
        assert_eq!(Simulation::<1>::minmod2(1.0, 2.0), 1.0);
        assert_eq!(Simulation::<1>::minmod2(-1.0, 2.0), 0.0);
        assert_eq!(Simulation::<1>::minmod2(-1.0, -2.0), -1.0);
        assert_eq!(Simulation::<1>::minmod2(-1.0, 0.0), 0.0);
    }
}
