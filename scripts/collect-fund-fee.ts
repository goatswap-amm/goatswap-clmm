import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";

import {
  AddressUtil,
  GoatswapContext,
  Network,
  ZERO,
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
  const poolConfig = pool.getConfig();
  console.log(
    `Fee info: fundFeesToken0=${poolData.fundFeesToken0.toString()}, fundFeesToken1=${poolData.fundFeesToken1.toString()}}`
  );

  const owner = ctx.wallet.publicKey;

  invariant(
    owner.equals(poolConfig.fundOwner) ||
      owner.equals(new PublicKey(getProgramConfigs[Network.Devnet].admin)),
    "Only the fund owner or admin can collect fund fee"
  );

  invariant(
    poolData.fundFeesToken0.gt(ZERO) || poolData.fundFeesToken1.gt(ZERO),
    "No pending fund fees to collect"
  );

  console.log("Sending transaction...");

  const { tx } = await pool.collectFundFee(owner, Network.Devnet);

  const id = await tx.buildAndExecute(undefined, {
    skipPreflight: true,
    maxRetries: 0,
  });

  console.log(`Collect fund fee done: ${id}`);
}

main();
