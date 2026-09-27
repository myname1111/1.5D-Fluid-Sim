use fluid_simulation_1_5d::{App, simulation::Simulation};

const NUM_CELLS: usize = 100;

fn main() -> anyhow::Result<()> {
    let mut simulation = Simulation::<NUM_CELLS>::new(3.0, 1.0);
    for i in 25..75 {
        simulation.height_map[i] += 2.0 * (25.0 - (i as f32 - 50.0).abs()) / 25.0;
    }
    App::<NUM_CELLS>::run(simulation)
}
