use crate::{ca_simulator::CaSimulator, matter::MatterId};
use bevy::math::IVec2;
use vulkano_util::context::VulkanoContext;

fn test_setup() -> (VulkanoContext, CaSimulator) {
    // Create vulkano context
    let vulkano_context = VulkanoContext::default();
    // Create Simulation pipeline
    let simulator = CaSimulator::new(vulkano_context.compute_queue());
    (vulkano_context, simulator)
}

#[test]
fn test_example_sandfall() {
    let (_ctx, mut simulator) = test_setup();
    let pos = IVec2::new(10, 10);
    // Empty matter first
    assert_eq!(simulator.query_matter(pos), Some(MatterId::Empty));
    simulator.draw_matter(&[pos], 0.5, MatterId::Sand);
    // After drawing, We have Sand
    assert_eq!(simulator.query_matter(pos), Some(MatterId::Sand));
    // Step once
    simulator.step(1, false);
    // Old position is empty
    assert_eq!(simulator.query_matter(pos), Some(MatterId::Empty));
    // New position under has Sand
    assert_eq!(
        simulator.query_matter(pos + IVec2::new(0, -1)),
        Some(MatterId::Sand)
    );
}
