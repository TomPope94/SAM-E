# SAM-E

Welcome to the SAM-E tool! In here you'll find all the code related to running and building SAM-E.

## What is SAM-E?
SAM-E is a tool that allows you to run AWS serverless architecture in your local environment. How it differs from other tools (i.e. Serverless framework) is that rather than building within a specific framework which translates to AWS cloudformation under-the-hood, SAM-E runs from your Cloudformation templates. This way there is no disconnect between your local environment (i.e. developers) and your live AWS environment (i.e. your devops). 

If you're curious about the name, SAM-E stands for SAM (Serverless Application Model) - E (environment)... pronounced "Sammy".

## What is in this repo?

| Crate | Description |
|-------|-------------|
| [cli](./sam-e-cli/) | The command line interface for SAM-E. All features of the tool should be implemented via a variety of CLI commands & arguments. |
| [invoker](./sam-e-invoker/) | The invoker is repsonsible for running the Lambda runtime among other things. For a full description please see the README in the invoker crate. |
| [types](./sam-e-types/) | The types crate contains all the shared types between the invoker and the CLI. |

## How do I use SAM-E?

To use SAM-E, you'll need to install the CLI. There are a few options for you to do this:

```bash
# Install from source - note: you'll need to drop the binary in your path/bin directory if you want to use globally
cargo install --path sam-e-cli

# Use the build script
./build.sh
```

Once you have the CLI installed, follow the steps highlighted in the [CLI README](./sam-e-cli/) to get started.

