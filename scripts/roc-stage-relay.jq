.bootNodes = []
    | .chainType = "Live"
    | .name = "Staging Relay Testnet"
    | .id = "rococo_staging_relay_testnet"
    | .genesis.runtime.runtime_genesis_config.palletBalances.balances += [
        ["5CA1Ym7i37qggvsK8D6nMAeRGHZo6gz8oEQvZ5qsvNeZdyqy", 1000000000000000000],
        ["5DjkAWLDGvesjGfvZkcSfaTTZnUUEVgyUFATQv583nqGG1rm", 1000000000000000000],
        ["5HQBuyGFKqqBkoK2cwv4a6s9dEv34cudoMfoT1wxkEn1xcic", 1000000000000000000]
        ]
    | .genesis.runtime.runtime_genesis_config.palletSession.keys = [
        [
            "5CA1Ym7i37qggvsK8D6nMAeRGHZo6gz8oEQvZ5qsvNeZdyqy",
            "5CA1Ym7i37qggvsK8D6nMAeRGHZo6gz8oEQvZ5qsvNeZdyqy",
            {
                "grandpa": "5FA9nQDVg267DEd8m1ZypXLBnvN7SFxYwV7ndqSYGiN9TTpu",
                "babe": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                "im_online": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                "para_validator": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                "para_assignment": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                "authority_discovery": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                "beefy": "KW39r9CJjAVzmkf9zQ4YDb2hqfAVGdRqn53eRqyruqpxAP5YL"
            }
        ],[
            "5DjkAWLDGvesjGfvZkcSfaTTZnUUEVgyUFATQv583nqGG1rm",
            "5DjkAWLDGvesjGfvZkcSfaTTZnUUEVgyUFATQv583nqGG1rm",
            {
                "grandpa": "5GoNkf6WdbxCFnPdAnYYQyCjAKPJgLNxXwPjwTh6DGg6gN3E",
                "babe": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
                "im_online": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
                "para_validator": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
                "para_assignment": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
                "authority_discovery": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
                "beefy": "KWByAN7WfZABWS5AoWqxriRmF5f2jnDqy3rB5pfHLGkY93ibN"
            }
        ],[
            "5HQBuyGFKqqBkoK2cwv4a6s9dEv34cudoMfoT1wxkEn1xcic",
            "5HQBuyGFKqqBkoK2cwv4a6s9dEv34cudoMfoT1wxkEn1xcic",
            {
                "grandpa": "5DbKjhNLpqX3zqZdNBc9BGb4fHU1cRBaDhJUskrvkwfraDi6",
                "babe": "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y",
                "im_online": "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y",
                "para_validator": "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y",
                "para_assignment": "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y",
                "authority_discovery": "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y",
                "beefy": "KWBpGtyJLBkJERdZT1a1uu19c2uPpZm9nFd8SGtCfRUAT3Y4w"
            }
        ]
    ]
    | .genesis.runtime.runtime_genesis_config.palletSudo.key = "5CA1Ym7i37qggvsK8D6nMAeRGHZo6gz8oEQvZ5qsvNeZdyqy"
