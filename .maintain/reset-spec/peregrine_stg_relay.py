import typing


def update_spec(input: typing.Dict):
    acc_alice = "5DEx6rzF742xUcTCf3KwvNw8gZd82hQsG7WGMiqsji9AiDNZ"
    acc_bob = "5DL9V1dmkuZnzRD9R3cwdzowim3sBZZvz1iJhNxC5QjofikK"
    acc_charlie = "5DcKRxsjojmbJW7Scxnu7Ck5zXfpg1RxtrcyVjaMRx5YFWUR"

    input.update({
        "bootNodes": [],
        "chainType": "Live",
        "name": "Peregrine Westend-Relay Stagenet",
        "id": "westend_peregrine_relay_stagenet",
    })
    input["genesis"]["runtime"]["balances"]["balances"] += [
        [
            acc_alice,
            10000000000000000000000000000
        ],
        [
            acc_bob,
            10000000000000000000000000000
        ],
        [
            acc_charlie,
            10000000000000000000000000000
        ],
    ]
    input["genesis"]["runtime"]["session"]["keys"] = [
        [
            acc_alice,
            acc_alice,
            {
                "grandpa": "5FA9nQDVg267DEd8m1ZypXLBnvN7SFxYwV7ndqSYGiN9TTpu",
                "babe": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                "im_online": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                "para_validator": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                "para_assignment": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                "authority_discovery": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
            }
        ],
        [
            acc_bob,
            acc_bob,
            {
                "grandpa": "5GoNkf6WdbxCFnPdAnYYQyCjAKPJgLNxXwPjwTh6DGg6gN3E",
                "babe": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
                "im_online": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
                "para_validator": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
                "para_assignment": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
                "authority_discovery": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"
            }
        ],
        [
            acc_charlie,
            acc_charlie,
            {
                "grandpa": "5DbKjhNLpqX3zqZdNBc9BGb4fHU1cRBaDhJUskrvkwfraDi6",
                "babe": "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y",
                "im_online": "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y",
                "para_validator": "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y",
                "para_assignment": "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y",
                "authority_discovery": "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y"
            }
        ]
    ]
    input["genesis"]["runtime"]["sudo"]["key"] = acc_alice
    input["genesis"]["runtime"]["staking"].update({
        "validatorCount": 3,
        "stakers": [
                    [
                        acc_alice,
                        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                        1000000000000,
                        "Validator"
                    ],
            [
                        acc_bob,
                        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
                        1000000000000,
                        "Validator"
                    ],
            [
                        acc_charlie,
                        "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y",
                        1000000000000,
                        "Validator"
                    ]
        ]
    })
    input["genesis"]["runtime"]["parachainsConfiguration"]["config"].update(
        {
            "max_code_size": 5242880,
            "max_head_data_size": 20480,
            "max_upward_queue_count": 10,
            "max_upward_queue_size": 51200,
            "max_upward_message_size": 51200,
            "max_upward_message_num_per_candidate": 10,
            "hrmp_max_message_num_per_candidate": 10,
            "validation_upgrade_frequency": 14400,
            "validation_upgrade_delay": 600,
            "max_pov_size": 5242880,
            "max_downward_message_size": 51200,
            "preferred_dispatchable_upward_messages_step_weight": 100000000000,
            "hrmp_max_parachain_outbound_channels": 10,
            "hrmp_max_parathread_outbound_channels": 0,
            "hrmp_open_request_ttl": 2,
            "hrmp_sender_deposit": 1009100000000000,
            "hrmp_recipient_deposit": 1009100000000000,
            "hrmp_channel_max_capacity": 1000,
            "hrmp_channel_max_total_size": 102400,
            "hrmp_max_parachain_inbound_channels": 10,
            "hrmp_max_parathread_inbound_channels": 0,
            "hrmp_channel_max_message_size": 102400,
            "code_retention_period": 28800,
            "parathread_cores": 0,
            "parathread_retries": 0,
            "group_rotation_frequency": 10,
            "chain_availability_period": 5,
            "thread_availability_period": 5,
            "scheduling_lookahead": 1,
            "max_validators_per_core": 5,
            "max_validators": 200,
            "dispute_period": 6,
            "dispute_post_conclusion_acceptance_period": 600,
            "dispute_max_spam_slots": 2,
            "dispute_conclusion_by_time_out_period": 600,
            "no_show_slots": 2,
            "n_delay_tranches": 40,
            "zeroth_delay_tranche_width": 0,
            "needed_approvals": 15,
            "relay_vrf_modulo_samples": 1
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
