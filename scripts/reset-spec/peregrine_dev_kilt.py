import typing


def update_spec(input: typing.Dict):
    para_id = 2000

    input.update({
        "bootNodes": [],
        "chainType": "Local",
        "name": "KILT Peregrine Testnet",
        "id": "peregrine_kilt",
        "para_id": para_id,
    })
    input["properties"]["tokenSymbol"] = "PILT"
    input["genesis"]["runtime"]["parachainInfo"]["parachainId"] = para_id
    input["genesis"]["runtime"]["sudo"]["key"] = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
    input["genesis"]["runtime"]["parachainStaking"]["stakers"] = [
        [
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
            None,
            100000000000000000000
        ], [
            "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
            None,
            100000000000000000000
        ]
    ]
    input["genesis"]["runtime"]["session"]["keys"] = [
        [
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
            {
                "aura": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
            }
        ],
        [
            "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
            "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
            {
                "aura": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"
            }
        ],
        [
            "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y",
            "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y",
            {
                "aura": "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y"
            }
        ]
    ]


if __name__ == "__main__":
    import json
    import sys
    import subprocess

    docker_img = sys.argv[1]
    out_file = sys.argv[2]

    process = subprocess.run(["docker", "run", docker_img, "build-spec", "--runtime",
                              "peregrine", "--chain", "dev", "--disable-default-bootnode"], capture=True)

    in_json = json.load(process.stdout)
    update_spec(in_json)

    with open(out_file, "w") as f:
        json.dump(in_json, f)
