import * as sdk from "sdk-core"

let a  = sdk.calculate_key_id("asdf")
let b  = sdk.is_valid_web3_name("asdfasdf")
let c  = sdk.is_valid_web3_name("a")
console.log(a, b,c)
