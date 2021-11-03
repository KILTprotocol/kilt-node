import typing


def update_spec(input: typing.Dict):
    acc_alice = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
    acc_alice_ed = "5FA9nQDVg267DEd8m1ZypXLBnvN7SFxYwV7ndqSYGiN9TTpu"
    acc_bob = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"
    acc_bob_ed = "5GoNkf6WdbxCFnPdAnYYQyCjAKPJgLNxXwPjwTh6DGg6gN3E"
    acc_charlie = "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y"
    acc_charlie_ed = "5DbKjhNLpqX3zqZdNBc9BGb4fHU1cRBaDhJUskrvkwfraDi6"

    input.update({
        "bootNodes": [],
        "chainType": "Local",
        "name": "Peregrine Relay Devnet",
        "id": "westend_peregrine_relay_devnet",
    })
    input["genesis"]["runtime"]["balances"]["balances"] += [
    ]
    input["genesis"]["runtime"]["session"]["keys"] = [
        [
            acc_alice,
            acc_alice,
            {
                "grandpa": acc_alice_ed,
                "babe": acc_alice,
                "im_online": acc_alice,
                "para_validator": acc_alice,
                "para_assignment": acc_alice,
                "authority_discovery": acc_alice
            }
        ],
        [
            acc_bob,
            acc_bob,
            {
                "grandpa": acc_bob_ed,
                "babe": acc_bob,
                "im_online": acc_bob,
                "para_validator": acc_bob,
                "para_assignment": acc_bob,
                "authority_discovery": acc_bob
            }
        ],
        [
            acc_charlie,
            acc_charlie,
            {
                "grandpa": acc_charlie_ed,
                "babe": acc_charlie,
                "im_online": acc_charlie,
                "para_validator": acc_charlie,
                "para_assignment": acc_charlie,
                "authority_discovery": acc_charlie
            }
        ]
    ]
    input["genesis"]["runtime"]["sudo"]["key"] = acc_alice
    input["genesis"]["runtime"]["staking"].update({
        "validatorCount": 3,
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
                    ]
        ]
    })
    input["genesis"]["runtime"]["configuration"]["config"].update(
        {
            "max_code_size": 3145728,
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
