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
    `Fee info: protocolFeesToken0=${poolData.protocolFeesToken0.toString()}, protocolFeesToken1=${poolData.protocolFeesToken1.toString()}}`
  );

  const owner = ctx.wallet.publicKey;

  invariant(
    owner.equals(poolConfig.protocolOwner) ||
      owner.equals(new PublicKey(getProgramConfigs[Network.Devnet].admin)),
    "Only the protocol owner or admin can collect protocol fee"
  );

  invariant(
    poolData.protocolFeesToken0.gt(ZERO) ||
      poolData.protocolFeesToken1.gt(ZERO),
    "No pending protocol fees to collect"
  );

  console.log("Sending transaction...");

  const { tx } = await pool.collectProtocolFee(owner, Network.Devnet);

  const id = await tx.buildAndExecute(undefined, {
    skipPreflight: true,
    maxRetries: 0,
  });

  console.log(`Collect protocol fee done: ${id}`);
}

main();
