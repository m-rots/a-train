import click
import gzip
import os
import shutil


targets = {
    "linux/amd64": "x86_64-unknown-linux-musl",
    "linux/arm64": "aarch64-unknown-linux-musl",
    "linux/arm/v7": "armv7-unknown-linux-musleabihf",
    "linux/arm/v6": "arm-unknown-linux-musleabihf",
}


@click.command()
@click.argument("platform", type=str)
@click.option(
    "-d", "--directory", "directory", type=click.Path(exists=True, dir_okay=True)
)
def copy_binary(directory: str, platform: str):
    target = targets[platform]
    src = f"a-train-{target}.gz"
    dst = "a-train"

    if directory is not None:
        os.chdir(directory)

    # Decompress the gz
    with gzip.open(src, "rb") as f_in:
        with open(dst, "wb") as f_out:
            shutil.copyfileobj(f_in, f_out)

    print(f"Decompressed {src!r} into {dst!r}")


if __name__ == "__main__":
    copy_binary()