#!/usr/bin/env python3
"""
Build the raw chainspec.

Update only test networks:
`python3 scripts/update-spec.py -ds`

Update all networks:
`python3 scripts/update-spec.py -tdrs`

!Verify the chainspec after each update!

requires atleast python 3.6
"""
import argparse
import pathlib
import subprocess
import typing
import json
import uuid
import logging


logger = logging.getLogger(__name__)


def build_spec(node, chain, parachain_id=None, raw=False):
    # cmd = ["cargo", "run", "--release", "-p",
    #        node, "--", "build-spec", "--chain", chain]
    cmd = [f"./target/release/{node}", "build-spec",
           "--disable-default-bootnode", "--chain", chain]
    if raw:
        cmd.append("--raw")

    logger.info("exec: '" + " ".join(cmd) + "'")
    process = subprocess.run(" ".join(cmd), shell=True,
                             stdout=subprocess.PIPE, close_fds=True)
    process.check_returncode()
    printed_stuff = process.stdout.decode("utf-8")

    return json.loads(printed_stuff)


def fill_balances(balances, endowed):
    balances["balances"] = [[acc, money] for acc, money in endowed.items()]


def fill_session(session, authorities, session_keys):
    session["keys"] = [[acc, acc, {
        k: mapper(acc) for k, mapper in session_keys.items()
    }] for acc in authorities]

    logger.debug("new sesion keys: ", session)


def set_root(sudo, account):
    sudo["key"] = account


def fill_spec(spec, authorities, session_keys, endowed, root):
    runtime = spec["genesis"]["runtime"]

    try:
        balance_config = runtime["balances"]
    except KeyError:
        balance_config = runtime["palletBalances"]

    fill_balances(balance_config, endowed)
    if authorities and session_keys:
        fill_session(runtime["session"], authorities, session_keys)

    try:
        sudo_config = runtime["sudo"]
    except KeyError:
        sudo_config = runtime["palletSudo"]
    set_root(sudo_config, root)


def build_and_setup_spec(node, authorities, session_keys, endowed, root, outpath, parachain_id=None, extras=None, chain_spec="dev"):
    chain_spec = build_spec(node, chain_spec, parachain_id=parachain_id)

    if extras:
        chain_spec.update(extras)

    fill_spec(chain_spec, authorities, session_keys, endowed, root)

    plain_file = pathlib.Path("/tmp/" + uuid.uuid4().hex)
    with plain_file.open("w") as f:
        json.dump(chain_spec, f)
    logger.info("plain spec stored at %s",
                plain_file.absolute().as_posix())

    raw = build_spec(
        node,
        plain_file.absolute().as_posix(),
        raw=True
    )

    with outpath.open("w") as f:
        json.dump(raw, f, indent="  ")


def setup_spec(authorities, session_keys, endowed, root, outpath: pathlib.Path, chain_file: pathlib.Path, extras=None):
    with chain_file.open("r") as f:
        chain_spec = json.load(chain_spec, f)

    if extras:
        chain_spec.update(extras)

    fill_spec(chain_spec, authorities, session_keys, endowed, root)

    plain_file = pathlib.Path("/tmp/" + uuid.uuid4().hex)
    with plain_file.open("w") as f:
        json.dump(chain_spec, f)
    logger.info("plain spec stored at %s",
                plain_file.absolute().as_posix())


if __name__ == "__main__":
    logging.basicConfig(format='%(asctime)s:%(levelname)s: %(message)s',
                        datefmt='%m-%d-%Y %H:%M:%S', level=logging.DEBUG)

    parser = argparse.ArgumentParser(
        description=("Update the chainspec for our networks."
                     "VERIFY THAT THE SPEC IS CORRECT AFTER USE!!"
                     "Make sure that the current directory is the project root."),
        epilog="Example usage: python3 scripts/update-spec.py -ds")
    parser.add_argument("--devnet", "-d", action="store_true", dest="devnet",
                        help="update devnet spec")
    parser.add_argument("--testnet", "-t", action="store_true", dest="testnet",
                        help="update testnet spec")
    parser.add_argument("--rococo", "-r", action="store_true", dest="rococo",
                        help="update rococo parachain spec")
    parser.add_argument("--roc-stage", "-s", action="store_true", dest="roc_staging",
                        help="update rococo staging parachain spec")
    parser.add_argument("--roc-stage-relay", "-k", dest="roc_staging_relay", type=pathlib.Path,
                        help="update rococo staging relay spec")

    args = parser.parse_args()

    DEFAULT_MONEY = 10 ** 27

    # ##########################################################################
    # ############################      DEVNET      ############################
    # ##########################################################################

    DEV_ALICE = "5Gs55Km8u2168cCsMkmarYcx824HzTYEnAL74NK1jj2RKuGz"
    DEV_BOB = "5CDEZYctMwDKQUXEg42pyuTS7ui3yyTHXtDJBj8YTxctpHnE"
    DEV_CHARLIE = "5EXram9t3NNzSedQegPZu7eM1CPQAJx6DMcrWJ7C9f1M5VXW"
    DEV_FAUCET = "5D5D5fSDUFVvn6RroC85zgaKL93oFv7R332RGwdCdBvAQzUn"

    DEVNET_SPEC_PATH = pathlib.Path.cwd() / "dev-spec/mashnet-node/devnet.json"

    if args.devnet:
        logger.info("update devnet spec")
        build_and_setup_spec(
            "mashnet-node",
            [DEV_ALICE, DEV_BOB, DEV_CHARLIE],
            {
                "aura": lambda x: x,
                "grandpa": lambda x: x,
            },
            {
                DEV_ALICE: DEFAULT_MONEY,
                DEV_BOB: DEFAULT_MONEY,
                DEV_CHARLIE: DEFAULT_MONEY,
                DEV_FAUCET: DEFAULT_MONEY
            },
            DEV_ALICE,
            DEVNET_SPEC_PATH,
            extras={
                "name": "KILT Devnet",
                "id": "kilt_devnet",
                "chainType": "Live",
                "telemetryEndpoints": [["wss://telemetry-backend.kilt.io:8080", 9]]
            }
        )

    # ##########################################################################
    # ########################### KILT_ROC PRODUCTION ##########################
    # ##########################################################################
    if args.rococo:
        logger.info("update rococo spec")
        build_and_setup_spec(
            "kilt-parachain",
            [],
            [],
            {
                "5H1EZkED258UDeTAGf29UCxeqvz32Aqsk1Pxy9QHccEBYwRo": DEFAULT_MONEY,
                "5HVZg213KmoLnjStTL5KUVp4iPGVt3MaAWtAcdxUKmSMXGBH": DEFAULT_MONEY,
            },
            "5H1EZkED258UDeTAGf29UCxeqvz32Aqsk1Pxy9QHccEBYwRo",
            pathlib.Path.cwd() / "nodes/parachain/res/kilt-prod.json",
            parachain_id=12623,
            extras={
                "name": "KILT Collator Rococo",
                "id": "kilt_parachain_rococo",
                "chainType": "Live",
                "bootNodes": [],
                "relay_chain": "rococo_v1",
            }
        )

    # ##########################################################################
    # ############################# RELAY STAGING ##############################
    # ##########################################################################
    if args.roc_staging_relay:
        logger.info("update rococo-staging relay spec")
        val_1_pub = "5CA1Ym7i37qggvsK8D6nMAeRGHZo6gz8oEQvZ5qsvNeZdyqy"
        val_2_pub = "5DjkAWLDGvesjGfvZkcSfaTTZnUUEVgyUFATQv583nqGG1rm"
        val_3_pub = "5HQBuyGFKqqBkoK2cwv4a6s9dEv34cudoMfoT1wxkEn1xcic"

        # ed25519 public keys
        ed_pub = {
            val_1_pub: "5DGcrtir4xLdaBGJSi6zmmmQBHC6RZwPhvvCzNZs6FywsauM",
            val_2_pub: "5Dmib5zkJtPmYkMRhN5Gtv67s2v4wCvXgG7cVha6sR6Sw9wu",
            val_3_pub: "5D3WF3GbcW2Boc2byJ43zfFjWavXk6L4hdfZnCxu3XGiHh3A",
        }

        # sr25519 public keys
        sr_pub = {
            val_1_pub: "5Cyjbb9wHLuhihKgDyxF13mu4ybDq2c33YZLF337vaDaZdKC",
            val_2_pub: "5EJBtjPyUsYADxugWuWEahWsbrs1uSYsaXBXbFguBYWo2wFB",
            val_3_pub: "5HDqrcoJMLHND2eBhPPrchUXQWdYSTmpB1TfevQtTATkqSpz",
        }
        build_and_setup_spec(
            "kilt-parachain",
            [
                val_1_pub,
                val_2_pub,
                val_3_pub,
            ],
            {
                "grandpa": lambda x: ed_pub[x],
                "babe": lambda x: sr_pub[x],
                "im_online": lambda x: sr_pub[x],
                "para_validator": lambda x: sr_pub[x],
                "para_assignment": lambda x: sr_pub[x],
                "authority_discovery": lambda x: sr_pub[x],
                "parachain_validator": lambda x: sr_pub[x],
            },
            {
                "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY": DEFAULT_MONEY,
                "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty": DEFAULT_MONEY,
                val_1_pub: DEFAULT_MONEY,
                val_2_pub: DEFAULT_MONEY,
                val_3_pub: DEFAULT_MONEY,
            },
            val_1_pub,
            pathlib.Path.cwd() / "dev-spec/kilt-parachain/relay-stage.json",
            extras={
                "name": "KILT Staging Relaychain",
                "id": "kilt_staging_relay",
                "chainType": "Live",
                "bootNodes": [],
                "telemetryEndpoints": [["wss://telemetry-backend.kilt.io:8080", 9]],
                "protocolId": "dot",
            },
            chain_spec="rococo-local"
        )
