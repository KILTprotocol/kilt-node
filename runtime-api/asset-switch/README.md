## Asset switch runtime API

The asset switch runtime API allows clients to query the following information:
* `fn pool_account_id(pair_id: Vec<u8>, asset_id: AssetId) -> AccountId`: the pool address of a to-be-set switch pair given the pallet name in which it will be stored and the switch pair remote asset ID.
