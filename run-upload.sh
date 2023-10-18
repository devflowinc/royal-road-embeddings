#!/bin/bash

# Start processes
for i in {1..6}; do
    python ./upload.py $i &
done

# Wait for user input or a signal to kill the processes
echo "Processes are running. Press Enter to kill them all."
read -p "Press Ctrl+C to stop this script."

# Kill processes
pkill -f "python ./upload.py"
