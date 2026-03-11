# pairing
This is the repository accompanying the bachelor's thesis "Comparing differences in pairing strategies of genetic algorithms based on testing with NP-hard flow assignment problems" written at IU International University.

## Dependencies
The programming language Rust (https://rust-lang.org) and its tooling is required.
Trunk is required for the dashboard (https://crates.io/crates/trunk).

## Structure
All code neccessary for experiments can be found in the folder `simulation/`.
Experiments can be started from within the `simulation/` folder with `cargo run --release`.
Results will appear in folders starting with a dot.
To synchronize results to the dashboard, the `shared/sync.sh` script can be used.

Displaying plots can be done with the `dashboard\`.
It can be started with `trunk serve --release` and will only show results that were previously synchronized with the `shared/sync.sh` script.

`shared\` contains code, that is reused across dasboard and simulation.

## Further content
There are scripts and further files (`**/*.{sh,service.template}`) to automatically deploy and start the simulations, dashboard and automatic result updates on a linux machine.
