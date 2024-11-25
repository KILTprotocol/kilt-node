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
The current system implements the [LMSR][https://mason.gmu.edu/~rhanson/mktscore.pdf], square root, and polynomial bonding curves.

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

- Permissioned Pools: Only the pool manager has the authority to mint and burn bonded currencies. The manager can be updated, or the privileges can be dropped entirely.
- Trustless Pools: Any participant can mint and burn bonded currencies without requiring special permissions.

## Storage Items

- `Pools`: Stores details of each pool, including its bonding curve, collateral type, and current state.
- `NextAssetId`:  Tracks the next available asset ID for new currencies. 
                  If the max value is reached, an [ArithmeticError][arithmetic-error] will be thrown.

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
- `Slippage`: The cost exceeds the maximum allowed slippage.
- `Internal`: An internal error occurred. This error should never happen.

## Config Trait

The `Config` trait defines the configuration parameters and associated types required by the pallet. It includes the following associated types and constants:

### Associated Types

- `DepositCurrency`: The currency used for storage deposits.
- `PoolId`: The type representing the identifier for a pool.
- `AssetId`: The type representing the identifier for an asset.
- `CurveParameterType`:    The type representing the parameters for the curve.
                           Used for the actual bonding curve calculation and stored on chain.
- `CurveParameterInput`:   The type representing the unchecked input for the curve parameters. 
                           Negative curve coefficients make no sense and are therefore denied. 
                           Signed operations are still necessary for some transcendental functions in the bonding curves. 
                           Therefore, the input parameters are used to take unsigned parameters and translate them to the signed equivalent.
- `Fungibles`: Implementation of creating and managing new bonded fungibles.
- `CollateralCurrencies`: Implementation to withdraw and deposit collateral currencies.

### Constants

- `MaxDenomination`: The maximum denomination allowed.
- `DepositPerCurrency`: The deposit required per currency.
- `BaseDeposit`: The base deposit amount to create a new pool.
- `MaxStringLength`: The maximum length of strings for the currencies symbol and name.
- `MaxCurrencies`: The maximum number of currencies allowed per pool.

### Origins 

- `DefaultOrigin`: The default origin for operations, which require no special privileges.
- `PoolCreateOrigin`: The origin required to create a pool.
- `ForceOrigin`: The origin for privileged operations.

### Feature guided 

- `BenchmarkHelper`: Helper type for benchmarking to calculate asset ids for collateral and bonded currencies.

## Life Cycle of a Pool

1. **Creation**:
   - A pool is created using the `create_pool` function.
   - The manager specifies the bonding curve, collateral type, currencies, and whether the bonded coins are transferable.
   - A deposit is taken from the caller, which can be reclaimed once the pool is destroyed.

2. **Refund Process**:
   - The refund process can be started by the manager using the `start_refund` function.
   - The refund process can be forced to start using the `force_start_refund` function, which requires force privileges.
   - Collateral can be refunded to a specific account using the `refund_account` function, based on the owned bonded currency.

3. **Destruction**:
   - The destruction process can be started by the manager using the `start_destroy` function. This operation will fail if accounts with bonded currencies still exist.
   - The destruction process can be forced to start using the `force_start_destroy` function, which requires force privileges. This operation will fail if there are still accounts with bonded currencies.
   - The destruction process is completed using the `finish_destroy` function, which refunds any taken deposits.

## Functions

### Public Functions

- `create_pool`:  Creates a new pool with the specified bonding curve, collateral type, and currencies. 
                  During the pool creation step, the manager must specify whether the bonded coins are transferable or not. 
                  This flag can not be changed later on. 
                  The selected denomination is used not only as metadata but also to scale down the existing supply or the amount specified in a mint or burn operation. 
                  A deposit is taken from the caller, which is returned once the pool is getting destroyed. 
- `reset_team`:   Resets the managing team of a pool. 
                  Only the admin and the freezer can be updated in this operation. 
- `reset_manager`: Resets the manager of a pool. 
- `set_lock`:  Sets a lock on a pool. 
               Locks specify who is able to mint and burn a bonded currency. 
               After applying the lock, the pool becomes permissioned, and only the manager is able to mint or burn bonded currencies.
- `unlock`:    Unlocks the pool. 
               Can only be called by the manager.
- `mint_into`:    Mints new tokens by locking up collateral. 
                  In the mint_into operation the beneficiary must be specified. 
                  The collateral is taken from the caller. 
- `burn_into`:    Burns tokens to release collateral. 
                  In the burn_into operation the beneficiary must be specified. 
                  The funds are burned from the caller.
- `start_refund`:    Starts the refund process for a pool. 
                     Only the manager is able to start the refund process. 
- `force_start_refund`:    Forces the start of the refund process for a pool. 
                           Requires force privileges. 
- `refund_account`:  Refunds collateral to a specific account. 
                     The amount of refunded collateral is determined by the owned bonded currency.
- `start_destroy`:   Starts the destruction process for a pool. 
                     Only the manager is able to start the destroy process. 
                     If accounts with bonded currencies still exist, this operation will fail.
- `force_start_destroy`:   Forces the start of the destruction process for a pool. 
                           Requires force privileges. 
                           This operation will fail if there are still accounts with bonded currencies.
- `finish_destroy`:  Completes the destruction process for a pool. 
                     Refunds any taken deposits.

[bonding-curve]: ./src/curves/mod.rs
[pool-details]: ./src/types.rs
[fixed-point]: https://github.com/encointer/substrate-fixed
[lmsr]: https://mason.gmu.edu/~rhanson/mktscore.pdf
[arithmetic-error]: https://github.com/paritytech/substrate/blob/master/primitives/arithmetic/src/lib.rs#L64
