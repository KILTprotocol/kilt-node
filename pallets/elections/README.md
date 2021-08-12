
Idea: Use elections-phragmen as template with more simple election algorithm and only allow delegated voting for council candidates.

## Extrinsics

Format: `extrinsic`: required change
* `vote`: 1 candidate per vote instead of a vec of candidates
* `submit_candidacy`: Can probably stay "as is"
* `renounce_candidacy`: Can probably stay "as is"
* `remove_member`: If the member did not have a replacement, take the top RunnerUp?
* `clean_defunct_voters`: Can probably stay "as is"