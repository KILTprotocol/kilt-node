import typing


def update_spec(input: typing.Dict):
    input.update({
        "bootNodes": [],
        "chainType": "Live",
        "name": "KILT-Westend",
        "id": "kilt_westend",
        "para_id": 2009,
    })
    input["properties"]["tokenSymbol"] = "WILT"
    input["genesis"]["runtime"]["parachainInfo"]["parachainId"] = 2009
    input["genesis"]["runtime"]["palletSudo"]["key"] = "5CniRAWZ8PoyG4iiJHThG2xtcuGBBPdpLuHZn6TeXtfmcjt4"
    input["genesis"]["runtime"]["parachainStaking"]["stakers"] = [
        [
            "5HHLPJtACk7EZazmfZ8FhBuy3jHEmaq29Hsy6QNs95C8ztPG",
            None,
            100000000000000000000
        ], [
            "5HL7PEpJityg6AJ9KVy8o1pBjmMdgNCsqjDFbC6G1Q7Tzsz6",
            None,
            100000000000000000000
        ]
    ]
    input["genesis"]["runtime"]["palletSession"]["keys"] = [
        [
            "5HHLPJtACk7EZazmfZ8FhBuy3jHEmaq29Hsy6QNs95C8ztPG",
            "5HHLPJtACk7EZazmfZ8FhBuy3jHEmaq29Hsy6QNs95C8ztPG",
            {
                "aura": "5HBqbnRfea78tRqVzQkcTdG6KEjtceLWTAJnpqSQcWaDKxBU"
            }
        ],
        [
            "5HL7PEpJityg6AJ9KVy8o1pBjmMdgNCsqjDFbC6G1Q7Tzsz6",
            "5HL7PEpJityg6AJ9KVy8o1pBjmMdgNCsqjDFbC6G1Q7Tzsz6",
            {
                "aura": "5EtBBVbj7UBwpgvW3ufMPjF1ZX13WPiJDWknLLcaEtctWxMb"
            }
        ]
    ]
    input["genesis"]["runtime"]["palletBalances"]["balances"] += [
        [
            "5HHLPJtACk7EZazmfZ8FhBuy3jHEmaq29Hsy6QNs95C8ztPG",
            10000000000000000000000000000
        ],
        [
            "5HL7PEpJityg6AJ9KVy8o1pBjmMdgNCsqjDFbC6G1Q7Tzsz6",
            10000000000000000000000000000
        ],
        [
            "5CniRAWZ8PoyG4iiJHThG2xtcuGBBPdpLuHZn6TeXtfmcjt4",
            10000000000000000000000000000
        ]
    ]


if __name__ == "__main__":
    import json
    import sys

    in_file = sys.argv[1]
    with open(in_file, "r") as f:
        in_json = json.load(f)
        update_spec(in_json)

    out_file = sys.argv[2]
    with open(out_file, "w") as f:
        json.dump(in_json, f)
