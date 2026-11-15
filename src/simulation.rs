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

    fn difference(grid: &[f32], ghosts: (f32, f32), is_forward: bool) -> Box<[f32; N]> {
        let mut out = Box::new([0.0; N]);
        for i in 0..N {
            out[i] = match (i == 0, i == N - 1, is_forward) {
                (true, _, false) => grid[0] - ghosts.0,
                (_, true, true) => ghosts.1 - grid[N - 1],
                (_, _, true) => grid[i + 1] - grid[i],
                (_, _, false) => grid[i] - grid[i - 1],
            };
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

    fn compute_potential(&self) -> (Vec<f32>, Vec<f32>) {
        let ghost_height = Self::looping_bc(self.height_map.as_slice());
        let ghost_mom = Self::looping_bc(self.momentum.as_slice());

        std::iter::once((&ghost_height.0, &ghost_mom.0))
            .chain(self.height_map.iter().zip(self.momentum.iter()))
            .chain(std::iter::once((&ghost_height.1, &ghost_mom.1)))
            .map(|(&height, &momentum)| Self::potential(height, momentum))
            .collect::<(Vec<f32>, Vec<f32>)>()
    }

    fn maccormack(&self, dt: f32, potential_height: Vec<f32>, potential_mom: Vec<f32>) -> (Box<[f32; N]>, Box<[f32; N]>){       
        let pred_height = Self::difference(
            &potential_height[1..(N + 1)],
            (potential_height[0], potential_height[N + 1]),
            true,
        )
        .iter()
        .zip(self.height_map.iter())
        .map(|(delta, height)| height - dt / self.dx * delta)
        .collect::<Vec<f32>>();
        let pred_mom = Self::difference(
            &potential_mom[1..(N + 1)],
            (potential_mom[0], potential_mom[N + 1]),
            true,
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
            false,
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
            false,
        )
        .iter()
        .zip(pred_mom)
        .enumerate()
        {
            new_momentum[idx] = (self.momentum[idx] + momentum) / 2.0 - dt / (2.0 * self.dx) * delta
        }

        (new_height, new_momentum)
    }

    fn rusanov(&self, dt: f32, potential_height: Vec<f32>, potential_mom: Vec<f32>) -> (Box<[f32; N]>, Box<[f32; N]>) {
        let mut alpha_raw = vec![];
        for i in 0..N {
            alpha_raw.push((self.momentum[i] / self.height_map[i]).abs() + (GRAVITY * self.height_map[i]).sqrt());
        }
        let mut alpha = vec![0.0; N];
        for i in 0..N {
            alpha.push(alpha_raw[i].max(alpha_raw[i + 1]));
        }

        let mut flux_height_center = vec![];
        let mut flux_momentum_center = vec![];
        for i in 0..N  {
            flux_height_center.push((potential_height[i] + potential_height[i + 1]) / 2.0 - 1.0 / 2.0 * alpha[i] * (self.height_map[i + 1] - self.height_map[i]));
            flux_momentum_center.push((potential_mom[i] + potential_mom[i + 1]) / 2.0 - 1.0 / 2.0 * alpha[i] * (self.momentum[i + 1] - self.momentum[i]));
        }

        let mut new_height = Box::new([0.0; N]);
        let mut new_mom = Box::new([0.0; N]);

        for i in 0..N {
            new_height[i] = self.height_map[i] - dt / self.dx * (flux_height_center[i] - flux_height_center[i - 1]);
            new_mom[i] = self.momentum[i] - dt / self.dx * (flux_momentum_center[i] - flux_momentum_center[i - 1]);
        }

        (new_height, new_mom)
    }

    fn flux_limiter(ratio: f32) -> f32 {
        ratio.clamp(0.0, 1.0)
    }

    pub(crate) fn update(&mut self, dt: f32) {
        let (potential_height, potential_mom) = self.compute_potential();
        let (new_height_mac, new_mom_mac) = self.maccormack(dt, potential_height.clone(), potential_mom.clone());
        let (new_height_rus, new_mom_rus) = self.rusanov(dt, potential_height, potential_mom);

        let mut height_ratios = vec![];
        let mut mom_ratios = vec![];
        for i in 0..N {
            height_ratios.push(Self::flux_limiter((self.height_map[i + 1] - self.height_map[i]) / (self.height_map[i] - self.height_map[i - 1])));
            mom_ratios.push(Self::flux_limiter((self.momentum[i + 1] - self.momentum[i]) / (self.momentum[i] - self.momentum[i - 1])))
        }

        let mut new_height = Box::new([0.0; N]);
        let mut new_mom = Box::new([0.0; N]);
        for i in 0..N {
            new_height[i] = new_height_rus[i] + height_ratios[i] * (new_height_mac[i] - new_height_rus[i]);
            new_mom[i] = new_mom[i] + mom_ratios[i] * (new_mom_mac[i] - new_mom_rus[i]);
        }

        self.height_map = new_height;
        self.momentum = new_mom;
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
    fn test_forward_diff() {
        let grid = Box::new([0.0, 1.0, 1.0, 0.0]);
        let ghosts = (1.0, 1.0);
        let expected = Box::new([1.0, 0.0, -1.0, 1.0]);
        let result: Box<[f32; 4]> = Simulation::difference(grid.as_slice(), ghosts, true);
        dbg!(&result, &expected);
        assert!(
            expected
                .iter()
                .zip(result.iter())
                .all(|(a, b)| (a - b).abs() < TOLERANCE)
        );
    }

    #[test]
    fn test_backward_diff() {
        let grid = Box::new([0.0, 1.0, 1.0, 0.0]);
        let ghosts = (1.0, 1.0);
        let expected = Box::new([-1.0, 1.0, 0.0, -1.0]);
        let result: Box<[f32; 4]> = Simulation::difference(grid.as_slice(), ghosts, false);
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
}
