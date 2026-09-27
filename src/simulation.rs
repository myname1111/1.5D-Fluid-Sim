pub(crate) struct Simulation<const N: usize> {
    height_map: Box<[f32; N]>,
    velocity_map: Box<[f32; N]>,
}

impl<const N: usize> Simulation<N> {
    pub(crate) fn new(initial_height: f32) -> Self {
        Self {
            height_map: Box::new([initial_height; N]),
            velocity_map: Box::new([0.0; N]),
        }
    }

    fn difference(grid: Box<[f32; N]>, ghosts: (f32, f32), is_forward: bool) -> Box<[f32; N]> {
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

    pub(crate) fn update(&mut self, dt: f32) {
        for i in 0..self.height_map.len() {
            self.height_map[i] += dt * 0.1;
        }
    }

    pub(crate) fn height(&self) -> &[f32] {
        self.height_map.as_slice()
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
        let result = Simulation::difference(grid, ghosts, true);
        dbg!(&result, &expected);
        assert!(expected.iter().zip(result.iter()).all(|(a, b)| (a - b).abs() < TOLERANCE));
    }
    
    #[test]
    fn test_backward_diff() {
        let grid = Box::new([0.0, 1.0, 1.0, 0.0]);
        let ghosts = (1.0, 1.0);
        let expected = Box::new([-1.0, 1.0, 0.0, -1.0]);
        let result = Simulation::difference(grid, ghosts, false);
        dbg!(&result, &expected);
        assert!(expected.iter().zip(result.iter()).all(|(a, b)| (a - b).abs() < TOLERANCE));
    }
}
