import numpy as np
import cvxpy as cp

def solve_arbitrage(global_indices, local_indices, reserves, fees, market_value):
    """
    Solves the arbitrage optimization problem and returns the results.
    """
    # Build local-global matrices
    n = len(global_indices)
    A = []
    for l in local_indices:
        n_i = len(l)
        A_i = np.zeros((n, n_i))
        for i, idx in enumerate(l):
            A_i[idx, i] = 1
        A.append(A_i)

    # Build variables
    deltas = [cp.Variable(len(l), nonneg=True) for l in local_indices]
    lambdas = [cp.Variable(len(l), nonneg=True) for l in local_indices]
    psi = cp.sum([A_i @ (L - D) for A_i, D, L in zip(A, deltas, lambdas)])

    # Objective
    obj = cp.Maximize(market_value @ psi)

    # Reserves after trade
    new_reserves = [R + gamma_i * D - L for R, gamma_i, D, L in zip(reserves, fees, deltas, lambdas)]

    # Constraints
    cons = [
        cp.geo_mean(new_reserves[0], p=np.array([4, 3, 2, 1])) >= cp.geo_mean(reserves[0]),
        cp.geo_mean(new_reserves[1]) >= cp.geo_mean(reserves[1]),
        cp.geo_mean(new_reserves[2]) >= cp.geo_mean(reserves[2]),
        cp.geo_mean(new_reserves[3]) >= cp.geo_mean(reserves[3]),
        cp.sum(new_reserves[4]) >= cp.sum(reserves[4]),
        new_reserves[4] >= 0,
        psi >= 0
    ]

    # Solve problem
    prob = cp.Problem(obj, cons)
    prob.solve()

    return prob, deltas, lambdas, A