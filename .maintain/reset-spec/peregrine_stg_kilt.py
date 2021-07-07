import typing


def update_spec(input: typing.Dict):
    input.update({
        "bootNodes": [],
        "chainType": "Live",
        "name": "KILT Peregrine Stagenet",
        "id": "peregrine_stg_kilt",
        "para_id": 2000,
    })
    input["properties"]["tokenSymbol"] = "PILT"
    input["genesis"]["runtime"]["parachainInfo"]["parachainId"] = 2000
    input["genesis"]["runtime"]["sudo"]["key"] = "5HVf8YPXPzp4vTF9dRNh9yLQHemNc4wASoyvFCc2pPz1pbWq"
    input["genesis"]["runtime"]["parachainStaking"]["stakers"] = [
        [
            "5HVf8YPXPzp4vTF9dRNh9yLQHemNc4wASoyvFCc2pPz1pbWq",
            None,
            100000000000000000000
        ], [
            "5FxhtaNtvGWTUQzmqq8NbKVVvz8AiXvaXxnSs8WbfBXYs79M",
            None,
            100000000000000000000
        ]
    ]
    input["genesis"]["runtime"]["session"]["keys"] = [
        [
            "5HVf8YPXPzp4vTF9dRNh9yLQHemNc4wASoyvFCc2pPz1pbWq",
            "5HVf8YPXPzp4vTF9dRNh9yLQHemNc4wASoyvFCc2pPz1pbWq",
            {
                "aura": "5DSMMuNSVxc6Jz3n8AK4PLEBQQjKSAtRcQXq9MTrAEHpdGDL"
            }
        ],
        [
            "5FxhtaNtvGWTUQzmqq8NbKVVvz8AiXvaXxnSs8WbfBXYs79M",
            "5FxhtaNtvGWTUQzmqq8NbKVVvz8AiXvaXxnSs8WbfBXYs79M",
            {
                "aura": "5FzsPPWs7hnviHt3VhuSP3bHprpdXfwobWrUwQ57C22eBayW"
            }
        ],
        [
            "5CvmyN8kLcPKNg98A6nMmrPDqoNN8hJrmFfoYyCesCmfd3se",
            "5CvmyN8kLcPKNg98A6nMmrPDqoNN8hJrmFfoYyCesCmfd3se",
            {
                "aura": "5GjATpyZpKdmJeFDTRgv4Z2aBGYVQDJSQdGRok8uJKEpC4je"
            }
        ]
    ]
    input["genesis"]["runtime"]["balances"]["balances"] += [
        [
            "5HVf8YPXPzp4vTF9dRNh9yLQHemNc4wASoyvFCc2pPz1pbWq",
            10000000000000000000000000000
        ],
        [
            "5FxhtaNtvGWTUQzmqq8NbKVVvz8AiXvaXxnSs8WbfBXYs79M",
            10000000000000000000000000000
        ],
        [
            "5CvmyN8kLcPKNg98A6nMmrPDqoNN8hJrmFfoYyCesCmfd3se",
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
