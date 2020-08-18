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


def build_spec(node, chain, raw=False):
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


def fill_spec(spec, authorities, session_keys, endowed, root, parachain_id):
    runtime = spec["genesis"]["runtime"]
    if "runtime_genesis_config" in runtime:
        runtime = runtime["runtime_genesis_config"]

    if parachain_id:
        spec["para_id"] = parachain_id
        runtime["parachainInfo"]["parachainId"] = parachain_id

    try:
        balance_config = runtime["balances"]
    except KeyError:
        balance_config = runtime["palletBalances"]

    fill_balances(balance_config, endowed)

    try:
        session = runtime["session"]
    except KeyError:
        session = runtime["palletSession"]
    if authorities and session_keys:
        fill_session(session, authorities, session_keys)

    try:
        sudo_config = runtime["sudo"]
    except KeyError:
        sudo_config = runtime["palletSudo"]
    set_root(sudo_config, root)


def build_and_setup_spec(node, authorities, session_keys, endowed, root, outpath, parachain_id=None, extras=None, chain_spec="dev"):
    chain_spec = build_spec(node, chain_spec)

    if extras:
        chain_spec.update(extras)

    fill_spec(chain_spec, authorities, session_keys, endowed, root, parachain_id)

    plain_file = pathlib.Path("/tmp/" + uuid.uuid4().hex)
    with plain_file.open("w") as f:
        json.dump(chain_spec, f, indent="  ")
    logger.info("plain spec stored at %s",
                plain_file.absolute().as_posix())

    raw = build_spec(
        node,
        plain_file.absolute().as_posix(),
        raw=True
    )

    with outpath.open("w") as f:
        json.dump(raw, f, indent="  ")


def setup_spec(authorities, session_keys, endowed, root, chain_file: pathlib.Path, extras=None):
    with chain_file.open("r") as f:
        chain_spec = json.load(f)

    if extras:
        chain_spec.update(extras)

    fill_spec(chain_spec, authorities, session_keys, endowed, root, None)

    plain_file = pathlib.Path("/tmp/" + uuid.uuid4().hex)
    with plain_file.open("w") as f:
        json.dump(chain_spec, f, indent="  ")
    logger.info("plain spec stored at %s",
                plain_file.absolute().as_posix())


if __name__ == "__main__":
    logging.basicConfig(format='%(asctime)s:%(levelname)s: %(message)s',
                        datefmt='%m-%d-%Y %H:%M:%S', level=logging.DEBUG)

    parser = argparse.ArgumentParser(
        description=("Update the chainspec for our networks."
                     "VERIFY THAT THE SPEC IS CORRECT AFTER USE!!"
                     "Make sure that the current directory is the project root."),
        epilog="Example usage: python3 scripts/update-spec.py --roc-stage-relay")
    parser.add_argument("--testnet", "-t", action="store_true", dest="testnet",
                        help="update testnet spec")
    parser.add_argument("--rococo", "-r", action="store_true", dest="rococo",
                        help="update rococo parachain spec")
    parser.add_argument("--roc-stage-relay", "-k", dest="roc_staging_relay",
                        help="update rococo staging relay spec", type=pathlib.Path)

    args = parser.parse_args()

    DEFAULT_MONEY = 10 ** 27

    # ##########################################################################
    # ############################     TESTNET      ############################
    # ##########################################################################

    # hex: 0x58d3bb9e9dd245f3dec8d8fab7b97578c00a10cf3ca9d224caaa46456f91c46c
    TESTNET_ALICE = "5E5Ay9N93vijY5jAZMRcZUAxfyCqqg7a74DYB7zXbEvkr4Ab"
    # hex: 0xd660b4470a954ecc99496d4e4b012ee9acac3979e403967ef09de20da9bdeb28
    TESTNET_BOB = "5GunqBt9noWvqLpbehi4b96PauHCWSHM76Mext8QtG9pxnAj"
    # hex: 0x2ecb6a4ce4d9bc0faab70441f20603fcd443d6d866e97c9e238a2fb3e982ae2f
    TESTNET_CHARLIE = "5D84VBrtsBX7L9mJkH21Y4eVFRXSCvJUQ88MWxpXu6rfR6s6"
    # hex: 0x3cd78d9e468030ac8eff5b5d2b40e35aa9db01a9e48997e61f97f0da8c572411
    TESTNET_FAUCET = "5DSUmChuuD74E84ybM7xerzjD37DHmqEgi1ByvLgPViGvCQD"

    TESTNET_SPEC_PATH = pathlib.Path.cwd() / "nodes/standalone/res/testnet.json"

    if args.testnet:
        logger.info("update testnet spec")
        build_and_setup_spec(
            "mashnet-node",
            [TESTNET_ALICE, TESTNET_BOB, TESTNET_CHARLIE],
            {
                "aura": lambda x: x,
                "grandpa": lambda x: x,
            },
            {
                TESTNET_ALICE: DEFAULT_MONEY,
                TESTNET_BOB: DEFAULT_MONEY,
                TESTNET_CHARLIE: DEFAULT_MONEY,
                TESTNET_FAUCET: DEFAULT_MONEY
            },
            TESTNET_ALICE,
            TESTNET_SPEC_PATH,
            extras={
                "name": "KILT Testnet",
                "id": "kilt_testnet",
                "chainType": "Live",
                "bootNodes": [
                    "/dns4/bootnode-alice.kilt-prototype.tk/tcp/30333/p2p/12D3KooWPuXafPUY8E8zo7m4GuWkgj9ByfsanrNUznZShBgJrW4A",
                    "/dns4/bootnode-bob.kilt-prototype.tk/tcp/30334/p2p/12D3KooWPVLgaJoD4CdGzAFzaZSgiDSBM4jqt34LjH1XdtGnVsss",
                    "/dns4/bootnode-charlie.kilt-prototype.tk/tcp/30335/p2p/12D3KooWKBLU9T9MTxLJuy6hddPChRdnw91TBk3wYJzDyBQH9dXx",
                ],
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
        setup_spec(
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
            },
            {
                "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY": DEFAULT_MONEY,
                "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty": DEFAULT_MONEY,
                val_1_pub: DEFAULT_MONEY,
                val_2_pub: DEFAULT_MONEY,
                val_3_pub: DEFAULT_MONEY,
            },
            val_1_pub,
            chain_file=args.roc_staging_relay,
            extras={
                # changing those breaks the chainspec building. :(
                # "name": "KILT Staging Relaychain",
                # "id": "kilt_staging_relay",
                # "chainType": "Live",
                "bootNodes": [],
                "protocolId": "dot",
            },
        )
