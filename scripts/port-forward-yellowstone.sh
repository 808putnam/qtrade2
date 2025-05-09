#!/bin/bash

# Notes:
# 1. On mac:  
#    ssh -NT -L 10000:localhost:10000 wmcroberts3@185.26.11.237
# 2. We expect the public ip for mac to remain the same
sshpass -p '!518Ajb3990468' ssh -o StrictHostKeyChecking=no -NT -L 10000:localhost:10000 billmcroberts@100.1.255.13
