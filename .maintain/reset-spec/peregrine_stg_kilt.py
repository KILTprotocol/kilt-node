import typing


def update_spec(input: typing.Dict):
    acc_col_1 = "5HVf8YPXPzp4vTF9dRNh9yLQHemNc4wASoyvFCc2pPz1pbWq"
    acc_col_2 = "5FxhtaNtvGWTUQzmqq8NbKVVvz8AiXvaXxnSs8WbfBXYs79M"
    # not an initial collator
    acc_col_3 = "5CvmyN8kLcPKNg98A6nMmrPDqoNN8hJrmFfoYyCesCmfd3se"
    para_id = 2000

    input.update({
        "bootNodes": [
            "/dns4/eyrie-1.kilt.io/tcp/30340/p2p/12D3KooWDRzUz3SenRC737aFrY1aPAVbiioqVMUL7otbupWtuk3B",
            "/dns4/eyrie-2.kilt.io/tcp/30341/p2p/12D3KooWLSzt9LjJwvQrZmM3AW6cR5ypVFHpmYJCRKs4HWFmTj5a",
        ],
        "chainType": "Live",
        "name": "KILT Peregrine Stagenet",
        "id": "peregrine_stg_kilt",
        "protocolId": "pkilt4",
        "para_id": para_id,
        "telemetryEndpoints": [
            [
                "/dns/telemetry-backend.kilt.io/tcp/8080/x-parity-wss/%2Fsubmit",
                0
            ]
        ]
    })
    input["properties"]["tokenSymbol"] = "PILT"
    input["genesis"]["runtime"]["parachainInfo"]["parachainId"] = para_id
    input["genesis"]["runtime"]["sudo"]["key"] = acc_col_1
    input["genesis"]["runtime"]["parachainStaking"]["stakers"] = [
        [
            acc_col_1,
            None,
            100000000000000000000
        ], [
            acc_col_2,
            None,
            100000000000000000000
        ]
    ]
    input["genesis"]["runtime"]["session"]["keys"] = [
        [
            acc_col_1,
            acc_col_1,
            {
                "aura": "5DSMMuNSVxc6Jz3n8AK4PLEBQQjKSAtRcQXq9MTrAEHpdGDL"
            }
        ],
        [
            acc_col_2,
            acc_col_2,
            {
                "aura": "5FzsPPWs7hnviHt3VhuSP3bHprpdXfwobWrUwQ57C22eBayW"
            }
        ],
        [
            acc_col_3,
            acc_col_3,
            {
                "aura": "5GjATpyZpKdmJeFDTRgv4Z2aBGYVQDJSQdGRok8uJKEpC4je"
            }
        ]
    ]
    input["genesis"]["runtime"]["balances"]["balances"] += [
        [
            acc_col_1,
            10000000000000000000000000000
        ],
        [
            acc_col_2,
            10000000000000000000000000000
        ],
        [
            acc_col_3,
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
