import * as anchor from "@coral-xyz/anchor";
import { NATIVE_MINT } from "@solana/spl-token";

import {
  AddressUtil,
  GoatswapContext,
  Network,
  Percentage,
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

  const poolAddress = "2Yx8MP38gWLDnMWsP9qa21pxi7v2rY5okWvuRXuqVyha";

  const pool = await client.getPool(poolAddress);

  console.log(
    pool.reserve0().toString(),
    pool.reserve1().toString(),
    pool.getData().lpSupply.toString()
  );

  const quote = await pool.getWithdrawQuote(
    new anchor.BN(315227766016),
    new Percentage(new anchor.BN(10), new anchor.BN(100))
  );

  console.log(
    `Quote: amount0=${quote.minAmount0Out.toString()}, amount1=${quote.minAmount1Out.toString()}, Lp amount=${quote.withdrawAmount.toString()}`
  );

  const { tx } = await pool.withdraw(ctx.wallet.publicKey, quote);

  console.log("Sending transaction...");

  const id = await tx.buildAndExecute(undefined, {
    skipPreflight: true,
    maxRetries: 0,
  });

  console.log(`Withdraw done: ${id}`);
}

main();
