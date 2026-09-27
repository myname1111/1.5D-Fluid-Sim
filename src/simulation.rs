pub(crate) struct Simulation<const N: usize> {
    height_map: Box<[f32; N]>,
}

impl<const N: usize> Simulation<N> {
    pub(crate) fn new(initial_height: f32) -> Self {
        Self {
            height_map: Box::new([initial_height; N]),
        }
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
