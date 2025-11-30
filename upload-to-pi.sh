#!/bin/bash

cross build --release --target aarch64-unknown-linux-gnu ; scp target/aarch64-unknown-linux-gnu/release/bonnetje-daemon root@10.0.0.20:/usr/local/bin/bonnetje-daemon
ssh root@10.0.0.20 -C "systemctl stop bonnetje"
ssh root@10.0.0.20 -C "chown root:root /usr/local/bin/bonnetje-daemon"
ssh root@10.0.0.20 -C "systemctl restart bonnetje"