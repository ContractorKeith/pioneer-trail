# Balance report

`balance-report` runs the reference headless player repeatedly and writes CSV tables to standard output.

The reference buys a fixed outfit, chooses the first affordable event response, uses ferries when
affordable, and treats illness whenever medicine or a living trained medic is available. Below
75 pounds of food it hunts with real ammunition, then fishes by rivers or forages when ammunition
is exhausted. It makes at most four consecutive food-gathering attempts before moving again.
Hunts use the real text-mode target controls; the raft pilot holds center in the real collision world.
This is a reproducible policy, not an optimal player. It does not adapt rations, rest, or route choice
to every situation; low-cash occupations remain substantially harder.

```sh
cargo run --release -p pioneer-tools --bin balance-report -- --runs 10000
```

By default it reports every occupation available in Oregon/1848 for departure months March through July. `soldier` is included only for 1866. Use `--occupation`, `--month`, `--trail`, `--era`, `--seed`, or `--runs` to narrow the report. Each primary CSV row is one occupation/month scenario; the two following tables record death context and arrival months. `unknown_<ailment>_present` means one ailment was present at death, not that it caused the death. Other deaths are labeled `unknown`.
