import typing


def update_spec(input: typing.Dict):
    input.update({
        "bootNodes": [],
        "chainType": "Live",
        "name": "Peregrine Relay Testnet",
        "id": "rococo_peregrine_relay_testnet",
    })
    input["genesis"]["runtime"]["runtime_genesis_config"]["palletBalances"]["balances"] += [
        ["5EPjVNeHEV1zNtY7zY9iGx9CJpRo5jyZvmTaez2RthWa9F4i", 1000000000000000000],
        ["5Chu2cTRJ3ex4YLx849G7CrJiXKcgBq6kAWb9G56eapG1Svb", 1000000000000000000],
        ["5GwwoJrFSSFbuSE6u6uZAgPjgjhWMS6JHTAzFxworh8VWgwM", 1000000000000000000]
    ]
    input["genesis"]["runtime"]["runtime_genesis_config"]["palletSession"]["keys"] = [
        [
            "5EPjVNeHEV1zNtY7zY9iGx9CJpRo5jyZvmTaez2RthWa9F4i",
            "5EPjVNeHEV1zNtY7zY9iGx9CJpRo5jyZvmTaez2RthWa9F4i",
            {
                "grandpa": "5FA9nQDVg267DEd8m1ZypXLBnvN7SFxYwV7ndqSYGiN9TTpu",
                "babe": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                "im_online": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                "para_validator": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                "para_assignment": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                "authority_discovery": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                "beefy": "KW39r9CJjAVzmkf9zQ4YDb2hqfAVGdRqn53eRqyruqpxAP5YL"
            }
        ], [
            "5Chu2cTRJ3ex4YLx849G7CrJiXKcgBq6kAWb9G56eapG1Svb",
            "5Chu2cTRJ3ex4YLx849G7CrJiXKcgBq6kAWb9G56eapG1Svb",
            {
                "grandpa": "5GoNkf6WdbxCFnPdAnYYQyCjAKPJgLNxXwPjwTh6DGg6gN3E",
                "babe": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
                "im_online": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
                "para_validator": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
                "para_assignment": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
                "authority_discovery": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
                "beefy": "KWByAN7WfZABWS5AoWqxriRmF5f2jnDqy3rB5pfHLGkY93ibN"
            }
        ], [
            "5GwwoJrFSSFbuSE6u6uZAgPjgjhWMS6JHTAzFxworh8VWgwM",
            "5GwwoJrFSSFbuSE6u6uZAgPjgjhWMS6JHTAzFxworh8VWgwM",
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
    input["genesis"]["runtime"]["runtime_genesis_config"]["palletSudo"]["key"] = "5EPjVNeHEV1zNtY7zY9iGx9CJpRo5jyZvmTaez2RthWa9F4i"


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
