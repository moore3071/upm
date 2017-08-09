# Universal Package Manager(upm)

upm is meant to be a frontend for a number of package managers. Ideal for when:
- a package is offerred by multiple package managers (e.g. Pacman and Python-pip)
- You constantly switch package managers and have to relearn their syntax

upm is written in Rust, and uses your path to determine what package managers are
available. If a supported package manager isn't showing up then double check that
it exists in your path with the expected name.

## Supported package managers

As this project is just starting the package manager list is rather small. You 
can help to expand this list by putting in the data for another package manager.
For more information see contributing.

### Planned to support in initial version

- Pacman
- npm
- pip2
- pip3

### Local versus Global packages
Some package managers like npm allow you to install a package in a project 
directory. This is not planned to be supported in the initial version of upm, 
but should be supported in later versions.

## Contributing
Currently upm is still being scaffolded and properly architected and is not 
ready to add new package managers. Once the design is settled on, a standardized
format will be established for package manager addition.

## FAQ (Frequently Anticipated Questions)
These questions may or may not have been asked already
- Why not use alpm instead of Pacman (or the backend of any other package manager)
  - Commands are run with pacman and like commands to allow the easy addition of
    further package managers via a simplified format.
