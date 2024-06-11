import * as anchor from "@coral-xyz/anchor";
import { PublicKey, Keypair } from "@solana/web3.js";

import {
  AddressUtil,
  GoatswapContext,
  Network,
  buildGoatswapClient,
  getProgramConfigs,
} from "../sdk/src";
import { invariant } from "../sdk/src/common/utils";

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const ctx = GoatswapContext.withProvider(
    provider,
    AddressUtil.toPubKey(getProgramConfigs[Network.Devnet].programId),
    undefined,
    undefined,
    { userDefaultSendOptions: { skipPreflight: true } }
  );

  const client = buildGoatswapClient(ctx);

  const poolAddress = "9qYxv1H4M5gcNqfQ2nEVwNRfgJ6wegbLwGK6Fjjk1u2C";
  const pool = await client.getPool(poolAddress);

  // must be admin
  const authority = ctx.wallet.publicKey;
  const newOwner = Keypair.generate().publicKey;

  invariant(
    authority.equals(new PublicKey(getProgramConfigs[Network.Devnet].admin)),
    "Only the admin can transfer pool owner"
  );

  console.log("Sending transaction...");

  const { tx } = await pool.transferPoolOwner(
    authority,
    newOwner,
    Network.Devnet
  );

  const id = await tx.buildAndExecute(undefined, {
    skipPreflight: true,
    maxRetries: 0,
  });

  console.log(`Transfer pool owner done: ${id}`);
}

main();
