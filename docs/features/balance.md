# Balance report

`balance-report` runs the reference headless player repeatedly and writes CSV tables to standard output.

```sh
cargo run --release -p pioneer-tools --bin balance-report -- --runs 10000
```

By default it reports every occupation available in Oregon/1848 for departure months March through July. `soldier` is included only for 1866. Use `--occupation`, `--month`, `--trail`, `--era`, `--seed`, or `--runs` to narrow the report. Each primary CSV row is one occupation/month scenario; the two following tables record observed death causes and arrival months. A death receives an ailment label only when the observer sees exactly one ailment on the party member at health zero; otherwise it is `unknown`.
