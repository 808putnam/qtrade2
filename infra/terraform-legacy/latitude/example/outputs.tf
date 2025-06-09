# Prints the IPv4 of the servers we just created when the deploy
# is finished on the terminal
output "example_node_ip" {
  value = latitudesh_server.example_node.primary_ipv4
}
