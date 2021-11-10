import typing


def update_spec(input: typing.Dict):
    acc_col_1 = "5FNHPF1epsZHJC3LSGMbVJzP5ykcgursQAwPjJiNJB5eAGmW"
    acc_col_2 = "5GvFCmt5FMqV15tZUHsATAzafYvVT1HDQoRRcJq4gJ52NCHr"
    # not an initial collator
    acc_col_3 = "5EvVhMthVR1EHGEdDoMrhx9iqU2aJqD3gJu3q3xb68A5rjFZ"
    para_id = 2000

    input.update({
        "bootNodes": [
            "/dns4/bootnode.kilt.io/tcp/30340/p2p/12D3KooWALJtiCZzcUPVsCa5f5egGfQyFhPY67kKosDw95bJqK7M",
            "/dns4/bootnode.kilt.io/tcp/30341/p2p/12D3KooWCRgcGtFRsvqxqgysiR6Ah9SAzUNkM12Ef9sy59ZEspSQ",
        ],
        "chainType": "Live",
        "name": "KILT Peregrine",
        "id": "peregrine_kilt",
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
    input["genesis"]["runtime"]["kiltLaunch"]["transferAccount"] = acc_col_2
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
                "aura": "5GMw7mZsyWnL8M47ZuqUKKbd1C6LRKuWZDYQLVbkFnM8MS53"
            }
        ],
        [
            acc_col_2,
            acc_col_2,
            {
                "aura": "5DMAVHz2yDhDKKUTCJH8cQTVhLZTviJvy5SQxVZjGXUC8B2o"
            }
        ],
        [
            acc_col_3,
            acc_col_3,
            {
                "aura": "5Dvq2MZ22wys4obTHEttjje6GxHVjJo7NQVz7VswDRRRtNwB"
            }
        ]
    ]
    input["genesis"]["runtime"]["balances"]["balances"] = [
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
