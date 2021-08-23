# Pallet order in construct runtime

during the on_initialize phase the following will happen:

1. Authorship: reward the block author
2. Staking: switch to a new round (if round ended)
3. Session: start a new session (if round ended)
4. Aura: update slot number
5. AuraExt: fetch authorities to include them in the storage proof of the PoV
