# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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