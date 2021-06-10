import typing


def update_spec(input: typing.Dict):
    input.update({
        "bootNodes": [],
        "chainType": "Live",
        "name": "KILT Peregrine Testnet",
        "id": "peregrine_kilt",
        "para_id": 12555,
    })
    input["properties"]["tokenSymbol"] = "PILT"
    input["genesis"]["runtime"]["parachainInfo"]["parachainId"] = 12555
    input["genesis"]["runtime"]["palletSudo"]["key"] = "5EKxJLF55ArWS38yJp2Rsyz9CEBJxsoRe9UQ4bQyX1imwZxk"
    input["genesis"]["runtime"]["parachainStaking"]["stakers"] = [
        [
            "5EKxJLF55ArWS38yJp2Rsyz9CEBJxsoRe9UQ4bQyX1imwZxk",
            None,
            100000000000000000000
        ], [
            "5DJH8AgnXEjG8jzT1B8UzV9MW9Mfsmw3vajoEFJQHqpTXeoj",
            None,
            100000000000000000000
        ]
    ]
    input["genesis"]["runtime"]["palletSession"]["keys"] = [
        [
            "5EKxJLF55ArWS38yJp2Rsyz9CEBJxsoRe9UQ4bQyX1imwZxk",
            "5EKxJLF55ArWS38yJp2Rsyz9CEBJxsoRe9UQ4bQyX1imwZxk",
            {
                "aura": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
            }
        ],
        [
            "5DJH8AgnXEjG8jzT1B8UzV9MW9Mfsmw3vajoEFJQHqpTXeoj",
            "5DJH8AgnXEjG8jzT1B8UzV9MW9Mfsmw3vajoEFJQHqpTXeoj",
            {
                "aura": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"
            }
        ],
        [
            "5GRJdyoQtAKFSX4wSDjzYMtm6zs7fqti2bLcyRQwKF7UnGYv",
            "5GRJdyoQtAKFSX4wSDjzYMtm6zs7fqti2bLcyRQwKF7UnGYv",
            {
                "aura": "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y"
            }
        ]
    ]
    input["genesis"]["runtime"]["palletBalances"]["balances"] += [
        [
            "5EKxJLF55ArWS38yJp2Rsyz9CEBJxsoRe9UQ4bQyX1imwZxk",
            10000000000000000000000000000
        ],
        [
            "5DJH8AgnXEjG8jzT1B8UzV9MW9Mfsmw3vajoEFJQHqpTXeoj",
            10000000000000000000000000000
        ],
        [
            "5GRJdyoQtAKFSX4wSDjzYMtm6zs7fqti2bLcyRQwKF7UnGYv",
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
