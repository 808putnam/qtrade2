# Prints the IPv4 of the servers we just created when the deploy
# is finished on the terminal
output "qtrade_nautilus_node_ip" {
  value = latitudesh_server.qtrade_nautilus_node.primary_ipv4
}
