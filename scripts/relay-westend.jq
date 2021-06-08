.bootNodes = []
    | .chainType = "Live"
    | .genesis.runtime.palletBalances.balances += [
        ["5CA1Ym7i37qggvsK8D6nMAeRGHZo6gz8oEQvZ5qsvNeZdyqy", 1000000000000000000],
        ["5DjkAWLDGvesjGfvZkcSfaTTZnUUEVgyUFATQv583nqGG1rm", 1000000000000000000],
        ["5HQBuyGFKqqBkoK2cwv4a6s9dEv34cudoMfoT1wxkEn1xcic", 1000000000000000000]
        ]
    | .genesis.runtime.palletSession.keys = [
          [
            "5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY",
            "5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY",
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
            "5HpG9w8EBLe5XCrbczpwq5TSXvedjrBGCwqxK1iQ7qUsSWFc",
            "5HpG9w8EBLe5XCrbczpwq5TSXvedjrBGCwqxK1iQ7qUsSWFc",
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
            "5Ck5SLSHYac6WFt5UZRSsdJjwmpSZq85fd5TRNAdZQVzEAPT",
            "5Ck5SLSHYac6WFt5UZRSsdJjwmpSZq85fd5TRNAdZQVzEAPT",
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
    | .genesis.runtime.palletStaking.stakers = [
          [
            "5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY",
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
            1000000000000,
            "Validator"
          ],
          [
            "5HpG9w8EBLe5XCrbczpwq5TSXvedjrBGCwqxK1iQ7qUsSWFc",
            "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
            1000000000000,
            "Validator"
          ],
          [
            "5Ck5SLSHYac6WFt5UZRSsdJjwmpSZq85fd5TRNAdZQVzEAPT",
            "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y",
            1000000000000,
            "Validator"
          ]
        ]
    | .genesis.runtime.palletStaking.invulnerables = [
            "5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY",
            "5HpG9w8EBLe5XCrbczpwq5TSXvedjrBGCwqxK1iQ7qUsSWFc",
            "5Ck5SLSHYac6WFt5UZRSsdJjwmpSZq85fd5TRNAdZQVzEAPT"
        ]
    | .genesis.runtime.palletSudo.key = "5CA1Ym7i37qggvsK8D6nMAeRGHZo6gz8oEQvZ5qsvNeZdyqy"
    | .genesis.runtime.parachainsConfiguration.config = {
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
