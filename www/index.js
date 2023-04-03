import * as sdk from "sdk-core"

let a =  sdk.DidVerificationKeyRelationship.CapabilityDelegation
let d = sdk.derive_keys("3E00")

console.log(d)
