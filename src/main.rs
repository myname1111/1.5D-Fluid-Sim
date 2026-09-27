use fluid_simulation_1_5d::App;

const NUM_CELLS: usize = 100;

fn main() -> anyhow::Result<()> {
    App::<NUM_CELLS>::run()
}
