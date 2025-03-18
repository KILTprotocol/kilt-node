# Pallet Bonded Coins

The Pallet Bonded Coins module allows for the creation of new currencies that can be minted by locking up tokens of an existing currency as collateral. 
The exchange rate of collateral to tokens minted is determined by a bonding curve that links the total issuance of the new currency to its mint or burn price.

## Overview

This pallet provides functionality to:
- Create new currency pools with specified bonding curves and collateral types.
- Mint new tokens by locking up collateral.
- Burn tokens to release collateral.
- Manage the lifecycle of currency pools, including refunding and destroying pools.

### Rounding 

Rounding issues are a problem and cannot be completely avoided due to the nature of limited resources on a computer, resulting in a lack of representation for irrational numbers. 
This pallet cannot guarantee mathematically exact calculations. 
However, it can guarantee the reproducibility of the same result based on the usage of [fixed-point][fixed-point] numbers. 


## Key Concepts

### Bonding Curve
A bonding curve is a mathematical curve that defines the relationship between the supply of a token and its price. 
In this pallet, the bonding curve determines the cost of minting or burning tokens based on the current supply. 
The current system implements the [LMSR][lmsr], square root, and polynomial bonding curves.

- LMSR (Logarithmic Market Scoring Rule): A market-making algorithm that adjusts prices based on the logarithm of the token supply.
- Square Root: A bonding curve where the price is proportional to the square root of the token supply.
- Polynomial: A bonding curve where the price is determined by a polynomial function of the token supply.

More information and implementation details can be found [here][bonding-curve].

### Collateral
Collateral is an existing currency that must be locked up to mint new tokens. 
The amount of collateral required is determined by the bonding curve in the creation step.

### Pool
A [pool][pool-details] is a collection of bonded currencies and their associated collateral. 
Each pool has a unique ID and can be managed independently. 
Pools can be either permissioned or trustless, as specified during the creation step.

- Permissioned Pools:   Only the pool manager can decide whether minting and burning should be permissioned or open to all, and can enable and disable mint and burn independently. 
                        Those privileges can be dropped entirely.
- Trustless Pools: Any participant can mint and burn bonded currencies without requiring special permissions.

## Storage Items

- `Pools`: Stores details of each pool, including its bonding curve, collateral type, and current state.

## Events

- `LockSet`: Emitted when a lock is set on a pool.
- `Unlocked`: Emitted when a pool is unlocked.
- `PoolCreated`: Emitted when a new pool is created.
- `RefundingStarted`: Emitted when a pool starts the refunding process.
- `DestructionStarted`: Emitted when a pool starts the destruction process.
- `RefundComplete`: Emitted when the refund process is complete.
- `Destroyed`: Emitted when a pool is fully destroyed.
- `ManagerUpdated`: Emitted when the manager of a pool is updated.

## Errors

- `PoolUnknown`: The specified pool ID is not registered.
- `IndexOutOfBounds`:   The specified currency index in the pool details is out of bounds. 
                        The target asset ID is provided by the index of the pool details.
- `NothingToRefund`: There is no collateral to refund or no remaining tokens to exchange.
- `NoPermission`: The user does not have permission to perform the operation.
- `PoolNotLive`: The pool is not available for use.
- `LivePool`: The pool cannot be destroyed because there are active accounts associated with it.
- `NotRefunding`: The operation can only be performed when the pool is in the refunding state.
- `CurrencyCount`: The number of currencies linked to a pool exceeds the limit.
- `InvalidInput`: The input provided is invalid.
- `Slippage`: The transaction would debit more than the user-specified maximum collateral (on mint) or would credit less than the user-specified minimum (on burn). 
- `Internal`: An internal error occurred. This error should never happen.

## Config Trait

The `Config` trait defines the configuration parameters and associated types required by the pallet. It includes the following associated types and constants:

### Associated Types

- `DepositCurrency`: The currency used for storage deposits.
- `PoolId`: The type representing the identifier for a pool.
- `CurveParameterType`:    The type representing the parameters for the curve.
                           Used for the actual bonding curve calculation and stored on chain.
- `CurveParameterInput`:   The type representing the unchecked input for the curve parameters. 
                           Negative curve coefficients make no sense and are therefore denied. 
                           Signed operations are still necessary for some transcendental functions in the bonding curves. 
                           Therefore, the input parameters are used to take unsigned parameters and translate them to the signed equivalent.
- `Fungibles`: Implementation of creating and managing new bonded fungibles.
- `Collaterals`: Implementation to withdraw and deposit collateral currencies.

### Constants

- `MaxDenomination`: The maximum denomination allowed.
- `DepositPerCurrency`: The deposit required per currency.
- `BaseDeposit`: The base deposit amount to create a new pool.
- `MaxStringInputLength`: The maximum length of strings for the currencies symbol and name.
- `MaxCurrenciesPerPool`: The maximum number of currencies allowed per pool.

### Origins 

- `DefaultOrigin`: The default origin for operations, which require no special privileges.
- `PoolCreateOrigin`: The origin required to create a pool.
- `ForceOrigin`: The origin for privileged operations.

### Hooks

`NextAssetIds`:  Takes care of producing asset ids to be used in creating new bonded currencies during initialization of a new pool. The hook must ensure that all asset ids returned are not yet in use.

### Feature guided 

- `BenchmarkHelper`: Helper type for benchmarking to calculate asset ids for collateral and bonded currencies.

## Life Cycle of a Pool

1. An __Owner__ initializes a new pool by calling `create_pool`, specifying the bonding curve, metadata of the new currencies, which currency to use as a collateral, and whether currencies should be `transferable`. The __Owner__ will be the new pool’s first __Manager__. 
   - A storage deposit will be paid by the __Owner__ in addition to transaction fees.
2. Optional: __Owner__ makes manager-level changes to the new pool or associated assets, such as setting locks on mint/burn functionality, or changing the asset management team.
3. Optional: __Owner__ re-assigns or un-assigns management privileges. In the second case, no further management-level changes can be made, including the initialization of the refund mechanism.
4. __Traders__ buy into one of the associated currencies by calling `mint_into`.
   - If the __Owner__ has flagged the pool’s associated assets as `transferable` upon creation, __Traders__ may transfer their holdings to other accounts, enabling, for example, secondary markets. This is done by interacting with the assets pallet directly via its extrinsics (`transfer`, `transfer_keep_alive`, `approve_transfer`, etc).
5. __Traders__ sell their holdings of any of the associated currencies by calling `burn_into`.
6. Optional: __Manager__ can end trading of the associated assets and distribute all collateral collected among holders. To do so, they call `start_refund`. All minting and burning is halted.
   - This is followed by calling `refund_account` for each asset and account holding funds. This call can be called by anyone for any account.
7. When no collateral remains in the pool _OR_ when all linked assets have a total supply of 0 (all funds burnt), the pool __Owner__ or __Manager__ can initialize the destruction of the pool by calling start_destroy. All minting, burning, transferring, and refunding is halted.
   - If the pool __Manager__ has been unassigned, the pool cannot be drained forcefully by the __Owner__. Either all __Traders__ need to be convinced to burn their holdings, or an appeal must be made to the configured force origin (typically blockchain governance) to call `force_start_refund` (enabling collateral distribution as in 6.) or `force_start_destroy` (forcefully destroying the pool despite value still being locked in the pool).
8. If any balance remains for some account on any asset associated with the pool, these accounts have to be destroyed by calling the asset pallet’s `destroy_accounts` extrinsic for that asset.
   - This can be called by any account.
   - This scenario may occur either because destruction was initiated via `force_start_destroy`, or in rare cases where during refunding there is less collateral than bonded currency and not all accounts receive a share of collateral, leaving the collateral exhausted before all accounts have been refunded.
9. If any approvals have been created for some account by __Traders__ (via calling `approve_transfer` on the assets pallet) on any asset associated with the pool, these approvals have to be destroyed by calling the asset pallet’s `destroy_approvals` extrinsic for that asset.
   - This can be called by any account.
10. Once no accounts or approvals remain on any associated asset, the pool record can be purged from the blockchain state by calling `finish_destroy`.
   - The storage deposit and (if any) residual collateral will be transferred to the __Owner__.
   - This can be called by any account.

## Functions

### Public Functions

#### Permissionless

Can be called by any regular origin (`DefaultOrigin`/`PoolCreateOrigin`; subject to runtime configuration).

- `create_pool`:  Creates a new pool with the specified bonding curve, collateral type, and currencies. 
                  Additional configuration determines the denomination of bonded coins, whether they are 
                  transferable or not, and whether their asset management teams may be re-assigned.
                  These settings cannot be changed after pool creation.
                  The selected denomination is used not only as metadata but also to scale down the existing supply or the amount specified in a mint or burn operation.
                  A deposit is taken from the caller, which is returned once the pool is destroyed. 
- `mint_into`:    Mints new tokens by locking up collateral. 
                  In the mint_into operation the beneficiary must be specified. 
                  The collateral is taken from the caller. 
- `burn_into`:    Burns tokens to release collateral. 
                  In the burn_into operation the beneficiary must be specified. 
                  The funds are burned from the caller.
- `refund_account`:  Can only be called on pools in 'Refunding' state. Refunds collateral to a specific account. 
                     The amount of refunded collateral is determined by the owned bonded currency.
- `finish_destroy`:  Can only be called on pools in 'Destroying' state. Completes the destruction process for a pool. 
                     Refunds any taken deposits.

#### Permissioned

Can only be called by a pool's manager origin.

- `reset_team`:   Resets the managing team of a pool. 
                  Only the admin and the freezer can be updated in this operation.
                  This will always fail for pools where the `allow_reset_team` setting is `false`.
- `reset_manager`: Resets the manager of a pool. 
- `set_lock`:  Sets a lock on a pool. 
               Locks specify who is able to mint and burn a bonded currency. 
               After applying the lock, the pool becomes permissioned, and only the manager is able to mint or burn bonded currencies.
- `unlock`:    Unlocks the pool. 
- `start_refund`:    Starts the refund process for a pool. 
- `start_destroy`:   Starts the destruction process for a pool. 
                     Both the manager and the owner are able to start the destroy process. 
                     If accounts with bonded currencies still exist, this operation will fail.

#### Privileged

Can only be called by the force origin (`ForceOrigin`; subject to runtime configuration).

- `force_start_refund`:    Forces the start of the refund process for a pool. 
                           Requires force privileges. 
- `force_start_destroy`:   Forces the start of the destruction process for a pool. 
                           Requires force privileges. 

## Permissions Structure & (De-)Centralization

In its current form, the pallet allows three different levels of central control over a pool:

#### Full manager authority

Upon creation, every pool has the `manager` field set to the account creating the pool (the __Owner__). The __Owner__ can also choose to set the `allow_reset_team` flag on the pool to `true` during creation (this cannot be changed afterwards\!). In this configuration, the __Owner__/__Manager__ has near-total authority over the pool’s lifecycle and its associated assets, allowing them to:

- Transfer pool management privileges to another account  
- Impose and lift a Lock on the pool, halting/resuming all minting and/or burning  
- Mint and burn bonded tokens even though mint/burn is locked for all others  
- Initiate the refund process, halting bonded token mints and burns  
  - Since the `refund_account` extrinsic applies for refunds proportionally across all bonded currencies, assuming equal value, the refund process could potentially be abused to perform market manipulations.  
- Assign or modify the currency management team, and thereby exercise control over bonded currency funds held by other wallets, including:  
  - Freeze or unfreeze funds  
  - Force-transfer a wallet’s balance to another account  
  - Slash/destroy a wallet’s balance

For these reasons users are advised to exercise caution when a pool’s `allow_reset_team` is set to `true`. **This is true even if the `manager` field is set to `None`**; even though the pool configuration is now immutable, asset management team changes may have been made prior to un-assigning the __Manager__, which means that individuals may still exercise privileged control over the bonded assets linked to the pool.

#### Restricted manager authority

Pools where the `allow_reset_team` is set to `false` do not allow changes to the asset management team, even if a pool `manager` exists. Therefore, the __Manager__ has a much more restricted set of privileges, limited to:

- Transferring pool management privileges to another account  
- Imposing and lifting a Lock on the pool, halting/resuming all minting and/or burning  
- Minting and burning bonded tokens even though mint/burn is locked for all others  
- Initiating the refund process, halting bonded token mints and burns

Still, these permissions can potentially allow a __Manager__ to:

- Trap user’s funds by imposing burn locks after they minted coins in exchange for collateral  
- Perform market manipulations via imposing locks or via initiating refund when token distribution and price conditions are in their favor

#### Unmanaged / unprivileged pools

While the `manager` field is set to the pool creator at the time of creation, a __Manager__ can drop all privileges by calling the `reset_manager` transaction with a `None` argument, resulting in the `manager` field being unset.  
Pools with no __Manager__ and the `allow_reset_team` set to `false` can be considered unprivileged, as:

- Their current configuration is immutable  
- The pool __Owner__ has given up all relevant privileges and control over the pool’s bonded assets

While there is still a single transaction that only the __Owner__ is privileged to make (`start_destroy`), this does not significantly impact bonded token economics, as it can only be called once all users have burnt all their holdings of bonded tokens linked to this pool.  
Unprivileged pools where bonded currency supplies are non-zero thus cannot be purged unless by force origin intervention.

### Force Origin

The pallet implements several transactions, prefixed with `force_*`, that are restricted to use by a ‘force’-origin. This origin can be configured differently in each chain/runtime this pallet is integrated into, but is assumed to be the blockchain’s governing body which can also decide on runtime upgrades and thus has unlimited control over blockchain state and state transition functions.  
The transactions `force_start_refund` & `force_start_destroy` are designed to allow the forced closure of pools, which may be necessary, e.g., because of inactive accounts preventing purge of a pool which is otherwise unused, or because the pool has been found to be involved in illegitimate activities.  

[bonding-curve]: ./src/curves/mod.rs
[pool-details]: ./src/types.rs
[fixed-point]: https://github.com/encointer/substrate-fixed
[lmsr]: https://mason.gmu.edu/~rhanson/mktscore.pdf
[arithmetic-error]: https://github.com/paritytech/substrate/blob/master/primitives/arithmetic/src/lib.rs#L64
