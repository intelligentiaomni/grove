import streamlit as st
import requests

st.set_page_config(page_title="Spatial Matrix Optimizer", layout="wide")
st.title("Computational Spatial-Optimization Dashboard")
st.write("Cross-Disciplinary Node Allocation System")

# Setup UI input parameters
st.sidebar.header("Network Nodes Parameters")prod_a = st.sidebar.slider("Producer A Output (kW / Data Units)", 0.0, 50.0, 20.0)cons_b = st.sidebar.slider("Consumer B Demand (kW / Data Units)", 0.0, 50.0, 12.0)

# Build the payload mapping structure to feed to the microservicemock_payload = {
    "system_id": "stockholm_distribution_mesh",
    "nodes": [
        {"id": "Source_Alpha", "balance": prod_a},
        {"id": "Source_Beta", "balance": max(0.0, cons_b - prod_a + 5.0)}, # Ensure balancing mathematically
        {"id": "Sink_Delta", "balance": -cons_b}
    ],
    "edges": [
        {"source": "Source_Alpha", "target": "Sink_Delta", "resistance": 0.05},
        {"source": "Source_Beta", "target": "Sink_Delta", "resistance": 0.25}
    ]
}
if st.button("Compute Optimal Network Routing Paths"):
    st.write("Sending request to containerized User-Space Daemon on port `:8000`...")
    
    try:
        # Issue IPC call to the isolated Computational Optimization Server boundary
        response = requests.post("http://localhost:8000/api/v1/optimize", json=mock_payload)
        
        if response.status_code == 200:
            data = response.json()
            st.success(f"Optimal Matrix Calculated. Total Network Friction Loss: {data['metrics']['total_system_loss']}")
            
            # Print beautiful visual data matrix layout metrics
            st.subheader("Optimized Matrix Vectors Output")
            st.table(data["routing_matrix"])
        else:
            st.error(f"Server rejection profile error: {response.text}")
    except requests.exceptions.ConnectionError:
        st.error("Connection failure. Ensure your optimization backend container is running on localhost:8000.")
