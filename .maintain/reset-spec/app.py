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

logger = logging.getLogger(__name__)

WILT_KILT = "dev-specs/kilt-parachain/kilt-westend.json"
SPIRITNET_KILT = "nodes/parachain/res/spiritnet.json"

PERE_DEV_KILT = "dev-specs/kilt-parachain/peregrine-dev-kilt.json"
PERE_DEV_RELAY = "dev-specs/kilt-parachain/peregrine-dev-relay.json"

PERE_STG_KILT = "dev-specs/kilt-parachain/peregrine-stg-kilt.json"
PERE_STG_RELAY = "dev-specs/kilt-parachain/peregrine-stg-relay.json"

PERE_KILT = "dev-specs/kilt-parachain/peregrine-kilt.json"
PERE_RELAY = "dev-specs/kilt-parachain/peregrine-relay.json"


def check_process(process: subprocess.CompletedProcess):
    if process.returncode != 0:
        logger.error("Error while executing:", process.args)
        logger.error("Got stderr:")
        logger.error(process.stderr.decode("utf-8"))
        logger.error("Got stdout:")
        logger.error(process.stdout.decode("utf-8"))
        raise RuntimeError


def make_custom_spec(tmp_dir, docker_img, plain_file, out_file, update_spec, spec, runtime=None):
    """Build a custom spec by exporting a chain spec and customize it using a python script.
    """
    cmd_plain_spec = ["docker", "run", docker_img, "build-spec",
                      "--chain", spec, "--disable-default-bootnode"]

    if runtime is not None:
        cmd_plain_spec += ["--runtime", runtime]

    process = subprocess.run(cmd_plain_spec, capture_output=True)
    check_process(process)

    in_json = json.loads(process.stdout)
    update_spec(in_json)

    plain_path = os.path.join(tmp_dir, plain_file)
    with open(plain_path, "w") as f:
        json.dump(in_json, f)

    cmd_raw_spec = ["docker", "run", "-v", f"{tmp_dir}:/data/", docker_img, "build-spec",
                    "--chain", os.path.join("/data/", plain_file), "--disable-default-bootnode"]
    if runtime is not None:
        cmd_raw_spec += ["--runtime", runtime]

    process = subprocess.run(cmd_raw_spec, capture_output=True)
    check_process(process)

    with open(out_file, "wb") as f:
        f.write(process.stdout)


def make_native(docker_img, out_file, chain, runtime):
    """Build a custom spec by exporting a chain spec and customize it using a python script.
    """
    process = subprocess.run(["docker", "run", docker_img, "build-spec", "--runtime", runtime, "--chain", chain, "--raw"],
                             capture_output=True)
    check_process(process)

    with open(out_file, "wb") as f:
        f.write(process.stdout)


if __name__ == "__main__":
    import peregrine_kilt
    import peregrine_relay
    import peregrine_dev_kilt
    import peregrine_stg_kilt
    import peregrine_stg_relay

    logging.basicConfig(format='%(asctime)s:%(levelname)s: %(message)s',
                        datefmt='%m-%d-%Y %H:%M:%S', level=logging.DEBUG)

    parser = argparse.ArgumentParser(
        description=("Reset the chainspec for our networks."
                     "VERIFY THAT THE SPEC IS CORRECT AFTER USE!!"
                     "Make sure that the current directory is the project root."),
        epilog="")
    parser.add_argument("--image", "-i", dest="image", required=True,
                        help="docker image to use for building chain spec")

    parser.add_argument("--wilt", "-w", action="store_true", dest="wilt",
                        help="reset the wilt (westend) chainspec")

    parser.add_argument("--spiritnet", "-s", action="store_true", dest="spiritnet",
                        help="reset the spiritnet chainspec")

    parser.add_argument("--peregrine", "-p", action="store_true", dest="peregrine",
                        help="reset the peregrine chainspec")
    parser.add_argument("--peregrine-relay", "-r", action="store_true", dest="peregrine_relay",
                        help="reset the peregrine relaychain chainspec")

    parser.add_argument("--peregrine-stg", action="store_true", dest="peregrine_stg",
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

    if args.wilt:
        make_native(args.image, WILT_KILT, "wilt-new", "spiritnet")

    if args.spiritnet:
        make_native(args.image, SPIRITNET_KILT, "spiritnet-new", "spiritnet")

    if args.peregrine:
        make_custom_spec(
            tmp_dir, args.image, "peregrine_dev_kilt.plain.json",
            PERE_KILT, peregrine_kilt.update_spec, "dev", "peregrine"
        )

    if args.peregrine_relay:
        try:
            make_custom_spec(
                tmp_dir, args.image, "peregrine_relay.plain.json",
                PERE_RELAY, peregrine_relay.update_spec, "westend-local"
            )
        except KeyError as e:
            raise RuntimeError(
                "Could not customize spec. Make sure to use a relay chain image.") from e

    if args.peregrine_dev:
        make_custom_spec(
            tmp_dir, args.image, "peregrine_dev_kilt.plain.json",
            PERE_STG_KILT, peregrine_dev_kilt.update_spec, "dev", "peregrine"
        )

    if args.peregrine_stg:
        make_custom_spec(
            tmp_dir, args.image, "peregrine_stg.plain.json",
            PERE_STG_KILT, peregrine_stg_kilt.update_spec, "dev", "peregrine"
        )

    if args.peregrine_relay_stg:
        try:
            make_custom_spec(
                tmp_dir, args.image, "peregrine_stg_relay.plain.json",
                PERE_STG_RELAY, peregrine_stg_relay.update_spec, "westend-local"
            )
        except KeyError as e:
            raise RuntimeError(
                "Could not customize spec. Make sure to use a relay chain image.") from e
