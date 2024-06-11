import * as anchor from "@coral-xyz/anchor";

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

  const poolData = pool.getData();
  console.log(
    `Tax info: taxAmount0=${poolData.taxAmount0.toString()}, taxAmount1=${poolData.taxAmount1.toString()}}`
  );

  const owner = ctx.wallet.publicKey;

  invariant(
    poolData.poolCreator.equals(owner),
    "You are not the owner of the pool"
  );

  invariant(
    poolData.taxAmount0.gt(new anchor.BN(0)) ||
      poolData.taxAmount1.gt(new anchor.BN(0)),
    "No pending tax"
  );

  console.log("Sending transaction...");

  const { tx } = await pool.collectTax(owner);

  const id = await tx.buildAndExecute(undefined, {
    skipPreflight: true,
    maxRetries: 0,
  });

  console.log(`Collect done: ${id}`);
}

main();
