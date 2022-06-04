/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet'
import * as web3 from '@solana/web3.js'

/**
 * @category Instructions
 * @category UpdateTokenData
 * @category generated
 */
export type UpdateTokenDataInstructionArgs = {
  name: string
  discount: beet.bignum
  rewardUsdcToken: beet.bignum
}
/**
 * @category Instructions
 * @category UpdateTokenData
 * @category generated
 */
export const updateTokenDataStruct = new beet.FixableBeetArgsStruct<
  UpdateTokenDataInstructionArgs & {
    instructionDiscriminator: number[] /* size: 8 */
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['name', beet.utf8String],
    ['discount', beet.u64],
    ['rewardUsdcToken', beet.u64],
  ],
  'UpdateTokenDataInstructionArgs'
)
/**
 * Accounts required by the _updateTokenData_ instruction
 *
 * @property [_writable_] tokenData
 * @property [**signer**] user
 * @category Instructions
 * @category UpdateTokenData
 * @category generated
 */
export type UpdateTokenDataInstructionAccounts = {
  tokenData: web3.PublicKey
  user: web3.PublicKey
}

export const updateTokenDataInstructionDiscriminator = [
  226, 213, 133, 247, 203, 107, 101, 108,
]

/**
 * Creates a _UpdateTokenData_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category UpdateTokenData
 * @category generated
 */
export function createUpdateTokenDataInstruction(
  accounts: UpdateTokenDataInstructionAccounts,
  args: UpdateTokenDataInstructionArgs
) {
  const { tokenData, user } = accounts

  const [data] = updateTokenDataStruct.serialize({
    instructionDiscriminator: updateTokenDataInstructionDiscriminator,
    ...args,
  })
  const keys: web3.AccountMeta[] = [
    {
      pubkey: tokenData,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: user,
      isWritable: false,
      isSigner: true,
    },
  ]

  const ix = new web3.TransactionInstruction({
    programId: new web3.PublicKey(
      'Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS'
    ),
    keys,
    data,
  })
  return ix
}
