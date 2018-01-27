#! /usr/bin/env sh
pacman --version | sed -n -e 's/^.*Pacman v\([0-9\.]*\).*/\1/p'
