.bootNodes = []
    | .chainType = "Live"
    | .name = "Reregrine Relay Testnet"
    | .id = "rococo_peregrine_relay_testnet"
    | .genesis.runtime.runtime_genesis_config.palletBalances.balances += [
        ["5EPjVNeHEV1zNtY7zY9iGx9CJpRo5jyZvmTaez2RthWa9F4i", 1000000000000000000],
        ["5Chu2cTRJ3ex4YLx849G7CrJiXKcgBq6kAWb9G56eapG1Svb", 1000000000000000000],
        ["5GwwoJrFSSFbuSE6u6uZAgPjgjhWMS6JHTAzFxworh8VWgwM", 1000000000000000000]
        ]
    | .genesis.runtime.runtime_genesis_config.palletSession.keys = [
        [
            "5EPjVNeHEV1zNtY7zY9iGx9CJpRo5jyZvmTaez2RthWa9F4i",
            "5EPjVNeHEV1zNtY7zY9iGx9CJpRo5jyZvmTaez2RthWa9F4i",
            {
                "grandpa": "5CPPhfbQ7zC9JgnSteRxa6hKniVLK6v6QjC8SQLJ9KRs9G92",
                "babe": "5Coq3A2bHU4h8s93w5YJWVGB4wn5EoroNCV8kF7YfRgivmn5",
                "im_online": "5Coq3A2bHU4h8s93w5YJWVGB4wn5EoroNCV8kF7YfRgivmn5",
                "para_validator": "5Coq3A2bHU4h8s93w5YJWVGB4wn5EoroNCV8kF7YfRgivmn5",
                "para_assignment": "5Coq3A2bHU4h8s93w5YJWVGB4wn5EoroNCV8kF7YfRgivmn5",
                "authority_discovery": "5Coq3A2bHU4h8s93w5YJWVGB4wn5EoroNCV8kF7YfRgivmn5",
                "beefy": "KWCPsEJgTZbjKGmGEP9Hr7LyGeLNqRDM24YRUodyaRPwbrhhu"
            }
        ],[
            "5Chu2cTRJ3ex4YLx849G7CrJiXKcgBq6kAWb9G56eapG1Svb",
            "5Chu2cTRJ3ex4YLx849G7CrJiXKcgBq6kAWb9G56eapG1Svb",
            {
                "grandpa": "5FDViDwZ9YAtH8hcCyc5XHVEtEdAwTBU9PZ72Y8RWzuiFza2",
                "babe": "5FPPmay3GRzfAfEay8edHjcWoGJ8Gaz5xqUfSoMg9czPLqMj",
                "im_online": "5FPPmay3GRzfAfEay8edHjcWoGJ8Gaz5xqUfSoMg9czPLqMj",
                "para_validator": "5FPPmay3GRzfAfEay8edHjcWoGJ8Gaz5xqUfSoMg9czPLqMj",
                "para_assignment": "5FPPmay3GRzfAfEay8edHjcWoGJ8Gaz5xqUfSoMg9czPLqMj",
                "authority_discovery": "5FPPmay3GRzfAfEay8edHjcWoGJ8Gaz5xqUfSoMg9czPLqMj",
                "beefy": "KWCCQmmroMRLbqzkCjUaRD1c8mMbnpueFf4wStdrBTutTXdYd"
            }
        ],[
            "5GwwoJrFSSFbuSE6u6uZAgPjgjhWMS6JHTAzFxworh8VWgwM",
            "5GwwoJrFSSFbuSE6u6uZAgPjgjhWMS6JHTAzFxworh8VWgwM",
            {
                "grandpa": "5FPoYBJBvddDzrTVmvBtvYravhmPdcFPNMEk1Fcn6DvjsRo4",
                "babe": "5G7N7yFkZhAbMQ5ehijXgCLeD2WyGNcK2NvyMAHDEtUQBXFW",
                "im_online": "5G7N7yFkZhAbMQ5ehijXgCLeD2WyGNcK2NvyMAHDEtUQBXFW",
                "para_validator": "5G7N7yFkZhAbMQ5ehijXgCLeD2WyGNcK2NvyMAHDEtUQBXFW",
                "para_assignment": "5G7N7yFkZhAbMQ5ehijXgCLeD2WyGNcK2NvyMAHDEtUQBXFW",
                "authority_discovery": "5G7N7yFkZhAbMQ5ehijXgCLeD2WyGNcK2NvyMAHDEtUQBXFW",
                "beefy": "KW4NKToV6bsgwjNXmUEpKR3bknB2u5QqXUxDdzi7zqu6oENZW"
            }
        ]
    ]
    | .genesis.runtime.runtime_genesis_config.palletSudo.key = "5EPjVNeHEV1zNtY7zY9iGx9CJpRo5jyZvmTaez2RthWa9F4i"
