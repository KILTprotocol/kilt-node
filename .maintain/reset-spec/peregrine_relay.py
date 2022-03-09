import typing


def update_spec(input: typing.Dict, base_chain="westend"):
    acc_alice = "5DEx6rzF742xUcTCf3KwvNw8gZd82hQsG7WGMiqsji9AiDNZ"
    acc_bob = "5DL9V1dmkuZnzRD9R3cwdzowim3sBZZvz1iJhNxC5QjofikK"
    acc_charlie = "5DcKRxsjojmbJW7Scxnu7Ck5zXfpg1RxtrcyVjaMRx5YFWUR"
    acc_dave = "5E4ZYy9tTPpJPoScqm6PvLtr1MjsBEjbDufJQYhcuBtk6rNa"

    input.update({
        "bootNodes": [
            "/dns4/eyrie-1.kilt.io/tcp/30360/p2p/12D3KooWEeezCpJauUmWw3zfgEtYzhZTc5LgukQYtGTMaZfzgVfE",
            "/dns4/eyrie-2.kilt.io/tcp/30361/p2p/12D3KooWHq5j9tLdZEu4tnr6ii2k33zp5DCoKREQ6KzuabC9Gihu",
            "/dns4/eyrie-2.kilt.io/tcp/30362/p2p/12D3KooWQ8iTGLH98zLz9BZmq5FXDmR1NytDsJ2VToXvcjvHV16a",
            "/dns4/eyrie-1.kilt.io/tcp/30363/p2p/12D3KooWNWNptEoH443LVUgwC5kd7DBVoNYwQtJh6dp4TQxUsAST",
            "/dns4/eyrie-3.kilt.io/tcp/30365/p2p/12D3KooWCYnLwW3eEWiWqBNraUDjp8qVg2uwyJxh9fKAs32PMutR",
            "/dns4/eyrie-3.kilt.io/tcp/30366/p2p/12D3KooWCED7zzribj75pE98739qp8QNDDfXFmL2w6JMLwBWFLxL",
        ],
        "chainType": "Live",
        "name": "Peregrine Relay",
        "protocolId": "Rkilt3",
        "id": f"{base_chain}_peregrine3_relay",
        "telemetryEndpoints": [
            [
                "/dns/telemetry-backend.kilt.io/tcp/8080/x-parity-wss/%2Fsubmit",
                0
            ]
        ]

    })
    input["genesis"]["runtime"]["balances"]["balances"] = [
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
                100000000000000,
                "Validator"
            ],
            [
                acc_bob,
                acc_bob,
                100000000000000,
                "Validator"
            ],
            [
                acc_charlie,
                acc_charlie,
                100000000000000,
                "Validator"
            ],
            [
                acc_dave,
                acc_dave,
                100000000000000,
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
    input["genesis"]["runtime"]["configuration"]["config"].update(
        {
            "max_code_size": 3145728,
            "max_head_data_size": 32768,
            "max_upward_queue_count": 8,
            "max_upward_queue_size": 1048576,
            "max_upward_message_size": 1048576,
            "max_upward_message_num_per_candidate": 5,
            "hrmp_max_message_num_per_candidate": 5,
            "validation_upgrade_cooldown": 2,
            "validation_upgrade_delay": 2,
            "max_pov_size": 5242880,
            "max_downward_message_size": 1048576,
            "ump_service_total_weight": 100000000000,
            "hrmp_max_parachain_outbound_channels": 4,
            "hrmp_max_parathread_outbound_channels": 4,
            "hrmp_sender_deposit": 0,
            "hrmp_recipient_deposit": 0,
            "hrmp_channel_max_capacity": 8,
            "hrmp_channel_max_total_size": 8192,
            "hrmp_max_parachain_inbound_channels": 4,
            "hrmp_max_parathread_inbound_channels": 4,
            "hrmp_channel_max_message_size": 1048576,
            "code_retention_period": 1200,
            "parathread_cores": 0,
            "parathread_retries": 0,
            "group_rotation_frequency": 20,
            "chain_availability_period": 4,
            "thread_availability_period": 4,
            "scheduling_lookahead": 1,
            "max_validators_per_core": 5,
            "max_validators": None,
            "dispute_period": 6,
            "dispute_post_conclusion_acceptance_period": 100,
            "dispute_max_spam_slots": 2,
            "dispute_conclusion_by_time_out_period": 200,
            "no_show_slots": 2,
            "n_delay_tranches": 25,
            "zeroth_delay_tranche_width": 0,
            "needed_approvals": 2,
            "relay_vrf_modulo_samples": 2,
            "ump_max_individual_weight": 20000000000,
            "pvf_checking_enabled": False,
            "pvf_voting_ttl": 2,
            "minimum_validation_upgrade_delay": 5
        }
    )


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
