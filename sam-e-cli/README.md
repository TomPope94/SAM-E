# SAM-E-CLI

SAM-E comes with a command line interface (CLI) that allows you to interact with the tool. The CLI is the main way to interact with SAM-E and all features of the tool should be implemented via a variety of CLI commands & arguments.

## Prerequisites

There are a number of dependencies you need before you can use the SAM-E. Please ensure you have these installed first:
* Rust - you can install this via [rustup](https://rustup.rs/)
* Docker - you can install this via [Docker Desktop](https://www.docker.com/products/docker-desktop)

## Installation

To install the CLI, you will need to build from source. To do this, you will need to have Rust installed on your machine. 

Once installed, you can build the CLI by running the following command in the `sam-e-cli` directory:

```bash
cargo build --release
```

The CLI will be available in the `target/release` directory. However, for ease of use, it is recommended to copy the binary to your bin directory.

```bash
cp target/release/sam-e-cli /usr/local/bin/sam-e
```

For ease of use, you can also use the build script to build the CLI from the root SAM-E directory:

```bash
./build.sh
```

Note: this may be different if operating on a Windows machine.
Note*2: You may need to run the command with `sudo` if you don't have the correct permissions.

## Usage

The CLI has a number of commands that you can use to interact with SAM-E. To see a list of all available commands, you can run the following command:

```bash
sam-e --help
```

For help within each individual sub-command run `sam-e <command> --help`. A help utility should be available at each level of the CLI.

### Basic use of environment commands

SAM-E works by creating a `.sam-e` directory in the root of your project. This directory contains all the necessary files to run your Lambda function locally. To create this directory, you can run the following command:

```bash
sam-e environment init
```

You will then be prompted with which cloudformation templates you would like to use. This works by recursively checking all child directories for yaml files. Note: the consequence of this is that your cloudformation templates need to a) exist and b) be yaml files.

Once the configuration is complete, you can now build out your environment as required by running:

```bash
sam-e environment build
```

This will first prompt you to select which Lambda functions you would like to build (if any available). Once selected, the CLI will build the necessary files in the `.sam-e` directory.

For context on what this step does:
* Adds Lambda information to the sam-e-config
* Finds infrastructure resources in the cloudformation templates and adds to sam-e-config
* Uses the config to build a docker-compose file containing all required services

Now the environment is built, you can run your E2E serverless project locally by running:

```bash
sam-e environment start
```

This will now prompt you to select whether to run `Infrastructure`, `Functions` or `Frontend`. Once selected, the CLI will run the necessary docker-compose services to run your project locally.

The reason for this separation is so you can pause and restart individual services. Otherwise, you'll be stopping and starting all the services on every save of a lambda... no thank you!

### Changing the environment

If you would like to change any details of the environment (such as environment variables) you can edit the sam-e-config file in the `.sam-e` directory (CLI command to be created). 

Once happy with your edit, you can rebuild the environment by running:

```bash
sam-e environment rebuild
```
This is similar to the `build` command, but will make sure your environment variables chosen prior are not overwritten.
