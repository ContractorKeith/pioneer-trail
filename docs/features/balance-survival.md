# Survival-aware balance — v0.1.0

Source: `a89abb8`, release `v0.1.0`. Each scenario uses seeds 0–9,999 on Oregon,
normal difficulty; Soldier uses 1866, all other occupations use 1848. Months 3–7
are March–July. All nine report processes completed successfully: 450,000 runs.

The [reference policy](balance.md) uses real hunting/rafting controls and bounded food gathering,
but does not adapt rations, pace, rest, or routing. These are policy results, not predictions of
human win rates. Every scenario has arrivals; low-cash occupations are difficult with this outfit
and strategy. Banker/March is the recommended first journey (99.88% arrived with three survivors
under this reference). The [shopping-only baseline](balance-shopping-baseline.md) records a
separate 450,000 runs without survival actions; this comparison changes policy, not game rules.

`mean_days` and `mean_score` include all runs, including losses.

```csv
occupation,month,runs,arrivals,three_survivors,mean_days,mean_score
banker,3,10000,9995,9988,215.62,3181.86
banker,4,10000,9999,9993,215.44,3093.62
banker,5,10000,9997,9992,211.99,2993.66
banker,6,10000,9989,9980,213.69,2973.65
banker,7,10000,9928,9894,226.99,2938.19
blacksmith,3,10000,2999,1442,226.89,811.26
blacksmith,4,10000,2834,1304,227.26,737.11
blacksmith,5,10000,3578,1815,221.89,919.87
blacksmith,6,10000,1480,478,230.55,361.00
blacksmith,7,10000,1670,571,240.24,398.56
carpenter,3,10000,1577,546,230.97,399.23
carpenter,4,10000,1572,532,229.80,396.89
carpenter,5,10000,1462,478,226.75,360.75
carpenter,6,10000,1704,584,214.42,435.99
carpenter,7,10000,1209,402,209.45,365.97
doctor,3,10000,9993,9926,214.78,4366.84
doctor,4,10000,9994,9970,215.05,4352.89
doctor,5,10000,9999,9992,211.85,4321.10
doctor,6,10000,9948,9782,212.45,4027.10
doctor,7,10000,9610,8793,223.95,3447.11
farmer,3,10000,2034,569,221.70,870.83
farmer,4,10000,1890,514,216.29,820.62
farmer,5,10000,1882,515,213.24,803.13
farmer,6,10000,2046,580,218.46,863.66
farmer,7,10000,2252,676,223.19,939.28
hunter,3,10000,1020,172,219.61,434.83
hunter,4,10000,874,138,210.15,397.25
hunter,5,10000,879,116,206.43,387.09
hunter,6,10000,1797,382,231.44,619.30
hunter,7,10000,1852,432,231.52,645.48
merchant,3,10000,9995,9988,215.62,3126.61
merchant,4,10000,9998,9992,215.45,3040.20
merchant,5,10000,9998,9993,211.98,2943.96
merchant,6,10000,9992,9986,213.77,2922.06
merchant,7,10000,9910,9870,226.85,2872.53
preacher,3,10000,1572,650,243.06,661.21
preacher,4,10000,1397,598,234.48,624.48
preacher,5,10000,1266,556,228.92,580.27
preacher,6,10000,1602,749,238.20,687.47
preacher,7,10000,1588,749,237.21,684.38
soldier,3,10000,1909,703,229.11,468.98
soldier,4,10000,1862,640,227.70,450.49
soldier,5,10000,1775,584,225.65,411.73
soldier,6,10000,1574,518,211.50,414.63
soldier,7,10000,1053,329,206.43,337.92
```

