# A-Train

A-Train is the official Autoscan trigger that listens for changes within Google Drive.
It is the successor of Autoscan's Bernard trigger, which unfortunately contains enough logic errors to prompt a rewrite.

- Supports **Shared Drives**
- **Service Account**-based authentication
- Does not support My Drive
- Does not support encrypted files
- Does not support alternative authentication methods

## Prerequisites

A-Train works exclusively through [Shared Drives](https://support.google.com/a/answer/7212025) and [Service Accounts](https://cloud.google.com/iam/docs/service-accounts).
Shared Drives can only be created on GSuite / Google Workspace accounts.

First, you must [create a Service Account](https://cloud.google.com/iam/docs/creating-managing-service-accounts#iam-service-accounts-create-console) and [create a JSON key](https://cloud.google.com/iam/docs/creating-managing-service-account-keys#creating_service_account_keys).
Afterwards, if you do not have one already, [create a Shared Drive within Google Drive](https://support.google.com/a/users/answer/9310249) and then add the email address of the Service Account to the Shared Drive (with `Reader` permission).

Finally, activate the Drive API in the Google Cloud Project where you created the Service Account.

## Installation

A-Train has multiple installation options:

- Building A-Train from source (requires Rust to be installed)
- Downloading the pre-compiled binary for your platform
- Using the Docker image

### Docker

The Docker image requires you to mount a volume with the A-Train configuration file (`a-train.toml`) and the Service Account key file (`account.json`) to `/data`.

```bash
docker run -d \
    --name a-train \
    --volume /path/to/data:/data \
    --restart unless-stopped \
    ghcr.io/m-rots/a-train
```

### From Source

Requires the latest stable version of [Rust](https://www.rust-lang.org/tools/install) to be installed.

```bash
cargo install --git https://github.com/m-rots/a-train --branch main
```

## Configuration

The configuration file should be named `a-train.toml`.

- If you have installed A-Train from source or are using the binary, then A-Train will look for the `a-train.toml` and `account.json` files within the current working directory.
- If you're using A-Train within Docker, then a directory should be mounted to `/data` containing the `a-train.toml` and `account.json` files.

```toml
# a-train.toml
[autoscan]
# Replace the URL with your Autoscan URL.
url = "http://localhost:3030"
username = "hello there"
password = "general kenobi"

[drive]
# Path to the Service Account key file,
# relative to the configuration file.
account = "./account.json"
# One or more Shared Drive IDs
drives = ["0A1xxxxxxxxxUk9PVA", "0A2xxxxxxxxxUk9PVA"]
```

### How to get the ID of a Shared Drive?

1. Open Google Drive in your preferred browser.
2. Click on Shared Drives on the left.
3. Double click on the Shared Drive you want to add.
4. You should now see all the files and folders at the root of this Shared Drive.
5. Now click on your browser's URL bar and copy the most-right value.
It should have a structure similar to `0A1xxxxxxxxxUk9PVA`.