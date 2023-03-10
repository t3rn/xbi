# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.3.7] - 2023-03-10
### :sparkles: New Features
- [`b29d569`](https://github.com/t3rn/xbi/commit/b29d56931ef3023329acc467ee1c041c04e8b9ed) - **channel**: introduce async types for the channel *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*
- [`9cc9b4e`](https://github.com/t3rn/xbi/commit/9cc9b4e33839025ee63efe84bb5383f225ce36e5) - **channel**: implement async queue *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*
- [`51f3436`](https://github.com/t3rn/xbi/commit/51f3436b7abfcd604d8c4985de4d7bb0e99e6ef2) - **portal**: call queue on intervals *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*
- [`4d0bfc4`](https://github.com/t3rn/xbi/commit/4d0bfc4488bfaf9a7c82537fd1d5aa04a7136aa6) - **portal**: pass a status to an error *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*
- [`4aaa64a`](https://github.com/t3rn/xbi/commit/4aaa64a70fe3c22cee4cd4b65e3f8d56a0ea8307) - **portal**: async receiver *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*
- [`749de02`](https://github.com/t3rn/xbi/commit/749de02fbe2353d90bca571915c84db33dbee0c1) - **format**: introduce a received field for the timesheet *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*
- [`67192bc`](https://github.com/t3rn/xbi/commit/67192bc1ea0a09c1a3c98e50d57b4234ccbfcdf2) - **channel**: asset reservation with first pass approach *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*
- [`0058a24`](https://github.com/t3rn/xbi/commit/0058a2409ef2f2616c76cf5badeb9ef924e0c903) - **portal**: origin should always be enriched *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*
- [`c89a985`](https://github.com/t3rn/xbi/commit/c89a9850f9e767d6c8179cc4b145c19f211c445e) - **portal**: aggregates should be calculated by the handler *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*
- [`c119fec`](https://github.com/t3rn/xbi/commit/c119fecc3c3291f04654d1c2265f576a9897f708) - **channel**: testable user payment for messages *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*
- [`4cd53fb`](https://github.com/t3rn/xbi/commit/4cd53fb2706ed786eb9b1b802eeda6f169d0d1a6) - charge for notifications *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*

### :white_check_mark: Tests
- [`31fb9aa`](https://github.com/t3rn/xbi/commit/31fb9aaec494bc7ace7a6d0973bf58d3db2177b5) - **portal**: process the queue in async tests *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*
- [`cc05015`](https://github.com/t3rn/xbi/commit/cc0501577363a42cc408672352be94a681198113) - **portal**: test that the async channel sends the right signals *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*
- [`eee5dbb`](https://github.com/t3rn/xbi/commit/eee5dbb22afdd585a886c81a017aca67c3e79d6d) - add asset macro assertions *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*
- [`138f12b`](https://github.com/t3rn/xbi/commit/138f12b3fd2a6079fada4cff1b9f2d701c6338b1) - fix testing spaghetti for accounts *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*
- [`a09da93`](https://github.com/t3rn/xbi/commit/a09da93b6a5cd72d517a7a2d5fa1ed6c7155b3b9) - make tests compile *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*

### :wrench: Chores
- [`408b3a2`](https://github.com/t3rn/xbi/commit/408b3a2aab19e6aa1058afea31f92f096ab926a6) - **queue**: add logging to queue *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*
- [`8ae236d`](https://github.com/t3rn/xbi/commit/8ae236d0321b5c7def1dcc88ea629981dd80c856) - update log targets *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*
- [`b51e962`](https://github.com/t3rn/xbi/commit/b51e962af248fd606f9ba1e222ce0961a91f0e41) - clean pallet and move to impls *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*
- [`997e423`](https://github.com/t3rn/xbi/commit/997e42334b39454cc739603b823f9f62b44ac1cb) - clippy *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*


## [v0.3.6] - 2023-03-01
### :sparkles: New Features
- [`edcb5db`](https://github.com/t3rn/xbi/commit/edcb5db0043be65b8cba0b0b763065674e926041) - store xbi responses in sync channel *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*

### :white_check_mark: Tests
- [`ef7c723`](https://github.com/t3rn/xbi/commit/ef7c723a1f0e33e3f105416970cdd555cd2b62fc) - introduce low level unhappy path test *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*

### :wrench: Chores
- [`f1547be`](https://github.com/t3rn/xbi/commit/f1547be590af386b6896d290bfa2d8424fbaf9d0) - make the queue use the same logic *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*
- [`f95f84f`](https://github.com/t3rn/xbi/commit/f95f84fe60d6367feddef377c150a06e98e31cb9) - **asset-registry**: clippy lints *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*


## [v0.3.5] - 2023-03-01
### :bug: Bug Fixes
- [`0af6565`](https://github.com/t3rn/xbi/commit/0af6565f782bba5db1d9d35a7b7940c2cc533091) - scabi module error when publishing *(commit by [@AwesomeIbex](https://github.com/AwesomeIbex))*


[v0.3.5]: https://github.com/t3rn/xbi/compare/v0.3.4...v0.3.5

## [v0.3.4] - 2023-03-01
### :boom: BREAKING CHANGES
- due to [`7fd8d50`](https://github.com/t3rn/xbi/commit/7fd8d5027476a0c05cb65b934edcaac137ff783f) - timestamps can be progressed by an enum *(commit by @AwesomeIbex)*:

  timestamps can be progressed by an enum

- due to [`7576d2b`](https://github.com/t3rn/xbi/commit/7576d2bfe4024e0f56708bef213e561f7ffddd38) - remove id from the result *(commit by @AwesomeIbex)*:

  we handle results in a different way so result has been removed from the codec

- due to [`03ffbb5`](https://github.com/t3rn/xbi/commit/03ffbb58c7ef7ad92d4aa98572197e5c962c14a3) - hide the timesheet from requesting users *(commit by @AwesomeIbex)*:

  hide the timesheet from requesting users

- due to [`f7c8e50`](https://github.com/t3rn/xbi/commit/f7c8e504a1cf35e4edb72f46dab4f2e174df1678) - generate ids on user requests *(commit by @AwesomeIbex)*:

  this removes the ability to provide an id when sending an xbi message


### :sparkles: New Features
- [`f7c8e50`](https://github.com/t3rn/xbi/commit/f7c8e504a1cf35e4edb72f46dab4f2e174df1678) - **format**: generate ids on user requests *(commit by @AwesomeIbex)*

### :recycle: Refactors
- [`7fd8d50`](https://github.com/t3rn/xbi/commit/7fd8d5027476a0c05cb65b934edcaac137ff783f) - **format**: timestamps can be progressed by an enum *(commit by @AwesomeIbex)*
- [`7576d2b`](https://github.com/t3rn/xbi/commit/7576d2bfe4024e0f56708bef213e561f7ffddd38) - **codec**: remove id from the result *(commit by @AwesomeIbex)*
- [`03ffbb5`](https://github.com/t3rn/xbi/commit/03ffbb58c7ef7ad92d4aa98572197e5c962c14a3) - **format**: hide the timesheet from requesting users *(commit by @AwesomeIbex)*

[v0.3.4]: https://github.com/t3rn/xbi/compare/v0.3.3...v0.3.4
[v0.3.6]: https://github.com/t3rn/xbi/compare/v0.3.5...v0.3.6
[v0.3.7]: https://github.com/t3rn/xbi/compare/v0.3.6...v0.3.7