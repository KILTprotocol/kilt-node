## fixes KILTProtocol/ticket#312
Please include a summary of the changes provided with this pull request and which issue has been fixed.
Please also provide some context if necessary.

## How to test:
Please provide a brief step-by-step instruction.
If necessary provide information about dependencies (specific configuration, branches, database dumps, etc.)

- Step 1
- Step 2
- etc.

### Custom types

This PR introduces new custom JS-types which are required for compatibility with [our SDK](https://github.com/KILTprotocol/sdk-js) and the [Polkadot Apps](https://polkadot.js.org/apps/#/extrinsics). Please use the following types to test the code with the Polkadot Apps:

<details>
  <summary>JS-Types</summary>

  ```json
  {}
  ```
</details>

## Checklist:

- [ ] I have verified that the code works
  - [ ] No panics! (checked arithmetic ops, no indexing `array[3]` use `get(3)`, ...)
- [ ] I have verified that the code is easy to understand
  - [ ] If not, I have left a well-balanced amount of inline comments
- [ ] This PR does not introduce new custom types
  - [ ] If not, I have opened a companion PR with the type changes in the [KILT types-definitions repository](https://github.com/KILTprotocol/type-definitions/pulls)
- [ ] I have [left the code in a better state](https://deviq.com/principles/boy-scout-rule)
- [ ] I have documented the changes (where applicable)
