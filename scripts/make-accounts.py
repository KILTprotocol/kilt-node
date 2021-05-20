import json
import subprocess
import typing


def subkey_gen(subkey_bin) -> typing.Dict[str, str]:
    cmd = [subkey_bin, "generate", "--output-type", "json"]
    result = subprocess.run(cmd, check=True, capture_output=True)
    try:
        return json.loads(result.stdout)
    except json.decoder.JSONDecodeError as err:
        print(f"Error while parsing output! ({err})")
        print("command: ({})".format(" ".join(cmd)))
        print(f"Output: ({result.stdout})")
        raise RuntimeError("invalid output from subkey") from err


if __name__ == "__main__":
    x = []
    for i in range(5000):
        x.extend([
            [
                subkey_gen("subkey")["ss58Address"],
                10000000000000000000000 + i,
                5*i,
                0,
            ], [
                subkey_gen("subkey")["ss58Address"],
                10000000000000000000000 + i,
                0,
                5*i,
            ]
        ])

    with open("./nodes/parachain/res/genesis-testing/genesis-accounts.json", "w") as f:
        json.dump(x, f)
