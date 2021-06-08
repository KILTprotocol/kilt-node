import typing


def update_spec(input: typing.Dict):
    input.update({
        "bootNodes": [],
        "para_id": 2005,
    })
    input["genesis"]["runtime"]["parachainInfo"]["parachainId"] = 2005


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
