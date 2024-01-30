import typing


def update_spec(input: typing.Dict):
    acc_alice = "5DEx6rzF742xUcTCf3KwvNw8gZd82hQsG7WGMiqsji9AiDNZ"
    acc_bob = "5DL9V1dmkuZnzRD9R3cwdzowim3sBZZvz1iJhNxC5QjofikK"
    acc_charlie = "5DcKRxsjojmbJW7Scxnu7Ck5zXfpg1RxtrcyVjaMRx5YFWUR"
    acc_dave = "5E4ZYy9tTPpJPoScqm6PvLtr1MjsBEjbDufJQYhcuBtk6rNa"
    acc_eve = "5ELCvQBGu8ur9UDSMAiqB4PrYXTnLGwcaUg63gtkxWtScEYm"
    acc_ferdie = "5G6ThxmfSbHVt2u8WmZmTH3xeKBckFFDGA69E6cSXtYPaiwT"

    input.update({
        "bootNodes": [
            "/dns4/eyrie-1.kilt.io/tcp/30380/p2p/12D3KooWMJvZTCSNc2t8DKPSJH1LBqLrAWK5yxENfSJ3WPj1UMnP",
            "/dns4/eyrie-1.kilt.io/tcp/30381/p2p/12D3KooWPkTuR4PFgYAXA5UPRkXJRLrE7t45AXM3obwrweNUUfuR",
            "/dns4/eyrie-2.kilt.io/tcp/30382/p2p/12D3KooWEPScFexvmgxesjbwqzsVEuYYYf8t9wUnFK2U6SnPNWMY",
            "/dns4/eyrie-2.kilt.io/tcp/30383/p2p/12D3KooWFYzkpamEy3M415hd93W7iNLt81kuqzhLKMdvTGmq9zuV",
            "/dns4/eyrie-3.kilt.io/tcp/30384/p2p/12D3KooWSa1TqsrBuB32fMcGWfS3aZDT1NicsSVs7egPNXtsD5Am",
            "/dns4/eyrie-3.kilt.io/tcp/30385/p2p/12D3KooWQNXPgLttsQLKA1PAEWNHzJBLQShPx1tUaMoJew4d3yke",
        ],
        "chainType": "Live",
        "name": "Peregrine-stg Westend-Relay",
        "id": "westend_peregrine_stg_relay",
        "protocolId": "Rkilt4",
        "telemetryEndpoints": [
            [
                "/dns/telemetry-backend.kilt.io/tcp/8080/x-parity-wss/%2Fsubmit",
                0
            ]
        ]
    })
    input["genesis"]["runtimeGenesis"]["patch"]["balances"]["balances"] = [
        [
            acc_alice,
            1000000000000000000
        ],
        [
            acc_bob,
            1000000000000000000
        ],
        [
            acc_charlie,
            1000000000000000000
        ],
        [
            acc_dave,
            1000000000000000000
        ],
        [
            acc_eve,
            1000000000000000000
        ],
        [
            acc_ferdie,
            1000000000000000000
        ],
    ]
    input["genesis"]["runtimeGenesis"]["patch"]["session"]["keys"] = [
        [
            acc_alice,
            acc_alice,
            {
                "beefy": "KW4eVXdGVk5uK8GeucUxXmyHyUbLJu7Uoge7qHryUud3UzTRk",
                "grandpa": "5CivBtb51w9YRUFosRuriWCJDxke7ePPzHfVsQv4xCFWB3cW",
                "babe": "5DLN2JrTfFw3gz6fowJwNzVZ6eqY17Y4cRa3McZHRPtcbbGp",
                "para_validator": "5DLN2JrTfFw3gz6fowJwNzVZ6eqY17Y4cRa3McZHRPtcbbGp",
                "para_assignment": "5DLN2JrTfFw3gz6fowJwNzVZ6eqY17Y4cRa3McZHRPtcbbGp",
                "authority_discovery": "5DLN2JrTfFw3gz6fowJwNzVZ6eqY17Y4cRa3McZHRPtcbbGp"
            }
        ],
        [
            acc_bob,
            acc_bob,
            {
                "beefy": "KW3zUEVDPjM9xc1DMbVNe2GbSFWZN35x264VrnUFJk5KLKRs8",
                "grandpa": "5HG2xP1NVfDnYFpzZghNWp19zmrFdmcidS8ijW3eHyVgDarW",
                "babe": "5Fbhgb2YW52EjYn5eyUa22dTLFPv3rhmzujrkcfGxXZieSnD",
                "para_validator": "5Fbhgb2YW52EjYn5eyUa22dTLFPv3rhmzujrkcfGxXZieSnD",
                "para_assignment": "5Fbhgb2YW52EjYn5eyUa22dTLFPv3rhmzujrkcfGxXZieSnD",
                "authority_discovery": "5Fbhgb2YW52EjYn5eyUa22dTLFPv3rhmzujrkcfGxXZieSnD"
            }
        ],
        [
            acc_charlie,
            acc_charlie,
            {
                "beefy": "KW87Cp6e4x17zyBJ9dxmet3EJMYNZLtTUFxgKDDsiEJCaPKht",
                "grandpa": "5FKEiJd9D8CEDdmMNWTuxqQUxQGdXoFdLUpnZwjiRxUM8Q2J",
                "babe": "5EZikizy2cNFQPioevi2JxL2VsqWJMWfL4BtnDPK7mds8v7T",
                "para_validator": "5EZikizy2cNFQPioevi2JxL2VsqWJMWfL4BtnDPK7mds8v7T",
                "para_assignment": "5EZikizy2cNFQPioevi2JxL2VsqWJMWfL4BtnDPK7mds8v7T",
                "authority_discovery": "5EZikizy2cNFQPioevi2JxL2VsqWJMWfL4BtnDPK7mds8v7T"
            }
        ],
        [
            acc_dave,
            acc_dave,
            {
                "beefy": "KWBtBmKhK629mhDtTVhZwRXgVVEyFqzSkUEdVzyW5vrBWE1H9",
                "grandpa": "5HMx5YbAjzv715hkCQwpcqedvWFU82ytsWj9n4sobhBQv8Dc",
                "babe": "5Hj5gZH93HZZyj7Lo8zPvBWLFufwtP6qXy2NUKepsXjYKgMR",
                "para_validator": "5Hj5gZH93HZZyj7Lo8zPvBWLFufwtP6qXy2NUKepsXjYKgMR",
                "para_assignment": "5Hj5gZH93HZZyj7Lo8zPvBWLFufwtP6qXy2NUKepsXjYKgMR",
                "authority_discovery": "5Hj5gZH93HZZyj7Lo8zPvBWLFufwtP6qXy2NUKepsXjYKgMR"
            }
        ],
        [
            acc_eve,
            acc_eve,
            {
                "authority_discovery": "5H6Vwmomuc9mProwMz1uj64rwqcwAReC3guXTJdWutjATZ5o",
                "babe": "5H6Vwmomuc9mProwMz1uj64rwqcwAReC3guXTJdWutjATZ5o",
                "beefy": "KW4zpcJ1b1op86FtyKXuomJzzHrMaeMFhLtZHTGnao2HdgoXr",
                "grandpa": "5HZQiPE7pN7BWv94vPbBmMeuTL5maxp2op4VeftBSaGVuqRA",
                "para_assignment": "5H6Vwmomuc9mProwMz1uj64rwqcwAReC3guXTJdWutjATZ5o",
                "para_validator": "5H6Vwmomuc9mProwMz1uj64rwqcwAReC3guXTJdWutjATZ5o"
            }
        ],
        [
            acc_ferdie,
            acc_ferdie,
            {
                "authority_discovery": "5DcY86VRU8V68theAJwCVubsBbbz5fGDyq9XuCfkexzN5FEi",
                "babe": "5DcY86VRU8V68theAJwCVubsBbbz5fGDyq9XuCfkexzN5FEi",
                "beefy": "KW4gMUL2VVYejMyhJQTjPVHveWfNf3NznvWCLLGzVDDR3qpee",
                "grandpa": "5EXkVwaWd4bKKLVyW2G5K6ZfhvFyJUpf5qoFvyJvEYrxchsF",
                "para_assignment": "5DcY86VRU8V68theAJwCVubsBbbz5fGDyq9XuCfkexzN5FEi",
                "para_validator": "5DcY86VRU8V68theAJwCVubsBbbz5fGDyq9XuCfkexzN5FEi"
            }
        ]
    ]
    input["genesis"]["runtimeGenesis"]["patch"]["sudo"]["key"] = acc_alice
    input["genesis"]["runtimeGenesis"]["patch"]["staking"].update({
        "validatorCount": 6,
        "invulnerables":[acc_alice, acc_bob],
        "stakers": [
            [
                acc_alice,
                acc_alice,
                1000000000000000,
                "Validator"
            ],
            [
                acc_bob,
                acc_bob,
                1000000000000000,
                "Validator"
            ],
            [
                acc_charlie,
                acc_charlie,
                1000000000000000,
                "Validator"
            ],
            [
                acc_dave,
                acc_dave,
                1000000000000000,
                "Validator"
            ],
            [
                acc_eve,
                acc_eve,
                1000000000000000,
                "Validator"
            ],
            [
                acc_ferdie,
                acc_ferdie,
                1000000000000000,
                "Validator"
            ]
        ]
    })
    input["genesis"]["runtimeGenesis"]["patch"]["configuration"]["config"].update(
        {
            "approval_voting_params": {
              "max_approval_coalesce_count": 1
            },
            "async_backing_params": {
              "allowed_ancestry_len": 0,
              "max_candidate_depth": 0
            },
            "code_retention_period": 1200,
            "coretime_cores": 0,
            "dispute_period": 6,
            "dispute_post_conclusion_acceptance_period": 100,
            "executor_params": [],
            "group_rotation_frequency": 20,
            "hrmp_channel_max_capacity": 8,
            "hrmp_channel_max_message_size": 1048576,
            "hrmp_channel_max_total_size": 8192,
            "hrmp_max_message_num_per_candidate": 5,
            "hrmp_max_parachain_inbound_channels": 4,
            "hrmp_max_parachain_outbound_channels": 4,
            "hrmp_recipient_deposit": 0,
            "hrmp_sender_deposit": 0,
            "max_code_size": 3145728,
            "max_downward_message_size": 1048576,
            "max_head_data_size": 32768,
            "max_pov_size": 5242880,
            "max_upward_message_num_per_candidate": 5,
            "max_upward_message_size": 51200,
            "max_upward_queue_count": 8,
            "max_upward_queue_size": 1048576,
            "max_validators": 200,
            "max_validators_per_core": 5,
            "minimum_backing_votes": 2,
            "minimum_validation_upgrade_delay": 5,
            "n_delay_tranches": 25,
            "needed_approvals": 2,
            "no_show_slots": 2,
            "node_features": {
              "bits": 0,
              "data": [],
              "head": {
                "index": 0,
                "width": 8
              },
              "order": "bitvec::order::Lsb0"
            },
            "on_demand_base_fee": 10000000,
            "on_demand_fee_variability": 30000000,
            "on_demand_queue_max_size": 10000,
            "on_demand_retries": 0,
            "on_demand_target_queue_utilization": 250000000,
            "on_demand_ttl": 5,
            "paras_availability_period": 4,
            "pvf_voting_ttl": 2,
            "relay_vrf_modulo_samples": 2,
            "scheduling_lookahead": 1,
            "validation_upgrade_cooldown": 2,
            "validation_upgrade_delay": 2,
            "zeroth_delay_tranche_width": 0
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
