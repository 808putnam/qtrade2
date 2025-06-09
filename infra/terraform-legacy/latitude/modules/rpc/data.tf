# Just put in place as an example of latitude.sh data sources, not used in the code yet
# data "latitudesh_region" "region" {
#   slug = "NYC"
# }
# 
# data "latitudesh_plan" "region" {
#   name = "c3.large.x86"
# }

$ terraform import latitudesh_ssh_key.ssh_key projectID:sshKeyID
