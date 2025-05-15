// Mock implementations for testing DEX functionality
//
// This module provides mock data and implementations that can be used
// for testing the DEX quoting functionality

use orca_whirlpools_core::{TickArrays, TickArrayFacade, TickFacade, TICK_ARRAY_SIZE};

/// Create mock tick arrays for Orca testing
///
/// This is a simplified implementation that creates empty tick arrays
/// In a real implementation, we would use actual on-chain data
pub fn create_mock_tick_arrays() -> TickArrays {
    // In a real case, we'd fetch tick arrays from the chain
    // Create a mock TickArrayFacade with minimal data needed

    // Create an array of TickFacade elements filled with default values
    let default_tick = TickFacade::default();
    let ticks = [default_tick; TICK_ARRAY_SIZE];

    let tick_array = TickArrayFacade {
        start_tick_index: 0,
        ticks,
    };

    // Return the simplest variant of TickArrays with just one tick array
    TickArrays::One(tick_array)
}
