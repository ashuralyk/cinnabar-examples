# dao-certificate-type

Used as a certificate bonding with DAO cell

Verification Tree:
- Root
  - Deposit
  - Mint
  - Withdraw

Detail:
1. `Root`: parse operation mode and dispatch into particular leaf part
2. `Deposit`: check wether created a dao-certificate cell with type-id in DAO deposit operation
3. `Mint`: check wether created a spore dob cell that contains dao capacity and deposit block header while linking to dao-certificate cell
4. `Withdraw`: empty check

*This contract was bootstrapped with [ckb-script-templates].*

[ckb-script-templates]: https://github.com/cryptape/ckb-script-templates
