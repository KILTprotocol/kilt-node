# Asset switch pallet

The asset switch pallet allows for switching the chain local currency 1:1 with a remote asset at a remote destination, according to the provided configuration, and using XCM.

This is possible by creating a *switch pair*, which contains information about the remote asset's identifier (e.g., its `Location`), the remote location where the asset lives (and with which XCM communication takes place), the circulating supply of the remote asset, which can be switched back for the local currency, and additional information relevant for the XCM communication, which is explained more in-depth later on.

## Summary

The pallet lets users on the local chain lock `N` tokens of the chain's native currency into a switch pair-specific account to receive `N` remote assets on the configured remote location, which are transferred to them from the source chain's sovereign account on the remote chain.

## Design choices

The pallet aims to be generic enough for most parachains to be able to add one or more instances of the asset switch pallet into their runtime, after configuring it according to their needs.

The pallet makes the following assumptions:

* The local asset to switch is always the chain local currency, implementing the `fungible::Mutate` trait.
* The switch ratio is pre-defined to be 1:1, i.e., one unit of local currency for one unit of remote asset.
* The pallet only exposes calls to allow local -> remote switches. The reverse is assumed to happen via XCM reserve transfer from the configured remote location, for which the pallet crate provides all the XCM components that are dynamically configured based on the switch pair information stored within each instance of this pallet.
* The sovereign account of the source chain at destination owns the remote assets in the amount that is specified when the switch pair is created. The validation of this requirement is delegated to the source chain governance in the act leading to the creation of the switch pair, as the source chain itself has currently no means of doing that.
* Similarly, the pallet has currently no way to verify that a transfer of remote tokens from the chain sovereign account to the **specified** beneficiary has been completed successfully on the remote location, hence it takes an optimistic approach given that all the verifiable preconditions for the switch are verified on the source chain, i.e., the chain from which the switch originates. Any unexpected issues in the transfer on the remote location will most likely require intervention of the source chain governance to re-balance the state and make it consistent again.

## Add the pallet to the runtime

Add the following line to the runtime `Cargo.toml` dependencies section:

```toml
pallet-asset-switch = {git = "https://github.com/KILTprotocol/kilt-node.git", branch = "release-1.14.0"}
```

The asset switch pallet is available in the KILT node release 1.14.0 and later.

## Configure the pallet

The pallet can be added one or more times to the runtime.

For multiple deployments of the same pallet (e.g., to bridge the local currency to different remote assets), pass runtime configuration to the pallet's `Config` trait.

```rust,ignore
pub type SwitchPool1 = pallet_asset_switch::Instance1;
impl pallet_asset_switch::Config<SwitchPool1> for Runtime {
	// Config
}

pub type SwitchPool2 = pallet_asset_switch::Instance2;
impl pallet_asset_switch::Config<SwitchPool2> for Runtime {
	// Config
}
```

If a single instance is required, then use the default instance:

```rust,ignore
impl pallet_asset_switch::Config for Runtime {
    // Config
}
```

## The `Config` trait

As the pallet is generic over the runtime specifics, the `Config` trait requires the following configuration parameters passed to it:

- `type AccountIdConverter: TryConvert<Self::AccountId, Junction>`: Because the `AccountId` type can be anything in the runtime, the converter is responsible for converting such a `AccountId` into a `Junction`, which is then used for some XCM processing.
- `type AssetTransactor: TransactAsset`: This component is used when charging the extrinsic submitter with the XCM fees that the chain will pay at the remote chain. For instance, if the transfer on the remote chain will cost 0.1 DOTs, the `AssetTransactor` might deduct 0.1 DOTs from the user's previously topped up balance on the source chain (more details below).
- `type FeeOrigin: EnsureOrigin<Self::RuntimeOrigin>`: The origin that can update the XCM fee to be paid for the transfer on the remote chain.
- `type LocalCurrency: MutateFungible<Self::AccountId>`: The chain's local currency.
- `type PauseOrigin: EnsureOrigin<Self::RuntimeOrigin>`: The origin that can pause a switch pair, e.g., if a vulnerability is found.
- `type RuntimeEvent: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::RuntimeEvent>`: The aggregate `Event` type.
- `type SubmitterOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>`: The origin that can call the `switch` extrinsic and perform the switch.
- `type SwitchHooks: SwitchHooks<Self, I>`: Any additional runtime-specific logic that can be injected both before and after local tokens are exchanged for the remote assets, and before and after the remote assets are converted into local tokens.
- `type SwitchOrigin: EnsureOrigin<Self::RuntimeOrigin>`: The origin that can set, resume, and delete a switch pair.
- `type WeightInfo: WeightInfo`: The computed weights of the pallet after benchmarking it.
- `type XcmRouter: SendXcm`: The component responsible for routing XCM messages to the switch pair remote location to perform the remote asset transfer from the chain's sovereign account to the specified beneficiary.

### Benchmark-only `Config` components

- `type BenchmarkHelper: BenchmarkHelper`: Helper trait to allow the runtime to set the ground before the benchmark logic is executed. It allows the runtime to return any of the parameters that are used in the extrinsic benchmarks, or `None` if the runtime has no special conditions to fulfil.

## Storage

The pallet has a single `SwitchPair` storage value that contains a `Option<SwitchPairInfo>`.
If unset, no switch pair is configured hence no switch can happen.
When set and its status is `Running`, switches are enabled in both directions.

## Events

The pallet generates the following events:

- `SwitchPairCreated`: when a new switch pair is created by the required origin, e.g., governance.
- `SwitchPairRemoved`: when a switch pair is removed by the root origin.
- `SwitchPairResumed`: when a switch pair has (re-)enabled local to remote asset switches.
- `SwitchPairPaused`: when a switch pair has been paused.
- `SwitchPairFeeUpdated`: when the XCM fee for the switch transfer has been updated.
- `LocalToRemoteSwitchExecuted`: when a switch of some local tokens for the remote asset has taken place.
- `RemoteToLocalSwitchExecuted`: when a switch of some remote assets for the local tokens has taken place.

## Calls

1. `pub fn set_switch_pair(origin: OriginFor<T>, remote_asset_total_supply: u128, remote_asset_id: Box<VersionedAssetId>, remote_asset_circulating_supply: u128, remote_reserve_location: Box<Location>, remote_asset_ed: u128, remote_xcm_fee: Box<Asset>) -> DispatchResult`: Set a new switch pair between the local currency and the specified `remote_asset_id` on the `reserve_location`. The specified `total_issuance` includes both the `circulating_supply` (i.e., the remote asset amount that the chain does not control on the `reserve_location`) and the locked supply under the control of the chain's sovereign account on the `reserve_location`. For this reason, the value of `total_issuance` must be at least as large as `circulating_supply`. It is possible for `circulating_supply` to be `0`, in which case it means this chain controls all the `total_issuance` of the remote asset, which can be obtained by locking a corresponding amount of local tokens via the `switch` call below.

   Furthermore, the pallet calculates the account that will hold the local tokens locked in exchange for remote tokens. This account is based on the pallet runtime name as returned by the `PalletInfoAccess` trait and the value of `remote_asset_id`. The generated account must already have a balance of at least `circulating_supply`, ensuring enough local tokens are locked to satisfy all requests to exchange the remote asset for local tokens. The balance of such an account can be increased with a simple transfer after obtaining the to-be-created switch pair pool account by interacting with the [asset-switch runtime API][asset-switch-runtime-api].

   This requirement can be bypassed with the `force_set_switch_pair` call. Only `SwitchOrigin` can call this, and in most cases it will most likely be a governance-based origin such as the one provided by referenda or collectives with high privileges.
2. `pub fn force_set_switch_pair(origin: OriginFor<T>, remote_asset_total_supply: u128, remote_asset_id: Box<VersionedAssetId>, remote_asset_circulating_supply: u128, remote_reserve_location: Box<Location>, remote_asset_ed: u128, remote_xcm_fee: Box<Asset>) -> DispatchResult`: The same as the `set_switch_pair`, but skips the check over the switch pair pool account balance, and requires the `root` origin for the call to be dispatched.
3. `pub fn force_unset_switch_pair(origin: OriginFor<T>) -> DispatchResult`: Forcibly remove a previously-stored switch pair. This operation can only be called by the `root` origin.

	**Any intermediate state, such as local tokens locked in the switch pair pool or remote assets that are not switchable anymore for local tokens, must be taken care of with subsequent governance operations.**
4. `pub fn pause_switch_pair(origin: OriginFor<T>) -> DispatchResult`: Allows the `PauseOrigin` to immediately pause switches in both directions.
5. `pub fn resume_switch_pair(origin: OriginFor<T>) -> DispatchResult`: Allows the `SwitchOrigin` to resume switches in both directions.
6. `pub fn remote_xcm_fee(origin: OriginFor<T>, new: Box<Asset>) -> DispatchResult`: Allows the `FeeOrigin` to update the required XCM fee to execute the transfer of remote asset on the reserve location from the chain's sovereign account to the beneficiary specified in the `switch` operation.

	For example, if the cost of sending an XCM message containing a `TransferAsset` instruction from the source chain to AssetHub (reserve location) changes from 0.1 DOTs to 0.2 DOTs, the fee will need to be updated accordingly to avoid transfers failing on AssetHub, leaving the whole system in an inconsistent state. Since the pallet refunds any unused assets on the reserve location to the account initiating the switch on the source chain, it is not a big issue to overestimate this value here since no funds will be burnt or unfairly taken from the user during the switch process.
7. `pub fn switch(origin: OriginFor<T>, local_asset_amount: LocalCurrencyBalanceOf<T, I>, beneficiary: Box<Location>) -> DispatchResult`: Allows the `SubmitterOrigin` to perform a switch of some local tokens for the corresponding amount of remote assets on the configured `reserve_location`. The switch will fail on the source chain if any of the following preconditions are not met:
	1. The submitter does not have enough balance to pay for the tx fees on the source chain or to cover the amount of local tokens requested. Hence, the user's local balance must be greater than or equal to the amount of tokens requested in the switch + the cost of executing the extrinsic on the source chain.
	2. No switch pair is set or the switch pair is currently not allowing switches.
	3. There are not enough locked remote assets on the `reserve_location` to cover the switch request. e.g., if the chain sovereign account on the `reserve_location` only controls `10` remote assets, users can only switch up to `10` local tokens. Once the limit is reached, someone needs to perform the reverse operation (remote -> local switch) to free up some remote tokens.
	4. The switch pair `reserve_location` is not reachable from the source chain, because the configured `XcmRouter` returns an error (e.g., there is no XCM channel between the two chains).
	5. The configured `SwitchHooks` returns an error in either the `pre-` or the `post-` switch checks.
	6. The user does not have enough assets to pay for the required remote XCM fees as specified in the switch pair info and as returned by the configured `AssetTransactor`.

## XCM components

Because the switch functionality relies on XCM, the pallet provides a few XCM components that should be included in a runtime to enable the whole set of interactions between the source chain and the configured remote reserve location.

* `AccountId32ToAccountId32JunctionConverter` in [xcm::convert][xcm-convert]: provides an implementation for the pallet's `AccountIdConverter` config component, that converts local `AccountId32`s into a `AccountId32` XCM `Junction`. This works only for chains that use `AccountId32` as their overarching `AccountId` type.
* `MatchesSwitchPairXcmFeeFungibleAsset` in [xcm::match][xcm-match]: provides an implementation of the `MatchesFungibles<Location, Fungibles::Balance>` that returns the input `Asset` if its ID matches the XCM fee asset ID as configured in the switch pair, if present. If no switch pair is present or if the ID does not match, it returns a [XcmExecutorError::AssetNotHandled][XcmExecutorError::AssetNotHandled], which does not prevent other matchers after it to apply their matching logic. It can be used for the `AssetTransactor` property of the [XcmExecutor::Config][XcmExecutor::Config] and as the `AssetTransactor` component of this pallet in the runtime.
* `UsingComponentsForXcmFeeAsset` in [xcm::trade][xcm-trade]: provides an implementation of `WeightTrader` that allows buying weight using the XCM fee asset configured in the switch pair. That is, if the XCM fee asset is DOT, and users need to send DOTs to this chain in order to pay for XCM fees, this component lets them use those very same DOTs that are being sent to pay for the XCM fees on this chain. Any unused weight is burnt, since this chain's sovereign account already controls the whole amount on the reserve location due to the nature of reserve-based transfers. It can be used for the `Trader` property of the [XcmExecutor::Config][XcmExecutor::Config].
* `UsingComponentsForSwitchPairRemoteAsset` in [xcm::trade][xcm-trade]: provides an implementation of `WeightTrader` that allows buying weight using the remote asset configured in the switch pair when sending it to this chain to be switched for local tokens. Any unused weight is transferred from the switch pair account to the configured `FeeDestinationAccount`, as those local tokens do not need to back any remote assets because they have been used to pay for XCM fees. It can be used for the `Trader` property of the [XcmExecutor::Config][XcmExecutor::Config].
* `SwitchPairRemoteAssetTransactor` in [xcm::transact][xcm-transact]: provides an implementation of `TransactAsset::deposit_asset` that matches the asset to be deposited with the remote asset configured in the switch pair '(else it returns [Error::AssetNotFound][Error::AssetNotFound]) and moves as many local tokens from the switch pair account to the specified `who` destination. It also calls into the `SwitchHooks` pre- and post- checks, and generates a `RemoteToLocalSwitchExecuted` if everything is completed successfully. It can be used for the `AssetTransactor` property of the [XcmExecutor::Config][XcmExecutor::Config].
* `IsSwitchPairXcmFeeAsset` in [xcm::transfer][xcm-transfer]: provides an implementation of `ContainsPair<Asset, Location>` that returns `true` if the given asset and sender match the stored switch pair XCM fee asset and reserve location respectively. It can be used for the `IsReserve` property of the [XcmExecutor::Config][XcmExecutor::Config].
* `IsSwitchPairRemoteAsset` in [xcm::transfer][xcm-transfer]: provides an implementation of `ContainsPair<Asset, Location>` that returns `true` if the given asset and sender match the stored switch pair remote asset and reserve location respectively. It can be used for the `IsReserve` property of the [XcmExecutor::Config][XcmExecutor::Config].

[asset-switch-runtime-api]: ../../runtime-api/asset-switch/
[xcm-convert]: ./src/xcm/convert.rs
[xcm-match]: ./src/xcm/match.rs
[XcmExecutorError::AssetNotHandled]: https://github.com/paritytech/polkadot-sdk/blob/33324fe01c5b1f341687cef2aa6e767f6acf40f3/polkadot/xcm/xcm-executor/src/traits/token_matching.rs#L54
[XcmExecutor::Config]: https://github.com/paritytech/polkadot-sdk/blob/33324fe01c5b1f341687cef2aa6e767f6acf40f3/polkadot/xcm/xcm-executor/src/config.rs#L31
[xcm-trade]: ./src/xcm/trade.rs
[Error::AssetNotFound]: https://github.com/paritytech/polkadot-sdk/blob/e5791a56dcc35e308a80985cc3b6b7f2ed1eb6ec/polkadot/xcm/src/v3/traits.rs#L68
[xcm-transact]: ./src/xcm/transact.rs
[xcm-transfer]: ./src/xcm/transfer.rs
