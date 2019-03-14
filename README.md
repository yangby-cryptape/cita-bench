# [Unofficial] CITA-Bench

[![License]](#license)
[![Travis CI]](https://travis-ci.com/yangby-cryptape/cita-bench)

A simple tool to benchmark [CITA].

And it's a simple example to show how to use [CITA Rust SDK].

[License]: https://img.shields.io/badge/License-Apache--2.0%20OR%20MIT-blue.svg
[Travis CI]: https://img.shields.io/travis/com/yangby-cryptape/cita-bench.svg
[CITA]: https://github.com/cryptape/cita
[CITA Rust SDK]: https://github.com/cryptape/cita-common/tree/develop/cita-web3

## Usage

**Notice: All data below are meaningless. Those tests ran in a very old laptop.**

### Help

- Command:

  ```bash
  cita-bench --help
  ```

### Send empty transactions

- Command:

  ```bash
  cita-bench \
      --quiet \
      --node "${IP1}:${PORT1},${IP2}:${PORT2},${IP3}:${PORT3}" \
      --protocol http \
      --thread 10 \
      --amount 100 \
      --interval 10 \
      --category sendRawTransaction
  ```

- **[Preview Only]** Result:

  ```
  ----    ----    ----    ----    ----    ----    ----    ----    ----    ----
                          Benchmark [sendRawTransaction]

  Node                 Amount  Thread  Success  Failure  Missing  SuccCostAvg (ms)
  xxx.xxx.xxx.1:xxxx1  1000    10      1000     0        0        56.300945
  xxx.xxx.xxx.2:xxxx2  1000    10      1000     0        0        56.100531
  xxx.xxx.xxx.3:xxxx3  1000    10      1000     0        0        61.210867

                          Total Cost :     7558.764 ms
                          Total Succ :         3000 tx
                              TPS    :      396.890 tx/s
  ----    ----    ----    ----    ----    ----    ----    ----    ----    ----
  ```

### Query the latest block height

- Command:

  ```bash
  cita-bench \
      --quiet \
      --node "${IP1}:${PORT1},${IP2}:${PORT2},${IP3}:${PORT3}" \
      --protocol http \
      --thread 10 \
      --amount 100 \
      --interval 10 \
      --category blockNumber
  ```

- **[Preview Only]** Result:

  ```
  ----    ----    ----    ----    ----    ----    ----    ----    ----    ----
                          Benchmark [blockNumber]

  Node                 Amount  Thread  Success  Failure  Missing  SuccCostAvg (ms)
  xxx.xxx.xxx.1:xxxx1  1000    10      1000     0        0        60.353403
  xxx.xxx.xxx.2:xxxx2  1000    10      1000     0        0        59.958938
  xxx.xxx.xxx.3:xxxx3  1000    10      1000     0        0        62.318052

                          Total Cost :     7403.398 ms
                          Total Succ :         3000 tx
                              TPS    :      405.219 tx/s
  ----    ----    ----    ----    ----    ----    ----    ----    ----    ----
  ```

## License

Licensed under either of [Apache License, Version 2.0] or [MIT License], at
your option.

[Apache License, Version 2.0]: LICENSE-APACHE
[MIT License]: LICENSE-MIT
