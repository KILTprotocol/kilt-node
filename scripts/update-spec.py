#!/usr/bin/env python3
"""


requires atleast python 3.6
"""
import argparse
import shutil
import subprocess
import json
import uuid
import logging


def build_binary(crate, cargo_args):
    subprocess.run(["cargo", "build", "--release", "-p",
                    crate, *cargo_args], check=True)
    temp_dest = f"/tmp/{crate}-{uuid.uuid1()}"
    shutil.move(f"target/release/{crate}", temp_dest)
    return temp_dest


def build_spec(binary, extra_args):
    subprocess.run([binary, "build-spec", "--disable-default-bootnode", *extra_args], check=True)


if __name__ == "__main__":
    logging.basicConfig(format='%(asctime)s:%(levelname)s: %(message)s',
                        datefmt='%m-%d-%Y %H:%M:%S', level=logging.DEBUG)

    parser = argparse.ArgumentParser(
        description=("Reset the chainspec for our networks."
                     "VERIFY THAT THE SPEC IS CORRECT AFTER USE!!"
                     "Make sure that the current directory is the project root."),
        epilog="")
    parser.add_argument("--westend", "-w", action="store_true", dest="westend",
                        help="reset the westend chainspec")
    parser.add_argument("--peregrine", "-w", action="store_true", dest="peregrine",
                        help="reset the peregrine chainspec")

    args = parser.parse_args()

    if args.westend:
        pass

    if args.peregrine:
        pass
