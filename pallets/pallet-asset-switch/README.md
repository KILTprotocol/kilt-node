# Asset switch pallet

The asset switch pallet introduces the possibility for the chain local currency to be switched 1:1 with a remote asset at a remote destination, according to the provided configuration, using XCM.

This is possible via the creation of a *switch pair*, which contains information about the identifier of the remote asset (e.g., its `MultiLocation`), the remote location on which the asset lives (and with which XCM communication takes place), the circulating supply of the remote asset which can be switched back for the local currency, and additional information relevant for the XCM communication, which are explained more in depth later on.

## In a gist

The pallet lets users on the source chain lock `N` tokens of the chain's native currency into a switch pair specific account, to receive `N` remote assets on the configured remote location, which are transferred to them from the source chain's sovereign account at destination.
Then, the chain, which i

## Design choices

The pallet aims to be generic enough for most parachains to be able to add one or more instances of the asset switch pallet into their runtime, after configuring it according to their needs.

The pallet makes the following assumptions:

* The local asset to switch is always the chain local currency, implementing the `fungible::Mutate` trait.
* The switch ratio is pre-defined to be 1:1, i.e., one unit of local currency for one unit of remote asset.
* The pallet only exposes calls to allow local -> remote switches. The reverse is assumed to happen via XCM reserve transfer from the configured remote location, for which the pallet crate provides all the XCM components that are dynamically configured based on the switch pair information stored within each instance of this pallet.
* The sovereign account of the source chain at destination owns the remote assets in the amount that is specified when the switch pair is created. The validation of this requirement is delegated to the source chain governance in the act leading to the creation of the switch pair, as the source chain itself has currently no means of doing that.
* Similarly, the pallet has currently no way to verify that a transfer of remote tokens from the chain sovereign account to the specified beneficiary has been completed successfully on the remote location, hence it takes an optimistic approach given that all the verifiable preconditions for the switch are verified on the source chain, i.e., the chain from which the switch originates. Any unexpected issues in the transfer on the remote location will most likely require intervention of the source chain governance to re-balance the state and make it consistent again.

## Add the pallet to the runtime

Add the following line to the runtime `Cargo.toml` dependencies section:

```toml
pallet-asset-switch = {git = "https://github.com/KILTprotocol/kilt-node.git", branch = "release-1.13.0"}
```

The asset switch pallet is available in the KILT node release 1.13.0 and later.

## Configure the pallet

The pallet can be added one or more times to the runtime.

For multiple deployments of the same pallet (e.g., to bridge the local currency to different remote assets), pass runtime configuration to the pallet's `Config` trait.

```rust
pub type SwitchPool1 = pallet_asset_switch::Instance1;
impl pallet_asset_switch::Config<SwitchPool1> for Runtime {
	// Config
}

pub type SwitchPool2 = pallet_asset_switch::Instance2;
impl pallet_asset_switch::Config<SwitchPool2> for Runtime {
	// Config
}
```

If a single instance is required, then simply use the default instance:

```rust
impl pallet_asset_switch::Config for Runtime {
    // Config
}
```

## The `Config` trait

As the pallet is generic over the runtime specifics, the `Config` trait requires the following configuration parameters passed to it:

- `type AccountIdConverter: TryConvert<Self::AccountId, Junction>`: Because the `AccountId` type can be anything in the runtime, the converter is responsible for converting such a `AccountId` into a `Junction`, which is then used for some XCM processing.
- `type AssetTransactor: TransactAsset`: This component is used when charging the extrinsic submitter with the XCM fees that the chain will pay at destination. For instance, if the transfer on the remote chain will cost 0.1 DOTs, the `AssetTransactor` might deduct 0.1 DOTs from the user's previously topped up balance on the source chain (more details below).
- `type FeeOrigin: EnsureOrigin<Self::RuntimeOrigin>`: The origin that can update the XCM fee to be paid for the transfer on destination.
- `type LocalCurrency: MutateFungible<Self::AccountId>`: The chain's local currency.
- `type PauseOrigin: EnsureOrigin<Self::RuntimeOrigin>`: The origin that can pause a swap pair, e.g., if a vulnerability is found.
- `type RuntimeEvent: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::RuntimeEvent>`: The aggregate `Event` type.
- `type SubmitterOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>`: The origin that can call the `swap` extrinsic and perform the swap.
- `type SwitchHooks: SwitchHooks<Self, I>`: Any additional runtime-specific logic that can be injected both before and after local tokens are exchanged for the remote assets, and before and after the remote assets are being converted into local tokens.
- `type SwitchOrigin: EnsureOrigin<Self::RuntimeOrigin>`: The origin that can set, resume, and delete a switch pair.
- `type XcmRouter: SendXcm`: The component responsible for routing XCM messages to the switch pair remote location to perform the remote asset transfer from the chain's sovereign account to the specified beneficiary.

## Storage

The pallet has a single `SwitchPair` storage value that contains a `Option<SwitchPairInfo>`.

If unset, no switch pair is configured hence no switch can happen.

When set and its status is `Running`, switches are enabled in both directions.
