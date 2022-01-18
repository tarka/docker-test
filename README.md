# Docker Test Helper

This is a small helper library to build and run Rust applications inside a
Docker container. Its main use is to test applications that require testing with
specific file paths or permissions that would be inappropriate to perform
locally on a development machine (e.g. `chmod` to root or enabling `setuid`).

Right now it is somewhat specific to my use-cases, but may be of use to others.

See https://github.com/tarka/rsu/blob/main/tests/lib.rs for an example of use.
