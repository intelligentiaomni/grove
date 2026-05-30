import networkx as nx
import numpy as np
from scipy.optimize import minimize

def run_network_optimization(nodes: list, edges: list) -> dict:
    """
    Core mathematical engine. 
    Accepts raw node/edge lists and minimizes quadratic network friction/loss.
    """
    # 1. Initialize Network Topology
    G = nx.DiGraph()
    for node in nodes:
        G.add_node(node["id"], power=node["balance"])
    for edge in edges:
        G.add_edge(edge["source"], edge["target"], resistance=edge["resistance"])
        
    edge_list = list(G.edges())
    num_edges = len(edge_list)
    
    if num_edges == 0:
        return {"minimized_loss": 0.0, "matrix": []}

    # 2. Objective Function: Minimize Power Loss as Heat (Flow^2 * Resistance)
    def objective_function(flows):
        total_loss = 0
        for i, edge in enumerate(edge_list):
            R = G[edge[0]][edge[1]]['resistance']
            total_loss += (flows[i] ** 2) * R
        return total_loss

    # 3. Constraints: Kirchhoff's Current Law (Net flow balances node state)
    def flow_constraints(flows):
        constraints = []
        for node in G.nodes():
            node_power = G.nodes[node]['power']
            flow_in = sum(flows[i] for i, edge in enumerate(edge_list) if edge[1] == node)
            flow_out = sum(flows[i] for i, edge in enumerate(edge_list) if edge[0] == node)
            constraints.append(flow_in - flow_out + node_power)
        return np.array(constraints)

    # 4. Run Numerical Solver
    bounds = [(0, 100) for _ in range(num_edges)]
    initial_flows = np.zeros(num_edges)
    
    result = minimize(
        objective_function, 
        initial_flows, 
        method='SLSQP', 
        bounds=bounds, 
        constraints={'type': 'eq', 'fun': flow_constraints}
    )
    
    # 5. Package Output Vector Matrix
    routing_matrix = []
    for i, edge in enumerate(edge_list):
        routing_matrix.append({
            "source": edge[0],
            "target": edge[1],
            "optimized_flow": round(float(result.x[i]), 2)
        })
        
    return {
        "minimized_loss": round(float(result.fun), 4),
        "matrix": routing_matrix
    }