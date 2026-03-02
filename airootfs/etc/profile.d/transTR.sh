#!/bin/sh
# Add transTR scripts to the system PATH
if [ -d "/usr/local/bin/transTR" ]; then
    export PATH="$PATH:/usr/local/bin/transTR"
fi
