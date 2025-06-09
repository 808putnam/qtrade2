# Prints the IPv4 of the servers we just created when the deploy
# is finished on the terminal
output "qtrade_postgres_node_ip" {
  value = latitudesh_server.qtrade_metrics_node.primary_ipv4
}
