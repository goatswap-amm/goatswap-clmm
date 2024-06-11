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
  const authority = ctx.wallet.publicKey;

  invariant(
    pool.getData().taxAuthority.equals(authority),
    "You are not authority of this pool"
  );

  console.log("Sending transaction...");

  const { tx } = await pool.updateTax(authority, {
    inTaxRate: 50_000,
    outTaxRate: 100_000,
    taxUseToken0: true,
  });

  const id = await tx.buildAndExecute(undefined, {
    skipPreflight: true,
    maxRetries: 0,
  });

  console.log(`Update tax done: ${id}`);
}

main();
