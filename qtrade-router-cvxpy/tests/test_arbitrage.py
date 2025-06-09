import numpy as np
from qtrade.arbitrage.core import solve_arbitrage

def test_arbitrage():
    global_indices = list(range(4))
    local_indices = [
        [0, 1, 2, 3],
        [0, 1],
        [1, 2],
        [2, 3],
        [2, 3]
    ]
    reserves = list(map(np.array, [
        [4, 4, 4, 4],
        [10, 1],
        [1, 5],
        [40, 50],
        [10, 10]
    ]))
    fees = [0.998, 0.997, 0.997, 0.997, 0.999]
    market_value = [1.5, 10, 2, 3]

    prob, deltas, lambdas, A = solve_arbitrage(global_indices, local_indices, reserves, fees, market_value)

    # Assertions with messages
    assert prob.status == "optimal", "The optimization problem did not reach an optimal solution."
    assert len(deltas) == len(global_indices), f"Expected {len(global_indices)} deltas, but got {len(deltas)}."
    assert len(lambdas) == len(local_indices), f"Expected {len(local_indices)} lambdas, but got {len(lambdas)}."
    assert A.shape == (len(global_indices), len(global_indices)), (
        f"Expected A to have shape ({len(global_indices)}, {len(global_indices)}), but got {A.shape}.")
