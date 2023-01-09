# `typed-arena-benchmarks`

## Why is this a separate workspace?

This is in a separate workspace to avoid issues with having criterion as a
dev-dependency. Specifically:

1. Criterion and its transitive dependencies have a much higher MSRV. Specifically, high enough that cargo fails to parse their manifest toml when building, so we can't even build anything that requires dev-dependencies, such as tests.

2. Criterion is slow to build. Having it as a dev-dependency means we need to build it in order to run tests. Some of the CI runners are very slow, and this dominates their time. It also slows down local builds for little benefit.

In exchange, the repository setup is slightly weirder, and so users may not realize there are two things to check.
