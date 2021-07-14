import typing


def update_spec(input: typing.Dict, base_chain="westend"):
    acc_alice = "5DEx6rzF742xUcTCf3KwvNw8gZd82hQsG7WGMiqsji9AiDNZ"
    acc_bob = "5DL9V1dmkuZnzRD9R3cwdzowim3sBZZvz1iJhNxC5QjofikK"
    acc_charlie = "5DcKRxsjojmbJW7Scxnu7Ck5zXfpg1RxtrcyVjaMRx5YFWUR"
    acc_dave = "5E4ZYy9tTPpJPoScqm6PvLtr1MjsBEjbDufJQYhcuBtk6rNa"

    input.update({
        "bootNodes": [],
        "chainType": "Live",
        "name": "Peregrine Relay",
        "id": f"{base_chain}_peregrine_relay",
    })
    input["genesis"]["runtime"]["balances"]["balances"] += [
        [acc_alice, 1000000000000000000],
        [acc_bob, 1000000000000000000],
        [acc_charlie, 1000000000000000000],
        [acc_dave, 1000000000000000000]
    ]
    input["genesis"]["runtime"]["staking"].update({
        "validatorCount": 4,
        "stakers": [
            [
                acc_alice,
                acc_alice,
                1000000000000,
                "Validator"
            ],
            [
                acc_bob,
                acc_bob,
                1000000000000,
                "Validator"
            ],
            [
                acc_charlie,
                acc_charlie,
                1000000000000,
                "Validator"
            ],
            [
                acc_dave,
                acc_dave,
                1000000000000,
                "Validator"
            ]
        ]
    })
    input["genesis"]["runtime"]["session"]["keys"] = [
        [
            acc_alice,
            acc_alice,
            {
                "grandpa": "5H7KYuAdFtXTjdSeWjEKidoHGLCDuRe3vWpVK5pxEux1ysrU",
                "babe": "5F7iGThj9n28FdQUzBfrWRALMqLvELEndVbXQ1e9DatM23Rz",
                "im_online": "5F7iGThj9n28FdQUzBfrWRALMqLvELEndVbXQ1e9DatM23Rz",
                "para_validator": "5F7iGThj9n28FdQUzBfrWRALMqLvELEndVbXQ1e9DatM23Rz",
                "para_assignment": "5F7iGThj9n28FdQUzBfrWRALMqLvELEndVbXQ1e9DatM23Rz",
                "authority_discovery": "5F7iGThj9n28FdQUzBfrWRALMqLvELEndVbXQ1e9DatM23Rz",
            }
        ], [
            acc_bob,
            acc_bob,
            {
                "grandpa": "5H25SYu7RHgvNrJbcpHKdRv5gjeKYxfn4wvAD3hpu2cbhRyW",
                "babe": "5CGQvLkqjGb1v5ptKoeQdtXJXQkEXmfhB9CFDLSS8MUYkDg2",
                "im_online": "5CGQvLkqjGb1v5ptKoeQdtXJXQkEXmfhB9CFDLSS8MUYkDg2",
                "para_validator": "5CGQvLkqjGb1v5ptKoeQdtXJXQkEXmfhB9CFDLSS8MUYkDg2",
                "para_assignment": "5CGQvLkqjGb1v5ptKoeQdtXJXQkEXmfhB9CFDLSS8MUYkDg2",
                "authority_discovery": "5CGQvLkqjGb1v5ptKoeQdtXJXQkEXmfhB9CFDLSS8MUYkDg2",
            }
        ], [
            acc_charlie,
            acc_charlie,
            {
                "grandpa": "5FBpVyAAB4E9woWhgg19LWiKDqUdMMkCZ41b6wTFVLR3qfxS",
                "babe": "5Cz484xJRDU1MzbdC853TomhfPbuwVftTefAraaqReWZhaMu",
                "im_online": "5Cz484xJRDU1MzbdC853TomhfPbuwVftTefAraaqReWZhaMu",
                "para_validator": "5Cz484xJRDU1MzbdC853TomhfPbuwVftTefAraaqReWZhaMu",
                "para_assignment": "5Cz484xJRDU1MzbdC853TomhfPbuwVftTefAraaqReWZhaMu",
                "authority_discovery": "5Cz484xJRDU1MzbdC853TomhfPbuwVftTefAraaqReWZhaMu",
            }
        ], [
            acc_dave,
            acc_dave,
            {
                "grandpa": "5CEDZib61ec64jfRpZUe7Q78Yh4oAmzMwkr519rYWBkH8hGZ",
                "babe": "5FxM6yYBFvMeX3C2QrQDXJ8LhhzG5sPjgUtX77cX1wihHhVq",
                "im_online": "5FxM6yYBFvMeX3C2QrQDXJ8LhhzG5sPjgUtX77cX1wihHhVq",
                "para_validator": "5FxM6yYBFvMeX3C2QrQDXJ8LhhzG5sPjgUtX77cX1wihHhVq",
                "para_assignment": "5FxM6yYBFvMeX3C2QrQDXJ8LhhzG5sPjgUtX77cX1wihHhVq",
                "authority_discovery": "5FxM6yYBFvMeX3C2QrQDXJ8LhhzG5sPjgUtX77cX1wihHhVq",
            }
        ]
    ]
    input["genesis"]["runtime"]["sudo"]["key"] = acc_alice


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
