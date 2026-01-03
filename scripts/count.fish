#!/usr/bin/env fish

# Wrapper script to run count.py with -dfc flags
python3 (dirname (status --current-filename))/count.py -dfc $argv
