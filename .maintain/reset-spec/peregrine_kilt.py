import typing


def update_spec(input: typing.Dict):
    input.update({
        "bootNodes": [],
        "chainType": "Live",
        "name": "KILT Peregrine Testnet",
        "id": "peregrine_kilt",
        "para_id": 2000,
    })
    input["properties"]["tokenSymbol"] = "PILT"
    input["genesis"]["runtime"]["parachainInfo"]["parachainId"] = 2000
    input["genesis"]["runtime"]["sudo"]["key"] = "5FNHPF1epsZHJC3LSGMbVJzP5ykcgursQAwPjJiNJB5eAGmW"
    input["genesis"]["runtime"]["parachainStaking"]["stakers"] = [
        [
            "5FNHPF1epsZHJC3LSGMbVJzP5ykcgursQAwPjJiNJB5eAGmW",
            None,
            100000000000000000000
        ], [
            "5GvFCmt5FMqV15tZUHsATAzafYvVT1HDQoRRcJq4gJ52NCHr",
            None,
            100000000000000000000
        ]
    ]
    input["genesis"]["runtime"]["session"]["keys"] = [
        [
            "5FNHPF1epsZHJC3LSGMbVJzP5ykcgursQAwPjJiNJB5eAGmW",
            "5FNHPF1epsZHJC3LSGMbVJzP5ykcgursQAwPjJiNJB5eAGmW",
            {
                "aura": "5GMw7mZsyWnL8M47ZuqUKKbd1C6LRKuWZDYQLVbkFnM8MS53"
            }
        ],
        [
            "5GvFCmt5FMqV15tZUHsATAzafYvVT1HDQoRRcJq4gJ52NCHr",
            "5GvFCmt5FMqV15tZUHsATAzafYvVT1HDQoRRcJq4gJ52NCHr",
            {
                "aura": "5DMAVHz2yDhDKKUTCJH8cQTVhLZTviJvy5SQxVZjGXUC8B2o"
            }
        ],
        [
            "5EvVhMthVR1EHGEdDoMrhx9iqU2aJqD3gJu3q3xb68A5rjFZ",
            "5EvVhMthVR1EHGEdDoMrhx9iqU2aJqD3gJu3q3xb68A5rjFZ",
            {
                "aura": "5Dvq2MZ22wys4obTHEttjje6GxHVjJo7NQVz7VswDRRRtNwB"
            }
        ]
    ]
    input["genesis"]["runtime"]["balances"]["balances"] += [
        [
            "5FNHPF1epsZHJC3LSGMbVJzP5ykcgursQAwPjJiNJB5eAGmW",
            10000000000000000000000000000
        ],
        [
            "5GvFCmt5FMqV15tZUHsATAzafYvVT1HDQoRRcJq4gJ52NCHr",
            10000000000000000000000000000
        ],
        [
            "5EvVhMthVR1EHGEdDoMrhx9iqU2aJqD3gJu3q3xb68A5rjFZ",
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
