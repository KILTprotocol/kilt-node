#!/usr/bin/env python3
"""


requires atleast python 3.6
"""
import argparse
import os
import subprocess
import json
import uuid
import logging

import peregrine_dev_kilt
import peregrine_stg_kilt
import peregrine_stg_relay


PERE_DEV_KILT="dev-specs/kilt-parachain/peregrine-dev-kilt.json"
PERE_DEV_RELAY="dev-specs/kilt-parachain/peregrine-dev-relay.json"

PERE_STG_KILT="dev-specs/kilt-parachain/peregrine-stg-kilt.json"
PERE_STG_RELAY="dev-specs/kilt-parachain/peregrine-stg-relay.json"

PERE_KILT="dev-specs/kilt-parachain/peregrine-kilt.json"
PERE_RELAY="dev-specs/kilt-parachain/peregrine-relay.json"


def check_process(process: subprocess.CompletedProcess):
    if process.returncode != 0:
        print("Error while executing:", process.args)
        print("Got stderr:")
        print(process.stderr.decode("utf-8"))
        print("Got stdout:")
        print(process.stdout.decode("utf-8"))
        raise RuntimeError


def reset_spec(tmp_dir, docker_img, plain_file, out_file, update_spec):
    process = subprocess.run(["docker", "run", docker_img, "build-spec", "--runtime",
                              "peregrine", "--chain", "dev", "--disable-default-bootnode"],
                             capture_output=True)
    check_process(process)

    in_json = json.loads(process.stdout)
    update_spec(in_json)

    plain_path = os.path.join(tmp_dir, plain_file)
    with open(plain_path, "w") as f:
        json.dump(in_json, f)

    process = subprocess.run(["docker", "run", "-v", f"{tmp_dir}:/data/", docker_img, "build-spec", "--runtime",
                              "peregrine", "--chain", os.path.join("/data/", plain_file), "--disable-default-bootnode"],
                             capture_output=True)
    check_process(process)

    with open(out_file, "wb") as f:
        f.write(process.stdout)


if __name__ == "__main__":
    logging.basicConfig(format='%(asctime)s:%(levelname)s: %(message)s',
                        datefmt='%m-%d-%Y %H:%M:%S', level=logging.DEBUG)

    parser = argparse.ArgumentParser(
        description=("Reset the chainspec for our networks."
                     "VERIFY THAT THE SPEC IS CORRECT AFTER USE!!"
                     "Make sure that the current directory is the project root."),
        epilog="")
    parser.add_argument("--image", "-i", dest="image", required=True,
                        help="docker image to use for building chain spec")

    parser.add_argument("--westend", "-w", action="store_true", dest="westend",
                        help="reset the westend chainspec")

    parser.add_argument("--peregrine", "-p", action="store_true", dest="peregrine",
                        help="reset the peregrine chainspec")
    parser.add_argument("--peregrine-relay", "-r", action="store_true", dest="peregrine_relay",
                        help="reset the peregrine relaychain chainspec")

    parser.add_argument("--peregrine-stg", "-s", action="store_true", dest="peregrine_stg",
                        help="reset the peregrine staging chainspec")
    parser.add_argument("--peregrine-relay-stg", action="store_true", dest="peregrine_relay_stg",
                        help="reset the peregrine staging chainspec")

    parser.add_argument("--peregrine-dev", action="store_true", dest="peregrine_dev",
                        help="reset the peregrine staging chainspec")
    parser.add_argument("--peregrine-relay-dev", action="store_true", dest="peregrine_relay_dev",
                        help="reset the peregrine staging chainspec")


    args = parser.parse_args()
    tmp_dir = os.path.join("/tmp/", str(uuid.uuid1()))
    os.mkdir(tmp_dir)

    if args.westend:
        pass

    if args.peregrine:
        pass
        # reset_spec(
        #     tmp_dir, args.image, "peregrine_dev_kilt.plain.json",
        #     PERE_DEV_KILT, peregrine_dev_kilt.update_spec
        # )

    if args.peregrine_dev:
        reset_spec(
            tmp_dir, args.image, "peregrine_dev_kilt.plain.json",
            PERE_STG_KILT, peregrine_dev_kilt.update_spec
        )

    if args.peregrine_stg:
        reset_spec(
            tmp_dir, args.image, "peregrine_stg.plain.json",
            PERE_STG_KILT, peregrine_stg_kilt.update_spec
        )

    if args.peregrine_relay_stg:
        reset_spec(
            tmp_dir, args.image, "peregrine_stg_relay.plain.json",
            PERE_STG_RELAY, peregrine_stg_relay.update_spec
        )
