import * as anchor from "@coral-xyz/anchor";

import {
  AddressUtil,
  GoatswapContext,
  Network,
  PoolStatus,
  buildGoatswapClient,
  getProgramConfigs,
} from "../sdk/src";

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

  const admin = ctx.wallet.publicKey;

  console.log("Sending transaction...");

  const { tx } = await pool.updateStatus(admin, PoolStatus.DisableSwap);

  const id = await tx.buildAndExecute(undefined, {
    skipPreflight: true,
    maxRetries: 0,
  });

  console.log(`Disable swap done: ${id}`);
}

main();
