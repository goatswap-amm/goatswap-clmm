import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import {
  AddressUtil,
  FEE_RATE_DENOMINATOR,
  GoatswapContext,
  Network,
  buildGoatswapClient,
  getProgramConfigs,
} from "../sdk/src";
import { invariant } from "../sdk/src/common/utils";

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const network = Network.Devnet;
  const ctx = GoatswapContext.withProvider(
    provider,
    AddressUtil.toPubKey(getProgramConfigs[network].programId),
    undefined,
    undefined,
    { userDefaultSendOptions: { skipPreflight: true } }
  );

  const client = buildGoatswapClient(ctx);

  const poolAddress = "2V5aEzqTAtPP6XtSWrUB7V6MWrHmeV2zgZNGeZQi3JRM";

  const pool = await client.getPool(poolAddress);

  const authority = ctx.wallet.publicKey;
  const lpFeeRate = 3000;

  const isOwnerOrAdmin =
    pool.getData().poolCreator.equals(authority) ||
    authority.equals(new PublicKey(getProgramConfigs[network].admin));

  invariant(isOwnerOrAdmin, "Only the pool owner or admin can update lp fee");

  const tradeFeeRate = pool.getConfig().tradeFeeRate;
  invariant(
    lpFeeRate >= 0 &&
      tradeFeeRate.add(new anchor.BN(lpFeeRate)).lte(FEE_RATE_DENOMINATOR),
    `LP fee must less than or equal to ${FEE_RATE_DENOMINATOR.sub(tradeFeeRate).toString()}`
  );

  console.log("Sending transaction...");

  const { tx } = await pool.updateLpFee(authority, lpFeeRate, network);

  const id = await tx.buildAndExecute(undefined, {
    skipPreflight: true,
    maxRetries: 0,
  });

  console.log(`Update lp fee done: ${id}`);
}

main();
