import {
  PublicKey,
  Connection,
  Keypair,
  SystemProgram,
  AddressLookupTableProgram,
  TransactionInstruction,
  AddressLookupTableAccount,
  TransactionMessage,
  VersionedTransaction,
} from "@solana/web3.js";

import {
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { sleep } from "../sdk/src";

export const MEMO_PROGRAM_ID = new PublicKey(
  "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"
);
export const RENT_PROGRAM_ID = new PublicKey(
  "SysvarRent111111111111111111111111111111111"
);

export async function createAndSendV0Tx(
  connection: Connection,
  signers: Keypair[],
  txInstructions: TransactionInstruction[],
  addressLookupTableAccount?: AddressLookupTableAccount
) {
  // Step 1 - Fetch Latest Blockhash
  let latestBlockhash = await connection.getLatestBlockhash("finalized");
  // Step 2 - Generate Transaction Message
  const messageV0 = new TransactionMessage({
    payerKey: signers[0].publicKey,
    recentBlockhash: latestBlockhash.blockhash,
    instructions: txInstructions,
  }).compileToV0Message(
    addressLookupTableAccount ? [addressLookupTableAccount] : undefined
  );
  const transaction = new VersionedTransaction(messageV0);
  // Step 3 - Sign your transaction with the required `Signers`
  transaction.sign(signers);

  // Step 4 - Send our v0 transaction to the cluster
  const txid = await connection.sendTransaction(transaction, {
    skipPreflight: true,
    maxRetries: 5,
  });
  // Step 5 - Confirm Transaction
  const confirmation = await connection.confirmTransaction({
    signature: txid,
    blockhash: latestBlockhash.blockhash,
    lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
  });
  if (confirmation.value.err) {
    console.log(confirmation.value.err);
    throw new Error(
      `   ❌ - Transaction not confirmed: ${confirmation.value.err.toString()}`
    );
  }
  return txid;
}

export async function createLookupTable(
  connection: Connection,
  signer: Keypair,
  addresses?: PublicKey[]
) {
  const allAddresses = [
    SystemProgram.programId,
    RENT_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    MEMO_PROGRAM_ID,
  ];
  if (addresses) {
    allAddresses.push(...addresses);
  }
  const set = new Set<PublicKey>();
  allAddresses.forEach((a) => set.add(a));

  const [lookupTableInst, lookupTableAddress] =
    AddressLookupTableProgram.createLookupTable({
      authority: signer.publicKey,
      payer: signer.publicKey,
      recentSlot: await connection.getSlot(),
    });

  await createAndSendV0Tx(connection, [signer], [lookupTableInst]);

  await sleep(3000);

  const addAddressesInstruction = AddressLookupTableProgram.extendLookupTable({
    payer: signer.publicKey,
    authority: signer.publicKey,
    lookupTable: lookupTableAddress,
    addresses: Array.from(set),
  });

  await createAndSendV0Tx(connection, [signer], [addAddressesInstruction]);

  await sleep(3000);

  return lookupTableAddress;
}
