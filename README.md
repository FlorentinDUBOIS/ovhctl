# ovhctl

> A command line interface to interact with the ovhcloud api.

## Installation

To install the command line interface, you will need a rust environment and then
running the following command:

```shell
$ cargo install --git https://github.com/FlorentinDUBOIS/ovhctl.git
```

This will put the binary in the directory `$HOME/.cargo/bin`, if you use the default
installation of the rust eco-system. 

## Usage

```shell
$ ovhctl -h
ovhctl 0.1.2
Commands parsed from the command line

USAGE:
    ovhctl [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -t               Validate the configuration
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v               Increase log verbosity

OPTIONS:
    -c, --config <config>    Path to the configuration file

SUBCOMMANDS:
    cloud        Manage cloud resources across the ovh api
    connect      Login to the ovh api
    dedicated    Manage dedicated infrastructure
    domain       Manage domain across the ovh api
    help         Prints this message or the help of the given subcommand(s)
```

## Get in touch

- [@FlorentinDUBOIS](https://twitter.com/FlorentinDUBOIS)
