.bootNodes = []
    | .chainType = "Live"
    | .name = "KILT-Westend"
    | .id = "kilt_westend"
    | .properties.tokenSymbol = "WILT"
    | .para_id = 12555
    | .genesis.runtime.parachainInfo.parachainId = 12555
    | .genesis.runtime.palletSudo.key = "5CniRAWZ8PoyG4iiJHThG2xtcuGBBPdpLuHZn6TeXtfmcjt4"
    | .genesis.runtime.parachainStaking.stakers = [
            [
                "5HHLPJtACk7EZazmfZ8FhBuy3jHEmaq29Hsy6QNs95C8ztPG",
                null,
                100000000000000000000
            ],[
                "5HL7PEpJityg6AJ9KVy8o1pBjmMdgNCsqjDFbC6G1Q7Tzsz6",
                null,
                100000000000000000000
            ]
        ]
    | .genesis.runtime.palletSession.keys = [
            [
                "5HHLPJtACk7EZazmfZ8FhBuy3jHEmaq29Hsy6QNs95C8ztPG",
                "5HHLPJtACk7EZazmfZ8FhBuy3jHEmaq29Hsy6QNs95C8ztPG",
                {
                    "aura": "5HBqbnRfea78tRqVzQkcTdG6KEjtceLWTAJnpqSQcWaDKxBU"
                }
            ],
            [
                "5HL7PEpJityg6AJ9KVy8o1pBjmMdgNCsqjDFbC6G1Q7Tzsz6",
                "5HL7PEpJityg6AJ9KVy8o1pBjmMdgNCsqjDFbC6G1Q7Tzsz6",
                {
                    "aura": "5EtBBVbj7UBwpgvW3ufMPjF1ZX13WPiJDWknLLcaEtctWxMb"
                }
            ]
        ]
    | .genesis.runtime.palletBalances.balances += [
            [
                "5HHLPJtACk7EZazmfZ8FhBuy3jHEmaq29Hsy6QNs95C8ztPG",
                10000000000000000000000000000
            ],
            [
                "5HL7PEpJityg6AJ9KVy8o1pBjmMdgNCsqjDFbC6G1Q7Tzsz6",
                10000000000000000000000000000
            ],
            [
                "5CniRAWZ8PoyG4iiJHThG2xtcuGBBPdpLuHZn6TeXtfmcjt4",
                10000000000000000000000000000
            ]
        ]
