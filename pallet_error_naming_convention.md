# Pallet Error Naming Conventions

1) Use capitalized camel case in the variant names. For example, instead of using "NOT_FOUND" you should use "NotFound".

2) Avoid using the word "error" as a suffix for the variant names. For example, instead of using "NotFoundError" you should use "NotFound". It's clear from the caller's context that this is an error.

3) Avoid using the pallet name in the variant names. For example instead of "Web3NameNotFound" you should use "NotFound". 

4) Try to take words from the vocabulary already defined in the code base. For example instead of introducing a new variant "NotExisting" you should use, once again, "NotFound". Common vocabulary includes: NotFound, NotAuthorized, AlreadyExists, MaxXYZExceeded.

5) Use descriptive and concise names for the variants. Avoid using abbreviations or acronyms unless they are widely recognized and understood by other developers who may be working on the codebase.