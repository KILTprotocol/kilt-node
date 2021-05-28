.bootNodes = []
    | .chainType = "Live"
    | .name = "KILT Collator Peregrine Testnet"
    | .id = "peregrine_kilt"
    | .properties.tokenSymbol = "PILT"
    | .para_id = 12555
    | .genesis.runtime.parachainInfo.parachainId = 12555
    | .genesis.runtime.palletSudo.key = "5EKxJLF55ArWS38yJp2Rsyz9CEBJxsoRe9UQ4bQyX1imwZxk"
    | .genesis.runtime.parachainStaking.stakers = [
          [
            "5EKxJLF55ArWS38yJp2Rsyz9CEBJxsoRe9UQ4bQyX1imwZxk",
            null,
            100000000000000000000
          ],[
            "5DJH8AgnXEjG8jzT1B8UzV9MW9Mfsmw3vajoEFJQHqpTXeoj",
            null,
            100000000000000000000
          ]
        ]
    | .genesis.runtime.palletSession.keys = [
          [
            "5EKxJLF55ArWS38yJp2Rsyz9CEBJxsoRe9UQ4bQyX1imwZxk",
            "5EKxJLF55ArWS38yJp2Rsyz9CEBJxsoRe9UQ4bQyX1imwZxk",
            {
              "aura": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
            }
          ],
          [
            "5DJH8AgnXEjG8jzT1B8UzV9MW9Mfsmw3vajoEFJQHqpTXeoj",
            "5DJH8AgnXEjG8jzT1B8UzV9MW9Mfsmw3vajoEFJQHqpTXeoj",
            {
              "aura": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"
            }
          ],
          [
            "5GRJdyoQtAKFSX4wSDjzYMtm6zs7fqti2bLcyRQwKF7UnGYv",
            "5GRJdyoQtAKFSX4wSDjzYMtm6zs7fqti2bLcyRQwKF7UnGYv",
            {
              "aura": "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y"
            }
          ]
        ]
    | .genesis.runtime.palletBalances.balances += [
            [
                "5EKxJLF55ArWS38yJp2Rsyz9CEBJxsoRe9UQ4bQyX1imwZxk",
                10000000000000000000000000000
            ],
            [
                "5DJH8AgnXEjG8jzT1B8UzV9MW9Mfsmw3vajoEFJQHqpTXeoj",
                10000000000000000000000000000
            ],
            [
              "5GRJdyoQtAKFSX4wSDjzYMtm6zs7fqti2bLcyRQwKF7UnGYv",
              10000000000000000000000000000
            ]
        ]
