#!/bin/bash

# Start a tmux session
tmux new-session -s iridium -d
tmux rename-window -t iridium "server"
tmux new-window -t iridium -n "client1"
tmux new-window -t iridium -n "client2"
tmux send-keys -t iridium:server "RUST_BACKTRACE=1 RUST_LOG=debug iridium --node-alias server" C-m
tmux send-keys -t iridium:client1 "RUST_LOG=debug iridium --server-bind-port 2255 --node-alias client1" C-m
tmux send-keys -t iridium:client2 "RUST_LOG=debug iridium --server-bind-port 2256 --node-alias client2" C-m
tmux send-keys -t iridium:server "!start_cluster" C-m
sleep 3
tmux send-keys -t iridium:client1 "!join_cluster localhost 2254" C-m
sleep 3
tmux send-keys -t iridium:client2 "!join_cluster localhost 2254" C-m
sleep 3
tmux attach -t iridium